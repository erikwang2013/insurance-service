//! 路由注册总表（对齐 backend-architecture.md §4）
//!
//! 说明：规划文档使用 `bee_router::{Router, controller::Controller}` 注册路由，并配
//! bee 过滤器链。bee-rust 当前无法在编译环境拉取，阶段 0 将路由表定义为**数据驱动的
//! 描述结构**（`RouteTable`），`/healthz` 健康检查给出框架无关的可执行实现。待
//! bee_router 可拉取后，按 `build_bee_router()` 中的注释对接 bee `Router::new()`
//! `.namespace(...)` 即可。

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

/// 路由表（阶段 0 描述结构）
pub fn route_table() -> Vec<Route> {
    use Auth::*;
    use Method::*;
    vec![
        // auth
        Route {
            method: Post,
            path: "/api/v1/auth/register",
            handler: "auth.register",
            auth: Public,
        },
        Route {
            method: Post,
            path: "/api/v1/auth/login",
            handler: "auth.login",
            auth: Public,
        },
        Route {
            method: Post,
            path: "/api/v1/auth/wechat/login",
            handler: "auth.wechat_login",
            auth: Public,
        },
        Route {
            method: Post,
            path: "/api/v1/auth/refresh",
            handler: "auth.refresh",
            auth: Public,
        },
        Route {
            method: Post,
            path: "/api/v1/auth/logout",
            handler: "auth.logout",
            auth: Authenticated,
        },
        // products（公开）
        Route {
            method: Get,
            path: "/api/v1/products",
            handler: "product.list",
            auth: Public,
        },
        Route {
            method: Get,
            path: "/api/v1/products/{id}",
            handler: "product.detail",
            auth: Public,
        },
        Route {
            method: Get,
            path: "/api/v1/products/{id}/clauses",
            handler: "product.clauses",
            auth: Public,
        },
        Route {
            method: Get,
            path: "/api/v1/products/featured",
            handler: "product.featured",
            auth: Public,
        },
        // quotes
        Route {
            method: Post,
            path: "/api/v1/quotes",
            handler: "quote.create",
            auth: Authenticated,
        },
        Route {
            method: Get,
            path: "/api/v1/quotes/{id}",
            handler: "quote.detail",
            auth: Authenticated,
        },
        // orders
        Route {
            method: Post,
            path: "/api/v1/orders",
            handler: "order.create",
            auth: Authenticated,
        },
        Route {
            method: Get,
            path: "/api/v1/orders",
            handler: "order.my_orders",
            auth: Authenticated,
        },
        Route {
            method: Get,
            path: "/api/v1/orders/{id}",
            handler: "order.detail",
            auth: Authenticated,
        },
        // payments
        Route {
            method: Post,
            path: "/api/v1/payments/{order_id}/prepay",
            handler: "payment.prepay",
            auth: Authenticated,
        },
        Route {
            method: Post,
            path: "/api/v1/payments/{order_id}/pay",
            handler: "payment.pay",
            auth: Authenticated,
        },
        Route {
            method: Post,
            path: "/api/v1/payments/wechat/prepay",
            handler: "payment.wechat_prepay",
            auth: Authenticated,
        },
        Route {
            method: Post,
            path: "/api/v1/payments/callback/{provider}",
            handler: "payment.callback",
            auth: Public,
        },
        // policies
        Route {
            method: Get,
            path: "/api/v1/policies",
            handler: "policy.my_policies",
            auth: Authenticated,
        },
        Route {
            method: Get,
            path: "/api/v1/policies/{id}",
            handler: "policy.detail",
            auth: Authenticated,
        },
        // contracts
        Route {
            method: Get,
            path: "/api/v1/contracts/{id}",
            handler: "contract.detail",
            auth: Authenticated,
        },
        Route {
            method: Post,
            path: "/api/v1/contracts/{id}/sign",
            handler: "contract.sign",
            auth: Authenticated,
        },
        Route {
            method: Get,
            path: "/api/v1/contracts/{id}/sign-url",
            handler: "contract.sign_url",
            auth: Authenticated,
        },
        Route {
            method: Post,
            path: "/api/v1/contracts/callback/{provider}",
            handler: "contract.callback",
            auth: Public,
        },
        // search（公开）
        Route {
            method: Get,
            path: "/api/v1/search",
            handler: "search.search",
            auth: Public,
        },
        // claims
        Route {
            method: Post,
            path: "/api/v1/claims",
            handler: "claim.create",
            auth: Authenticated,
        },
        Route {
            method: Get,
            path: "/api/v1/claims",
            handler: "claim.my_claims",
            auth: Authenticated,
        },
        Route {
            method: Post,
            path: "/api/v1/claims/{id}/review",
            handler: "claim.review",
            auth: Authenticated,
        },
        // user
        Route {
            method: Get,
            path: "/api/v1/user/me",
            handler: "user.me",
            auth: Authenticated,
        },
        // admin（需 ADMIN/OPERATOR）
        Route {
            method: Post,
            path: "/api/v1/admin/products",
            handler: "admin.product_upsert",
            auth: AdminOrOperator,
        },
        Route {
            method: Post,
            path: "/api/v1/admin/products/{id}/status",
            handler: "admin.product_status",
            auth: AdminOrOperator,
        },
    ]
}

