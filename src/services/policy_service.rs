//! 保单服务（交易闭环）
//!
//! 支付成功后签发保单：校验订单已 PAID → INSERT policy → 订单进 POLICY_ISSUED → 回读。

use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use mysql_async::prelude::Queryable;
use mysql_async::Value;
use mysql_async::Row;
use rust_decimal::Decimal;
use serde::Deserialize;
use uuid::Uuid;

use crate::db::db_error;
use crate::db::Db;
use crate::error::{AppError, Result};
use crate::models::policy::Policy;

#[derive(Debug, Deserialize)]
pub struct IssuePolicyReq {
    pub order_id: i64,
    pub quote_id: i64,
    pub user_id: i64,
    #[serde(default)]
    pub issue_type: String,
    #[serde(default)]
    pub is_renewable: bool,
}

pub struct PolicyService {
    db: Db,
}

impl PolicyService {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    fn policy_no() -> String {
        format!("POL{}{:04x}", Utc::now().timestamp_millis(), Uuid::new_v4().as_u128() as u64 & 0xFFFF)
    }

    pub async fn issue(&self, req: IssuePolicyReq) -> Result<Policy> {
        let policy_no = Self::policy_no();
        let issue_type = if req.issue_type.is_empty() { "NEW".to_string() } else { req.issue_type.clone() };

        let row: Option<Row> = self
            .db
            .with_tx(|tx| {
                Box::pin(async move {
                    let order: Option<Row> = tx
                        .exec_first(
                            "SELECT id, product_id, product_name, holder_name, \
                             insurance_amount, term_months, payable_amount, status \
                             FROM orders WHERE id = ? AND user_id = ? AND deleted_at IS NULL LIMIT 1",
                            vec![req.order_id, req.user_id],
                        )
                        .await
                        .map_err(db_error)?;
                    let order = order.ok_or_else(|| AppError::business("订单不存在"))?;
                    let order_status: String = order.get("status").unwrap_or_default();
                    if order_status != OrderStatus::PAID {
                        return Err(AppError::business("订单未支付，无法签发保单"));
                    }
                    let product_id: i64 = order.get("product_id").unwrap_or_default();
                    let product_name: String = order.get("product_name").unwrap_or_default();
                    let holder_name: String = order.get("holder_name").unwrap_or_default();
                    let insurance_amount: Decimal = dec_opt_row(&order, "insurance_amount").unwrap_or_default();
                    let term_months: i32 = order.get("term_months").unwrap_or_default();
                    let premium: Decimal = dec_opt_row(&order, "payable_amount").unwrap_or_default();

                    let q: Option<Row> = tx
                        .exec_first(
                            "SELECT effective_date, expire_date, premium_detail, holder_id \
                             FROM quotes WHERE id = ? AND user_id = ? LIMIT 1",
                            vec![req.quote_id, req.user_id],
                        )
                        .await
                        .map_err(db_error)?;
                    let q = q.ok_or_else(|| AppError::business("报价不存在"))?;
                    let parse_date =
                        |s: String| NaiveDate::parse_from_str(&s, "%Y-%m-%d").unwrap_or_default();
                    let effective_date: NaiveDate = q
                        .get::<Option<String>, &str>("effective_date")
                        .flatten()
                        .map(parse_date)
                        .unwrap_or_else(|| Utc::now().date_naive());
                    let expire_date: NaiveDate = q
                        .get::<Option<String>, &str>("expire_date")
                        .flatten()
                        .map(parse_date)
                        .unwrap_or_else(|| {
                            effective_date.checked_add_days(chrono::Days::new(365)).unwrap_or(effective_date)
                        });
                    let premium_detail = json_opt_row(&q, "premium_detail");
                    let holder_id: Option<i64> = q.get("holder_id").flatten();

                    let now = Utc::now();
                    let dt = now.format("%Y-%m-%d %H:%M:%S").to_string();
                    let ed = effective_date.format("%Y-%m-%d").to_string();
                    let xd = expire_date.format("%Y-%m-%d").to_string();

                    tx.exec_drop(
                        "INSERT INTO policies (policy_no, order_id, quote_id, user_id, holder_id, \
                         product_id, product_name, holder_name, insurance_amount, term_months, \
                         premium, effective_date, expire_date, status, issue_type, is_renewable, \
                         premium_detail, issued_at, created_at, updated_at) \
                         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                        vec![
                            Value::from(&policy_no),
                            Value::from(req.order_id),
                            Value::from(req.quote_id),
                            Value::from(req.user_id),
                            value_opt_int(holder_id),
                            Value::from(product_id),
                            Value::from(&product_name),
                            Value::from(&holder_name),
                            Value::from(insurance_amount.to_string()),
                            Value::from(term_months),
                            Value::from(premium.to_string()),
                            Value::from(&ed),
                            Value::from(&xd),
                            Value::from(Policy::STATUS_ACTIVE.to_string()),
                            Value::from(issue_type),
                            Value::from(req.is_renewable),
                            json_val(premium_detail),
                            Value::from(&dt),
                            Value::from(&dt),
                            Value::from(&dt),
                        ],
                    )
                    .await
                    .map_err(db_error)?;

                    tx.exec_drop(
                        "UPDATE orders SET status = ? WHERE id = ? AND user_id = ?",
                        vec![
                            Value::from(OrderStatus::POLICY_ISSUED),
                            Value::from(req.order_id),
                            Value::from(req.user_id),
                        ],
                    )
                    .await
                    .map_err(db_error)?;

                    let pid = tx.last_insert_id().unwrap_or_default() as i64;
                    tx.exec_first("SELECT * FROM policies WHERE id = ? LIMIT 1", vec![pid])
                        .await
                        .map_err(db_error)
                })
            })
            .await?;

