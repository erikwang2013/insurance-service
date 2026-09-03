//! 认证服务：注册 / 登录 / 微信登录 stub + JWT 签发（对齐 backend-architecture.md §7）
//!
//! 说明：阶段 0 未接入 bee_orm，注册/登录的持久化操作用 `todo!()` 占位，但
//! 密码哈希（argon2）、JWT 签发/校验、令牌模型为完整可编译实现。

use argon2::Argon2;
use argon2::password_hash::{
    PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use mysql_async::prelude::Queryable;
use mysql_async::Value;

use crate::config::JwtConfig;
use crate::crypto::CryptoService;
use crate::db::{db_error, Db};
use crate::error::{AppError, Result};
use crate::middleware::auth::{JwtService, Role};
use crate::models::user::User;
use crate::providers::wechat::{WechatClient, WechatSession};
use crate::utils::validator::check_phone;

/// 注册请求
#[derive(Debug, Deserialize)]
pub struct RegisterReq {
    pub username: String,
    /// argon2 哈希后存 password_hash
    pub password: String,
    /// AES 加密存 phone_enc，另存 phone_masked
    pub phone: String,
}

/// 登录请求
#[derive(Debug, Deserialize)]
pub struct LoginReq {
    pub username: String,
    pub password: String,
}

/// 微信登录请求
#[derive(Debug, Deserialize)]
pub struct WechatLoginReq {
    /// wx.login 返回的 code
    pub code: String,
}

/// 令牌刷新请求
#[derive(Debug, Deserialize)]
pub struct RefreshReq {
    /// 登录/注册时签发的 refresh_token（另一枚同型 JWT，见 issue_tokens）
    pub refresh_token: String,
}

/// 修改密码请求
#[derive(Debug, Deserialize)]
pub struct ChangePasswordReq {
    pub old_password: String,
    pub new_password: String,
}

/// 换绑手机请求
#[derive(Debug, Deserialize)]
pub struct BindPhoneReq {
    /// 登录密码校验（安全操作需验明正身）
    pub password: String,
    pub new_phone: String,
}

/// 令牌对（Access + Refresh）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// 登录返回（含令牌与用户摘要）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResult {
    pub tokens: TokenPair,
    pub user_id: i64,
    pub username: String,
    pub role: String,
}

/// 微信 code2session 会话获取抽象。
///
/// 生产实现为 `WechatClient`（凭据齐全即真实调用、未配置即业务错误降级）；
/// 抽象成 trait 仅为集成测试注入 fake（测试环境不配置、也不允许触达真实
/// 微信接口，见任务 C1 测试约束），AuthService 以 `Box<dyn>` 持有。
#[async_trait]
pub trait SessionProvider: Send + Sync {
    /// 以登录 code 换取微信会话（成功含 openid；失败按真实语义返回错误）
    async fn code2session(&self, code: &str) -> Result<WechatSession>;
}

#[async_trait]
impl SessionProvider for WechatClient {
    async fn code2session(&self, code: &str) -> Result<WechatSession> {
        WechatClient::code2session(self, code).await
    }
}

/// 认证服务
pub struct AuthService {
    jwt: JwtService,
    crypto: CryptoService,
    db: Db,
    /// 微信 code2session 提供者（生产 = WechatClient，测试可注入 fake）
    wechat: Box<dyn SessionProvider>,
}

impl AuthService {
    pub fn new(jwt_cfg: JwtConfig, crypto: CryptoService, db: Db, wechat: WechatClient) -> Self {
        Self::new_with_provider(jwt_cfg, crypto, db, Box::new(wechat))
    }

    /// 测试注入入口：直接提供 `Box<dyn SessionProvider>`（见 `SessionProvider`）。
    /// 生产路径统一走 `AuthService::new` / `auth_service()`，四参签名保持不变。
    pub fn new_with_provider(
        jwt_cfg: JwtConfig,
        crypto: CryptoService,
        db: Db,
        wechat: Box<dyn SessionProvider>,
    ) -> Self {
        Self {
            jwt: JwtService::new(jwt_cfg),
            crypto,
            db,
            wechat,
        }
    }

