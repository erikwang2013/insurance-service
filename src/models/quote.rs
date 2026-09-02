//! quotes 报价 / 投保方案（db-schema.md §6.7）

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 报价 / 投保方案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub id: i64,
    /// 报价单号（唯一）
    pub quote_no: String,
    pub product_id: i64,
    /// 投保人账户
    pub user_id: i64,
    /// 被保人档案（可空，内联信息）
    pub holder_id: Option<i64>,
    pub holder_name: Option<String>,
    /// 被保人身份证密文（不对外序列化）
    #[serde(skip_serializing)]
    pub holder_id_card_enc: Option<Vec<u8>>,
    /// 保额
    pub insurance_amount: Decimal,
    /// 保障期（月）
    pub term_months: i32,
    /// 试算保费
    pub premium: Decimal,
    /// 保费构成明细 {base, extra, discount, total}
    pub premium_detail: Option<serde_json::Value>,
    pub effective_date: Option<NaiveDate>,
    pub expire_date: Option<NaiveDate>,
    /// 健康告知问卷答案
    pub health_declaration: Option<serde_json::Value>,
    /// 核保风险分（0-100）
    pub risk_score: Option<i32>,
    /// 状态："PENDING"|"APPROVED"|"REJECTED"|"EXPIRED"|"CONVERTED"|"CANCELLED"
    pub status: String,
    /// 报价有效期
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Quote {
    pub const STATUS_PENDING: &'static str = "PENDING";
    pub const STATUS_APPROVED: &'static str = "APPROVED";
    pub const STATUS_REJECTED: &'static str = "REJECTED";
    pub const STATUS_EXPIRED: &'static str = "EXPIRED";
    pub const STATUS_CONVERTED: &'static str = "CONVERTED";
    pub const STATUS_CANCELLED: &'static str = "CANCELLED";
}
