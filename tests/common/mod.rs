//! 审计回归测试共享夹具：配置/状态装配、router 调用、DB fixture、令牌签发。
//!
//! 各集成测试 crate 通过 `mod common;` 复用本模块（Cargo 不会把 tests/ 子目录
//! 当作独立测试目标）。不同测试二进制用到的 helper 不同，故允许未使用项。

#![allow(dead_code)]

use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use insurance_service::config::{
    AppConfig, CryptoConfig, DbConfig, JwtConfig, LogConfig, PaymentConfig, ServerConfig,
    WechatConfig,
};
use insurance_service::controllers::AppState;
use insurance_service::db::Db;
use insurance_service::middleware::auth::Role;
use insurance_service::utils::idgen::next_id;
use mysql_async::Value;
use rust_decimal::Decimal;
use serde_json::{Value as Json, json};
use tower::ServiceExt;

pub const CALLBACK_TOKEN: &str = "test-callback-token";

/// 测试数据库地址（未设置 → DB 用例跳过）
pub fn db_url() -> Option<String> {
    std::env::var("TEST_DATABASE_URL")
        .ok()
        .filter(|url| !url.is_empty())
}

/// 固定测试配置：JWT 密钥 >= 32 字节，主密钥为 32 个零字节的 base64。
fn test_config(db_url: &str) -> AppConfig {
    AppConfig {
        server: ServerConfig {
            host: "127.0.0.1".into(),
            port: 0,
        },
        database: DbConfig {
            url: db_url.to_string(),
        },
        jwt: JwtConfig {
            secret: "0123456789abcdef0123456789abcdef".into(),
            issuer: "insurance-test".into(),
            access_expiry: 3600,
            refresh_expiry: 7200,
        },
        crypto: CryptoConfig {
            master_key: "base64:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".into(),
        },
        log: LogConfig {
            level: "error".into(),
        },
        wechat: WechatConfig {
            app_id: String::new(),
            app_secret: String::new(),
        },
        payment: PaymentConfig {
            callback_token: CALLBACK_TOKEN.into(),
        },
    }
}

pub fn build_state(db_url: &str) -> AppState {
    AppState::new(&test_config(db_url)).expect("测试状态装配失败")
}

/// 每请求使用独立 clone 的 router（同 state 共享限流器与连接池）。
async fn send(app: &axum::Router, req: Request<Body>) -> (StatusCode, Json) {
    let resp = app.clone().oneshot(req).await.expect("router 调用失败");
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .expect("读取响应体失败");
    (status, serde_json::from_slice(&bytes).unwrap_or(Json::Null))
}

pub async fn call(
    app: &axum::Router,
    method: Method,
    uri: &str,
    token: Option<&str>,
    body: Option<Json>,
) -> (StatusCode, Json) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    let req = match body {
        Some(v) => builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(v.to_string()))
            .unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    };
    send(app, req).await
}

/// 支付回调：`token` 为 None 时不带 `X-Callback-Token` 头。
pub async fn callback(app: &axum::Router, token: Option<&str>, body: Json) -> (StatusCode, Json) {
    let mut builder = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/payments/callback/mock")
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(token) = token {
        builder = builder.header("x-callback-token", token);
    }
    send(app, builder.body(Body::from(body.to_string())).unwrap()).await
}

/// Decimal 以字符串序列化（rust_decimal serde 默认形态）
pub fn dec(v: &Json) -> Decimal {
    v.as_str()
        .expect("Decimal 应为字符串")
        .parse()
        .expect("Decimal 解析失败")
}

// ---------- DB fixtures ----------

pub async fn insert_user(db: &Db, role: &str) -> i64 {
    let id = next_id();
    db.exec_drop(
        "INSERT INTO users (id, username, password_hash, role, status, token_version) \
         VALUES (?, ?, 'placeholder', ?, 'ACTIVE', 0)",
        vec![
            Value::from(id),
            Value::from(format!("audit-{id}")),
            Value::from(role),
        ],
    )
    .await
    .expect("插入测试用户失败");
    id
}

/// 插入产品（`status` 由调用方指定；`name` 为 None 时用唯一默认名），
/// 返回 (id, 唯一产品编码, 名称)。
pub async fn insert_product(db: &Db, status: &str, name: Option<&str>) -> (i64, String, String) {
    let id = next_id();
    let code = format!("AUDIT{id}");
    let name = name
        .map(str::to_owned)
        .unwrap_or_else(|| format!("保险回归{id}"));
    db.exec_drop(
        "INSERT INTO insurance_products \
         (id, product_code, name, product_type, status, search_enabled, \
          min_amount, max_amount, min_term_months, max_term_months) \
         VALUES (?, ?, ?, 'HEALTH', ?, 1, 1.00, 1000000.00, 1, 360)",
        vec![
            Value::from(id),
            Value::from(&code),
            Value::from(&name),
            Value::from(status),
        ],
    )
    .await
    .expect("插入测试产品失败");
    (id, code, name)
}

pub async fn insert_rate(db: &Db, product_id: i64, term_months: i32, rate: &str) {
    db.exec_drop(
        "INSERT INTO quote_rates (id, product_id, term_months, amount_min, amount_max, rate) \
         VALUES (?, ?, ?, 0, NULL, ?)",
        vec![
            Value::from(next_id()),
            Value::from(product_id),
            Value::from(term_months),
            Value::from(rate),
        ],
    )
    .await
    .expect("插入费率失败");
}

pub async fn row_status(db: &Db, table: &str, id: i64) -> String {
    // table 由测试内硬编码传入，不构成注入面
    db.query_one::<String>(
        &format!("SELECT status FROM {table} WHERE id = ? LIMIT 1"),
        vec![Value::from(id)],
    )
    .await
    .expect("查询状态失败")
    .expect("状态行不存在")
}

/// 签发 access token（token_version 0，与 fixture 用户一致）
pub fn access_token(state: &AppState, user_id: i64, role: Role) -> String {
    state
        .jwt
        .issue_access_token(user_id, role, None)
        .expect("签发测试令牌失败")
}

/// 报价 → 下单（附带未知 `discount_amount`）→ 预支付，返回 (quote, order, payment_id)。
pub async fn order_fixture(app: &axum::Router, token: &str, product_id: i64) -> (Json, Json, i64) {
    let (_, quote) = call(
        app,
        Method::POST,
        "/api/v1/quotes",
        Some(token),
        Some(json!({
            "product_id": product_id,
            "holder_name": "张三",
            "insured_name": "张三",
            "insurance_amount": "10000.00",
            "term_months": 12,
            "beneficiaries": [],
        })),
    )
    .await;
    assert_eq!(quote["code"], 0, "报价失败: {quote}");
    let quote_id = quote["data"]["id"].as_i64().expect("报价 id");

    let (_, order) = call(
        app,
        Method::POST,
        "/api/v1/orders",
        Some(token),
        Some(json!({ "quote_id": quote_id, "discount_amount": "9999.00" })),
    )
    .await;
    assert_eq!(order["code"], 0, "下单失败: {order}");
    let order_id = order["data"]["id"].as_i64().expect("订单 id");

    let (_, pay) = call(
        app,
        Method::POST,
        &format!("/api/v1/payments/{order_id}/prepay"),
        Some(token),
        None,
    )
    .await;
    assert_eq!(pay["code"], 0, "预支付失败: {pay}");
    let pay_id = pay["data"]["id"].as_i64().expect("支付单 id");
    (quote, order, pay_id)
}
