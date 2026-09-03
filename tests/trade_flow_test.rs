//! 交易闭环集成测试（任务 A1：报价 → 下单 → 支付回调 → 订单 PAID → 保单签发）
//!
//! 覆盖：1) 完整闭环 + 金额一致（优惠 300.50 → 应付 4699.50，payment.amount == payable）；
//! 2) provider_tx_id 幂等（同 tx 二次回调不重复处理）；3) 状态机（PAID 后拒绝再支付/重复
//! 签发，再回调幂等返回）；4) 归属/存在性守卫 + FAILED 支付须重新预支付。
//!
//! 按实测语义断言（非臆想）：payment callback 只把 payment 置 SUCCESS/FAILED，**不**更新
//! orders、**不**签发保单；policy issue 要求订单 == PAID（否则「订单未支付，无法签发保单」），
//! PAID 签发 → 保单 ACTIVE、订单 POLICY_ISSUED。服务层无 CREATED→PAID 迁移、orders.paid_at
//! 无写入点（缺陷详见报告），用例以 SQL `UPDATE orders SET status='PAID'` 编排前置态
//! （测试编排而非产品 API）。
//!
//! 依赖 `insurance_service` 库 + 本地 MySQL（install.sql 建库）；MySQL 不可用 = **FAIL
//! （panic）**，禁止 SKIP 假绿。数据唯一化并逆 FK 序清理（policies → orders → quotes →
//! 产品/用户，payments 随 order 级联），防并行 agent 污染。

mod common;

use insurance_service::db::Db;
use insurance_service::error::AppError;
use insurance_service::models::order::Order;
use insurance_service::models::payment::Payment;
use insurance_service::models::policy::Policy;
use insurance_service::models::quote::Quote;
use insurance_service::services::order_service::{CreateOrderReq, OrderService};
use insurance_service::services::payment_service::{CallbackReq, CreatePaymentReq, PaymentService};
use insurance_service::services::policy_service::{IssuePolicyReq, PolicyService};
use insurance_service::services::quote_service::{CreateQuoteReq, QuoteService};
use mysql_async::prelude::Queryable;
use mysql_async::Value;
use rust_decimal::Decimal;

/// 最小数据链（用户 + 产品），其余行全部走真实服务创建。
struct Chain {
    username: String,
    product_code: String,
    user_id: i64,
    product_id: i64,
}

/// DB 不可用 = FAIL（任务要求：禁止 SKIP 假绿）。
async fn db_or_panic() -> Db {
    common::test_db().await.expect(
        "测试数据库不可用：请设置 DATABASE_URL 指向已执行 install.sql 的 MySQL（本测试禁止 SKIP）",
    )
}

/// 断言错误为业务错误且消息包含指定片段（Business → HTTP 40001）。
fn expect_business(err: AppError, needle: &str) {
    match err {
        AppError::Business(m) => assert!(m.contains(needle), "业务错误消息应含 {needle:?}: {m}"),
        other => panic!("预期 Business({needle:?})，得到 {other:?}"),
    }
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

/// 建链：唯一用户名 + 唯一产品（清理按用户名/产品码精确命中本链行）。
async fn setup_chain(db: &Db, prefix: &str) -> Chain {
    let username = common::unique(prefix);
    let user_id = insert_user(db, &username).await;

    let product_code = common::unique("tp");
    let mut conn = db.conn().await.expect("连接测试库");
    conn.exec_drop(
        "INSERT INTO insurance_products (product_code, name, product_type) VALUES (?, ?, ?)",
        vec![Value::from(&product_code), Value::from("测试产品"), Value::from("HEALTH")],
    )
    .await
    .expect("插入产品");
    let product_id = conn.last_insert_id().expect("取得自增 id") as i64;

    Chain { username, product_code, user_id, product_id }
}

/// 构造报价请求：保费 5000.00 / 保额 100000.00 / 12 个月。effective_date/expire_date 留
/// None、受益人为空（本测试聚焦状态机/金额/幂等；日期解码与受益人落库由 quote_service_test 覆盖）。
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
        premium_detail: Some(serde_json::json!({"base": "5000.00"})),
        effective_date: None,
        expire_date: None,
        health_declaration: None,
        risk_score: Some(60),
        beneficiaries: vec![],
    }
}

