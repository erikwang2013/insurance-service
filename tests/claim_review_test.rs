//! 理赔审核集成测试（任务 #6，MySQL 集成）
//!
//! 覆盖：非运营/管理员审核 → Forbidden；action/金额非法 → 业务错误；
//! 单证不存在或已审核 → 业务错误；APPROVE（金额/审核人/备注落库+回读）；
//! REJECT（核定金额置空）→ 再审核被拒。
//!
//! 依赖 `insurance_service` 库 + 本地 MySQL（install.sql 建库）。
//! MySQL 不可用时 SKIP（打印提示并提前返回），保证 `cargo test` 在无库环境不失败。

mod common;

use chrono::NaiveDate;
use insurance_service::db::Db;
use insurance_service::error::AppError;
use insurance_service::services::claim_service::{ClaimService, CreateClaimReq, ReviewClaimReq};
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
    policy_id: i64,
}

/// 插入一个指定角色的测试用户（role: USER / OPERATOR / ADMIN），返回自增 id。
async fn insert_user(db: &Db, username: &str, role: &str) -> i64 {
    let mut conn = db.conn().await.expect("连接测试库");
    conn.exec_drop(
        "INSERT INTO users (username, password_hash, role) VALUES (?, ?, ?)",
        vec![Value::from(username), Value::from("test-hash"), Value::from(role)],
    )
    .await
    .expect("插入用户");
    conn.last_insert_id().expect("取得自增 id") as i64
}

/// 按 FK 顺序插入 用户 → 产品 → 报价 → 订单 → 保单，生成一条最小保单链。
async fn insert_chain(db: &Db, username: &str) -> Chain {
    let user_id = insert_user(db, username, "USER").await;
    let mut conn = db.conn().await.expect("连接测试库");

    let product_code = common::unique("cp");
    conn.exec_drop(
        "INSERT INTO insurance_products (product_code, name, product_type) VALUES (?, ?, ?)",
        vec![
            Value::from(&product_code),
            Value::from("测试产品"),
            Value::from("HEALTH"),
        ],
    )
    .await
    .expect("插入产品");
    let product_id = conn.last_insert_id().expect("取得自增 id") as i64;

    let quote_no = common::unique("cq");
    conn.exec_drop(
        "INSERT INTO quotes (quote_no, product_id, user_id, insurance_amount, term_months, premium, expires_at) \
         VALUES (?, ?, ?, ?, ?, ?, DATE_ADD(NOW(), INTERVAL 7 DAY))",
        vec![
            Value::from(&quote_no),
            Value::from(product_id),
            Value::from(user_id),
            Value::from("100000.00"),
            Value::from(12_i32),
            Value::from("5000.00"),
        ],
    )
    .await
    .expect("插入报价");
    let quote_id = conn.last_insert_id().expect("取得自增 id") as i64;

    let order_no = common::unique("co");
    conn.exec_drop(
        "INSERT INTO orders \
           (order_no, quote_id, user_id, product_id, product_name, holder_name, \
            insurance_amount, term_months, total_amount, payable_amount) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        vec![
            Value::from(&order_no),
            Value::from(quote_id),
            Value::from(user_id),
            Value::from(product_id),
            Value::from("测试产品"),
            Value::from("测试被保人"),
            Value::from("100000.00"),
            Value::from(12_i32),
            Value::from("5000.00"),
            Value::from("5000.00"),
        ],
    )
    .await
    .expect("插入订单");
    let order_id = conn.last_insert_id().expect("取得自增 id") as i64;

    let policy_no = common::unique("cpn");
    conn.exec_drop(
        "INSERT INTO policies \
           (policy_no, order_id, quote_id, user_id, product_id, product_name, holder_name, \
            insurance_amount, premium, term_months, effective_date, expire_date) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        vec![
            Value::from(&policy_no),
            Value::from(order_id),
            Value::from(quote_id),
            Value::from(user_id),
            Value::from(product_id),
            Value::from("测试产品"),
            Value::from("测试被保人"),
            Value::from("100000.00"),
            Value::from("5000.00"),
            Value::from(12_i32),
            Value::from("2026-01-01"),
            Value::from("2036-01-01"),
        ],
    )
    .await
    .expect("插入保单");
    let policy_id = conn.last_insert_id().expect("取得自增 id") as i64;

    Chain {
        username: username.to_string(),
        product_code,
        quote_no,
        order_no,
        policy_no,
        user_id,
        policy_id,
    }
}

