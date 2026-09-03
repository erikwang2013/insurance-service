//! 用户中心集成测试：修改密码 / 换绑手机（MySQL 集成）
//!
//! 覆盖：旧密码错误被拒 → 正确改密 → 旧密码登录失败 / 新密码登录成功 →
//! 换绑：错误密码被拒 / 非法手机号被拒 → 正确换绑后 me() 脱敏手机号变化；
//! 另经 `AppState::run` 走 HTTP 信封校验控制器分派（/user/password、/user/phone）。
//!
//! MySQL 不可用时 SKIP（打印提示并提前返回）；测试用户以 `unique()` 命名，
//! 结束按唯一用户名删除，不干扰并行 agent 数据。

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};

use insurance_service::config::AppConfig;
use insurance_service::controllers::AppState;
use insurance_service::crypto::Masker;
use insurance_service::db::Db;
use insurance_service::error::AppError;
use insurance_service::response::ResponseEnvelope;
use insurance_service::services::auth_service::{
    AuthService, LoginReq, LoginResult, RegisterReq,
};

// 测试专用密钥（32 字节 0x07 的 base64；与 tests/common/mod.rs::crypto() 一致）
const TEST_CRYPTO_KEY_B64: &str = "BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwc=";
const PHONE_A: &str = "13800138000";
const PHONE_B: &str = "13900139000";
const PWD_OLD: &str = "P@ssw0rd!";
const PWD_NEW: &str = "NewPass1!";

/// 构造 AuthService（测试固定密钥/配置 + 测试库连接）
async fn service() -> Option<AuthService> {
    let db = common::test_db().await?;
    Some(AuthService::new(common::jwt_cfg(3600), common::crypto(), db, common::wechat_client()))
}

async fn db() -> Option<Db> {
    common::test_db().await
}

/// 注册唯一测试用户（先插）
async fn register_user(svc: &AuthService) -> LoginResult {
    let req = RegisterReq {
        username: common::unique("uc"),
        password: PWD_OLD.to_string(),
        phone: PHONE_A.to_string(),
    };
    svc.register(req).await.expect("注册成功")
}

/// 删除测试用户（后删；失败不冒泡）
async fn cleanup_user(username: &str) {
    if let Some(db) = db().await {
        common::delete_user(&db, username).await;
    }
}

// ---------------------------------------------------------------------------
// 修改密码（服务层）
// ---------------------------------------------------------------------------

#[tokio::test]
async fn change_password_service_level() {
    let Some(svc) = service().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let reg = register_user(&svc).await;

    // 旧密码错误 → Business（旧密码错误）
    let err = svc
        .change_password(reg.user_id, "WrongOld1!", PWD_NEW)
        .await
        .expect_err("旧密码错误应被拒绝");
    assert!(
        matches!(&err, AppError::Business(m) if m.contains("旧密码错误")),
        "应提示旧密码错误，实际 {err:?}"
    );

    // 正确旧密码 → 改密成功
    svc.change_password(reg.user_id, PWD_OLD, PWD_NEW)
        .await
        .expect("正确旧密码改密成功");

    // 旧密码登录失败（登录统一提示，不区分用户名/密码）
    let err = svc
        .login(LoginReq { username: reg.username.clone(), password: PWD_OLD.to_string() })
        .await
        .expect_err("旧密码登录应失败");
    assert!(
        matches!(&err, AppError::Business(m) if m.contains("用户名或密码错误")),
        "旧密码登录应被拒，实际 {err:?}"
    );

    // 新密码登录成功
    let ok = svc
        .login(LoginReq { username: reg.username.clone(), password: PWD_NEW.to_string() })
        .await
        .expect("新密码登录成功");
    assert_eq!(ok.user_id, reg.user_id, "登录用户一致");

    // 不存在的用户 → NotFound
    let err = svc
        .change_password(9_999_999_999, PWD_NEW, PWD_OLD)
        .await
        .expect_err("不存在用户应 NotFound");
    assert!(matches!(err, AppError::NotFound), "应为 NotFound，实际 {err:?}");

    cleanup_user(&reg.username).await;
}

// ---------------------------------------------------------------------------
// 换绑手机（服务层）
// ---------------------------------------------------------------------------

#[tokio::test]
async fn bind_phone_service_level() {
    let Some(svc) = service().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let reg = register_user(&svc).await;

    // 密码错误 → Business
    let err = svc
        .bind_phone(reg.user_id, "WrongOld1!", PHONE_B)
        .await
        .expect_err("错误密码换绑应被拒绝");
    assert!(
        matches!(&err, AppError::Business(m) if m.contains("密码错误")),
        "应提示密码错误，实际 {err:?}"
    );

    // 非法手机号 → Validation
    let err = svc
        .bind_phone(reg.user_id, PWD_OLD, "12345")
        .await
        .expect_err("非法手机号应被拒绝");
    assert!(
        matches!(err, AppError::Validation(_)),
        "非法手机号应为校验错误，实际 {err:?}"
    );

    // 正确密码 + 合法新手机号 → 换绑成功
    svc.bind_phone(reg.user_id, PWD_OLD, PHONE_B)
        .await
        .expect("正确密码换绑成功");

    // me()：脱敏手机号随换绑变化，密文可解密回新手机号
    let user = svc.me(reg.user_id).await.expect("资料查询命中");
    let before = Masker::phone(PHONE_A);
    let after = Masker::phone(PHONE_B);
    assert_ne!(before, after, "新旧手机号脱敏值应不同（测试前提）");
    assert_eq!(user.phone_masked.as_deref(), Some(after.as_str()), "脱敏值已更新");
    let enc = user.phone_enc.as_deref().expect("密文存在");
    let plain = common::crypto().decrypt_str(enc).expect("解密成功");
    assert_eq!(plain, PHONE_B, "密文为新手机号");

    cleanup_user(&reg.username).await;
}

