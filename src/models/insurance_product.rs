//! insurance_products 保险产品（db-schema.md §6.3）

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 保险产品
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsuranceProduct {
    pub id: i64,
    /// 产品编码（对外唯一）
    pub product_code: String,
    pub name: String,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    /// 产品类型："LIFE"|"HEALTH"|"ACCIDENT"|"TRAVEL"|"PROPERTY"
    pub product_type: String,
    /// 销售渠道："ONLINE"|"AGENT"|"BROKER"|"OFFLINE"
    pub sale_channel: String,
    /// 运营/销售方用户
    pub operator_user_id: Option<i64>,
    /// 承保保险公司名称
    pub insurer_name: Option<String>,
    pub currency: String,
    pub min_amount: Option<Decimal>,
    pub max_amount: Option<Decimal>,
    pub min_term_months: Option<i32>,
    pub max_term_months: Option<i32>,
    pub waiting_period_days: Option<i32>,
    /// 首页推荐
    pub is_featured: bool,
    pub cover_image_url: Option<String>,
    /// 状态："DRAFT"|"ON_SALE"|"OFF_SHELF"|"DISCONTINUED"
    pub status: String,
    /// 是否入 OpenSearch
    pub search_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl InsuranceProduct {
    pub const STATUS_DRAFT: &'static str = "DRAFT";
    pub const STATUS_ON_SALE: &'static str = "ON_SALE";
    pub const STATUS_OFF_SHELF: &'static str = "OFF_SHELF";
    pub const STATUS_DISCONTINUED: &'static str = "DISCONTINUED";
}
