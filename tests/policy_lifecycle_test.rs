//! 保单生命周期集成测试：续保（renew）与退保（lapse）
//!
//! 覆盖：
//! - 续保成功（ACTIVE 无缝接续日期 / EXPIRED 从今天起保）、issue_type=RENEW、
//!   原保单不受影响；不可续保标记 / 终态 / 他人保单 / 不存在保单 均被拒；
//! - 退保成功（ACTIVE → SURRENDERED + audit_logs 留痕）；重复退保 / 非在保态 /
//!   他人保单 / 不存在保单 均被拒。
//!
//! 依赖 `insurance_service` 库 + 本地 MySQL（install.sql 建库，DATABASE_URL 指向
//! 13307 测试实例）。与 claim_service_test 不同：本文件 MySQL 不可用时直接
//! panic（FAIL），不做 SKIP——任务要求"DB 不可用 = FAIL"。
//!
//! policies 有外键链：policies → orders → quotes → products/users，
//! 故每条链按序插入 用户 → 产品 → 报价 → 订单，再按需插入保单，结束逆序清理。

mod common;

use chrono::{Datelike, NaiveDate, Utc};
use insurance_service::db::Db;
use insurance_service::error::AppError;
use insurance_service::models::policy::Policy;
use insurance_service::services::policy_service::PolicyService;
use mysql_async::prelude::Queryable;
use mysql_async::Value;

/// 保单依赖链（用户→产品→报价→订单），policy 由各用例按需追加。
struct Chain {
    username: String,
    product_code: String,
    quote_no: String,
    order_no: String,
    user_id: i64,
    order_id: i64,
    quote_id: i64,
}

/// 一条测试保单（id + policy_no，供断言与清理）。
struct TestPolicy {
    id: i64,
    policy_no: String,
}

fn svc(db: &Db) -> PolicyService {
    PolicyService::new(db.clone())
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

/// 按 FK 顺序插入 用户 → 产品 → 报价 → 订单。
async fn insert_chain(db: &Db, username: &str) -> Chain {
    let mut conn = db.conn().await.expect("连接测试库");
    let user_id = insert_user(db, username).await;

    let product_code = common::unique("lpc");
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

    let quote_no = common::unique("lq");
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

    let order_no = common::unique("lo");
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

    Chain { username: username.to_string(), product_code, quote_no, order_no, user_id, order_id, quote_id }
}

/// 插入一张指定状态的保单，返回其 id 与单号。
async fn insert_policy(
    db: &Db,
    c: &Chain,
    status: &str,
    is_renewable: bool,
    effective: NaiveDate,
    expire: NaiveDate,
) -> TestPolicy {
    let mut conn = db.conn().await.expect("连接测试库");
    let policy_no = common::unique("lpn");
    conn.exec_drop(
        "INSERT INTO policies \
           (policy_no, order_id, quote_id, user_id, product_id, product_name, holder_name, \
            insurance_amount, premium, term_months, effective_date, expire_date, \
            status, issue_type, is_renewable) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        vec![
            Value::from(&policy_no),
            Value::from(c.order_id),
            Value::from(c.quote_id),
            Value::from(c.user_id),
            Value::from(1_i64), // product_id 占位，无 FK 约束校验
            Value::from("测试产品"),
            Value::from("测试被保人"),
            Value::from("100000.00"),
            Value::from("5000.00"),
            Value::from(12_i32),
            Value::from(effective.format("%Y-%m-%d").to_string()),
            Value::from(expire.format("%Y-%m-%d").to_string()),
            Value::from(status),
            Value::from("NEW"),
            Value::from(is_renewable),
        ],
    )
    .await
    .expect("插入保单");
    let id = conn.last_insert_id().expect("取得自增 id") as i64;
    TestPolicy { id, policy_no }
}

/// 删除该链的保单（含续保新单，按单号逐个）、审计留痕（按 entity_id）、
/// 订单、报价、产品与用户。
async fn cleanup_chain(db: &Db, c: &Chain, policies: &[TestPolicy]) {
    for p in policies {
        let _ = db
            .exec_drop("DELETE FROM policies WHERE policy_no = ?", vec![p.policy_no.as_str()])
            .await;
        let _ = db
            .exec_drop(
                "DELETE FROM audit_logs WHERE entity_type = 'POLICY' AND entity_id = ?",
                vec![p.id],
            )
            .await;
    }
    let _ = db.exec_drop("DELETE FROM orders WHERE order_no = ?", vec![c.order_no.as_str()]).await;
    let _ = db.exec_drop("DELETE FROM quotes WHERE quote_no = ?", vec![c.quote_no.as_str()]).await;
    let _ = db
        .exec_drop("DELETE FROM insurance_products WHERE product_code = ?", vec![c.product_code.as_str()])
        .await;
    common::delete_user(db, &c.username).await;
}

