//! insurance_product_categories 产品分类（树）（db-schema.md §6.5）

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 产品分类（parent_id 自关联形成树）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsuranceProductCategory {
    pub id: i64,
    /// 父分类（根为 NULL）
    pub parent_id: Option<i64>,
    pub name: String,
    /// URL 友好标识（唯一）
    pub slug: String,
    pub sort_order: i32,
    /// 状态："ACTIVE"|"HIDDEN"
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}
