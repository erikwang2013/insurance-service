//! 认证控制器：注册 / 登录 / 微信登录 / 刷新 / 登出 / 我的资料（user/me）

use async_trait::async_trait;
use axum::http::StatusCode;
use axum::response::Response;

use bee_rust::bee_router::context::RouterError;
use bee_rust::bee_router::{Context, Controller};

use crate::services::auth_service::{
    AuthService, BindPhoneReq, ChangePasswordReq, LoginReq, RefreshReq, RegisterReq,
    WechatLoginReq,
};

use super::{error_response, json_envelope, ok_response, query_map, read_json};

/// 认证控制器（持有 AuthService，按请求路径分派动作）
pub struct AuthController {
    service: AuthService,
}

impl AuthController {
    pub fn new(service: AuthService) -> Self {
        Self { service }
    }

    async fn register(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let req: RegisterReq = match read_json(ctx).await {
            Ok(r) => r,
            Err(resp) => return self.reply(ctx, resp).await,
        };
        match self.service.register(req).await {
            Ok(r) => {
                let data = serde_json::to_value(&r)
                    .map_err(|e| RouterError::SerializeError(e.to_string()))?;
                self.reply(ctx, ok_response(data)).await
            }
            Err(e) => self.reply(ctx, error_response(&e)).await,
        }
    }

    async fn login(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let req: LoginReq = match read_json(ctx).await {
            Ok(r) => r,
            Err(resp) => return self.reply(ctx, resp).await,
        };
        match self.service.login(req).await {
            Ok(r) => {
                let data = serde_json::to_value(&r)
                    .map_err(|e| RouterError::SerializeError(e.to_string()))?;
                self.reply(ctx, ok_response(data)).await
            }
            Err(e) => self.reply(ctx, error_response(&e)).await,
        }
    }

    async fn wechat_login(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let req: WechatLoginReq = match read_json(ctx).await {
            Ok(r) => r,
            Err(resp) => return self.reply(ctx, resp).await,
        };
        match self.service.wechat_login(req).await {
            Ok(r) => {
                let data = serde_json::to_value(&r)
                    .map_err(|e| RouterError::SerializeError(e.to_string()))?;
                self.reply(ctx, ok_response(data)).await
            }
            Err(e) => self.reply(ctx, error_response(&e)).await,
        }
    }

    async fn refresh(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let req: RefreshReq = match read_json(ctx).await {
            Ok(r) => r,
            Err(resp) => return self.reply(ctx, resp).await,
        };
        match self.service.refresh(&req.refresh_token).await {
            Ok(r) => {
                let data = serde_json::to_value(&r)
                    .map_err(|e| RouterError::SerializeError(e.to_string()))?;
                self.reply(ctx, ok_response(data)).await
            }
            Err(e) => self.reply(ctx, error_response(&e)).await,
        }
    }

    async fn logout(&self, ctx: &mut Context) -> Result<(), RouterError> {
        // 阶段 0：无服务端会话态，直接成功（后续随 session/令牌注销闭环实现）
        self.reply(ctx, ok_response(serde_json::json!({ "logout": true })))
            .await
    }

    /// 修改密码（POST /user/password，user_id 经 query 显式传入，同 me()）
    async fn change_password(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let q = query_map(ctx.request.uri().query());
        let user_id: i64 = match q.get("user_id").and_then(|s| s.parse().ok()) {
            Some(id) => id,
            None => return self.reply(ctx, json_envelope(40000, "缺少 user_id")).await,
        };
        let req: ChangePasswordReq = match read_json(ctx).await {
            Ok(r) => r,
            Err(resp) => return self.reply(ctx, resp).await,
        };
        match self
            .service
            .change_password(user_id, &req.old_password, &req.new_password)
            .await
        {
            Ok(_) => self
                .reply(ctx, ok_response(serde_json::json!({ "changed": true })))
                .await,
            Err(e) => self.reply(ctx, error_response(&e)).await,
        }
    }

    /// 换绑手机（POST /user/phone，user_id 经 query 显式传入，同 me()）
    async fn bind_phone(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let q = query_map(ctx.request.uri().query());
        let user_id: i64 = match q.get("user_id").and_then(|s| s.parse().ok()) {
            Some(id) => id,
            None => return self.reply(ctx, json_envelope(40000, "缺少 user_id")).await,
        };
        let req: BindPhoneReq = match read_json(ctx).await {
            Ok(r) => r,
            Err(resp) => return self.reply(ctx, resp).await,
        };
        match self
            .service
            .bind_phone(user_id, &req.password, &req.new_phone)
            .await
        {
            Ok(_) => self
                .reply(ctx, ok_response(serde_json::json!({ "bound": true })))
                .await,
            Err(e) => self.reply(ctx, error_response(&e)).await,
        }
    }

    /// 当前用户资料（GET /user/me，user_id 经 query 显式传入，见 claim.rs my_claims）
    async fn me(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let q = query_map(ctx.request.uri().query());
        let user_id: i64 = match q.get("user_id").and_then(|s| s.parse().ok()) {
            Some(id) => id,
            None => return self.reply(ctx, json_envelope(40000, "缺少 user_id")).await,
        };
        match self.service.me(user_id).await {
            Ok(u) => {
                let data = serde_json::to_value(&u)
                    .map_err(|e| RouterError::SerializeError(e.to_string()))?;
                self.reply(ctx, ok_response(data)).await
            }
            Err(e) => self.reply(ctx, error_response(&e)).await,
        }
    }

    /// 将已构造好的响应写入 Context（bee 管线以 HTTP 200 + 业务码信封返回）
    async fn reply(&self, ctx: &mut Context, resp: Response) -> Result<(), RouterError> {
        let bytes = axum::body::to_bytes(resp.into_body(), 2 * 1024 * 1024)
            .await
            .map_err(|e| RouterError::Internal(e.to_string()))?;
        let value: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|e| RouterError::SerializeError(e.to_string()))?;
        ctx.json(&value)?;
        Ok(())
    }
}

#[async_trait]
impl Controller for AuthController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let path = ctx.request.uri().path().to_string();
        if path.ends_with("/register") {
            self.register(ctx).await
        } else if path.ends_with("/wechat/login") {
            self.wechat_login(ctx).await
        } else if path.ends_with("/login") {
            self.login(ctx).await
        } else if path.ends_with("/refresh") {
            self.refresh(ctx).await
        } else if path.ends_with("/logout") {
            self.logout(ctx).await
        } else if path.ends_with("/user/password") {
            self.change_password(ctx).await
        } else if path.ends_with("/user/phone") {
            self.bind_phone(ctx).await
        } else if path.ends_with("/user/me") {
            self.me(ctx).await
        } else {
            ctx.abort(StatusCode::NOT_FOUND, "接口不存在");
            Ok(())
        }
    }
}