    /// argon2 密码哈希
    pub fn hash_password(password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| AppError::Business(format!("密码哈希失败: {e}")))
    }

    /// 校验密码与哈希是否匹配
    pub fn verify_password(password: &str, hash: &str) -> bool {
        let Ok(parsed) = PasswordHash::new(hash) else {
            return false;
        };
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok()
    }

    /// 注册：事务内校验用户名唯一 + 落库，成功后签发双令牌。
    pub async fn register(&self, req: RegisterReq) -> Result<LoginResult> {
        // 1. 密码哈希
        let password_hash = Self::hash_password(&req.password)?;
        // 2. 手机号加密 + 脱敏
        let phone_enc = self.crypto.encrypt_str(&req.phone)?;
        let phone_masked = crate::crypto::Masker::phone(&req.phone);

        // 3. 事务闭环：唯一性校验 + 插入原子完成；冲突或失败自动回滚
        let username = req.username.clone();
        let user_id = self
            .db
            .with_tx(|tx| {
                Box::pin(async move {
                    let exists: Option<User> = tx
                        .exec_first(
                            "SELECT * FROM users WHERE username = ? AND deleted_at IS NULL LIMIT 1",
                            vec![username.clone()],
                        )
                        .await
                        .map_err(db_error)?;
                    if exists.is_some() {
                        return Err(AppError::business("用户名已存在"));
                    }
                    let params: Vec<Value> = vec![
                        username.into(),
                        password_hash.into(),
                        phone_enc.into(),
                        phone_masked.into(),
                        User::ROLE_USER.to_string().into(),
                        User::STATUS_ACTIVE.to_string().into(),
                    ];
                    tx.exec_drop(
                        "INSERT INTO users \
                         (username, password_hash, phone_enc, phone_masked, role, status, \
                          created_at, updated_at) \
                         VALUES (?, ?, ?, ?, ?, ?, NOW(), NOW())",
                        params,
                    )
                    .await
                    .map_err(db_error)?;
                    Ok(tx.last_insert_id().unwrap_or_default() as i64)
                })
            })
            .await?;

        // 4. 签发双令牌（新行 token_version 恒为库默认 0）
        self.issue_tokens(user_id, &req.username, Role::User, 0)
    }

    /// 登录：按用户名查用户 → 校验密码 → 更新最后登录时间 → 签发令牌。
    pub async fn login(&self, req: LoginReq) -> Result<LoginResult> {
        // 1. 按 username 查用户
        let user: Option<User> = self
            .db
            .query_one(
                "SELECT * FROM users WHERE username = ? AND deleted_at IS NULL LIMIT 1",
                vec![req.username.clone()],
            )
            .await?;
        let user = user.ok_or_else(|| AppError::business("用户名或密码错误"))?;

        // 2. 校验密码
        if !Self::verify_password(&req.password, &user.password_hash) {
            return Err(AppError::business("用户名或密码错误"));
        }

        // 3. 更新最后登录时间（非关键路径，失败仅记录）
        let _ = self
            .db
            .exec_drop(
                "UPDATE users SET last_login_at = NOW() WHERE id = ?",
                vec![user.id],
            )
            .await;

        // 4. 签发令牌（携带当前 token_version）
        let role = Role::from_str(&user.role).unwrap_or(Role::User);
        self.issue_tokens(user.id, &user.username, role, user.token_version)
    }

    /// 微信登录：code2session（未配置 → 「未配置」业务错误，天然降级不发请求）→
    /// 按 openid 查未删除用户：
    /// - 命中 → 直接签发双令牌（携带当前 token_version）；
    /// - 未命中 → 业务错误提示先登录绑定（本阶段不自动建号）。
    pub async fn wechat_login(&self, req: WechatLoginReq) -> Result<LoginResult> {
        let session = self.wechat.code2session(&req.code).await?;
        let user: Option<User> = self
            .db
            .query_one(
                "SELECT * FROM users WHERE openid = ? AND deleted_at IS NULL LIMIT 1",
                vec![session.openid],
            )
            .await?;
        let user = user.ok_or_else(|| {
            AppError::business("该微信未绑定账号，请先登录后在个人中心绑定")
        })?;
        let role = Role::from_str(&user.role).unwrap_or(Role::User);
        self.issue_tokens(user.id, &user.username, role, user.token_version)
    }

    /// 微信绑定：登录用户 + 小程序 code → code2session 换 openid →
    /// 写入 users.openid / unionid（覆盖式，重复绑定同一微信幂等）。
    pub async fn bind_wechat(&self, user_id: i64, code: &str) -> Result<()> {
        // 1. 登录用户须存在且未删除（不存在 → NotFound）
        let user: Option<User> = self
            .db
            .query_one(
                "SELECT * FROM users WHERE id = ? AND deleted_at IS NULL LIMIT 1",
                vec![user_id],
            )
            .await?;
        if user.is_none() {
            return Err(AppError::NotFound);
        }
        // 2. code2session（未配置 → 「未配置」业务错误，与微信登录同路降级）
        let session = self.wechat.code2session(code).await?;
        // 3. openid 唯一性预检：已被其他账号绑定 → 状态冲突（防 UNIQUE 撞库暴露 Db 错误）
        //    混合类型参数须显式标注 Vec<Value>（同 register/change_password 模式）
        let params: Vec<Value> = vec![session.openid.clone().into(), user_id.into()];
        let owner: Option<User> = self
            .db
            .query_one(
                "SELECT * FROM users WHERE openid = ? AND deleted_at IS NULL AND id <> ? LIMIT 1",
                params,
            )
            .await?;
        if owner.is_some() {
            return Err(AppError::state_conflict("该微信已绑定其他账号"));
        }
        // 4. 落库（同一账号重复绑定同 openid → 幂等覆盖成功）
        let params: Vec<Value> = vec![session.openid.into(), session.unionid.into(), user_id.into()];
        self.db
            .exec_drop(
                "UPDATE users SET openid = ?, unionid = ?, updated_at = NOW() WHERE id = ?",
                params,
            )
            .await?;
        Ok(())
    }

    /// 登出：token_version + 1，使此前签发的全部 refresh 立即失效
    /// （无状态 JWT 无法服务端删除，版本轮换即撤销，见 refresh 校验）。
    pub async fn logout(&self, user_id: i64) -> Result<()> {
        let affected = self
            .db
            .exec_drop(
                "UPDATE users SET token_version = token_version + 1, updated_at = NOW() \
                 WHERE id = ? AND deleted_at IS NULL",
                vec![user_id],
            )
            .await?;
        if affected == 0 {
            return Err(AppError::NotFound);
        }
        Ok(())
    }

    /// 令牌刷新：校验 refresh_token（与 access 同型 JWT，签名/过期/issuer 由
    /// `JwtService::verify_token` 把关，失败 → Unauthorized）→ 按载荷 `sub`
    /// 查用户，须存在、未删除、status='ACTIVE' **且载荷 token_version 与库中一致**
    /// （不一致 = 该令牌签发后发生过 logout/改密/换绑手机 → 已撤销）→
    /// 按库中最新的 username/role/token_version 重签双令牌。
    pub async fn refresh(&self, refresh_token: &str) -> Result<LoginResult> {
        let claims = self.jwt.verify_token(refresh_token)?;
        let user: Option<User> = self
            .db
            .query_one(
                "SELECT * FROM users WHERE id = ? AND deleted_at IS NULL LIMIT 1",
                vec![claims.sub],
            )
            .await?;
        let user = user.ok_or(AppError::Unauthorized)?;
        // 令牌版本落后于库 → 已撤销（旧令牌 serde 缺省 0，仅当用户从未轮换时放行）
        if claims.token_version != user.token_version {
            return Err(AppError::Unauthorized);
        }
        if user.status != User::STATUS_ACTIVE {
            return Err(AppError::business("账号已禁用或冻结，无法刷新令牌"));
        }
        let role = Role::from_str(&user.role).unwrap_or(Role::User);
        self.issue_tokens(user.id, &user.username, role, user.token_version)
    }

    /// 修改密码：旧密码校验通过后，以新哈希覆盖 password_hash。
    pub async fn change_password(
        &self,
        user_id: i64,
        old_password: &str,
        new_password: &str,
    ) -> Result<()> {
        // 1. 取当前用户（不存在/已软删 → NotFound）
        let user: Option<User> = self
            .db
            .query_one(
                "SELECT * FROM users WHERE id = ? AND deleted_at IS NULL LIMIT 1",
                vec![user_id],
            )
            .await?;
        let user = user.ok_or(AppError::NotFound)?;
        // 2. 旧密码校验（不通过 → 业务错误，不泄露哈希细节）
        if !Self::verify_password(old_password, &user.password_hash) {
            return Err(AppError::business("旧密码错误"));
        }
        // 3. 落库新哈希 + token_version +1（单行 UPDATE 原子完成；
        //    版本递增使改密前签发的全部 refresh 立即失效）
        let new_hash = Self::hash_password(new_password)?;
        let params: Vec<Value> = vec![new_hash.into(), user_id.into()];
        self.db
            .exec_drop(
                "UPDATE users SET password_hash = ?, token_version = token_version + 1, \
                 updated_at = NOW() WHERE id = ?",
                params,
            )
            .await?;
        Ok(())
    }

    /// 换绑手机：登录密码校验通过后，重加密 + 重脱敏覆盖 phone_enc / phone_masked。
    pub async fn bind_phone(
        &self,
        user_id: i64,
        password: &str,
        new_phone: &str,
    ) -> Result<()> {
        // 0. 手机号格式校验（大陆手机号 11 位）
        check_phone(new_phone)?;
        // 1. 取当前用户（不存在/已软删 → NotFound）
        let user: Option<User> = self
            .db
            .query_one(
                "SELECT * FROM users WHERE id = ? AND deleted_at IS NULL LIMIT 1",
                vec![user_id],
            )
            .await?;
        let user = user.ok_or(AppError::NotFound)?;
        // 2. 登录密码校验（换绑属敏感操作，需验明正身）
        if !Self::verify_password(password, &user.password_hash) {
            return Err(AppError::business("密码错误"));
        }
        // 3. 新手机号 AES 加密 + 脱敏，单行 UPDATE 覆盖（与 register 落库格式一致）；
        //    换绑手机属敏感操作，一并 token_version +1 撤销旧令牌
        let phone_enc = self.crypto.encrypt_str(new_phone)?;
        let phone_masked = crate::crypto::Masker::phone(new_phone);
        let params: Vec<Value> = vec![phone_enc.into(), phone_masked.into(), user_id.into()];
        self.db
            .exec_drop(
                "UPDATE users SET phone_enc = ?, phone_masked = ?, \
                 token_version = token_version + 1, updated_at = NOW() \
                 WHERE id = ?",
                params,
            )
            .await?;
        Ok(())
    }

    /// 当前用户资料（user/me）：按 id 查未删除用户，不存在 → NotFound。
    /// 返回完整 `User`，敏感字段（phone_enc/id_card_enc/password_hash）由模型
    /// `#[serde(skip_serializing)]` 保证不进 JSON 输出。
    pub async fn me(&self, user_id: i64) -> Result<User> {
        self.db
            .query_one(
                "SELECT * FROM users WHERE id = ? AND deleted_at IS NULL LIMIT 1",
                vec![user_id],
            )
            .await?
            .ok_or(AppError::NotFound)
    }

    /// 签发双令牌（Access + Refresh），载荷携带 token_version（调用方取自
    /// users 行当前版本；refresh 校验时比对，落后即视为已撤销）。
    pub fn issue_tokens(
        &self,
        user_id: i64,
        username: &str,
        role: Role,
        token_version: i64,
    ) -> Result<LoginResult> {
        let access_token =
            self.jwt
                .issue_token_with_version(user_id, role, None, token_version)?;
        // Refresh Token：长时效，阶段 0 直接复用 JWT 载荷（refresh 校验放 Redis 后实现）
        let refresh_token =
            self.jwt
                .issue_token_with_version(user_id, role, None, token_version)?;
        Ok(LoginResult {
            tokens: TokenPair {
                access_token,
                refresh_token,
                token_type: "Bearer".into(),
                expires_in: 7200,
            },
            user_id,
            username: username.to_string(),
            role: role.as_str().to_string(),
        })
    }
}

/// 便捷工厂（供控制器注入）
pub fn auth_service(
    cfg: JwtConfig,
    crypto: CryptoService,
    db: Db,
    wechat: WechatClient,
) -> AuthService {
    AuthService::new(cfg, crypto, db, wechat)
}
