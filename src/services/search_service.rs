//! 搜索门面 SearchService（对齐 backend-architecture.md §10.1）
//!
//! 业务 Controller 通过 SearchService 访问搜索，不直接接触 rust-scout 底层。
//!
//! 说明：规划文档引用 `rust_scout::{Engine, EngineManager, ScoutConfig, SearchBuilder}`。
//! crates.io 当前可见最新为 0.1.0，且 OpenSearch 未运行（Cargo.toml 注释说明）。阶段 0
//! 以「降级实现」提供可用搜索：引擎不可用时回退 MySQL 模糊检索（LIKE），返回与规划
//! 一致的 `SearchResult` 结构；依赖可用后再在 `search()` 内替换为 rust-scout 引擎调用。

use mysql_async::prelude::FromRow;
use mysql_async::Row;
use serde::{Deserialize, Serialize};

use crate::db::Db;
use crate::error::{AppError, Result};
use crate::models::InsuranceProduct;
use crate::search::Searchable;

/// 单条搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub index: String,
    pub doc_id: String,
    pub doc: serde_json::Value,
}

/// 搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub hits: Vec<SearchHit>,
    pub total: u64,
    pub page: u32,
    pub size: u32,
}

/// 搜索服务门面（阶段 0 持 Db 走降级检索；rust-scout 接入后换引擎字段）
pub struct SearchService {
    db: Db,
}

impl SearchService {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// 全文搜索：keyword 命中指定索引（阶段 0 降级：MySQL LIKE 模糊检索）
    pub async fn search(
        &self,
        index: &str,
        keyword: &str,
        status: &str,
        page: u32,
        size: u32,
    ) -> Result<SearchResult> {
        match index {
            "insurance_products" | "product" | "" => {
                self.search_products(keyword, status, page, size).await
            }
            other => Err(AppError::Search(format!(
                "索引 {other} 尚未接入降级搜索（阶段 0 仅支持保险产品）"
            ))),
        }
    }

    /// 降级实现：对 insurance_products 做参数化 LIKE 检索（name/subtitle/description/
    /// insurer_name/product_code），值一律参数绑定，杜绝注入。
    async fn search_products(
        &self,
        keyword: &str,
        status: &str,
        page: u32,
        size: u32,
    ) -> Result<SearchResult> {
        let size = size.clamp(1, 100) as usize;
        let page_echo = page;
        let page = page.max(1) as usize;
        let offset = (page - 1) * size;
        let kw = format!("%{keyword}%");

        let mut where_sql = String::from(
            "deleted_at IS NULL AND (name LIKE ? OR subtitle LIKE ? \
             OR description LIKE ? OR insurer_name LIKE ? OR product_code LIKE ?)",
        );
        let mut params: Vec<String> =
            vec![kw.clone(), kw.clone(), kw.clone(), kw.clone(), kw.clone()];
        if !status.is_empty() {
            where_sql.push_str(" AND status = ?");
            params.push(status.to_string());
        }

        let total: u64 = self
            .db
            .query_one::<CountRow>(
                &format!("SELECT COUNT(*) FROM insurance_products WHERE {where_sql}"),
                params.clone(),
            )
            .await?
            .map(|c| c.0)
            .unwrap_or_default();

        let mut q_params = params;
        q_params.push(size.to_string());
        q_params.push(offset.to_string());
        let sql = format!(
            "SELECT * FROM insurance_products WHERE {where_sql} \
             ORDER BY is_featured DESC, id DESC LIMIT ? OFFSET ?"
        );
        let products: Vec<InsuranceProduct> = self.db.query_all(&sql, q_params).await?;
        let hits = products
            .into_iter()
            .map(|p| SearchHit {
                index: "insurance_products".to_string(),
                doc_id: p.doc_id(),
                doc: p.to_doc(),
            })
            .collect();

        Ok(SearchResult {
            hits,
            total,
            page: page_echo,
            size: size as u32,
        })
    }
}

impl Default for SearchService {
    fn default() -> Self {
        // 仅供内部测试/占位；业务入口经 AppState 注入真实 Db
        unreachable!("SearchService 必须经 AppState 注入 Db")
    }
}

/// COUNT(*) 首列读取辅助
struct CountRow(u64);

impl FromRow for CountRow {
    fn from_row_opt(row: Row) -> std::result::Result<Self, mysql_async::FromRowError> {
        Ok(CountRow(row.get(0).unwrap_or_default()))
    }
}

/// 类型化多索引搜索（backend-architecture.md §11 SearchController）
pub async fn search(
    db: &Db,
    keyword: &str,
    type_: Option<&str>,
    page: u32,
    size: u32,
) -> Result<SearchResult> {
    let index = type_.unwrap_or("insurance_products");
    SearchService::new(db.clone())
        .search(index, keyword, "", page, size)
        .await
}
