//! C1 认证扩展集成测试（任务 #21，MySQL 集成）
//!
//! 覆盖：微信绑定闭环（绑定 → 微信登录直登；未绑定提示；未配置降级保留）、
//! refresh 令牌撤销轮换（logout / 修改密码后旧 refresh 失效、重新登录可用）。
//!
//! 依赖 `insurance_service` 库 + 本地 MySQL（install.sql 建库 + C1 的 ALTER：
//! users 增 openid / unionid / token_version 列）。MySQL 不可用时 SKIP。
//!
//! 微信 code2session 以 `FakeSessionProvider` 注入（测试不配置、也不允许
//! 触达真实微信接口），固定返回预置 openid —— 见 `SessionProvider`。

mod common;

use async_trait::async_trait;

use insurance_service::error::AppError;
use insurance_service::providers::wechat::WechatSession;
use insurance_service::services::auth_service::{
    AuthService, RegisterReq, SessionProvider, WechatLoginReq,
};

/// 假微信会话提供者：code2session 固定返回预置 openid（不触达真实微信接口）。
struct FakeSessionProvider {
    openid: String,
}

#[async_trait]
impl SessionProvider for FakeSessionProvider {
    async fn code2session(&self, _code: &str) -> Result<WechatSession, AppError> {
        Ok(WechatSession {
            openid: self.openid.clone(),
            session_key: "fake-session-key".to_string(),
            unionid: None,
        })
    }
}

/// 构造 AuthService（测试固定密钥/配置 + 测试库连接 + 假微信会话提供者）
async fn service() -> Option<AuthService> {
    let db = common::test_db().await?;
    let wechat = Box::new(FakeSessionProvider {
        // openid 唯一（wx_ 前缀 + 随机串），避免并行用例相互踩 UNIQUE 索引
        openid: common::unique("wx"),
    }) as Box<dyn SessionProvider>;
    Some(AuthService::new_with_provider(
        common::jwt_cfg(3600),
        common::crypto(),
        db,
        wechat,
    ))
}

/// 清理辅助：独立连接池，仅用于删除测试行。
async fn cleanup_db() -> Option<insurance_service::db::Db> {
    common::test_db().await
}

/// 注册一个测试用户（固定口令，便于后续登录/改密）
async fn register_user(
    svc: &AuthService,
    username: &str,
) -> insurance_service::services::auth_service::LoginResult {
    svc.register(RegisterReq {
        username: username.to_string(),
        password: "P@ssw0rd!".to_string(),
        phone: "13800138000".to_string(),
    })
    .await
    .expect("注册成功")
}

/// 微信绑定 → 微信登录命中并签发双令牌
#[tokio::test]
async fn bind_then_wechat_login_succeeds() {
    let Some(svc) = service().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let username = common::unique("tester");
    let registered = register_user(&svc, &username).await;

    // 直接经服务层绑定：绑定所需 code 任意（fake provider 不看 code）
    svc.bind_wechat(registered.user_id, "wx-code-any")
        .await
        .expect("绑定成功");

    // 微信登录（同一 fake 固定 openid）→ 命中绑定账号，签发双令牌
    let result = svc
        .wechat_login(WechatLoginReq {
            code: "wx-code-any".to_string(),
        })
        .await
        .expect("微信登录成功");
    assert_eq!(result.user_id, registered.user_id);
    assert_eq!(result.username, username);
    assert!(!result.tokens.access_token.is_empty());
    assert!(!result.tokens.refresh_token.is_empty());
    if let Some(db) = cleanup_db().await {
        common::delete_user(&db, &username).await;
    }
}

/// 未绑定微信的 openid → 提示先登录绑定（业务错误，不自动建号）
#[tokio::test]
async fn wechat_login_unbound_openid_rejected() {
    let Some(svc) = service().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    // fake provider 固定返回一个从未绑定过的 openid（wx_ 前缀，绝无残留）
    let err = svc
        .wechat_login(WechatLoginReq {
            code: "wx-code-any".to_string(),
        })
        .await
        .expect_err("未绑定 openid 应拒绝");
    match err {
        AppError::Business(msg) => assert!(msg.contains("未绑定"), "msg={msg}"),
        other => panic!("预期 Business 错误，得到 {other:?}"),
    }
}

