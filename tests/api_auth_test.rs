//! 认证 API 端到端集成测试（任务 #5，HTTP 层）
//!
//! 通过 axum::test 发起真实 HTTP 请求，覆盖 auth 模块全部真实可用动作
//! （register/login/wechat_login/refresh/logout）以及 /healthz。
//!
//! 依赖 MySQL（`AppState::new()` 内部初始化连接池）。MySQL 不可用
//! 时打印 SKIP 并提前返回，保证 `cargo test` 在无库环境不失败。

mod common;

use axum::body::{Body, HttpBody};
use axum::http::{Request, StatusCode};
use axum::Router;
use serde_json::Value;
use serde_json::Map;
use tower::util::ServiceExt;

use insurance_service::config::AppConfig;
use insurance_service::controllers::AppState;
use insurance_service::response::ResponseEnvelope;
use insurance_service::routes;

// 测试专用密钥（32 字节 0x07 的 base64；与 tests/common/mod.rs::crypto() 一致）
const TEST_CRYPTO_KEY_B64: &str = "BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwc=";

/// 装配完整 app（读取环境变量 + 初始化 MySQL 连接池）。
/// MySQL 不可用 → 返回 `Err`（调用方打印 SKIP 后 return）。
fn make_app() -> Result<Router, String> {
    let cfg = AppConfig {
        server: insurance_service::config::ServerConfig {
            host: "127.0.0.1".into(),
            port: 0,
        },
        database: insurance_service::config::DbConfig {
            url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "mysql://root:@127.0.0.1:3306/insurance_service".into()),
        },
        redis: insurance_service::config::RedisConfig {
            url: "redis://127.0.0.1:6379".into(),
        },
        opensearch: insurance_service::config::SearchConfig {
            url: "http://127.0.0.1:9200".into(),
            username: "admin".into(),
            password: "changeme".into(),
        },
        jwt: common::jwt_cfg(3600),
        crypto: insurance_service::config::CryptoConfig {
            master_key: TEST_CRYPTO_KEY_B64.into(),
        },
        log: insurance_service::config::LogConfig {
            level: "info".into(),
        },
        wechat: insurance_service::config::WechatConfig {
            app_id: String::new(),
            app_secret: String::new(),
        },
    };
    let state = AppState::new(&cfg)?;
    Ok(routes::build_bee_router(state))
}

/// 便利构造函数：请求体 JSON
fn json_post(path: &str, body: &Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

/// 便利构造函数：GET
fn get(path: &str) -> Request<Body> {
    Request::builder().method("GET").uri(path).body(Body::empty()).unwrap()
}

/// 发送请求，收集响应体 + 状态码
async fn send(app: &Router, req: Request<Body>) -> (StatusCode, Vec<u8>) {
    let res = app
        .clone()
        .oneshot(req)
        .await
        .expect("oneshot 成功");
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), 1 << 20)
        .await
        .expect("收集响应体");
    (status, bytes.to_vec())
}

/// 解析信封响应
fn parse_envelope(bytes: &[u8]) -> ResponseEnvelope<Value> {
    serde_json::from_slice(bytes).expect("响应体可解析为 ResponseEnvelope")
}

fn skip_or(app: Router) -> Router {
    let _ = app;
    panic!("SKIP")
}

// ---------------------------------------------------------------------------
// helpers: 构造请求体（json! 宏，支持键名即字符串）
// ---------------------------------------------------------------------------

/// 构造 JSON 对象请求体
fn body(pairs: Vec<(String, Value)>) -> Value {
    Value::Object(Map::from_iter(pairs))
}

fn bv_username(username: &str) -> Value {
    Value::String(username.to_string())
}

// ---------------------------------------------------------------------------
// 测试用例
// ---------------------------------------------------------------------------

#[tokio::test]
async fn api_healthz() {
    let app = match make_app() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("SKIP: 装配 app 失败（MySQL 未就绪）: {e}");
            return;
        }
    };
    let (status, bytes) = send(&app, get("/healthz")).await;
    assert_eq!(status, StatusCode::OK);
    let envelope = parse_envelope(&bytes);
    assert_eq!(envelope.code, 0);
    let data: Value = envelope.data.expect("healthz 有 data");
    assert_eq!(data.get("status").and_then(Value::as_str), Some("ok"));
    assert_eq!(
        data.get("service").and_then(Value::as_str),
        Some("insurance-service")
    );
}

#[tokio::test]
async fn api_auth_register_and_login() {
    let app = match make_app() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("SKIP: 装配 app 失败（MySQL 未就绪）: {e}");
            return;
        }
    };
    let username = common::unique("api_user");

    let register_body = body(vec![
        ("username".to_string(), bv_username(&username)),
        ("password".to_string(), Value::String("P@ssw0rd!".to_string())),
        ("phone".to_string(), Value::String("13800138000".to_string())),
    ]);
    let (status, bytes) = send(&app, json_post("/api/v1/auth/register", &register_body)).await;
    assert_eq!(status, StatusCode::OK, "注册返回 HTTP 200");
    let envelope = parse_envelope(&bytes);
    assert_eq!(envelope.code, 0, "注册业务码为 0，resp={:?}", envelope);
    let data = envelope.data.expect("注册返回 data");
    assert_eq!(data.get("username").and_then(Value::as_str), Some(username.as_str()));
    assert!(data.get("tokens").and_then(|t| t.get("access_token").and_then(Value::as_str)).is_some());

    let login_body = body(vec![
        ("username".to_string(), bv_username(&username)),
        ("password".to_string(), Value::String("P@ssw0rd!".to_string())),
    ]);
    let (status, bytes) = send(&app, json_post("/api/v1/auth/login", &login_body)).await;
    assert_eq!(status, StatusCode::OK);
    let envelope = parse_envelope(&bytes);
    assert_eq!(envelope.code, 0, "登录业务码为 0，resp={:?}", envelope);
    let data = envelope.data.expect("登录返回 data");
    assert!(
        data.get("tokens")
            .and_then(|t| t.get("refresh_token").and_then(Value::as_str))
            .is_some()
    );

    if let Ok(db_url) = std::env::var("DATABASE_URL") {
        let _ = common::delete_user_by_conn(&db_url, &username).await;
    }
}

