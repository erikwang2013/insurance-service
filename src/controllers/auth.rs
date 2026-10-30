//! 认证控制器：注册 / 登录 / 微信登录 / 刷新 / 登出 / 我的资料（user/me）

use async_trait::async_trait;
use axum::http::StatusCode;

use bee_rust::bee_router::context::RouterError;
use bee_rust::bee_router::{Context, Controller};

use crate::services::auth_service::{
    AuthService, BindPhoneReq, ChangePasswordReq, LoginReq, RefreshReq, RegisterReq,
    WechatLoginReq,
};

use super::{current_user_id, error_response, ok_response, read_json, reply};

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
            Err(resp) => return reply(ctx, resp).await,
        };
        match self.service.register(req).await {
            Ok(r) => {
                let data = serde_json::to_value(&r)
                    .map_err(|e| RouterError::SerializeError(e.to_string()))?;
                reply(ctx, ok_response(data)).await
            }
            Err(e) => reply(ctx, error_response(&e)).await,
        }
    }

    async fn login(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let req: LoginReq = match read_json(ctx).await {
            Ok(r) => r,
            Err(resp) => return reply(ctx, resp).await,
        };
        match self.service.login(req).await {
            Ok(r) => {
                let data = serde_json::to_value(&r)
                    .map_err(|e| RouterError::SerializeError(e.to_string()))?;
                reply(ctx, ok_response(data)).await
            }
            Err(e) => reply(ctx, error_response(&e)).await,
        }
    }

    async fn wechat_login(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let req: WechatLoginReq = match read_json(ctx).await {
            Ok(r) => r,
            Err(resp) => return reply(ctx, resp).await,
        };
        match self.service.wechat_login(req).await {
            Ok(r) => {
                let data = serde_json::to_value(&r)
                    .map_err(|e| RouterError::SerializeError(e.to_string()))?;
                reply(ctx, ok_response(data)).await
            }
            Err(e) => reply(ctx, error_response(&e)).await,
        }
    }

    async fn refresh(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let req: RefreshReq = match read_json(ctx).await {
            Ok(r) => r,
            Err(resp) => return reply(ctx, resp).await,
        };
        match self.service.refresh(&req.refresh_token).await {
            Ok(r) => {
                let data = serde_json::to_value(&r)
                    .map_err(|e| RouterError::SerializeError(e.to_string()))?;
                reply(ctx, ok_response(data)).await
            }
            Err(e) => reply(ctx, error_response(&e)).await,
        }
    }

    async fn logout(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let user_id = match current_user_id(ctx) {
            Ok(id) => id,
            Err(resp) => return reply(ctx, resp).await,
        };
        match self.service.logout(user_id).await {
            // token_version +1：此前签发的 refresh 全部失效（无状态 JWT 撤销闭环）
            Ok(_) => reply(ctx, ok_response(serde_json::json!({ "logout": true })))
                .await,
            Err(e) => reply(ctx, error_response(&e)).await,
        }
    }

    /// 微信绑定（POST /auth/wechat/bind，身份取自已认证会话）
    async fn bind_wechat(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let user_id = match current_user_id(ctx) {
            Ok(id) => id,
            Err(resp) => return reply(ctx, resp).await,
        };
        // 请求体与微信登录一致：{ "code": "<wx.login 返回的 code>" }
        let req: WechatLoginReq = match read_json(ctx).await {
            Ok(r) => r,
            Err(resp) => return reply(ctx, resp).await,
        };
        match self.service.bind_wechat(user_id, &req.code).await {
            Ok(_) => reply(ctx, ok_response(serde_json::json!({ "bound": true })))
                .await,
            Err(e) => reply(ctx, error_response(&e)).await,
        }
    }

    /// 修改密码（POST /user/password，身份取自已认证会话）
    async fn change_password(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let user_id = match current_user_id(ctx) {
            Ok(id) => id,
            Err(resp) => return reply(ctx, resp).await,
        };
        let req: ChangePasswordReq = match read_json(ctx).await {
            Ok(r) => r,
            Err(resp) => return reply(ctx, resp).await,
        };
        match self
            .service
            .change_password(user_id, &req.old_password, &req.new_password)
            .await
        {
            Ok(_) => reply(ctx, ok_response(serde_json::json!({ "changed": true })))
                .await,
            Err(e) => reply(ctx, error_response(&e)).await,
        }
    }

    /// 换绑手机（POST /user/phone，身份取自已认证会话）
    async fn bind_phone(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let user_id = match current_user_id(ctx) {
            Ok(id) => id,
            Err(resp) => return reply(ctx, resp).await,
        };
        let req: BindPhoneReq = match read_json(ctx).await {
            Ok(r) => r,
            Err(resp) => return reply(ctx, resp).await,
        };
        match self
            .service
            .bind_phone(user_id, &req.password, &req.new_phone)
            .await
        {
            Ok(_) => reply(ctx, ok_response(serde_json::json!({ "bound": true })))
                .await,
            Err(e) => reply(ctx, error_response(&e)).await,
        }
    }

    /// 当前用户资料（GET /user/me，身份取自已认证会话）
    async fn me(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let user_id = match current_user_id(ctx) {
            Ok(id) => id,
            Err(resp) => return reply(ctx, resp).await,
        };
        match self.service.me(user_id).await {
            Ok(u) => {
                let data = serde_json::to_value(&u)
                    .map_err(|e| RouterError::SerializeError(e.to_string()))?;
                reply(ctx, ok_response(data)).await
            }
            Err(e) => reply(ctx, error_response(&e)).await,
        }
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
        } else if path.ends_with("/wechat/bind") {
            self.bind_wechat(ctx).await
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
