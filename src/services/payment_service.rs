//! 支付服务（交易闭环）
//!
//! MockProvider：预支付直接落 CREATED，回调幂等（相同 provider_tx_id 只处理一次）。

use chrono::{DateTime, NaiveDateTime, Utc};
use mysql_async::prelude::Queryable;
use mysql_async::Value;
use mysql_async::Row;
use rust_decimal::Decimal;
use serde::Deserialize;
use uuid::Uuid;

use crate::db::db_error;
use crate::db::Db;
use crate::error::{AppError, Result};
use crate::models::payment::Payment;

#[derive(Debug, Deserialize)]
pub struct CreatePaymentReq {
    pub order_id: i64,
    pub user_id: i64,
    #[serde(default)]
    pub channel: String,
}

#[derive(Debug, Deserialize)]
pub struct CallbackReq {
    pub payment_id: i64,
    pub provider_tx_id: Option<String>,
    pub success: Option<bool>,
    pub payload: Option<serde_json::Value>,
}

pub struct PaymentService {
    db: Db,
}

impl PaymentService {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    fn payment_no() -> String {
        format!("PAY{}{:04x}", Utc::now().timestamp_millis(), Uuid::new_v4().as_u128() as u64 & 0xFFFF)
    }

    pub async fn prepay(&self, req: CreatePaymentReq) -> Result<Payment> {
        let payment_no = Self::payment_no();
        let channel = if req.channel.is_empty() {
            Payment::CHANNEL_MOCK.to_string()
        } else {
            req.channel
        };

        let row: Option<Row> = self
            .db
            .with_tx(|tx| {
                Box::pin(async move {
                    let order: Option<Row> = tx
                        .exec_first(
                            "SELECT id, payable_amount, currency, status FROM orders WHERE id = ? AND user_id = ? AND deleted_at IS NULL LIMIT 1",
                            vec![req.order_id, req.user_id],
                        )
                        .await
                        .map_err(db_error)?;
                    let order = order.ok_or_else(|| AppError::business("订单不存在"))?;
                    let status: String = order.get("status").unwrap_or_default();
                    if status != OrderStatus::CREATED && status != OrderStatus::EXPIRED {
                        return Err(AppError::business("订单状态不可支付"));
                    }
                    let amount: Decimal = dec_opt_row(&order, "payable_amount").unwrap_or_default();
                    let currency: String = order.get("currency").unwrap_or_default();

                    let now = Utc::now();
                    let dt = now.format("%Y-%m-%d %H:%M:%S").to_string();
                    // 主键由应用层 snowflake 预生成后显式插入（全库自增迁移，见 idgen）
                    let pid = crate::utils::idgen::next_id();
                    tx.exec_drop(
                        "INSERT INTO payments (id, payment_no, order_id, user_id, amount, currency, \
                         channel, provider, provider_tx_id, status, prepay_payload, callback_payload, \
                         created_at, updated_at) \
                         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                        vec![
                            pid.into(),
                            Value::from(&payment_no),
                            Value::from(req.order_id),
                            Value::from(req.user_id),
                            Value::from(amount.to_string()),
                            Value::from(&currency),
                            Value::from(&channel),
                            Value::from(Payment::CHANNEL_MOCK),
                            Value::NULL,
                            Value::from(Payment::STATUS_CREATED.to_string()),
                            Value::NULL,
                            Value::NULL,
                            Value::from(&dt),
                            Value::from(&dt),
                        ],
                    )
                    .await
                    .map_err(db_error)?;
                    tx.exec_first("SELECT * FROM payments WHERE id = ? LIMIT 1", vec![pid])
                        .await
                        .map_err(db_error)
                })
            })
            .await?;

