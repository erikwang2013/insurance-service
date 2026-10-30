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

use std::net::SocketAddr;

use axum::extract::{ConnectInfo, Request, State};
use axum::http::{Method as HttpMethod, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use bee_rust::bee_router::Router;

use crate::controllers::{
    admin_handler, AppState, auth_handler, claim_handler, contract_handler, order_handler,
    payment_handler, policy_handler, product_handler, quote_handler, search_handler,
};
use crate::middleware::auth::{AuthUser, Role};
use crate::middleware::trace::trace_mw;
use crate::models::user::User;
use crate::response::{ApiResponse, ResponseEnvelope};

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
                .post("/policies", policy_handler)
                .get("/policies", policy_handler)
                .get("/policies/{id}", policy_handler)
                .post("/policies/{id}/lapse", policy_handler)
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
                // admin（运营后台：商品列表 / 建档 / 上下架）
                .get("/admin/products", admin_handler)
                .post("/admin/products", admin_handler)
                .post("/admin/products/{id}/status", admin_handler)
                // admin/audit-logs（审计查询，OPERATOR/ADMIN）
                .get("/admin/audit-logs", admin_handler)
        })
        .build();
    // 鉴权与限流仅覆盖业务路由（/healthz 不受限）。
    // 先加 auth 再加 rate_limit：后加者在最外层，超限请求不会进入 JWT/DB 校验。
    let router = router.layer(axum::middleware::from_fn_with_state(
        state.clone(),
        auth_mw,
    ));
    let router = router.layer(axum::middleware::from_fn_with_state(
        state.clone(),
        rate_limit_mw,
    ));
    // 健康检查 + 统一 JSON 404（无 HTML 面）；trace 最外层：404/401/限流响应也带 trace 头
    axum::Router::<AppState>::new()
        .route("/healthz", axum::routing::get(healthz_handler))
        .merge(router)
        .fallback(not_found)
        .with_state(state)
        .layer(axum::middleware::from_fn(trace_mw))
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

/// 限流中间件：key 取对端 socket IP（`ConnectInfo`，缺失回退 `unknown`）。
/// 不读取 `X-Forwarded-For`（可伪造）；反代部署应在反代层限流。
/// 超过窗口上限 → HTTP 429 + 统一信封（业务码 42900）。
async fn rate_limit_mw(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let key = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    if state.rate_limiter.allow(&key) {
        next.run(req).await
    } else {
        let envelope: ApiResponse = ResponseEnvelope::err(42900, "请求过于频繁，请稍后再试");
        let mut resp = axum::Json(envelope).into_response();
        *resp.status_mut() = StatusCode::TOO_MANY_REQUESTS;
        resp
    }
}

/// 路由访问级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Access {
    /// 无需 token，且不做 users 表查询
    Public,
    /// 需登录且角色为 OPERATOR / ADMIN
    Operator,
    /// 需登录（任意角色）
    User,
}

/// 访问级别分类（纯函数）：公共路径、运营后台、其余登录用户。
fn classify(method: &HttpMethod, path: &str) -> Access {
    const PUBLIC_EXACT: &[&str] = &[
        "/api/v1/auth/register",
        "/api/v1/auth/login",
        "/api/v1/auth/wechat/login",
        "/api/v1/auth/refresh",
        "/api/v1/search",
    ];
    if PUBLIC_EXACT.contains(&path)
        || path == "/api/v1/products"
        || path.starts_with("/api/v1/products/")
        || path.starts_with("/api/v1/payments/callback/")
        || path.starts_with("/api/v1/contracts/callback/")
    {
        return Access::Public;
    }
    if path == "/api/v1/admin" || path.starts_with("/api/v1/admin/") || is_claim_review(method, path)
    {
        return Access::Operator;
    }
    if path.starts_with("/api/v1") {
        return Access::User;
    }
    // 非 /api/v1 路径不归业务路由层管（/healthz、404 等），按公开处理
    Access::Public
}

