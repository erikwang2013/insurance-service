//! 运营侧商品管理控制器（建档/更新 + 上下架/停售）
//!
//! 仅含 POST 动作，与公开 product 控制器分离；动作执行前由服务层校验
//! 操作人角色（OPERATOR/ADMIN，否则 40300）。
//!
//! TODO(lead): 接线 —— controllers/mod.rs 增 `pub mod admin;`、AppState 增
//! `pub admin: Arc<AdminController>` 与 `AdminController::new(db.clone())`、
//! `pub admin_handler` 适配器；routes.rs `build_bee_router` 注册
//! `POST /admin/products` 与 `POST /admin/products/{id}/status`（对齐
//! route_table() 中 admin.product_upsert 条目；status 条目待补）。

use async_trait::async_trait;
use axum::http::StatusCode;
use axum::response::Response;

use bee_rust::bee_router::context::RouterError;
use bee_rust::bee_router::{Context, Controller};

use crate::db::Db;
use crate::services::product_service::{
    admin_change_status, admin_upsert, AdminStatusReq, AdminUpsertReq,
};

use super::{error_response, json_envelope, ok_response, read_json};

/// 运营侧商品控制器（持有 Db，按请求路径分派动作）
pub struct AdminController {
    db: Db,
}

impl AdminController {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    async fn upsert(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let req: AdminUpsertReq = match read_json(ctx).await {
            Ok(r) => r,
            Err(resp) => return self.reply(ctx, resp).await,
        };
        match admin_upsert(&self.db, &req).await {
            Ok(p) => {
                let data = serde_json::to_value(&p)
                    .map_err(|e| RouterError::SerializeError(e.to_string()))?;
                self.reply(ctx, ok_response(data)).await
            }
            Err(e) => self.reply(ctx, error_response(&e)).await,
        }
    }

    async fn change_status(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let product_id = match ctx.param("id").and_then(|s| s.parse::<i64>().ok()) {
            Some(id) => id,
            None => {
                return self.reply(ctx, json_envelope(40000, "商品 id 参数无效")).await;
            }
        };
        let req: AdminStatusReq = match read_json(ctx).await {
            Ok(r) => r,
            Err(resp) => return self.reply(ctx, resp).await,
        };
        match admin_change_status(&self.db, product_id, &req).await {
            Ok(p) => {
                let data = serde_json::to_value(&p)
                    .map_err(|e| RouterError::SerializeError(e.to_string()))?;
                self.reply(ctx, ok_response(data)).await
            }
            Err(e) => self.reply(ctx, error_response(&e)).await,
        }
    }

    /// 将响应写入 Context（与 claim/product 控制器同模式）
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
impl Controller for AdminController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let path = ctx.request.uri().path().to_string();
        let method = ctx.request.method().clone();
        if method == axum::http::Method::POST && path.ends_with("/status") {
            self.change_status(ctx).await
        } else if method == axum::http::Method::POST && path.ends_with("/admin/products") {
            self.upsert(ctx).await
        } else {
            ctx.abort(StatusCode::NOT_FOUND, "接口不存在");
            Ok(())
        }
    }
}
