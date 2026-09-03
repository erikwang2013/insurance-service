//! C2 保单批改-受益人变更集成测试（MySQL 集成）
//!
//! 覆盖：成功整单替换（先删后插 + audit_logs 前后快照）→ 保单不存在 → 他人保单 →
//! 非 ACTIVE 保单 → 名单非法（空名单 / 指定受益人占比合计≠100）。
//!
//! 依赖 `insurance_service` 库 + 本地 MySQL（install.sql 建库）。
//! MySQL 不可用时 SKIP（打印提示并提前返回），保证 `cargo test` 在无库环境不失败。
//!
//! policies 有外键链：policies → orders → quotes → products/users，
//! 故用例先按序插入一条最小保单链（用户→产品→报价→订单→保单），结束逆序清理；
//! policy_beneficiaries 随 policies 级联删除，audit_logs 按操作人手动清理。

mod common;

use insurance_service::db::Db;
use insurance_service::error::AppError;
use insurance_service::models::policy::{BeneficiaryInput, EndorseBeneficiariesReq};
use insurance_service::services::policy_service::PolicyService;
use mysql_async::prelude::Queryable;
use mysql_async::Value;
use rust_decimal::Decimal;

/// 一条最小保单链（含其全部依赖行），供批改用。
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

/// 插入一个测试用户，返回自增 id。
async fn insert_user(db: &Db, username: &str) -> i64 {
    let mut conn = db.conn().await.expect("连接测试库");
    conn.exec_drop(
        "INSERT INTO users (username, password_hash) VALUES (?, ?)",
        vec![Value::from(username), Value::from("test-hash")],
    )
    .await
    .expect("插入用户");
    conn.last_insert_id().expect("取得自增 id") as i64
}

/// 按 FK 顺序插入 用户 → 产品 → 报价 → 订单 → 保单，生成一条最小保单链（默认 PENDING_ISSUE）。
async fn insert_chain(db: &Db, username: &str) -> Chain {
    let mut conn = db.conn().await.expect("连接测试库");
    let user_id = insert_user(db, username).await;

    let product_code = common::unique("ep");
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

    let quote_no = common::unique("eq");
    conn.exec_drop(
        "INSERT INTO quotes (quote_no, product_id, user_id, insurance_amount, term_months, premium, expires_at) \
         VALUES (?, ?, ?, ?, ?, ?, DATE_ADD(NOW(), INTERVAL 7 DAY))",
        vec![
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
    let quote_id = conn.last_insert_id().expect("取得自增 id") as i64;

    let order_no = common::unique("eo");
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
            Value::from("100000.00"), // insurance_amount
            Value::from(12_i32),      // term_months
            Value::from("5000.00"),   // total_amount
            Value::from("5000.00"),   // payable_amount
        ],
    )
    .await
    .expect("插入订单");
    let order_id = conn.last_insert_id().expect("取得自增 id") as i64;

    let policy_no = common::unique("epn");
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
            Value::from("100000.00"), // insurance_amount
            Value::from("5000.00"),   // premium
            Value::from(12_i32),      // term_months
            Value::from("2026-01-01"), // effective_date
            Value::from("2036-01-01"), // expire_date
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
        order_id,
        policy_id,
    }
}

