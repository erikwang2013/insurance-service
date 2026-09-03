//! claims 理赔（db-schema.md §6.15）

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 理赔
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    pub id: i64,
    /// 理赔单号（唯一）
    pub claim_no: String,
    pub policy_id: i64,
    pub order_id: i64,
    /// 报案人
    pub user_id: i64,
    /// 出险日期
    pub accident_date: Option<NaiveDate>,
    /// 出险类型/原因
    pub accident_type: Option<String>,
    /// 事故描述
    pub accident_desc: Option<String>,
    /// 申请赔付金额
    pub claim_amount: Decimal,
    /// 核定赔付金额
    pub approved_amount: Option<Decimal>,
    /// 状态：SUBMITTED → UNDER_REVIEW → PENDING_INFO → REVIEWING
    ///            → APPROVED → PAID / REJECTED / CLOSED / WITHDRAWN
    pub status: String,
    pub reviewer_id: Option<i64>,
    pub review_remark: Option<String>,
    /// 关联 payments.id 或渠道回执
    pub pay_ref: Option<String>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// 理赔资料（claim_documents，db-schema.md 扩展）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimDocument {
    pub id: i64,
    pub claim_id: i64,
    /// 资料类型：申报单/病历/发票/其他（自由文本）
    pub doc_type: String,
    /// 客户端原始文件名
    pub file_name: String,
    /// 对象键或占位 URL（本阶段仅存元数据，不接真实上传）
    pub file_key: String,
    pub created_at: DateTime<Utc>,
}

impl Claim {
    pub const STATUS_SUBMITTED: &'static str = "SUBMITTED";
    pub const STATUS_UNDER_REVIEW: &'static str = "UNDER_REVIEW";
    pub const STATUS_PENDING_INFO: &'static str = "PENDING_INFO";
    pub const STATUS_REVIEWING: &'static str = "REVIEWING";
    pub const STATUS_APPROVED: &'static str = "APPROVED";
    pub const STATUS_PAID: &'static str = "PAID";
    pub const STATUS_REJECTED: &'static str = "REJECTED";
    pub const STATUS_CLOSED: &'static str = "CLOSED";
    pub const STATUS_WITHDRAWN: &'static str = "WITHDRAWN";
}
