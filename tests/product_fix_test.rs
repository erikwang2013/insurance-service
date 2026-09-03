//! 产品模块修复集成测试（任务 #9，HTTP 层）
//!
//! 覆盖三个修复点：
//! 1. `GET /api/v1/products/{id}/clauses` — 接库返回条款；产品不存在 / 无条款 → 40400。
//! 2. `GET /api/v1/products/featured` — 只含 ON_SALE + is_featured=1，不含下架/草稿。
//! 3. 公开 `GET /api/v1/products` 缺省仅暴露 ON_SALE；显式传 status 仍按显式过滤。
//!
//! 依赖 MySQL（install.sql 建库）。MySQL 不可用时打印 SKIP 并提前返回，
//! 保证 `cargo test` 在无库环境不失败。测试数据先插后删（unique 码 + delete_product）。

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use serde_json::Value;
use tower::util::ServiceExt;

use insurance_service::config::AppConfig;
use insurance_service::controllers::AppState;
use insurance_service::db::Db;
use insurance_service::response::ResponseEnvelope;
use insurance_service::routes;
use mysql_async::prelude::Queryable;
use mysql_async::Value as SqlValue;

// 测试专用密钥（与 tests/common/mod.rs::crypto() 一致）
const TEST_CRYPTO_KEY_B64: &str = "BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwc=";

/// 装配完整 app（MySQL 不可用 → Err，调用方打印 SKIP 后 return）
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

fn get(path: &str) -> Request<Body> {
    Request::builder().method("GET").uri(path).body(Body::empty()).unwrap()
}

/// 发送请求，返回 (状态码, 响应体)
async fn send(app: &Router, req: Request<Body>) -> (StatusCode, Vec<u8>) {
    let res = app.clone().oneshot(req).await.expect("oneshot 成功");
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), 1 << 20)
        .await
        .expect("收集响应体");
    (status, bytes.to_vec())
}

fn parse_envelope(bytes: &[u8]) -> ResponseEnvelope<Value> {
    serde_json::from_slice(bytes).expect("响应体可解析为 ResponseEnvelope")
}

// ---------------------------------------------------------------------------
// 数据构造 helper（先插后删）
// ---------------------------------------------------------------------------

/// 插入商品，返回预生成 id（与 product_service_test.rs 同构）
async fn insert_product(db: &Db, code: &str, name: &str, status: &str, featured: u8) -> i64 {
    let mut conn = db.conn().await.expect("连接测试库");
    let product_id = insurance_service::utils::idgen::next_id();
    conn.exec_drop(
        "INSERT INTO insurance_products
            (id, product_code, name, subtitle, description, product_type, sale_channel,
             insurer_name, currency, min_amount, max_amount, min_term_months,
             max_term_months, waiting_period_days, is_featured, status, search_enabled,
             created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NOW(), NOW())",
        vec![
            SqlValue::from(product_id),
            SqlValue::from(code),
            SqlValue::from(name),
            SqlValue::from(format!("{name} 副标题")),
            SqlValue::from(format!("{name} 详情描述")),
            SqlValue::from("HEALTH"),
            SqlValue::from("ONLINE"),
            SqlValue::from("测试保险公司"),
            SqlValue::from("CNY"),
            SqlValue::from(100000.00_f64),
            SqlValue::from(200000.00_f64),
            SqlValue::from(12_i32),
            SqlValue::from(120_i32),
            SqlValue::from(0_i32),
            SqlValue::from(i64::from(featured)),
            SqlValue::from(status),
            SqlValue::from(1_i64),
        ],
    )
    .await
    .expect("插入商品");
    product_id
}

/// 插入一条条款，返回预生成 id
async fn insert_clause(db: &Db, product_id: i64, title: &str, sort_order: i32) -> i64 {
    let mut conn = db.conn().await.expect("连接测试库");
    let clause_id = insurance_service::utils::idgen::next_id();
    conn.exec_drop(
        "INSERT INTO insurance_product_clauses
            (id, product_id, clause_type, title, content, sort_order)
         VALUES (?, ?, ?, ?, ?, ?)",
        vec![
            SqlValue::from(clause_id),
            SqlValue::from(product_id),
            SqlValue::from("MAIN"),
            SqlValue::from(title),
            SqlValue::from(format!("条款正文 {title}")),
            SqlValue::from(sort_order),
        ],
    )
    .await
    .expect("插入条款");
    clause_id
}

/// 断言业务失败信封：code == expected_code
fn assert_biz_err(bytes: &[u8], expected_code: i64) {
    let envelope = parse_envelope(bytes);
    assert_eq!(
        envelope.code, expected_code as i32,
        "预期业务码 {expected_code}，resp={envelope:?}"
    );
}

// ---------------------------------------------------------------------------
// 测试用例
// ---------------------------------------------------------------------------

#[tokio::test]
async fn clauses_hit_returns_sorted_list() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let app = match make_app() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("SKIP: 装配 app 失败（MySQL 未就绪）: {e}");
            return;
        }
    };
    let code = common::unique("fix_cl");
    let product_id = insert_product(&db, &code, "条款命中商品", "ON_SALE", 1).await;
    insert_clause(&db, product_id, "后置条款", 20).await;
    insert_clause(&db, product_id, "前置条款", 10).await;

    let (status, bytes) = send(&app, get(&format!("/api/v1/products/{product_id}/clauses"))).await;
    assert_eq!(status, StatusCode::OK);
    let envelope = parse_envelope(&bytes);
    assert_eq!(envelope.code, 0, "条款列表业务码为 0，resp={envelope:?}");
    let data = envelope.data.expect("条款列表有 data").as_array().cloned().unwrap_or_default();
    assert_eq!(data.len(), 2, "应返回 2 条条款");
    let titles: Vec<&str> = data
        .iter()
        .map(|c| c.get("title").and_then(Value::as_str).unwrap_or(""))
        .collect();
    assert_eq!(titles, vec!["前置条款", "后置条款"], "应按 sort_order 升序返回");

    common::delete_product(&db, &code).await;
}

