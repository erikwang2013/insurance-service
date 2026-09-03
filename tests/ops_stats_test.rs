//! 运营统计 API 集成测试（任务 #15，MySQL 集成）
//!
//! 覆盖：非 OPERATOR/ADMIN 操作人 → Forbidden；用户/商品计数与软删隔离；
//! 成交订单（PAID）与成功支付（SUCCESS）金额汇总，FAILED 不计；理赔/保单按
//! 状态分组计数。
//!
//! 共享测试库下采用「插入前后 overview 差量」断言：自身写入可精确归因；
//! 与并行任务并发运行时有毫秒级交错窗口（重跑即绿），串行全量验证在 #20。
//!
//! 依赖 `insurance_service` 库 + 本地 MySQL（install.sql 建库）。
//! MySQL 不可用时 SKIP（打印提示并提前返回），保证 `cargo test` 无库环境不失败。

mod common;

use std::str::FromStr;

use insurance_service::db::Db;
use insurance_service::error::AppError;
use insurance_service::services::stats_service::{Overview, StatsReq, StatsService};
use mysql_async::prelude::Queryable;
use mysql_async::Value;
use rust_decimal::Decimal;

/// 一条最小业务链（用户 → 产品 → 报价 → 订单 → 保单）。
struct Chain {
    username: String,
    product_code: String,
    quote_no: String,
    order_no: String,
    policy_no: String,
}

/// 插入指定角色的测试用户，返回自增 id。
async fn insert_user(db: &Db, username: &str, role: &str) -> i64 {
    let mut conn = db.conn().await.expect("连接测试库");
    conn.exec_drop(
        "INSERT INTO users (username, password_hash, role) VALUES (?, ?, ?)",
        vec![
            Value::from(username),
            Value::from("test-hash"),
            Value::from(role),
        ],
    )
    .await
    .expect("插入用户");
    conn.last_insert_id().expect("取得自增 id") as i64
}

/// 插入产品（status 显式指定），返回自增 id。
async fn insert_product(db: &Db, product_code: &str, status: &str) -> i64 {
    let mut conn = db.conn().await.expect("连接测试库");
    conn.exec_drop(
        "INSERT INTO insurance_products (product_code, name, product_type, status) \
         VALUES (?, ?, ?, ?)",
        vec![
            Value::from(product_code),
            Value::from("统计测试产品"),
            Value::from("HEALTH"),
            Value::from(status),
        ],
    )
    .await
    .expect("插入产品");
    conn.last_insert_id().expect("取得自增 id") as i64
}

/// 按 FK 顺序插入 用户 → 产品 → 报价 → 订单（order_status 显式）→ 保单。
async fn insert_chain(db: &Db, username: &str, order_status: &str) -> Chain {
    let user_id = insert_user(db, username, "USER").await;
    let mut conn = db.conn().await.expect("连接测试库");

    let product_code = common::unique("sp");
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

    let quote_no = common::unique("sq");
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

    let order_no = common::unique("so");
    conn.exec_drop(
        "INSERT INTO orders \
           (order_no, quote_id, user_id, product_id, product_name, holder_name, \
            insurance_amount, term_months, total_amount, payable_amount, status) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
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
            Value::from(order_status),
        ],
    )
    .await
    .expect("插入订单");
    let order_id = conn.last_insert_id().expect("取得自增 id") as i64;

    let policy_no = common::unique("spn");
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

    Chain {
        username: username.to_string(),
        product_code,
        quote_no,
        order_no,
        policy_no,
    }
}

/// 插入支付流水（channel MOCK；amount/status 显式），随订单级联清理。
async fn insert_payment(db: &Db, order_id: i64, user_id: i64, amount: &str, status: &str) {
    let mut conn = db.conn().await.expect("连接测试库");
    conn.exec_drop(
        "INSERT INTO payments (payment_no, order_id, user_id, amount, channel, status) \
         VALUES (?, ?, ?, ?, 'MOCK', ?)",
        vec![
            Value::from(common::unique("spy")),
            Value::from(order_id),
            Value::from(user_id),
            Value::from(amount),
            Value::from(status),
        ],
    )
    .await
    .expect("插入支付流水");
}

