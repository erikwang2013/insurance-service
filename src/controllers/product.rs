//! 产品控制器：列表 / 详情 / 条款 / 精选

use async_trait::async_trait;
use axum::http::StatusCode;
use axum::response::Response;

use bee_rust::bee_router::context::RouterError;
use bee_rust::bee_router::{Context, Controller};

use crate::db::Db;
use crate::services::product_service;

use super::{error_response, json_envelope, ok_response, query_map};

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
        let status = q.get("status").map(String::as_str).unwrap_or("");
        let page = q.get("page").and_then(|s| s.parse().ok()).unwrap_or(1);
        let size = q.get("size").and_then(|s| s.parse().ok()).unwrap_or(20);
        match product_service::list(&self.db, status, page, size).await {
            Ok(list) => {
                let data = serde_json::to_value(&list)
                    .map_err(|e| RouterError::SerializeError(e.to_string()))?;
                self.reply(ctx, ok_response(data)).await
            }
            Err(e) => self.reply(ctx, error_response(&e)).await,
        }
    }

    async fn detail(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let id = match ctx.param("id").and_then(|s| s.parse::<i64>().ok()) {
            Some(id) => id,
            None => {
                return self.reply(ctx, json_envelope(40000, "产品 id 参数无效")).await;
            }
        };
        match product_service::detail(&self.db, id).await {
            Ok(p) => {
                let data = serde_json::to_value(&p)
                    .map_err(|e| RouterError::SerializeError(e.to_string()))?;
                self.reply(ctx, ok_response(data)).await
            }
            Err(e) => self.reply(ctx, error_response(&e)).await,
        }
    }

    async fn clauses(&self, ctx: &mut Context) -> Result<(), RouterError> {
        // 阶段 0：条款随产品详情一起返回，独立条款接口后续按规划实现
        self.reply(ctx, json_envelope(40001, "条款接口未接入数据库（任务 #3）"))
            .await
    }

    async fn featured(&self, ctx: &mut Context) -> Result<(), RouterError> {
        // 阶段 0：精选产品位后续按规划实现
        self.reply(ctx, json_envelope(40001, "精选产品接口未接入数据库（任务 #3）"))
            .await
    }

    /// 将已构造好的响应写入 Context
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
