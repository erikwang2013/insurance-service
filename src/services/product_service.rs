//! 产品服务（对齐 backend-architecture.md §11 ProductController 依赖）
//!
//! 基于 db 层执行参数化查询；SQL 仅由受信常量拼接，值一律经参数绑定（任务 #3）。
//! 运营侧建档/上下架（任务 #7）也落于此文件：与公开 list/detail 共用 Db 与模型。

use chrono::{DateTime, NaiveDateTime, Utc};
use mysql_async::prelude::Queryable;
use mysql_async::{Row, Value};
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::db::{db_error, Db};
use crate::error::{AppError, Result};
use crate::models::user::User;
use crate::models::{InsuranceProduct, InsuranceProductClause};

/// 公开产品列表（分页）：只返回在售产品，客户端传入的 status 不参与查询。
pub async fn list(db: &Db, page: u32, size: u32) -> Result<Vec<InsuranceProduct>> {
    let (sql, params) = list_query(page, size);
    db.query_all(sql, params).await
}

/// 公开列表 SQL/参数（仅在售 + 未软删）；独立成纯函数便于无库测试。
fn list_query(page: u32, size: u32) -> (&'static str, Vec<String>) {
    let size = size.clamp(1, 100) as usize;
    let offset = ((page.max(1) as usize) - 1) * size;
    (
        "SELECT * FROM insurance_products \
         WHERE deleted_at IS NULL AND status = ? ORDER BY id DESC LIMIT ? OFFSET ?",
        vec![
            InsuranceProduct::STATUS_ON_SALE.to_string(),
            size.to_string(),
            offset.to_string(),
        ],
    )
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

/// 公开产品详情：非在售（DRAFT/OFF_SHELF/DISCONTINUED）一律按不存在处理，
/// 运营态数据不经公开接口外泄；运营侧回读仍用 `detail`。
pub async fn public_detail(db: &Db, id: i64) -> Result<InsuranceProduct> {
    let product = detail(db, id).await?;
    if product.status == InsuranceProduct::STATUS_ON_SALE {
        Ok(product)
    } else {
        Err(AppError::NotFound)
    }
}

/// 产品条款列表（公开）：先确认产品在售（未软删），再查条款（未软删），
/// 按 sort_order 升序；产品不可见或该产品无条款 → NotFound。
pub async fn clauses(db: &Db, product_id: i64) -> Result<Vec<InsuranceProductClause>> {
    public_detail(db, product_id).await?;
    let mut conn = db.conn().await?;
    let rows: Vec<Row> = conn
        .exec(
            "SELECT * FROM insurance_product_clauses \
             WHERE product_id = ? AND deleted_at IS NULL \
             ORDER BY sort_order ASC, id ASC",
            vec![product_id],
        )
        .await
        .map_err(db_error)?;
    let items: Vec<InsuranceProductClause> = rows.iter().map(row_to_clause).collect();
    if items.is_empty() {
        return Err(AppError::NotFound);
    }
    Ok(items)
}

/// 首页精选（公开）：is_featured=1 且 ON_SALE 且未软删；分页/limit 语义同 list
pub async fn featured(db: &Db, page: u32, size: u32) -> Result<Vec<InsuranceProduct>> {
    let size = size.clamp(1, 100) as usize;
    let offset = ((page.max(1) as usize) - 1) * size;
    db.query_all(
        "SELECT * FROM insurance_products \
         WHERE deleted_at IS NULL AND status = ? AND is_featured = 1 \
         ORDER BY id DESC LIMIT ? OFFSET ?",
        vec![
            InsuranceProduct::STATUS_ON_SALE.to_string(),
            size.to_string(),
            offset.to_string(),
        ],
    )
    .await
}

// ---------- 运营侧（admin，任务 #7）----------

/// 运营侧商品列表：可按状态筛选（空串 = 全部），未软删，按 id 倒序分页。
pub async fn admin_list(
    db: &Db,
    status: &str,
    page: u32,
    size: u32,
) -> Result<Vec<InsuranceProduct>> {
    if !admin_status_allowed(status) {
        return Err(AppError::business(
            "status 仅支持 DRAFT / ON_SALE / OFF_SHELF / DISCONTINUED",
        ));
    }
    let (sql, params) = admin_list_query(status, page, size);
    db.query_all(sql, params).await
}

/// 运营列表状态白名单（空串表示不过滤）。
fn admin_status_allowed(status: &str) -> bool {
    status.is_empty()
        || [
            InsuranceProduct::STATUS_DRAFT,
            InsuranceProduct::STATUS_ON_SALE,
            InsuranceProduct::STATUS_OFF_SHELF,
            InsuranceProduct::STATUS_DISCONTINUED,
        ]
        .contains(&status)
}

/// 运营列表 SQL/参数；独立成纯函数便于无库测试。
fn admin_list_query(status: &str, page: u32, size: u32) -> (&'static str, Vec<String>) {
    let size = size.clamp(1, 100) as usize;
    let offset = ((page.max(1) as usize) - 1) * size;
    if status.is_empty() {
        (
            "SELECT * FROM insurance_products \
             WHERE deleted_at IS NULL ORDER BY id DESC LIMIT ? OFFSET ?",
            vec![size.to_string(), offset.to_string()],
        )
    } else {
        (
            "SELECT * FROM insurance_products \
             WHERE deleted_at IS NULL AND status = ? ORDER BY id DESC LIMIT ? OFFSET ?",
            vec![status.to_string(), size.to_string(), offset.to_string()],
        )
    }
}

/// 商品建档/更新请求体（同 product_code → UPDATE，否则 INSERT；status 缺省 DRAFT）
#[derive(Debug, Deserialize)]
pub struct AdminUpsertReq {
    pub product_code: String,
    pub name: String,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub product_type: String,
    /// 缺省 ONLINE（与建库默认一致）
    #[serde(default)]
    pub sale_channel: Option<String>,
    pub insurer_name: Option<String>,
    /// 缺省 CNY（与建库默认一致）
    #[serde(default)]
    pub currency: Option<String>,
    pub min_amount: Option<Decimal>,
    pub max_amount: Option<Decimal>,
    pub min_term_months: Option<i32>,
    pub max_term_months: Option<i32>,
    pub waiting_period_days: Option<i32>,
    /// 缺省 false
    #[serde(default)]
    pub is_featured: Option<bool>,
    pub cover_image_url: Option<String>,
    /// 缺省 true（与建库默认一致）
    #[serde(default)]
    pub search_enabled: Option<bool>,
    #[serde(default)]
    pub status: Option<String>,
}

/// 商品上下架请求体
#[derive(Debug, Deserialize)]
pub struct AdminStatusReq {
    pub status: String,
}

/// 校验操作人为运营/管理员（users.role IN OPERATOR/ADMIN 且未软删），否则 Forbidden
async fn ensure_operator(db: &Db, user_id: i64) -> Result<()> {
    let role: Option<String> = db
        .conn()
        .await?
        .exec_first(
            "SELECT role FROM users WHERE id = ? AND deleted_at IS NULL LIMIT 1",
            vec![user_id],
        )
        .await
        .map_err(db_error)?;
    match role.as_deref() {
        Some(User::ROLE_OPERATOR) | Some(User::ROLE_ADMIN) => Ok(()),
        _ => Err(AppError::Forbidden),
    }
}

/// 商品建档/更新：同 product_code（未软删）→ UPDATE，否则 INSERT；成功后回读整行。
/// `operator_id` 取自已认证身份（控制器从 token 取），不信请求体。
pub async fn admin_upsert(db: &Db, operator_id: i64, req: &AdminUpsertReq) -> Result<InsuranceProduct> {
    ensure_operator(db, operator_id).await?;
    let status = req
        .status
        .clone()
        .unwrap_or_else(|| InsuranceProduct::STATUS_DRAFT.to_string());
    if ![
        InsuranceProduct::STATUS_DRAFT,
        InsuranceProduct::STATUS_ON_SALE,
        InsuranceProduct::STATUS_OFF_SHELF,
        InsuranceProduct::STATUS_DISCONTINUED,
    ]
    .contains(&status.as_str())
    {
        return Err(AppError::business(
            "status 仅支持 DRAFT / ON_SALE / OFF_SHELF / DISCONTINUED",
        ));
    }
    // 服务层补缺省（与建库默认一致），保持 INSERT/UPDATE 参数形态统一
    let sale_channel = req.sale_channel.clone().unwrap_or_else(|| "ONLINE".to_string());
    let currency = req.currency.clone().unwrap_or_else(|| "CNY".to_string());
    let is_featured = req.is_featured.unwrap_or(false);
    let search_enabled = req.search_enabled.unwrap_or(true);

    // 同 code 存在 → 更新；否则插入（uk_product_code 兜底唯一性）
    let existing: Option<String> = db
        .conn()
        .await?
        .exec_first(
            "SELECT status FROM insurance_products \
             WHERE product_code = ? AND deleted_at IS NULL LIMIT 1",
            vec![req.product_code.clone()],
        )
        .await
        .map_err(db_error)?;

    if existing.is_some() {
        let params: Vec<Value> = vec![
            Value::from(&req.name),
            value_opt_str(req.subtitle.clone()),
            value_opt_str(req.description.clone()),
            Value::from(&req.product_type),
            Value::from(&sale_channel),
            Value::from(operator_id),
            value_opt_str(req.insurer_name.clone()),
            Value::from(&currency),
            value_opt_str(req.min_amount.map(|d| d.to_string())),
            value_opt_str(req.max_amount.map(|d| d.to_string())),
            value_opt_str(req.min_term_months.map(|m| m.to_string())),
            value_opt_str(req.max_term_months.map(|m| m.to_string())),
            value_opt_str(req.waiting_period_days.map(|d| d.to_string())),
            Value::from(is_featured),
            value_opt_str(req.cover_image_url.clone()),
            Value::from(&status),
            Value::from(search_enabled),
            Value::from(req.product_code.clone()),
        ];
        db.exec_drop(
            "UPDATE insurance_products \
             SET name = ?, subtitle = ?, description = ?, product_type = ?, \
                 sale_channel = ?, operator_user_id = ?, insurer_name = ?, \
                 currency = ?, min_amount = ?, max_amount = ?, min_term_months = ?, \
                 max_term_months = ?, waiting_period_days = ?, is_featured = ?, \
                 cover_image_url = ?, status = ?, search_enabled = ?, updated_at = NOW() \
             WHERE product_code = ? AND deleted_at IS NULL",
            params,
        )
        .await?;
    } else {
        // 主键由应用层 snowflake 预生成后显式插入（全库自增迁移，见 idgen）
        let id = crate::utils::idgen::next_id();
        let params: Vec<Value> = vec![
            id.into(),
            Value::from(&req.product_code),
            Value::from(&req.name),
            value_opt_str(req.subtitle.clone()),
            value_opt_str(req.description.clone()),
            Value::from(&req.product_type),
            Value::from(&sale_channel),
            Value::from(operator_id),
            value_opt_str(req.insurer_name.clone()),
            Value::from(&currency),
            value_opt_str(req.min_amount.map(|d| d.to_string())),
            value_opt_str(req.max_amount.map(|d| d.to_string())),
            value_opt_str(req.min_term_months.map(|m| m.to_string())),
            value_opt_str(req.max_term_months.map(|m| m.to_string())),
            value_opt_str(req.waiting_period_days.map(|d| d.to_string())),
            Value::from(is_featured),
            value_opt_str(req.cover_image_url.clone()),
            Value::from(&status),
            Value::from(search_enabled),
        ];
        db.exec_drop(
            "INSERT INTO insurance_products \
             (id, product_code, name, subtitle, description, product_type, sale_channel, \
              operator_user_id, insurer_name, currency, min_amount, max_amount, \
              min_term_months, max_term_months, waiting_period_days, is_featured, \
              cover_image_url, status, search_enabled) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params,
        )
        .await?;
    }

    // 回读整行（同 code 唯一，写入后必然可见）
    db.query_one::<InsuranceProduct>(
        "SELECT * FROM insurance_products \
         WHERE product_code = ? AND deleted_at IS NULL LIMIT 1",
        vec![req.product_code.clone()],
    )
    .await?
    .ok_or_else(|| AppError::business("写入后回读失败"))
}

/// 商品上架/下架/停售：仅切换 status + operator_user_id（禁回 DRAFT 防呆）。
pub async fn admin_change_status(
    db: &Db,
    product_id: i64,
    operator_id: i64,
    req: &AdminStatusReq,
) -> Result<InsuranceProduct> {
    ensure_operator(db, operator_id).await?;
    let status = req.status.as_str();
    if ![
        InsuranceProduct::STATUS_ON_SALE,
        InsuranceProduct::STATUS_OFF_SHELF,
        InsuranceProduct::STATUS_DISCONTINUED,
    ]
    .contains(&status)
    {
        return Err(AppError::business(
            "status 仅支持 ON_SALE / OFF_SHELF / DISCONTINUED（禁用回 DRAFT）",
        ));
    }
    let affected = db
        .exec_drop(
            "UPDATE insurance_products \
             SET status = ?, operator_user_id = ?, updated_at = NOW() \
             WHERE id = ? AND deleted_at IS NULL",
            vec![
                Value::from(status.to_string()),
                Value::from(operator_id),
                Value::from(product_id),
            ],
        )
        .await?;
    if affected == 0 {
        return Err(AppError::NotFound);
    }
    detail(db, product_id).await
}

fn value_opt_str(v: Option<String>) -> Value {
    v.map(Value::from).unwrap_or(Value::NULL)
}

// ---------- helpers: Row → InsuranceProductClause（与 quote_service 同模式） ----------

/// 读行 → DateTime<Utc>（DATETIME(3) 二进制协议以 NaiveDateTime 到达）
fn dt_row(row: &Row, col: &str) -> DateTime<Utc> {
    row.get::<NaiveDateTime, &str>(col)
        .unwrap_or_default()
        .and_utc()
}

/// 读行 → Option<DateTime<Utc>>
fn dt_opt_row(row: &Row, col: &str) -> Option<DateTime<Utc>> {
    row.get::<Option<NaiveDateTime>, &str>(col)
        .flatten()
        .map(|d| d.and_utc())
}

/// 从 Row 重建条款（TINYINT(1) → bool 同 db.rs InsuranceProduct 写法）
fn row_to_clause(row: &Row) -> InsuranceProductClause {
    InsuranceProductClause {
        id: row.get("id").unwrap_or_default(),
        product_id: row.get("product_id").unwrap_or_default(),
        clause_type: row.get("clause_type").unwrap_or_default(),
        title: row.get("title").unwrap_or_default(),
        content: row.get("content").unwrap_or_default(),
        sort_order: row.get("sort_order").unwrap_or_default(),
        is_required: row.get("is_required").unwrap_or_default(),
        version: row.get("version").unwrap_or_default(),
        status: row.get("status").unwrap_or_default(),
        created_at: dt_row(row, "created_at"),
        updated_at: dt_row(row, "updated_at"),
        deleted_at: dt_opt_row(row, "deleted_at"),
    }
}

#[cfg(test)]
mod tests {
    use super::{admin_list_query, admin_status_allowed, list_query, InsuranceProduct};

    /// 公开列表始终按 ON_SALE 过滤，分页参数位于状态之后
    #[test]
    fn public_list_always_filters_on_sale() {
        let (sql, params) = list_query(2, 10);
        assert!(sql.contains("status = ?"), "{sql}");
        assert_eq!(params, vec!["ON_SALE", "10", "10"]);
    }

    /// 运营列表：指定状态时加过滤；空状态返回全部
    #[test]
    fn admin_list_filters_requested_status() {
        let (sql, params) = admin_list_query(InsuranceProduct::STATUS_DRAFT, 1, 20);
        assert!(sql.contains("status = ?"), "{sql}");
        assert_eq!(params, vec!["DRAFT", "20", "0"]);

        let (sql_all, params_all) = admin_list_query("", 1, 20);
        assert!(!sql_all.contains("status = ?"), "{sql_all}");
        assert_eq!(params_all, vec!["20", "0"]);
    }

    /// 运营列表状态白名单：四个合法状态 + 空串，其余拒绝
    #[test]
    fn admin_list_status_whitelist() {
        for status in ["", "DRAFT", "ON_SALE", "OFF_SHELF", "DISCONTINUED"] {
            assert!(admin_status_allowed(status), "{status} 应合法");
        }
        assert!(!admin_status_allowed("BOGUS"));
        assert!(!admin_status_allowed("ON_SALE' OR '1'='1"));
    }
}
