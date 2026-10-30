//! 支付服务（交易闭环）
//!
//! MockProvider：预支付直接落 CREATED；回调须携带 `X-Callback-Token`（常量时间比较），
//! 成功后同一事务内把订单置为 PAID；相同交易号/终态重复回调不重复流转。

use chrono::Utc;
use mysql_async::prelude::Queryable;
use mysql_async::Value;
use mysql_async::Row;
use rust_decimal::Decimal;
use serde::Deserialize;
use uuid::Uuid;

use crate::db::{db_error, dec_opt_row, dt_opt_row, dt_row, json_opt_row, json_val, Db};
use crate::error::{AppError, Result};
use crate::models::payment::Payment;

#[derive(Debug, Deserialize)]
pub struct CreatePaymentReq {
    pub order_id: i64,
    #[serde(default)]
    pub channel: String,
}

#[derive(Debug, Deserialize)]
pub struct CallbackReq {
    pub payment_id: i64,
    pub provider_tx_id: Option<String>,
    /// 仅显式 true 视为成功；缺省即失败。
    pub success: Option<bool>,
    pub payload: Option<serde_json::Value>,
}

pub struct PaymentService {
    db: Db,
    callback_token: String,
}

impl PaymentService {
    pub fn new(db: Db, callback_token: String) -> Self {
        Self { db, callback_token }
    }

    fn payment_no() -> String {
        format!("PAY{}{:04x}", Utc::now().timestamp_millis(), Uuid::new_v4().as_u128() as u64 & 0xFFFF)
    }

