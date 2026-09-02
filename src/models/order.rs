//! orders 订单（db-schema.md §6.9）

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 订单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: i64,
    /// 订单号（唯一）
    pub order_no: String,
    pub quote_id: i64,
    /// 下单人
    pub user_id: i64,
    pub product_id: i64,
    /// 产品名快照
    pub product_name: String,
    /// 被保人快照
    pub holder_name: String,
    pub insurance_amount: Decimal,
    pub term_months: i32,
    /// 应付总额
    pub total_amount: Decimal,
    pub discount_amount: Decimal,
    /// 实付（应付 - 优惠）
    pub payable_amount: Decimal,
    pub currency: String,
    /// 状态：CREATED → PAID → POLICY_ISSUED → COMPLETED
    ///        └→ CANCELLED / EXPIRED / REFUNDING → REFUNDED
    pub status: String,
    pub paid_at: Option<DateTime<Utc>>,
    pub policy_issued_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub remark: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Order {
    pub const STATUS_CREATED: &'static str = "CREATED";
    pub const STATUS_PAID: &'static str = "PAID";
    pub const STATUS_POLICY_ISSUED: &'static str = "POLICY_ISSUED";
    pub const STATUS_COMPLETED: &'static str = "COMPLETED";
    pub const STATUS_CANCELLED: &'static str = "CANCELLED";
    pub const STATUS_EXPIRED: &'static str = "EXPIRED";
    pub const STATUS_REFUNDING: &'static str = "REFUNDING";
    pub const STATUS_REFUNDED: &'static str = "REFUNDED";
}