/// 插入理赔（claim_no 唯一，status 走默认 SUBMITTED），返回 claim_no。
async fn insert_claim(db: &Db, policy_id: i64, order_id: i64, user_id: i64) -> String {
    let claim_no = common::unique("scl");
    let mut conn = db.conn().await.expect("连接测试库");
    conn.exec_drop(
        "INSERT INTO claims (claim_no, policy_id, order_id, user_id, claim_amount) \
         VALUES (?, ?, ?, ?, ?)",
        vec![
            Value::from(&claim_no),
            Value::from(policy_id),
            Value::from(order_id),
            Value::from(user_id),
            Value::from("3000.00"),
        ],
    )
    .await
    .expect("插入理赔");
    claim_no
}

/// 软删用户 / 产品（统计应不再计入）。
async fn soft_delete_user(db: &Db, username: &str) {
    let _ = db
        .exec_drop(
            "UPDATE users SET deleted_at = NOW() WHERE username = ?",
            vec![username],
        )
        .await;
}

async fn soft_delete_product(db: &Db, product_code: &str) {
    let _ = db
        .exec_drop(
            "UPDATE insurance_products SET deleted_at = NOW() WHERE product_code = ?",
            vec![product_code],
        )
        .await;
}

/// 逆 FK 序清理一条链（payments 随订单 CASCADE；claims 需用例先按号删除）。
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

/// 断言错误为 Forbidden。
fn expect_forbidden(err: AppError) {
    match err {
        AppError::Forbidden => {}
        other => panic!("预期 Forbidden，得到 {other:?}"),
    }
}

/// 金额字符串（SUM CAST AS CHAR）→ Decimal，便于差量比较。
fn money(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap_or_else(|e| panic!("金额串 {s:?} 解析失败: {e}"))
}

fn svc(db: &Db) -> StatsService {
    StatsService::new(db.clone())
}

async fn overview(db: &Db, operator_user_id: i64) -> Overview {
    svc(db)
        .overview(StatsReq { operator_user_id })
        .await
        .expect("运营总览应成功")
}

