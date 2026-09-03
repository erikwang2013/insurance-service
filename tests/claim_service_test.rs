//! 理赔服务集成测试（MySQL 集成）
//!
//! 覆盖：报案成功（校验单号/状态/归属）→ 金额 ≤0 → 保单不存在 → 保单归属他人 →
//! 我的理赔分页（含 created_at 降序）。
//!
//! 依赖 `insurance_service` 库 + 本地 MySQL（install.sql 建库）。
//! MySQL 不可用时 SKIP（打印提示并提前返回），保证 `cargo test` 在无库环境不失败。
//!
//! claims 有外键链：claims → policies/orders → quotes → products/users，
//! 故用例先按序插入一条最小保单链（用户→产品→报价→订单→保单），结束逆序清理。

mod common;

use chrono::NaiveDate;
use insurance_service::db::Db;
use insurance_service::error::AppError;
use insurance_service::services::claim_service::{ClaimService, CreateClaimReq};
use mysql_async::prelude::Queryable;
use mysql_async::Value;
use rust_decimal::Decimal;

/// 一条最小保单链（含其全部依赖行），供报案用。
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
    let user_id = insurance_service::utils::idgen::next_id();
    conn.exec_drop(
        "INSERT INTO users (id, username, password_hash) VALUES (?, ?, ?)",
        vec![Value::from(user_id), Value::from(username), Value::from("test-hash")],
    )
    .await
    .expect("插入用户");
    user_id
}

/// 按 FK 顺序插入 用户 → 产品 → 报价 → 订单 → 保单，生成一条最小保单链。
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
            insurance_amount, premium, term_months, effective_date, expire_date) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        vec![
            Value::from(policy_id),
            Value::from(&policy_no),
            Value::from(order_id),
            Value::from(quote_id),
            Value::from(user_id),
            Value::from(product_id),
            Value::from("测试产品"),
            Value::from("测试被保人"),
            Value::from("100000.00"), // insurance_amount
            Value::from("5000.00"),   // premium
            Value::from(12_i32),      // term_months
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

/// 逆 FK 序清理一条链（claims 需先由用例按 claim_no 单独删除）。
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

/// 删除本次测试插入的理赔单（按唯一单号）。
async fn cleanup_claim(db: &Db, claim_no: &str) {
    let _ = db
        .exec_drop("DELETE FROM claims WHERE claim_no = ?", vec![claim_no])
        .await;
}

/// 断言错误为业务错误且消息包含指定片段。
fn expect_business(err: AppError, needle: &str) {
    match err {
        AppError::Business(m) => assert!(m.contains(needle), "业务错误消息应含 {needle:?}: {m}"),
        other => panic!("预期 Business({needle:?})，得到 {other:?}"),
    }
}

fn svc(db: &Db) -> ClaimService {
    ClaimService::new(db.clone())
}

/// 构造报案请求
fn create_req(policy_id: i64, user_id: i64, amount: &str) -> CreateClaimReq {
    CreateClaimReq {
        policy_id,
        user_id,
        accident_date: Some(NaiveDate::from_ymd_opt(2026, 8, 1).unwrap()),
        accident_type: Some("TRAFFIC".to_string()),
        accident_desc: Some("测试事故描述".to_string()),
        claim_amount: amount.parse().unwrap(),
    }
}

#[tokio::test]
async fn create_claim_succeeds_with_policy() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let chain = insert_chain(&db, &common::unique("claim_owner")).await;

    let claim = svc(&db)
        .create(create_req(chain.policy_id, chain.user_id, "1000.00"))
        .await
        .expect("报案成功");
    assert!(claim.claim_no.starts_with("CLM"), "单号应以 CLM 开头: {}", claim.claim_no);
    assert_eq!(claim.status, "SUBMITTED", "报案后状态应为 SUBMITTED");
    assert_eq!(claim.policy_id, chain.policy_id);
    assert_eq!(claim.order_id, chain.order_id, "应回填保单关联订单");
    assert_eq!(claim.user_id, chain.user_id);
    assert_eq!(claim.claim_amount, "1000.00".parse::<Decimal>().unwrap());
    assert_eq!(claim.accident_type.as_deref(), Some("TRAFFIC"));
    assert_eq!(
        claim.accident_date,
        Some(NaiveDate::from_ymd_opt(2026, 8, 1).unwrap())
    );

    cleanup_claim(&db, &claim.claim_no).await;
    cleanup_chain(&db, &chain).await;
}

