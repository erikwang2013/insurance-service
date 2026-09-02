//! policy_holders 被保人档案（db-schema.md §6.2）

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// 被保人档案（可独立于投保账户，支持"为他人投保"）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyHolder {
    pub id: i64,
    /// 关联投保人账户；为他人投保时为 NULL
    pub user_id: Option<i64>,
    pub name: String,
    /// 身份证密文（不对外序列化）
    #[serde(skip_serializing)]
    pub id_card_enc: Option<Vec<u8>>,
    /// 证件类型："ID_CARD"|"PASSPORT"|"OTHER"
    pub id_type: String,
    /// 性别："MALE"|"FEMALE"|"UNKNOWN"
    pub gender: Option<String>,
    pub birthday: Option<NaiveDate>,
    /// 手机号密文（不对外序列化）
    #[serde(skip_serializing)]
    pub phone_enc: Option<Vec<u8>>,
    pub email: Option<String>,
    pub address: Option<String>,
    /// 与投保人关系："SELF"|"SPOUSE"|"CHILD"|"PARENT"|"OTHER"
    pub relationship: Option<String>,
    /// 状态："ACTIVE"|"DELETED"
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}
