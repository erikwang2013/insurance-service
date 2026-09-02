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

    pub fn from_str(s: &str) -> Option<Self> {
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
}

/// 认证通过后注入上下文的当前用户
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: i64,
    pub role: Role,
    pub platform: Option<String>,
}

/// 签发/校验 token 的服务（封装 jsonwebtoken）
pub struct JwtService {
    cfg: JwtConfig,
}

impl JwtService {
    pub fn new(cfg: JwtConfig) -> Self {
        Self { cfg }
    }

    /// 签发 access token
    pub fn issue_access_token(
        &self,
        user_id: i64,
        role: Role,
        platform: Option<String>,
    ) -> Result<String, crate::error::AppError> {
        let now = chrono::Utc::now().timestamp();
        let claims = Claims {
            sub: user_id,
            role: role.as_str().to_string(),
            platform,
            iat: now,
            exp: now + self.cfg.access_expiry,
            iss: self.cfg.issuer.clone(),
        };
        let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS256);
        jsonwebtoken::encode(
            &header,
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(self.cfg.secret.as_bytes()),
        )
        .map_err(|e| crate::error::AppError::Internal(Box::new(e)))
    }

    /// 校验并解析 token
    pub fn verify_token(&self, token: &str) -> Result<Claims, crate::error::AppError> {
        let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
        // jsonwebtoken 默认 exp/nbf 有 60s 宽容窗口；这里按签发语义精确执行过期
        validation.leeway = 0;
        validation.set_issuer(&[self.cfg.issuer.clone()]);
        let data = jsonwebtoken::decode::<Claims>(
            token,
            &jsonwebtoken::DecodingKey::from_secret(self.cfg.secret.as_bytes()),
            &validation,
        )
        .map_err(|_| crate::error::AppError::Unauthorized)?;
        Ok(data.claims)
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
        let role = Role::from_str(&claims.role).unwrap_or(Role::User);
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
