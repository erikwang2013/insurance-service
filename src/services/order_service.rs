//! 订单服务（任务 #4）
//!
//! 事务内原子创建订单：校验 quote 属于该用户 → INSERT orders → 回读。
//! 订单号格式：`OD{UTC毫秒}{4位hex}`；状态初始 CREATED。

use chrono::{DateTime, Utc};
use mysql_async::prelude::Queryable;
use mysql_async::Value;
use rust_decimal::Decimal;
use serde::Deserialize;
use uuid::Uuid;

use crate::db::db_error;
use crate::db::Db;
use crate::error::{AppError, Result};
use crate::models::order::Order;

/// 创建订单请求体
#[derive(Debug, Deserialize)]
pub struct CreateOrderReq {
    pub quote_id: i64,
    pub user_id: i64,
    /// 订单备注
    pub remark: Option<String>,
    /// 优惠金额（可选，默认为 0）
    pub discount_amount: Option<Decimal>,
}

/// 订单服务
pub struct OrderService {
    db: Db,
}

impl OrderService {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// 生成订单号：`OD{UTC毫秒}{4位hex}`
    fn generate_order_no() -> String {
        let ms = Utc::now().timestamp_millis();
        let hex = Uuid::new_v4().as_u128() as u64 & 0xFFFF;
        format!("OD{}{:04x}", ms, hex)
    }

    /// 事务内创建订单：校验 quote 存在且属于该用户 → INSERT orders → 回读。
    pub async fn create(&self, req: CreateOrderReq) -> Result<Order> {
        let order_no = Self::generate_order_no();
        let now = Utc::now();

        let quote: Option<mysql_async::Row> = self
            .db
            .with_tx(|tx| {
                Box::pin(async move {
                    // 1) 校验 quote 存在且属于该用户
                    let q: Option<mysql_async::Row> = tx
                        .exec_first(
                            "SELECT * FROM quotes WHERE id = ? AND user_id = ? AND status = 'PENDING' LIMIT 1",
                            vec![req.quote_id, req.user_id],
                        )
                        .await
                        .map_err(db_error)?;
                    let q = q.ok_or_else(|| AppError::business("报价不存在或已失效"))?;

                    let product_id: i64 = q.get("product_id").unwrap_or_default();
                    let holder_name: String = q.get("holder_name").unwrap_or_default();
                    let insurance_amount: Decimal =
                        dec_from_row(&q, "insurance_amount").unwrap_or_default();
                    let term_months: i32 = q.get("term_months").unwrap_or_default();
                    let premium: Decimal =
                        dec_from_row(&q, "premium").unwrap_or_default();

                    // 2) 查产品名称 + 币种（走同一事务，避免读到 stale 行）
                    let prod: Option<mysql_async::Row> = tx
                        .exec_first(
                            "SELECT name, currency FROM insurance_products WHERE id = ? AND deleted_at IS NULL LIMIT 1",
                            vec![product_id],
                        )
                        .await
                        .map_err(db_error)?;
                    let (product_name, currency): (String, String) = match prod {
                        Some(p) => (
                            p.get("name").unwrap_or_default(),
                            p.get("currency").unwrap_or_default(),
                        ),
                        None => return Err(AppError::business("产品不存在")),
                    };

                    // 3) 计算金额：total_amount = premium, payable_amount = premium - discount
                    let discount = req
                        .discount_amount
                        .unwrap_or_else(|| Decimal::ZERO);
                    let total_amount = premium;
                    let payable_amount = total_amount.saturating_sub(discount);
                    let remark_opt: Option<String> = req.remark.clone();

                    let params: Vec<Value> = vec![
                        Value::from(&order_no),
                        Value::from(req.quote_id),
                        Value::from(req.user_id),
                        Value::from(product_id),
                        Value::from(&product_name),
                        Value::from(&holder_name),
                        Value::from(insurance_amount.to_string()),
                        Value::from(term_months),
                        Value::from(total_amount.to_string()),
                        Value::from(discount.to_string()),
                        Value::from(payable_amount.to_string()),
                        Value::from(&currency),
                        Value::from(Order::STATUS_CREATED.to_string()),
                        value_opt_str(remark_opt),
                        Value::from(now.format("%Y-%m-%d %H:%M:%S").to_string()),
                        Value::from(now.format("%Y-%m-%d %H:%M:%S").to_string()),
                    ];

                    tx.exec_drop(
                        "INSERT INTO orders \
                         (order_no, quote_id, user_id, product_id, product_name, \
                          holder_name, insurance_amount, term_months, total_amount, \
                          discount_amount, payable_amount, currency, status, remark, \
                          created_at, updated_at) \
                         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                        params,
                    )
                    .await
                    .map_err(db_error)?;

                    let oid = tx.last_insert_id().unwrap_or_default() as i64;

                    // 4) 回读订单
                    tx.exec_first(
                        "SELECT * FROM orders WHERE id = ? LIMIT 1",
                        vec![oid],
                    )
                    .await
                    .map_err(db_error)
                })
            })
            .await?;

        quote.map(|r| row_to_order(&r)).transpose()?.ok_or_else(|| AppError::business("订单创建后回读失败"))
    }

