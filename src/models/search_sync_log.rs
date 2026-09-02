//! search_sync_logs DB→OpenSearch 同步队列（db-schema.md §6.16 / §9）

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// DB→OpenSearch 同步队列
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSyncLog {
    pub id: i64,
    /// 业务实体类型："PRODUCT"|"CLAUSE"|"POLICY"
    pub entity_type: String,
    /// 业务实体主键
    pub entity_id: i64,
    /// 操作："UPSERT"|"DELETE"
    pub op: String,
    /// 状态：PENDING → PROCESSING → SUCCESS / FAILED → RETRYING → SUCCESS/DEAD
    pub status: String,
    /// 已重试次数
    pub attempts: i32,
    pub max_attempts: i32,
    /// 下次重试时间（指数退避）
    pub next_retry_at: Option<DateTime<Utc>>,
    /// 最近一次失败原因
    pub last_error: Option<String>,
    /// 待写入索引文档快照（幂等重放）
    pub payload_json: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
}

impl SearchSyncLog {
    pub const OP_UPSERT: &'static str = "UPSERT";
    pub const OP_DELETE: &'static str = "DELETE";
    pub const STATUS_PENDING: &'static str = "PENDING";
    pub const STATUS_PROCESSING: &'static str = "PROCESSING";
    pub const STATUS_SUCCESS: &'static str = "SUCCESS";
    pub const STATUS_FAILED: &'static str = "FAILED";
    pub const STATUS_RETRYING: &'static str = "RETRYING";
    pub const STATUS_DEAD: &'static str = "DEAD";
}
