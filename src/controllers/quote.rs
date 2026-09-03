//! 报价控制器：创建 / 详情

use async_trait::async_trait;
use axum::http::StatusCode;
use axum::response::Response;

use bee_rust::bee_router::context::RouterError;
use bee_rust::bee_router::{Context, Controller};

use crate::services::quote_service::{QuoteService, CreateQuoteReq};

use super::{error_response, json_envelope, ok_response, read_json};

/// 报价控制器（持有 QuoteService，按请求路径分派动作）
pub struct QuoteController {
    service: QuoteService,
}

impl QuoteController {
    pub fn new(service: QuoteService) -> Self {
        Self { service }
    }

    async fn create(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let req: CreateQuoteReq = match read_json(ctx).await {
            Ok(r) => r,
            Err(resp) => return self.reply(ctx, resp).await,
        };
        match self.service.create(req).await {
            Ok(q) => {
                let data = serde_json::to_value(&q)
                    .map_err(|e| RouterError::SerializeError(e.to_string()))?;
                self.reply(ctx, ok_response(data)).await
            }
            Err(e) => self.reply(ctx, error_response(&e)).await,
        }
    }

    async fn detail(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let id = match ctx.param("id").and_then(|s| s.parse::<i64>().ok()) {
            Some(id) => id,
            None => return self.reply(ctx, json_envelope(40000, "报价 id 参数无效")).await,
        };
        // 阶段 0：QuoteService 未暴露 by_id，详情接口先占位
        self.reply(
            ctx,
            json_envelope(40001, format!("报价详情接口未接入数据库（仅报价 id {}）", id)),
        )
        .await
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
impl Controller for QuoteController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let path = ctx.request.uri().path().to_string();
        let method = ctx.request.method().clone();
        if path.ends_with("/quotes") && method == axum::http::Method::POST {
            self.create(ctx).await
        } else if method == axum::http::Method::GET && ctx.param("id").is_some() {
            self.detail(ctx).await
        } else {
            ctx.abort(StatusCode::NOT_FOUND, "接口不存在");
            Ok(())
        }
    }
}