/// 逆 FK 序清理一条链（audit_logs 需先由用例按 user 删除）。
async fn cleanup_chain(db: &Db, c: &Chain) {
    let _ = db
        .exec_drop("DELETE FROM audit_logs WHERE user_id = ?", vec![c.user_id])
        .await;
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

/// 置保单状态（成功用例转 ACTIVE，拒绝用例转 EXPIRED）。
async fn set_policy_status(db: &Db, policy_id: i64, status: &str) {
    let mut conn = db.conn().await.expect("连接测试库");
    conn.exec_drop(
        "UPDATE policies SET status = ? WHERE id = ?",
        vec![Value::from(status), Value::from(policy_id)],
    )
    .await
    .expect("更新保单状态");
}

/// 断言错误为业务错误且消息包含指定片段。
fn expect_business(err: AppError, needle: &str) {
    match err {
        AppError::Business(m) => assert!(m.contains(needle), "业务错误消息应含 {needle:?}: {m}"),
        other => panic!("预期 Business({needle:?})，得到 {other:?}"),
    }
}

fn svc(db: &Db) -> PolicyService {
    PolicyService::new(db.clone())
}

/// 构造指定受益人批改请求（操作人 user_id 与保单归属同源）。
fn endorse_req(user_id: i64, list: Vec<BeneficiaryInput>) -> EndorseBeneficiariesReq {
    EndorseBeneficiariesReq { user_id, beneficiaries: list }
}

fn named(name: &str, relationship: &str, share: &str) -> BeneficiaryInput {
    BeneficiaryInput {
        name: name.to_string(),
        relationship: Some(relationship.to_string()),
        beneficiary_type: Some("NAMED".to_string()),
        share_percent: Some(share.parse().unwrap()),
    }
}

#[tokio::test]
async fn endorse_succeeds_swaps_beneficiaries_and_audits() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let chain = insert_chain(&db, &common::unique("endorse_owner")).await;
    set_policy_status(&db, chain.policy_id, "ACTIVE").await;
    // 预置一名旧受益人（旧快照应以 JSON 数组入审计 before_json）
    let mut conn = db.conn().await.expect("连接测试库");
    conn.exec_drop(
        "INSERT INTO policy_beneficiaries (policy_id, name, beneficiary_type, sort_order) \
         VALUES (?, ?, 'LEGAL', 0)",
        vec![Value::from(chain.policy_id), Value::from("旧受益人甲")],
    )
    .await
    .expect("预置旧受益人");

    let (policy, bens) = svc(&db)
        .endorse_beneficiaries(
            chain.user_id,
            chain.policy_id,
            endorse_req(chain.user_id, vec![
                named("新受益人乙", "SPOUSE", "60.00"),
                named("新受益人丙", "CHILD", "40.00"),
            ]),
        )
        .await
        .expect("批改成功");
    assert_eq!(policy.id, chain.policy_id, "返回保单应为被批改保单");
    assert_eq!(policy.status, "ACTIVE");
    assert_eq!(bens.len(), 2, "整单替换后应返回 2 名受益人");
    assert_eq!(bens[0].name, "新受益人乙");
    assert_eq!(bens[0].beneficiary_type, "NAMED");
    assert_eq!(bens[0].relationship.as_deref(), Some("SPOUSE"));
    assert_eq!(bens[0].share_percent, Some("60.00".parse::<Decimal>().unwrap()));
    assert_eq!(bens[0].sort_order, 0);
    assert_eq!(bens[1].name, "新受益人丙");
    assert_eq!(bens[1].sort_order, 1);

    // DB 回读：旧行已被替换，新行按 sort_order 落库
    let rows: Vec<(String, String, Option<String>, i32)> = conn
        .exec(
            "SELECT name, beneficiary_type, share_percent, sort_order \
             FROM policy_beneficiaries WHERE policy_id = ? ORDER BY sort_order",
            vec![chain.policy_id],
        )
        .await
        .expect("回读受益人");
    assert_eq!(rows.len(), 2, "库里应只剩 2 名新受益人");
    assert_eq!(rows[0].0, "新受益人乙");
    assert_eq!(rows[0].2.as_deref(), Some("60.00"));
    assert_eq!(rows[1].0, "新受益人丙");

    // 审计：POLICY_ENDORSE，before 含旧名单、after 含新名单
    let audit: Option<(String, Option<String>, Option<String>)> = conn
        .exec_first(
            "SELECT action, before_json, after_json FROM audit_logs \
             WHERE entity_type = 'POLICY' AND entity_id = ? ORDER BY id DESC LIMIT 1",
            vec![chain.policy_id],
        )
        .await
        .expect("查询审计");
    let (action, before, after) = audit.expect("应有一条批改审计记录");
    assert_eq!(action, "POLICY_ENDORSE");
    let before = before.expect("before_json 不应为空");
    let after = after.expect("after_json 不应为空");
    assert!(before.contains("旧受益人甲"), "before 快照应含旧受益人: {before}");
    assert!(!before.contains("新受益人乙"), "before 快照不应含新受益人: {before}");
    assert!(after.contains("新受益人乙") && after.contains("新受益人丙"), "after 快照应含新名单: {after}");
    assert!(!after.contains("旧受益人甲"), "after 快照不应含旧受益人: {after}");

    cleanup_chain(&db, &chain).await;
}

#[tokio::test]
async fn endorse_rejects_missing_policy() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    // 不存在的保单（超大 id 必不命中），user_id 用占位值即可——校验先于事务提交。
    let err = svc(&db)
        .endorse_beneficiaries(1, i64::MAX, endorse_req(1, vec![named("张三", "SPOUSE", "100.00")]))
        .await
        .expect_err("保单不存在应失败");
    expect_business(err, "保单不存在");
}

#[tokio::test]
async fn endorse_forbids_other_users_policy() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    // 保单归属 user B；user A 越权批改应被拒。
    let chain_b = insert_chain(&db, &common::unique("endorse_owner_b")).await;
    set_policy_status(&db, chain_b.policy_id, "ACTIVE").await;
    let user_a = common::unique("endorse_trespasser_a");
    let user_a_id = insert_user(&db, &user_a).await;

    let err = svc(&db)
        .endorse_beneficiaries(
            user_a_id,
            chain_b.policy_id,
            endorse_req(user_a_id, vec![named("张三", "SPOUSE", "100.00")]),
        )
        .await
        .expect_err("他人保单批改应失败");
    match err {
        AppError::Forbidden => {}
        other => panic!("预期 Forbidden，得到 {other:?}"),
    }

    common::delete_user(&db, &user_a).await;
    cleanup_chain(&db, &chain_b).await;
}

#[tokio::test]
async fn endorse_rejects_non_active_policy() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    // 已失效保单（EXPIRED）不可批改。
    let chain = insert_chain(&db, &common::unique("endorse_expired")).await;
    set_policy_status(&db, chain.policy_id, "EXPIRED").await;

    let err = svc(&db)
        .endorse_beneficiaries(
            chain.user_id,
            chain.policy_id,
            endorse_req(chain.user_id, vec![named("张三", "SPOUSE", "100.00")]),
        )
        .await
        .expect_err("非 ACTIVE 保单批改应失败");
    match err {
        AppError::StateConflict(m) => assert!(m.contains("不可批改"), "冲突消息应含「不可批改」: {m}"),
        other => panic!("预期 StateConflict，得到 {other:?}"),
    }

    cleanup_chain(&db, &chain).await;
}

#[tokio::test]
async fn endorse_rejects_invalid_payload_before_db() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    // 名单校验发生在任何 DB 访问之前，policy_id 用占位值即可。
    let err = svc(&db)
        .endorse_beneficiaries(1, 1, endorse_req(1, vec![]))
        .await
        .expect_err("空名单应失败");
    expect_business(err, "名单不能为空");

    let err = svc(&db)
        .endorse_beneficiaries(
            1,
            1,
            endorse_req(1, vec![
                named("张三", "SPOUSE", "30.00"),
                named("李四", "CHILD", "30.00"),
            ]),
        )
        .await
        .expect_err("指定受益人占比合计不为 100 应失败");
    expect_business(err, "合计须为 100");
}