/// `POST /api/v1/claims/{id}/review`（id 为单段路径参数）
fn is_claim_review(method: &HttpMethod, path: &str) -> bool {
    *method == HttpMethod::POST
        && path
            .strip_prefix("/api/v1/claims/")
            .and_then(|rest| rest.strip_suffix("/review"))
            .is_some_and(|id| !id.is_empty() && !id.contains('/'))
}

/// 认证中间件：公共路由直接放行（不查库）；其余路由校验 access JWT，
/// 并从 users 表加载身份/状态/令牌版本，注入 `AuthUser` 供控制器读取。
async fn auth_mw(State(state): State<AppState>, mut req: Request, next: Next) -> Response {
    let access = classify(req.method(), req.uri().path());
    if access == Access::Public {
        return next.run(req).await;
    }

    let Some(token) = bearer_token(&req) else {
        return auth_error(StatusCode::UNAUTHORIZED, 40100, "未认证");
    };
    let Ok(claims) = state.jwt.verify_access(token) else {
        return auth_error(StatusCode::UNAUTHORIZED, 40100, "未认证");
    };
    // 身份以库中行为准：角色/状态/token_version 均不信 token 载荷
    let user: Option<User> = match state
        .db
        .query_one(
            "SELECT * FROM users WHERE id = ? AND deleted_at IS NULL LIMIT 1",
            vec![claims.sub],
        )
        .await
    {
        Ok(user) => user,
        // 数据库故障是服务端错误，不能伪装成 401
        Err(_) => return auth_error(StatusCode::INTERNAL_SERVER_ERROR, 50000, "服务器内部错误"),
    };
    let Some(user) = user else {
        return auth_error(StatusCode::UNAUTHORIZED, 40100, "未认证");
    };
    if user.status != User::STATUS_ACTIVE || user.token_version != claims.token_version {
        return auth_error(StatusCode::UNAUTHORIZED, 40100, "未认证");
    }
    let role = Role::parse(&user.role).unwrap_or(Role::User);
    if access == Access::Operator && !matches!(role, Role::Operator | Role::Admin) {
        return auth_error(StatusCode::FORBIDDEN, 40300, "无权限");
    }
    req.extensions_mut().insert(AuthUser {
        id: user.id,
        role,
        platform: claims.platform,
    });
    next.run(req).await
}

/// 提取 `Authorization: Bearer <token>`
fn bearer_token(req: &Request) -> Option<&str> {
    req.headers()
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .filter(|token| !token.is_empty())
}

