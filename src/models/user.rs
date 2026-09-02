//! users 用户账户（db-schema.md §6.1）

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 用户 / 账户（投保人账户）
///
/// 敏感字段 `phone_enc` / `id_card_enc` 存 AES 密文，仅后端解密，不参与 API JSON 输出；
/// `phone_masked` 为脱敏展示值。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// 主键（BIGINT UNSIGNED）
    pub id: i64,
    /// 用户名（唯一）
    pub username: String,
    /// 手机号 AES 密文（VARBINARY(512)），不对外序列化
    #[serde(skip_serializing)]
    pub phone_enc: Option<Vec<u8>>,
    /// 身份证号 AES 密文（VARBINARY(1024)），不对外序列化
    #[serde(skip_serializing)]
    pub id_card_enc: Option<Vec<u8>>,
    /// 脱敏手机号（138****1234）
    pub phone_masked: Option<String>,
    /// 密码哈希（argon2/bcrypt），不对外序列化
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub email: Option<String>,
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
    /// 角色："USER" | "ADMIN" | "OPERATOR"
    pub role: String,
    /// 状态："ACTIVE" | "DISABLED" | "FROZEN"
    pub status: String,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl User {
    pub const ROLE_USER: &'static str = "USER";
    pub const ROLE_ADMIN: &'static str = "ADMIN";
    pub const ROLE_OPERATOR: &'static str = "OPERATOR";
    pub const STATUS_ACTIVE: &'static str = "ACTIVE";
    pub const STATUS_DISABLED: &'static str = "DISABLED";
    pub const STATUS_FROZEN: &'static str = "FROZEN";
}
