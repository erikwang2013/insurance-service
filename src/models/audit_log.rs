//! audit_logs 操作审计（db-schema.md §6.17）

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 操作审计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: i64,
    /// 操作人
    pub user_id: Option<i64>,
    /// 动作："ORDER_PAY"|"POLICY_ISSUE"|"CONTRACT_SIGN"|...
    pub action: String,
    /// 实体类型："ORDER"|"POLICY"|"CONTRACT"|"PAYMENT"|...
    pub entity_type: String,
    pub entity_id: i64,
    /// 变更前快照
    pub before_json: Option<serde_json::Value>,
    /// 变更后快照
    pub after_json: Option<serde_json::Value>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    /// 与响应 ResponseEnvelope.trace_id 对齐
    pub trace_id: Option<String>,
    pub created_at: DateTime<Utc>,
}
