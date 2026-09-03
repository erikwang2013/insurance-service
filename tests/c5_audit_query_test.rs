//! C5 运营端审计日志查询 API 集成测试（HTTP 层，任务 #25）
//!
//! 覆盖：GET /api/v1/admin/audit-logs —— USER 角色 → 40300；缺失
//! operator_user_id → 40000；OPERATOR 可按 action/user_id/entity_id/
//! entity_type/时间范围过滤；分页（size/page 回显与 LIMIT 正确性）；
//! created_at DESC 排序；日期型 created_to 含当日整日。
//!
//! 审计数据直插 audit_logs（不走业务动作），全部过滤断言限定在本次测试的
//! 唯一 action 令牌内，与并行 agent 写入共享库的行互不干扰。
//!
//! 路由尚未由 lead 合入 routes.rs，故本测试自建最小 axum Router 挂载
//! `admin_handler`（与 bee 注册同一控制器分派管线，仅略过命名空间层）；
//! lead 接线后该测试仍成立。
//!
//! 依赖 MySQL（默认 127.0.0.1:13307，可用 DATABASE_URL 覆盖）。
//! 不可用时 SKIP（打印提示并提前返回），保证无库环境不失败。

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use serde_json::Value;
use tower::util::ServiceExt;

use insurance_service::config::{AppConfig, DbConfig};
use insurance_service::controllers::{admin_handler, AppState};
use insurance_service::db::Db;
use insurance_service::response::ResponseEnvelope;

use mysql_async::prelude::Queryable;
use mysql_async::Value as MyValue;

// 测试专用密钥（32 字节 0x07 的 base64；与 tests/common/mod.rs::crypto() 一致）
const TEST_CRYPTO_KEY_B64: &str = "BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwc=";

/// 测试库地址（DATABASE_URL 优先，缺省任务库 13307）。
fn db_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://root:@127.0.0.1:13307/insurance_service".to_string())
}

/// 插入指定角色的测试用户，返回自增 id。
async fn insert_user(db: &Db, username: &str, role: &str) -> i64 {
    let mut conn = db.conn().await.expect("连接测试库");
    conn.exec_drop(
        "INSERT INTO users (username, password_hash, role) VALUES (?, ?, ?)",
        vec![
            MyValue::from(username),
            MyValue::from("test-hash"),
            MyValue::from(role),
        ],
    )
    .await
    .expect("插入用户");
    conn.last_insert_id().expect("取得自增 id") as i64
}