/// 认证/授权失败：HTTP 401/403/500 + 统一信封
fn auth_error(status: StatusCode, code: i32, message: &str) -> Response {
    let envelope: ApiResponse = ResponseEnvelope::err(code, message);
    (status, axum::Json(envelope)).into_response()
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

    /// 公共路由：登录/注册/刷新、商品、搜索、支付与合同回调
    #[test]
    fn classifies_public_routes() {
        let get = HttpMethod::GET;
        let post = HttpMethod::POST;
        for path in [
            "/api/v1/auth/register",
            "/api/v1/auth/login",
            "/api/v1/auth/wechat/login",
            "/api/v1/auth/refresh",
            "/api/v1/products",
            "/api/v1/products/123",
            "/api/v1/products/123/clauses",
            "/api/v1/products/featured",
            "/api/v1/search",
            "/api/v1/payments/callback/wechat",
            "/api/v1/contracts/callback/esign",
            "/healthz",
        ] {
            assert_eq!(classify(&get, path), Access::Public, "{path} 应为公开");
        }
        assert_eq!(
            classify(&post, "/api/v1/auth/login"),
            Access::Public
        );
    }

    /// 运营路由：admin 前缀 + POST claims/{id}/review
    #[test]
    fn classifies_operator_routes() {
        let get = HttpMethod::GET;
        let post = HttpMethod::POST;
        for path in [
            "/api/v1/admin/products",
            "/api/v1/admin/products/1/status",
            "/api/v1/admin/audit-logs",
        ] {
            assert_eq!(classify(&get, path), Access::Operator, "{path} 应为运营");
        }
        assert_eq!(
            classify(&post, "/api/v1/claims/42/review"),
            Access::Operator
        );
        // 仅 POST review 需要运营；GET 同路径不是运营路由
        assert_ne!(classify(&get, "/api/v1/claims/42/review"), Access::Operator);
        assert_eq!(classify(&post, "/api/v1/claims/42/documents"), Access::User);
        assert_eq!(classify(&post, "/api/v1/claims//review"), Access::User);
        assert_eq!(classify(&post, "/api/v1/claims/a/b/review"), Access::User);
    }

    /// 其余 /api/v1 路由一律要求登录
    #[test]
    fn classifies_authenticated_routes() {
        let get = HttpMethod::GET;
        let post = HttpMethod::POST;
        for path in [
            "/api/v1/user/me",
            "/api/v1/user/password",
            "/api/v1/orders",
            "/api/v1/orders/1",
            "/api/v1/policies",
            "/api/v1/policies/1/beneficiaries",
            "/api/v1/contracts/1/sign",
            "/api/v1/claims",
            "/api/v1/payments/1/prepay",
            "/api/v1/auth/logout",
            "/api/v1/auth/wechat/bind",
        ] {
            assert_eq!(classify(&get, path), Access::User, "{path} 应为登录用户");
        }
        assert_eq!(classify(&post, "/api/v1/quotes"), Access::User);
        assert_eq!(classify(&post, "/api/v1/policies"), Access::User);
        assert_eq!(classify(&post, "/api/v1/policies/1/lapse"), Access::User);
    }

    /// 构造可离线装配的 AppState（Db 惰性建连，测试不会真正连库）
    fn test_state() -> AppState {
        use crate::config::{
            CryptoConfig, DbConfig, JwtConfig, LogConfig, PaymentConfig, ServerConfig, WechatConfig,
        };
        AppState::new(&crate::config::AppConfig {
            server: ServerConfig { host: "127.0.0.1".into(), port: 0 },
            database: DbConfig { url: "mysql://u:p@127.0.0.1:3306/test".into() },
            jwt: JwtConfig {
                secret: "0123456789abcdef0123456789abcdef".into(),
                issuer: "insurance-test".into(),
                access_expiry: 100,
                refresh_expiry: 1000,
            },
            crypto: CryptoConfig {
                master_key: "base64:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".into(),
            },
            log: LogConfig { level: "error".into() },
            wechat: WechatConfig { app_id: String::new(), app_secret: String::new() },
            payment: PaymentConfig { callback_token: String::new() },
        })
        .expect("测试状态装配失败")
    }

    /// 验收：受保护路由无 token → HTTP 401 + 统一信封，且响应带 X-Trace-Id
    #[tokio::test]
    async fn protected_route_without_token_is_401_with_trace() {
        use tower::ServiceExt;
        let app = build_bee_router(test_state());
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/orders")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        let header_trace = resp
            .headers()
            .get("x-trace-id")
            .expect("认证失败响应也应带 X-Trace-Id")
            .to_str()
            .unwrap()
            .to_string();
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["code"], 40100);
        assert_eq!(body["trace_id"], header_trace, "信封 trace_id 与响应头一致");
    }

    /// /healthz 无 token 可用且带 trace 头（不在鉴权/限流覆盖范围）
    #[tokio::test]
    async fn healthz_is_public_and_traced() {
        use tower::ServiceExt;
        let app = build_bee_router(test_state());
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(resp.headers().get("x-trace-id").is_some());
    }

    /// Bearer 头解析：缺失/空值/非 Bearer 一律视为无 token
    #[test]
    fn parses_bearer_header() {
        let with = |value: &str| {
            Request::builder()
                .header(axum::http::header::AUTHORIZATION, value)
                .body(axum::body::Body::empty())
                .unwrap()
        };
        assert_eq!(bearer_token(&with("Bearer abc")), Some("abc"));
        assert_eq!(bearer_token(&with("Bearer ")), None);
        assert_eq!(bearer_token(&with("Basic abc")), None);
        let no_header = Request::builder()
            .body(axum::body::Body::empty())
            .unwrap();
        assert_eq!(bearer_token(&no_header), None);
    }
}
