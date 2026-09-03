//! 保单服务（交易闭环）
//!
//! 支付成功后签发保单：校验订单已 PAID → INSERT policy → 订单进 POLICY_ISSUED → 回读。
//! 另提供续保（renew，原单生成一条 RENEW 保单并接续保障期）与退保（lapse，
//! ACTIVE → SURRENDERED）。状态机见 `Policy` 模型注释：
//! `PENDING_ISSUE → ACTIVE → EXPIRED / CANCELLED / SURRENDERED / LAPSED`。

use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, Utc};
use mysql_async::prelude::Queryable;
use mysql_async::Value;
use mysql_async::Row;
use rust_decimal::Decimal;
use serde::Deserialize;
use uuid::Uuid;

use crate::db::db_error;
use crate::db::Db;
use crate::error::{AppError, Result};
use crate::models::policy::{validate_beneficiaries, Beneficiary, EndorseBeneficiariesReq, Policy};

/// 签发类型常量（policies.issue_type）：续保单
const ISSUE_TYPE_RENEW: &str = "RENEW";
/// 审计动作常量（audit_logs.action）：退保 / 批改-受益人变更
const POLICY_ACTION_LAPSE: &str = "POLICY_LAPSE";
const POLICY_ACTION_ENDORSE: &str = "POLICY_ENDORSE";

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
                    let effective_date: NaiveDate = q
                        .get::<Option<NaiveDate>, &str>("effective_date")
                        .flatten()
                        .unwrap_or_else(|| Utc::now().date_naive());
                    let expire_date: NaiveDate = q
                        .get::<Option<NaiveDate>, &str>("expire_date")
                        .flatten()
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

                    // 先取 INSERT 自增 id 再做 UPDATE：last_insert_id 读最近一次
                    // OK 包，UPDATE 会把 insert_id 归零，晚取会拿到 0 导致回读落空。
                    let pid = tx.last_insert_id().unwrap_or_default() as i64;
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

    /// 续保：校验原保单归属本人、is_renewable=1 且状态可续（ACTIVE 在保 / EXPIRED 已满期）
    /// 后，生成一条 issue_type=RENEW 的新保单：
    /// - 保障起期：原单仍在保 → 原止期次日无缝接续；原单止期已过 → 从今天起保；
    /// - 保障止期：起期 + term_months 个月（对齐原单缴费期）；
    /// - 初始状态 ACTIVE（与 issue() 新单一致），is_renewable 继承原单。
    ///
    /// 简化说明（续保不新建订单/报价）：order_id/quote_id 沿用原单以满足外键约束，
    /// 保额/保费沿用原单数值；若需按新报价续保，应上层先走报价-下单-支付再签发。
    pub async fn renew(&self, user_id: i64, policy_id: i64) -> Result<Policy> {
        let new_id: i64 = self
            .db
            .with_tx(|tx| {
                Box::pin(async move {
                    // 1) 取原保单并校验归属
                    let row: Option<Row> = tx
                        .exec_first(
                            "SELECT * FROM policies WHERE id = ? AND deleted_at IS NULL LIMIT 1",
                            vec![policy_id],
                        )
                        .await
                        .map_err(db_error)?;
                    let old = match row {
                        Some(r) => row_to_policy(&r)?,
                        None => return Err(AppError::business("保单不存在")),
                    };
                    if old.user_id != user_id {
                        return Err(AppError::Forbidden);
                    }

                    // 2) 可续校验：开放续保标记 + 状态在保或已满期
                    if !old.is_renewable {
                        return Err(AppError::business("该保单不支持续保"));
                    }
                    let can_renew = old.status == Policy::STATUS_ACTIVE
                        || old.status == Policy::STATUS_EXPIRED;
                    if !can_renew {
                        return Err(AppError::state_conflict("当前保单状态不可续保"));
                    }

                    // 3) 新保障期：原止期次日与今天取较晚者（无缝接续 / 过期从今天起）
                    let today = Utc::now().date_naive();
                    let effective = old
                        .expire_date
                        .checked_add_days(chrono::Days::new(1))
                        .unwrap_or(old.expire_date)
                        .max(today);
                    let expire = add_months(effective, old.term_months);

                    // 4) 生成续保保单行
                    let policy_no = Self::policy_no();
                    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
                    let ed = effective.format("%Y-%m-%d").to_string();
                    let xd = expire.format("%Y-%m-%d").to_string();
                    tx.exec_drop(
                        "INSERT INTO policies (policy_no, order_id, quote_id, user_id, holder_id, \
                         product_id, product_name, holder_name, insurance_amount, term_months, \
                         premium, effective_date, expire_date, status, issue_type, is_renewable, \
                         premium_detail, issued_at, created_at, updated_at) \
                         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                        vec![
                            Value::from(&policy_no),
                            Value::from(old.order_id),
                            Value::from(old.quote_id),
                            Value::from(user_id),
                            value_opt_int(old.holder_id),
                            Value::from(old.product_id),
                            Value::from(&old.product_name),
                            Value::from(&old.holder_name),
                            Value::from(old.insurance_amount.to_string()),
                            Value::from(old.term_months),
                            Value::from(old.premium.to_string()),
                            Value::from(&ed),
                            Value::from(&xd),
                            Value::from(Policy::STATUS_ACTIVE.to_string()),
                            Value::from(ISSUE_TYPE_RENEW),
                            Value::from(old.is_renewable),
                            json_val(old.premium_detail),
                            Value::from(&now),
                            Value::from(&now),
                            Value::from(&now),
                        ],
                    )
                    .await
                    .map_err(db_error)?;

                    Ok(tx.last_insert_id().unwrap_or_default() as i64)
                })
            })
            .await?;

        self.by_id(new_id).await
    }

    /// 退保：校验原保单归属本人且状态为 ACTIVE（在保）后，置为 SURRENDERED（退保）。
    ///
    /// - 状态机中 LAPSED 为未缴保费等致保单失效的另一终态，用户主动退保走 SURRENDERED；
    /// - policies 无原因列，退保原因（含前后状态快照）写入 audit_logs（action=POLICY_LAPSE）；
    /// - 已退保/已失效/未生效等非在保状态一律拒绝（重复退保在此被拦）。
    pub async fn lapse(
        &self,
        user_id: i64,
        policy_id: i64,
        reason: Option<String>,
    ) -> Result<Policy> {
        let row: Option<Row> = self
            .db
            .with_tx(|tx| {
                Box::pin(async move {
                    // 1) 取原保单并校验归属
                    let cur: Option<Row> = tx
                        .exec_first(
                            "SELECT * FROM policies WHERE id = ? AND deleted_at IS NULL LIMIT 1",
                            vec![policy_id],
                        )
                        .await
                        .map_err(db_error)?;
                    let old = match cur {
                        Some(r) => row_to_policy(&r)?,
                        None => return Err(AppError::business("保单不存在")),
                    };
                    if old.user_id != user_id {
                        return Err(AppError::Forbidden);
                    }
                    if old.status != Policy::STATUS_ACTIVE {
                        return Err(AppError::state_conflict("当前保单状态不可退保(仅 ACTIVE 在保可退)"));
                    }

                    // 2) 状态流转 ACTIVE → SURRENDERED
                    tx.exec_drop(
                        "UPDATE policies SET status = ?, updated_at = NOW() WHERE id = ?",
                        vec![
                            Value::from(Policy::STATUS_SURRENDERED.to_string()),
                            Value::from(policy_id),
                        ],
                    )
                    .await
                    .map_err(db_error)?;

                    // 3) 退保原因入审计日志（policies 无原因列）
                    let before = json_val(Some(serde_json::json!({ "status": old.status })));
                    let after = json_val(Some(serde_json::json!({
                        "status": Policy::STATUS_SURRENDERED,
                        "reason": reason,
                    })));
                    tx.exec_drop(
                        "INSERT INTO audit_logs (user_id, action, entity_type, entity_id, \
                         before_json, after_json) VALUES (?, ?, ?, ?, ?, ?)",
                        vec![
                            Value::from(user_id),
                            Value::from(POLICY_ACTION_LAPSE),
                            Value::from("POLICY"),
                            Value::from(policy_id),
                            before,
                            after,
                        ],
                    )
                    .await
                    .map_err(db_error)?;

                    // 4) 事务内回读
                    tx.exec_first("SELECT * FROM policies WHERE id = ? LIMIT 1", vec![policy_id])
                        .await
                        .map_err(db_error)
                })
            })
            .await?;

        row.map(|r| row_to_policy(&r))
            .transpose()?
            .ok_or_else(|| AppError::business("退保后回读失败"))
    }

    /// 批改-受益人变更：整单替换受益人（先删后插，快照入 audit_logs）；保单须 ACTIVE 且归属请求用户
    pub async fn endorse_beneficiaries(&self, user_id: i64, policy_id: i64,
        req: EndorseBeneficiariesReq) -> Result<(Policy, Vec<Beneficiary>)> {
        validate_beneficiaries(&req.beneficiaries)?;
        let sel_p = "SELECT * FROM policies WHERE id = ? AND deleted_at IS NULL LIMIT 1";
        let sel_b = "SELECT id, name, relationship, beneficiary_type, share_percent, sort_order \
                     FROM policy_beneficiaries WHERE policy_id = ? ORDER BY sort_order, id";
        let (policy, new_rows) = self.db.with_tx(|tx| {
            Box::pin(async move {
                let old = match tx.exec_first(sel_p, vec![policy_id]).await.map_err(db_error)? {
                    Some(r) => row_to_policy(&r)?,
                    None => return Err(AppError::business("保单不存在")),
                };
                if old.user_id != user_id {
                    return Err(AppError::Forbidden);
                }
                if old.status != Policy::STATUS_ACTIVE {
                    return Err(AppError::state_conflict("当前保单状态不可批改(仅 ACTIVE 在保可变更受益人)"));
                }
                // 快照与返回同源(同列回读)；id_card_enc 保全密文不随批改进出
                let old_rows: Vec<Row> = tx.exec(sel_b, vec![policy_id]).await.map_err(db_error)?;
                let before = json_val(Some(bens_json(&old_rows)));
                tx.exec_drop("DELETE FROM policy_beneficiaries WHERE policy_id = ?", vec![policy_id])
                    .await.map_err(db_error)?;
                for (i, b) in req.beneficiaries.iter().enumerate() {
                    tx.exec_drop(
                        "INSERT INTO policy_beneficiaries (policy_id, name, relationship, \
                         beneficiary_type, share_percent, sort_order) VALUES (?, ?, ?, ?, ?, ?)",
                        vec![
                            Value::from(policy_id), Value::from(b.name.trim()),
                            b.relationship.clone().map(Value::from).unwrap_or(Value::NULL), Value::from(b.beneficiary_type()),
                            b.share_percent.map(|d| Value::from(d.to_string())).unwrap_or(Value::NULL), Value::from(i as i32),
                        ],
                    ).await.map_err(db_error)?;
                }
                let new_rows: Vec<Row> = tx.exec(sel_b, vec![policy_id]).await.map_err(db_error)?;
                tx.exec_drop(
                    "INSERT INTO audit_logs (user_id, action, entity_type, entity_id, \
                     before_json, after_json) VALUES (?, ?, ?, ?, ?, ?)",
                    vec![
                        Value::from(user_id), Value::from(POLICY_ACTION_ENDORSE),
                        Value::from("POLICY"), Value::from(policy_id),
                        before, json_val(Some(bens_json(&new_rows))),
                    ],
                ).await.map_err(db_error)?;
                Ok((old, new_rows))
            })
        }).await?;
        Ok((policy, new_rows.iter().map(row_to_beneficiary).collect::<Result<Vec<_>>>()?))
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

/// 日期加整月：跨年进位，目标月天数不足时钳制到月末（如 1/31 + 1 月 → 2/28/29）。
fn add_months(d: NaiveDate, months: i32) -> NaiveDate {
    let months0 = d.year() * 12 + d.month0() as i32 + months;
    let (y, m0) = (months0.div_euclid(12), months0.rem_euclid(12) as u32);
    let last = if m0 == 1 && (y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)) {
        29
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][m0 as usize]
    };
    NaiveDate::from_ymd_opt(y, m0 + 1, d.day().min(last)).expect("月份日期恒合法")
}

fn bens_json(rows: &[Row]) -> serde_json::Value {
    let list = rows.iter().map(row_to_beneficiary).collect::<Result<Vec<_>>>();
    serde_json::to_value(list.unwrap_or_default()).unwrap_or_else(|_| serde_json::Value::Null)
}

fn row_to_beneficiary(row: &Row) -> Result<Beneficiary> {
    Ok(Beneficiary {
        id: row.get("id").unwrap_or_default(),
        policy_id: row.get("policy_id").unwrap_or_default(),
        name: row.get("name").unwrap_or_default(),
        relationship: row.get("relationship").flatten(),
        beneficiary_type: row.get("beneficiary_type").unwrap_or_default(),
        share_percent: dec_opt_row(row, "share_percent"),
        sort_order: row.get("sort_order").unwrap_or_default(),
    })
}

fn row_to_policy(row: &Row) -> Result<Policy> {
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
        effective_date: row.get::<Option<NaiveDate>, &str>("effective_date").flatten().unwrap_or_default(),
        expire_date: row.get::<Option<NaiveDate>, &str>("expire_date").flatten().unwrap_or_default(),
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
