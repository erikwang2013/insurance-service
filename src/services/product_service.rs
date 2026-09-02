//! 产品服务（对齐 backend-architecture.md §11 ProductController 依赖）
//!
//! 基于 db 层执行参数化查询；SQL 仅由受信常量拼接，值一律经参数绑定（任务 #3）。

use crate::db::Db;
use crate::error::{AppError, Result};
use crate::models::InsuranceProduct;

/// 产品列表（分页/筛选）；`status` 为空串时不过滤状态
pub async fn list(db: &Db, status: &str, page: u32, size: u32) -> Result<Vec<InsuranceProduct>> {
    let size = size.clamp(1, 100) as usize;
    let offset = ((page.max(1) as usize) - 1) * size;
    let sql = if status.is_empty() {
        "SELECT * FROM insurance_products \
         WHERE deleted_at IS NULL ORDER BY id DESC LIMIT ? OFFSET ?"
    } else {
        "SELECT * FROM insurance_products \
         WHERE deleted_at IS NULL AND status = ? ORDER BY id DESC LIMIT ? OFFSET ?"
    };
    let mut params: Vec<String> = Vec::new();
    if !status.is_empty() {
        params.push(status.to_string());
    }
    params.push(size.to_string());
    params.push(offset.to_string());
    db.query_all(sql, params).await
}

/// 产品详情 + 条款（条款字段随产品一起返回；独立条款接口后续按规划实现）
pub async fn detail(db: &Db, id: i64) -> Result<InsuranceProduct> {
    db.query_one(
        "SELECT * FROM insurance_products WHERE id = ? AND deleted_at IS NULL LIMIT 1",
        vec![id],
    )
    .await?
    .ok_or(AppError::NotFound)
}