/// 直插一条审计日志（created_at 显式指定以断言排序/时间过滤）。
async fn insert_audit(
    db: &Db,
    user_id: Option<i64>,
    action: &str,
    entity_type: &str,
    entity_id: i64,
    created_at: &str,
    with_json: bool,
) -> i64 {
    let mut conn = db.conn().await.expect("连接测试库");
    if with_json {
        conn.exec_drop(
            "INSERT INTO audit_logs (user_id, action, entity_type, entity_id, before_json, created_at) \
             VALUES (?, ?, ?, ?, ?, ?)",
            vec![
                user_id.map(MyValue::from).unwrap_or(MyValue::NULL),
                MyValue::from(action),
                MyValue::from(entity_type),
                MyValue::from(entity_id),
                MyValue::from(r#"{"note":"c5"}"#),
                MyValue::from(created_at),
            ],
        )
        .await
        .expect("插入审计日志");
    } else {
        conn.exec_drop(
            "INSERT INTO audit_logs (user_id, action, entity_type, entity_id, created_at) \
             VALUES (?, ?, ?, ?, ?)",
            vec![
                user_id.map(MyValue::from).unwrap_or(MyValue::NULL),
                MyValue::from(action),
                MyValue::from(entity_type),
                MyValue::from(entity_id),
                MyValue::from(created_at),
            ],
        )
        .await
        .expect("插入审计日志");
    }
    conn.last_insert_id().expect("取得自增 id") as i64
}

/// 清理直插的审计行（按唯一 action 令牌，只命中本测试数据）。
async fn cleanup_audits(db: &Db, actions: &[&str]) {
    for a in actions {
        let _ = db
            .exec_drop("DELETE FROM audit_logs WHERE action = ?", vec![a])
            .await;
    }
}

/// 装配最小 Router：仅挂载 /api/v1/admin/audit-logs（与 bee 注册同一
/// admin_handler 适配器，走 AppState::run 完整控制器管线）。
/// MySQL 不可用 → Err（调用方打印 SKIP 后 return）。
fn make_app() -> Result<Router, String> {
    let cfg = AppConfig {
        server: insurance_service::config::ServerConfig {
            host: "127.0.0.1".into(),
            port: 0,
        },
        database: DbConfig { url: db_url() },
        redis: insurance_service::config::RedisConfig {
            url: "redis://127.0.0.1:6379".into(),
        },
        opensearch: insurance_service::config::SearchConfig {
            url: "http://127.0.0.1:9200".into(),
            username: "admin".into(),
            password: "changeme".into(),
        },
        jwt: common::jwt_cfg(3600),
        crypto: insurance_service::config::CryptoConfig {
            master_key: TEST_CRYPTO_KEY_B64.into(),
        },
        log: insurance_service::config::LogConfig {
            level: "info".into(),
        },
        wechat: insurance_service::config::WechatConfig {
            app_id: String::new(),
            app_secret: String::new(),
        },
    };
    let state = AppState::new(&cfg)?;
    Ok(Router::new()
        .route(
            "/api/v1/admin/audit-logs",
            axum::routing::get(admin_handler),
        )
        .with_state(state))
}

/// GET 请求（query 拼在 path 上）。
fn get_q(path: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(path)
        .body(Body::empty())
        .expect("构造 GET 请求")
}

/// 发送请求，收集状态码 + 响应体。
async fn send(app: &Router, req: Request<Body>) -> (StatusCode, Vec<u8>) {
    let res = app.clone().oneshot(req).await.expect("oneshot 成功");
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), 1 << 20)
        .await
        .expect("收集响应体");
    (status, bytes.to_vec())
}

/// 解析信封响应。
fn parse_envelope(bytes: &[u8]) -> ResponseEnvelope<Value> {
    serde_json::from_slice(bytes).expect("响应体可解析为 ResponseEnvelope")
}

/// 断言成功信封并取出 data。
fn ok_data(env: ResponseEnvelope<Value>) -> Value {
    assert_eq!(env.code, 0, "业务码应为 0，resp={env:?}");
    env.data.expect("成功响应带 data")
}

/// 断言分页查询返回：total、list 长度、page/size 回显，并返回 list。
/// list 长度 = 本页实有行数（越界页为空，而非 min(total, size)）。
fn assert_list(data: Value, expect_total: i64, page: i64, size: i64) -> Vec<Value> {
    assert_eq!(data["total"].as_i64(), Some(expect_total), "total 不符，data={data}");
    assert_eq!(data["page"].as_i64(), Some(page), "page 回显不符");
    assert_eq!(data["size"].as_i64(), Some(size), "size 回显不符");
    let start = (page - 1) * size;
    let expect_len = if start >= expect_total {
        0
    } else {
        (expect_total - start).min(size)
    };
    let list = data["list"].as_array().expect("list 为数组");
    assert_eq!(list.len() as i64, expect_len, "list 长度不符，data={data}");
    list.clone()
}

