//! 控制器层（MVC Controller）
//!
//! bee_router 的 `RouteGroup::get/post` 只接受普通 axum handler（
//! `H: axum::handler::Handler<T, S>`），不接受 `dyn Controller`。因此在
//! 此处提供薄适配器 handler：提取 `State<AppState>` + `Path` 参数后构造
//! `Context`，交由对应 Controller 的 `handle()` 走完整 bee 管线
//! （session 恢复 → prepare → handle → finish → session 持久化）。

pub mod auth;
pub mod claim;
pub mod contract;
pub mod order;
pub mod payment;
pub mod policy;
pub mod product;
pub mod quote;
pub mod search;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::Request;
use axum::response::{IntoResponse, Response};

use bee_rust::bee_cache::{Cache, MemoryCache};
use bee_rust::bee_router::context::RouterError;
use bee_rust::bee_router::{Context, Controller};
use bee_rust::bee_session::Session;
use bee_rust::bee_template::TemplateEngine;

use crate::config::AppConfig;
use crate::crypto::CryptoService;
use crate::db::Db;
use crate::response::{ApiResponse, ResponseEnvelope};
use crate::services::auth_service::auth_service;
use crate::services::claim_service::ClaimService;
use crate::services::contract_service::ContractService;
use crate::services::order_service::OrderService;
use crate::services::payment_service::PaymentService;
use crate::services::policy_service::PolicyService;
use crate::services::quote_service::QuoteService;

pub use auth::AuthController;
pub use claim::ClaimController;
pub use contract::ContractController;
pub use order::OrderController;
pub use payment::PaymentController;
pub use policy::PolicyController;
pub use product::ProductController;
pub use quote::QuoteController;
pub use search::SearchController;

/// 会话 TTL（滚动刷新基准；后续按业务细分）
const SESSION_TTL: Duration = Duration::from_secs(3600);

/// 共享应用状态（Clone 进每个 handler，Arc 内持各 Controller）
#[derive(Clone)]
pub struct AppState {
    pub cache: Arc<dyn Cache>,
    pub templates: Arc<TemplateEngine>,
    pub ttl: Duration,
    pub auth: Arc<AuthController>,
    pub product: Arc<ProductController>,
    pub search: Arc<SearchController>,
    pub claim: Arc<ClaimController>,
    pub quote: Arc<QuoteController>,
    pub order: Arc<OrderController>,
    pub payment: Arc<PaymentController>,
    pub policy: Arc<PolicyController>,
    pub contract: Arc<ContractController>,
}

impl AppState {
    /// 依据配置构建共享状态（失败时返回可读错误串）
    pub fn new(cfg: &AppConfig) -> Result<Self, String> {
        let cache: Arc<dyn Cache> = Arc::new(MemoryCache::new());
        let template_dir: PathBuf =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/templates");
        let templates = Arc::new(
            TemplateEngine::new(&template_dir)
                .map_err(|e| format!("模板引擎初始化失败: {e}"))?,
        );
        let crypto = CryptoService::from_master_key_b64(&cfg.crypto.master_key)
            .map_err(|e| format!("加密主密钥无效: {e}"))?;
        let db = Db::new(&cfg.database).map_err(|e| format!("数据库连接池初始化失败: {e}"))?;
        let auth = Arc::new(AuthController::new(auth_service(
            cfg.jwt.clone(),
            crypto,
            db.clone(),
        )));
        let product = Arc::new(ProductController::new(db.clone()));
        let search = Arc::new(SearchController::new(db.clone()));
        let claim = Arc::new(ClaimController::new(ClaimService::new(db.clone())));
        let quote = Arc::new(QuoteController::new(QuoteService::new(db.clone())));
        let order = Arc::new(OrderController::new(OrderService::new(db.clone())));
        let payment = Arc::new(PaymentController::new(PaymentService::new(db.clone())));
        let policy = Arc::new(PolicyController::new(PolicyService::new(db.clone())));
        let contract = Arc::new(ContractController::new(ContractService::new(db)));
        Ok(Self {
            cache,
            templates,
            ttl: SESSION_TTL,
            auth,
            product,
            search,
            claim,
            quote,
            order,
            payment,
            policy,
            contract,
        })
    }

    /// 运行一条 bee 管线：恢复 session → 前置过滤 → prepare → handle →
    /// 后置过滤 → finish → 持久化。业务异常在 Controller 内以 JSON 信封
    /// 返回；此处仅兜底管线级错误（5xxxx）。
    pub async fn run(
        &self,
        controller: &dyn Controller,
        params: HashMap<String, String>,
        request: Request<Body>,
    ) -> Response {
        let mut ctx = Context::new(
            request,
            Session::new(self.cache.clone(), self.ttl),
            self.templates.clone(),
        );
        ctx.set_params(params);
        match ctx
            .dispatch(self.cache.clone(), self.ttl, &[], controller)
            .await
        {
            Ok(()) => ctx.into_response(),
            Err(e) => internal_error_response(e),
        }
    }
}

