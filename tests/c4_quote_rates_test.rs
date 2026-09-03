//! C4 报价费率表化：集成测试
//!
//! 覆盖：
//! 1) 命中费率行（产品+保障期+保额区间）→ 保费=保额×rate，覆盖请求 premium（费率行优先）；
//! 2) 无费率行产品 → 沿用请求 premium（保底回退，与 quote_service_test 断言值一致）；
//! 3) 费率行按产品隔离：他人产品的费率行不影响本产品报价；
//! 4) 档位维度不匹配（期限/保额区间外）→ 回退请求 premium。
//!
//! quote_rates 为 C4 新表（SQL 由 lead 合并进 install.sql），本文件自备
//! `CREATE TABLE IF NOT EXISTS` 保证测试库可直接运行、且多次运行幂等。
//! MySQL 不可用时 SKIP（common::test_db() 返回 None）。

mod common;

use insurance_service::db::Db;
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

/// 建 quote_rates 表（幂等，测试库直接运行；正式建表 SQL 由 lead 合并 install.sql）。
async fn ensure_rate_table(db: &Db) {
    let mut conn = db.conn().await.expect("连接测试库");
    conn.query_drop(
        "CREATE TABLE IF NOT EXISTS quote_rates (
           id          BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
           product_id  BIGINT UNSIGNED NOT NULL,
           term_months INT           NOT NULL,
           amount_min  DECIMAL(14,2) NOT NULL DEFAULT 0,
           amount_max  DECIMAL(14,2) NULL,
           rate        DECIMAL(10,6) NOT NULL,
           created_at  DATETIME(3)   NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
           KEY idx_qrate_product (product_id, term_months),
           CONSTRAINT fk_qrate_product FOREIGN KEY (product_id)
             REFERENCES insurance_products (id)
         ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci",
    )
    .await
    .expect("创建 quote_rates 表");
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
    let user_id = insert_user(db, username).await;
    let mut conn = db.conn().await.expect("连接测试库");
    let product_code = common::unique("qr");
    let product_id = insurance_service::utils::idgen::next_id();
    conn.exec_drop(
        "INSERT INTO insurance_products (id, product_code, name, product_type) VALUES (?, ?, ?, ?)",
        vec![
            Value::from(product_id),
            Value::from(&product_code),
            Value::from("费率测试产品"),
            Value::from("HEALTH"),
        ],
    )
    .await
    .expect("插入产品");
    Chain { username: username.to_string(), product_code, user_id, product_id }
}

/// 插入一条费率行：整期保费 = 保额 × rate。
async fn insert_rate(db: &Db, product_id: i64, term: i32, min: &str, max: Option<&str>, rate: &str) {
    let mut conn = db.conn().await.expect("连接测试库");
    conn.exec_drop(
        "INSERT INTO quote_rates (id, product_id, term_months, amount_min, amount_max, rate) \
         VALUES (?, ?, ?, ?, ?, ?)",
        vec![
            Value::from(insurance_service::utils::idgen::next_id()),
            Value::from(product_id),
            Value::from(term),
            Value::from(min),
            max.map(Value::from).unwrap_or(Value::NULL),
            Value::from(rate),
        ],
    )
    .await
    .expect("插入费率行");
}

