//! 微信 code2session 渠道集成测试（任务 B，可配置骨架）
//!
//! 微信真实对接属「需外部凭据」路径（WECHAT_APPID / WECHAT_SECRET 注入即激活），
//! 无凭据环境无法真实验证网络路径。因此本文件仅覆盖**可真实通过的路径**：
//! - 「未配置分支」：空凭据构造 client → code2session 返回业务错误（不发网络请求）
//! - errcode/成功响应解析分支已由 `src/providers/wechat.rs` 内单测覆盖（纯逻辑）
//!
//! 其余路径（凭据注入后的真实 code2session 调用、HTTP/网络异常）无法在
//! 无凭据/无外网环境验证，需凭据注入后手工验证——如实注明。

use insurance_service::config::WechatConfig;
use insurance_service::error::AppError;
use insurance_service::providers::wechat::WechatClient;

/// 未配置（app_id 为空）→ 业务错误，且不发起网络请求
#[tokio::test]
async fn unconfigured_app_id_returns_business_error() {
    let client = WechatClient::from_config(WechatConfig {
        app_id: String::new(),
        app_secret: "secret".into(),
    });
    let err = client.code2session("test-code").await.unwrap_err();
    assert!(matches!(err, AppError::Business(_)));
    let msg = err.to_string();
    assert!(msg.contains("未配置"), "应提示未配置,实际: {msg}");
    assert!(msg.contains("WECHAT_APPID"), "应提示缺哪个环境变量,实际: {msg}");
}

/// 未配置（app_secret 为空）→ 同样业务错误
#[tokio::test]
async fn unconfigured_app_secret_returns_business_error() {
    let client = WechatClient::from_config(WechatConfig {
        app_id: "appid".into(),
        app_secret: String::new(),
    });
    let err = client.code2session("test-code").await.unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("未配置"), "应提示未配置,实际: {msg}");
}
