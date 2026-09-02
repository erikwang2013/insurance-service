//! insurance_product_clauses 产品条款（db-schema.md §6.4）

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 产品条款
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsuranceProductClause {
    pub id: i64,
    /// 关联产品
    pub product_id: i64,
    /// 条款类型："MAIN"|"EXCLUSION"|"WAIVER"|"RIDER"|"OBLIGATION"
    pub clause_type: String,
    pub title: String,
    /// 条款正文（Markdown/HTML）
    pub content: String,
    pub sort_order: i32,
    /// 是否必须勾选阅读
    pub is_required: bool,
    pub version: String,
    /// 状态："ACTIVE"|"DEPRECATED"
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}
