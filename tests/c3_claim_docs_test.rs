//! 理赔资料上传与列表集成测试（任务 C3，MySQL 集成）
//!
//! 覆盖：报案人上传成功且落库、列表可见（created_at 降序）；他人（非报案人
//! 非 ADMIN）上传/查看被拒（Forbidden）；ADMIN 代传放行；类型/文件名/file_key
//! 为空或超长（doc_type>32、file_name/file_key>255）→ 业务错误。
//!
//! 依赖 `insurance_service` 库 + 本地 MySQL（install.sql 建库 +
//! claim_documents 表，见任务 C3 建表 SQL）。MySQL 不可用时 SKIP，
//! 保证 `cargo test` 在无库环境不失败。

mod common;

use chrono::NaiveDate;
use insurance_service::db::Db;
use insurance_service::error::AppError;
use insurance_service::services::claim_service::{
    ClaimService, CreateClaimReq, UploadDocumentReq,
};
use mysql_async::prelude::Queryable;
use mysql_async::Value;
use rust_decimal::Decimal;

/// 一条最小保单链 + 报案后的理赔单，供资料上传用。
struct Chain {
    username: String,
    product_code: String,
    quote_no: String,
    order_no: String,
    policy_no: String,
    user_id: i64,
    claim_id: i64,
    claim_no: String,
}

/// 插入指定角色测试用户，返回自增 id。
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

/// 按 FK 顺序插入最小保单链并报案，返回理赔上下文。
async fn insert_chain(db: &Db, username: &str) -> Chain {
    let mut conn = db.conn().await.expect("连接测试库");
    let user_id = insert_user(db, username, "USER").await;

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

    let claim = ClaimService::new(db.clone())
        .create(CreateClaimReq {
            policy_id,
            user_id,
            accident_date: Some(NaiveDate::from_ymd_opt(2026, 8, 1).unwrap()),
            accident_type: Some("TRAFFIC".to_string()),
            accident_desc: Some("测试事故描述".to_string()),
            claim_amount: "1000.00".parse().unwrap(),
        })
        .await
        .expect("报案成功");

    Chain {
        username: username.to_string(),
        product_code,
        quote_no,
        order_no,
        policy_no,
        user_id,
        claim_id: claim.id,
        claim_no: claim.claim_no,
    }
}

/// 逆 FK 序清理链：先按 claim_no 删资料与理赔，再删保单/订单/报价/产品/用户。
async fn cleanup_chain(db: &Db, c: &Chain) {
    let _ = db
        .exec_drop(
            "DELETE FROM claim_documents WHERE claim_id IN \
             (SELECT id FROM claims WHERE claim_no = ?)",
            vec![c.claim_no.as_str()],
        )
        .await;
    let _ = db
        .exec_drop("DELETE FROM claims WHERE claim_no = ?", vec![c.claim_no.as_str()])
        .await;
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
    let _ = db
        .exec_drop("DELETE FROM users WHERE username = ?", vec![c.username.as_str()])
        .await;
}

fn svc(db: &Db) -> ClaimService {
    ClaimService::new(db.clone())
}

/// 构造上传请求（缺省报案人本人）
fn upload_req(user_id: i64, doc_type: &str, file_name: &str, file_key: &str) -> UploadDocumentReq {
    UploadDocumentReq {
        user_id,
        doc_type: doc_type.to_string(),
        file_name: file_name.to_string(),
        file_key: file_key.to_string(),
    }
}

/// 断言业务错误消息包含指定片段。
fn expect_business(err: AppError, needle: &str) {
    match err {
        AppError::Business(m) => assert!(m.contains(needle), "业务错误消息应含 {needle:?}: {m}"),
        other => panic!("预期 Business({needle:?})，得到 {other:?}"),
    }
}

#[tokio::test]
async fn owner_upload_succeeds_and_listed() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql + claim_documents 建表）");
        return;
    };
    let chain = insert_chain(&db, &common::unique("doc_owner")).await;

    // 报案人本人上传两张资料
    let d1 = svc(&db)
        .add_document(
            chain.claim_id,
            upload_req(chain.user_id, "病历", "出院小结.pdf", "mock://claims/1/a.pdf"),
        )
        .await
        .expect("报案人上传成功");
    assert!(d1.id > 0);
    assert_eq!(d1.claim_id, chain.claim_id);
    assert_eq!(d1.doc_type, "病历");
    assert_eq!(d1.file_name, "出院小结.pdf");
    assert_eq!(d1.file_key, "mock://claims/1/a.pdf");

    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
    let d2 = svc(&db)
        .add_document(
            chain.claim_id,
            upload_req(chain.user_id, "发票", "发票.jpg", "mock://claims/1/b.jpg"),
        )
        .await
        .expect("第二张上传成功");
    assert_ne!(d1.id, d2.id);

    // 列表可见且新的在前
    let list = svc(&db)
        .list_documents(chain.claim_id, chain.user_id)
        .await
        .expect("列表成功");
    assert!(list.len() >= 2, "应至少 2 条，实际 {}", list.len());
    assert_eq!(list[0].file_name, "发票.jpg", "created_at 降序，新上传应在前");
    let names: Vec<&str> = list.iter().map(|d| d.file_name.as_str()).collect();
    assert!(names.contains(&"出院小结.pdf") && names.contains(&"发票.jpg"));

    cleanup_chain(&db, &chain).await;
}

