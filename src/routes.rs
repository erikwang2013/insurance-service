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
            path: "/api/v1/auth/wechat/bind",
            handler: "auth.bind_wechat",
            auth: Authenticated,
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
        Route {
            method: Post,
            path: "/api/v1/policies/{id}/beneficiaries",
            handler: "policy.endorse_beneficiaries",
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
        Route {
            method: Post,
            path: "/api/v1/claims/{id}/documents",
            handler: "claim.upload_document",
            auth: Authenticated,
        },
        Route {
            method: Get,
            path: "/api/v1/claims/{id}/documents",
            handler: "claim.documents",
            auth: Authenticated,
        },
        // user
        Route {
            method: Get,
            path: "/api/v1/user/me",
            handler: "user.me",
            auth: Authenticated,
        },
        Route {
            method: Post,
            path: "/api/v1/user/password",
            handler: "user.change_password",
            auth: Authenticated,
        },
        Route {
            method: Post,
            path: "/api/v1/user/phone",
            handler: "user.bind_phone",
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
        Route {
            method: Post,
            path: "/api/v1/admin/stats",
            handler: "admin.stats",
            auth: AdminOrOperator,
        },
        Route {
            method: Get,
            path: "/api/v1/admin/audit-logs",
            handler: "admin.audit_logs",
            auth: AdminOrOperator,
        },
    ]
}

