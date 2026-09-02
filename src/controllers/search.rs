//! 搜索控制器：全文搜索

use async_trait::async_trait;
use axum::http::StatusCode;
use axum::response::Response;

use bee_rust::bee_router::context::RouterError;
use bee_rust::bee_router::{Context, Controller};

use crate::db::Db;
use crate::services::search_service;

use super::{error_response, json_envelope, ok_response, query_map};

/// 搜索控制器（阶段 0 未接入 rust-scout，走服务层 MySQL 降级检索）
pub struct SearchController {
    db: Db,
}

impl SearchController {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    async fn search(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let q = query_map(ctx.request.uri().query());
        let keyword = q.get("keyword").cloned().unwrap_or_default();
        let type_ = q.get("type").cloned();
        let page = q.get("page").and_then(|s| s.parse().ok()).unwrap_or(1);
        let size = q.get("size").and_then(|s| s.parse().ok()).unwrap_or(20);
        if keyword.is_empty() {
            return self
                .reply(ctx, json_envelope(40000, "搜索关键字 keyword 不能为空"))
                .await;
        }
        match search_service::search(&self.db, &keyword, type_.as_deref(), page, size).await {
            Ok(res) => {
                let data = serde_json::to_value(&res)
                    .map_err(|e| RouterError::SerializeError(e.to_string()))?;
                self.reply(ctx, ok_response(data)).await
            }
            Err(e) => self.reply(ctx, error_response(&e)).await,
        }
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
impl Controller for SearchController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let path = ctx.request.uri().path().to_string();
        if path.ends_with("/search") {
            self.search(ctx).await
        } else {
            ctx.abort(StatusCode::NOT_FOUND, "接口不存在");
            Ok(())
        }
    }
}
