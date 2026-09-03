//! policies 保单（db-schema.md §6.11）

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};

/// 保单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: i64,
    /// 保单号（对外展示）
    pub policy_no: String,
    pub order_id: i64,
    pub quote_id: i64,
    /// 投保人
    pub user_id: i64,
    /// 被保人档案
    pub holder_id: Option<i64>,
    pub product_id: i64,
    pub product_name: String,
    /// 被保人姓名
    pub holder_name: String,
    /// 被保人身份证密文（不对外序列化）
    #[serde(skip_serializing)]
    pub holder_id_card_enc: Option<Vec<u8>>,
    /// 保额
    pub insurance_amount: Decimal,
    /// 实缴保费
    pub premium: Decimal,
    pub term_months: i32,
    /// 保险起期
    pub effective_date: NaiveDate,
    /// 保险止期
    pub expire_date: NaiveDate,
    /// 状态：PENDING_ISSUE → ACTIVE → EXPIRED / CANCELLED / SURRENDERED / LAPSED
    pub status: String,
    /// 签发类型："NEW"|"RENEW"
    pub issue_type: String,
    pub is_renewable: bool,
    /// 电子保单 PDF 存储路径
    pub pdf_path: Option<String>,
    pub premium_detail: Option<serde_json::Value>,
    pub issued_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Policy {
    pub const STATUS_PENDING_ISSUE: &'static str = "PENDING_ISSUE";
    pub const STATUS_ACTIVE: &'static str = "ACTIVE";
    pub const STATUS_EXPIRED: &'static str = "EXPIRED";
    pub const STATUS_CANCELLED: &'static str = "CANCELLED";
    pub const STATUS_SURRENDERED: &'static str = "SURRENDERED";
    pub const STATUS_LAPSED: &'static str = "LAPSED";
}

/// 保单受益人（表 policy_beneficiaries 的行；身份证密文列不外泄）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Beneficiary {
    pub id: i64,
    pub policy_id: i64,
    pub name: String,
    /// 与被保人关系：SELF/SPOUSE/CHILD/PARENT/OTHER
    pub relationship: Option<String>,
    /// 受益人类型：LEGAL(法定)/NAMED(指定)
    pub beneficiary_type: String,
    /// 占比(0-100)，NAMED 时使用，同单合计=100
    pub share_percent: Option<Decimal>,
    pub sort_order: i32,
}

/// 批改-受益人变更请求体（user_id 为操作人；与仓内 POST 动作一致自证式取身份）
#[derive(Debug, Deserialize)]
pub struct EndorseBeneficiariesReq {
    pub user_id: i64,
    /// 新受益人名单（整单替换；至少 1 人，LEGAL 与 NAMED 不可混用）
    pub beneficiaries: Vec<BeneficiaryInput>,
}

/// 单条受益人输入（beneficiary_type 缺省 LEGAL）
#[derive(Debug, Clone, Deserialize)]
pub struct BeneficiaryInput {
    pub name: String,
    #[serde(default)]
    pub relationship: Option<String>,
    #[serde(default)]
    pub beneficiary_type: Option<String>,
    #[serde(default)]
    pub share_percent: Option<Decimal>,
}

impl BeneficiaryInput {
    /// 受益人类型：缺省为法定（LEGAL）
    pub fn beneficiary_type(&self) -> &str {
        self.beneficiary_type.as_deref().unwrap_or("LEGAL")
    }
}

/// 受益人名单整体校验（批改入口调用）：
/// 非空；姓名非空且 ≤64 字；关系仅限枚举；类型仅 LEGAL/NAMED 且不可混用；
/// LEGAL 不得带占比，NAMED 必须带占比（0,100] 且合计=100。
pub fn validate_beneficiaries(list: &[BeneficiaryInput]) -> Result<()> {
    if list.is_empty() {
        return Err(AppError::business("受益人名单不能为空"));
    }
    let (mut has_legal, mut has_named, mut total) = (false, false, Decimal::ZERO);
    for b in list {
        let name = b.name.trim();
        if name.is_empty() {
            return Err(AppError::business("受益人姓名不能为空"));
        }
        if name.chars().count() > 64 {
            return Err(AppError::business("受益人姓名过长(最多 64 字)"));
        }
        if let Some(r) = &b.relationship {
            if !["SELF", "SPOUSE", "CHILD", "PARENT", "OTHER"].contains(&r.as_str()) {
                return Err(AppError::business("受益人关系取值非法(仅 SELF/SPOUSE/CHILD/PARENT/OTHER)"));
            }
        }
        match b.beneficiary_type() {
            "LEGAL" => {
                has_legal = true;
                if b.share_percent.is_some() {
                    return Err(AppError::business("法定受益人无需设置占比"));
                }
            }
            "NAMED" => {
                has_named = true;
                let s = b
                    .share_percent
                    .ok_or_else(|| AppError::business("指定受益人(NAMED)必须设置占比"))?;
                if s <= Decimal::ZERO || s > Decimal::from(100) {
                    return Err(AppError::business("受益人占比须在 (0,100] 区间"));
                }
                total += s;
            }
            t => return Err(AppError::business(format!("受益人类型取值非法(仅 LEGAL/NAMED): {t}"))),
        }
    }
    if has_legal && has_named {
        return Err(AppError::business("法定(LEGAL)与指定(NAMED)受益人不可混用"));
    }
    if has_named && total != Decimal::from(100) {
        return Err(AppError::business("指定受益人占比合计须为 100"));
    }
    Ok(())
}