        row.map(|r| row_to_payment(&r)).transpose()?
            .ok_or_else(|| AppError::business("支付单创建后回读失败"))
    }

    pub async fn callback(&self, req: CallbackReq) -> Result<Payment> {
        let now = Utc::now();
        let dt = now.format("%Y-%m-%d %H:%M:%S").to_string();
        let tx_id = req.provider_tx_id.clone().unwrap_or_default();

        let row: Option<Row> = self
            .db
            .with_tx(|tx| {
                Box::pin(async move {
                    let pay: Option<Row> = tx
                        .exec_first(
                            "SELECT id, provider_tx_id, status FROM payments WHERE id = ? LIMIT 1",
                            vec![req.payment_id],
                        )
                        .await
                        .map_err(db_error)?;
                    let pay = pay.ok_or_else(|| AppError::business("支付单不存在"))?;
                    // 幂等：相同 provider_tx_id 命中即返回当前状态
                    if let Some(ref etx) = pay.get::<Option<String>, &str>("provider_tx_id").flatten() {
                        if etx == &tx_id && !tx_id.is_empty() {
                            return tx.exec_first(
                                "SELECT * FROM payments WHERE id = ? LIMIT 1",
                                vec![req.payment_id],
                            )
                            .await
                            .map_err(db_error);
                        }
                    }
                    let status = if req.success.unwrap_or(true) {
                        Payment::STATUS_SUCCESS
                    } else {
                        Payment::STATUS_FAILED
                    };
                    tx.exec_drop(
                        "UPDATE payments SET status = ?, provider_tx_id = ?, \
                         callback_payload = ?, paid_at = IFNULL(paid_at, ?), updated_at = ? \
                         WHERE id = ? AND status IN (?, ?)",
                        vec![
                            Value::from(status.to_string()),
                            Value::from(&tx_id),
                            json_val(req.payload.clone()),
                            Value::from(&dt),
                            Value::from(&dt),
                            Value::from(req.payment_id),
                            Value::from(Payment::STATUS_CREATED),
                            Value::from(Payment::STATUS_PROCESSING),
                        ],
                    )
                    .await
                    .map_err(db_error)?;
                    tx.exec_first("SELECT * FROM payments WHERE id = ? LIMIT 1", vec![req.payment_id])
                        .await
                        .map_err(db_error)
                })
            })
            .await?;

        row.map(|r| row_to_payment(&r)).transpose()?
            .ok_or_else(|| AppError::business("回调处理后回读失败"))
    }

    pub async fn by_id(&self, id: i64) -> Result<Payment> {
        let row: Option<Row> = self
            .db
            .conn()
            .await?
            .exec_first("SELECT * FROM payments WHERE id = ? LIMIT 1", vec![id])
            .await
            .map_err(db_error)?;
        row.map(|r| row_to_payment(&r)).transpose()?.ok_or(AppError::NotFound)
    }

    pub async fn by_user(&self, user_id: i64, page: u32, size: u32) -> Result<Vec<Payment>> {
        let size = size.clamp(1, 100) as usize;
        let offset = ((page.max(1) as usize) - 1) * size;
        let rows: Vec<Row> = self
            .db
            .conn()
            .await?
            .exec(
                "SELECT * FROM payments WHERE user_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?",
                vec![user_id, size as i64, offset as i64],
            )
            .await
            .map_err(db_error)?;
        Ok(rows.iter().map(|r| row_to_payment(r)).collect::<Result<Vec<_>>>()?)
    }
}

// ---------- helpers ----------

#[allow(non_snake_case)]
mod OrderStatus {
    pub const CREATED: &str = "CREATED";
    pub const EXPIRED: &str = "EXPIRED";
}

fn dt_row(row: &Row, col: &str) -> DateTime<Utc> {
    row.get::<NaiveDateTime, &str>(col).unwrap_or_default().and_utc()
}
fn dt_opt_row(row: &Row, col: &str) -> Option<DateTime<Utc>> {
    row.get::<Option<NaiveDateTime>, &str>(col).flatten().map(|d| d.and_utc())
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

fn row_to_payment(row: &Row) -> Result<Payment> {
    Ok(Payment {
        id: row.get("id").unwrap_or_default(),
        payment_no: row.get("payment_no").unwrap_or_default(),
        order_id: row.get("order_id").unwrap_or_default(),
        user_id: row.get("user_id").unwrap_or_default(),
        amount: dec_opt_row(row, "amount").unwrap_or_default(),
        currency: row.get("currency").unwrap_or_default(),
        channel: row.get("channel").unwrap_or_default(),
        provider: row.get("provider").unwrap_or_default(),
        provider_tx_id: row.get("provider_tx_id").flatten(),
        status: row.get("status").unwrap_or_default(),
        prepay_payload: json_opt_row(row, "prepay_payload"),
        callback_payload: json_opt_row(row, "callback_payload"),
        paid_at: dt_opt_row(row, "paid_at"),
        refunded_at: dt_opt_row(row, "refunded_at"),
        created_at: dt_row(row, "created_at"),
        updated_at: dt_row(row, "updated_at"),
    })
}