/// 管线级错误的 JSON 信封兜底（内部错误 50000）
fn internal_error_response(e: RouterError) -> Response {
    let envelope: ApiResponse = ResponseEnvelope::err(50000, format!("服务器内部错误: {e}"));
    axum::Json(envelope).into_response()
}

/// 解析 query 字符串为键值表（阶段 0：不做百分号解码，关键字接口后续完善）
pub(crate) fn query_map(query: Option<&str>) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Some(q) = query {
        for pair in q.split('&') {
            if let Some((k, v)) = pair.split_once('=') {
                map.insert(k.to_string(), v.to_string());
            }
        }
    }
    map
}

/// 读取请求体 JSON；失败时返回信封响应（40000 校验错误）
pub(crate) async fn read_json<T: serde::de::DeserializeOwned>(ctx: &mut Context) -> Result<T, Response> {
    let body = std::mem::take(ctx.request.body_mut());
    let bytes = match axum::body::to_bytes(body, 2 * 1024 * 1024).await {
        Ok(b) => b,
        Err(e) => {
            return Err(json_envelope(40000, format!("请求体读取失败: {e}")));
        }
    };
    match serde_json::from_slice(&bytes) {
        Ok(v) => Ok(v),
        Err(e) => Err(json_envelope(40000, format!("请求体 JSON 解析失败: {e}"))),
    }
}

/// 构造业务信封响应（HTTP 200 + 业务码）
pub(crate) fn json_envelope(code: i32, msg: impl Into<String>) -> Response {
    let envelope: ApiResponse = ResponseEnvelope::err(code, msg);
    axum::Json(envelope).into_response()
}

/// 业务错误 → 信封响应
pub(crate) fn error_response(e: &crate::error::AppError) -> Response {
    json_envelope(e.code(), e.to_string())
}

/// 序列化成功数据 → 信封响应（失败视为内部错误）
pub(crate) fn ok_response(data: serde_json::Value) -> Response {
    axum::Json(ResponseEnvelope::ok(data)).into_response()
}

// ---------------------------------------------------------------------------
// 适配器 handler（每个资源一个，按 path 在 Controller 内分派）
// ---------------------------------------------------------------------------

pub async fn auth_handler(
    State(state): State<AppState>,
    Path(params): Path<HashMap<String, String>>,
    request: Request<Body>,
) -> Response {
    state.run(state.auth.as_ref(), params, request).await
}

pub async fn product_handler(
    State(state): State<AppState>,
    Path(params): Path<HashMap<String, String>>,
    request: Request<Body>,
) -> Response {
    state.run(state.product.as_ref(), params, request).await
}

pub async fn search_handler(
    State(state): State<AppState>,
    Path(params): Path<HashMap<String, String>>,
    request: Request<Body>,
) -> Response {
    state.run(state.search.as_ref(), params, request).await
}

pub async fn claim_handler(
    State(state): State<AppState>,
    Path(params): Path<HashMap<String, String>>,
    request: Request<Body>,
) -> Response {
    state.run(state.claim.as_ref(), params, request).await
}

pub async fn quote_handler(
    State(state): State<AppState>,
    Path(params): Path<HashMap<String, String>>,
    request: Request<Body>,
) -> Response {
    state.run(state.quote.as_ref(), params, request).await
}

pub async fn order_handler(
    State(state): State<AppState>,
    Path(params): Path<HashMap<String, String>>,
    request: Request<Body>,
) -> Response {
    state.run(state.order.as_ref(), params, request).await
}

pub async fn payment_handler(
    State(state): State<AppState>,
    Path(params): Path<HashMap<String, String>>,
    request: Request<Body>,
) -> Response {
    state.run(state.payment.as_ref(), params, request).await
}

pub async fn policy_handler(
    State(state): State<AppState>,
    Path(params): Path<HashMap<String, String>>,
    request: Request<Body>,
) -> Response {
    state.run(state.policy.as_ref(), params, request).await
}

pub async fn contract_handler(
    State(state): State<AppState>,
    Path(params): Path<HashMap<String, String>>,
    request: Request<Body>,
) -> Response {
    state.run(state.contract.as_ref(), params, request).await
}

pub async fn healthz_handler() -> Response {
    ok_response(serde_json::json!({
        "status": "ok",
        "service": "insurance-service"
    }))
}
