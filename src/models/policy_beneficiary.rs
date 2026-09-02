//! policy_beneficiaries 保单受益人（占比）（db-schema.md §6.12）

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 保单受益人
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyBeneficiary {
    pub id: i64,
    pub policy_id: i64,
    pub name: String,
    #[serde(skip_serializing)]
    pub id_card_enc: Option<Vec<u8>>,
    /// 关系："SELF"|"SPOUSE"|"CHILD"|"PARENT"|"OTHER"
    pub relationship: Option<String>,
    /// 受益人类型："LEGAL"|"NAMED"
    pub beneficiary_type: String,
    /// 占比（0-100），NAMED 时使用，同单合计 = 100
    pub share_percent: Option<Decimal>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
