//! 报价服务（任务 #4）
//!
//! 事务内原子创建报价：校验产品存在 → INSERT quotes → INSERT 受益人 → 回读 quote。
//! 报价号格式：`QT{UTC毫秒}{4位hex}`；默认 PENDING，7 天有效期。

use chrono::NaiveDate;
use chrono::{DateTime, Utc};
use mysql_async::prelude::Queryable;
use mysql_async::Value;
use rust_decimal::Decimal;
use serde::Deserialize;
use uuid::Uuid;

use crate::db::db_error;
use crate::db::Db;
use crate::error::{AppError, Result};
use crate::models::quote::Quote;

/// 创建报价请求体
#[derive(Debug, Deserialize)]
pub struct CreateQuoteReq {
    pub product_id: i64,
    pub user_id: i64,

    /// 投保人
    pub holder_name: String,
    pub holder_id_card_enc: Option<Vec<u8>>,

    /// 被保人
    pub insured_name: String,
    pub insured_id_card_enc: Option<Vec<u8>>,

    /// 保险金额
    pub insurance_amount: Decimal,
    /// 保险期间（月）
    pub term_months: i32,
    /// 每期保费
    pub premium: Decimal,
    /// 保费明细（JSON）
    pub premium_detail: Option<serde_json::Value>,
    /// 生效日
    pub effective_date: Option<NaiveDate>,
    /// 失效日
    pub expire_date: Option<NaiveDate>,
    /// 健康告知（JSON）
    pub health_declaration: Option<serde_json::Value>,
    /// 风险评估分
    pub risk_score: Option<i32>,
    /// 受益人列表
    pub beneficiaries: Vec<BeneficiaryReq>,
}

/// 受益人条目
#[derive(Debug, Clone, Deserialize)]
pub struct BeneficiaryReq {
    pub name: String,
    pub id_card_enc: Option<Vec<u8>>,
    pub relationship: Option<String>,
    pub beneficiary_type: String,
    pub share_percent: Option<Decimal>,
    pub sort_order: i32,
}

/// 报价服务
pub struct QuoteService {
    db: Db,
}

impl QuoteService {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// 生成报价号：`QT{UTC毫秒}{4位hex}`
    fn generate_quote_no() -> String {
        let ms = Utc::now().timestamp_millis();
        let hex = Uuid::new_v4().as_u128() as u64 & 0xFFFF;
        format!("QT{}{:04x}", ms, hex)
    }

    /// 事务内创建报价：产品存在性校验 → 落库 quote → 落库受益人 → 回读。
    pub async fn create(&self, req: CreateQuoteReq) -> Result<Quote> {
        let quote_no = Self::generate_quote_no();
        let now = Utc::now();
        let expires_at = now + chrono::Duration::days(7);

        // 序列化可空 JSON / Date 字段为字符串参数
        let premium_detail_str: Option<String> =
            req.premium_detail.as_ref().and_then(|v| serde_json::to_string(v).ok());
        let health_declaration_str: Option<String> =
            req.health_declaration.as_ref().and_then(|v| serde_json::to_string(v).ok());
        let effective_date_str: Option<String> =
            req.effective_date.map(|d| d.format("%Y-%m-%d").to_string());
        let expire_date_str: Option<String> =
            req.expire_date.map(|d| d.format("%Y-%m-%d").to_string());

        let quote_id: i64;
        let beneficiaries = req.beneficiaries.clone();

        quote_id = self
            .db
            .with_tx(|tx| {
                Box::pin(async move {
                    // 1) 校验产品存在
                    let exists: Option<i64> = tx
                        .exec_first(
                            "SELECT id FROM insurance_products WHERE id = ? AND deleted_at IS NULL LIMIT 1",
                            vec![req.product_id],
                        )
                        .await
                        .map_err(db_error)?;
                    if exists.is_none() {
                        return Err(AppError::business("产品不存在"));
                    }

                    // 2) INSERT quote
                    let params: Vec<Value> = vec![
                        Value::from(&quote_no),
                        Value::from(req.product_id),
                        Value::from(req.user_id),
                        Value::from(req.holder_name.clone()),
                        value_opt_vec(req.holder_id_card_enc.clone()),
                        Value::from(req.insurance_amount.to_string()),
                        Value::from(req.term_months),
                        Value::from(req.premium.to_string()),
                        value_opt_str(premium_detail_str),
                        value_opt_str(effective_date_str),
                        value_opt_str(expire_date_str),
                        value_opt_str(health_declaration_str),
                        value_opt_int(req.risk_score),
                        Value::from(now.format("%Y-%m-%d %H:%M:%S").to_string()),
                        Value::from(expires_at.format("%Y-%m-%d %H:%M:%S").to_string()),
                        Value::from(Quote::STATUS_PENDING.to_string()),
                    ];

                    tx.exec_drop(
                        "INSERT INTO quotes \
                         (quote_no, product_id, user_id, holder_name, holder_id_card_enc, \
                          insurance_amount, term_months, \
                          premium, premium_detail, effective_date, expire_date, \
                          health_declaration, risk_score, created_at, expires_at, status) \
                         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                        params,
                    )
                    .await
                    .map_err(db_error)?;

                    let id = tx.last_insert_id().unwrap_or_default() as i64;

                    // 3) 插入受益人
                    for b in &beneficiaries {
                        let bparams: Vec<Value> = vec![
                            Value::from(id),
                            Value::from(b.name.clone()),
                            value_opt_vec(b.id_card_enc.clone()),
                            value_opt_str(b.relationship.clone()),
                            Value::from(b.beneficiary_type.clone()),
                            value_opt_dec(b.share_percent.clone()),
                            Value::from(b.sort_order),
                        ];
                        tx.exec_drop(
                            "INSERT INTO quotes_beneficiaries \
                             (quote_id, name, id_card_enc, relationship, beneficiary_type, \
                              share_percent, sort_order) \
                             VALUES (?, ?, ?, ?, ?, ?, ?)",
                            bparams,
                        )
                        .await
                        .map_err(db_error)?;
                    }

                    Ok(id)
                })
            })
            .await?;

        // 4) 回读整条 quote（含时间戳 & expires_at）
        let row: Option<mysql_async::Row> = self
            .db
            .conn()
            .await?
            .exec_first(
                "SELECT * FROM quotes WHERE id = ? LIMIT 1",
                vec![quote_id],
            )
            .await
            .map_err(db_error)?;

        let row = row.ok_or_else(|| AppError::business("报价创建后回读失败"))?;

        let quote = row_to_quote(&row)?;
        Ok(quote)
    }

