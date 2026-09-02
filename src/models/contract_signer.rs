//! contract_signers 合同签署方（db-schema.md §6.14）

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 合同签署方
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractSigner {
    pub id: i64,
    pub contract_id: i64,
    /// 登录用户签署（可空）
    pub user_id: Option<i64>,
    pub name: String,
    /// 签署方类型："APPLICANT"|"INSURED"|"BENEFICIARY"|"WITNESS"
    pub signer_type: String,
    /// 签署顺序
    pub sign_order: i32,
    /// 状态：PENDING → SIGNING → SIGNED → COMPLETED / REJECTED / ABANDONED
    pub status: String,
    /// 签署链接（电子签平台）
    pub sign_url: Option<String>,
    /// 签署凭证
    pub sign_token: Option<String>,
    pub signed_at: Option<DateTime<Utc>>,
    /// 签署环境/IP/时间/落款坐标
    pub sign_detail: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
