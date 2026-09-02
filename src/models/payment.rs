//! payments 支付流水（db-schema.md §6.10）

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 支付流水
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: i64,
    /// 支付流水号（唯一）
    pub payment_no: String,
    pub order_id: i64,
    pub user_id: i64,
    pub amount: Decimal,
    pub currency: String,
    /// 渠道："WECHAT"|"ALIPAY"|"UNIONPAY"|"BALANCE"|"MOCK"
    pub channel: String,
    /// PayProvider 实现名："MOCK"|"WECHAT"
    pub provider: String,
    /// 支付渠道交易号
    pub provider_tx_id: Option<String>,
    /// 状态：CREATED → PROCESSING → SUCCESS / FAILED / CANCELLED / REFUNDED
    pub status: String,
    /// 预支付参数（前端拉起收银台）
    pub prepay_payload: Option<serde_json::Value>,
    /// 渠道回调原始报文（审计留痕）
    pub callback_payload: Option<serde_json::Value>,
    pub paid_at: Option<DateTime<Utc>>,
    pub refunded_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Payment {
    pub const STATUS_CREATED: &'static str = "CREATED";
    pub const STATUS_PROCESSING: &'static str = "PROCESSING";
    pub const STATUS_SUCCESS: &'static str = "SUCCESS";
    pub const STATUS_FAILED: &'static str = "FAILED";
    pub const STATUS_CANCELLED: &'static str = "CANCELLED";
    pub const STATUS_REFUNDED: &'static str = "REFUNDED";
    pub const CHANNEL_WECHAT: &'static str = "WECHAT";
    pub const CHANNEL_ALIPAY: &'static str = "ALIPAY";
    pub const CHANNEL_MOCK: &'static str = "MOCK";
}
