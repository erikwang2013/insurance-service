//! 交易闭环 / 报价锁定 / 回调鉴权回归测试（任务 8）。
//!
//! DB 用例由 `TEST_DATABASE_URL` 门控：未设置时用例直接返回（跳过而非失败），
//! 因此无数据库环境 `cargo test` 仍全绿。设置后需先用 `install.sql` 建表。
//! 每个用例自建独立用户/产品（snowflake 唯一值），可并行执行、不依赖执行顺序。

mod common;

use axum::http::{Method, StatusCode};
use insurance_service::middleware::auth::Role;
use insurance_service::models::insurance_product::InsuranceProduct;
use insurance_service::models::user::User;
use insurance_service::routes::build_bee_router;
use rust_decimal::Decimal;
use serde_json::json;

use common::{
    CALLBACK_TOKEN, access_token, build_state, call, callback, db_url, dec, insert_product,
    insert_rate, insert_user, order_fixture, row_status,
};

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
