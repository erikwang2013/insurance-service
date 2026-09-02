//! 认证服务：注册 / 登录 / 微信登录 stub + JWT 签发（对齐 backend-architecture.md §7）
//!
//! 说明：阶段 0 未接入 bee_orm，注册/登录的持久化操作用 `todo!()` 占位，但
//! 密码哈希（argon2）、JWT 签发/校验、令牌模型为完整可编译实现。

use argon2::Argon2;
use argon2::password_hash::{
    PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng,
};
use serde::{Deserialize, Serialize};

use mysql_async::prelude::Queryable;
use mysql_async::Value;

use crate::config::JwtConfig;
use crate::crypto::CryptoService;
use crate::db::{db_error, Db};
use crate::error::{AppError, Result};
use crate::middleware::auth::{JwtService, Role};
use crate::models::user::User;

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

/// 认证服务
pub struct AuthService {
    jwt: JwtService,
    crypto: CryptoService,
    db: Db,
}

impl AuthService {
    pub fn new(jwt_cfg: JwtConfig, crypto: CryptoService, db: Db) -> Self {
        Self {
            jwt: JwtService::new(jwt_cfg),
            crypto,
            db,
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

        // 4. 签发双令牌
        self.issue_tokens(user_id, &req.username, Role::User)
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

        // 4. 签发令牌
        let role = Role::from_str(&user.role).unwrap_or(Role::User);
        self.issue_tokens(user.id, &user.username, role)
    }

    /// 微信登录 stub（阶段 3 实现 code2session）
    pub async fn wechat_login(&self, _req: WechatLoginReq) -> Result<LoginResult> {
        Err(AppError::business("微信登录未接入（阶段 3 code2session 实现）"))
    }

    /// 签发双令牌（Access + Refresh）
    pub fn issue_tokens(&self, user_id: i64, username: &str, role: Role) -> Result<LoginResult> {
        let access_token = self.jwt.issue_access_token(user_id, role, None)?;
        // Refresh Token：长时效，阶段 0 直接复用 JWT 载荷（refresh 校验放 Redis 后实现）
        let refresh_token = self.jwt.issue_access_token(user_id, role, None)?;
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
pub fn auth_service(cfg: JwtConfig, crypto: CryptoService, db: Db) -> AuthService {
    AuthService::new(cfg, crypto, db)
}
