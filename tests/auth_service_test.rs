//! 认证服务集成测试（任务 #5，MySQL 集成）
//!
//! 覆盖：注册（唯一用户名 + 双令牌签发）、重复用户名拒绝、
//! 登录（正确/错误密码、未注册用户）、微信登录 stub。
//!
//! 依赖 `insurance_service` 库 + 本地 MySQL（install.sql 建库）。
//! MySQL 不可用时 SKIP（打印提示并提前返回），保证 `cargo test` 在无库环境不失败。

mod common;

use insurance_service::error::AppError;
use insurance_service::services::auth_service::{AuthService, LoginReq, RegisterReq, WechatLoginReq};

/// 构造 AuthService（测试固定密钥/配置 + 测试库连接）
async fn service() -> Option<AuthService> {
    let db = common::test_db().await?;
    Some(AuthService::new(common::jwt_cfg(3600), common::crypto(), db))
}

/// 清理辅助：按 DATABASE_URL 独立建一个连接池（共享同一数据库），仅用于删除测试行。
async fn cleanup_db() -> Option<insurance_service::db::Db> {
    common::test_db().await
}

#[tokio::test]
async fn register_returns_token_pair() {
    let Some(svc) = service().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let username = common::unique("tester");
    let req = RegisterReq {
        username: username.clone(),
        password: "P@ssw0rd!".to_string(),
        phone: "13800138000".to_string(),
    };
    let result = svc.register(req).await.expect("注册成功");
    assert_eq!(result.username, username);
    assert!(result.user_id > 0);
    assert_eq!(result.role, "USER");
    assert!(!result.tokens.access_token.is_empty());
    assert!(!result.tokens.refresh_token.is_empty());
    assert_eq!(result.tokens.token_type, "Bearer");
    assert_eq!(result.tokens.expires_in, 7200);
    if let Some(db) = cleanup_db().await {
        common::delete_user(&db, &username).await;
    }
}

#[tokio::test]
async fn register_duplicate_username_rejected() {
    let Some(svc) = service().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let username = common::unique("tester");
    let req = RegisterReq {
        username: username.clone(),
        password: "P@ssw0rd!".to_string(),
        phone: "13800138000".to_string(),
    };
    svc.register(req).await.expect("首次注册成功");
    // 再次注册同名用户 → 业务错误
    let dup = RegisterReq {
        username: username.clone(),
        password: "Another1!".to_string(),
        phone: "13900139000".to_string(),
    };
    let err = svc.register(dup).await.expect_err("重复用户名应失败");
    match err {
        AppError::Business(msg) => assert!(msg.contains("用户名已存在"), "msg={msg}"),
        other => panic!("预期 Business 错误，得到 {other:?}"),
    }
    if let Some(db) = cleanup_db().await {
        common::delete_user(&db, &username).await;
    }
}

#[tokio::test]
async fn login_success() {
    let Some(svc) = service().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let username = common::unique("tester");
    let password = "P@ssw0rd!".to_string();
    svc.register(RegisterReq {
        username: username.clone(),
        password: password.clone(),
        phone: "13800138000".to_string(),
    })
    .await
    .expect("注册成功");

    let result = svc
        .login(LoginReq {
            username: username.clone(),
            password: password.clone(),
        })
        .await
        .expect("登录成功");
    assert_eq!(result.username, username);
    assert!(!result.tokens.access_token.is_empty());
    if let Some(db) = cleanup_db().await {
        common::delete_user(&db, &username).await;
    }
}

#[tokio::test]
async fn login_wrong_password_rejected() {
    let Some(svc) = service().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let username = common::unique("tester");
    svc.register(RegisterReq {
        username: username.clone(),
        password: "P@ssw0rd!".to_string(),
        phone: "13800138000".to_string(),
    })
    .await
    .expect("注册成功");

    let err = svc
        .login(LoginReq {
            username: username.clone(),
            password: "WrongPass1!".to_string(),
        })
        .await
        .expect_err("错误密码应失败");
    match err {
        AppError::Business(msg) => assert!(msg.contains("用户名或密码错误"), "msg={msg}"),
        other => panic!("预期 Business 错误，得到 {other:?}"),
    }
    if let Some(db) = cleanup_db().await {
        common::delete_user(&db, &username).await;
    }
}

#[tokio::test]
async fn login_unknown_user_rejected() {
    let Some(svc) = service().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let err = svc
        .login(LoginReq {
            username: common::unique("nobody"),
            password: "P@ssw0rd!".to_string(),
        })
        .await
        .expect_err("未注册用户应失败");
    match err {
        AppError::Business(msg) => assert!(msg.contains("用户名或密码错误"), "msg={msg}"),
        other => panic!("预期 Business 错误，得到 {other:?}"),
    }
}

#[tokio::test]
async fn wechat_login_stub_rejected() {
    let Some(svc) = service().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let err = svc
        .wechat_login(WechatLoginReq {
            code: "wx-code-123".to_string(),
        })
        .await
        .expect_err("微信登录 stub 应失败");
    match err {
        AppError::Business(msg) => assert!(msg.contains("微信登录未接入"), "msg={msg}"),
        other => panic!("预期 Business 错误，得到 {other:?}"),
    }
}
