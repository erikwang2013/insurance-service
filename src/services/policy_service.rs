//! 保单服务（交易闭环）
//!
//! 支付成功后签发保单：校验订单已 PAID → INSERT policy → 订单进 POLICY_ISSUED → 回读。
//! 另提供退保（lapse，ACTIVE → SURRENDERED）与受益人批改。状态机见 `Policy` 模型注释：
//! `PENDING_ISSUE → ACTIVE → EXPIRED / CANCELLED / SURRENDERED / LAPSED`。

use chrono::{NaiveDate, Utc};
use mysql_async::prelude::Queryable;
use mysql_async::Value;
use mysql_async::Row;
use rust_decimal::Decimal;
use serde::Deserialize;
use uuid::Uuid;

use crate::db::{
    db_error, dec_opt_row, dt_opt_row, dt_row, json_opt_row, json_val, value_opt_int, Db,
};
use crate::error::{AppError, Result};
use crate::models::policy::{validate_beneficiaries, Beneficiary, EndorseBeneficiariesReq, Policy};

/// 审计动作常量（audit_logs.action）：退保 / 批改-受益人变更
const POLICY_ACTION_LAPSE: &str = "POLICY_LAPSE";
const POLICY_ACTION_ENDORSE: &str = "POLICY_ENDORSE";

/// 签发请求体：报价取订单行自身的 quote_id，身份取 token，均不信请求体。
#[derive(Debug, Deserialize)]
pub struct IssuePolicyReq {
    pub order_id: i64,
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

    /// 签发保单：锁定订单，归属本人且已支付 → 按订单自身的 quote_id 取报价 → 落库。
    /// 订单已是 POLICY_ISSUED 时返回已有保单（幂等），不再重复签发。
    /// `user_id` 取自已认证身份（控制器从 token 取），不信请求体。
    pub async fn issue(&self, user_id: i64, req: IssuePolicyReq) -> Result<Policy> {
        let policy_no = Self::policy_no();

        let row: Option<Row> = self
            .db
            .with_tx(|tx| {
                Box::pin(async move {
                    let order: Option<Row> = tx
                        .exec_first(
                            "SELECT id, quote_id, product_id, product_name, holder_name, \
                             insurance_amount, term_months, payable_amount, status \
                             FROM orders WHERE id = ? AND user_id = ? AND deleted_at IS NULL \
                             LIMIT 1 FOR UPDATE",
                            vec![req.order_id, user_id],
                        )
                        .await
                        .map_err(db_error)?;
                    let order = order.ok_or_else(|| AppError::business("订单不存在"))?;
                    let order_status: String = order.get("status").unwrap_or_default();
                    // 幂等：同一订单重复签发返回同一张保单
                    if order_status == OrderStatus::POLICY_ISSUED {
                        let existing: Option<Row> = tx
                            .exec_first(
                                "SELECT * FROM policies WHERE order_id = ? AND deleted_at IS NULL LIMIT 1",
                                vec![req.order_id],
                            )
                            .await
                            .map_err(db_error)?;
                        let existing = existing
                            .ok_or_else(|| AppError::state_conflict("订单已签发但保单缺失"))?;
                        return Ok(Some(existing));
                    }
                    if order_status != OrderStatus::PAID {
                        return Err(AppError::business("订单未支付，无法签发保单"));
                    }
                    let quote_id: i64 = order.get("quote_id").unwrap_or_default();
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
                            vec![quote_id, user_id],
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

                    // 主键由应用层 snowflake 预生成后显式插入（全库自增迁移，见 idgen）
                    let id = crate::utils::idgen::next_id();
                    tx.exec_drop(
                        "INSERT INTO policies (id, policy_no, order_id, quote_id, user_id, holder_id, \
                         product_id, product_name, holder_name, insurance_amount, term_months, \
                         premium, effective_date, expire_date, status, issue_type, is_renewable, \
                         premium_detail, issued_at, created_at, updated_at) \
                         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                        vec![
                            id.into(),
                            Value::from(&policy_no),
                            Value::from(req.order_id),
                            Value::from(quote_id),
                            Value::from(user_id),
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
                            Value::from("NEW"),
                            Value::from(false),
                            json_val(premium_detail),
                            Value::from(&dt),
                            Value::from(&dt),
                            Value::from(&dt),
                        ],
                    )
                    .await
                    .map_err(db_error)?;

                    tx.exec_drop(
                        "UPDATE orders SET status = ?, policy_issued_at = NOW(), updated_at = NOW() \
                         WHERE id = ? AND user_id = ? AND status = ?",
                        vec![
                            Value::from(OrderStatus::POLICY_ISSUED),
                            Value::from(req.order_id),
                            Value::from(user_id),
                            Value::from(OrderStatus::PAID),
                        ],
                    )
                    .await
                    .map_err(db_error)?;
                    if tx.affected_rows() != 1 {
                        return Err(AppError::state_conflict("订单状态已变化，保单签发失败"));
                    }

                    let policy_row = tx
                        .exec_first("SELECT * FROM policies WHERE id = ? LIMIT 1", vec![id])
                        .await
                        .map_err(db_error)?;
                    Ok(policy_row)
                })
            })
            .await?;

