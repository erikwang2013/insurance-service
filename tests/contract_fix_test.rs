//! 合同 sign-url Mock 真实化集成测试（任务 #11，MySQL 集成）
//!
//! 覆盖：Mock 已签合同（service.sign 直签落库）sign-url 返回可用签署 URL；
//! 未走平台建流程（sign_flow_id 为空）的可签合同返回可派生 Mock URL；
//! 合同不存在 → NotFound；VOID 等终态 → 业务错误。
//!
//! 依赖 `insurance_service` 库 + 本地 MySQL（install.sql 建库）。
//! MySQL 不可用时 SKIP（打印提示并提前返回），保证 `cargo test` 在无库环境不失败。
//!
//! contracts 有外键链：contracts → policies → orders/quotes → products/users，
//! 故用例先按序插入一条最小保单链（用户→产品→报价→订单→ACTIVE 保单），
//! 结束逆序清理（删保单时 contracts 由 ON DELETE CASCADE 一并清除）。

mod common;

use insurance_service::db::Db;
use insurance_service::error::AppError;
use insurance_service::services::contract_service::{
    ContractService, CreateContractReq,
};
use mysql_async::prelude::Queryable;
use mysql_async::Value;

/// 一条最小保单链（含其全部依赖行），policy 置 ACTIVE 供合同签发。
struct Chain {
    username: String,
    product_code: String,
    quote_no: String,
    order_no: String,
    policy_no: String,
    user_id: i64,
    order_id: i64,
    policy_id: i64,
}

/// 插入一个测试用户，返回 snowflake id。
async fn insert_user(db: &Db, username: &str) -> i64 {
    let mut conn = db.conn().await.expect("连接测试库");
    let id = insurance_service::utils::idgen::next_id();
    conn.exec_drop(
        "INSERT INTO users (id, username, password_hash) VALUES (?, ?, ?)",
        vec![Value::from(id), Value::from(username), Value::from("test-hash")],
    )
    .await
    .expect("插入用户");
    id
}

/// 按 FK 顺序插入 用户 → 产品 → 报价 → 订单 → ACTIVE 保单。
async fn insert_chain(db: &Db, username: &str) -> Chain {
    let mut conn = db.conn().await.expect("连接测试库");
    let user_id = insert_user(db, username).await;

    let product_code = common::unique("cp");
    let product_id = insurance_service::utils::idgen::next_id();
    conn.exec_drop(
        "INSERT INTO insurance_products (id, product_code, name, product_type) VALUES (?, ?, ?, ?)",
        vec![
            Value::from(product_id),
            Value::from(&product_code),
            Value::from("测试产品"),
            Value::from("HEALTH"),
        ],
    )
    .await
    .expect("插入产品");

    let quote_no = common::unique("cq");
    let quote_id = insurance_service::utils::idgen::next_id();
    conn.exec_drop(
        "INSERT INTO quotes (id, quote_no, product_id, user_id, insurance_amount, term_months, premium, expires_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, DATE_ADD(NOW(), INTERVAL 7 DAY))",
        vec![
            Value::from(quote_id),
            Value::from(&quote_no),
            Value::from(product_id),
            Value::from(user_id),
            Value::from("100000.00"), // insurance_amount
            Value::from(12_i32),      // term_months
            Value::from("5000.00"),   // premium
        ],
    )
    .await
    .expect("插入报价");

    let order_no = common::unique("co");
    let order_id = insurance_service::utils::idgen::next_id();
    conn.exec_drop(
        "INSERT INTO orders \
           (id, order_no, quote_id, user_id, product_id, product_name, holder_name, \
            insurance_amount, term_months, total_amount, payable_amount) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        vec![
            Value::from(order_id),
            Value::from(&order_no),
            Value::from(quote_id),
            Value::from(user_id),
            Value::from(product_id),
            Value::from("测试产品"),
            Value::from("测试被保人"),
            Value::from("100000.00"), // insurance_amount
            Value::from(12_i32),      // term_months
            Value::from("5000.00"),   // total_amount
            Value::from("5000.00"),   // payable_amount
        ],
    )
    .await
    .expect("插入订单");

    let policy_no = common::unique("cpn");
    let policy_id = insurance_service::utils::idgen::next_id();
    conn.exec_drop(
        "INSERT INTO policies \
           (id, policy_no, order_id, quote_id, user_id, product_id, product_name, holder_name, \
            insurance_amount, premium, term_months, effective_date, expire_date, status) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'ACTIVE')",
        vec![
            Value::from(policy_id),
            Value::from(&policy_no),
            Value::from(order_id),
            Value::from(quote_id),
            Value::from(user_id),
            Value::from(product_id),
            Value::from("测试产品"),
            Value::from("测试被保人"),
            Value::from("100000.00"),  // insurance_amount
            Value::from("5000.00"),    // premium
            Value::from(12_i32),       // term_months
            Value::from("2026-01-01"), // effective_date
            Value::from("2036-01-01"), // expire_date
        ],
    )
    .await
    .expect("插入保单");

    Chain {
        username: username.to_string(),
        product_code,
        quote_no,
        order_no,
        policy_no,
        user_id,
        order_id,
        policy_id,
    }
}