    /// 按 id 查订单
    pub async fn by_id(&self, id: i64) -> Result<Order> {
        let row: Option<mysql_async::Row> = self
            .db
            .conn()
            .await?
            .exec_first("SELECT * FROM orders WHERE id = ? AND deleted_at IS NULL LIMIT 1", vec![id])
            .await
            .map_err(db_error)?;
        row.map(|r| row_to_order(&r)).transpose()?
            .ok_or(AppError::NotFound)
    }

    /// 按 user_id 分页查订单
    pub async fn by_user(&self, user_id: i64, page: u32, size: u32) -> Result<Vec<Order>> {
        let size = size.clamp(1, 100) as usize;
        let offset = ((page.max(1) as usize) - 1) * size;
        let rows: Vec<mysql_async::Row> = self
            .db
            .conn()
            .await?
            .exec(
                "SELECT * FROM orders WHERE user_id = ? AND deleted_at IS NULL \
                 ORDER BY created_at DESC LIMIT ? OFFSET ?",
                vec![user_id, size as i64, offset as i64],
            )
            .await
            .map_err(db_error)?;
        let mut orders = Vec::with_capacity(rows.len());
        for r in rows {
            orders.push(row_to_order(&r)?);
        }
        Ok(orders)
    }
}

// ---------- helpers ----------

use mysql_async::Row;
use chrono::NaiveDateTime;

/// 读行 → DateTime<Utc>
fn dt_row(row: &Row, col: &str) -> DateTime<Utc> {
    row.get::<NaiveDateTime, &str>(col)
        .unwrap_or_default()
        .and_utc()
}

fn dt_opt_row(row: &Row, col: &str) -> Option<DateTime<Utc>> {
    row.get::<Option<NaiveDateTime>, &str>(col)
        .flatten()
        .map(|d| d.and_utc())
}

fn dec_from_row(row: &Row, col: &str) -> Option<Decimal> {
    row.get::<Option<String>, &str>(col)
        .flatten()
        .and_then(|s| s.parse().ok())
}

fn value_opt_str(v: Option<String>) -> Value {
    v.map(Value::from).unwrap_or(Value::NULL)
}

fn row_to_order(row: &Row) -> Result<Order> {
    Ok(Order {
        id: row.get("id").unwrap_or_default(),
        order_no: row.get("order_no").unwrap_or_default(),
        quote_id: row.get("quote_id").unwrap_or_default(),
        user_id: row.get("user_id").unwrap_or_default(),
        product_id: row.get("product_id").unwrap_or_default(),
        product_name: row.get("product_name").unwrap_or_default(),
        holder_name: row.get("holder_name").unwrap_or_default(),
        insurance_amount: dec_from_row(row, "insurance_amount").unwrap_or_default(),
        term_months: row.get("term_months").unwrap_or_default(),
        total_amount: dec_from_row(row, "total_amount").unwrap_or_default(),
        discount_amount: dec_from_row(row, "discount_amount").unwrap_or_default(),
        payable_amount: dec_from_row(row, "payable_amount").unwrap_or_default(),
        currency: row.get("currency").unwrap_or_default(),
        status: row.get("status").unwrap_or_default(),
        paid_at: dt_opt_row(row, "paid_at"),
        policy_issued_at: dt_opt_row(row, "policy_issued_at"),
        cancelled_at: dt_opt_row(row, "cancelled_at"),
        remark: row.get("remark").flatten(),
        created_at: dt_row(row, "created_at"),
        updated_at: dt_row(row, "updated_at"),
        deleted_at: dt_opt_row(row, "deleted_at"),
    })
}
