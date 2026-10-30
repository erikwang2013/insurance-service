//! 令牌生命周期（任务 2.7）：access / refresh 类型隔离、expires_in 与版本携带。
//!
//! 使用公开构造入口（Pool 惰性建连，issue_tokens 为纯签发，不触网不连库）。

use insurance_service::config::{DbConfig, JwtConfig, WechatConfig};
use insurance_service::crypto::CryptoService;
use insurance_service::db::Db;
use insurance_service::middleware::auth::{JwtService, Role};
use insurance_service::providers::wechat::WechatClient;
use insurance_service::services::auth_service::AuthService;

fn jwt_cfg() -> JwtConfig {
    JwtConfig {
        // 配置规则要求 >= 32 字节
        secret: "0123456789abcdef0123456789abcdef".into(),
        issuer: "insurance-test".into(),
        access_expiry: 100,
        refresh_expiry: 1000,
    }
}

fn service() -> AuthService {
    AuthService::new(
        jwt_cfg(),
        CryptoService::from_master_key_b64(
            "base64:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
        )
        .unwrap(),
        Db::new(&DbConfig {
            url: "mysql://u:p@127.0.0.1:3306/test".into(),
        })
        .unwrap(),
        WechatClient::from_config(WechatConfig {
            app_id: String::new(),
            app_secret: String::new(),
        }),
    )
}

#[test]
fn issue_tokens_returns_typed_pair_and_access_expiry() {
    let result = service().issue_tokens(7, "alice", Role::User, 3).unwrap();
    assert_eq!(result.tokens.expires_in, 100, "expires_in 取 access 有效期");
    assert_ne!(result.tokens.access_token, result.tokens.refresh_token);

    let jwt = JwtService::new(jwt_cfg());
    let access = jwt.verify_access(&result.tokens.access_token).unwrap();
    assert_eq!(access.typ, "access");
    assert_eq!(access.token_version, 3);
    assert_eq!(access.exp - access.iat, 100);

    let refresh = jwt.verify_refresh(&result.tokens.refresh_token).unwrap();
    assert_eq!(refresh.typ, "refresh");
    assert_eq!(refresh.exp - refresh.iat, 1000);

    assert!(jwt.verify_refresh(&result.tokens.access_token).is_err());
    assert!(jwt.verify_access(&result.tokens.refresh_token).is_err());
}