fn qs(db: &Db) -> QuoteService { QuoteService::new(db.clone()) }
fn os(db: &Db) -> OrderService { OrderService::new(db.clone()) }
fn ps(db: &Db) -> PaymentService { PaymentService::new(db.clone()) }
fn pols(db: &Db) -> PolicyService { PolicyService::new(db.clone()) }

/// 构造签发请求（IssuePolicyReq 无 Clone，重复断言需重建）。
fn issue_req(order_id: i64, quote_id: i64, user_id: i64) -> IssuePolicyReq {
    IssuePolicyReq { order_id, quote_id, user_id, issue_type: String::new(), is_renewable: false }
}

/// 走真实 QuoteService 创建 PENDING 报价。
async fn make_quote(db: &Db, c: &Chain) -> Quote {
    qs(db).create(quote_req(c.user_id, c.product_id)).await.expect("创建报价成功")
}

/// 走真实 OrderService 下单（可带优惠），返回 CREATED 订单。
async fn make_order(db: &Db, c: &Chain, q: &Quote, discount: Option<Decimal>) -> Order {
    os(db)
        .create(CreateOrderReq {
            quote_id: q.id,
            user_id: c.user_id,
            remark: Some("交易闭环测试".to_string()),
            discount_amount: discount,
        })
        .await
        .expect("创建订单成功")
}

/// 走真实 PaymentService 预支付（空 channel = 默认 MOCK）。
async fn make_payment(db: &Db, order: &Order, user_id: i64) -> Payment {
    ps(db)
        .prepay(CreatePaymentReq { order_id: order.id, user_id, channel: String::new() })
        .await
        .expect("预支付成功")
}

/// 统计订单的支付流水条数（幂等断言用）。
async fn count_payments(db: &Db, order_id: i64) -> i64 {
    let mut conn = db.conn().await.expect("连接测试库");
    let n: Option<i64> = conn
        .exec_first("SELECT COUNT(*) FROM payments WHERE order_id = ?", vec![order_id])
        .await
        .expect("统计支付流水");
    n.unwrap_or(0)
}

/// 统计订单的保单条数（重复签发拦截断言用）。
async fn count_policies(db: &Db, order_id: i64) -> i64 {
    let mut conn = db.conn().await.expect("连接测试库");
    let n: Option<i64> = conn
        .exec_first("SELECT COUNT(*) FROM policies WHERE order_id = ?", vec![order_id])
        .await
        .expect("统计保单");
    n.unwrap_or(0)
}

/// SQL 编排渠道到账前置态（服务层无 CREATED→PAID 迁移，实测缺陷见文件头），回读 PAID 订单。
async fn mark_order_paid(db: &Db, order: &Order) -> Order {
    db.exec_drop(
        "UPDATE orders SET status = 'PAID' WHERE id = ? AND deleted_at IS NULL",
        vec![order.id],
    )
    .await
    .expect("编排订单 PAID");
    os(db).by_id(order.id).await.expect("回读订单")
}

/// 逆 FK 序清理：policies → orders（payments 级联）→ quotes → 产品 → 用户。
async fn cleanup_all(db: &Db, c: &Chain, q: &Quote, o: &Order, policy_no: Option<&str>) {
    if let Some(p) = policy_no {
        let _ = db.exec_drop("DELETE FROM policies WHERE policy_no = ?", vec![p]).await;
    }
    let _ = db
        .exec_drop("DELETE FROM orders WHERE order_no = ?", vec![o.order_no.as_str()])
        .await;
    let _ = db
        .exec_drop("DELETE FROM quotes WHERE quote_no = ?", vec![q.quote_no.as_str()])
        .await;
    let _ = db
        .exec_drop(
            "DELETE FROM insurance_products WHERE product_code = ?",
            vec![c.product_code.as_str()],
        )
        .await;
    common::delete_user(db, &c.username).await;
}

/// 断言签发成功：POL 前缀、ACTIVE、关联订单正确、保费 == 订单实付。
fn assert_policy_ok(pol: &Policy, order: &Order) {
    assert!(pol.policy_no.starts_with("POL"), "保单号应以 POL 开头: {}", pol.policy_no);
    assert_eq!(pol.status, "ACTIVE", "签发后保单应为 ACTIVE");
    assert_eq!(pol.order_id, order.id);
    assert_eq!(pol.premium, order.payable_amount, "保单保费 == 订单实付");
}