    /// 报价详情：按 id 查（未删除），无行返回 NotFound。
    pub async fn by_id(&self, id: i64) -> Result<Quote> {
        let row: Option<mysql_async::Row> = self
            .db
            .conn()
            .await?
            .exec_first(
                "SELECT * FROM quotes WHERE id = ? AND deleted_at IS NULL LIMIT 1",
                vec![id],
            )
            .await
            .map_err(db_error)?;
        row.map(|r| row_to_quote(&r)).transpose()?.ok_or(AppError::NotFound)
    }
}

// ---------- helpers: Value slice → mysql_async Row / Quote ----------

use mysql_async::Row;
use chrono::NaiveDateTime;

/// (已移除) Vec<Value> → Row 的包装被删除：mysql_async::Row 无公开构造器，
/// 回读直接用 exec_first 得到 Row，见 QuoteService::create。

/// 读行 → DateTime<Utc>
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

/// 读行 → Option<Decimal>（DECIMAL 列以字符串到达）
fn dec_opt_row(row: &Row, col: &str) -> Option<Decimal> {
    row.get::<Option<String>, &str>(col)
        .flatten()
        .and_then(|s| s.parse().ok())
}

/// 读行 → Option<NaiveDate>（DATE 列经 mysql_async chrono feature 直接解码为 NaiveDate，非字符串）
fn date_opt_row(row: &Row, col: &str) -> Option<NaiveDate> {
    row.get::<Option<NaiveDate>, &str>(col).flatten()
}

/// 读行 → Option<serde_json::Value>（JSON 文本）
fn json_opt_row(row: &Row, col: &str) -> Option<serde_json::Value> {
    row.get::<Option<String>, &str>(col)
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok())
}

/// 从 Row 重建 Quote
fn row_to_quote(row: &Row) -> Result<Quote> {
    Ok(Quote {
        id: row.get("id").unwrap_or_default(),
        quote_no: row.get("quote_no").unwrap_or_default(),
        product_id: row.get("product_id").unwrap_or_default(),
        user_id: row.get("user_id").unwrap_or_default(),
        holder_id: row.get::<Option<i64>, &str>("holder_id").flatten(),
        holder_name: row.get("holder_name").unwrap_or_default(),
        holder_id_card_enc: row.get("holder_id_card_enc").flatten(),
        insurance_amount: dec_opt_row(row, "insurance_amount").unwrap_or_default(),
        term_months: row.get("term_months").unwrap_or_default(),
        premium: dec_opt_row(row, "premium").unwrap_or_default(),
        premium_detail: json_opt_row(row, "premium_detail"),
        effective_date: date_opt_row(row, "effective_date"),
        expire_date: date_opt_row(row, "expire_date"),
        health_declaration: json_opt_row(row, "health_declaration"),
        risk_score: row.get("risk_score").flatten(),
        status: row.get("status").unwrap_or_default(),
        created_at: dt_row(row, "created_at"),
        expires_at: dt_row(row, "expires_at"),
        updated_at: dt_row(row, "updated_at"),
        deleted_at: dt_opt_row(row, "deleted_at"),
    })
}

// ---------- 参数构造 helper（Option → Value） ----------

fn value_opt_str(v: Option<String>) -> Value {
    v.map(Value::from).unwrap_or(Value::NULL)
}

fn value_opt_vec(v: Option<Vec<u8>>) -> Value {
    v.map(Value::from).unwrap_or(Value::NULL)
}

fn value_opt_int(v: Option<i32>) -> Value {
    v.map(Value::from).unwrap_or(Value::NULL)
}

fn value_opt_dec(v: Option<Decimal>) -> Value {
    v.map(|d| Value::from(d.to_string()))
        .unwrap_or(Value::NULL)
}