fn expect_business(err: AppError, needle: &str) {
    match err {
        AppError::Business(m) => assert!(m.contains(needle), "业务错误消息应含 {needle:?}: {m}"),
        other => panic!("预期 Business({needle:?})，得到 {other:?}"),
    }
}

fn expect_state_conflict(err: AppError, needle: &str) {
    match err {
        AppError::StateConflict(m) => assert!(m.contains(needle), "状态冲突消息应含 {needle:?}: {m}"),
        other => panic!("预期 StateConflict({needle:?})，得到 {other:?}"),
    }
}

fn expect_forbidden(err: AppError) {
    match err {
        AppError::Forbidden => {}
        other => panic!("预期 Forbidden，得到 {other:?}"),
    }
}

/// 目标月最后一天的日号（测试侧独立实现，用于日期夹取断言）。
fn last_day_of(y: i32, m: u32) -> u32 {
    if m == 2 && (y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)) {
        29
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][m as usize - 1]
    }
}

/// 期望的整年后续期日：下一年同日；不存在（2/29）时夹取到当月最后一天。
fn expire_in_year(effective: NaiveDate) -> NaiveDate {
    let y = effective.year() + 1;
    NaiveDate::from_ymd_opt(y, effective.month(), effective.day())
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(y, effective.month(), last_day_of(y, effective.month())).unwrap())
}

#[tokio::test]
async fn renew_extends_active_policy_seamlessly() {
    let db = common::test_db().await.expect("MySQL 不可用 = FAIL（任务要求，不做 SKIP）");
    let chain = insert_chain(&db, &common::unique("renew_owner")).await;
    let old = insert_policy(
        &db,
        &chain,
        Policy::STATUS_ACTIVE,
        true,
        NaiveDate::from_ymd_opt(2035, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2036, 1, 1).unwrap(),
    )
    .await;

    let renewed = svc(&db)
        .renew(chain.user_id, old.id)
        .await
        .expect("续保成功");
    assert_eq!(renewed.issue_type, "RENEW", "续保单 issue_type 应为 RENEW");
    assert_eq!(renewed.status, Policy::STATUS_ACTIVE, "续保单初始状态应为 ACTIVE");
    assert_ne!(renewed.policy_no, old.policy_no, "续保单号应不同");
    assert_eq!(renewed.user_id, chain.user_id, "续保归属同一用户");
    assert_eq!(renewed.order_id, chain.order_id, "沿用原单 order_id");
    assert_eq!(renewed.effective_date, NaiveDate::from_ymd_opt(2036, 1, 2).unwrap(), "止期次日无缝接续");
    assert_eq!(renewed.expire_date, NaiveDate::from_ymd_opt(2037, 1, 2).unwrap(), "接续期 +12 个月");
    assert!(renewed.is_renewable, "is_renewable 继承原单");
    assert_eq!(renewed.insurance_amount.to_string(), "100000.00", "保额沿用原单");
    assert_eq!(renewed.premium.to_string(), "5000.00", "保费沿用原单");

    // 原保单保持 ACTIVE 不变，到期日未被改写
    let still = svc(&db).by_id(old.id).await.expect("原保单仍可查");
    assert_eq!(still.status, Policy::STATUS_ACTIVE);
    assert_eq!(still.expire_date, NaiveDate::from_ymd_opt(2036, 1, 1).unwrap());
    assert_eq!(still.policy_no, old.policy_no);

    cleanup_chain(&db, &chain, &[old, TestPolicy { id: renewed.id, policy_no: renewed.policy_no.clone() }]).await;
}

#[tokio::test]
async fn renew_expired_policy_starts_today() {
    let db = common::test_db().await.expect("MySQL 不可用 = FAIL（任务要求，不做 SKIP）");
    let chain = insert_chain(&db, &common::unique("renew_lapse_owner")).await;
    // 早已满期的保单：止期 2021-01-01，今日早已超过
    let old = insert_policy(
        &db,
        &chain,
        Policy::STATUS_EXPIRED,
        true,
        NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
    )
    .await;

    let renewed = svc(&db)
        .renew(chain.user_id, old.id)
        .await
        .expect("满期保单可续保");
    let today = Utc::now().date_naive();
    assert_eq!(renewed.issue_type, "RENEW");
    assert_eq!(renewed.status, Policy::STATUS_ACTIVE);
    assert_eq!(renewed.effective_date, today, "止期已过应自今天起保");
    assert_eq!(renewed.expire_date, expire_in_year(today), "今天起保 +12 个月");
    assert!(renewed.expire_date > today);

    cleanup_chain(&db, &chain, &[old, TestPolicy { id: renewed.id, policy_no: renewed.policy_no.clone() }]).await;
}