        row.map(|r| row_to_policy(&r)).transpose()?
            .ok_or_else(|| AppError::business("保单签发后回读失败"))
    }

    pub async fn by_id(&self, id: i64) -> Result<Policy> {
        let row: Option<Row> = self
            .db
            .conn()
            .await?
            .exec_first("SELECT * FROM policies WHERE id = ? AND deleted_at IS NULL LIMIT 1", vec![id])
            .await
            .map_err(db_error)?;
        row.map(|r| row_to_policy(&r)).transpose()?.ok_or(AppError::NotFound)
    }

    pub async fn by_user(&self, user_id: i64, page: u32, size: u32) -> Result<Vec<Policy>> {
        let size = size.clamp(1, 100) as usize;
        let offset = ((page.max(1) as usize) - 1) * size;
        let rows: Vec<Row> = self
            .db
            .conn()
            .await?
            .exec(
                "SELECT * FROM policies WHERE user_id = ? AND deleted_at IS NULL ORDER BY created_at DESC LIMIT ? OFFSET ?",
                vec![user_id, size as i64, offset as i64],
            )
            .await
            .map_err(db_error)?;
        Ok(rows.iter().map(|r| row_to_policy(r)).collect::<Result<Vec<_>>>()?)
    }
}

// ---------- helpers ----------

#[allow(non_snake_case)]
mod OrderStatus {
    pub const PAID: &str = "PAID";
    pub const POLICY_ISSUED: &str = "POLICY_ISSUED";
}

fn dt_row(row: &Row, col: &str) -> DateTime<Utc> {
    row.get::<NaiveDateTime, &str>(col).unwrap_or_default().and_utc()
}
fn dec_opt_row(row: &Row, col: &str) -> Option<Decimal> {
    row.get::<Option<String>, &str>(col).flatten().and_then(|s| s.parse().ok())
}
fn json_val(v: Option<serde_json::Value>) -> Value {
    v.and_then(|j| serde_json::to_string(&j).ok()).map(Value::from).unwrap_or(Value::NULL)
}
fn json_opt_row(row: &Row, col: &str) -> Option<serde_json::Value> {
    row.get::<Option<String>, &str>(col).flatten().and_then(|s| serde_json::from_str(&s).ok())
}
fn value_opt_int(v: Option<i64>) -> Value {
    v.map(Value::from).unwrap_or(Value::NULL)
}

fn row_to_policy(row: &Row) -> Result<Policy> {
    let parse_date = |s: String| NaiveDate::parse_from_str(&s, "%Y-%m-%d").unwrap_or_default();
    Ok(Policy {
        id: row.get("id").unwrap_or_default(),
        policy_no: row.get("policy_no").unwrap_or_default(),
        order_id: row.get("order_id").unwrap_or_default(),
        quote_id: row.get("quote_id").unwrap_or_default(),
        user_id: row.get("user_id").unwrap_or_default(),
        holder_id: row.get("holder_id").flatten(),
        product_id: row.get("product_id").unwrap_or_default(),
        product_name: row.get("product_name").unwrap_or_default(),
        holder_name: row.get("holder_name").unwrap_or_default(),
        holder_id_card_enc: row.get("holder_id_card_enc").flatten(),
        insurance_amount: dec_opt_row(row, "insurance_amount").unwrap_or_default(),
        premium: dec_opt_row(row, "premium").unwrap_or_default(),
        term_months: row.get("term_months").unwrap_or_default(),
        effective_date: row.get::<Option<String>, &str>("effective_date").flatten().map(parse_date).unwrap_or_default(),
        expire_date: row.get::<Option<String>, &str>("expire_date").flatten().map(parse_date).unwrap_or_default(),
        status: row.get("status").unwrap_or_default(),
        issue_type: row.get("issue_type").unwrap_or_default(),
        is_renewable: row.get("is_renewable").unwrap_or_default(),
        pdf_path: row.get("pdf_path").flatten(),
        premium_detail: json_opt_row(row, "premium_detail"),
        issued_at: dt_opt_row(row, "issued_at"),
        created_at: dt_row(row, "created_at"),
        updated_at: dt_row(row, "updated_at"),
        deleted_at: dt_opt_row(row, "deleted_at"),
    })
}
fn dt_opt_row(row: &Row, col: &str) -> Option<DateTime<Utc>> {
    row.get::<Option<NaiveDateTime>, &str>(col).flatten().map(|d| d.and_utc())
}
