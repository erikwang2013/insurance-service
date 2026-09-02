//! 统一响应 ResponseEnvelope（对齐 backend-architecture.md §6.1）
//!
//! 三端（Flutter / 小程序 / 鸿蒙）共用统一结构 `{ code, message, data, trace_id }`。

use serde::{Deserialize, Serialize};

/// 泛型统一响应信封
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseEnvelope<T> {
    /// 0 = 成功；非 0 = 业务错误码（见 `AppError::code`）
    pub code: i32,
    /// 人类可读信息
    pub message: String,
    /// 业务数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    /// 链路追踪 ID
    pub trace_id: String,
}

impl<T> ResponseEnvelope<T> {
    /// 成功响应
    pub fn ok(data: T) -> Self {
        Self {
            code: 0,
            message: "ok".into(),
            data: Some(data),
            trace_id: crate::middleware::trace::current_trace_id(),
        }
    }

    /// 失败响应（无数据）
    pub fn err(code: i32, msg: impl Into<String>) -> Self {
        Self {
            code,
            message: msg.into(),
            data: None,
            trace_id: crate::middleware::trace::current_trace_id(),
        }
    }

    /// 显式指定 trace_id（在 trace 中间件之前构造时使用）
    pub fn with_trace(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = trace_id.into();
        self
    }
}

/// 无数据响应的类型别名
pub type ApiResponse = ResponseEnvelope<serde_json::Value>;
