//! 搜索服务集成测试（任务 #5，MySQL 集成）
//!
//! 覆盖：关键词命中（LIKE）、未命中返回空、分页保护、索引路由（未知索引 → Search 错误）。
//!
//! 依赖 `insurance_service` 库 + 本地 MySQL（install.sql 建库）。
//! MySQL 不可用时 SKIP（打印提示并提前返回），保证 `cargo test` 在无库环境不失败。

mod common;

use insurance_service::db::Db;
use insurance_service::error::AppError;
use insurance_service::services::search_service::search;
use mysql_async::prelude::Queryable;
use mysql_async::Value;

/// 往 insurance_products 插入一行（与 product_service_test 相同结构）。
async fn insert_product(db: &Db, code: &str, name: &str) -> i64 {
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
            Value::from(product_id),
            Value::from(code),
            Value::from(name),
            Value::from(format!("{name} 副标题")),
            Value::from(format!("{name} 详情描述")),
            Value::from("HEALTH"), // product_type
            Value::from("ONLINE"), // sale_channel
            Value::from("测试保险公司"),
            Value::from("CNY"),
            Value::from(100000.00_f64), // min_amount
            Value::from(200000.00_f64), // max_amount
            Value::from(12_i32),        // min_term_months
            Value::from(120_i32),       // max_term_months
            Value::from(0_i32),         // waiting_period_days
            Value::from(0_i64),         // is_featured
            Value::from("ON_SALE"),
            Value::from(1_i64), // search_enabled
        ],
    )
    .await
    .expect("插入商品");
    product_id
}

#[tokio::test]
async fn search_matches_keyword_in_name() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let code = common::unique("p");
    insert_product(&db, &code, "中英守护重疾险").await;

    let res = search(&db, "中英守护", Some("insurance_products"), 1, 10)
        .await
        .expect("搜索成功");
    let hit = res
        .hits
        .iter()
        .find(|h| h.doc.get("product_code").and_then(|v| v.as_str()) == Some(code.as_str()));
    assert!(hit.is_some(), "LIKE 搜索应命中刚插入的商品 {code}");

    common::delete_product(&db, &code).await;
}

#[tokio::test]
async fn search_no_match_returns_empty() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let code = common::unique("p");
    insert_product(&db, &code, "中英守护重疾险").await;

    // 不可能命中的关键词
    let res = search(&db, "绝不存在的关键词xyz", Some("insurance_products"), 1, 10)
        .await
        .expect("搜索成功");
    assert_eq!(res.total, 0);
    assert!(res.hits.is_empty(), "无匹配时应返回空结果");

    common::delete_product(&db, &code).await;
}

#[tokio::test]
async fn search_paging_protection() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let code = common::unique("p");
    insert_product(&db, &code, "分页保护测试险").await;

    // page=0 → 按第 1 页处理；size 超限 → clamp（均不 panic）
    let res = search(&db, "分页保护", Some("insurance_products"), 0, 999)
        .await
        .expect("搜索成功");
    assert_eq!(res.page, 0, "page 原样回传（查询内部按 max(1) 处理）");
    let _ = res;

    common::delete_product(&db, &code).await;
}

#[tokio::test]
async fn search_unknown_index_rejected() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let err = search(&db, "中英", Some("不存在的索引"), 1, 10)
        .await
        .expect_err("未知索引应失败");
    match err {
        AppError::Search(msg) => assert!(msg.contains("索引") || msg.contains("index"), "msg={msg}"),
        other => panic!("预期 Search 错误，得到 {other:?}"),
    }
}