/// 微信绑定已归属他人 → 状态冲突
#[tokio::test]
async fn bind_wechat_openid_conflict_rejected() {
    let Some(svc) = service().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let username_a = common::unique("tester");
    let username_b = common::unique("tester");
    let user_a = register_user(&svc, &username_a).await;

    // A 绑定该 openid 后，B 再绑同 openid → 409 状态冲突
    svc.bind_wechat(user_a.user_id, "wx-code-any")
        .await
        .expect("A 绑定成功");
    let user_b = register_user(&svc, &username_b).await;
    let err = svc
        .bind_wechat(user_b.user_id, "wx-code-any")
        .await
        .expect_err("同 openid 二次绑定应冲突");
    match err {
        AppError::StateConflict(msg) => assert!(msg.contains("已绑定其他账号"), "msg={msg}"),
        other => panic!("预期 StateConflict 错误，得到 {other:?}"),
    }
    if let Some(db) = cleanup_db().await {
        common::delete_user(&db, &username_a).await;
        common::delete_user(&db, &username_b).await;
    }
}

/// 微信登录未配置凭据 → 保持「未配置」业务错误降级（回归保护，同既有测试语义）
#[tokio::test]
async fn wechat_login_unconfigured_degrades() {
    let Some(db) = common::test_db().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    // 生产构造路径：空凭据 WechatClient（common::wechat_client）经四参 new() 进入，
    // 与 tests/auth_service_test.rs 的构造一致 —— 保证未配置降级不被测试 seam 改变
    let svc = AuthService::new(
        common::jwt_cfg(3600),
        common::crypto(),
        db,
        common::wechat_client(),
    );
    let err = svc
        .wechat_login(WechatLoginReq {
            code: "wx-code-123".to_string(),
        })
        .await
        .expect_err("微信登录（未配置）应降级为业务错误");
    match err {
        AppError::Business(msg) => assert!(msg.contains("未配置"), "msg={msg}"),
        other => panic!("预期 Business 错误，得到 {other:?}"),
    }
}

/// logout 撤销：登出后旧 refresh 立即失效，重新登录签发的新 refresh 仍可用
#[tokio::test]
async fn logout_revokes_old_refresh_token() {
    let Some(svc) = service().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let username = common::unique("tester");
    let registered = register_user(&svc, &username).await;
    let old_refresh = registered.tokens.refresh_token.clone();

    // 登出前：旧 refresh 可正常轮换
    svc.refresh(&old_refresh).await.expect("登出前旧 refresh 可用");

    // 登出 → token_version +1
    svc.logout(registered.user_id).await.expect("登出成功");

    // 旧 refresh → 版本落后 → Unauthorized
    let err = svc
        .refresh(&old_refresh)
        .await
        .expect_err("登出后旧 refresh 应失效");
    assert!(
        matches!(err, AppError::Unauthorized),
        "预期 Unauthorized，得到 {err:?}"
    );

    // 重新登录 → 新 refresh 携带新版本，可正常轮换
    let again = svc
        .login(insurance_service::services::auth_service::LoginReq {
            username: username.clone(),
            password: "P@ssw0rd!".to_string(),
        })
        .await
        .expect("重新登录成功");
    svc.refresh(&again.tokens.refresh_token)
        .await
        .expect("新 refresh 可用");

    if let Some(db) = cleanup_db().await {
        common::delete_user(&db, &username).await;
    }
}

/// 修改密码撤销：改密后旧 refresh 立即失效，新登录 refresh 可用
#[tokio::test]
async fn change_password_revokes_old_refresh_token() {
    let Some(svc) = service().await else {
        eprintln!("SKIP: MySQL 不可用（需 DATABASE_URL + install.sql 建库）");
        return;
    };
    let username = common::unique("tester");
    let registered = register_user(&svc, &username).await;
    let old_refresh = registered.tokens.refresh_token.clone();

    // 改密（token_version +1）
    svc.change_password(registered.user_id, "P@ssw0rd!", "NewPass1!")
        .await
        .expect("改密成功");

    // 旧 refresh → Unauthorized
    let err = svc
        .refresh(&old_refresh)
        .await
        .expect_err("改密后旧 refresh 应失效");
    assert!(
        matches!(err, AppError::Unauthorized),
        "预期 Unauthorized，得到 {err:?}"
    );

    // 新口令登录 → 新 refresh 可用
    let again = svc
        .login(insurance_service::services::auth_service::LoginReq {
            username: username.clone(),
            password: "NewPass1!".to_string(),
        })
        .await
        .expect("新口令登录成功");
    svc.refresh(&again.tokens.refresh_token)
        .await
        .expect("新 refresh 可用");

    if let Some(db) = cleanup_db().await {
        common::delete_user(&db, &username).await;
    }
}