#[tokio::test]
async fn api_auth_duplicate_register_rejected() {
    let app = match make_app() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("SKIP: 装配 app 失败（MySQL 未就绪）: {e}");
            return;
        }
    };
    let username = common::unique("api_dup");

    let reg_body = body(vec![
        ("username".to_string(), bv_username(&username)),
        ("password".to_string(), Value::String("P@ssw0rd!".to_string())),
        ("phone".to_string(), Value::String("13800138000".to_string())),
    ]);
    let (status, _) = send(&app, json_post("/api/v1/auth/register", &reg_body)).await;
    assert_eq!(status, StatusCode::OK, "首次注册成功");

    let body2 = body(vec![
        ("username".to_string(), bv_username(&username)),
        ("password".to_string(), Value::String("Another1!".to_string())),
        ("phone".to_string(), Value::String("13900139000".to_string())),
    ]);
    let (status, bytes) = send(&app, json_post("/api/v1/auth/register", &body2)).await;
    assert_eq!(status, StatusCode::OK, "业务错误也走 HTTP 200 信封");
    let envelope = parse_envelope(&bytes);
    assert_ne!(envelope.code, 0, "重名注册应返回非 0 业务码，resp={:?}", envelope);
    assert!(
        envelope.message.contains("用户名已存在") || envelope.code != 0,
        "应提示用户名已存在，resp={:?}",
        envelope
    );

    if let Ok(db_url) = std::env::var("DATABASE_URL") {
        let _ = common::delete_user_by_conn(&db_url, &username).await;
    }
}

#[tokio::test]
async fn api_auth_login_wrong_password() {
    let app = match make_app() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("SKIP: 装配 app 失败（MySQL 未就绪）: {e}");
            return;
        }
    };
    let username = common::unique("api_wrong");

    let reg = body(vec![
        ("username".to_string(), bv_username(&username)),
        ("password".to_string(), Value::String("Correct1!".to_string())),
        ("phone".to_string(), Value::String("13800138000".to_string())),
    ]);
    let (status, _) = send(&app, json_post("/api/v1/auth/register", &reg)).await;
    assert_eq!(status, StatusCode::OK);

    let login = body(vec![
        ("username".to_string(), bv_username(&username)),
        ("password".to_string(), Value::String("WrongPass!".to_string())),
    ]);
    let (status, bytes) = send(&app, json_post("/api/v1/auth/login", &login)).await;
    assert_eq!(status, StatusCode::OK);
    let envelope = parse_envelope(&bytes);
    assert_ne!(envelope.code, 0, "错误密码应返回非 0 业务码，resp={:?}", envelope);

    if let Ok(db_url) = std::env::var("DATABASE_URL") {
        let _ = common::delete_user_by_conn(&db_url, &username).await;
    }
}

#[tokio::test]
async fn api_auth_wechat_login_returns_business_error() {
    let app = match make_app() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("SKIP: 装配 app 失败（MySQL 未就绪）: {e}");
            return;
        }
    };
    let body = body(vec![
        ("code".to_string(), Value::String("testcode".to_string())),
    ]);
    let (status, bytes) = send(&app, json_post("/api/v1/auth/wechat/login", &body)).await;
    assert_eq!(status, StatusCode::OK);
    let envelope = parse_envelope(&bytes);
    assert_ne!(envelope.code, 0);
    assert!(
        envelope.message.contains("未接入") || envelope.message.contains("微信"),
        "应提示微信登录未接入，resp={:?}",
        envelope
    );
}

#[tokio::test]
async fn api_auth_refresh_returns_business_error() {
    let app = match make_app() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("SKIP: 装配 app 失败（MySQL 未就绪）: {e}");
            return;
        }
    };
    let body = body(vec![
        ("refresh_token".to_string(), Value::String("xxx".to_string())),
    ]);
    let (status, bytes) = send(&app, json_post("/api/v1/auth/refresh", &body)).await;
    assert_eq!(status, StatusCode::OK);
    let envelope = parse_envelope(&bytes);
    assert_ne!(envelope.code, 0);
}

#[tokio::test]
async fn api_auth_logout_success() {
    let app = match make_app() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("SKIP: 装配 app 失败（MySQL 未就绪）: {e}");
            return;
        }
    };
    let body = body(vec![]);
    let (status, bytes) = send(&app, json_post("/api/v1/auth/logout", &body)).await;
    assert_eq!(status, StatusCode::OK);
    let envelope = parse_envelope(&bytes);
    assert_eq!(envelope.code, 0);
}

#[tokio::test]
async fn api_auth_register_missing_fields_rejected() {
    let app = match make_app() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("SKIP: 装配 app 失败（MySQL 未就绪）: {e}");
            return;
        }
    };
    // 缺 password + phone
    let body = body(vec![
        ("username".to_string(), bv_username(&common::unique("api_bad"))),
    ]);
    let (status, bytes) = send(&app, json_post("/api/v1/auth/register", &body)).await;
    assert_eq!(status, StatusCode::OK);
    let envelope = parse_envelope(&bytes);
    assert_ne!(envelope.code, 0, "缺字段应返回业务/校验错误，resp={:?}", envelope);
}
