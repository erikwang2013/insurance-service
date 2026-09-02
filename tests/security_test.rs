//! 安全/加密/认证集成测试（任务 #5，无 DB 依赖）
//!
//! 覆盖：CryptoService 加解密与防篡改、Masker 脱敏、argon2 密码哈希、
//! JwtService 签发/校验（含过期、密钥不符、签发者不符）、
//! AuthFilter 认证过滤器、RequireRoleFilter 角色守卫。

mod common;

use insurance_service::config::JwtConfig;
use insurance_service::crypto::{CryptoService, Masker};
use insurance_service::error::AppError;
use insurance_service::middleware::auth::{
    AuthFilter, AuthUser, JwtService, RequireRoleFilter, Role,
};
use insurance_service::middleware::{Filter, RequestCtx};
use insurance_service::services::auth_service::AuthService;

// ---------------------------------------------------------------------------
// CryptoService：加解密往返 + 防篡改
// ---------------------------------------------------------------------------

#[test]
fn crypto_roundtrip_encrypt_decrypt() {
    let crypto = common::crypto();
    let plain = "13800138000";
    let blob = crypto.encrypt_str(plain).expect("加密");
    // 密文 ≠ 明文，且长度 = 12(IV) + 明文 + 16(Tag)
    assert_ne!(blob, plain.as_bytes());
    assert_eq!(blob.len(), 12 + plain.len() + 16);
    assert_eq!(crypto.decrypt_str(&blob).expect("解密"), plain);
}

#[test]
fn crypto_wrong_key_fails() {
    let crypto = common::crypto();
    let other = CryptoService::from_key(&[8u8; 32]).expect("另一密钥");
    let blob = crypto.encrypt_str("secret-data").expect("加密");
    assert!(other.decrypt(&blob).is_err());
}

#[test]
fn crypto_tampered_ciphertext_rejected() {
    let crypto = common::crypto();
    let mut blob = crypto.encrypt_str("integrity-check").expect("加密");
    // 翻转密文区（IV 之后）的一个字节，解密必须失败
    let idx = 12 + 2;
    blob[idx] ^= 0x01;
    assert!(crypto.decrypt(&blob).is_err());
}

#[test]
fn crypto_short_blob_rejected() {
    let crypto = common::crypto();
    assert!(crypto.decrypt(&[0u8; 5]).is_err());
}

// ---------------------------------------------------------------------------
// Masker：脱敏
// ---------------------------------------------------------------------------

#[test]
fn masker_phone_id_card_bank_card() {
    assert_eq!(Masker::phone("13800138000"), "138****8000");
    assert_eq!(Masker::id_card("110101199001011234"), "110***********1234");
    assert_eq!(Masker::bank_card("6222021234567890"), "************7890");
    // 过短输入原样返回
    assert_eq!(Masker::phone("123"), "123");
    assert_eq!(Masker::bank_card("1234"), "1234");
}

// ---------------------------------------------------------------------------
// argon2 密码哈希
// ---------------------------------------------------------------------------

#[test]
fn password_hash_verify() {
    let hash = AuthService::hash_password("P@ssw0rd!").expect("哈希");
    assert_ne!(hash, "P@ssw0rd!");
    assert!(AuthService::verify_password("P@ssw0rd!", &hash));
    assert!(!AuthService::verify_password("wrong-password", &hash));
}

#[test]
fn password_hash_verify_invalid_hash() {
    assert!(!AuthService::verify_password("any", "not-a-valid-hash"));
}

// ---------------------------------------------------------------------------
// JwtService：签发 / 校验
// ---------------------------------------------------------------------------

#[test]
fn jwt_issue_verify_roundtrip() {
    let jwt = JwtService::new(common::jwt_cfg(3600));
    let token = jwt
        .issue_access_token(42, Role::Admin, Some("flutter".into()))
        .expect("签发");
    let claims = jwt.verify_token(&token).expect("校验");
    assert_eq!(claims.sub, 42);
    assert_eq!(claims.role, "ADMIN");
    assert_eq!(claims.platform.as_deref(), Some("flutter"));
    assert_eq!(claims.iss, "insurance-service");
}