/// 吉祥物（霜霜，雪豹幼崽）—— 内嵌 SVG，另存于 docs/mascot.svg，两处同源
pub const MASCOT_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 400 400" role="img" aria-label="霜霜，insurance-service 吉祥物——雪豹幼崽，尾尖是一片雪花结晶，怀里抱着统一信封"> <title>霜霜 · insurance-service 吉祥物（雪豹幼崽）</title> <defs> <linearGradient id="ice" x1="0" y1="0" x2="0" y2="1"> <stop offset="0" stop-color="#6E9BF5"/> <stop offset="1" stop-color="#3B6FD4"/> </linearGradient> <linearGradient id="fur" gradientUnits="userSpaceOnUse" x1="120" y1="96" x2="280" y2="384"> <stop offset="0" stop-color="#FFFFFF"/> <stop offset="1" stop-color="#E4EEFB"/> </linearGradient> <linearGradient id="seal" x1="0" y1="0" x2="1" y2="1"> <stop offset="0" stop-color="#FF9A44"/> <stop offset="1" stop-color="#E8721F"/> </linearGradient> <g id="sfArm" fill="#FFFFFF"> <path d="M316 190 L309 186 L316 134 L323 186 Z"/> <path d="M316 158 L303 147 L311 161 Z"/> <path d="M316 158 L329 147 L321 161 Z"/> </g> </defs> <circle cx="200" cy="200" r="188" fill="url(#ice)"/> <circle cx="200" cy="200" r="150" fill="#FFFFFF" opacity="0.06"/> <path d="M200 30 L347 115 L347 285 L200 370 L53 285 L53 115 Z" fill="#FFFFFF" opacity="0.05"/> <path d="M200 200 L347 115 L347 285 Z" fill="#FFFFFF" opacity="0.04"/> <path d="M92 88 A150 150 0 0 1 188 52" stroke="#FFFFFF" stroke-width="9" fill="none" opacity="0.16" stroke-linecap="round"/> <g fill="#FFFFFF" opacity="0.26"> <circle cx="96" cy="122" r="4.5"/> <circle cx="314" cy="106" r="3.5"/> <circle cx="288" cy="336" r="4"/> <circle cx="112" cy="300" r="3"/> </g> <g fill="#FFFFFF" opacity="0.5"> <path d="M84 236 Q86 246 96 248 Q86 250 84 260 Q82 250 72 248 Q82 246 84 236 Z"/> <path d="M116 152 Q117 158 123 159 Q117 160 116 166 Q115 160 109 159 Q115 158 116 152 Z"/> </g> <path d="M268 322 Q320 314 330 264 Q334 246 330 232" stroke="url(#fur)" stroke-width="32" fill="none" stroke-linecap="round"/> <path d="M272 326 Q314 314 324 268 Q326 254 324 244" stroke="url(#fur)" stroke-width="22" fill="none" stroke-linecap="round"/> <path d="M284 320 Q312 308 320 278" stroke="#E4EEFB" stroke-width="8" fill="none" stroke-linecap="round" opacity="0.85"/> <circle cx="316" cy="186" r="58" fill="#FFFFFF" opacity="0.12"/> <g> <use href="#sfArm"/> <use href="#sfArm" transform="rotate(60 316 186)"/> <use href="#sfArm" transform="rotate(120 316 186)"/> <g transform="rotate(180 316 186)"> <use href="#sfArm"/> <use href="#sfArm" transform="rotate(60 316 186)"/> <use href="#sfArm" transform="rotate(120 316 186)"/> </g> </g> <circle cx="316" cy="186" r="14" fill="#D8E8FC"/> <circle cx="316" cy="186" r="6.5" fill="#FFFFFF"/> <g fill="none" stroke="#A9C6EE" stroke-width="3"> <circle cx="300" cy="294" r="5"/> <circle cx="328" cy="252" r="4.5"/> </g> <ellipse cx="200" cy="306" rx="76" ry="68" fill="url(#fur)"/> <ellipse cx="200" cy="322" rx="52" ry="44" fill="#FFFFFF" opacity="0.55"/> <g fill="none" stroke="#A9C6EE" stroke-width="3.4"> <circle cx="150" cy="288" r="7"/> <circle cx="250" cy="294" r="6"/> <circle cx="132" cy="324" r="6.5"/> <circle cx="268" cy="328" r="5.5"/> <circle cx="176" cy="264" r="5"/> <circle cx="226" cy="266" r="5"/> </g> <ellipse cx="154" cy="316" rx="21" ry="40" fill="url(#fur)" transform="rotate(16 154 316)"/> <ellipse cx="246" cy="316" rx="21" ry="40" fill="url(#fur)" transform="rotate(-16 246 316)"/> <g> <rect x="154" y="312" width="92" height="56" rx="6" fill="#F7FAFF" stroke="#C9D8EE" stroke-width="2"/> <path d="M156 315 L200 345 L244 315" fill="#E9F1FE" stroke="#C9D8EE" stroke-width="2" stroke-linejoin="round"/> <rect x="166" y="352" width="30" height="4" rx="2" fill="#C9D8EE"/> <rect x="166" y="360" width="18" height="4" rx="2" fill="#DDE7F7"/> <circle cx="222" cy="356" r="13" fill="url(#seal)"/> <path d="M216 356 L220 360 L228 351" stroke="#FFFFFF" stroke-width="3" fill="none" stroke-linecap="round" stroke-linejoin="round"/> </g> <ellipse cx="160" cy="370" rx="26" ry="14" fill="url(#fur)"/> <ellipse cx="240" cy="370" rx="26" ry="14" fill="url(#fur)"/> <g fill="#C9D8EE" opacity="0.75"> <circle cx="152" cy="368" r="3"/> <circle cx="161" cy="366" r="3"/> <circle cx="170" cy="368" r="3"/> <circle cx="232" cy="368" r="3"/> <circle cx="241" cy="366" r="3"/> <circle cx="250" cy="368" r="3"/> </g> <ellipse cx="140" cy="124" rx="32" ry="28" fill="#F1F6FE"/> <ellipse cx="260" cy="124" rx="32" ry="28" fill="#F1F6FE"/> <ellipse cx="140" cy="126" rx="15" ry="12" fill="#C9DDFA"/> <ellipse cx="260" cy="126" rx="15" ry="12" fill="#C9DDFA"/> <ellipse cx="200" cy="186" rx="86" ry="80" fill="url(#fur)"/> <circle cx="120" cy="212" r="19" fill="url(#fur)"/> <circle cx="280" cy="212" r="19" fill="url(#fur)"/> <circle cx="134" cy="238" r="14" fill="url(#fur)"/> <circle cx="266" cy="238" r="14" fill="url(#fur)"/> <g stroke="#CFE3FB" stroke-width="3" stroke-linecap="round" fill="none"> <path d="M200 135 V157 M190.5 140.5 L209.5 151.5 M190.5 151.5 L209.5 140.5"/> </g> <ellipse cx="164" cy="190" rx="25" ry="21" fill="#C9DDFA" transform="rotate(-8 164 190)"/> <ellipse cx="236" cy="190" rx="25" ry="21" fill="#C9DDFA" transform="rotate(8 236 190)"/> <circle cx="164" cy="188" r="13.5" fill="#FFFFFF"/> <circle cx="236" cy="188" r="13.5" fill="#FFFFFF"/> <circle cx="167" cy="190" r="9.5" fill="#5B8DEF"/> <circle cx="233" cy="190" r="9.5" fill="#5B8DEF"/> <circle cx="168" cy="191" r="5.5" fill="#16233A"/> <circle cx="232" cy="191" r="5.5" fill="#16233A"/> <circle cx="163" cy="185" r="4" fill="#FFFFFF"/> <circle cx="172" cy="196" r="1.8" fill="#FFFFFF"/> <circle cx="237" cy="185" r="4" fill="#FFFFFF"/> <circle cx="228" cy="196" r="1.8" fill="#FFFFFF"/> <path d="M192 212 Q200 205 208 212 Q205 223 200 223 Q195 223 192 212 Z" fill="#2B3A55"/> <path d="M186 228 Q194 238 200 229 Q206 238 214 228" stroke="#2B3A55" stroke-width="3.6" fill="none" stroke-linecap="round" stroke-linejoin="round"/> <g stroke="#B7C9E6" stroke-width="3" stroke-linecap="round" opacity="0.9"> <path d="M166 222 L136 216"/> <path d="M166 230 L138 232"/> <path d="M234 222 L264 216"/> <path d="M234 230 L262 232"/> </g> <ellipse cx="136" cy="214" rx="13" ry="7" fill="#FFB8C6" opacity="0.5"/> <ellipse cx="264" cy="214" rx="13" ry="7" fill="#FFB8C6" opacity="0.5"/> </svg>"##;

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
    payment_handler, policy_handler, product_handler, quote_handler, search_handler, stats_handler,
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
                // admin/stats（运营统计，OPERATOR/ADMIN）
                .post("/admin/stats", stats_handler)
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
    /// 与 `<use href="#id">` 引用都能找到对应 `id` 定义——id 写错会让冰面底盘 /
    /// 尾尖雪花渲染成空白，编译期查不出来（不依赖 DB，可独立运行）。
    #[test]
    fn mascot_svg_is_self_contained() {
        let svg = MASCOT_SVG;
        assert!(svg.starts_with("<svg "), "应以 <svg 开头");
        assert!(svg.ends_with("</svg>"), "应以 </svg> 收尾");
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
