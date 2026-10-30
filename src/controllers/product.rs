//! 产品控制器：列表 / 详情 / 条款 / 精选

use async_trait::async_trait;
use axum::http::StatusCode;

use bee_rust::bee_router::context::RouterError;
use bee_rust::bee_router::{Context, Controller};

use crate::db::Db;
use crate::services::product_service;

use super::{error_response, json_envelope, ok_response, query_map, reply};

/// 产品控制器（持有 Db，任务 #3 接入真实查询）
pub struct ProductController {
    db: Db,
}

impl ProductController {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    async fn list(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let q = query_map(ctx.request.uri().query());
        // 公开列表只暴露在售；客户端 status 一律忽略（运营态走 /admin/products）
        let page = q.get("page").and_then(|s| s.parse().ok()).unwrap_or(1);
        let size = q.get("size").and_then(|s| s.parse().ok()).unwrap_or(20);
        match product_service::list(&self.db, page, size).await {
            Ok(list) => {
                let data = serde_json::to_value(&list)
                    .map_err(|e| RouterError::SerializeError(e.to_string()))?;
                reply(ctx, ok_response(data)).await
            }
            Err(e) => reply(ctx, error_response(&e)).await,
        }
    }

    async fn detail(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let id = match ctx.param("id").and_then(|s| s.parse::<i64>().ok()) {
            Some(id) => id,
            None => {
                return reply(ctx, json_envelope(40000, "产品 id 参数无效")).await;
            }
        };
        match product_service::public_detail(&self.db, id).await {
            Ok(p) => {
                let data = serde_json::to_value(&p)
                    .map_err(|e| RouterError::SerializeError(e.to_string()))?;
                reply(ctx, ok_response(data)).await
            }
            Err(e) => reply(ctx, error_response(&e)).await,
        }
    }

    async fn clauses(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let id = match ctx.param("id").and_then(|s| s.parse::<i64>().ok()) {
            Some(id) => id,
            None => {
                return reply(ctx, json_envelope(40000, "产品 id 参数无效")).await;
            }
        };
        match product_service::clauses(&self.db, id).await {
            Ok(items) => {
                let data = serde_json::to_value(&items)
                    .map_err(|e| RouterError::SerializeError(e.to_string()))?;
                reply(ctx, ok_response(data)).await
            }
            Err(e) => reply(ctx, error_response(&e)).await,
        }
    }

    async fn featured(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let q = query_map(ctx.request.uri().query());
        let page = q.get("page").and_then(|s| s.parse().ok()).unwrap_or(1);
        let size = q.get("size").and_then(|s| s.parse().ok()).unwrap_or(20);
        match product_service::featured(&self.db, page, size).await {
            Ok(list) => {
                let data = serde_json::to_value(&list)
                    .map_err(|e| RouterError::SerializeError(e.to_string()))?;
                reply(ctx, ok_response(data)).await
            }
            Err(e) => reply(ctx, error_response(&e)).await,
        }
    }

}

#[async_trait]
impl Controller for ProductController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let path = ctx.request.uri().path().to_string();
        if path.ends_with("/featured") {
            self.featured(ctx).await
        } else if path.ends_with("/clauses") {
            self.clauses(ctx).await
        } else if path.ends_with("/products") {
            self.list(ctx).await
        } else if ctx.param("id").is_some() {
            self.detail(ctx).await
        } else {
            ctx.abort(StatusCode::NOT_FOUND, "接口不存在");
            Ok(())
        }
    }
}