#[tokio::test]
async fn renew_rejects_non_renewable_policy() {
    let db = common::test_db().await.expect("MySQL 不可用 = FAIL（任务要求，不做 SKIP）");
    let chain = insert_chain(&db, &common::unique("renew_flag_owner")).await;
    // 在保但 is_renewable=0（如核保标记为不可续的产品）
    let old = insert_policy(
        &db,
        &chain,
        Policy::STATUS_ACTIVE,
        false,
        NaiveDate::from_ymd_opt(2035, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2036, 1, 1).unwrap(),
    )
    .await;

    let err = svc(&db).renew(chain.user_id, old.id).await.expect_err("不可续保单应被拒");
    expect_business(err, "不支持续保");

    // 未产生任何新保单：用户保单列表仍只有原单
    let list = svc(&db).by_user(chain.user_id, 1, 100).await.expect("列表查询成功");
    let mine: Vec<&str> = list.iter().filter(|p| p.policy_no == old.policy_no).map(|p| p.policy_no.as_str()).collect();
    assert_eq!(mine.len(), 1, "续保被拒后不应产生新保单");

    cleanup_chain(&db, &chain, &[old]).await;
}

#[tokio::test]
async fn renew_rejects_terminal_surrendered_policy() {
    let db = common::test_db().await.expect("MySQL 不可用 = FAIL（任务要求，不做 SKIP）");
    let chain = insert_chain(&db, &common::unique("renew_term_owner")).await;
    let old = insert_policy(
        &db,
        &chain,
        Policy::STATUS_SURRENDERED,
        true,
        NaiveDate::from_ymd_opt(2035, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2036, 1, 1).unwrap(),
    )
    .await;

    let err = svc(&db).renew(chain.user_id, old.id).await.expect_err("已退保保单不可续");
    expect_state_conflict(err, "不可续保");

    cleanup_chain(&db, &chain, &[old]).await;
}

#[tokio::test]
async fn renew_forbids_other_users_policy() {
    let db = common::test_db().await.expect("MySQL 不可用 = FAIL（任务要求，不做 SKIP）");
    let chain_b = insert_chain(&db, &common::unique("renew_owner_b")).await;
    let old = insert_policy(
        &db,
        &chain_b,
        Policy::STATUS_ACTIVE,
        true,
        NaiveDate::from_ymd_opt(2035, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2036, 1, 1).unwrap(),
    )
    .await;
    let user_a = insert_user(&db, &common::unique("renew_trespasser")).await;

    let err = svc(&db).renew(user_a, old.id).await.expect_err("他人保单续保应被拒");
    expect_forbidden(err);

    let _ = db.exec_drop("DELETE FROM users WHERE id = ?", vec![user_a]).await;
    cleanup_chain(&db, &chain_b, &[old]).await;
}

#[tokio::test]
async fn renew_rejects_missing_policy() {
    let db = common::test_db().await.expect("MySQL 不可用 = FAIL（任务要求，不做 SKIP）");
    // 超大 id 必不命中，无需真实用户（校验先于 INSERT）
    let err = svc(&db).renew(1, i64::MAX).await.expect_err("不存在的保单续保应失败");
    expect_business(err, "保单不存在");
}

