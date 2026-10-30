//! trace_id 生成 / 透传 + 请求日志（对齐 backend-architecture.md §5.2）
//!
//! 入口生成或透传 `trace_id`（UUID），以 task-local 绑定请求作用域，写入响应头
//! `X-Trace-Id`；与 `audit_logs.trace_id`、`ResponseEnvelope.trace_id` 对齐。

use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;

tokio::task_local! {
    /// 当前请求 trace_id（任务本地：并发请求互不串扰）
    pub static TRACE_ID: String;
}

/// 取当前 trace_id；不在 `trace_mw` 作用域内时惰性生成一个。
pub fn current_trace_id() -> String {
    TRACE_ID
        .try_with(|id| id.clone())
        .unwrap_or_else(|_| generate_trace_id())
}

/// 生成一个新的 trace_id（UUID v4）
pub fn generate_trace_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// 是否接受客户端传入的 trace_id：1–64 个 ASCII 字母数字/`-`/`_`。
///
/// 收紧格式避免日志注入（换行、控制字符）与超长值。
fn is_valid_trace_id(id: &str) -> bool {
    (1..=64).contains(&id.len())
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

/// Trace 中间件：透传/生成 trace_id，绑定任务作用域并回写响应头。
///
/// 挂在最终路由最外层：鉴权失败与 404 响应同样带 `X-Trace-Id`。
pub async fn trace_mw(req: Request, next: Next) -> Response {
    let id = req
        .headers()
        .get("x-trace-id")
        .and_then(|v| v.to_str().ok())
        .filter(|v| is_valid_trace_id(v))
        .map(str::to_string)
        .unwrap_or_else(generate_trace_id);
    let mut resp = TRACE_ID.scope(id.clone(), next.run(req)).await;
    // id 已通过字符白名单或 UUID 生成，HeaderValue 构造不会失败
    if let Ok(value) = HeaderValue::from_str(&id) {
        resp.headers_mut().insert("x-trace-id", value);
    }
    resp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outside_scope_generates_uuid() {
        let id = current_trace_id();
        assert_eq!(id.len(), 36);
        assert!(uuid::Uuid::parse_str(&id).is_ok());
    }

    #[tokio::test]
    async fn inside_scope_returns_scoped_id() {
        let id = TRACE_ID
            .scope("trace-abc_123".to_string(), async { current_trace_id() })
            .await;
        assert_eq!(id, "trace-abc_123");
    }

    #[test]
    fn trace_id_format_validation() {
        assert!(is_valid_trace_id("a"));
        assert!(is_valid_trace_id(&"x".repeat(64)));
        assert!(is_valid_trace_id("aZ09-_"));
        assert!(!is_valid_trace_id(""));
        assert!(!is_valid_trace_id(&"x".repeat(65)));
        assert!(!is_valid_trace_id("bad id"));
        assert!(!is_valid_trace_id("bad\nid"));
        assert!(!is_valid_trace_id("中文"));
    }
}