// 测试项 1(+4)：完整闭环 —— 报价→下单→预支付→回调成功→（SQL 编排 PAID）→保单签发
#[tokio::test]
async fn full_closure_callback_then_issue_policy_amounts_match() {
    let db = db_or_panic().await;
    let c = setup_chain(&db, "tf_closure").await;
    let q = make_quote(&db, &c).await;
    let o = make_order(&db, &c, &q, Some("300.50".parse().unwrap())).await;

    // 金额口径：total=5000.00 − discount=300.50 → payable=4699.50（saturating_sub）
    assert_eq!(o.status, "CREATED", "下单后订单应为 CREATED");
    assert_eq!(o.total_amount, "5000.00".parse::<Decimal>().unwrap());
    assert_eq!(o.discount_amount, "300.50".parse::<Decimal>().unwrap());
    assert_eq!(o.payable_amount, "4699.50".parse::<Decimal>().unwrap());
    assert_eq!(o.currency, "CNY");

    // 未支付不可签发：CREATED 订单被状态机拒绝（业务错 40001）
    let err = pols(&db)
        .issue(issue_req(o.id, q.id, c.user_id))
        .await
        .expect_err("CREATED 订单签发保单应失败");
    expect_business(err, "订单未支付");

    // 预支付 → 回调成功
    let p = make_payment(&db, &o, c.user_id).await;
    assert_eq!(p.status, "CREATED", "预支付后支付单为 CREATED");
    // 测试项 4：预支付金额 == 订单实付金额（4699.50），币种随订单
    assert_eq!(p.amount, o.payable_amount, "payment.amount 必须等于 order.payable_amount");
    assert_eq!(p.currency, o.currency, "币种应随订单");
    assert_eq!(p.amount, "4699.50".parse::<Decimal>().unwrap());

    let tx_id = common::unique("TX");
    let p1 = ps(&db)
        .callback(CallbackReq {
            payment_id: p.id,
            provider_tx_id: Some(tx_id.clone()),
            success: Some(true),
            payload: Some(serde_json::json!({"out_trade_no": "wx-1"})),
        })
        .await
        .expect("回调成功应返回支付单");
    assert_eq!(p1.status, "SUCCESS", "回调 success=true 后支付单应为 SUCCESS");
    assert_eq!(p1.provider_tx_id.as_deref(), Some(tx_id.as_str()));
    assert!(p1.paid_at.is_some(), "成功后应落 paid_at");

    // 实测语义：回调只动 payments —— 订单仍 CREATED、无自动签发（两条缺陷佐证断言）
    let order_after = os(&db).by_id(o.id).await.expect("回读订单");
    assert_eq!(order_after.status, "CREATED", "缺陷佐证：回调不把订单推进 PAID（服务层无此迁移）");
    assert_eq!(count_policies(&db, o.id).await, 0, "缺陷佐证：回调链路不自动签发保单");

    // 编排渠道确认后走真实 PolicyService 签发。v1.1.0 的 issue() 两个阻断缺陷（报价 DATE 按
    // Option<String> 解码崩溃 / pid 在 UPDATE 后取致归零回读失败）已被工作树修复，此处锚定成功路径。
    let paid = mark_order_paid(&db, &order_after).await;
    assert_eq!(paid.status, "PAID");
    assert!(paid.paid_at.is_none(), "缺陷佐证：orders.paid_at 无写入点，恒为 NULL");
    let pol = pols(&db)
        .issue(issue_req(paid.id, q.id, c.user_id))
        .await
        .expect("PAID 订单签发保单成功");
    assert_policy_ok(&pol, &o);
    assert_eq!(pol.premium, "4699.50".parse::<Decimal>().unwrap(), "保单保费 == 应付 4699.50");

    // 订单进 POLICY_ISSUED 终态（终态的重复签发/再支付守卫由测试项 3 覆盖）
    let issued = os(&db).by_id(o.id).await.expect("回读订单");
    assert_eq!(issued.status, "POLICY_ISSUED", "签发后订单应进 POLICY_ISSUED");

    cleanup_all(&db, &c, &q, &o, Some(&pol.policy_no)).await;
}

