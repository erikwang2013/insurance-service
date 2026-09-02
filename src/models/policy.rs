//! policies 保单（db-schema.md §6.11）

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 保单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: i64,
    /// 保单号（对外展示）
    pub policy_no: String,
    pub order_id: i64,
    pub quote_id: i64,
    /// 投保人
    pub user_id: i64,
    /// 被保人档案
    pub holder_id: Option<i64>,
    pub product_id: i64,
    pub product_name: String,
    /// 被保人姓名
    pub holder_name: String,
    /// 被保人身份证密文（不对外序列化）
    #[serde(skip_serializing)]
    pub holder_id_card_enc: Option<Vec<u8>>,
    /// 保额
    pub insurance_amount: Decimal,
    /// 实缴保费
    pub premium: Decimal,
    pub term_months: i32,
    /// 保险起期
    pub effective_date: NaiveDate,
    /// 保险止期
    pub expire_date: NaiveDate,
    /// 状态：PENDING_ISSUE → ACTIVE → EXPIRED / CANCELLED / SURRENDERED / LAPSED
    pub status: String,
    /// 签发类型："NEW"|"RENEW"
    pub issue_type: String,
    pub is_renewable: bool,
    /// 电子保单 PDF 存储路径
    pub pdf_path: Option<String>,
    pub premium_detail: Option<serde_json::Value>,
    pub issued_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Policy {
    pub const STATUS_PENDING_ISSUE: &'static str = "PENDING_ISSUE";
    pub const STATUS_ACTIVE: &'static str = "ACTIVE";
    pub const STATUS_EXPIRED: &'static str = "EXPIRED";
    pub const STATUS_CANCELLED: &'static str = "CANCELLED";
    pub const STATUS_SURRENDERED: &'static str = "SURRENDERED";
    pub const STATUS_LAPSED: &'static str = "LAPSED";
}