/// 逆 FK 序清理一条链（claims 需先由用例按 claim_no 单独删除）。
async fn cleanup_chain(db: &Db, c: &Chain) {
    let _ = db
        .exec_drop("DELETE FROM policies WHERE policy_no = ?", vec![c.policy_no.as_str()])
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
fn create_req(policy_id: i64, user_id: i64) -> CreateClaimReq {
    CreateClaimReq {
        policy_id,
        user_id,
        accident_date: Some(NaiveDate::from_ymd_opt(2026, 8, 1).unwrap()),
        accident_type: Some("TRAFFIC".to_string()),
        accident_desc: Some("测试事故描述".to_string()),
        claim_amount: "1000.00".parse().unwrap(),
    }
}

/// 构造审核请求
fn review_req(reviewer_id: i64, action: &str, amount: Option<&str>, remark: Option<&str>) -> ReviewClaimReq {
    ReviewClaimReq {
        reviewer_id,
        action: action.to_string(),
        approved_amount: amount.map(|a| a.parse().unwrap()),
        remark: remark.map(String::from),
    }
}

#[tokio::test]
async fn review_forbids_non_operator() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let user_id = insert_user(&db, &common::unique("rv_user"), "USER").await;

    let err = svc(&db)
        .review(1, review_req(user_id, "APPROVE", Some("500.00"), None))
        .await
        .expect_err("普通用户审核应被拒");
    match err {
        AppError::Forbidden => {}
        other => panic!("预期 Forbidden，得到 {other:?}"),
    }

    let _ = db
        .exec_drop("DELETE FROM users WHERE id = ?", vec![user_id])
        .await;
}

#[tokio::test]
async fn review_rejects_invalid_action_or_amount() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    // 动作/金额校验发生在任何 DB 访问之前，id 用占位值即可。
    expect_business(
        svc(&db)
            .review(1, review_req(1, "SOMEHOW", Some("500.00"), None))
            .await
            .expect_err("非法 action 应失败"),
        "action",
    );
    expect_business(
        svc(&db)
            .review(1, review_req(1, "APPROVE", None, None))
            .await
            .expect_err("APPROVE 缺核定金额应失败"),
        "核定赔付金额",
    );
    expect_business(
        svc(&db)
            .review(1, review_req(1, "APPROVE", Some("0"), None))
            .await
            .expect_err("APPROVE 金额为 0 应失败"),
        "核定赔付金额",
    );
}

#[tokio::test]
async fn review_rejects_missing_or_reviewed_claim() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let op_id = insert_user(&db, &common::unique("rv_op"), "OPERATOR").await;

    expect_business(
        svc(&db)
            .review(i64::MAX, review_req(op_id, "APPROVE", Some("500.00"), None))
            .await
            .expect_err("不存在的理赔单应失败"),
        "理赔单不存在或已审核",
    );

    let _ = db
        .exec_drop("DELETE FROM users WHERE id = ?", vec![op_id])
        .await;
}

#[tokio::test]
async fn review_approve_updates_and_blocks_re_review() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let chain = insert_chain(&db, &common::unique("rv_apv_owner")).await;
    let op_id = insert_user(&db, &common::unique("rv_apv_op"), "OPERATOR").await;
    let claim = svc(&db).create(create_req(chain.policy_id, chain.user_id)).await.expect("报案成功");

    let reviewed = svc(&db)
        .review(
            claim.id,
            review_req(op_id, "APPROVE", Some("800.00"), Some("核损通过")),
        )
        .await
        .expect("审核通过");
    assert_eq!(reviewed.status, "APPROVED", "通过后状态应为 APPROVED");
    assert_eq!(reviewed.approved_amount, Some("800.00".parse::<Decimal>().unwrap()));
    assert_eq!(reviewed.reviewer_id, Some(op_id));
    assert_eq!(reviewed.review_remark.as_deref(), Some("核损通过"));
    assert_eq!(reviewed.claim_no, claim.claim_no, "回读应命中同一理赔单");

    // 已审核（APPROVED）→ 再次审核被拒
    expect_business(
        svc(&db)
            .review(
                claim.id,
                review_req(op_id, "REJECT", None, Some("重复审核")),
            )
            .await
            .expect_err("已审核单再审核应失败"),
        "理赔单不存在或已审核",
    );

    cleanup_claim(&db, &claim.claim_no).await;
    let _ = db
        .exec_drop("DELETE FROM users WHERE id = ?", vec![op_id])
        .await;
    cleanup_chain(&db, &chain).await;
}

#[tokio::test]
async fn review_reject_clears_amount_and_keeps_remark_null() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let chain = insert_chain(&db, &common::unique("rv_rjt_owner")).await;
    let admin_id = insert_user(&db, &common::unique("rv_rjt_admin"), "ADMIN").await;
    let claim = svc(&db).create(create_req(chain.policy_id, chain.user_id)).await.expect("报案成功");

    // REJECT 即使带核定金额也应忽略落库
    let reviewed = svc(&db)
        .review(claim.id, review_req(admin_id, "REJECT", Some("1.00"), None))
        .await
        .expect("审核驳回");
    assert_eq!(reviewed.status, "REJECTED", "驳回后状态应为 REJECTED");
    assert_eq!(reviewed.approved_amount, None, "驳回后核定金额应置空");
    assert_eq!(reviewed.reviewer_id, Some(admin_id));
    assert_eq!(reviewed.review_remark, None, "缺省备注存 NULL");

    cleanup_claim(&db, &claim.claim_no).await;
    let _ = db
        .exec_drop("DELETE FROM users WHERE id = ?", vec![admin_id])
        .await;
    cleanup_chain(&db, &chain).await;
}