// 测试项 2：支付回调幂等 —— 同一 provider_tx_id 回调两次不重复处理、不报错
#[tokio::test]
async fn same_provider_tx_callback_is_idempotent_and_no_second_payment() {
    let db = db_or_panic().await;
    let c = setup_chain(&db, "tf_idem").await;
    let q = make_quote(&db, &c).await;
    let o = make_order(&db, &c, &q, None).await;

    // 无优惠：payable == 5000.00（金额一致性基线）
    assert_eq!(o.payable_amount, "5000.00".parse::<Decimal>().unwrap());
    let p = make_payment(&db, &o, c.user_id).await;
    assert_eq!(p.amount, o.payable_amount);

    let tx = common::unique("TXID");
    let ok1 = ps(&db)
        .callback(CallbackReq {
            payment_id: p.id,
            provider_tx_id: Some(tx.clone()),
            success: Some(true),
            payload: None,
        })
        .await
        .expect("首次回调成功");
    assert_eq!(ok1.status, "SUCCESS");

    // 同 tx 二次回调：命中幂等守卫 → 提前返回当前状态，不报错、不重复入账
    let ok2 = ps(&db)
        .callback(CallbackReq {
            payment_id: p.id,
            provider_tx_id: Some(tx.clone()),
            success: Some(true),
            payload: None,
        })
        .await
        .expect("重复回调不应报错（幂等返回）");
    assert_eq!(ok2.status, "SUCCESS", "重复回调应返回当前状态 SUCCESS");
    assert_eq!(ok2.provider_tx_id.as_deref(), Some(tx.as_str()));
    assert_eq!(count_payments(&db, o.id).await, 1, "同一 provider_tx_id 只允许一条支付流水");

    // 换 tx 的迟到失败回调：UPDATE WHERE status IN (CREATED,PROCESSING) 0 行 → 静默返回现状
    // —— SUCCESS 终态不可被失败回调翻转、provider_tx_id 不可被改写、流水数不变
    let late = ps(&db)
        .callback(CallbackReq {
            payment_id: p.id,
            provider_tx_id: Some(common::unique("TX2")),
            success: Some(false),
            payload: None,
        })
        .await
        .expect("终态后的迟到回调不报错（静默忽略）");
    assert_eq!(late.status, "SUCCESS", "SUCCESS 终态不可被失败回调翻转");
    assert_eq!(late.provider_tx_id.as_deref(), Some(tx.as_str()), "provider_tx_id 不可被覆盖");
    assert_eq!(count_payments(&db, o.id).await, 1);

    cleanup_all(&db, &c, &q, &o, None).await;
}

// 测试项 3：状态机 —— 已支付订单再支付/重复签发被拒；再回调幂等返回
#[tokio::test]
async fn paid_order_blocks_repay_and_duplicate_issue() {
    let db = db_or_panic().await;
    let c = setup_chain(&db, "tf_sm").await;
    let q = make_quote(&db, &c).await;
    let o = make_order(&db, &c, &q, None).await;
    let p = make_payment(&db, &o, c.user_id).await;

    let tx = common::unique("TXS");
    let paid_pay = ps(&db)
        .callback(CallbackReq {
            payment_id: p.id,
            provider_tx_id: Some(tx.clone()),
            success: Some(true),
            payload: None,
        })
        .await
        .expect("回调成功");
    assert_eq!(paid_pay.status, "SUCCESS");

    // 编排订单进入 PAID（服务层无此迁移，见文件头说明）
    let paid = mark_order_paid(&db, &o).await;
    assert_eq!(paid.status, "PAID");

    // 已支付订单再预支付 → 业务拒绝（prepay 只接受 CREATED/EXPIRED）
    let err = ps(&db)
        .prepay(CreatePaymentReq { order_id: o.id, user_id: c.user_id, channel: String::new() })
        .await
        .expect_err("PAID 订单再预支付应被拒");
    expect_business(err, "订单状态不可支付");

    // 已支付订单再回调：不因终态报错（幂等返回当前 SUCCESS，与 payment_service 语义一致）
    let again = ps(&db)
        .callback(CallbackReq {
            payment_id: p.id,
            provider_tx_id: Some(tx.clone()),
            success: Some(true),
            payload: None,
        })
        .await
        .expect("PAID 后重复回调仍幂等返回");
    assert_eq!(again.status, "SUCCESS");
    assert_eq!(count_payments(&db, o.id).await, 1, "不允许二次入账");

    // 已支付订单签发成功（v1.1.0 的 issue() 阻断缺陷已被工作树修复，见测试项 1 说明）
    let pol = pols(&db)
        .issue(issue_req(paid.id, q.id, c.user_id))
        .await
        .expect("PAID 订单签发保单成功");
    assert_policy_ok(&pol, &o);

    // POLICY_ISSUED 终态：重复签发被拒（订单已非 PAID → 同一「订单未支付」守卫）
    let issued = os(&db).by_id(o.id).await.expect("回读订单");
    assert_eq!(issued.status, "POLICY_ISSUED", "签发后订单应进 POLICY_ISSUED");
    assert!(issued.paid_at.is_none(), "缺陷佐证：paid_at 无写入点，终态订单仍为 NULL");
    let dup = pols(&db)
        .issue(issue_req(o.id, q.id, c.user_id))
        .await
        .expect_err("POLICY_ISSUED 后重复签发应被拒");
    expect_business(dup, "订单未支付");
    assert_eq!(count_policies(&db, o.id).await, 1, "仅应落库 1 张保单");

    cleanup_all(&db, &c, &q, &o, Some(&pol.policy_no)).await;
}