        row.map(|r| row_to_policy(&r)).transpose()?
            .ok_or_else(|| AppError::business("保单签发后回读失败"))
    }

    /// 保单详情：按 id 且归属 `user_id` 查（未删除）。
    pub async fn by_id(&self, id: i64, user_id: i64) -> Result<Policy> {
        let row: Option<Row> = self
            .db
            .conn()
            .await?
            .exec_first(
                "SELECT * FROM policies WHERE id = ? AND user_id = ? AND deleted_at IS NULL LIMIT 1",
                vec![id, user_id],
            )
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
        rows.iter().map(row_to_policy).collect::<Result<Vec<_>>>()
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
                    // 主键由应用层 snowflake 预生成后显式插入（全库自增迁移，见 idgen）
                    let audit_id = crate::utils::idgen::next_id();
                    tx.exec_drop(
                        "INSERT INTO audit_logs (id, user_id, action, entity_type, entity_id, \
                         before_json, after_json, trace_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                        vec![
                            audit_id.into(),
                            Value::from(user_id),
                            Value::from(POLICY_ACTION_LAPSE),
                            Value::from("POLICY"),
                            Value::from(policy_id),
                            before,
                            after,
                            Value::from(crate::middleware::trace::current_trace_id()),
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
                    // 主键由应用层 snowflake 预生成后显式插入（全库自增迁移，见 idgen）
                    let id = crate::utils::idgen::next_id();
                    tx.exec_drop(
                        "INSERT INTO policy_beneficiaries (id, policy_id, name, relationship, \
                         beneficiary_type, share_percent, sort_order) VALUES (?, ?, ?, ?, ?, ?, ?)",
                        vec![
                            id.into(),
                            Value::from(policy_id), Value::from(b.name.trim()),
                            b.relationship.clone().map(Value::from).unwrap_or(Value::NULL), Value::from(b.beneficiary_type()),
                            b.share_percent.map(|d| Value::from(d.to_string())).unwrap_or(Value::NULL), Value::from(i as i32),
                        ],
                    ).await.map_err(db_error)?;
                }
                let new_rows: Vec<Row> = tx.exec(sel_b, vec![policy_id]).await.map_err(db_error)?;
                // 主键由应用层 snowflake 预生成后显式插入（全库自增迁移，见 idgen）
                let audit_id = crate::utils::idgen::next_id();
                tx.exec_drop(
                    "INSERT INTO audit_logs (id, user_id, action, entity_type, entity_id, \
                     before_json, after_json, trace_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                    vec![
                        audit_id.into(),
                        Value::from(user_id), Value::from(POLICY_ACTION_ENDORSE),
                        Value::from("POLICY"), Value::from(policy_id),
                        before, json_val(Some(bens_json(&new_rows))),
                        Value::from(crate::middleware::trace::current_trace_id()),
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

fn bens_json(rows: &[Row]) -> serde_json::Value {
    let list = rows.iter().map(row_to_beneficiary).collect::<Result<Vec<_>>>();
    serde_json::to_value(list.unwrap_or_default()).unwrap_or(serde_json::Value::Null)
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