/// 逆 FK 序清理一条链：费率行 → 产品 → 用户（报价由用例按单号先删）。
async fn cleanup_chain(db: &Db, c: &Chain) {
    let _ = db
        .exec_drop(
            "DELETE FROM quote_rates WHERE product_id = ?",
            vec![c.product_id],
        )
        .await;
    common::delete_product(db, &c.product_code).await;
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

/// 构造报价请求：保费/保额/期限可调，受益人留空（C4 用例不关心）。
fn quote_req(user_id: i64, product_id: i64, premium: &str, amount: &str, term: i32) -> CreateQuoteReq {
    CreateQuoteReq {
        product_id,
        user_id,
        holder_name: "测试投保人".to_string(),
        holder_id_card_enc: None,
        insured_name: "测试被保人".to_string(),
        insured_id_card_enc: None,
        insurance_amount: amount.parse().unwrap(),
        term_months: term,
        premium: premium.parse().unwrap(),
        premium_detail: None,
        effective_date: None,
        expire_date: None,
        health_declaration: None,
        risk_score: None,
        beneficiaries: Vec::<BeneficiaryReq>::new(),
    }
}

#[tokio::test]
async fn rate_row_hit_premium_follows_rate() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    ensure_rate_table(&db).await;
    let chain = insert_chain(&db, &common::unique("qr_hit")).await;
    // 12 个月、保额 0~不限：rate 0.05 → 100000×0.05 = 5000.00
    insert_rate(&db, chain.product_id, 12, "0.00", None, "0.050000").await;

    // 请求 premium 仅 100.00：费率行命中应覆盖为 5000.00（费率行优先）。
    let quote = svc(&db)
        .create(quote_req(chain.user_id, chain.product_id, "100.00", "100000.00", 12))
        .await
        .expect("创建报价成功");
    assert_eq!(
        quote.premium,
        "5000.00".parse::<Decimal>().unwrap(),
        "命中费率行应按 保额×rate 计算保费"
    );

    cleanup_quote(&db, &quote.quote_no).await;
    cleanup_chain(&db, &chain).await;
}

#[tokio::test]
async fn no_rate_row_falls_back_to_request_premium() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    ensure_rate_table(&db).await;
    let chain = insert_chain(&db, &common::unique("qr_fb")).await;
    // 该产品无任何费率行 → 回退沿用请求 premium（与 quote_service_test 的 5000.00 断言一致）。
    let quote = svc(&db)
        .create(quote_req(chain.user_id, chain.product_id, "5000.00", "100000.00", 12))
        .await
        .expect("创建报价成功");
    assert_eq!(
        quote.premium,
        "5000.00".parse::<Decimal>().unwrap(),
        "无费率行应回退沿用请求 premium"
    );

    cleanup_quote(&db, &quote.quote_no).await;
    cleanup_chain(&db, &chain).await;
}

#[tokio::test]
async fn rate_row_isolated_by_product() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    ensure_rate_table(&db).await;
    let chain_a = insert_chain(&db, &common::unique("qr_iso_a")).await;
    let chain_b = insert_chain(&db, &common::unique("qr_iso_b")).await;
    // 仅产品 A 有费率行（0.05）；对产品 B 报价不得受其影响。
    insert_rate(&db, chain_a.product_id, 12, "0.00", None, "0.050000").await;

    let quote_b = svc(&db)
        .create(quote_req(chain_b.user_id, chain_b.product_id, "4321.00", "100000.00", 12))
        .await
        .expect("创建报价成功");
    assert_eq!(
        quote_b.premium,
        "4321.00".parse::<Decimal>().unwrap(),
        "他人产品的费率行不应对本产品生效"
    );

    cleanup_quote(&db, &quote_b.quote_no).await;
    cleanup_chain(&db, &chain_a).await;
    cleanup_chain(&db, &chain_b).await;
}

#[tokio::test]
async fn band_dimension_mismatch_falls_back() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    ensure_rate_table(&db).await;
    let chain = insert_chain(&db, &common::unique("qr_band")).await;
    // 费率行仅覆盖：12 个月、保额 ≤200000。
    insert_rate(&db, chain.product_id, 12, "0.00", Some("200000.00"), "0.050000").await;

    // 期限 24 个月 → 维度不匹配，回退请求 premium 888.00。
    let q1 = svc(&db)
        .create(quote_req(chain.user_id, chain.product_id, "888.00", "100000.00", 24))
        .await
        .expect("创建报价成功");
    assert_eq!(q1.premium, "888.00".parse::<Decimal>().unwrap(), "期限不匹配应回退");

    // 保额 500000 超出区间上限 → 维度不匹配，回退请求 premium 777.00。
    let q2 = svc(&db)
        .create(quote_req(chain.user_id, chain.product_id, "777.00", "500000.00", 12))
        .await
        .expect("创建报价成功");
    assert_eq!(q2.premium, "777.00".parse::<Decimal>().unwrap(), "保额超区间应回退");

    cleanup_quote(&db, &q1.quote_no).await;
    cleanup_quote(&db, &q2.quote_no).await;
    cleanup_chain(&db, &chain).await;
}
