//! 认证修复集成测试（#10，MySQL 集成）
//!
//! 覆盖：/auth/refresh 真实实现（重签成功 / 伪造 / 过期 / 禁用 / 软删用户拒绝）、
//! /user/me 资料查询（命中 / 不存在 / 已删除 NotFound）。
//!
//! 依赖 `insurance_service` 库 + 本地 MySQL（install.sql 建库）。
//! MySQL 不可用时 SKIP（打印提示并提前返回），保证 `cargo test` 在无库环境不失败。
//! 测试先插后删：行用 `unique()` 命名，结束后按唯一用户名删除。

mod common;

use insurance_service::db::Db;
use insurance_service::error::AppError;
use insurance_service::middleware::auth::{JwtService, Role};
use insurance_service::services::auth_service::{AuthService, LoginResult, RegisterReq};

/// 构造 AuthService（测试固定密钥/配置 + 测试库连接）
async fn service() -> Option<AuthService> {
    let db = common::test_db().await?;
    Some(AuthService::new(common::jwt_cfg(3600), common::crypto(), db, common::wechat_client()))
}

/// 独立连接池（用于行更新/清理，共享同一测试库）
async fn db() -> Option<Db> {
    common::test_db().await
}

/// 注册唯一测试用户（先插）
async fn register_user(svc: &AuthService) -> LoginResult {
    let req = RegisterReq {
        username: common::unique("fix"),
        password: "P@ssw0rd!".to_string(),
        phone: "13800138000".to_string(),
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
// /auth/refresh
// ---------------------------------------------------------------------------

#[tokio::test]
async fn refresh_reissues_token_pair() {
    let Some(svc) = service().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let reg = register_user(&svc).await;

    let refreshed = svc.refresh(&reg.tokens.refresh_token).await.expect("刷新成功");
    assert_eq!(refreshed.user_id, reg.user_id, "重签保持同一用户");
    assert_eq!(refreshed.username, reg.username);
    assert_eq!(refreshed.role, "USER");
    assert!(!refreshed.tokens.access_token.is_empty(), "重签返回 access_token");
    assert!(!refreshed.tokens.refresh_token.is_empty(), "重签返回 refresh_token");
    assert_eq!(refreshed.tokens.token_type, "Bearer");

    cleanup_user(&reg.username).await;
}

#[tokio::test]
async fn refresh_rejects_invalid_token() {
    let Some(svc) = service().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    // 伪造/损坏令牌：签名校验失败 → Unauthorized（不经数据库）
    let err = svc.refresh("not-a-jwt").await.expect_err("伪造令牌应被拒绝");
    assert!(matches!(err, AppError::Unauthorized), "应为 Unauthorized，实际 {err:?}");
}

#[tokio::test]
async fn refresh_rejects_expired_token() {
    let Some(svc) = service().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    // 过期令牌：负 access_expiry 使 exp 落于过去
    let expired = JwtService::new(common::jwt_cfg(-1))
        .issue_access_token(1, Role::User, None)
        .expect("过期令牌签发成功");
    let err = svc.refresh(&expired).await.expect_err("过期令牌应被拒绝");
    assert!(matches!(err, AppError::Unauthorized), "应为 Unauthorized，实际 {err:?}");
}

#[tokio::test]
async fn refresh_rejects_disabled_user() {
    let Some(svc) = service().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let Some(db) = db().await else {
        eprintln!("SKIP: MySQL 不可用");
        return;
    };
    let reg = register_user(&svc).await;

    // 禁用账号后重签应被拒绝（40001 业务错误）
    db.exec_drop(
        "UPDATE users SET status = 'DISABLED' WHERE username = ?",
        vec![reg.username.clone()],
    )
    .await
    .expect("禁用账号落库");
    let err = svc
        .refresh(&reg.tokens.refresh_token)
        .await
        .expect_err("禁用账号刷新应被拒绝");
    assert!(
        matches!(err, AppError::Business(_)),
        "应为 Business 业务错误，实际 {err:?}"
    );

    cleanup_user(&reg.username).await;
}

#[tokio::test]
async fn refresh_rejects_deleted_user() {
    let Some(svc) = service().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let Some(db) = db().await else {
        eprintln!("SKIP: MySQL 不可用");
        return;
    };
    let reg = register_user(&svc).await;

    // 软删后重签应被拒绝（查询过滤 deleted_at IS NULL → 视同用户不存在 → Unauthorized）
    db.exec_drop(
        "UPDATE users SET deleted_at = NOW() WHERE username = ?",
        vec![reg.username.clone()],
    )
    .await
    .expect("软删账号落库");
    let err = svc
        .refresh(&reg.tokens.refresh_token)
        .await
        .expect_err("软删用户刷新应被拒绝");
    assert!(matches!(err, AppError::Unauthorized), "应为 Unauthorized，实际 {err:?}");

    cleanup_user(&reg.username).await;
}

// ---------------------------------------------------------------------------
// /user/me
// ---------------------------------------------------------------------------

#[tokio::test]
async fn me_returns_profile_fields() {
    let Some(svc) = service().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let reg = register_user(&svc).await;

    let user = svc.me(reg.user_id).await.expect("资料查询命中");
    assert_eq!(user.id, reg.user_id);
    assert_eq!(user.username, reg.username, "用户名一致");
    assert_eq!(user.role, "USER");
    assert_eq!(user.status, "ACTIVE");
    assert_eq!(user.nickname, None);
    assert_eq!(user.email, None);
    assert!(
        user.phone_masked.as_deref().is_some_and(|m| m.contains("****")),
        "返回脱敏手机号，实际 {:?}",
        user.phone_masked
    );
    // 敏感字段只留在模型内部（响应 JSON 由 #[serde(skip_serializing)] 剔除）
    assert!(user.password_hash.len() > 20, "内部仍持有哈希供校验");
    assert!(user.phone_enc.is_some(), "内部仍持有密文供解密");

    cleanup_user(&reg.username).await;
}

#[tokio::test]
async fn me_not_found_for_missing_user() {
    let Some(svc) = service().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    // 不可能存在的超大 id → NotFound（40400）
    let err = svc.me(9_999_999_999).await.expect_err("不存在用户应 NotFound");
    assert!(matches!(err, AppError::NotFound), "应为 NotFound，实际 {err:?}");
}

#[tokio::test]
async fn me_not_found_for_deleted_user() {
    let Some(svc) = service().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let Some(db) = db().await else {
        eprintln!("SKIP: MySQL 不可用");
        return;
    };
    let reg = register_user(&svc).await;

    db.exec_drop(
        "UPDATE users SET deleted_at = NOW() WHERE username = ?",
        vec![reg.username.clone()],
    )
    .await
    .expect("软删账号落库");
    let err = svc.me(reg.user_id).await.expect_err("软删用户应 NotFound");
    assert!(matches!(err, AppError::NotFound), "应为 NotFound，实际 {err:?}");

    cleanup_user(&reg.username).await;
}
