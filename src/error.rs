//! 统一错误枚举 + thiserror（对齐 backend-architecture.md §6.2）

use std::fmt;

/// 业务/系统统一错误类型。
///
/// 所有错误经统一 handler 转为 `ResponseEnvelope`（见 `response.rs`），
/// Controller 只 `return Err(...)`，由框架层兜底序列化。
#[derive(Debug)]
pub enum AppError {
    /// 参数校验失败（HTTP 400，code 40000）
    Validation(String),
    /// 未认证（HTTP 401，code 40100）
    Unauthorized,
    /// 无权限（HTTP 403，code 40300）
    Forbidden,
    /// 资源不存在（HTTP 404，code 40400）
    NotFound,
    /// 非法状态流转（HTTP 409，code 40900）
    StateConflict(String),
    /// 业务错误（HTTP 400，code 40001）
    Business(String),
    /// 安全检测拦截（HTTP 403，code 40301）
    SecurityRejected(String),
    /// 支付失败（HTTP 422，code 42201）
    Payment(String),
    /// 电子签失败（HTTP 422，code 42202）
    Esign(String),
    /// 搜索失败（HTTP 422，code 42203）
    Search(String),
    /// 数据库错误（HTTP 500，code 50000）
    Db(String),
    /// 内部错误（HTTP 500，code 50000）
    Internal(Box<dyn std::error::Error + Send + Sync>),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Validation(m) => write!(f, "参数校验失败: {m}"),
            AppError::Unauthorized => write!(f, "未认证"),
            AppError::Forbidden => write!(f, "无权限"),
            AppError::NotFound => write!(f, "资源不存在"),
            AppError::StateConflict(m) => write!(f, "状态冲突: {m}"),
            AppError::Business(m) => write!(f, "业务错误: {m}"),
            AppError::SecurityRejected(m) => write!(f, "安全检测拦截: {m}"),
            AppError::Payment(m) => write!(f, "支付失败: {m}"),
            AppError::Esign(m) => write!(f, "电子签失败: {m}"),
            AppError::Search(m) => write!(f, "搜索失败: {m}"),
            AppError::Db(m) => write!(f, "数据库错误: {m}"),
            AppError::Internal(e) => write!(f, "内部错误: {e}"),
        }
    }
}

impl std::error::Error for AppError {}

impl AppError {
    /// 业务错误码（对齐 backend-architecture.md §6.3 映射表）
    pub fn code(&self) -> i32 {
        match self {
            AppError::Validation(_) => 40000,
            AppError::Unauthorized => 40100,
            AppError::Forbidden => 40300,
            AppError::SecurityRejected(_) => 40301,
            AppError::NotFound => 40400,
            AppError::StateConflict(_) => 40900,
            AppError::Business(_) => 40001,
            AppError::Payment(_) => 42201,
            AppError::Esign(_) => 42202,
            AppError::Search(_) => 42203,
            AppError::Db(_) => 50000,
            AppError::Internal(_) => 50000,
        }
    }

    /// 对应 HTTP 状态码
    pub fn http_status(&self) -> u16 {
        match self {
            AppError::Validation(_) | AppError::Business(_) => 400,
            AppError::Unauthorized => 401,
            AppError::Forbidden | AppError::SecurityRejected(_) => 403,
            AppError::NotFound => 404,
            AppError::StateConflict(_) => 409,
            AppError::Payment(_) | AppError::Esign(_) | AppError::Search(_) => 422,
            AppError::Db(_) | AppError::Internal(_) => 500,
        }
    }
}

// 便捷构造函数
impl AppError {
    pub fn validation(msg: impl Into<String>) -> Self {
        AppError::Validation(msg.into())
    }
    pub fn business(msg: impl Into<String>) -> Self {
        AppError::Business(msg.into())
    }
    pub fn state_conflict(msg: impl Into<String>) -> Self {
        AppError::StateConflict(msg.into())
    }
    pub fn db(msg: impl Into<String>) -> Self {
        AppError::Db(msg.into())
    }
    pub fn internal(e: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Self {
        AppError::Internal(e.into())
    }
}

// 常用 From 转换：方便 `?` 传播底层错误。
impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Business(format!("JSON 解析失败: {e}"))
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Internal(Box::new(e))
    }
}

/// 常用 Result 别名
pub type Result<T> = std::result::Result<T, AppError>;
