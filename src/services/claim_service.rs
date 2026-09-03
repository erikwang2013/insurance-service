//! 理赔服务（报案）
//!
//! 报案即创建理赔单：校验保单归属 → INSERT claims（状态 SUBMITTED）→ 回读。
//! 理赔单号格式：`CLM{UTC毫秒}{4位hex}`。

use chrono::{DateTime, NaiveDate, Utc};
use mysql_async::prelude::Queryable;
use mysql_async::{Row, Value};
use rust_decimal::Decimal;
use serde::Deserialize;
use uuid::Uuid;

use crate::db::db_error;
use crate::db::Db;
use crate::error::{AppError, Result};
use crate::models::claim::Claim;
use crate::models::user::User;

/// 报案请求体
#[derive(Debug, Deserialize)]
pub struct CreateClaimReq {
    pub policy_id: i64,
    pub user_id: i64,
    /// 出险日期（可空）
    pub accident_date: Option<NaiveDate>,
    /// 出险类型/原因（可空）
    pub accident_type: Option<String>,
    /// 事故描述（可空）
    pub accident_desc: Option<String>,
    /// 申请赔付金额
    pub claim_amount: Decimal,
}

/// 理赔审核请求体
#[derive(Debug, Deserialize)]
pub struct ReviewClaimReq {
    /// 审核人 id（须 OPERATOR / ADMIN）
    pub reviewer_id: i64,
    /// 审核动作：APPROVE | REJECT
    pub action: String,
    /// 核定赔付金额（APPROVE 必填且 > 0；REJECT 时忽略）
    pub approved_amount: Option<Decimal>,
    /// 审核备注（可空，REJECT 缺省存 NULL）
    pub remark: Option<String>,
}

/// 理赔服务
pub struct ClaimService {
    db: Db,
}

impl ClaimService {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// 生成理赔单号：`CLM{UTC毫秒}{4位hex}`
    fn claim_no() -> String {
        format!("CLM{}{:04x}", Utc::now().timestamp_millis(), Uuid::new_v4().as_u128() as u64 & 0xFFFF)
    }

    /// 报案：事务内校验保单归属 → INSERT claim → 回读。
    pub async fn create(&self, req: CreateClaimReq) -> Result<Claim> {
        if req.claim_amount <= Decimal::ZERO {
            return Err(AppError::business("赔付金额必须大于 0"));
        }
        let claim_no = Self::claim_no();
        let now = Utc::now();
        let accident_date_str: Option<String> =
            req.accident_date.map(|d| d.format("%Y-%m-%d").to_string());

        let claim_id: i64 = self
            .db
            .with_tx(|tx| {
                Box::pin(async move {
                    // 1) 校验保单归属，并顺带取保单关联的订单 id（claims.order_id 非空）
                    let holder: Option<(i64, i64)> = tx
                        .exec_first(
                            "SELECT user_id, order_id FROM policies WHERE id = ? AND deleted_at IS NULL LIMIT 1",
                            vec![req.policy_id],
                        )
                        .await
                        .map_err(db_error)?;
                    let (owner, order_id) = holder.ok_or_else(|| AppError::business("保单不存在"))?;
                    if owner != req.user_id {
                        return Err(AppError::Forbidden);
                    }

                    // 2) INSERT claim
                    let params: Vec<Value> = vec![
                        Value::from(&claim_no),
                        Value::from(req.policy_id),
                        Value::from(order_id),
                        Value::from(req.user_id),
                        value_opt_str(accident_date_str),
                        value_opt_str(req.accident_type),
                        value_opt_str(req.accident_desc),
                        Value::from(req.claim_amount.to_string()),
                        Value::from(Claim::STATUS_SUBMITTED.to_string()),
                        Value::from(now.format("%Y-%m-%d %H:%M:%S").to_string()),
                        Value::from(now.format("%Y-%m-%d %H:%M:%S").to_string()),
                        Value::from(now.format("%Y-%m-%d %H:%M:%S").to_string()),
                    ];

                    tx.exec_drop(
                        "INSERT INTO claims \
                         (claim_no, policy_id, order_id, user_id, accident_date, \
                          accident_type, accident_desc, claim_amount, status, \
                          submitted_at, created_at, updated_at) \
                         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                        params,
                    )
                    .await
                    .map_err(db_error)?;

                    Ok(tx.last_insert_id().unwrap_or_default() as i64)
                })
            })
            .await?;

        // 3) 回读整条理赔单
        let row: Option<Row> = self
            .db
            .conn()
            .await?
            .exec_first("SELECT * FROM claims WHERE id = ? LIMIT 1", vec![claim_id])
            .await
            .map_err(db_error)?;

        let row = row.ok_or_else(|| AppError::business("报案后回读失败"))?;
        row_to_claim(&row)
    }

