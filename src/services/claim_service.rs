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
use crate::models::claim::{Claim, ClaimDocument};
use crate::models::policy::Policy;
use crate::models::user::User;

/// 报案请求体
#[derive(Debug, Deserialize)]
pub struct CreateClaimReq {
    pub policy_id: i64,
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
    /// 审核动作：APPROVE | REJECT
    pub action: String,
    /// 核定赔付金额（APPROVE 必填且 > 0；REJECT 时忽略）
    pub approved_amount: Option<Decimal>,
    /// 审核备注（可空，REJECT 缺省存 NULL）
    pub remark: Option<String>,
}

/// 理赔资料上传请求体（操作者身份取自已认证 token）
#[derive(Debug, Deserialize)]
pub struct UploadDocumentReq {
    /// 资料类型（如 申报单/病历/发票/其他）
    pub doc_type: String,
    /// 客户端原始文件名
    pub file_name: String,
    /// 对象键或占位 URL（仅存元数据，不接真实上传）
    pub file_key: String,
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
    /// `user_id` 取自已认证身份（控制器从 token 取），不信请求体。
    pub async fn create(&self, user_id: i64, req: CreateClaimReq) -> Result<Claim> {
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
                    // 1) 校验保单归属/状态/保障期，并取订单 id（claims.order_id 非空）与保额上限
                    let policy: Option<Row> = tx
                        .exec_first(
                            "SELECT user_id, order_id, status, insurance_amount, \
                             effective_date, expire_date \
                             FROM policies WHERE id = ? AND deleted_at IS NULL LIMIT 1",
                            vec![req.policy_id],
                        )
                        .await
                        .map_err(db_error)?;
                    let policy = policy.ok_or_else(|| AppError::business("保单不存在"))?;
                    let owner: i64 = policy.get("user_id").unwrap_or_default();
                    if owner != user_id {
                        return Err(AppError::Forbidden);
                    }
                    let policy_status: String = policy.get("status").unwrap_or_default();
                    if policy_status != Policy::STATUS_ACTIVE {
                        return Err(AppError::business("保单当前状态不可报案"));
                    }
                    let order_id: i64 = policy.get("order_id").unwrap_or_default();
                    let insurance_amount: Decimal =
                        dec_opt_row(&policy, "insurance_amount").unwrap_or_default();
                    validate_accident_window(
                        req.accident_date,
                        date_opt_row(&policy, "effective_date"),
                        date_opt_row(&policy, "expire_date"),
                        Utc::now().date_naive(),
                    )?;
                    validate_claim_amount(req.claim_amount, insurance_amount)?;

                    // 2) INSERT claim（主键由应用层 snowflake 预生成后显式插入，见 idgen）
                    let id = crate::utils::idgen::next_id();
                    let params: Vec<Value> = vec![
                        id.into(),
                        Value::from(&claim_no),
                        Value::from(req.policy_id),
                        Value::from(order_id),
                        Value::from(user_id),
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
                         (id, claim_no, policy_id, order_id, user_id, accident_date, \
                          accident_type, accident_desc, claim_amount, status, \
                          submitted_at, created_at, updated_at) \
                         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                        params,
                    )
                    .await
                    .map_err(db_error)?;

                    Ok(id)
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
    /// `reviewer_id` 取自已认证身份（控制器从 token 取），不信请求体。
    pub async fn review(&self, claim_id: i64, reviewer_id: i64, req: ReviewClaimReq) -> Result<Claim> {
        // 1) 动作与金额正数校验（DB 前）；上限须对照理赔单/保额，事务内再做
        let (status, approved_amount): (&str, Option<Decimal>) = match req.action.as_str() {
            "APPROVE" => {
                let amount = req
                    .approved_amount
                    .ok_or_else(|| AppError::business("核定赔付金额必填"))?;
                if amount <= Decimal::ZERO {
                    return Err(AppError::business("核定赔付金额必须大于 0"));
                }
                (Claim::STATUS_APPROVED, Some(amount))
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
                vec![reviewer_id],
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

        // 3) 事务内：读理赔单/保额 → 校验核定金额上限 → 带状态守卫 UPDATE（并发下只有一次生效）
        let remark = req.remark;
        self.db
            .with_tx(|tx| {
                Box::pin(async move {
                    let cur: Option<Row> = tx
                        .exec_first(
                            "SELECT c.claim_amount, c.status, p.insurance_amount \
                             FROM claims c JOIN policies p ON p.id = c.policy_id \
                             WHERE c.id = ? AND c.deleted_at IS NULL LIMIT 1",
                            vec![claim_id],
                        )
                        .await
                        .map_err(db_error)?;
                    let cur = cur.ok_or_else(|| AppError::business("理赔单不存在"))?;
                    let status_now: String = cur.get("status").unwrap_or_default();
                    if status_now != Claim::STATUS_SUBMITTED {
                        return Err(AppError::state_conflict("理赔单已审核"));
                    }
                    if let Some(amount) = approved_amount {
                        let claim_amount: Decimal =
                            dec_opt_row(&cur, "claim_amount").unwrap_or_default();
                        let insurance_amount: Decimal =
                            dec_opt_row(&cur, "insurance_amount").unwrap_or_default();
                        validate_approved_amount(amount, claim_amount, insurance_amount)?;
                    }

                    let params: Vec<Value> = vec![
                        Value::from(status.to_string()),
                        value_opt_str(approved_amount.map(|d| d.to_string())),
                        Value::from(reviewer_id),
                        value_opt_str(remark),
                        Value::from(claim_id),
                        Value::from(Claim::STATUS_SUBMITTED),
                    ];
                    tx.exec_drop(
                        "UPDATE claims \
                         SET status = ?, approved_amount = ?, reviewer_id = ?, \
                             review_remark = ?, updated_at = NOW() \
                         WHERE id = ? AND status = ?",
                        params,
                    )
                    .await
                    .map_err(db_error)?;
                    if tx.affected_rows() != 1 {
                        return Err(AppError::state_conflict("理赔单已审核"));
                    }
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

    /// 校验理赔存在且操作者为其报案人或 ADMIN（他人 USER 一律拒绝）。
    /// 供资料上传/列表共用。
    async fn check_claim_access(
        &self,
        claim_id: i64,
        actor_id: i64,
    ) -> Result<()> {
        let mut conn = self.db.conn().await?;
        let owner: Option<i64> = conn
            .exec_first(
                "SELECT user_id FROM claims WHERE id = ? AND deleted_at IS NULL LIMIT 1",
                vec![claim_id],
            )
            .await
            .map_err(db_error)?;
        let owner = owner.ok_or_else(|| AppError::business("理赔单不存在"))?;
        if owner == actor_id {
            return Ok(());
        }
        // 非报案人：仅 ADMIN 放行（对齐任务要求）
        let role: Option<String> = conn
            .exec_first(
                "SELECT role FROM users WHERE id = ? AND deleted_at IS NULL LIMIT 1",
                vec![actor_id],
            )
            .await
            .map_err(db_error)?;
        if matches!(role.as_deref(), Some(User::ROLE_ADMIN)) {
            Ok(())
        } else {
            Err(AppError::Forbidden)
        }
    }

    /// 上传理赔资料：校验归属 → INSERT 元数据 → 回读整行。
    /// `user_id` 取自已认证身份（控制器从 token 取），不信请求体。
    pub async fn add_document(
        &self,
        claim_id: i64,
        user_id: i64,
        req: UploadDocumentReq,
    ) -> Result<ClaimDocument> {
        // DB 前校验（doc_type≤32、file_name/file_key≤255，与表列宽一致）
        let doc_type = req.doc_type.trim();
        let file_name = req.file_name.trim();
        let file_key = req.file_key.trim();
        if doc_type.is_empty() {
            return Err(AppError::business("资料类型必填"));
        }
        if file_name.is_empty() {
            return Err(AppError::business("文件名为空"));
        }
        if file_key.is_empty() {
            return Err(AppError::business("file_key 必填"));
        }
        if doc_type.len() > 32 {
            return Err(AppError::business("资料类型最长 32 字符"));
        }
        if file_name.len() > 255 || file_key.len() > 255 {
            return Err(AppError::business("文件名/file_key 最长 255 字符"));
        }
        self.check_claim_access(claim_id, user_id).await?;

        let mut conn = self.db.conn().await?;
        // 主键由应用层 snowflake 预生成后显式插入（全库自增迁移，见 idgen）
        let doc_id = crate::utils::idgen::next_id();
        conn.exec_drop(
            "INSERT INTO claim_documents (id, claim_id, doc_type, file_name, file_key) VALUES (?, ?, ?, ?, ?)",
            vec![
                doc_id.into(),
                Value::from(claim_id),
                Value::from(doc_type),
                Value::from(file_name),
                Value::from(file_key),
            ],
        )
        .await
        .map_err(db_error)?;

        let row: Option<Row> = conn
            .exec_first(
                "SELECT * FROM claim_documents WHERE id = ? LIMIT 1",
                vec![doc_id],
            )
            .await
            .map_err(db_error)?;
        let row = row.ok_or_else(|| AppError::business("上传后回读失败"))?;
        row_to_document(&row)
    }

    /// 理赔资料列表（全量，理赔单资料量小；归属校验与上传一致）。
    pub async fn list_documents(&self, claim_id: i64, user_id: i64) -> Result<Vec<ClaimDocument>> {
        self.check_claim_access(claim_id, user_id).await?;
        let rows: Vec<Row> = self
            .db
            .conn()
            .await?
            .exec(
                "SELECT * FROM claim_documents WHERE claim_id = ? ORDER BY created_at DESC, id DESC",
                vec![claim_id],
            )
            .await
            .map_err(db_error)?;
        rows.iter().map(row_to_document).collect::<Result<Vec<_>>>()
    }
}

// ---------- helpers ----------

/// 出险日期必须落在保障期内（含首尾日）；缺失或倒置的保障期视为无效。
/// `accident_date` 缺省时用 `today` 校验。
fn validate_accident_window(
    accident_date: Option<NaiveDate>,
    effective_date: Option<NaiveDate>,
    expire_date: Option<NaiveDate>,
    today: NaiveDate,
) -> Result<()> {
    let (Some(effective), Some(expire)) = (effective_date, expire_date) else {
        return Err(AppError::business("保单保障期无效"));
    };
    if effective > expire {
        return Err(AppError::business("保单保障期无效"));
    }
    let date = accident_date.unwrap_or(today);
    if date < effective || date > expire {
        return Err(AppError::business("出险日期不在保障期内"));
    }
    Ok(())
}

/// 申请赔付金额须为正且不超过保额。
fn validate_claim_amount(claim_amount: Decimal, insurance_amount: Decimal) -> Result<()> {
    if claim_amount <= Decimal::ZERO {
        return Err(AppError::business("赔付金额必须大于 0"));
    }
    if claim_amount > insurance_amount {
        return Err(AppError::business("赔付金额不能超过保额"));
    }
    Ok(())
}

/// 核定赔付金额须为正，且不超过申请金额与保额。
fn validate_approved_amount(
    approved_amount: Decimal,
    claim_amount: Decimal,
    insurance_amount: Decimal,
) -> Result<()> {
    if approved_amount <= Decimal::ZERO {
        return Err(AppError::business("核定赔付金额必须大于 0"));
    }
    if approved_amount > claim_amount {
        return Err(AppError::business("核定赔付金额不能超过申请金额"));
    }
    if approved_amount > insurance_amount {
        return Err(AppError::business("核定赔付金额不能超过保额"));
    }
    Ok(())
}

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

/// 从 Row 重建 ClaimDocument
fn row_to_document(row: &Row) -> Result<ClaimDocument> {
    Ok(ClaimDocument {
        id: row.get("id").unwrap_or_default(),
        claim_id: row.get("claim_id").unwrap_or_default(),
        doc_type: row.get("doc_type").unwrap_or_default(),
        file_name: row.get("file_name").unwrap_or_default(),
        file_key: row.get("file_key").unwrap_or_default(),
        created_at: dt_row(row, "created_at"),
    })
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
    fn accident_date_must_fall_inside_coverage_period() {
        let start = date(2026, 1, 1);
        let end = date(2026, 12, 31);
        let today = date(2026, 6, 1);
        // 缺省用今天；首尾日含在保障期内
        assert!(validate_accident_window(None, Some(start), Some(end), today).is_ok());
        assert!(validate_accident_window(Some(start), Some(start), Some(end), today).is_ok());
        assert!(validate_accident_window(Some(end), Some(start), Some(end), today).is_ok());
        // 保障期外
        assert!(
            validate_accident_window(Some(date(2025, 12, 31)), Some(start), Some(end), today)
                .is_err()
        );
        assert!(
            validate_accident_window(Some(date(2027, 1, 1)), Some(start), Some(end), today).is_err()
        );
        // 缺省日期为今天，但今天不在保障期内
        assert!(
            validate_accident_window(None, Some(start), Some(end), date(2027, 1, 1)).is_err()
        );
        // 保障期缺失或倒置
        assert!(validate_accident_window(Some(start), None, Some(end), today).is_err());
        assert!(validate_accident_window(Some(start), Some(end), Some(start), today).is_err());
    }

    #[test]
    fn claim_amount_must_be_positive_and_within_insurance_amount() {
        assert!(validate_claim_amount(d(1), d(100)).is_ok());
        assert!(validate_claim_amount(d(100), d(100)).is_ok());
        assert!(validate_claim_amount(Decimal::ZERO, d(100)).is_err());
        assert!(validate_claim_amount(d(-1), d(100)).is_err());
        assert!(validate_claim_amount(d(101), d(100)).is_err());
    }

    #[test]
    fn approved_amount_must_be_positive_and_within_caps() {
        assert!(validate_approved_amount(d(50), d(80), d(100)).is_ok());
        assert!(validate_approved_amount(d(80), d(80), d(100)).is_ok());
        assert!(validate_approved_amount(Decimal::ZERO, d(80), d(100)).is_err());
        assert!(validate_approved_amount(d(81), d(80), d(100)).is_err());
        // 申请金额低于核定上限但保额低于核定金额
        assert!(validate_approved_amount(d(95), d(200), d(90)).is_err());
    }
}
