//! JWT 认证 + RBAC 角色守卫（对齐 backend-architecture.md §7）
//!
//! Access Token（JWT）：短时效、无状态，携带 `sub`（user_id）、`role`、`platform`。
//! 认证过滤器解析 `Authorization: Bearer <jwt>` 注入 `current_user`；角色守卫校验权限。

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::{Filter, RequestCtx};
use crate::config::JwtConfig;

/// RBAC 角色
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Role {
    User,
    Agent,
    Operator,
    Admin,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::User => "USER",
            Role::Agent => "AGENT",
            Role::Operator => "OPERATOR",
            Role::Admin => "ADMIN",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "USER" => Some(Role::User),
            "AGENT" => Some(Role::Agent),
            "OPERATOR" => Some(Role::Operator),
            "ADMIN" => Some(Role::Admin),
            _ => None,
        }
    }
}

/// JWT 载荷（Access Token）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// 用户 ID
    pub sub: i64,
    /// 角色字符串（"USER" | "ADMIN" | "OPERATOR" | "AGENT"）
    pub role: String,
    /// 客户端平台（flutter/miniprogram/harmony）
    pub platform: Option<String>,
    /// 签发时间（epoch 秒）
    pub iat: i64,
    /// 过期时间（epoch 秒）
    pub exp: i64,
    /// 签发者
    pub iss: String,
    /// 令牌版本（对齐 users.token_version）：签发时记录、refresh 校验比对，
    /// logout/改密/换绑手机后旧令牌因版本落后失效。老令牌无此载荷，
    /// serde 缺省为 0，与库中默认 0 一致，兼容既有会话。
    #[serde(default)]
    pub token_version: i64,
    /// 令牌类型（"access" | "refresh"）：禁止两种令牌互换使用。
    /// 缺省空串供旧令牌反序列化（随后被 verify_access/verify_refresh 拒绝）。
    #[serde(default)]
    pub typ: String,
}

/// 认证通过后注入上下文的当前用户
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: i64,
    pub role: Role,
    pub platform: Option<String>,
}

/// Claims.typ 取值：access / refresh 不可互换
const TYP_ACCESS: &str = "access";
const TYP_REFRESH: &str = "refresh";

/// 签发/校验 token 的服务（封装 jsonwebtoken）
#[derive(Clone)]
pub struct JwtService {
    cfg: JwtConfig,
}

impl JwtService {
    pub fn new(cfg: JwtConfig) -> Self {
        Self { cfg }
    }

    /// access token 有效期（秒），供登录响应的 `expires_in` 使用
    pub fn access_expiry(&self) -> i64 {
        self.cfg.access_expiry
    }

    /// 签发 access token（token_version 固定 0；兼容入口，供测试/骨架路径使用）
    pub fn issue_access_token(
        &self,
        user_id: i64,
        role: Role,
        platform: Option<String>,
    ) -> Result<String, crate::error::AppError> {
        self.issue_access_token_with_version(user_id, role, platform, 0)
    }

    /// 签发带令牌版本的 access JWT（version 由调用方取自 users.token_version）
    pub fn issue_access_token_with_version(
        &self,
        user_id: i64,
        role: Role,
        platform: Option<String>,
        token_version: i64,
    ) -> Result<String, crate::error::AppError> {
        self.issue(
            user_id,
            role,
            platform,
            token_version,
            TYP_ACCESS,
            self.cfg.access_expiry,
        )
    }

    /// 签发带令牌版本的 refresh JWT（独立 typ + 长时效）
    pub fn issue_refresh_token_with_version(
        &self,
        user_id: i64,
        role: Role,
        platform: Option<String>,
        token_version: i64,
    ) -> Result<String, crate::error::AppError> {
        self.issue(
            user_id,
            role,
            platform,
            token_version,
            TYP_REFRESH,
            self.cfg.refresh_expiry,
        )
    }

    /// 兼容入口：等同 access token
    pub fn issue_token_with_version(
        &self,
        user_id: i64,
        role: Role,
        platform: Option<String>,
        token_version: i64,
    ) -> Result<String, crate::error::AppError> {
        self.issue_access_token_with_version(user_id, role, platform, token_version)
    }

