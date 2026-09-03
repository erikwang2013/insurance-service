//! 支付控制器：预支付 / 支付 / 微信预支付 / 回调

use async_trait::async_trait;
use axum::http::StatusCode;
use axum::response::Response;

use bee_rust::bee_router::context::RouterError;
use bee_rust::bee_router::{Context, Controller};

use crate::services::payment_service::{CallbackReq, CreatePaymentReq, PaymentService};

use super::{error_response, json_envelope, ok_response, read_json};

/// 支付控制器（持有 PaymentService，按请求路径与方法分派动作）
pub struct PaymentController {
    service: PaymentService,
}

impl PaymentController {
    pub fn new(service: PaymentService) -> Self {
        Self { service }
    }

    async fn prepay(&self, ctx: &mut Context, channel: Option<&str>) -> Result<(), RouterError> {
        let order_id = match ctx.param("order_id").and_then(|s| s.parse::<i64>().ok()) {
            Some(id) => id,
            None => return self.reply(ctx, json_envelope(40000, "缺少 order_id")).await,
        };
        let user_id = match ctx.param("user_id").and_then(|s| s.parse::<i64>().ok()) {
            Some(id) => id,
            None => return self.reply(ctx, json_envelope(40000, "缺少 user_id")).await,
        };
        let req = CreatePaymentReq {
            order_id,
            user_id,
            channel: channel.unwrap_or_default().to_string(),
        };
        match self.service.prepay(req).await {
            Ok(p) => {
                let data = serde_json::to_value(&p)
                    .map_err(|e| RouterError::SerializeError(e.to_string()))?;
                self.reply(ctx, ok_response(data)).await
            }
            Err(e) => self.reply(ctx, error_response(&e)).await,
        }
    }

    async fn pay(&self, ctx: &mut Context) -> Result<(), RouterError> {
        self.prepay(ctx, Some("MOCK")).await
    }

    async fn wechat_prepay(&self, ctx: &mut Context) -> Result<(), RouterError> {
        // 阶段 0：微信支付走 MockProvider（真实 WechatPayProvider 待阶段 4 接入）
        self.prepay(ctx, Some("WECHAT")).await
    }

    async fn callback(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let _provider = match ctx.param("provider") {
            Some(p) => p,
            None => return self.reply(ctx, json_envelope(40000, "缺少 provider")).await,
        };
        let req: CallbackReq = match read_json(ctx).await {
            Ok(r) => r,
            Err(resp) => return self.reply(ctx, resp).await,
        };
        match self.service.callback(req).await {
            Ok(p) => {
                let data = serde_json::to_value(&p)
                    .map_err(|e| RouterError::SerializeError(e.to_string()))?;
                self.reply(ctx, ok_response(data)).await
            }
            Err(e) => self.reply(ctx, error_response(&e)).await,
        }
    }

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
impl Controller for PaymentController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let path = ctx.request.uri().path().to_string();
        let method = ctx.request.method().clone();
        if method != axum::http::Method::POST {
            ctx.abort(StatusCode::NOT_FOUND, "接口不存在");
            return Ok(());
        }
        if path.contains("/wechat/prepay") {
            self.wechat_prepay(ctx).await
        } else if path.contains("/callback") {
            self.callback(ctx).await
        } else if path.ends_with("/pay") {
            self.pay(ctx).await
        } else if path.ends_with("/prepay") {
            self.prepay(ctx, None).await
        } else {
            ctx.abort(StatusCode::NOT_FOUND, "接口不存在");
            Ok(())
        }
    }
}