#[tokio::test]
async fn create_claim_rejects_non_positive_amount() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    // 金额校验发生在任何 DB 访问之前，policy_id 用占位值即可。
    let err = svc(&db)
        .create(create_req(1, 1, "0"))
        .await
        .expect_err("赔付金额为 0 应失败");
    expect_business(err, "赔付金额");

    let err = svc(&db)
        .create(create_req(1, 1, "-1.00"))
        .await
        .expect_err("赔付金额为负应失败");
    expect_business(err, "赔付金额");
}

#[tokio::test]
async fn create_claim_rejects_missing_policy() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    // 不存在的保单（超大 id 必不命中），无需真实用户——校验先于 INSERT。
    let err = svc(&db)
        .create(create_req(i64::MAX, 1, "1000.00"))
        .await
        .expect_err("保单不存在应失败");
    expect_business(err, "保单不存在");
}

#[tokio::test]
async fn create_claim_forbids_other_users_policy() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    // 保单归属 user B；user A 越权报案应被拒。
    let chain_b = insert_chain(&db, &common::unique("owner_b")).await;
    let user_a_id = insert_user(&db, &common::unique("trespasser_a")).await;

    let err = svc(&db)
        .create(create_req(chain_b.policy_id, user_a_id, "1000.00"))
        .await
        .expect_err("他人保单报案应失败");
    match err {
        AppError::Forbidden => {}
        other => panic!("预期 Forbidden，得到 {other:?}"),
    }

    let _ = db
        .exec_drop("DELETE FROM users WHERE id = ?", vec![user_a_id])
        .await;
    cleanup_chain(&db, &chain_b).await;
}

#[tokio::test]
async fn by_user_paginates_newest_first() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let chain = insert_chain(&db, &common::unique("pager_owner")).await;

    let c1 = svc(&db)
        .create(create_req(chain.policy_id, chain.user_id, "100.00"))
        .await
        .expect("报案 1");
    // created_at 精度为秒：间隔 1.1s 再报第二单，保证降序断言确定。
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
    let c2 = svc(&db)
        .create(create_req(chain.policy_id, chain.user_id, "200.00"))
        .await
        .expect("报案 2");
    assert_ne!(c1.claim_no, c2.claim_no);

    // 第一页 size=20 应同时返回两单
    let all = svc(&db).by_user(chain.user_id, 1, 20).await.expect("分页查询成功");
    assert!(all.len() >= 2, "应返回至少 2 单，实际 {}", all.len());
    let nos: Vec<&str> = all.iter().map(|c| c.claim_no.as_str()).collect();
    assert!(nos.contains(&c1.claim_no.as_str()) && nos.contains(&c2.claim_no.as_str()));

    // size=1：返回 1 条且为最新（c2 在后，created_at 降序应排最前）
    let one = svc(&db).by_user(chain.user_id, 1, 1).await.expect("分页 size=1");
    assert_eq!(one.len(), 1);
    assert_eq!(one[0].claim_no, c2.claim_no, "最新报案应排在第一位");

    // 第二页 size=1：轮到第一单
    let page2 = svc(&db).by_user(chain.user_id, 2, 1).await.expect("第二页");
    assert_eq!(page2.len(), 1);
    assert_eq!(page2[0].claim_no, c1.claim_no);

    cleanup_claim(&db, &c1.claim_no).await;
    cleanup_claim(&db, &c2.claim_no).await;
    cleanup_chain(&db, &chain).await;
}