#[tokio::test]
async fn clauses_not_found_cases() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let app = match make_app() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("SKIP: 装配 app 失败（MySQL 未就绪）: {e}");
            return;
        }
    };
    // 产品不存在 → 40400
    let (status, bytes) = send(&app, get("/api/v1/products/987654321/clauses")).await;
    assert_eq!(status, StatusCode::OK);
    assert_biz_err(&bytes, 40400);

    // 产品存在但无任何条款 → 40400
    let code = common::unique("fix_nc");
    let product_id = insert_product(&db, &code, "无条款商品", "ON_SALE", 0).await;
    let (status, bytes) =
        send(&app, get(&format!("/api/v1/products/{product_id}/clauses"))).await;
    assert_eq!(status, StatusCode::OK);
    assert_biz_err(&bytes, 40400);

    common::delete_product(&db, &code).await;
}

#[tokio::test]
async fn featured_only_on_sale_and_featured() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let app = match make_app() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("SKIP: 装配 app 失败（MySQL 未就绪）: {e}");
            return;
        }
    };
    let in_code = common::unique("fix_f_in");
    insert_product(&db, &in_code, "在售精选", "ON_SALE", 1).await;
    let off_code = common::unique("fix_f_off");
    insert_product(&db, &off_code, "下架精选", "OFF_SHELF", 1).await;
    let draft_code = common::unique("fix_f_dr");
    insert_product(&db, &draft_code, "草稿精选", "DRAFT", 1).await;
    let plain_code = common::unique("fix_f_pl");
    insert_product(&db, &plain_code, "在售非精选", "ON_SALE", 0).await;

    let (status, bytes) = send(&app, get("/api/v1/products/featured")).await;
    assert_eq!(status, StatusCode::OK);
    let envelope = parse_envelope(&bytes);
    assert_eq!(envelope.code, 0, "精选列表业务码为 0，resp={envelope:?}");
    let data = envelope.data.expect("精选列表有 data").as_array().cloned().unwrap_or_default();
    let codes: Vec<String> = data
        .iter()
        .filter_map(|p| p.get("product_code").and_then(Value::as_str).map(String::from))
        .collect();
    assert!(codes.contains(&in_code), "在售精选应出现：{codes:?}");
    assert!(!codes.contains(&off_code), "下架商品不应出现在精选：{codes:?}");
    assert!(!codes.contains(&draft_code), "草稿不应出现在精选：{codes:?}");
    assert!(!codes.contains(&plain_code), "非精选不应出现在精选：{codes:?}");

    for c in [&in_code, &off_code, &draft_code, &plain_code] {
        common::delete_product(&db, c).await;
    }
}

#[tokio::test]
async fn public_list_defaults_to_on_sale() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let app = match make_app() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("SKIP: 装配 app 失败（MySQL 未就绪）: {e}");
            return;
        }
    };
    let on_code = common::unique("fix_l_on");
    insert_product(&db, &on_code, "公开在售", "ON_SALE", 0).await;
    let off_code = common::unique("fix_l_off");
    insert_product(&db, &off_code, "公开下架", "OFF_SHELF", 0).await;
    let draft_code = common::unique("fix_l_dr");
    insert_product(&db, &draft_code, "公开草稿", "DRAFT", 0).await;

    // 不传 status：缺省 ON_SALE，不应暴露下架/草稿
    let (status, bytes) = send(&app, get("/api/v1/products")).await;
    assert_eq!(status, StatusCode::OK);
    let envelope = parse_envelope(&bytes);
    assert_eq!(envelope.code, 0, "公开列表业务码为 0，resp={envelope:?}");
    let data = envelope.data.expect("列表有 data").as_array().cloned().unwrap_or_default();
    let codes: Vec<String> = data
        .iter()
        .filter_map(|p| p.get("product_code").and_then(Value::as_str).map(String::from))
        .collect();
    assert!(codes.contains(&on_code), "在售商品应出现在缺省列表：{codes:?}");
    assert!(!codes.contains(&off_code), "下架商品不应在缺省列表暴露：{codes:?}");
    assert!(!codes.contains(&draft_code), "草稿不应在缺省列表暴露：{codes:?}");

    // 显式 status=OFF_SHELF：仍按显式值过滤（锁定 service 层语义不被破坏）
    let (status, bytes) = send(&app, get("/api/v1/products?status=OFF_SHELF")).await;
    assert_eq!(status, StatusCode::OK);
    let envelope = parse_envelope(&bytes);
    assert_eq!(envelope.code, 0, "显式过滤业务码为 0，resp={envelope:?}");
    let data = envelope.data.expect("列表有 data").as_array().cloned().unwrap_or_default();
    let codes: Vec<String> = data
        .iter()
        .filter_map(|p| p.get("product_code").and_then(Value::as_str).map(String::from))
        .collect();
    assert!(codes.contains(&off_code), "显式 OFF_SHELF 应命中：{codes:?}");
    assert!(!codes.contains(&on_code), "显式 OFF_SHELF 不应含在售：{codes:?}");

    for c in [&on_code, &off_code, &draft_code] {
        common::delete_product(&db, c).await;
    }
}