/// 吉祥物（安安）—— 内嵌 SVG，另存于 docs/mascot.svg，两处同源
pub const MASCOT_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 400 400" role="img" aria-label="安安，保险服务平台吉祥物——守护熊猫"> <defs> <linearGradient id="bg" x1="0" y1="0" x2="0" y2="1"> <stop offset="0" stop-color="#5B8DEF"/> <stop offset="1" stop-color="#3B6FD4"/> </linearGradient> <linearGradient id="scarf" x1="0" y1="0" x2="1" y2="1"> <stop offset="0" stop-color="#FF8C33"/> <stop offset="1" stop-color="#E8721F"/> </linearGradient> </defs> <circle cx="200" cy="200" r="188" fill="url(#bg)"/> <circle cx="200" cy="200" r="150" fill="#FFFFFF" opacity="0.06"/> <circle cx="130" cy="118" r="36" fill="#20242E"/> <circle cx="270" cy="118" r="36" fill="#20242E"/> <circle cx="130" cy="118" r="16" fill="#FFD9E0"/> <circle cx="270" cy="118" r="16" fill="#FFD9E0"/> <ellipse cx="200" cy="330" rx="86" ry="78" fill="#FFFFFF"/> <g transform="rotate(14 122 312)"> <ellipse cx="122" cy="312" rx="26" ry="50" fill="#20242E"/> </g> <g transform="rotate(-14 278 312)"> <ellipse cx="278" cy="312" rx="26" ry="50" fill="#20242E"/> </g> <path d="M118 262 Q200 298 282 262 L288 284 Q200 316 112 284 Z" fill="url(#scarf)"/> <path d="M252 288 L258 358 Q259 366 250 366 L240 366 Q231 366 232 358 L236 288 Z" fill="#E8721F"/> <line x1="236" y1="312" x2="258" y2="312" stroke="#FFB25E" stroke-width="4"/> <line x1="236" y1="330" x2="258" y2="330" stroke="#FFB25E" stroke-width="4"/> <path d="M240 366 L236 376" stroke="#E8721F" stroke-width="5" stroke-linecap="round"/> <path d="M250 366 L254 376" stroke="#E8721F" stroke-width="5" stroke-linecap="round"/> <ellipse cx="162" cy="392" rx="34" ry="16" fill="#20242E"/> <ellipse cx="238" cy="392" rx="34" ry="16" fill="#20242E"/> <ellipse cx="200" cy="196" rx="92" ry="86" fill="#FFFFFF"/> <ellipse cx="160" cy="196" rx="24" ry="28" fill="#20242E" transform="rotate(-8 160 196)"/> <ellipse cx="240" cy="196" rx="24" ry="28" fill="#20242E" transform="rotate(8 240 196)"/> <circle cx="161" cy="193" r="13.5" fill="#FFFFFF"/> <circle cx="239" cy="193" r="13.5" fill="#FFFFFF"/> <circle cx="165" cy="196" r="8" fill="#1A1A1A"/> <circle cx="235" cy="196" r="8" fill="#1A1A1A"/> <circle cx="162" cy="193" r="3.6" fill="#FFFFFF"/> <circle cx="168.5" cy="200" r="1.7" fill="#FFFFFF"/> <circle cx="238" cy="193" r="3.6" fill="#FFFFFF"/> <circle cx="231.5" cy="200" r="1.7" fill="#FFFFFF"/> <ellipse cx="200" cy="226" rx="9" ry="6.5" fill="#20242E"/> <path d="M184 239 Q192 249 200 240 Q208 249 216 239" stroke="#20242E" stroke-width="4" fill="none" stroke-linecap="round"/> <ellipse cx="132" cy="222" rx="15" ry="9" fill="#FF9AA8" opacity="0.75"/> <ellipse cx="268" cy="222" rx="15" ry="9" fill="#FF9AA8" opacity="0.75"/></svg>"##;