// ---------------------------------------------------------------------------
// 控制器分派（HTTP 信封，经 AppState::run 直驱 auth 控制器）
// ---------------------------------------------------------------------------

/// 装配共享状态（读取环境变量 + 初始化 MySQL 连接池；失败 → Err → SKIP）
fn make_state() -> Result<AppState, String> {
    let cfg = AppConfig {
        server: insurance_service::config::ServerConfig {
            host: "127.0.0.1".into(),
            port: 0,
        },
        database: insurance_service::config::DbConfig {
            url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "mysql://root:@127.0.0.1:3306/insurance_service".into()),
        },
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
    AppState::new(&cfg)
}

/// 经 auth 控制器跑一个 JSON POST 请求，返回信封（HTTP 状态码 + 业务信封）
async fn post(state: &AppState, path: &str, body: Value) -> (StatusCode, ResponseEnvelope<Value>) {
    let request = Request::builder()
        .method("POST")
        .uri(path)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .expect("构造请求");
    let resp = state
        .run(state.auth.as_ref(), Default::default(), request)
        .await;
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), 1 << 20)
        .await
        .expect("收集响应体");
    let envelope = serde_json::from_slice(&bytes).expect("响应体可解析为 ResponseEnvelope");
    (status, envelope)
}

#[tokio::test]
async fn api_user_center_full_flow() {
    let state = match make_state() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("SKIP: 装配 app 失败（MySQL 未就绪）: {e}");
            return;
        }
    };
    let username = common::unique("uc_api");

    // 注册（POST /api/v1/auth/register）
    let (status, env) = post(
        &state,
        "/api/v1/auth/register",
        json!({ "username": username, "password": PWD_OLD, "phone": PHONE_A }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "注册 HTTP 200");
    assert_eq!(env.code, 0, "注册成功，resp={env:?}");
    let user_id = env
        .data
        .as_ref()
        .and_then(|d| d.get("user_id"))
        .and_then(Value::as_i64)
        .expect("注册返回 user_id");
    let pwd_path = format!("/api/v1/user/password?user_id={user_id}");
    let phone_path = format!("/api/v1/user/phone?user_id={user_id}");

    // 旧密码错误改密 → 40001 信封
    let (status, env) = post(
        &state,
        &pwd_path,
        json!({ "old_password": "WrongOld1!", "new_password": PWD_NEW }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "业务错误也走 HTTP 200 信封");
    assert_eq!(env.code, 40001, "旧密码错误业务码 40001，resp={env:?}");
    assert!(env.message.contains("旧密码错误"), "提示旧密码错误，resp={env:?}");

    // 正确改密 → code 0
    let (_, env) = post(
        &state,
        &pwd_path,
        json!({ "old_password": PWD_OLD, "new_password": PWD_NEW }),
    )
    .await;
    assert_eq!(env.code, 0, "改密成功，resp={env:?}");
    assert_eq!(
        env.data.as_ref().and_then(|d| d.get("changed")).and_then(Value::as_bool),
        Some(true)
    );

    // 旧密码登录失败 / 新密码登录成功（POST /api/v1/auth/login）
    let (_, env) = post(
        &state,
        "/api/v1/auth/login",
        json!({ "username": username, "password": PWD_OLD }),
    )
    .await;
    assert_eq!(env.code, 40001, "旧密码登录被拒，resp={env:?}");
    let (_, env) = post(
        &state,
        "/api/v1/auth/login",
        json!({ "username": username, "password": PWD_NEW }),
    )
    .await;
    assert_eq!(env.code, 0, "新密码登录成功，resp={env:?}");

    // 换绑：错误密码被拒 → 非法手机号被拒 → 正确换绑
    let (_, env) = post(
        &state,
        &phone_path,
        json!({ "password": "WrongOld1!", "new_phone": PHONE_B }),
    )
    .await;
    assert_eq!(env.code, 40001, "错误密码换绑被拒，resp={env:?}");
    let (_, env) = post(
        &state,
        &phone_path,
        json!({ "password": PWD_NEW, "new_phone": "12345" }),
    )
    .await;
    assert_eq!(env.code, 40000, "非法手机号校验错误，resp={env:?}");
    let (_, env) = post(
        &state,
        &phone_path,
        json!({ "password": PWD_NEW, "new_phone": PHONE_B }),
    )
    .await;
    assert_eq!(env.code, 0, "换绑成功，resp={env:?}");

    // GET /api/v1/user/me：脱敏手机号已变化为新号
    let me_path = format!("/api/v1/user/me?user_id={user_id}");
    let request = Request::builder()
        .method("GET")
        .uri(me_path)
        .body(Body::empty())
        .expect("构造请求");
    let resp = state.run(state.auth.as_ref(), Default::default(), request).await;
    let bytes = axum::body::to_bytes(resp.into_body(), 1 << 20)
        .await
        .expect("收集响应体");
    let env: ResponseEnvelope<Value> = serde_json::from_slice(&bytes).expect("信封可解析");
    assert_eq!(env.code, 0, "me 查询成功，resp={env:?}");
    let masked = env
        .data
        .as_ref()
        .and_then(|d| d.get("phone_masked"))
        .and_then(Value::as_str)
        .expect("me 返回 phone_masked");
    assert_eq!(masked, Masker::phone(PHONE_B), "脱敏值已随换绑更新");

    if let Ok(db_url) = std::env::var("DATABASE_URL") {
        let _ = common::delete_user_by_conn(&db_url, &username).await;
    }
}
