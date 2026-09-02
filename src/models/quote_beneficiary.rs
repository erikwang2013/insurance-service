//! quotes_beneficiaries 报价期受益人快照（db-schema.md §6.8）

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 报价期受益人快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteBeneficiary {
    pub id: i64,
    pub quote_id: i64,
    pub name: String,
    #[serde(skip_serializing)]
    pub id_card_enc: Option<Vec<u8>>,
    /// 关系："SELF"|"SPOUSE"|"CHILD"|"PARENT"|"OTHER"
    pub relationship: Option<String>,
    /// 受益人类型："LEGAL"(法定)|"NAMED"(指定)
    pub beneficiary_type: String,
    /// 占比（0-100），指定受益人时使用，合计 100
    pub share_percent: Option<Decimal>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}