    /// `user_id` 取自已认证身份（控制器从 token 取），不信请求体。
    ///
    /// 阶段 3 预支付统一记录 `provider = MOCK`，因此所有渠道的回调都必须走
    /// `/payments/callback/mock`；接入真实渠道（阶段 4）后按渠道分流。
    pub async fn prepay(&self, user_id: i64, req: CreatePaymentReq) -> Result<Payment> {
        let payment_no = Self::payment_no();
        let channel = normalize_channel(&req.channel)?;

        let row: Option<Row> = self
            .db
            .with_tx(|tx| {
                Box::pin(async move {
                    let order: Option<Row> = tx
                        .exec_first(
                            "SELECT id, payable_amount, currency, status FROM orders WHERE id = ? AND user_id = ? AND deleted_at IS NULL LIMIT 1",
                            vec![req.order_id, user_id],
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
                            Value::from(user_id),
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

    /// 渠道回调：`provider` 为路径值，`token_header` 为 `X-Callback-Token` 请求头。
    /// 配置为空、请求头缺失或不匹配一律 Forbidden（不暴露差异）。
    pub async fn callback(
        &self,
        provider: &str,
        token_header: Option<&str>,
        req: CallbackReq,
    ) -> Result<Payment> {
        if !callback_token_matches(&self.callback_token, token_header.unwrap_or_default()) {
            return Err(AppError::Forbidden);
        }
        let tx_id = req
            .provider_tx_id
            .as_deref()
            .map(str::trim)
            .unwrap_or_default()
            .to_string();
        if tx_id.is_empty() {
            return Err(AppError::business("provider_tx_id 必填"));
        }
        if tx_id.len() > 128 {
            return Err(AppError::business("provider_tx_id 过长"));
        }
        let provider = provider.trim().to_string();
        if provider.is_empty() {
            return Err(AppError::business("provider 必填"));
        }
        let success = req.success == Some(true);
        let payload = req.payload.clone();
        let payment_id = req.payment_id;
        let now = Utc::now();
        let dt = now.format("%Y-%m-%d %H:%M:%S").to_string();

        let row: Row = self
            .db
            .with_tx(|tx| {
                Box::pin(async move {
                    let pay: Option<Row> = tx
                        .exec_first(
                            "SELECT * FROM payments WHERE id = ? LIMIT 1 FOR UPDATE",
                            vec![payment_id],
                        )
                        .await
                        .map_err(db_error)?;
                    let pay = pay.ok_or_else(|| AppError::business("支付单不存在"))?;

                    let pay_provider: String = pay.get("provider").unwrap_or_default();
                    if !pay_provider.eq_ignore_ascii_case(&provider) {
                        return Err(AppError::business("支付渠道与支付单不匹配"));
                    }
                    let status: String = pay.get("status").unwrap_or_default();
                    // 终态（含 SUCCESS 重复回调）：不再做第二次状态流转
                    if status != Payment::STATUS_CREATED && status != Payment::STATUS_PROCESSING {
                        return Ok(pay);
                    }
                    // 交易号不得被其他支付单占用
                    let used: Option<i64> = tx
                        .exec_first(
                            "SELECT id FROM payments WHERE provider = ? AND provider_tx_id = ? \
                             AND id <> ? LIMIT 1",
                            vec![
                                Value::from(&pay_provider),
                                Value::from(&tx_id),
                                Value::from(payment_id),
                            ],
                        )
                        .await
                        .map_err(db_error)?;
                    if used.is_some() {
                        return Err(AppError::state_conflict("渠道交易号已被使用"));
                    }

                    if success {
                        tx.exec_drop(
                            "UPDATE payments SET status = ?, provider_tx_id = ?, \
                             callback_payload = ?, paid_at = IFNULL(paid_at, ?), updated_at = ? \
                             WHERE id = ?",
                            vec![
                                Value::from(Payment::STATUS_SUCCESS),
                                Value::from(&tx_id),
                                json_val(payload),
                                Value::from(&dt),
                                Value::from(&dt),
                                Value::from(payment_id),
                            ],
                        )
                        .await
                        .map_err(db_error)?;
                        let order_id: i64 = pay.get("order_id").unwrap_or_default();
                        tx.exec_drop(
                            "UPDATE orders SET status = ?, paid_at = IFNULL(paid_at, ?), updated_at = ? \
                             WHERE id = ? AND status IN (?, ?)",
                            vec![
                                Value::from(OrderStatus::PAID),
                                Value::from(&dt),
                                Value::from(&dt),
                                Value::from(order_id),
                                Value::from(OrderStatus::CREATED),
                                Value::from(OrderStatus::EXPIRED),
                            ],
                        )
                        .await
                        .map_err(db_error)?;
                        // 支付 SUCCESS 与订单 PAID 必须同事务生效：订单已被其他支付单置为 PAID 时
                        // 整笔回滚（支付单留在 CREATED 供对账），不能出现无订单效果的 SUCCESS。
                        if tx.affected_rows() != 1 {
                            return Err(AppError::state_conflict("订单状态已变化，支付回调未入账"));
                        }
                    } else {
                        // 失败回调不写 paid_at，也不触碰订单
                        tx.exec_drop(
                            "UPDATE payments SET status = ?, provider_tx_id = ?, \
                             callback_payload = ?, updated_at = ? WHERE id = ?",
                            vec![
                                Value::from(Payment::STATUS_FAILED),
                                Value::from(&tx_id),
                                json_val(payload),
                                Value::from(&dt),
                                Value::from(payment_id),
                            ],
                        )
                        .await
                        .map_err(db_error)?;
                    }

                    tx.exec_first("SELECT * FROM payments WHERE id = ? LIMIT 1", vec![payment_id])
                        .await
                        .map_err(db_error)?
                        .ok_or_else(|| AppError::business("回调处理后回读失败"))
                })
            })
            .await?;

        row_to_payment(&row)
    }
}

// ---------- helpers ----------

#[allow(non_snake_case)]
mod OrderStatus {
    pub const CREATED: &str = "CREATED";
    pub const EXPIRED: &str = "EXPIRED";
    pub const PAID: &str = "PAID";
}

/// 支付渠道归一化：空值默认 MOCK；仅接受已知渠道，超长/未知一律拒绝。
fn normalize_channel(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(Payment::CHANNEL_MOCK.to_string());
    }
    if trimmed.len() > 32 {
        return Err(AppError::business("不支持的支付渠道"));
    }
    const CHANNELS: [&str; 5] = ["WECHAT", "ALIPAY", "UNIONPAY", "BALANCE", "MOCK"];
    let upper = trimmed.to_ascii_uppercase();
    if CHANNELS.contains(&upper.as_str()) {
        Ok(upper)
    } else {
        Err(AppError::business("不支持的支付渠道"))
    }
}

/// 回调令牌常量时间比较：避免逐字符提前返回泄露前缀；配置为空或缺失一律不匹配。
fn callback_token_matches(expected: &str, provided: &str) -> bool {
    if expected.is_empty() {
        return false;
    }
    let (a, b) = (expected.as_bytes(), provided.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b.iter()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn callback_token_requires_exact_non_empty_match() {
        assert!(!callback_token_matches("", ""));
        assert!(!callback_token_matches("", "anything"));
        assert!(!callback_token_matches("s3cret", ""));
        assert!(callback_token_matches("s3cret", "s3cret"));
        assert!(!callback_token_matches("s3cret", "s3cres"));
        assert!(!callback_token_matches("s3cret", "s3cret2"));
    }

    #[test]
    fn channel_defaults_to_mock_and_rejects_unknown_values() {
        assert_eq!(normalize_channel("").unwrap(), Payment::CHANNEL_MOCK);
        assert_eq!(normalize_channel("  ").unwrap(), Payment::CHANNEL_MOCK);
        assert_eq!(normalize_channel("wechat").unwrap(), Payment::CHANNEL_WECHAT);
        assert_eq!(normalize_channel("UNIONPAY").unwrap(), "UNIONPAY");
        assert_eq!(normalize_channel("BALANCE").unwrap(), "BALANCE");
        assert!(normalize_channel("PAYPAL").is_err());
        assert!(normalize_channel(&"X".repeat(33)).is_err());
    }
}
