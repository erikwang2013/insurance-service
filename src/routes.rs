//! 路由注册总表（对齐 backend-architecture.md §4）
//!
//! 说明：规划文档使用 `bee_router::{Router, controller::Controller}` 注册路由，并配
//! bee 过滤器链。实际执行路由由本文件的 `build_bee_router()` 注册。

use serde::{Deserialize, Serialize};

/// HTTP 方法
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Method {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

/// 鉴权要求
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Auth {
    Public,
    Authenticated,
    AdminOrOperator,
}

/// 单条路由
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub method: Method,
    pub path: &'static str,
    /// 控制器动作标识（bee_router 注册时映射到 Controller 方法）
    pub handler: &'static str,
    pub auth: Auth,
}

/// 健康检查响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Healthz {
    pub status: String,
    pub service: String,
    pub version: String,
}

/// /healthz 处理器（框架无关，可独立调用）
pub fn healthz() -> serde_json::Value {
    serde_json::json!(Healthz {
        status: "ok".into(),
        service: "insurance-service".into(),
        version: env!("CARGO_PKG_VERSION").into(),
    })
}

use axum::extract::State;
use axum::response::IntoResponse;
use bee_rust::bee_router::Router;

use crate::controllers::{
    admin_handler, AppState, auth_handler, claim_handler, contract_handler, order_handler,
    payment_handler, policy_handler, product_handler, quote_handler, search_handler,
};

/// 对接 bee_router 的路由注册（bee-rust 已激活，见 Cargo.toml [workspace.dependencies] 注释）
///
/// 阶段 0→1：注册 /healthz 与业务路由（auth / products / quotes / orders / payments /
/// policies / contracts / search / claims），控制器经 `AppState` 注入各业务
/// Controller，与 API 文档对齐。
pub fn build_bee_router(state: AppState) -> axum::Router {
    let router = Router::new()
        .ns("/api/v1", |api| {
            api
                // auth
                .post("/auth/register", auth_handler)
                .post("/auth/login", auth_handler)
                .post("/auth/wechat/login", auth_handler)
                .post("/auth/wechat/bind", auth_handler)
                .post("/auth/refresh", auth_handler)
                .post("/auth/logout", auth_handler)
                .get("/user/me", auth_handler)
                .post("/user/password", auth_handler)
                .post("/user/phone", auth_handler)
                // products（公开）
                .get("/products", product_handler)
                .get("/products/{id}", product_handler)
                .get("/products/{id}/clauses", product_handler)
                .get("/products/featured", product_handler)
                // search（公开）
                .get("/search", search_handler)
                // quotes
                .post("/quotes", quote_handler)
                .get("/quotes/{id}", quote_handler)
                // orders
                .post("/orders", order_handler)
                .get("/orders", order_handler)
                .get("/orders/{id}", order_handler)
                // payments
                .post("/payments/{order_id}/prepay", payment_handler)
                .post("/payments/{order_id}/pay", payment_handler)
                .post("/payments/wechat/prepay", payment_handler)
                .post("/payments/callback/{provider}", payment_handler)
                // policies
                .get("/policies", policy_handler)
                .get("/policies/{id}", policy_handler)
                .post("/policies/{id}/beneficiaries", policy_handler)
                // contracts
                .get("/contracts/{id}", contract_handler)
                .post("/contracts/{id}/sign", contract_handler)
                .get("/contracts/{id}/sign-url", contract_handler)
                .post("/contracts/callback/{provider}", contract_handler)
                // claims（理赔）
                .post("/claims", claim_handler)
                .get("/claims", claim_handler)
                .post("/claims/{id}/review", claim_handler)
                .post("/claims/{id}/documents", claim_handler)
                .get("/claims/{id}/documents", claim_handler)
                // admin（运营后台：商品建档 / 上下架）
                .post("/admin/products", admin_handler)
                .post("/admin/products/{id}/status", admin_handler)
                // admin/audit-logs（审计查询，OPERATOR/ADMIN）
                .get("/admin/audit-logs", admin_handler)
        })
        .build();
    // 限流挂载：仅覆盖业务路由（/healthz 不受限）
    let router = router.layer(axum::middleware::from_fn_with_state(
        state.clone(),
        rate_limit_mw,
    ));
    // 健康检查 + 统一 JSON 404（无 HTML 面）
    axum::Router::<AppState>::new()
        .route("/healthz", axum::routing::get(healthz_handler))
        .merge(router)
        .fallback(not_found)
        .with_state(state)
}

/// 未匹配路径统一返回 JSON 404 信封。
async fn not_found() -> impl axum::response::IntoResponse {
    let envelope: crate::response::ApiResponse =
        crate::response::ResponseEnvelope::err(40400, "资源不存在");
    (axum::http::StatusCode::NOT_FOUND, axum::Json(envelope))
}

/// /healthz axum 处理器，返回统一 ResponseEnvelope。
async fn healthz_handler() -> axum::Json<crate::response::ResponseEnvelope<serde_json::Value>> {
    axum::Json(crate::response::ResponseEnvelope::ok(healthz()))
}

/// 限流中间件：key 取 `X-Forwarded-For`（缺失回退全局单桶），
/// 超过 AppState.rate_limiter 窗口上限 → HTTP 429 + 统一信封（业务码 42900）。
async fn rate_limit_mw(
    State(state): State<AppState>,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let key = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
        .unwrap_or_else(|| "global".to_string());
    if state.rate_limiter.allow(&key) {
        next.run(req).await
    } else {
        let envelope: crate::response::ApiResponse =
            crate::response::ResponseEnvelope::err(42900, "请求过于频繁，请稍后再试");
        let mut resp = axum::Json(envelope).into_response();
        *resp.status_mut() = axum::http::StatusCode::TOO_MANY_REQUESTS;
        resp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 健康检查回传服务名与版本号（纯函数，不依赖 DB）
    #[test]
    fn healthz_exposes_service_and_version() {
        let h = healthz();
        assert_eq!(h["status"], "ok");
        assert_eq!(h["service"], "insurance-service");
        assert_eq!(h["version"], env!("CARGO_PKG_VERSION"));
    }
}