#[test]
fn jwt_expired_token_rejected() {
    // access_expiry 为负 → exp 已过
    let jwt = JwtService::new(common::jwt_cfg(-10));
    let token = jwt
        .issue_access_token(1, Role::User, None)
        .expect("签发");
    assert!(matches!(
        jwt.verify_token(&token),
        Err(AppError::Unauthorized)
    ));
}

#[test]
fn jwt_wrong_secret_rejected() {
    let jwt_a = JwtService::new(common::jwt_cfg(3600));
    let jwt_b = JwtService::new(JwtConfig {
        secret: "another-secret-0123456789".to_string(),
        ..common::jwt_cfg(3600)
    });
    let token = jwt_a
        .issue_access_token(1, Role::User, None)
        .expect("签发");
    assert!(matches!(
        jwt_b.verify_token(&token),
        Err(AppError::Unauthorized)
    ));
}

#[test]
fn jwt_wrong_issuer_rejected() {
    let jwt_a = JwtService::new(common::jwt_cfg(3600));
    let jwt_b = JwtService::new(JwtConfig {
        issuer: "another-service".to_string(),
        ..common::jwt_cfg(3600)
    });
    let token = jwt_a
        .issue_access_token(1, Role::User, None)
        .expect("签发");
    assert!(matches!(
        jwt_b.verify_token(&token),
        Err(AppError::Unauthorized)
    ));
}

// ---------------------------------------------------------------------------
// AuthFilter：认证过滤器
// ---------------------------------------------------------------------------

fn auth_ctx_with(token: Option<String>) -> RequestCtx {
    let mut ctx = RequestCtx::default();
    ctx.auth_header = token;
    ctx
}

#[test]
fn auth_filter_accepts_bearer_token() {
    let jwt = JwtService::new(common::jwt_cfg(3600));
    let token = jwt
        .issue_access_token(7, Role::Operator, None)
        .expect("签发");
    let filter = AuthFilter::new(common::jwt_cfg(3600));
    let mut ctx = auth_ctx_with(Some(format!("Bearer {token}")));
    assert!(filter.before(&mut ctx).is_ok());
    let user = ctx.current_user.expect("注入 current_user");
    assert_eq!(user.id, 7);
    assert_eq!(user.role, Role::Operator);
}

#[test]
fn auth_filter_accepts_bare_token() {
    let jwt = JwtService::new(common::jwt_cfg(3600));
    let token = jwt
        .issue_access_token(1, Role::User, None)
        .expect("签发");
    let filter = AuthFilter::new(common::jwt_cfg(3600));
    let mut ctx = auth_ctx_with(Some(token));
    assert!(filter.before(&mut ctx).is_ok());
}

#[test]
fn auth_filter_rejects_missing_token() {
    let filter = AuthFilter::new(common::jwt_cfg(3600));
    let mut ctx = auth_ctx_with(None);
    assert!(filter.before(&mut ctx).is_err());
    assert!(ctx.current_user.is_none());
}

#[test]
fn auth_filter_rejects_garbage_token() {
    let filter = AuthFilter::new(common::jwt_cfg(3600));
    let mut ctx = auth_ctx_with(Some("Bearer not.a.jwt".to_string()));
    assert!(filter.before(&mut ctx).is_err());
}

// ---------------------------------------------------------------------------
// RequireRoleFilter：角色守卫
// ---------------------------------------------------------------------------

#[test]
fn require_role_allows_member() {
    let filter = RequireRoleFilter::new(&[Role::Admin]);
    let mut ctx = RequestCtx::default();
    ctx.current_user = Some(AuthUser {
        id: 1,
        role: Role::Admin,
        platform: None,
    });
    assert!(filter.before(&mut ctx).is_ok());
}

#[test]
fn require_role_denies_outsider() {
    let filter = RequireRoleFilter::new(&[Role::Admin]);
    let mut ctx = RequestCtx::default();
    ctx.current_user = Some(AuthUser {
        id: 2,
        role: Role::User,
        platform: None,
    });
    assert!(filter.before(&mut ctx).is_err());
}

#[test]
fn require_role_denies_unauthenticated() {
    let filter = RequireRoleFilter::new(&[Role::Admin]);
    let mut ctx = RequestCtx::default();
    assert!(filter.before(&mut ctx).is_err());
}
