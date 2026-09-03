//! 商品服务集成测试（任务 #5，MySQL 集成）
//!
//! 覆盖：创建商品 → 列表（分页/状态过滤/软删排除）→ 详情（命中/未命中）。
//!
//! 依赖 `insurance_service` 库 + 本地 MySQL（install.sql 建库）。
//! MySQL 不可用时 SKIP（打印提示并提前返回），保证 `cargo test` 在无库环境不失败。

mod common;

use insurance_service::db::Db;
use insurance_service::services::product_service::{detail, list};
use mysql_async::prelude::Queryable;
use mysql_async::Value;

/// 往 insurance_products 插入一行，返回预生成 id。
async fn insert_product(db: &Db, code: &str, name: &str, status: &str, featured: u8) -> i64 {
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
            Value::from(i64::from(featured)), // is_featured
            Value::from(status),
            Value::from(1_i64), // search_enabled
        ],
    )
    .await
    .expect("插入商品");
    product_id
}

#[tokio::test]
async fn list_returns_inserted_product_with_paging() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let code = common::unique("p");
    insert_product(&db, &code, "测试重疾险A", "ON_SALE", 1).await;

    // 分页命中：翻第一页，应包含刚插入的商品
    let items = list(&db, "ON_SALE", 1, 100).await.expect("列表查询成功");
    let found = items.iter().any(|p| p.product_code == code);
    assert!(found, "列表应包含新插入商品 {code}");

    // 分页参数下界保护：page=0 → 按第 1 页处理（不 panic）
    let _ = list(&db, "ON_SALE", 0, 100).await.expect("page=0 仍可查询");

    // size 上限保护：size=999 → clamp 到 100（不 panic）
    let _ = list(&db, "ON_SALE", 1, 999).await.expect("size 超限仍可查询");

    // 清理
    common::delete_product(&db, &code).await;
}

#[tokio::test]
async fn list_filters_by_status() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let code = common::unique("p");
    insert_product(&db, &code, "测试下架商品", "OFF_SHELF", 0).await;

    // ON_SALE 过滤：刚插入的是 OFF_SHELF，不应出现在 ON_SALE 列表
    let on_sale = list(&db, "ON_SALE", 1, 100).await.expect("查询成功");
    let found = on_sale.iter().any(|p| p.product_code == code);
    assert!(!found, "OFF_SHELF 商品不应出现在 ON_SALE 列表");

    // 显式按 OFF_SHELF 过滤：应命中
    let off_shelf = list(&db, "OFF_SHELF", 1, 100).await.expect("查询成功");
    let found = off_shelf.iter().any(|p| p.product_code == code);
    assert!(found, "OFF_SHELF 列表应包含 {code}");

    // 状态为空的调用（等效全部）：应命中
    let all = list(&db, "", 1, 100).await.expect("查询成功");
    let found = all.iter().any(|p| p.product_code == code);
    assert!(found, "全量列表应包含 {code}");

    common::delete_product(&db, &code).await;
}

#[tokio::test]
async fn list_excludes_soft_deleted() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let code = common::unique("p");
    let id = insert_product(&db, &code, "测试软删商品", "ON_SALE", 0).await;

    // 模拟软删除：置 deleted_at
    let mut conn = db.conn().await.expect("连接测试库");
    conn.exec_drop(
        "UPDATE insurance_products SET deleted_at = NOW() WHERE id = ?",
        vec![id],
    )
    .await
    .expect("软删除");

    let all = list(&db, "", 1, 100).await.expect("查询成功");
    let found = all.iter().any(|p| p.product_code == code);
    assert!(!found, "软删商品不应出现在列表");

    common::delete_product(&db, &code).await;
}

#[tokio::test]
async fn detail_returns_row() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let code = common::unique("p");
    let id = insert_product(&db, &code, "测试详情商品", "ON_SALE", 1).await;

    let item = detail(&db, id).await.expect("详情命中");
    assert_eq!(item.product_code, code);
    assert_eq!(item.status, "ON_SALE");
    assert!(item.is_featured, "应保存 is_featured=1");
    assert_eq!(item.min_amount.unwrap().to_string(), "100000.00", "应保存 min_amount");

    common::delete_product(&db, &code).await;
}

#[tokio::test]
async fn detail_missing_returns_not_found() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let err = detail(&db, i64::MAX).await.expect_err("不存在的 id 应失败");
    match err {
        insurance_service::error::AppError::NotFound => {}
        other => panic!("预期 NotFound，得到 {other:?}"),
    }
}
