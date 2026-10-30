//! 报价服务（任务 #4 / C4）
//!
//! 事务内原子创建报价：校验产品在售与保额/期限区间 → 费率行试算 → INSERT quotes → INSERT 受益人 → 回读 quote。
//! 报价号格式：`QT{UTC毫秒}{4位hex}`；默认 PENDING，7 天有效期。
//!
//! 保费只由服务端按 `保费=保额×rate` 计算：未配置费率行（或费率表未部署）时拒绝报价，
//! 不再信任请求体中的 premium。

use chrono::NaiveDate;
use chrono::Utc;
use mysql_async::prelude::Queryable;
use mysql_async::Value;
use rust_decimal::Decimal;
use serde::Deserialize;
use uuid::Uuid;

use crate::db::{
    date_opt_row, db_error, dec_opt_row, dt_opt_row, dt_row, json_opt_row, value_opt_dec,
    value_opt_int, value_opt_str, value_opt_vec, Db,
};
use crate::error::{AppError, Result};
use crate::models::insurance_product::InsuranceProduct;
use crate::models::quote::Quote;

/// 创建报价请求体
#[derive(Debug, Deserialize)]
pub struct CreateQuoteReq {
    pub product_id: i64,

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
    /// 保费明细（JSON）
    pub premium_detail: Option<serde_json::Value>,
    /// 生效日（缺省今天，不得早于今天）
    pub effective_date: Option<NaiveDate>,
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