/// 吉祥物 ASCII 版（启动日志 / 控制台）
pub const MASCOT_BANNER: &str = r#"  .___________________.
 / ,           ,   o   \
||    ____     __      ||
||  o/    \   /  \_    ||
||____  |____|   | \___||
||   \/   \/   ___      ||
 \_____________________/"#;

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
        mascot: "安安 🛡️ 保险服务平台吉祥物 (docs/mascot.svg)".into(),
    })
}

/// /favicon.svg：内嵌吉祥物 SVG（浏览器可直接引用）
pub async fn favicon_svg() -> impl axum::response::IntoResponse {
    (
        [(axum::http::header::CONTENT_TYPE, "image/svg+xml")],
        MASCOT_SVG,
    )
}

use bee_rust::bee_router::Router;

use crate::controllers::{
    admin_handler, AppState, auth_handler, claim_handler, contract_handler, order_handler,
    payment_handler, policy_handler, product_handler, quote_handler, search_handler,
};

/// 对接 bee_router 的路由注册（bee-rust 已激活，见 Cargo.toml [workspace.dependencies] 注释）
///
/// 阶段 0→1：注册 /healthz 与业务路由（auth / products / quotes / orders / payments /
/// policies / contracts / search / claims），控制器经 `AppState` 注入各业务
/// Controller，与 `route_table()` 对齐。
pub fn build_bee_router(state: AppState) -> axum::Router {
    let router = Router::new()
        .ns("/api/v1", |api| {
            api
                // auth
                .post("/auth/register", auth_handler)
                .post("/auth/login", auth_handler)
                .post("/auth/wechat/login", auth_handler)
                .post("/auth/refresh", auth_handler)
                .post("/auth/logout", auth_handler)
                .get("/user/me", auth_handler)
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
                // contracts
                .get("/contracts/{id}", contract_handler)
                .post("/contracts/{id}/sign", contract_handler)
                .get("/contracts/{id}/sign-url", contract_handler)
                .post("/contracts/callback/{provider}", contract_handler)
                // claims（理赔）
                .post("/claims", claim_handler)
                .get("/claims", claim_handler)
                .post("/claims/{id}/review", claim_handler)
                // admin（运营后台：商品建档 / 上下架）
                .post("/admin/products", admin_handler)
                .post("/admin/products/{id}/status", admin_handler)
        })
        .build();
    // 吉祥物 favicon + 健康检查：根路径（浏览器 / 前端直接引用）
    axum::Router::<AppState>::new()
        .route("/healthz", axum::routing::get(healthz_handler))
        .route("/favicon.svg", axum::routing::get(favicon_svg))
        .merge(router)
        .with_state(state)
}

/// /healthz axum 处理器，返回统一 ResponseEnvelope。
async fn healthz_handler() -> axum::Json<crate::response::ResponseEnvelope<serde_json::Value>> {
    axum::Json(crate::response::ResponseEnvelope::ok(healthz()))
}
