//! trace_id 生成 / 透传 + 请求日志（对齐 backend-architecture.md §5.2）
//!
//! 入口生成 `trace_id`（UUID），注入请求上下文，写入响应头 `X-Trace-Id`；
//! 与 `audit_logs.trace_id`、`ResponseEnvelope.trace_id` 对齐。

use std::sync::OnceLock;

use super::{Filter, RequestCtx};

/// 当前请求 trace_id（线程/任务本地）。
///
/// 实际框架集成时由 TraceFilter 在入口写入；此处用 `tokio::task_local!` 的骨架
/// 替代 —— 阶段 0 用 `OnceLock` + 响应包 `with_trace` 兜底，保证无中间件时也能拿到值。
static CURRENT_TRACE: OnceLock<String> = OnceLock::new();

/// 取当前 trace_id；无则惰性生成一个。
pub fn current_trace_id() -> String {
    CURRENT_TRACE
        .get()
        .cloned()
        .unwrap_or_else(|| generate_trace_id())
}

/// 生成一个新的 trace_id（UUID v4）
pub fn generate_trace_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// 设置当前 trace_id（供中间件/测试使用）
pub fn set_trace_id(id: impl Into<String>) {
    let _ = CURRENT_TRACE.set(id.into());
}

/// Trace 过滤器：生成 trace_id 并注入上下文
pub struct TraceFilter;

impl Filter for TraceFilter {
    fn name(&self) -> &'static str {
        "trace"
    }

    fn before(&self, ctx: &mut RequestCtx) -> Result<(), String> {
        let trace_id = if ctx.trace_id.is_empty() {
            generate_trace_id()
        } else {
            ctx.trace_id.clone()
        };
        ctx.trace_id = trace_id;
        set_trace_id(ctx.trace_id.clone());
        Ok(())
    }
}