/// 逆 FK 序清理一条链（contracts 由删保单 CASCADE 一并清除）。
async fn cleanup_chain(db: &Db, c: &Chain) {
    let _ = db
        .exec_drop(
            "DELETE FROM policies WHERE policy_no = ?",
            vec![c.policy_no.as_str()],
        )
        .await;
    let _ = db
        .exec_drop("DELETE FROM orders WHERE order_no = ?", vec![c.order_no.as_str()])
        .await;
    let _ = db
        .exec_drop("DELETE FROM quotes WHERE quote_no = ?", vec![c.quote_no.as_str()])
        .await;
    let _ = db
        .exec_drop(
            "DELETE FROM insurance_products WHERE product_code = ?",
            vec![c.product_code.as_str()],
        )
        .await;
    common::delete_user(db, &c.username).await;
}

/// 删除本次测试插入的合同行（按唯一合同号）。
async fn cleanup_contract(db: &Db, contract_no: &str) {
    let _ = db
        .exec_drop(
            "DELETE FROM contracts WHERE contract_no = ?",
            vec![contract_no],
        )
        .await;
}

/// 直接插一行指定状态的合同（provider 取默认 MOCK，sign_flow_id 置空）。
/// 返回 (合同 id, 合同号)。
async fn insert_contract(db: &Db, c: &Chain, status: &str) -> (i64, String) {
    let contract_no = common::unique("ct");
    let contract_id = insurance_service::utils::idgen::next_id();
    let mut conn = db.conn().await.expect("连接测试库");
    conn.exec_drop(
        "INSERT INTO contracts (id, contract_no, policy_id, order_id, title, status) \
         VALUES (?, ?, ?, ?, ?, ?)",
        vec![
            Value::from(contract_id),
            Value::from(&contract_no),
            Value::from(c.policy_id),
            Value::from(c.order_id),
            Value::from("测试合同"),
            Value::from(status),
        ],
    )
    .await
    .expect("插入合同");
    (contract_id, contract_no)
}

fn svc(db: &Db) -> ContractService {
    ContractService::new(db.clone())
}

/// 构造签发请求（直接签 → COMPLETED + sign_flow_id 落库）。
fn sign_req(c: &Chain) -> CreateContractReq {
    CreateContractReq {
        policy_id: c.policy_id,
        order_id: c.order_id,
        user_id: c.user_id,
        title: "测试合同".to_string(),
        contract_type: "POLICY".to_string(),
    }
}

#[tokio::test]
async fn signed_contract_sign_url_returns_mock_url() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let chain = insert_chain(&db, &common::unique("csign_owner")).await;

    // Mock 直签：随路径 service.sign 落库（COMPLETED + sign_flow_id + signed_at）。
    let c = svc(&db).sign(sign_req(&chain)).await.expect("Mock 直签成功");
    assert_eq!(c.status, "COMPLETED");
    assert_eq!(c.provider, "MOCK");
    let flow = c.sign_flow_id.clone().expect("直签应落 sign_flow_id");

    let url = svc(&db).sign_url(c.id).await.expect("sign-url 成功");
    assert_eq!(url.provider, "MOCK");
    assert_eq!(url.sign_url, format!("/sign/mock/{flow}"), "应命中合同平台流程");

    cleanup_chain(&db, &chain).await;
}

#[tokio::test]
async fn unsigned_contract_sign_url_uses_derived_flow() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let chain = insert_chain(&db, &common::unique("cpend_owner")).await;

    // 直接插 PENDING_SIGN 合同（无 sign_flow_id，模拟签署流程待发起）。
    let (contract_id, contract_no) = insert_contract(&db, &chain, "PENDING_SIGN").await;

    let url = svc(&db).sign_url(contract_id).await.expect("sign-url 成功");
    assert!(url.sign_url.starts_with("/sign/mock/MOCK-FLOW-"), "应走 Mock 派生流程");
    assert!(url.sign_url.contains(&contract_no), "派生 flow 应含合同号: {}", url.sign_url);

    cleanup_contract(&db, &contract_no).await;
    cleanup_chain(&db, &chain).await;
}

#[tokio::test]
async fn sign_url_missing_contract_returns_not_found() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    // 超大 id 必不命中 contracts 表。
    match svc(&db).sign_url(i64::MAX).await {
        Err(AppError::NotFound) => {}
        other => panic!("预期 NotFound，得到 {other:?}"),
    }
}

#[tokio::test]
async fn sign_url_rejects_terminated_contract() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let chain = insert_chain(&db, &common::unique("cvoid_owner")).await;
    let (contract_id, contract_no) = insert_contract(&db, &chain, "VOID").await;

    match svc(&db).sign_url(contract_id).await {
        Err(AppError::Business(m)) => assert!(m.contains("终止"), "提示应含终止: {m}"),
        other => panic!("预期 Business，得到 {other:?}"),
    }

    cleanup_contract(&db, &contract_no).await;
    cleanup_chain(&db, &chain).await;
}
