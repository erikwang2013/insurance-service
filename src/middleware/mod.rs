//! 中间件（对齐 backend-architecture.md §5）
//!
//! 请求链：`Trace → 限流 → JWT 认证（RBAC）→ Controller`。
//! Trace 最外层（401/404/限流响应同样带 trace 头）；限流在认证外层（超限请求
//! 不查库）；认证按路由访问级别校验 Bearer JWT，运营路由再校验 OPERATOR / ADMIN。

pub mod auth;
pub mod rate_limit;
pub mod trace;
