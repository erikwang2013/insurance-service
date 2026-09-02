//! contracts 电子合同（db-schema.md §6.13）

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 电子合同
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub id: i64,
    /// 合同号（唯一）
    pub contract_no: String,
    /// 关联保单（唯一）
    pub policy_id: i64,
    pub order_id: i64,
    /// 合同标题
    pub title: String,
    /// 合同类型："POLICY"|"ENDORSEMENT"|"RIDER"
    pub contract_type: String,
    /// 最终合同 PDF
    pub pdf_path: Option<String>,
    /// 合同 PDF 防篡改摘要（SHA-256）
    pub file_hash: Option<String>,
    /// 电子签服务端流程 ID（预留 e签宝）
    pub sign_flow_id: Option<String>,
    /// ElectronicSignature 实现名："MOCK"|"ESIGN"
    pub provider: String,
    /// 状态：DRAFT → PENDING_SIGN → SIGNING → COMPLETED / VOID / EXPIRED / REJECTED
    pub status: String,
    pub signed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Contract {
    pub const STATUS_DRAFT: &'static str = "DRAFT";
    pub const STATUS_PENDING_SIGN: &'static str = "PENDING_SIGN";
    pub const STATUS_SIGNING: &'static str = "SIGNING";
    pub const STATUS_COMPLETED: &'static str = "COMPLETED";
    pub const STATUS_VOID: &'static str = "VOID";
    pub const STATUS_EXPIRED: &'static str = "EXPIRED";
    pub const STATUS_REJECTED: &'static str = "REJECTED";
}