/// 单测试串行覆盖四个场景：同文件多测试并行会在共享库上互相污染全局计数
/// 差量断言（delta=2 实测命中），故合并为一个顺序流程；跨测试二进制由 cargo
/// 顺序执行，残余并发仅剩并行 agent 的独立进程（窗口毫秒级，重跑即绿）。
#[tokio::test]
async fn overview_stats_api_flow() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let op_name = common::unique("sop");
    let op_id = insert_user(&db, &op_name, "OPERATOR").await;

    // ---- 场景 1：角色鉴权（USER 与不存在的操作人 → Forbidden） ----
    let u_name = common::unique("sfo");
    let u_id = insert_user(&db, &u_name, "USER").await;
    let err = svc(&db)
        .overview(StatsReq { operator_user_id: u_id })
        .await
        .expect_err("USER 角色应被拒");
    expect_forbidden(err);
    let err = svc(&db)
        .overview(StatsReq {
            operator_user_id: 999_999_999,
        })
        .await
        .expect_err("不存在的操作人应被拒");
    expect_forbidden(err);
    common::delete_user(&db, &u_name).await;

    // ---- 场景 2：用户/产品计数与软删隔离 ----
    let before = overview(&db, op_id).await;
    let u_name = common::unique("sct");
    let p_code = common::unique("spd");
    insert_user(&db, &u_name, "USER").await;
    insert_product(&db, &p_code, "ON_SALE").await;
    let mid = overview(&db, op_id).await;
    assert_eq!(mid.users - before.users, 1, "新用户应计入总数");
    assert_eq!(mid.products.total - before.products.total, 1, "新产品应计入总数");
    assert_eq!(mid.products.on_sale - before.products.on_sale, 1, "ON_SALE 计入在售");
    assert_eq!(mid.products.others - before.products.others, 0, "ON_SALE 不计入其他");
    soft_delete_user(&db, &u_name).await;
    soft_delete_product(&db, &p_code).await;
    let after = overview(&db, op_id).await;
    assert_eq!(after.users, before.users, "软删用户不计入");
    assert_eq!(after.products.total, before.products.total, "软删产品不计入");

    // ---- 场景 3：PAID 订单成交额 + SUCCESS 支付额（FAILED 排除） ----
    let before = overview(&db, op_id).await;
    let c = insert_chain(&db, &common::unique("st3"), "PAID").await;
    let (user_id, order_id) = {
        let mut conn = db.conn().await.expect("连接测试库");
        let row: Option<(i64, i64)> = conn
            .exec_first(
                "SELECT u.id, o.id FROM orders o JOIN users u ON u.id = o.user_id \
                 WHERE o.order_no = ?",
                vec![c.order_no.as_str()],
            )
            .await
            .expect("回读链 id");
        row.expect("订单链应存在")
    };
    insert_payment(&db, order_id, user_id, "800.00", "SUCCESS").await;
    insert_payment(&db, order_id, user_id, "900.00", "FAILED").await;
    let after = overview(&db, op_id).await;
    assert_eq!(after.orders.total - before.orders.total, 1);
    assert_eq!(after.orders.paid - before.orders.paid, 1, "PAID 计入成交单数");
    assert_eq!(
        money(&after.orders.paid_amount) - money(&before.orders.paid_amount),
        Decimal::from_str("5000.00").unwrap(),
        "成交总额按 payable_amount 累计"
    );
    assert_eq!(
        money(&after.payments.success_amount) - money(&before.payments.success_amount),
        Decimal::from_str("800.00").unwrap(),
        "仅 SUCCESS 计入支付成功额（FAILED 900.00 应被排除）"
    );
    cleanup_chain(&db, &c).await;

    // ---- 场景 4：理赔/保单按状态分组 ----
    let before = overview(&db, op_id).await;
    let c = insert_chain(&db, &common::unique("st4"), "CREATED").await;
    let claim_no = {
        let mut conn = db.conn().await.expect("连接测试库");
        let row: Option<(i64, i64, i64)> = conn
            .exec_first(
                "SELECT p.id, p.order_id, p.user_id FROM policies p \
                 WHERE p.policy_no = ?",
                vec![c.policy_no.as_str()],
            )
            .await
            .expect("回读保单 id");
        let (policy_id, order_id, user_id) = row.expect("保单应存在");
        insert_claim(&db, policy_id, order_id, user_id).await
    };
    let after = overview(&db, op_id).await;
    let sub_before = before.claims.by_status.get("SUBMITTED").copied().unwrap_or(0);
    let sub_after = after.claims.by_status.get("SUBMITTED").copied().unwrap_or(0);
    assert_eq!(after.claims.total - before.claims.total, 1, "理赔总数 +1");
    assert_eq!(sub_after, sub_before + 1, "SUBMITTED 分组 +1");
    let pi_before = before
        .policies
        .by_status
        .get("PENDING_ISSUE")
        .copied()
        .unwrap_or(0);
    let pi_after = after
        .policies
        .by_status
        .get("PENDING_ISSUE")
        .copied()
        .unwrap_or(0);
    assert_eq!(after.policies.total - before.policies.total, 1, "保单总数 +1");
    assert_eq!(pi_after, pi_before + 1, "PENDING_ISSUE 分组 +1");
    cleanup_claim(&db, &claim_no).await;
    cleanup_chain(&db, &c).await;

    common::delete_user(&db, &op_name).await;
}