/// 单测试串行覆盖全部场景（共享库下多条断言只依赖本测试的 action 令牌/自建
/// 用户，与并行 agent 写入互不干扰；仍合并为顺序流程降低交错概率）。
#[tokio::test]
async fn c5_audit_query_api_flow() {
    // ---- 准备：库可用性 + 最小 app ----
    let db = match Db::new(&DbConfig { url: db_url() }) {
        Ok(d) => d,
        Err(_) => {
            eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
            return;
        }
    };
    if db.conn().await.is_err() {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    }
    let app = match make_app() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("SKIP: 装配 app 失败（MySQL 未就绪）: {e}");
            return;
        }
    };

    // ---- 准备：操作人 + 普通用户 + 4 条直插审计（唯一 action 令牌） ----
    let op_name = common::unique("c5o");
    let op_id = insert_user(&db, &op_name, "OPERATOR").await;
    let u_name = common::unique("c5u");
    let u_id = insert_user(&db, &u_name, "USER").await;

    let action_a = common::unique("c5a"); // 2 行（u_id）：10:00:01 / 10:00:02
    let action_b = common::unique("c5b"); // 2 行：10:00:03（u_id）/ 10:00:00（NULL 用户）
    let id_a1 = insert_audit(&db, Some(u_id), &action_a, "ORDER", 100, "2026-09-01 10:00:01", true).await;
    let id_a2 = insert_audit(&db, Some(u_id), &action_a, "ORDER", 200, "2026-09-01 10:00:02", false).await;
    let id_b1 = insert_audit(&db, Some(u_id), &action_b, "POLICY", 300, "2026-09-01 10:00:03", false).await;
    let id_b2 = insert_audit(&db, None, &action_b, "ORDER", 400, "2026-09-01 10:00:00", false).await;

    // ---- 场景 1：角色鉴权与参数校验 ----
    // 操作人缺省 → 40000
    let (status, bytes) = send(&app, get_q("/api/v1/admin/audit-logs")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(parse_envelope(&bytes).code, 40000, "缺 operator_user_id → 40000");

    // USER 角色操作人 → 40300
    let (_, bytes) = send(
        &app,
        get_q(&format!("/api/v1/admin/audit-logs?operator_user_id={u_id}")),
    )
    .await;
    assert_eq!(parse_envelope(&bytes).code, 40300, "USER 角色 → 40300");

    // 不存在的操作人 → 40300
    let (_, bytes) = send(
        &app,
        get_q("/api/v1/admin/audit-logs?operator_user_id=999999999"),
    )
    .await;
    assert_eq!(parse_envelope(&bytes).code, 40300, "不存在操作人 → 40300");

    // 非法时间参数 → 40000
    let (_, bytes) = send(
        &app,
        get_q(&format!(
            "/api/v1/admin/audit-logs?operator_user_id={op_id}&created_from=abc"
        )),
    )
    .await;
    assert_eq!(parse_envelope(&bytes).code, 40000, "非法时间参数 → 40000");

    // ---- 场景 2：OPERATOR 可查 + action 过滤 + DESC 排序 ----
    let (_, bytes) = send(
        &app,
        get_q(&format!(
            "/api/v1/admin/audit-logs?operator_user_id={op_id}&action={action_a}"
        )),
    )
    .await;
    let list = assert_list(ok_data(parse_envelope(&bytes)), 2, 1, 20);
    assert_eq!(list[0]["id"].as_i64(), Some(id_a2), "新行在前（DESC）");
    assert_eq!(list[1]["id"].as_i64(), Some(id_a1));
    // 排序键 created_at 也在响应中
    let t0 = list[0]["created_at"].as_str().expect("created_at 有值");
    let t1 = list[1]["created_at"].as_str().expect("created_at 有值");
    assert!(t0 > t1, "created_at 降序，{t0} 应晚于 {t1}");
    // 字段完整性：before_json（直插的 a1 行）应还原为 JSON 对象
    let j = list[1]["before_json"].as_object().expect("before_json 还原为对象");
    assert_eq!(j.get("note").and_then(Value::as_str), Some("c5"));
    assert_eq!(list[1]["user_id"].as_i64(), Some(u_id));
    assert_eq!(list[1]["entity_type"].as_str(), Some("ORDER"));
    assert_eq!(list[1]["entity_id"].as_i64(), Some(100));

    // ---- 场景 3：按 user_id 过滤（该用户共 3 行） ----
    let (_, bytes) = send(
        &app,
        get_q(&format!(
            "/api/v1/admin/audit-logs?operator_user_id={op_id}&user_id={u_id}"
        )),
    )
    .await;
    assert_list(ok_data(parse_envelope(&bytes)), 3, 1, 20);

    // ---- 场景 4：entity_type / entity_id 过滤 ----
    // 共享库下纯 entity_type 全库计数无界（并行 agent 也在写 POLICY），
    // 故叠加本测试的 action 令牌限定；组合过滤同样验证 entity_type 列条件。
    let (_, bytes) = send(
        &app,
        get_q(&format!(
            "/api/v1/admin/audit-logs?operator_user_id={op_id}&action={action_b}&entity_type=POLICY"
        )),
    )
    .await;
    let list = assert_list(ok_data(parse_envelope(&bytes)), 1, 1, 20);
    assert_eq!(list[0]["id"].as_i64(), Some(id_b1));

    let (_, bytes) = send(
        &app,
        get_q(&format!(
            "/api/v1/admin/audit-logs?operator_user_id={op_id}&action={action_a}&entity_id=200"
        )),
    )
    .await;
    let list = assert_list(ok_data(parse_envelope(&bytes)), 1, 1, 20);
    assert_eq!(list[0]["id"].as_i64(), Some(id_a2), "entity_id 精确命中");

    // NULL user_id 行可查且按 created_at 正确落位（同 action_b 下最旧）
    let (_, bytes) = send(
        &app,
        get_q(&format!(
            "/api/v1/admin/audit-logs?operator_user_id={op_id}&action={action_b}"
        )),
    )
    .await;
    let list = assert_list(ok_data(parse_envelope(&bytes)), 2, 1, 20);
    assert_eq!(list[0]["id"].as_i64(), Some(id_b1));
    assert_eq!(list[1]["id"].as_i64(), Some(id_b2), "NULL 用户行按时间落位");
    assert!(list[1]["user_id"].is_null(), "无操作用户的日志 user_id 为 null");

    // ---- 场景 5：时间范围（闭区间 + date-only 上界含当日） ----
    let (_, bytes) = send(
        &app,
        get_q(&format!(
            "/api/v1/admin/audit-logs?operator_user_id={op_id}&action={action_a}&created_from=2026-09-01%2010:00:02"
        )),
    )
    .await;
    let list = assert_list(ok_data(parse_envelope(&bytes)), 1, 1, 20);
    assert_eq!(list[0]["id"].as_i64(), Some(id_a2), "created_from 含边界、剔旧行");

    let (_, bytes) = send(
        &app,
        get_q(&format!(
            "/api/v1/admin/audit-logs?operator_user_id={op_id}&action={action_a}&created_to=2026-09-01"
        )),
    )
    .await;
    assert_list(ok_data(parse_envelope(&bytes)), 2, 1, 20);

    // ---- 场景 6：分页（size=1：page1 最新 → page2 次新 → page3 空） ----
    let (_, bytes) = send(
        &app,
        get_q(&format!(
            "/api/v1/admin/audit-logs?operator_user_id={op_id}&action={action_a}&page=1&size=1"
        )),
    )
    .await;
    let list = assert_list(ok_data(parse_envelope(&bytes)), 2, 1, 1);
    assert_eq!(list[0]["id"].as_i64(), Some(id_a2));

    let (_, bytes) = send(
        &app,
        get_q(&format!(
            "/api/v1/admin/audit-logs?operator_user_id={op_id}&action={action_a}&page=2&size=1"
        )),
    )
    .await;
    let list = assert_list(ok_data(parse_envelope(&bytes)), 2, 2, 1);
    assert_eq!(list[0]["id"].as_i64(), Some(id_a1), "第二页返回次新行");

    let (_, bytes) = send(
        &app,
        get_q(&format!(
            "/api/v1/admin/audit-logs?operator_user_id={op_id}&action={action_a}&page=3&size=1"
        )),
    )
    .await;
    assert_list(ok_data(parse_envelope(&bytes)), 2, 3, 1);

    // ---- 场景 7：空结果（未命中 action）也正常返回 ----
    let (_, bytes) = send(
        &app,
        get_q(&format!(
            "/api/v1/admin/audit-logs?operator_user_id={op_id}&action={action_a}_nope"
        )),
    )
    .await;
    let data = ok_data(parse_envelope(&bytes));
    assert_eq!(data["total"].as_i64(), Some(0));
    assert_eq!(data["list"].as_array().map(Vec::len), Some(0));

    // ---- 收尾：先删审计行，再删用户 ----
    cleanup_audits(&db, &[action_a.as_str(), action_b.as_str()]).await;
    common::delete_user(&db, &u_name).await;
    common::delete_user(&db, &op_name).await;
}
