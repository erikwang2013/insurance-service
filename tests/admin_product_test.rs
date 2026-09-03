//! 商品上架/下架管理端集成测试（任务 #7，MySQL 集成）
//!
//! 覆盖：非运营/管理员 upsert/上下架 → Forbidden；建档（含服务层补缺省）→ 回读；
//! 同 code 再 upsert → UPDATE（id 不变、内容更新、无唯一冲突）；
//! 上下架状态切换（含禁回 DRAFT 防呆、未命中 NotFound）；
//! 公开 list 过滤现状（ON_SALE 过滤生效；空 status 不过滤 —— 现状即测试锚点）。
//!
//! 依赖 `insurance_service` 库 + 本地 MySQL（install.sql 建库）。
//! MySQL 不可用时 SKIP（打印提示并提前返回），保证 `cargo test` 在无库环境不失败。

mod common;

use insurance_service::db::Db;
use insurance_service::error::AppError;
use insurance_service::services::product_service::{
    admin_change_status, admin_upsert, detail, list, AdminStatusReq, AdminUpsertReq,
};
use mysql_async::prelude::Queryable;
use mysql_async::Value;

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

/// 构造建档/更新请求（status 缺省走服务层 DRAFT 默认）
fn upsert_req(op: i64, code: &str, name: &str, status: Option<&str>) -> AdminUpsertReq {
    AdminUpsertReq {
        operator_user_id: op,
        product_code: code.to_string(),
        name: name.to_string(),
        subtitle: None,
        description: None,
        product_type: "HEALTH".to_string(),
        sale_channel: None,
        insurer_name: Some("测试保险公司".to_string()),
        currency: None,
        min_amount: Some("8000.00".parse().unwrap()),
        max_amount: None,
        min_term_months: None,
        max_term_months: None,
        waiting_period_days: None,
        is_featured: None,
        cover_image_url: None,
        search_enabled: None,
        status: status.map(String::from),
    }
}

/// 构造上下架请求
fn status_req(op: i64, status: &str) -> AdminStatusReq {
    AdminStatusReq {
        operator_user_id: op,
        status: status.to_string(),
    }
}

/// 断言错误为业务错误且消息包含指定片段。
fn expect_business(err: AppError, needle: &str) {
    match err {
        AppError::Business(m) => assert!(m.contains(needle), "业务错误消息应含 {needle:?}: {m}"),
        other => panic!("预期 Business({needle:?})，得到 {other:?}"),
    }
}

#[tokio::test]
async fn admin_upsert_forbids_non_operator() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let username = common::unique("ap_user");
    let user_id = insert_user(&db, &username, "USER").await;
    let code = common::unique("app");

    // 建档与上下架动作都必须被普通用户拒绝
    let err = admin_upsert(&db, &upsert_req(user_id, &code, "普通用户建档", None))
        .await
        .expect_err("普通用户建档应被拒");
    match err {
        AppError::Forbidden => {}
        other => panic!("预期 Forbidden，得到 {other:?}"),
    }
    let err = admin_change_status(&db, 1, &status_req(user_id, "ON_SALE"))
        .await
        .expect_err("普通用户上下架应被拒");
    match err {
        AppError::Forbidden => {}
        other => panic!("预期 Forbidden，得到 {other:?}"),
    }

    common::delete_user(&db, &username).await;
}

#[tokio::test]
async fn admin_upsert_inserts_with_defaults_and_reads_back() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let username_ap1 = common::unique("ap_op1");
    let op_id = insert_user(&db, &username_ap1, "OPERATOR").await;
    let code = common::unique("app");

    // 不传 status：服务层补缺省 DRAFT；其余缺省 ON_LINE/CNY/false/true
    let p = admin_upsert(&db, &upsert_req(op_id, &code, "运营建档测试", None))
        .await
        .expect("建档成功");
    assert_eq!(p.product_code, code);
    assert_eq!(p.name, "运营建档测试");
    assert_eq!(p.status, "DRAFT", "status 缺省应为 DRAFT");
    assert_eq!(p.sale_channel, "ONLINE", "sale_channel 缺省应为 ONLINE");
    assert_eq!(p.currency, "CNY", "currency 缺省应为 CNY");
    assert!(!p.is_featured, "is_featured 缺省应为 false");
    assert!(p.search_enabled, "search_enabled 缺省应为 true");
    assert_eq!(p.operator_user_id, Some(op_id));
    assert_eq!(p.min_amount.as_ref().unwrap().to_string(), "8000.00");

    // 详情接口应命中同一行（回读一致性）
    let by_id = detail(&db, p.id).await.expect("详情命中");
    assert_eq!(by_id.product_code, code);

    common::delete_product(&db, &code).await;
    common::delete_user(&db, &username_ap1).await;
}

