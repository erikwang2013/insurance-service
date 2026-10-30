//! 审计修复回归测试（任务 8）：交易闭环 / 归属隔离 / 公开可见性 / 回调鉴权 / trace。
//!
//! DB 用例由 `TEST_DATABASE_URL` 门控：未设置时用例直接返回（跳过而非失败），
//! 因此无数据库环境 `cargo test` 仍全绿。设置后需先用 `install.sql` 建表。
//! 每个用例自建独立用户/产品（snowflake 唯一值），可并行执行、不依赖执行顺序。

use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use insurance_service::config::{
    AppConfig, CryptoConfig, DbConfig, JwtConfig, LogConfig, PaymentConfig, ServerConfig,
    WechatConfig,
};
use insurance_service::controllers::AppState;
use insurance_service::db::Db;
use insurance_service::middleware::auth::Role;
use insurance_service::models::insurance_product::InsuranceProduct;
use insurance_service::models::user::User;
use insurance_service::routes::build_bee_router;
use insurance_service::utils::idgen::next_id;
use mysql_async::Value;
use rust_decimal::Decimal;
use serde_json::{Value as Json, json};
use tower::ServiceExt;

const CALLBACK_TOKEN: &str = "test-callback-token";

/// 测试数据库地址（未设置 → DB 用例跳过）
fn db_url() -> Option<String> {
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

fn build_state(db_url: &str) -> AppState {
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

async fn call(
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
async fn callback(app: &axum::Router, token: Option<&str>, body: Json) -> (StatusCode, Json) {
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
fn dec(v: &Json) -> Decimal {
    v.as_str()
        .expect("Decimal 应为字符串")
        .parse()
        .expect("Decimal 解析失败")
}

// ---------- DB fixtures ----------

async fn insert_user(db: &Db, role: &str) -> i64 {
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
async fn insert_product(db: &Db, status: &str, name: Option<&str>) -> (i64, String, String) {
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

async fn insert_rate(db: &Db, product_id: i64, term_months: i32, rate: &str) {
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

async fn row_status(db: &Db, table: &str, id: i64) -> String {
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
fn access_token(state: &AppState, user_id: i64, role: Role) -> String {
    state
        .jwt
        .issue_access_token(user_id, role, None)
        .expect("签发测试令牌失败")
}

/// 报价 → 下单（附带未知 `discount_amount`）→ 预支付，返回 (quote, order, payment_id)。
async fn order_fixture(app: &axum::Router, token: &str, product_id: i64) -> (Json, Json, i64) {
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

// ---------- a. 交易闭环 + 归属隔离 ----------

#[tokio::test]
async fn full_transaction_flow_and_ownership() {
    let Some(url) = db_url() else { return };
    let state = build_state(&url);
    let app = build_bee_router(state.clone());
    let db = &state.db;

    let user_a = insert_user(db, User::ROLE_USER).await;
    let user_b = insert_user(db, User::ROLE_USER).await;
    let operator = insert_user(db, User::ROLE_OPERATOR).await;
    let (product, _, _) = insert_product(db, InsuranceProduct::STATUS_ON_SALE, None).await;
    insert_rate(db, product, 12, "0.01").await;

    let token_a = access_token(&state, user_a, Role::User);
    let token_b = access_token(&state, user_b, Role::User);
    let token_op = access_token(&state, operator, Role::Operator);

    // 报价 + 下单 + 首次预支付
    let (quote, order, pay1) = order_fixture(&app, &token_a, product).await;
    let quote_id = quote["data"]["id"].as_i64().expect("报价 id");
    let order_id = order["data"]["id"].as_i64().expect("订单 id");

    // 报价已被订单锁定转化：再次下单必须拒绝
    assert_eq!(row_status(db, "quotes", quote_id).await, "CONVERTED");
    let (_, retry) = call(
        &app,
        Method::POST,
        "/api/v1/orders",
        Some(&token_a),
        Some(json!({ "quote_id": quote_id })),
    )
    .await;
    assert_ne!(retry["code"], 0, "已消费报价不应再次下单: {retry}");

    // 服务端定价：忽略请求体的 discount_amount，实付=应付=保费
    assert_eq!(dec(&quote["data"]["premium"]), Decimal::from(100));
    assert_eq!(dec(&order["data"]["total_amount"]), Decimal::from(100));
    assert_eq!(dec(&order["data"]["discount_amount"]), Decimal::ZERO);
    assert_eq!(dec(&order["data"]["payable_amount"]), Decimal::from(100));

    // 第二笔预支付：同一订单两张支付单
    let (_, pay2) = call(
        &app,
        Method::POST,
        &format!("/api/v1/payments/{order_id}/prepay"),
        Some(&token_a),
        None,
    )
    .await;
    assert_eq!(pay2["code"], 0, "{pay2}");
    let pay2_id = pay2["data"]["id"].as_i64().expect("支付单 id");

    // 回调 A：支付成功 → 订单 PAID
    let (_, cb1) = callback(
        &app,
        Some(CALLBACK_TOKEN),
        json!({
            "payment_id": pay1,
            "provider_tx_id": format!("TX-{pay1}"),
            "success": true,
        }),
    )
    .await;
    assert_eq!(cb1["code"], 0, "回调 A 失败: {cb1}");
    assert_eq!(cb1["data"]["status"], "SUCCESS");
    assert_eq!(row_status(db, "orders", order_id).await, "PAID");

    // 回调 B：订单已流转 → 40900，支付单 B 整笔回滚保持 CREATED
    let (_, cb2) = callback(
        &app,
        Some(CALLBACK_TOKEN),
        json!({
            "payment_id": pay2_id,
            "provider_tx_id": format!("TX-{pay2_id}"),
            "success": true,
        }),
    )
    .await;
    assert_eq!(cb2["code"], 40900, "回调 B 应被拒绝: {cb2}");
    assert_eq!(row_status(db, "payments", pay2_id).await, "CREATED");

    // 保单签发幂等：两次返回同一保单
    let issue = json!({ "order_id": order_id });
    let (_, p1) = call(
        &app,
        Method::POST,
        "/api/v1/policies",
        Some(&token_a),
        Some(issue.clone()),
    )
    .await;
    assert_eq!(p1["code"], 0, "签单失败: {p1}");
    assert_eq!(p1["data"]["status"], "ACTIVE");
    let policy_id = p1["data"]["id"].as_i64().expect("保单 id");
    let (_, p2) = call(
        &app,
        Method::POST,
        "/api/v1/policies",
        Some(&token_a),
        Some(issue),
    )
    .await;
    assert_eq!(p2["code"], 0, "{p2}");
    assert_eq!(p2["data"]["id"], p1["data"]["id"], "重复签发应返回同一保单");

    // 合同签署（保单 ACTIVE）
    let (_, contract) = call(
        &app,
        Method::POST,
        &format!("/api/v1/contracts/{policy_id}/sign"),
        Some(&token_a),
        Some(json!({ "policy_id": policy_id })),
    )
    .await;
    assert_eq!(contract["code"], 0, "签约失败: {contract}");
    let contract_id = contract["data"]["id"].as_i64().expect("合同 id");

    // 理赔：报案 → 超额拒绝 → 审核上限 → 审核成功 → 重复审核 40900
    let (_, claim) = call(
        &app,
        Method::POST,
        "/api/v1/claims",
        Some(&token_a),
        Some(json!({ "policy_id": policy_id, "claim_amount": "5000.00" })),
    )
    .await;
    assert_eq!(claim["code"], 0, "报案失败: {claim}");
    let claim_id = claim["data"]["id"].as_i64().expect("理赔 id");

    let (_, over_claim) = call(
        &app,
        Method::POST,
        "/api/v1/claims",
        Some(&token_a),
        Some(json!({ "policy_id": policy_id, "claim_amount": "20000.00" })),
    )
    .await;
    assert_eq!(over_claim["code"], 40001, "超过保额应拒绝: {over_claim}");

    let review_uri = format!("/api/v1/claims/{claim_id}/review");
    let (_, over_review) = call(
        &app,
        Method::POST,
        &review_uri,
        Some(&token_op),
        Some(json!({ "action": "APPROVE", "approved_amount": "6000.00" })),
    )
    .await;
    assert_eq!(
        over_review["code"], 40001,
        "核定金额超过申请额应拒绝: {over_review}"
    );

    let (_, reviewed) = call(
        &app,
        Method::POST,
        &review_uri,
        Some(&token_op),
        Some(json!({ "action": "APPROVE", "approved_amount": "50.00" })),
    )
    .await;
    assert_eq!(reviewed["code"], 0, "审核失败: {reviewed}");
    assert_eq!(reviewed["data"]["status"], "APPROVED");

    let (_, reviewed_again) = call(
        &app,
        Method::POST,
        &review_uri,
        Some(&token_op),
        Some(json!({ "action": "APPROVE", "approved_amount": "50.00" })),
    )
    .await;
    assert_eq!(
        reviewed_again["code"], 40900,
        "重复审核应拒绝: {reviewed_again}"
    );

    // 退保：ACTIVE → SURRENDERED
    let (_, lapse) = call(
        &app,
        Method::POST,
        &format!("/api/v1/policies/{policy_id}/lapse"),
        Some(&token_a),
        Some(json!({ "reason": "回归测试" })),
    )
    .await;
    assert_eq!(lapse["code"], 0, "退保失败: {lapse}");
    assert_eq!(lapse["data"]["status"], "SURRENDERED");

    // 归属隔离：用户 B 读取 A 的订单/保单/报价/合同一律 40400
    for uri in [
        format!("/api/v1/orders/{order_id}"),
        format!("/api/v1/policies/{policy_id}"),
        format!("/api/v1/quotes/{quote_id}"),
        format!("/api/v1/contracts/{contract_id}"),
    ] {
        let (status, body) = call(&app, Method::GET, &uri, Some(&token_b), None).await;
        assert_eq!(status, StatusCode::OK, "{uri}: {body}");
        assert_eq!(body["code"], 40400, "{uri} 不应跨用户可见: {body}");
    }
}

// ---------- b. 公开可见性 / 搜索 ----------

#[tokio::test]
async fn public_catalog_hides_drafts_and_search_pagination_is_stable() {
    let Some(url) = db_url() else { return };
    let state = build_state(&url);
    let app = build_bee_router(state.clone());
    let db = &state.db;

    let (on_sale, code, name) = insert_product(db, InsuranceProduct::STATUS_ON_SALE, None).await;
    // 非在售商品名称包含在售商品的唯一关键字：若列表/搜索漏掉状态过滤即会命中
    let draft_name = format!("{name}草稿");
    let (draft, _, _) = insert_product(db, InsuranceProduct::STATUS_DRAFT, Some(&draft_name)).await;
    let off_shelf_name = format!("{name}下架");
    let (off_shelf, _, _) = insert_product(
        db,
        InsuranceProduct::STATUS_OFF_SHELF,
        Some(&off_shelf_name),
    )
    .await;

    // 公开列表忽略 status=DRAFT，只返回在售；DRAFT / OFF_SHELF 均不可见
    let (_, list) = call(
        &app,
        Method::GET,
        "/api/v1/products?status=DRAFT&size=100",
        None,
        None,
    )
    .await;
    assert_eq!(list["code"], 0, "{list}");
    let items = list["data"].as_array().expect("列表应为数组");
    assert!(
        items.iter().all(|p| p["status"] == "ON_SALE"),
        "公开列表混入非在售: {list}"
    );
    assert!(
        !items.iter().any(|p| p["id"].as_i64() == Some(draft)),
        "DRAFT 不应出现在公开列表: {list}"
    );
    assert!(
        !items.iter().any(|p| p["id"].as_i64() == Some(off_shelf)),
        "OFF_SHELF 不应出现在公开列表: {list}"
    );

    // 非在售详情 / 条款一律按不存在处理
    for id in [draft, off_shelf] {
        let (_, detail) = call(
            &app,
            Method::GET,
            &format!("/api/v1/products/{id}"),
            None,
            None,
        )
        .await;
        assert_eq!(detail["code"], 40400, "非在售详情应 404({id}): {detail}");
        let (_, clauses) = call(
            &app,
            Method::GET,
            &format!("/api/v1/products/{id}/clauses"),
            None,
            None,
        )
        .await;
        assert_eq!(clauses["code"], 40400, "非在售条款应 404({id}): {clauses}");
    }

    // 中文唯一关键字搜索：ON_SALE 名被非在售名称包含，状态过滤后 total 精确为 1
    let query =
        serde_urlencoded::to_string([("keyword", name.as_str()), ("page", "1"), ("size", "100")])
            .expect("搜索查询串编码失败");
    let (_, search) = call(
        &app,
        Method::GET,
        &format!("/api/v1/search?{query}"),
        None,
        None,
    )
    .await;
    assert_eq!(search["code"], 0, "{search}");
    assert_eq!(
        search["data"]["total"], 1,
        "唯一中文关键字应精确命中: {search}"
    );
    let hits = search["data"]["hits"].as_array().expect("hits 应为数组");
    let on_sale_doc = on_sale.to_string();
    let draft_doc = draft.to_string();
    let off_shelf_doc = off_shelf.to_string();
    assert!(
        hits.iter()
            .any(|h| h["doc_id"].as_str() == Some(on_sale_doc.as_str())),
        "在售商品应可搜索到: {search}"
    );
    assert!(
        !hits
            .iter()
            .any(|h| h["doc_id"].as_str() == Some(draft_doc.as_str())),
        "DRAFT 不应出现在搜索结果: {search}"
    );
    assert!(
        !hits
            .iter()
            .any(|h| h["doc_id"].as_str() == Some(off_shelf_doc.as_str())),
        "OFF_SHELF 不应出现在搜索结果: {search}"
    );

    // 超出页：窗口函数无行，回退 COUNT 后 total 仍正确（唯一编码 → total 恒为 1）
    let (_, past) = call(
        &app,
        Method::GET,
        &format!("/api/v1/search?keyword={code}&page=99&size=1"),
        None,
        None,
    )
    .await;
    assert_eq!(past["code"], 0, "{past}");
    assert_eq!(
        past["data"]["total"], 1,
        "空页 total 应由 COUNT 回退补全: {past}"
    );
    assert!(past["data"]["hits"].as_array().unwrap().is_empty());
}

// ---------- c. 未配置费率拒绝报价 ----------

#[tokio::test]
async fn quote_without_rate_row_is_rejected() {
    let Some(url) = db_url() else { return };
    let state = build_state(&url);
    let app = build_bee_router(state.clone());
    let db = &state.db;

    let user = insert_user(db, User::ROLE_USER).await;
    let (product, _, _) = insert_product(db, InsuranceProduct::STATUS_ON_SALE, None).await;
    let token = access_token(&state, user, Role::User);

    let (_, body) = call(
        &app,
        Method::POST,
        "/api/v1/quotes",
        Some(&token),
        Some(json!({
            "product_id": product,
            "holder_name": "张三",
            "insured_name": "张三",
            "insurance_amount": "10000.00",
            "term_months": 12,
            "beneficiaries": [],
        })),
    )
    .await;
    assert_eq!(body["code"], 40001, "无费率应返回业务错误: {body}");
    assert!(
        body["message"]
            .as_str()
            .unwrap_or_default()
            .contains("费率"),
        "{body}"
    );
}

// ---------- d. 伪造/缺失回调令牌 ----------

#[tokio::test]
async fn callback_requires_valid_token_and_leaves_payment_untouched() {
    let Some(url) = db_url() else { return };
    let state = build_state(&url);
    let app = build_bee_router(state.clone());
    let db = &state.db;

    let user = insert_user(db, User::ROLE_USER).await;
    let (product, _, _) = insert_product(db, InsuranceProduct::STATUS_ON_SALE, None).await;
    insert_rate(db, product, 12, "0.01").await;
    let token = access_token(&state, user, Role::User);

    let (_, order, payment) = order_fixture(&app, &token, product).await;
    let order_id = order["data"]["id"].as_i64().expect("订单 id");
    let payload = json!({
        "payment_id": payment,
        "provider_tx_id": format!("TX-{payment}"),
        "success": true,
    });

    let (_, missing) = callback(&app, None, payload.clone()).await;
    assert_eq!(missing["code"], 40300, "缺失令牌应拒绝: {missing}");

    let (_, wrong) = callback(&app, Some("wrong-token"), payload).await;
    assert_eq!(wrong["code"], 40300, "错误令牌应拒绝: {wrong}");

    assert_eq!(row_status(db, "payments", payment).await, "CREATED");
    assert_eq!(row_status(db, "orders", order_id).await, "CREATED");
}

// ---------- e. trace id：404 与 429 ----------

/// 404 fallback 与限流 429 的 `X-Trace-Id` 必须与信封 `trace_id` 一致。
#[tokio::test]
async fn trace_id_matches_between_header_and_envelope() {
    // 无需数据库：Db 为惰性连接池，404/429 都不会触库
    let app = build_bee_router(build_state("mysql://u:p@127.0.0.1:3306/unused"));

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/no-such-path")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let header_trace = resp.headers()["x-trace-id"].to_str().unwrap().to_string();
    let body: Json = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(body["code"], 40400);
    assert_eq!(body["trace_id"], header_trace);

    // 独立 router：耗尽自身限流窗口后第 301 次请求 429（/search 缺 keyword 不触库）
    let limited = build_bee_router(build_state("mysql://u:p@127.0.0.1:3306/unused"));
    let mut last = None;
    for _ in 0..301 {
        last = Some(
            limited
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/api/v1/search")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap(),
        );
    }
    let resp = last.expect("至少发起一次请求");
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    let header_trace = resp.headers()["x-trace-id"].to_str().unwrap().to_string();
    let body: Json = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(body["code"], 42900);
    assert_eq!(body["trace_id"], header_trace);
}
