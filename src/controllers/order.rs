//! 订单控制器：创建 / 我的订单 / 详情

use async_trait::async_trait;
use axum::http::StatusCode;
use axum::response::Response;

use bee_rust::bee_router::context::RouterError;
use bee_rust::bee_router::{Context, Controller};

use crate::services::order_service::{OrderService, CreateOrderReq};

use super::{error_response, json_envelope, ok_response, query_map, read_json};

/// 订单控制器（持有 OrderService，按请求路径与方法分派动作）
pub struct OrderController {
    service: OrderService,
}

impl OrderController {
    pub fn new(service: OrderService) -> Self {
        Self { service }
    }

    async fn create(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let req: CreateOrderReq = match read_json(ctx).await {
            Ok(r) => r,
            Err(resp) => return self.reply(ctx, resp).await,
        };
        match self.service.create(req).await {
            Ok(o) => {
                let data = serde_json::to_value(&o)
                    .map_err(|e| RouterError::SerializeError(e.to_string()))?;
                self.reply(ctx, ok_response(data)).await
            }
            Err(e) => self.reply(ctx, error_response(&e)).await,
        }
    }

    async fn detail(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let id = match ctx.param("id").and_then(|s| s.parse::<i64>().ok()) {
            Some(id) => id,
            None => return self.reply(ctx, json_envelope(40000, "订单 id 参数无效")).await,
        };
        match self.service.by_id(id).await {
            Ok(o) => {
                let data = serde_json::to_value(&o)
                    .map_err(|e| RouterError::SerializeError(e.to_string()))?;
                self.reply(ctx, ok_response(data)).await
            }
            Err(e) => self.reply(ctx, error_response(&e)).await,
        }
    }

    async fn my_orders(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let q = query_map(ctx.request.uri().query());
        let user_id: i64 = match q.get("user_id").and_then(|s| s.parse().ok()) {
            Some(id) => id,
            None => return self.reply(ctx, json_envelope(40000, "缺少 user_id")).await,
        };
        let page = q.get("page").and_then(|s| s.parse().ok()).unwrap_or(1);
        let size = q.get("size").and_then(|s| s.parse().ok()).unwrap_or(20);
        match self.service.by_user(user_id, page, size).await {
            Ok(list) => {
                let data = serde_json::json!({
                    "list": list,
                    "page": page,
                    "size": size,
                });
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
impl Controller for OrderController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let path = ctx.request.uri().path().to_string();
        let method = ctx.request.method().clone();
        if path.ends_with("/orders") && method == axum::http::Method::POST {
            self.create(ctx).await
        } else if path.ends_with("/orders") && method == axum::http::Method::GET {
            self.my_orders(ctx).await
        } else if method == axum::http::Method::GET && ctx.param("id").is_some() {
            self.detail(ctx).await
        } else {
            ctx.abort(StatusCode::NOT_FOUND, "接口不存在");
            Ok(())
        }
    }
}