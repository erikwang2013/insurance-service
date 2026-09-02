//! insurance_product_category_rel 产品-分类 多对多（db-schema.md §6.6）

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 产品-分类多对多关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsuranceProductCategoryRel {
    pub id: i64,
    pub product_id: i64,
    pub category_id: i64,
    pub created_at: DateTime<Utc>,
}