    /// 审核：校验审核人角色 → 事务内校验状态并 UPDATE → 回读整行。
    pub async fn review(&self, claim_id: i64, req: ReviewClaimReq) -> Result<Claim> {
        // 1) 动作与金额校验（DB 前）
        let (status, approved_amount_str): (&str, Option<String>) = match req.action.as_str() {
            "APPROVE" => {
                let amount = req
                    .approved_amount
                    .ok_or_else(|| AppError::business("核定赔付金额必填"))?;
                if amount <= Decimal::ZERO {
                    return Err(AppError::business("核定赔付金额必须大于 0"));
                }
                (Claim::STATUS_APPROVED, Some(amount.to_string()))
            }
            "REJECT" => (Claim::STATUS_REJECTED, None),
            _ => return Err(AppError::business("action 仅支持 APPROVE / REJECT")),
        };

        // 2) 审核人须为运营/管理员
        let role: Option<String> = self
            .db
            .conn()
            .await?
            .exec_first(
                "SELECT role FROM users WHERE id = ? AND deleted_at IS NULL LIMIT 1",
                vec![req.reviewer_id],
            )
            .await
            .map_err(db_error)?;
        let is_operator = matches!(
            role.as_deref(),
            Some(User::ROLE_OPERATOR) | Some(User::ROLE_ADMIN)
        );
        if !is_operator {
            return Err(AppError::Forbidden);
        }

        // 3) 事务内：校验当前状态为 SUBMITTED → UPDATE
        let reviewer_id = req.reviewer_id;
        let remark = req.remark;
        self.db
            .with_tx(|tx| {
                Box::pin(async move {
                    let cur: Option<String> = tx
                        .exec_first(
                            "SELECT status FROM claims WHERE id = ? AND deleted_at IS NULL LIMIT 1",
                            vec![claim_id],
                        )
                        .await
                        .map_err(db_error)?;
                    match cur {
                        Some(s) if s == Claim::STATUS_SUBMITTED => {}
                        _ => return Err(AppError::business("理赔单不存在或已审核")),
                    }

                    let params: Vec<Value> = vec![
                        Value::from(status.to_string()),
                        value_opt_str(approved_amount_str),
                        Value::from(reviewer_id),
                        value_opt_str(remark),
                        Value::from(claim_id),
                    ];
                    tx.exec_drop(
                        "UPDATE claims \
                         SET status = ?, approved_amount = ?, reviewer_id = ?, \
                             review_remark = ?, updated_at = NOW() \
                         WHERE id = ?",
                        params,
                    )
                    .await
                    .map_err(db_error)?;
                    Ok(())
                })
            })
            .await?;

        // 4) 回读整条理赔单
        let row: Option<Row> = self
            .db
            .conn()
            .await?
            .exec_first("SELECT * FROM claims WHERE id = ? LIMIT 1", vec![claim_id])
            .await
            .map_err(db_error)?;
        let row = row.ok_or_else(|| AppError::business("审核后回读失败"))?;
        row_to_claim(&row)
    }

    /// 我的理赔列表（分页）
    pub async fn by_user(&self, user_id: i64, page: u32, size: u32) -> Result<Vec<Claim>> {
        let size = size.clamp(1, 100) as usize;
        let offset = ((page.max(1) as usize) - 1) * size;
        let rows: Vec<Row> = self
            .db
            .conn()
            .await?
            .exec(
                "SELECT * FROM claims WHERE user_id = ? AND deleted_at IS NULL \
                 ORDER BY created_at DESC LIMIT ? OFFSET ?",
                vec![user_id, size as i64, offset as i64],
            )
            .await
            .map_err(db_error)?;
        rows.iter().map(row_to_claim).collect::<Result<Vec<_>>>()
    }
}

// ---------- helpers ----------

fn dt_row(row: &Row, col: &str) -> DateTime<Utc> {
    row.get::<chrono::NaiveDateTime, &str>(col)
        .unwrap_or_default()
        .and_utc()
}

fn dt_opt_row(row: &Row, col: &str) -> Option<DateTime<Utc>> {
    row.get::<Option<chrono::NaiveDateTime>, &str>(col)
        .flatten()
        .map(|d| d.and_utc())
}

fn dec_opt_row(row: &Row, col: &str) -> Option<Decimal> {
    row.get::<Option<String>, &str>(col)
        .flatten()
        .and_then(|s| s.parse().ok())
}

fn date_opt_row(row: &Row, col: &str) -> Option<NaiveDate> {
    // DATE 列经 mysql_async chrono feature 直接解码为 NaiveDate（非字符串）
    row.get::<Option<NaiveDate>, &str>(col).flatten()
}

fn value_opt_str(v: Option<String>) -> Value {
    v.map(Value::from).unwrap_or(Value::NULL)
}

/// 从 Row 重建 Claim
fn row_to_claim(row: &Row) -> Result<Claim> {
    Ok(Claim {
        id: row.get("id").unwrap_or_default(),
        claim_no: row.get("claim_no").unwrap_or_default(),
        policy_id: row.get("policy_id").unwrap_or_default(),
        order_id: row.get("order_id").unwrap_or_default(),
        user_id: row.get("user_id").unwrap_or_default(),
        accident_date: date_opt_row(row, "accident_date"),
        accident_type: row.get("accident_type").flatten(),
        accident_desc: row.get("accident_desc").flatten(),
        claim_amount: dec_opt_row(row, "claim_amount").unwrap_or_default(),
        approved_amount: dec_opt_row(row, "approved_amount"),
        status: row.get("status").unwrap_or_default(),
        reviewer_id: row.get("reviewer_id").flatten(),
        review_remark: row.get("review_remark").flatten(),
        pay_ref: row.get("pay_ref").flatten(),
        submitted_at: dt_opt_row(row, "submitted_at"),
        paid_at: dt_opt_row(row, "paid_at"),
        created_at: dt_row(row, "created_at"),
        updated_at: dt_row(row, "updated_at"),
        deleted_at: dt_opt_row(row, "deleted_at"),
    })
}