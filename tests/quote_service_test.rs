//! 报价服务集成测试（MySQL 集成）
//!
//! 覆盖：create 带非空 effective_date/expire_date 成功且 roundtrip 一致（DATE 解码 bug 回归）→
//! by_id 命中返回同字段 → by_id 不存在的 id → NotFound；受益人随 create 落库。
//!
//! 依赖 `insurance_service` 库 + 本地 MySQL（install.sql 建库）。
//! MySQL 不可用时 SKIP（打印提示并提前返回），保证 `cargo test` 在无库环境不失败。
//!
//! quotes 外键只依赖 users / insurance_products，故用例仅需插入用户→产品（FK 逆向清理）。

mod common;

use chrono::NaiveDate;
use insurance_service::db::Db;
use insurance_service::error::AppError;
use insurance_service::services::quote_service::{
    BeneficiaryReq, CreateQuoteReq, QuoteService,
};
use mysql_async::prelude::Queryable;
use mysql_async::Value;
use rust_decimal::Decimal;

/// 一条最小数据链（用户 + 产品），供报价 create 使用。
struct Chain {
    username: String,
    product_code: String,
    user_id: i64,
    product_id: i64,
}

/// 插入一个测试用户，返回预生成 id。
async fn insert_user(db: &Db, username: &str) -> i64 {
    let mut conn = db.conn().await.expect("连接测试库");
    let user_id = insurance_service::utils::idgen::next_id();
    conn.exec_drop(
        "INSERT INTO users (id, username, password_hash) VALUES (?, ?, ?)",
        vec![
            Value::from(user_id),
            Value::from(username),
            Value::from("test-hash"),
        ],
    )
    .await
    .expect("插入用户");
    user_id
}

/// 按 FK 顺序插入 用户 → 产品，生成最小数据链。
async fn insert_chain(db: &Db, username: &str) -> Chain {
    let mut conn = db.conn().await.expect("连接测试库");
    let user_id = insert_user(db, username).await;

    let product_code = common::unique("qp");
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

    Chain {
        username: username.to_string(),
        product_code,
        user_id,
        product_id,
    }
}

/// 逆 FK 序清理一条链（报价需先由用例按 quote_no 单独删除，受益人级联删除）。
async fn cleanup_chain(db: &Db, c: &Chain) {
    let _ = db
        .exec_drop(
            "DELETE FROM insurance_products WHERE product_code = ?",
            vec![c.product_code.as_str()],
        )
        .await;
    common::delete_user(db, &c.username).await;
}

/// 删除本次测试创建的报价（按唯一单号，受益人 ON DELETE CASCADE 一并清理）。
async fn cleanup_quote(db: &Db, quote_no: &str) {
    let _ = db
        .exec_drop("DELETE FROM quotes WHERE quote_no = ?", vec![quote_no])
        .await;
}

fn svc(db: &Db) -> QuoteService {
    QuoteService::new(db.clone())
}

/// 构造报价请求：带非空生效/失效日、JSON 明细与一名指定受益人。
fn quote_req(user_id: i64, product_id: i64) -> CreateQuoteReq {
    CreateQuoteReq {
        product_id,
        user_id,
        holder_name: "测试投保人".to_string(),
        holder_id_card_enc: None,
        insured_name: "测试被保人".to_string(),
        insured_id_card_enc: None,
        insurance_amount: "100000.00".parse().unwrap(),
        term_months: 12,
        premium: "5000.00".parse().unwrap(),
        premium_detail: Some(serde_json::json!({"base": "4800.00", "extra": "200.00"})),
        effective_date: Some(NaiveDate::from_ymd_opt(2026, 9, 1).unwrap()),
        expire_date: Some(NaiveDate::from_ymd_opt(2036, 9, 1).unwrap()),
        health_declaration: None,
        risk_score: Some(60),
        beneficiaries: vec![BeneficiaryReq {
            name: "受益人甲".to_string(),
            id_card_enc: None,
            relationship: Some("SPOUSE".to_string()),
            beneficiary_type: "NAMED".to_string(),
            share_percent: Some("100.00".parse().unwrap()),
            sort_order: 1,
        }],
    }
}

#[tokio::test]
async fn create_with_dates_roundtrips_and_persists_beneficiary() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let chain = insert_chain(&db, &common::unique("q_owner")).await;

    // 回归：非空 effective_date/expire_date 回读不再 panic，且 roundtrip 一致。
    let quote = svc(&db)
        .create(quote_req(chain.user_id, chain.product_id))
        .await
        .expect("创建报价成功");
    assert!(quote.quote_no.starts_with("QT"), "单号应以 QT 开头: {}", quote.quote_no);
    assert_eq!(quote.status, "PENDING", "创建后状态应为 PENDING");
    assert_eq!(quote.product_id, chain.product_id);
    assert_eq!(quote.user_id, chain.user_id);
    assert_eq!(quote.insurance_amount, "100000.00".parse::<Decimal>().unwrap());
    assert_eq!(quote.premium, "5000.00".parse::<Decimal>().unwrap());
    assert_eq!(quote.effective_date, Some(NaiveDate::from_ymd_opt(2026, 9, 1).unwrap()));
    assert_eq!(quote.expire_date, Some(NaiveDate::from_ymd_opt(2036, 9, 1).unwrap()));
    assert_eq!(quote.risk_score, Some(60));
    assert_eq!(
        quote.premium_detail,
        Some(serde_json::json!({"base": "4800.00", "extra": "200.00"}))
    );

    // 受益人随 create 落库
    let mut conn = db.conn().await.expect("连接测试库");
    let n: Option<i64> = conn
        .exec_first(
            "SELECT COUNT(*) FROM quotes_beneficiaries WHERE quote_id = ?",
            vec![quote.id],
        )
        .await
        .expect("查询受益人");
    assert_eq!(n.unwrap_or(0), 1, "应落库 1 名受益人");

    cleanup_quote(&db, &quote.quote_no).await;
    cleanup_chain(&db, &chain).await;
}

#[tokio::test]
async fn by_id_returns_matching_quote() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let chain = insert_chain(&db, &common::unique("q_detail")).await;
    let created = svc(&db)
        .create(quote_req(chain.user_id, chain.product_id))
        .await
        .expect("创建报价成功");

    let found = svc(&db).by_id(created.id).await.expect("按 id 查详情成功");
    assert_eq!(found.id, created.id);
    assert_eq!(found.quote_no, created.quote_no);
    assert_eq!(found.product_id, chain.product_id);
    assert_eq!(found.user_id, chain.user_id);
    assert_eq!(found.insurance_amount, created.insurance_amount);
    assert_eq!(found.premium, created.premium);
    assert_eq!(found.effective_date, created.effective_date);
    assert_eq!(found.expire_date, created.expire_date);
    assert_eq!(found.status, "PENDING");

    cleanup_quote(&db, &created.quote_no).await;
    cleanup_chain(&db, &chain).await;
}

#[tokio::test]
async fn by_id_missing_returns_not_found() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    // 不存在的 id（超大必不命中），无需真实数据。
    let err = svc(&db).by_id(i64::MAX).await.expect_err("不存在的报价应报错");
    match err {
        AppError::NotFound => {}
        other => panic!("预期 NotFound，得到 {other:?}"),
    }
}