    fn issue(
        &self,
        user_id: i64,
        role: Role,
        platform: Option<String>,
        token_version: i64,
        typ: &str,
        expiry: i64,
    ) -> Result<String, crate::error::AppError> {
        let now = chrono::Utc::now().timestamp();
        let claims = Claims {
            sub: user_id,
            role: role.as_str().to_string(),
            platform,
            iat: now,
            exp: now + expiry,
            iss: self.cfg.issuer.clone(),
            token_version,
            typ: typ.to_string(),
        };
        let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS256);
        jsonwebtoken::encode(
            &header,
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(self.cfg.secret.as_bytes()),
        )
        .map_err(|e| crate::error::AppError::Internal(Box::new(e)))
    }

    /// 校验签名/issuer/exp（leeway 0），不做 typ 检查。
    /// 保留为公开入口：旧 `AuthFilter` 在 Task 6 移除前仍使用。
    pub fn verify_token(&self, token: &str) -> Result<Claims, crate::error::AppError> {
        let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
        // jsonwebtoken 默认 exp/nbf 有 60s 宽容窗口；这里按签发语义精确执行过期
        validation.leeway = 0;
        validation.set_issuer(std::slice::from_ref(&self.cfg.issuer));
        let data = jsonwebtoken::decode::<Claims>(
            token,
            &jsonwebtoken::DecodingKey::from_secret(self.cfg.secret.as_bytes()),
            &validation,
        )
        .map_err(|_| crate::error::AppError::Unauthorized)?;
        Ok(data.claims)
    }

    /// 校验 access token（签名/issuer/exp + typ == "access"）
    pub fn verify_access(&self, token: &str) -> Result<Claims, crate::error::AppError> {
        self.verify_typed(token, TYP_ACCESS)
    }

    /// 校验 refresh token（签名/issuer/exp + typ == "refresh"）
    pub fn verify_refresh(&self, token: &str) -> Result<Claims, crate::error::AppError> {
        self.verify_typed(token, TYP_REFRESH)
    }

    fn verify_typed(
        &self,
        token: &str,
        expected: &str,
    ) -> Result<Claims, crate::error::AppError> {
        let claims = self.verify_token(token)?;
        if claims.typ != expected {
            return Err(crate::error::AppError::Unauthorized);
        }
        Ok(claims)
    }
}

/// 认证过滤器：解析 `Authorization: Bearer <jwt>`，注入 ctx.current_user
pub struct AuthFilter {
    jwt: JwtService,
}

impl AuthFilter {
    pub fn new(cfg: JwtConfig) -> Self {
        Self {
            jwt: JwtService::new(cfg),
        }
    }
}

impl Filter for AuthFilter {
    fn name(&self) -> &'static str {
        "auth"
    }

    fn before(&self, ctx: &mut RequestCtx) -> Result<(), String> {
        // 从 ctx 头取 Authorization —— 骨架以 body_text 冗余承载头信息；真实集成
        // 由 bee_router Context.header 提供。此处定义契约：调用方在进入前注入。
        // 阶段 0 为简化，认证过滤器接受 `Authorization` 通过 `ctx` 的扩展字段传入，
        // 此处直接校验 —— 若无 token 视为未认证。
        let token = ctx
            .auth_token()
            .ok_or_else(|| crate::error::AppError::Unauthorized.to_string())?;
        let token = token.strip_prefix("Bearer ").unwrap_or(token);
        let claims = self
            .jwt
            .verify_token(token)
            .map_err(|_| crate::error::AppError::Unauthorized.to_string())?;
        let role = Role::parse(&claims.role).unwrap_or(Role::User);
        ctx.current_user = Some(AuthUser {
            id: claims.sub,
            role,
            platform: claims.platform,
        });
        Ok(())
    }
}

impl RequestCtx {
    /// 当前认证用户（无则 Unauthorized）
    pub fn current_user(&self) -> Result<&AuthUser, crate::error::AppError> {
        self.current_user
            .as_ref()
            .ok_or(crate::error::AppError::Unauthorized)
    }
}

/// 角色守卫：校验当前用户角色属于给定集合
pub struct RequireRoleFilter {
    roles: HashSet<Role>,
}

impl RequireRoleFilter {
    pub fn new(roles: &[Role]) -> Self {
        Self {
            roles: roles.iter().copied().collect(),
        }
    }
}

impl Filter for RequireRoleFilter {
    fn name(&self) -> &'static str {
        "require_role"
    }

    fn before(&self, ctx: &mut RequestCtx) -> Result<(), String> {
        let user = ctx.current_user().map_err(|e| e.to_string())?;
        if self.roles.contains(&user.role) {
            Ok(())
        } else {
            Err(crate::error::AppError::Forbidden.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn jwt_service() -> JwtService {
        JwtService::new(JwtConfig {
            // 新配置规则要求 >= 32 字节
            secret: "0123456789abcdef0123456789abcdef".into(),
            issuer: "insurance-test".into(),
            access_expiry: 100,
            refresh_expiry: 1000,
        })
    }

    #[test]
    fn access_and_refresh_tokens_are_typed_and_distinct() {
        let jwt = jwt_service();
        let access = jwt
            .issue_access_token_with_version(7, Role::Agent, Some("flutter".into()), 3)
            .unwrap();
        let refresh = jwt
            .issue_refresh_token_with_version(7, Role::Agent, Some("flutter".into()), 3)
            .unwrap();
        assert_ne!(access, refresh, "access 与 refresh 必须不同");

        let a = jwt.verify_access(&access).unwrap();
        assert_eq!(a.typ, "access");
        assert_eq!(a.sub, 7);
        assert_eq!(a.token_version, 3);
        assert_eq!(a.exp - a.iat, jwt.access_expiry());
        assert!(jwt.verify_refresh(&access).is_err(), "access 不可当 refresh 用");

        let r = jwt.verify_refresh(&refresh).unwrap();
        assert_eq!(r.typ, "refresh");
        assert_eq!(r.exp - r.iat, 1000);
        assert!(jwt.verify_access(&refresh).is_err(), "refresh 不可当 access 用");
    }
}
