//! SyncWorker：DB → OpenSearch 后台同步（对齐 backend-architecture.md §10.3 / db-schema.md §9）
//!
//! 说明：真实实现依赖 bee_orm（MySQL `FOR UPDATE SKIP LOCKED` 领取任务）与 rust-scout
//! Engine。阶段 0 以 `SyncRepo` / `SearchEngine` 抽象承载接口，`run` 循环骨架完整，
//! 具体 DB/引擎调用在依赖可拉取后替换 `todo!()` 为真实实现。

use std::time::Duration;

use tokio::time;

use crate::error::{AppError, Result};
use crate::models::SearchSyncLog;

/// 同步队列仓储抽象（bee_orm 实现见阶段 1）
#[async_trait::async_trait]
pub trait SyncRepo: Send + Sync {
    /// 领取待处理任务（PENDING/FAILED/RETRYING 且 next_retry_at <= NOW，LIMIT N）
    async fn claim_pending(&self, limit: u32) -> Result<Vec<SearchSyncLog>>;
    /// 标记成功
    async fn mark_success(&self, id: i64) -> Result<()>;
    /// 标记重试（指数退避）
    async fn mark_retry(&self, id: i64, err: &str) -> Result<()>;
    /// 标记死信
    async fn mark_dead(&self, id: i64, err: &str) -> Result<()>;
}

/// 搜索索引引擎抽象（rust-scout Engine 的骨架对应）
#[async_trait::async_trait]
pub trait SearchEngine: Send + Sync {
    /// 写入/更新索引文档（UPSERT，_id = doc_id，幂等）
    async fn update(&self, index: &str, doc_id: &str, doc: serde_json::Value) -> Result<()>;
    /// 删除索引文档（幂等）
    async fn delete(&self, index: &str, doc_id: &str) -> Result<()>;
}

/// 后台同步 Worker
pub struct SyncWorker<R: SyncRepo, E: SearchEngine> {
    pub repo: R,
    pub engine: E,
    pub batch_size: u32,
    pub poll_interval: Duration,
}

impl<R: SyncRepo, E: SearchEngine> SyncWorker<R, E> {
    pub fn new(repo: R, engine: E) -> Self {
        Self {
            repo,
            engine,
            batch_size: 50,
            poll_interval: Duration::from_secs(1),
        }
    }

    /// 主循环：轮询待处理任务 → 处理 → 标记结果
    pub async fn run(&self) {
        loop {
            match self.process_batch().await {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!(error = %e, "sync worker batch failed");
                }
            }
            time::sleep(self.poll_interval).await;
        }
    }

    async fn process_batch(&self) -> Result<()> {
        let rows = self.repo.claim_pending(self.batch_size).await?;
        for row in rows {
            if let Err(e) = self.process_one(&row).await {
                tracing::error!(log_id = row.id, error = %e, "process sync log failed");
                self.handle_failure(&row, &e.to_string()).await?;
            }
        }
        Ok(())
    }

    async fn process_one(&self, row: &SearchSyncLog) -> Result<()> {
        // 文档快照优先，缺失时重放 payload 或重新加载主表（此处按快照重放）
        let index = row.entity_type.to_lowercase() + "s";
        match row.op.as_str() {
            SearchSyncLog::OP_UPSERT => {
                let doc = row
                    .payload_json
                    .clone()
                    .ok_or_else(|| AppError::Business("UPSERT 缺少 payload_json".into()))?;
                self.engine
                    .update(&index, &row.entity_id.to_string(), doc)
                    .await?;
                self.repo.mark_success(row.id).await?;
            }
            SearchSyncLog::OP_DELETE => {
                self.engine
                    .delete(&index, &row.entity_id.to_string())
                    .await?;
                self.repo.mark_success(row.id).await?;
            }
            other => {
                return Err(AppError::Business(format!("未知同步操作 {other}")));
            }
        }
        Ok(())
    }

    async fn handle_failure(&self, row: &SearchSyncLog, err: &str) -> Result<()> {
        if row.attempts + 1 >= row.max_attempts {
            self.repo.mark_dead(row.id, err).await
        } else {
            self.repo.mark_retry(row.id, err).await
        }
    }
}
