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
    ]
}

/// 吉祥物（安安）—— 内嵌 SVG，另存于 docs/mascot.svg，两处同源
pub const MASCOT_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 400 400" role="img" aria-label="安安，保险服务平台吉祥物"><circle cx="200" cy="200" r="176" fill="#EAF3FF"/><circle cx="200" cy="200" r="150" fill="#F4F9FF"/><ellipse cx="168" cy="366" rx="26" ry="14" fill="#2E6FC9"/><ellipse cx="232" cy="366" rx="26" ry="14" fill="#2E6FC9"/><path d="M160 96 Q200 28 240 96 L240 104 Q200 80 160 104 Z" fill="#FFB02E"/><rect x="150" y="92" width="100" height="14" rx="7" fill="#FFA014"/><circle cx="108" cy="225" r="15" fill="#2E6FC9"/><circle cx="292" cy="225" r="15" fill="#2E6FC9"/><path d="M200 62 C258 62 300 104 310 160 L310 242 C310 292 268 334 200 362 C132 334 90 292 90 242 L90 160 C100 104 142 62 200 62 Z" fill="#2E6FC9"/><path d="M200 62 C258 62 300 104 310 160 L310 242 C310 292 268 334 200 362 C132 334 90 292 90 242 L90 160 C100 104 142 62 200 62 Z" transform="translate(200,200) scale(0.74) translate(-200,-200)" fill="#FFFFFF"/><circle cx="166" cy="182" r="16" fill="#26293A"/><circle cx="234" cy="182" r="16" fill="#26293A"/><circle cx="171" cy="176" r="5.5" fill="#FFFFFF"/><circle cx="239" cy="176" r="5.5" fill="#FFFFFF"/><ellipse cx="140" cy="202" rx="16" ry="9" fill="#FFB3C1" opacity="0.85"/><ellipse cx="260" cy="202" rx="16" ry="9" fill="#FFB3C1" opacity="0.85"/><path d="M185 202 Q200 218 215 202" stroke="#26293A" stroke-width="5" fill="none" stroke-linecap="round"/><path d="M200 250 C200 246 178 232 178 244 C178 252 196 260 200 264 C204 260 222 252 222 244 C222 232 200 246 200 250 Z" fill="#FF6B81"/><path d="M126 130 Q118 150 116 168" stroke="#A8CCF2" stroke-width="6" fill="none" stroke-linecap="round" opacity="0.8"/></svg>"##;

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

use crate::controllers::{AppState, auth_handler, product_handler, search_handler};

/// 对接 bee_router 的路由注册（bee-rust 已激活，见 Cargo.toml [workspace.dependencies] 注释）
///
/// 阶段 0→1：注册 /healthz 与业务路由（auth / products / search），控制器经
/// `AppState` 注入各业务 Controller。剩余模块（quotes / orders / payments /
/// policies / contracts / claims / user / admin）在后续任务按同模式挂载。
pub fn build_bee_router(state: AppState) -> axum::Router {
    let router = Router::new()
        .ns("/api/v1", |api| {
            api.get("/healthz", healthz_handler)
                // auth
                .post("/auth/register", auth_handler)
                .post("/auth/login", auth_handler)
                .post("/auth/wechat/login", auth_handler)
                .post("/auth/refresh", auth_handler)
                .post("/auth/logout", auth_handler)
                // products（公开）
                .get("/products", product_handler)
                .get("/products/{id}", product_handler)
                .get("/products/{id}/clauses", product_handler)
                .get("/products/featured", product_handler)
                // search（公开）
                .get("/search", search_handler)
        })
        .build();
    // 吉祥物 favicon：根路径 /favicon.svg（浏览器 / 前端直接引用）
    axum::Router::<AppState>::new()
        .route("/favicon.svg", axum::routing::get(favicon_svg))
        .merge(router)
        .with_state(state)
}

/// /healthz axum 处理器（bee Router 的 handler 为普通 axum handler）
async fn healthz_handler() -> axum::Json<serde_json::Value> {
    axum::Json(healthz())
}
