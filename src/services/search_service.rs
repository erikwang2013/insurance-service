//! 搜索门面 SearchService（对齐 backend-architecture.md §10.1）
//!
//! 业务 Controller 通过 SearchService 访问搜索。当前实现为 MySQL 模糊检索（LIKE），
//! 返回与规划一致的 `SearchResult` 结构。

use mysql_async::prelude::{FromRow, Queryable};
use mysql_async::Row;
use serde::{Deserialize, Serialize};

use crate::db::{db_error, Db};
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
        page: u32,
        size: u32,
    ) -> Result<SearchResult> {
        match index {
            "insurance_products" | "product" | "" => {
                self.search_products(keyword, page, size).await
            }
            other => Err(AppError::Search(format!(
                "索引 {other} 尚未接入降级搜索（阶段 0 仅支持保险产品）"
            ))),
        }
    }

    /// 降级实现：对 insurance_products 做参数化 LIKE 检索（name/subtitle/description/
    /// insurer_name/product_code）。仅检索在售且开启搜索的产品；关键字内 LIKE 元字符
    /// 已转义（SQL 显式 `ESCAPE '\'`），值一律参数绑定，杜绝注入。
    async fn search_products(
        &self,
        keyword: &str,
        page: u32,
        size: u32,
    ) -> Result<SearchResult> {
        let size = size.clamp(1, 100) as usize;
        let page_echo = page;
        let page = page.max(1) as usize;
        let offset = (page - 1) * size;
        let (where_sql, params) = product_search_where(keyword);

        // 窗口函数一次查询带回总数：首行 `_total` 即过滤后总数（省掉独立 COUNT）
        let mut page_params = params.clone();
        page_params.push(size.to_string());
        page_params.push(offset.to_string());
        let sql = format!(
            "SELECT *, COUNT(*) OVER() AS _total FROM insurance_products \
             WHERE {where_sql} ORDER BY is_featured DESC, id DESC LIMIT ? OFFSET ?"
        );
        let mut conn = self.db.conn().await?;
        let rows: Vec<Row> = conn.exec(&sql, page_params).await.map_err(db_error)?;

        let total = match rows.first().and_then(|r| r.get::<u64, _>("_total")) {
            Some(total) => total,
            // 空页非首页时窗口函数无行可携带总数，补一次 COUNT 保持 total 正确
            None if offset > 0 => {
                let count: Option<CountRow> = conn
                    .exec_first(
                        format!("SELECT COUNT(*) FROM insurance_products WHERE {where_sql}"),
                        params,
                    )
                    .await
                    .map_err(db_error)?;
                count.map(|c| c.0).unwrap_or_default()
            }
            None => 0,
        };

        // 行内多出 `_total` 列，按列名读取的 FromRow 会忽略它
        let products = rows
            .into_iter()
            .map(InsuranceProduct::from_row_opt)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| AppError::db(format!("搜索结果行映射失败: {e}")))?;
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

/// 产品搜索 WHERE 子句与参数：固定只检索在售且开启搜索的产品。
/// 关键字内 LIKE 元字符经 `escape_like` 转义，SQL 显式 `ESCAPE '\'`。
fn product_search_where(keyword: &str) -> (String, Vec<String>) {
    const WHERE: &str = concat!(
        "deleted_at IS NULL AND status = 'ON_SALE' AND search_enabled = 1 ",
        r"AND (name LIKE ? ESCAPE '\\' OR subtitle LIKE ? ESCAPE '\\' ",
        r"OR description LIKE ? ESCAPE '\\' OR insurer_name LIKE ? ESCAPE '\\' ",
        r"OR product_code LIKE ? ESCAPE '\\')",
    );
    (WHERE.to_string(), vec![format!("%{}%", escape_like(keyword)); 5])
}

/// 转义 LIKE 元字符（`\`、`%`、`_`），与 SQL 的 `ESCAPE '\'` 配套。
fn escape_like(keyword: &str) -> String {
    let mut out = String::with_capacity(keyword.len());
    for c in keyword.chars() {
        if matches!(c, '\\' | '%' | '_') {
            out.push('\\');
        }
        out.push(c);
    }
    out
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
        .search(index, keyword, page, size)
        .await
}

#[cfg(test)]
mod tests {
    use super::{escape_like, product_search_where};

    /// LIKE 元字符全部转义：反斜杠先自身转义，%/_ 前置反斜杠
    #[test]
    fn escapes_like_metacharacters() {
        assert_eq!(escape_like("100%"), r"100\%");
        assert_eq!(escape_like("a_b"), r"a\_b");
        assert_eq!(escape_like(r"c:\d"), r"c:\\d");
        assert_eq!(escape_like(r"50%\_off"), r"50\%\\\_off");
        assert_eq!(escape_like("保险"), "保险");
    }

    /// 搜索 WHERE 固定限制在售 + 入搜索，且每个 LIKE 均带 ESCAPE
    #[test]
    fn product_search_where_restricts_visibility() {
        let (sql, params) = product_search_where("保险");
        assert!(sql.contains("status = 'ON_SALE'"), "{sql}");
        assert!(sql.contains("search_enabled = 1"), "{sql}");
        assert_eq!(sql.matches("ESCAPE '\\\\'").count(), 5, "{sql}");
        assert_eq!(params, vec!["%保险%"; 5]);
    }
}
