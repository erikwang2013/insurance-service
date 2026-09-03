//! 运营统计控制器（OPERATOR/ADMIN 全局汇总）
//!
//! 接线完成（lead，任务 #20）：controllers/mod.rs 已注册模块 / AppState.stats /
//! stats_handler；routes.rs 已注册 `POST /api/v1/admin/stats`（AdminOrOperator）。

use async_trait::async_trait;
use axum::http::StatusCode;
use axum::response::Response;

use bee_rust::bee_router::context::RouterError;
use bee_rust::bee_router::{Context, Controller};

use crate::db::Db;
use crate::services::stats_service::{StatsReq, StatsService};

use super::{error_response, json_envelope, ok_response, read_json};

/// 运营统计控制器（持有 StatsService，按请求路径分派动作）
pub struct StatsController {
    service: StatsService,
}

impl StatsController {
    pub fn new(db: Db) -> Self {
        Self {
            service: StatsService::new(db),
        }
    }

    async fn overview(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let req: StatsReq = match read_json(ctx).await {
            Ok(r) => r,
            Err(resp) => return self.reply(ctx, resp).await,
        };
        match self.service.overview(req).await {
            Ok(o) => {
                let data = serde_json::to_value(&o)
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
impl Controller for StatsController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let path = ctx.request.uri().path().to_string();
        let method = ctx.request.method().clone();
        if method == axum::http::Method::POST && path.ends_with("/admin/stats") {
            self.overview(ctx).await
        } else {
            ctx.abort(StatusCode::NOT_FOUND, "接口不存在");
            Ok(())
        }
    }
}