#[tokio::test]
async fn other_user_upload_and_list_forbidden() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql + claim_documents 建表）");
        return;
    };
    let chain = insert_chain(&db, &common::unique("doc_owner_b")).await;
    // 他人（普通 USER，非报案人）
    let intruder = insert_user(&db, &common::unique("doc_intruder"), "USER").await;

    let err = svc(&db)
        .add_document(
            chain.claim_id,
            upload_req(intruder, "发票", "发票.jpg", "mock://claims/1/b.jpg"),
        )
        .await
        .expect_err("他人上传应被拒");
    match err {
        AppError::Forbidden => {}
        other => panic!("预期 Forbidden，得到 {other:?}"),
    }

    let err = svc(&db)
        .list_documents(chain.claim_id, intruder)
        .await
        .expect_err("他人查看列表应被拒");
    match err {
        AppError::Forbidden => {}
        other => panic!("预期 Forbidden，得到 {other:?}"),
    }

    let _ = db
        .exec_drop("DELETE FROM users WHERE id = ?", vec![intruder])
        .await;
    cleanup_chain(&db, &chain).await;
}

#[tokio::test]
async fn admin_can_upload_for_others_claim() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql + claim_documents 建表）");
        return;
    };
    let chain = insert_chain(&db, &common::unique("doc_owner_c")).await;
    let admin = insert_user(&db, &common::unique("doc_admin"), "ADMIN").await;

    // ADMIN 可代报案人上传
    let doc = svc(&db)
        .add_document(
            chain.claim_id,
            upload_req(admin, "申报单", "申请书.pdf", "mock://claims/1/c.pdf"),
        )
        .await
        .expect("ADMIN 代传成功");
    assert_eq!(doc.doc_type, "申报单");

    // ADMIN 亦可查看列表
    let list = svc(&db)
        .list_documents(chain.claim_id, admin)
        .await
        .expect("ADMIN 查看列表成功");
    assert_eq!(list.len(), 1);

    cleanup_chain(&db, &chain).await;
}

#[tokio::test]
async fn upload_validation_rejects_blank_and_oversize() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql + claim_documents 建表）");
        return;
    };
    let chain = insert_chain(&db, &common::unique("doc_valid")).await;
    let svc = svc(&db);

    // 类型/文件名/file_key 为空 → 业务错误
    let err = svc
        .add_document(
            chain.claim_id,
            upload_req(chain.user_id, "  ", "发票.jpg", "mock://k"),
        )
        .await
        .expect_err("空类型应失败");
    expect_business(err, "资料类型必填");

    let err = svc
        .add_document(
            chain.claim_id,
            upload_req(chain.user_id, "发票", "  ", "mock://k"),
        )
        .await
        .expect_err("空文件名应失败");
    expect_business(err, "文件名为空");

    let err = svc
        .add_document(
            chain.claim_id,
            upload_req(chain.user_id, "发票", "发票.jpg", "  "),
        )
        .await
        .expect_err("空 file_key 应失败");
    expect_business(err, "file_key 必填");

    // 超长（类型 >32；文件名/file_key >255）→ 业务错误
    let err = svc
        .add_document(
            chain.claim_id,
            upload_req(chain.user_id, &"超".repeat(33), "发票.jpg", "mock://k"),
        )
        .await
        .expect_err("超长类型应失败");
    expect_business(err, "最长 32");

    let err = svc
        .add_document(
            chain.claim_id,
            upload_req(chain.user_id, "发票", &"f".repeat(256), "mock://k"),
        )
        .await
        .expect_err("超长文件名应失败");
    expect_business(err, "最长 255");

    cleanup_chain(&db, &chain).await;
}

#[tokio::test]
async fn upload_rejects_missing_claim() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql + claim_documents 建表）");
        return;
    };
    // 不存在的理赔（超大 id 必不命中）→ 业务错误（校验先于 INSERT，无需真实用户）
    let err = svc(&db)
        .add_document(i64::MAX, upload_req(1, "发票", "发票.jpg", "mock://k"))
        .await
        .expect_err("理赔不存在应失败");
    expect_business(err, "理赔单不存在");
}
