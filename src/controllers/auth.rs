//! 认证控制器：注册 / 登录 / 微信登录 / 刷新 / 登出

use async_trait::async_trait;
use axum::http::StatusCode;
use axum::response::Response;

use bee_rust::bee_router::context::RouterError;
use bee_rust::bee_router::{Context, Controller};

use crate::services::auth_service::{AuthService, LoginReq, RegisterReq, WechatLoginReq};

use super::{error_response, json_envelope, ok_response, read_json};

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
        // 阶段 0：刷新令牌尚未接入数据库（任务 #3 后随登录闭环实现）
        self.reply(ctx, json_envelope(40001, "刷新令牌接口未接入数据库（任务 #3）"))
            .await
    }

    async fn logout(&self, ctx: &mut Context) -> Result<(), RouterError> {
        // 阶段 0：无服务端会话态，直接成功（后续随 session/令牌注销闭环实现）
        self.reply(ctx, ok_response(serde_json::json!({ "logout": true })))
            .await
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
        } else {
            ctx.abort(StatusCode::NOT_FOUND, "接口不存在");
            Ok(())
        }
    }
}