#[tokio::test]
async fn admin_upsert_same_code_updates_in_place() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let username_ap2 = common::unique("ap_op2");
    let op_id = insert_user(&db, &username_ap2, "ADMIN").await;
    let code = common::unique("app");

    let first = admin_upsert(&db, &upsert_req(op_id, &code, "初版名称", Some("ON_SALE")))
        .await
        .expect("首次建档");
    assert_eq!(first.status, "ON_SALE");

    // 同 code 再次 upsert（改名）→ 应 UPDATE：id 不变、无唯一冲突、内容更新
    let second = admin_upsert(&db, &upsert_req(op_id, &code, "更新后名称", Some("ON_SALE")))
        .await
        .expect("同 code 更新");
    assert_eq!(second.id, first.id, "同 code 更新不应产生新行");
    assert_eq!(second.name, "更新后名称");
    assert_eq!(second.status, "ON_SALE");
    let by_id = detail(&db, first.id).await.expect("详情命中");
    assert_eq!(by_id.name, "更新后名称", "更新应落库");

    common::delete_product(&db, &code).await;
    common::delete_user(&db, &username_ap2).await;
}

#[tokio::test]
async fn admin_change_status_toggles_shelf_state() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let username_ap3 = common::unique("ap_op3");
    let op_id = insert_user(&db, &username_ap3, "OPERATOR").await;
    let code = common::unique("app");
    let p = admin_upsert(&db, &upsert_req(op_id, &code, "上下架测试", Some("ON_SALE")))
        .await
        .expect("建档成功");

    // 下架：只切状态与操作人
    let off = admin_change_status(&db, p.id, &status_req(op_id, "OFF_SHELF"))
        .await
        .expect("下架成功");
    assert_eq!(off.status, "OFF_SHELF");
    assert_eq!(off.operator_user_id, Some(op_id));
    assert_eq!(off.name, "上下架测试", "切换状态不应改其他字段");

    // 公开列表过滤现状：OFF_SHELF 不出现在 ON_SALE 列表
    let on_sale = list(&db, "ON_SALE", 1, 100).await.expect("查询成功");
    assert!(
        !on_sale.iter().any(|x| x.product_code == code),
        "OFF_SHELF 商品不应出现在 ON_SALE 列表"
    );
    let off_shelf = list(&db, "OFF_SHELF", 1, 100).await.expect("查询成功");
    assert!(
        off_shelf.iter().any(|x| x.product_code == code),
        "OFF_SHELF 列表应包含 {code}"
    );
    // 现状锚点：status 为空（等效全部）时不会过滤 —— 公开接口若不带 status 会露出下架品
    let all = list(&db, "", 1, 100).await.expect("查询成功");
    assert!(
        all.iter().any(|x| x.product_code == code),
        "空 status 列表按现状不过滤，应包含 {code}"
    );

    common::delete_product(&db, &code).await;
    common::delete_user(&db, &username_ap3).await;
}

#[tokio::test]
async fn admin_change_status_validates_draft_and_missing() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let username_ap4 = common::unique("ap_op4");
    let op_id = insert_user(&db, &username_ap4, "ADMIN").await;
    let code = common::unique("app");
    let p = admin_upsert(&db, &upsert_req(op_id, &code, "防呆测试", Some("ON_SALE")))
        .await
        .expect("建档成功");

    // 防呆：不得回到 DRAFT
    expect_business(
        admin_change_status(&db, p.id, &status_req(op_id, "DRAFT"))
            .await
            .expect_err("回 DRAFT 应失败"),
        "禁用回 DRAFT",
    );
    // 状态仍为 ON_SALE（防呆失败不应留下半状态）
    let unchanged = detail(&db, p.id).await.expect("详情命中");
    assert_eq!(unchanged.status, "ON_SALE");

    // 未命中（不存在/已软删）→ NotFound
    let err = admin_change_status(&db, i64::MAX, &status_req(op_id, "ON_SALE"))
        .await
        .expect_err("不存在的商品应失败");
    match err {
        AppError::NotFound => {}
        other => panic!("预期 NotFound，得到 {other:?}"),
    }

    common::delete_product(&db, &code).await;
    common::delete_user(&db, &username_ap4).await;
}