    /// 事务内创建报价：产品在售/区间校验 → 费率试算 → 落库 quote → 落库受益人 → 回读。
    /// `user_id` 取自已认证身份（控制器从 token 取），不信请求体。
    pub async fn create(&self, user_id: i64, req: CreateQuoteReq) -> Result<Quote> {
        let quote_no = Self::generate_quote_no();
        let now = Utc::now();
        let expires_at = now + chrono::Duration::days(7);

        // 序列化可空 JSON / Date 字段为字符串参数
        let premium_detail_str: Option<String> =
            req.premium_detail.as_ref().and_then(|v| serde_json::to_string(v).ok());
        let health_declaration_str: Option<String> =
            req.health_declaration.as_ref().and_then(|v| serde_json::to_string(v).ok());
        // 保障期只由服务端推导：客户端失效日一律忽略（请求体中即使出现也不会被反序列化）
        let (effective_date, expire_date) =
            coverage_window(req.effective_date, req.term_months, now.date_naive())?;
        let effective_date_str = effective_date.format("%Y-%m-%d").to_string();
        let expire_date_str = expire_date.format("%Y-%m-%d").to_string();

        // 主键由应用层 snowflake 预生成后显式插入（全库自增迁移，见 idgen）
        let quote_id = crate::utils::idgen::next_id();
        let beneficiaries = req.beneficiaries.clone();

        self
            .db
            .with_tx(|tx| {
                Box::pin(async move {
                    // 1) 产品须存在、未删除且在售；读取保额/期限区间用于边界校验
                    let product: Option<Row> = tx
                        .exec_first(
                            "SELECT status, min_amount, max_amount, min_term_months, max_term_months \
                             FROM insurance_products WHERE id = ? AND deleted_at IS NULL LIMIT 1",
                            vec![req.product_id],
                        )
                        .await
                        .map_err(db_error)?;
                    let product = product.ok_or_else(|| AppError::business("产品不存在"))?;
                    let status: String = product
                        .get::<Option<String>, &str>("status")
                        .flatten()
                        .unwrap_or_default();
                    if status != InsuranceProduct::STATUS_ON_SALE {
                        return Err(AppError::business("产品未上架，暂不可报价"));
                    }
                    validate_quote_bounds(
                        req.insurance_amount,
                        req.term_months,
                        dec_opt_row(&product, "min_amount"),
                        dec_opt_row(&product, "max_amount"),
                        product.get::<Option<i32>, &str>("min_term_months").flatten(),
                        product.get::<Option<i32>, &str>("max_term_months").flatten(),
                    )?;

                    // 2) 费率行必须命中：保费一律由服务端按 保额×rate 计算
                    let rate = match tx
                        .exec_first(
                            "SELECT rate FROM quote_rates \
                             WHERE product_id = ? AND term_months = ? \
                               AND amount_min <= ? \
                               AND (amount_max IS NULL OR amount_max >= ?) \
                             ORDER BY amount_min ASC, id ASC LIMIT 1",
                            vec![
                                Value::from(req.product_id),
                                Value::from(req.term_months),
                                Value::from(req.insurance_amount.to_string()),
                                Value::from(req.insurance_amount.to_string()),
                            ],
                        )
                        .await
                    {
                        Ok(Some(row)) => dec_opt_row(&row, "rate"),
                        Ok(None) => None,
                        // 仅费率表本身未部署时视为未配置费率（代码先于建表上线不该 500）；
                        // 其他数据库错误照常上抛，不能伪装成「未配置费率」
                        Err(e)
                            if e.to_string().contains("quote_rates")
                                && e.to_string().contains("doesn't exist") =>
                        {
                            None
                        }
                        Err(e) => return Err(db_error(e)),
                    };
                    let rate =
                        rate.ok_or_else(|| AppError::business("该产品未配置费率，暂不可报价"))?;
                    let premium = compute_premium(req.insurance_amount, rate)?;

                    // 3) INSERT quote
                    let params: Vec<Value> = vec![
                        quote_id.into(),
                        Value::from(&quote_no),
                        Value::from(req.product_id),
                        Value::from(user_id),
                        Value::from(req.holder_name.clone()),
                        value_opt_vec(req.holder_id_card_enc.clone()),
                        Value::from(req.insurance_amount.to_string()),
                        Value::from(req.term_months),
                        Value::from(premium.to_string()),
                        value_opt_str(premium_detail_str),
                        Value::from(effective_date_str),
                        Value::from(expire_date_str),
                        value_opt_str(health_declaration_str),
                        value_opt_int(req.risk_score),
                        Value::from(now.format("%Y-%m-%d %H:%M:%S").to_string()),
                        Value::from(expires_at.format("%Y-%m-%d %H:%M:%S").to_string()),
                        Value::from(Quote::STATUS_PENDING.to_string()),
                    ];

                    tx.exec_drop(
                        "INSERT INTO quotes \
                         (id, quote_no, product_id, user_id, holder_name, holder_id_card_enc, \
                          insurance_amount, term_months, \
                          premium, premium_detail, effective_date, expire_date, \
                          health_declaration, risk_score, created_at, expires_at, status) \
                         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                        params,
                    )
                    .await
                    .map_err(db_error)?;

                    // 4) 插入受益人（子表主键同样显式预生成，避免依赖父表回读）
                    for b in &beneficiaries {
                        let bid = crate::utils::idgen::next_id();
                        let bparams: Vec<Value> = vec![
                            bid.into(),
                            quote_id.into(),
                            Value::from(b.name.clone()),
                            value_opt_vec(b.id_card_enc.clone()),
                            value_opt_str(b.relationship.clone()),
                            Value::from(b.beneficiary_type.clone()),
                            value_opt_dec(b.share_percent),
                            Value::from(b.sort_order),
                        ];
                        tx.exec_drop(
                            "INSERT INTO quotes_beneficiaries \
                             (id, quote_id, name, id_card_enc, relationship, beneficiary_type, \
                              share_percent, sort_order) \
                             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                            bparams,
                        )
                        .await
                        .map_err(db_error)?;
                    }

                    Ok(quote_id)
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

    /// 报价详情：按 id 且归属 `user_id` 查（未删除），无行返回 NotFound。
    pub async fn by_id(&self, id: i64, user_id: i64) -> Result<Quote> {
        let row: Option<mysql_async::Row> = self
            .db
            .conn()
            .await?
            .exec_first(
                "SELECT * FROM quotes WHERE id = ? AND user_id = ? AND deleted_at IS NULL LIMIT 1",
                vec![id, user_id],
            )
            .await
            .map_err(db_error)?;
        row.map(|r| row_to_quote(&r)).transpose()?.ok_or(AppError::NotFound)
    }
}

/// 报价边界校验：金额/期限必须为正，且落在产品配置的区间内（NULL 表示不限）。
fn validate_quote_bounds(
    amount: Decimal,
    term_months: i32,
    min_amount: Option<Decimal>,
    max_amount: Option<Decimal>,
    min_term: Option<i32>,
    max_term: Option<i32>,
) -> Result<()> {
    if amount <= Decimal::ZERO {
        return Err(AppError::business("保险金额必须大于 0"));
    }
    if min_amount.is_some_and(|m| amount < m) || max_amount.is_some_and(|m| amount > m) {
        return Err(AppError::business("保险金额超出产品承保范围"));
    }
    if term_months <= 0 {
        return Err(AppError::business("保障期必须大于 0"));
    }
    if min_term.is_some_and(|m| term_months < m) || max_term.is_some_and(|m| term_months > m) {
        return Err(AppError::business("保障期超出产品承保范围"));
    }
    Ok(())
}

/// 保障期推导：`expire = effective + term_months`（月粒度，月末自动夹到当月最后一天）。
/// 生效日缺省今天，且不得早于今天；保障期必须为正且不溢出日历。
fn coverage_window(
    effective: Option<NaiveDate>,
    term_months: i32,
    today: NaiveDate,
) -> Result<(NaiveDate, NaiveDate)> {
    if term_months <= 0 {
        return Err(AppError::business("保障期必须大于 0"));
    }
    let effective = effective.unwrap_or(today);
    if effective < today {
        return Err(AppError::business("生效日期不能早于今天"));
    }
    let expire = effective
        .checked_add_months(chrono::Months::new(term_months as u32))
        .ok_or_else(|| AppError::business("保障期超出可支持范围"))?;
    Ok((effective, expire))
}

/// 整期保费试算：`premium = round(保额 × rate, 2)`；费率与结果都必须为正。
fn compute_premium(amount: Decimal, rate: Decimal) -> Result<Decimal> {
    if rate <= Decimal::ZERO {
        return Err(AppError::business("该产品未配置有效费率，暂不可报价"));
    }
    let premium = (amount * rate).round_dp(2);
    if premium <= Decimal::ZERO {
        return Err(AppError::business("保费计算结果无效"));
    }
    Ok(premium)
}

// ---------- helpers: 行读取 ----------

use mysql_async::Row;

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


#[cfg(test)]
mod tests {
    use super::*;

    fn d(n: i64) -> Decimal {
        Decimal::from(n)
    }

    fn date(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).expect("测试日期合法")
    }

    #[test]
    fn coverage_window_derives_expire_from_term_months() {
        let today = date(2026, 1, 1);
        let effective = date(2026, 1, 15);
        assert_eq!(
            coverage_window(Some(effective), 1, today).unwrap(),
            (effective, date(2026, 2, 15))
        );
        assert_eq!(
            coverage_window(Some(effective), 6, today).unwrap(),
            (effective, date(2026, 7, 15))
        );
        assert_eq!(
            coverage_window(Some(effective), 24, today).unwrap(),
            (effective, date(2028, 1, 15))
        );
    }

    #[test]
    fn coverage_window_clamps_to_month_end() {
        // 2024 为闰年：1 月 31 日 + 1 个月 = 2 月 29 日
        let today = date(2024, 1, 1);
        assert_eq!(
            coverage_window(Some(date(2024, 1, 31)), 1, today).unwrap(),
            (date(2024, 1, 31), date(2024, 2, 29))
        );
    }

    #[test]
    fn coverage_window_defaults_to_today_and_rejects_backdating() {
        let today = date(2026, 5, 20);
        assert_eq!(
            coverage_window(None, 1, today).unwrap(),
            (today, date(2026, 6, 20))
        );
        let err = coverage_window(Some(date(2026, 5, 19)), 12, today).unwrap_err();
        assert_eq!(err.to_string(), "业务错误: 生效日期不能早于今天");
    }

    #[test]
    fn coverage_window_rejects_non_positive_or_overflowing_term() {
        let today = date(2026, 5, 20);
        assert!(coverage_window(Some(today), 0, today).is_err());
        assert!(coverage_window(Some(today), -1, today).is_err());
        assert!(coverage_window(Some(today), i32::MAX, today).is_err());
    }

    #[test]
    fn quote_bounds_accept_within_range_inclusive() {
        assert!(validate_quote_bounds(d(1000), 12, None, None, None, None).is_ok());
        assert!(
            validate_quote_bounds(d(100), 6, Some(d(100)), Some(d(2000)), Some(6), Some(24)).is_ok()
        );
        assert!(
            validate_quote_bounds(d(2000), 24, Some(d(100)), Some(d(2000)), Some(6), Some(24)).is_ok()
        );
    }

    #[test]
    fn quote_bounds_reject_invalid_amount_and_term() {
        assert!(validate_quote_bounds(Decimal::ZERO, 12, None, None, None, None).is_err());
        assert!(validate_quote_bounds(d(99), 12, Some(d(100)), None, None, None).is_err());
        assert!(validate_quote_bounds(d(2001), 12, None, Some(d(2000)), None, None).is_err());
        assert!(validate_quote_bounds(d(1000), 0, None, None, None, None).is_err());
        assert!(validate_quote_bounds(d(1000), 5, None, None, Some(6), None).is_err());
        assert!(validate_quote_bounds(d(1000), 25, None, None, None, Some(24)).is_err());
    }

    #[test]
    fn premium_is_amount_times_rate_rounded_to_cents() {
        assert_eq!(
            compute_premium(d(1000), Decimal::new(15, 3)).unwrap(),
            Decimal::from(15)
        );
        assert_eq!(
            compute_premium(d(100), Decimal::new(1355, 4)).unwrap(),
            Decimal::new(1355, 2)
        );
    }

    #[test]
    fn premium_requires_positive_rate_and_result() {
        assert!(compute_premium(d(1000), Decimal::ZERO).is_err());
        assert!(compute_premium(d(1000), Decimal::new(-15, 3)).is_err());
        // 1 × 0.000001 四舍五入到分后为 0，拒绝
        assert!(compute_premium(d(1), Decimal::new(1, 6)).is_err());
    }
}