// 归属/存在性守卫 + 失败支付语义（重试须重新预支付，终态不可原地翻转）
#[tokio::test]
async fn ownership_and_existence_guards_failed_payment_needs_new_prepay() {
    let db = db_or_panic().await;
    let c = setup_chain(&db, "tf_guard").await;
    let q = make_quote(&db, &c).await;

    // 他人下单：quote 不属于该用户 → 拒
    let other_user = insert_user(&db, &common::unique("tf_other")).await;
    let err = os(&db)
        .create(CreateOrderReq {
            quote_id: q.id,
            user_id: other_user,
            remark: None,
            discount_amount: None,
        })
        .await
        .expect_err("他人报价下单应被拒");
    expect_business(err, "报价不存在或已失效");

    // 本人正常下单后，他人对其订单预支付 → 拒（订单必须属于本人）
    let o = make_order(&db, &c, &q, None).await;
    let err2 = ps(&db)
        .prepay(CreatePaymentReq { order_id: o.id, user_id: other_user, channel: String::new() })
        .await
        .expect_err("他人订单预支付应被拒");
    expect_business(err2, "订单不存在");

    // 回调不存在的支付单 → 拒
    let err3 = ps(&db)
        .callback(CallbackReq {
            payment_id: i64::MAX,
            provider_tx_id: Some(common::unique("TXG")),
            success: Some(true),
            payload: None,
        })
        .await
        .expect_err("回调不存在的支付单应被拒");
    expect_business(err3, "支付单不存在");

    // 失败回调：支付单 → FAILED（订单仍是 CREATED，不推进）
    let p = make_payment(&db, &o, c.user_id).await;
    let fail = ps(&db)
        .callback(CallbackReq {
            payment_id: p.id,
            provider_tx_id: Some(common::unique("TXF")),
            success: Some(false),
            payload: None,
        })
        .await
        .expect("失败回调成功落 FAILED");
    assert_eq!(fail.status, "FAILED");
    let order_still = os(&db).by_id(o.id).await.expect("回读订单");
    assert_eq!(order_still.status, "CREATED", "失败回调不影响订单状态");

    // FAILED 支付单不可被后续成功回调原地翻转（WHERE 仅 CREATED/PROCESSING 可更新）
    let no_flip = ps(&db)
        .callback(CallbackReq {
            payment_id: p.id,
            provider_tx_id: Some(common::unique("TXG2")),
            success: Some(true),
            payload: None,
        })
        .await
        .expect("FAILED 后的回调不报错");
    assert_eq!(no_flip.status, "FAILED", "FAILED 终态不可原地翻转，重试须重新发起支付");

    // 重试路径：订单仍 CREATED → 重新预支付产生新流水（CREATED）→ 随后可正常支付成功
    let p2 = ps(&db)
        .prepay(CreatePaymentReq { order_id: o.id, user_id: c.user_id, channel: String::new() })
        .await
        .expect("重新预支付成功");
    assert_eq!(p2.status, "CREATED");
    assert_eq!(p2.amount, o.payable_amount, "新支付单金额仍 == 订单实付");
    assert_eq!(count_payments(&db, o.id).await, 2, "一次失败 + 一次重试 = 两条流水");

    let retry_ok = ps(&db)
        .callback(CallbackReq {
            payment_id: p2.id,
            provider_tx_id: Some(common::unique("TXOK")),
            success: Some(true),
            payload: None,
        })
        .await
        .expect("重试支付回调成功");
    assert_eq!(retry_ok.status, "SUCCESS");

    // 收尾：先删“他人用户”行（无业务行引用它），再逆 FK 序清本链
    let _ = db.exec_drop("DELETE FROM users WHERE id = ?", vec![other_user]).await;
    cleanup_all(&db, &c, &q, &o, None).await;
}
