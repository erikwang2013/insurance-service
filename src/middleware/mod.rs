//! 中间件 / 过滤器链（对齐 backend-architecture.md §5）
//!
//! 装配顺序：
//! ```text
//! 请求 ─► [1] SecurityFilter ─► [2] Trace ─► [3] JWT认证(RBAC) ─► Controller
//! ```
//!
//! 说明：规划文档使用 `bee_router::filter::Filter` trait。bee-rust 当前无法在编译环境
//! 拉取，故此处定义轻量 `Filter`/`RequestCtx` 抽象承载安全/追踪/鉴权逻辑；待 bee-rust
//! 可拉取后，通过 `bee_router` 适配器把本地 `Filter` 接入 bee 过滤器链即可复用全部逻辑。

pub mod auth;
pub mod security;
pub mod trace;

/// 请求上下文的最小抽象（bee_router Context 的骨架对应）。
///
/// 承载 trace_id、当前认证用户（JWT claims）、路径/查询串/body（供安全扫描）。
#[derive(Debug, Clone, Default)]
pub struct RequestCtx {
    pub trace_id: String,
    pub current_user: Option<auth::AuthUser>,
    pub path: String,
    pub query_string: String,
    pub body_text: String,
    /// 原始 `Authorization: Bearer <jwt>` 头（骨架：由适配层注入）
    pub auth_header: Option<String>,
}

impl RequestCtx {
    /// 从 Authorization 头提取裸 token
    pub fn auth_token(&self) -> Option<&str> {
        self.auth_header.as_deref()
    }
}

/// 过滤器 trait（bee_router Filter 的骨架对应）。
///
/// `before` 返回 `Result<(), String>`，`Err(msg)` 即中断请求（Abort）。
pub trait Filter: Send + Sync {
    fn name(&self) -> &'static str;
    fn before(&self, ctx: &mut RequestCtx) -> Result<(), String>;
}