#[tokio::test]
async fn lapse_surrenders_active_policy_with_reason() {
    let db = common::test_db().await.expect("MySQL 不可用 = FAIL（任务要求，不做 SKIP）");
    let chain = insert_chain(&db, &common::unique("lapse_owner")).await;
    let old = insert_policy(
        &db,
        &chain,
        Policy::STATUS_ACTIVE,
        true,
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2036, 1, 1).unwrap(),
    )
    .await;

    let reason = "经济原因不再续保".to_string();
    let lapsed = svc(&db).lapse(chain.user_id, old.id, Some(reason.clone())).await.expect("退保成功");
    assert_eq!(lapsed.status, Policy::STATUS_SURRENDERED, "退保后状态应为 SURRENDERED");
    assert_eq!(lapsed.policy_no, old.policy_no, "退保不换单号");

    // 回读确认已持久化
    let by_id = svc(&db).by_id(old.id).await.expect("退保保单可查");
    assert_eq!(by_id.status, Policy::STATUS_SURRENDERED);

    // 原因与前后状态落 audit_logs（policies 无原因列）
    let mut conn = db.conn().await.expect("连接测试库");
    let audit: Option<(String, String, Option<String>)> = conn
        .exec_first(
            "SELECT action, CAST(before_json AS CHAR), CAST(after_json AS CHAR) \
             FROM audit_logs WHERE entity_type = 'POLICY' AND entity_id = ? AND action = 'POLICY_LAPSE'",
            vec![old.id],
        )
        .await
        .expect("查询审计日志");
    let (action, before, after) = audit.expect("应存在退保审计记录");
    assert_eq!(action, "POLICY_LAPSE");
    assert!(before.contains(Policy::STATUS_ACTIVE), "before_json 应记录原状态: {before}");
    let after = after.expect("after_json 非空");
    assert!(after.contains(Policy::STATUS_SURRENDERED), "after_json 应记录退保状态: {after}");
    assert!(after.contains(&reason), "after_json 应含退保原因: {after}");

    cleanup_chain(&db, &chain, &[old]).await;
}

#[tokio::test]
async fn lapse_rejects_repeat_and_non_active_states() {
    let db = common::test_db().await.expect("MySQL 不可用 = FAIL（任务要求，不做 SKIP）");
    let chain = insert_chain(&db, &common::unique("lapse_states_owner")).await;
    let act = insert_policy(
        &db,
        &chain,
        Policy::STATUS_ACTIVE,
        true,
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2036, 1, 1).unwrap(),
    )
    .await;
    let pend = insert_policy(
        &db,
        &chain,
        Policy::STATUS_PENDING_ISSUE,
        true,
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2036, 1, 1).unwrap(),
    )
    .await;
    let exp = insert_policy(
        &db,
        &chain,
        Policy::STATUS_EXPIRED,
        true,
        NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
    )
    .await;

    // 未生效 / 已满期都不可退
    let err = svc(&db).lapse(chain.user_id, pend.id, None).await.expect_err("未生效保单不可退");
    expect_state_conflict(err, "不可退保");
    let err = svc(&db).lapse(chain.user_id, exp.id, None).await.expect_err("已满期保单不可退");
    expect_state_conflict(err, "不可退保");

    // 首次退保成功，重复退保被拦（同因不同原因都拦）
    svc(&db).lapse(chain.user_id, act.id, Some("test".to_string())).await.expect("首次退保成功");
    let err = svc(&db).lapse(chain.user_id, act.id, Some("again".to_string())).await.expect_err("重复退保应被拒");
    expect_state_conflict(err, "不可退保");

    // 审计只留一条退保记录
    let mut conn = db.conn().await.expect("连接测试库");
    let cnt: Option<i64> = conn
        .exec_first(
            "SELECT COUNT(*) FROM audit_logs WHERE entity_type = 'POLICY' AND entity_id = ? AND action = 'POLICY_LAPSE'",
            vec![act.id],
        )
        .await
        .expect("查询审计日志");
    assert_eq!(cnt, Some(1), "退保失败不应重复写审计");

    cleanup_chain(&db, &chain, &[act, pend, exp]).await;
}

#[tokio::test]
async fn lapse_forbids_other_user_and_missing_policy() {
    let db = common::test_db().await.expect("MySQL 不可用 = FAIL（任务要求，不做 SKIP）");
    let chain_b = insert_chain(&db, &common::unique("lapse_owner_b")).await;
    let old = insert_policy(
        &db,
        &chain_b,
        Policy::STATUS_ACTIVE,
        true,
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2036, 1, 1).unwrap(),
    )
    .await;
    let user_a = insert_user(&db, &common::unique("lapse_trespasser")).await;

    let err = svc(&db).lapse(user_a, old.id, None).await.expect_err("他人保单退保应被拒");
    expect_forbidden(err);
    // 越权被拒后原单仍 ACTIVE
    assert_eq!(svc(&db).by_id(old.id).await.expect("可查").status, Policy::STATUS_ACTIVE);

    let err = svc(&db).lapse(chain_b.user_id, i64::MAX, None).await.expect_err("不存在保单退保应失败");
    expect_business(err, "保单不存在");

    let _ = db.exec_drop("DELETE FROM users WHERE id = ?", vec![user_a]).await;
    cleanup_chain(&db, &chain_b, &[old]).await;
}
