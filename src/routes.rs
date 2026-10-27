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

/// 吉祥物（霜霜，雪豹幼崽）—— 编译期内嵌 `docs/mascot.svg`（单一真源）
pub const MASCOT_SVG: &str = include_str!("../docs/mascot.svg");

/// 吉祥物 ASCII 版（启动日志 / 控制台）
pub const MASCOT_BANNER: &str = r#"  ._________________________________.
 /       ,         *        ,       \
 /        /\_____________/\         \
||     (   o         o   )  *      ||
||        \       ω       /        ||
||        \___________/   *        ||
 \__________________________________/"#;

/// 健康检查响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Healthz {
    pub status: String,
    pub service: String,
    pub version: String,
    #[serde(default)]
    pub mascot: String,
}

/// /healthz 处理器（框架无关，可独立调用）
pub fn healthz() -> serde_json::Value {
    serde_json::json!(Healthz {
        status: "ok".into(),
        service: "insurance-service".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        mascot: "霜霜 ❄ 保险服务平台吉祥物——雪豹幼崽 (docs/mascot.svg)".into(),
    })
}

/// /favicon.svg：内嵌吉祥物 SVG（浏览器可直接引用）
pub async fn favicon_svg() -> impl axum::response::IntoResponse {
    (
        [(axum::http::header::CONTENT_TYPE, "image/svg+xml")],
        MASCOT_SVG,
    )
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
    // 限流挂载：仅覆盖业务路由（/、/healthz、/favicon.svg 不受限）
    let router = router.layer(axum::middleware::from_fn_with_state(
        state.clone(),
        rate_limit_mw,
    ));
    // 吉祥物 favicon + 健康检查 + 落地页：根路径（浏览器 / 前端直接引用）
    // fallback 放在 merge 之后，确保未匹配路径统一走吉祥物错误页（浏览器）
    // 或 JSON 信封 404（客户端），二者由 pages::not_found 按 Accept 协商。
    axum::Router::<AppState>::new()
        .route("/", axum::routing::get(crate::pages::landing))
        .route("/healthz", axum::routing::get(healthz_handler))
        .route("/favicon.svg", axum::routing::get(favicon_svg))
        .route("/pet/{slug}", axum::routing::get(crate::pet::svg_handler))
        .merge(router)
        .fallback(crate::pages::not_found)
        .with_state(state)
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

    /// 吉祥物内嵌 SVG 自检：标签闭合、无障碍标签齐备，且每个 `url(#id)` 填充
    /// 与 `<use href="#id">` 引用都能找到对应 `id` 定义——id 写错会让毛色 / 尾尖雪花
    /// 渲染成空白，编译期查不出来（不依赖 DB，可独立运行）。
    #[test]
    fn mascot_svg_is_self_contained() {
        let svg = MASCOT_SVG;
        assert!(svg.starts_with("<svg "), "应以 <svg 开头");
        assert!(svg.trim_end().ends_with("</svg>"), "应以 </svg> 收尾");
        assert!(svg.contains("aria-label="), "应声明 aria-label 无障碍标签");
        assert!(svg.contains("霜霜"), "应含吉祥物名");

        for (at, _) in svg.match_indices("url(#") {
            let rest = &svg[at + "url(#".len()..];
            let id = &rest[..rest.find(')').expect("url(# 引用未闭合")];
            assert!(
                svg.contains(&format!("id=\"{id}\"")),
                "url(#{id}) 缺少对应 id 定义，图形会渲染为空白"
            );
        }

        // 尾尖雪花走 <use href="#sfArm">，引用写错同样只会静默少一块图形
        let pat = "href=\"#";
        for (at, _) in svg.match_indices(pat) {
            let rest = &svg[at + pat.len()..];
            let id = &rest[..rest.find('"').expect("href 引用未闭合")];
            assert!(
                svg.contains(&format!("id=\"{id}\"")),
                "href=#{id} 缺少对应 id 定义，图形会渲染为空白"
            );
        }
    }

    /// 健康检查回传吉祥物与版本号（纯函数，不依赖 DB）
    #[test]
    fn healthz_exposes_mascot_and_version() {
        let h = healthz();
        assert_eq!(h["status"], "ok");
        assert_eq!(h["version"], env!("CARGO_PKG_VERSION"));
        assert!(
            h["mascot"].as_str().is_some_and(|m| !m.is_empty()),
            "/healthz 应回传非空 mascot 字段"
        );
    }
}
