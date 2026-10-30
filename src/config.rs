//! 应用配置（对齐 backend-architecture.md §12.1）
//!
//! 说明：规划文档使用 `bee_config #[derive(Config)]`；当前仍采用「手写 env 读取」
//! 实现等价配置结构（bee_config 快照已随 vendor 提供，后续可切换），字段名与
//! `.env.example` / `config/app.toml` 严格对齐。

use serde::Deserialize;

/// 服务端监听配置
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

/// MySQL 数据库配置
#[derive(Debug, Clone, Deserialize)]
pub struct DbConfig {
    pub url: String,
}

/// JWT 配置
#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub issuer: String,
    /// access token 有效期（秒）
    pub access_expiry: i64,
    /// refresh token 有效期（秒）
    pub refresh_expiry: i64,
}

/// AES 加密主密钥配置
#[derive(Debug, Clone, Deserialize)]
pub struct CryptoConfig {
    /// 32 字节主密钥（base64 或原始 bytes），见 .env.example CRYPTO_MASTER_KEY
    pub master_key: String,
}

/// 日志配置
#[derive(Debug, Clone, Deserialize)]
pub struct LogConfig {
    pub level: String,
}

/// 微信小程序登录渠道配置（code2session）
///
/// 凭据（WECHAT_APPID / WECHAT_SECRET）注入即激活；任一为空视为未配置，
/// 渠道客户端不会发起任何网络请求（见 `providers/wechat.rs`）。
#[derive(Debug, Clone, Deserialize)]
pub struct WechatConfig {
    pub app_id: String,
    pub app_secret: String,
}

/// 支付回调配置
///
/// `callback_token` 为空表示回调通道未启用（服务端拒绝回调请求）。
#[derive(Debug, Clone, Deserialize)]
pub struct PaymentConfig {
    pub callback_token: String,
}

/// 应用总配置
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DbConfig,
    pub jwt: JwtConfig,
    pub crypto: CryptoConfig,
    pub log: LogConfig,
    pub wechat: WechatConfig,
    pub payment: PaymentConfig,
}

/// 从环境变量读取配置（手写实现，替代 bee_config）
///
/// 读取失败（缺关键变量）返回 `Err(String)`，由 main 打印并退出。
impl AppConfig {
    pub fn from_env() -> Result<Self, String> {
        let server = ServerConfig {
            host: env_or("SERVER_HOST", "0.0.0.0"),
            port: env_or("SERVER_PORT", "8080")
                .parse()
                .map_err(|e| format!("SERVER_PORT 解析失败: {e}"))?,
        };
        let database = DbConfig {
            url: env_required("DATABASE_URL")?,
        };
        // JWT 密钥必须显式提供且足够长：弱默认值会让任何人伪造令牌
        let secret = env_required("JWT_SECRET")?;
        validate_jwt_secret(&secret)?;
        let jwt = JwtConfig {
            secret,
            issuer: env_or("JWT_ISSUER", "insurance-service"),
            access_expiry: env_or("JWT_ACCESS_EXPIRY", "7200")
                .parse()
                .map_err(|e| format!("JWT_ACCESS_EXPIRY 解析失败: {e}"))?,
            refresh_expiry: env_or("JWT_REFRESH_EXPIRY", "604800")
                .parse()
                .map_err(|e| format!("JWT_REFRESH_EXPIRY 解析失败: {e}"))?,
        };
        let crypto = CryptoConfig {
            master_key: env_or("CRYPTO_MASTER_KEY", ""),
        };
        let log = LogConfig {
            level: env_or("RUST_LOG", "info"),
        };
        let wechat = WechatConfig {
            app_id: env_or("WECHAT_APPID", ""),
            app_secret: env_or("WECHAT_SECRET", ""),
        };
        let payment = PaymentConfig {
            callback_token: env_or("PAYMENT_CALLBACK_TOKEN", ""),
        };
        Ok(AppConfig {
            server,
            database,
            jwt,
            crypto,
            log,
            wechat,
            payment,
        })
    }

    /// 服务器监听地址
    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}

/// JWT 密钥校验：拒绝示例配置中的占位符，以及不足 32 字节的短密钥
/// （二者都会让任何人都能伪造令牌）。
fn validate_jwt_secret(secret: &str) -> Result<(), String> {
    const PLACEHOLDERS: [&str; 1] = ["change-me-to-a-long-random-string"];
    if PLACEHOLDERS.contains(&secret) {
        return Err(
            "JWT_SECRET 仍是示例占位符，请用 `openssl rand -base64 48` 生成随机密钥".to_string(),
        );
    }
    if secret.len() < 32 {
        return Err(format!(
            "JWT_SECRET 长度不足 32 字节（当前 {} 字节），请配置足够长的随机密钥",
            secret.len()
        ));
    }
    Ok(())
}

fn env_required(key: &str) -> Result<String, String> {
    std::env::var(key).map_err(|_| format!("缺少必需环境变量 {key}（见 .env.example）"))
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

/// 日志用 DSN：把 `://` 与最后一个 `@` 之间的凭据段（`user` 或 `user:password`）
/// 整体替换为 `***`；无 `://` 的 `user:pass@host` 形式同样脱敏。
///
/// 无凭据段或非标准 DSN 原样返回（不猜测、不丢信息）。
pub fn redact_db_url(url: &str) -> String {
    let Some((scheme, rest)) = url.split_once("://") else {
        // 无 scheme 的 DSN（如 `user:pass@host/db`）：整段凭据替换为 ***
        return match url.rsplit_once('@') {
            Some((credentials, host)) if !credentials.is_empty() && !host.is_empty() => {
                format!("***@{host}")
            }
            _ => url.to_string(),
        };
    };
    let Some((_credentials, host)) = rest.rsplit_once('@') else {
        return url.to_string();
    };
    format!("{scheme}://***@{host}")
}

#[cfg(test)]
mod tests {
    use super::{redact_db_url, validate_jwt_secret};

    #[test]
    fn redacts_credentials_in_db_url() {
        assert_eq!(
            redact_db_url("mysql://alice:s3cret@db.local:3306/insurance"),
            "mysql://***@db.local:3306/insurance"
        );
        // 密码自身含 @ 时以最后一个 @ 为分界
        assert_eq!(
            redact_db_url("mysql://alice:p@ss@db.local/insurance"),
            "mysql://***@db.local/insurance"
        );
        assert_eq!(
            redact_db_url("mysql://alice@db.local/insurance"),
            "mysql://***@db.local/insurance"
        );
        // 无 `://` 的 DSN 形式同样脱敏
        assert_eq!(
            redact_db_url("alice:s3cret@db.local:3306/insurance"),
            "***@db.local:3306/insurance"
        );
        assert_eq!(
            redact_db_url("alice@db.local/insurance"),
            "***@db.local/insurance"
        );
    }

    #[test]
    fn rejects_known_placeholder_and_short_jwt_secrets() {
        // .env.example 中的示例值虽然够长，但公开可猜，必须拒绝
        assert!(validate_jwt_secret("change-me-to-a-long-random-string").is_err());
        assert!(validate_jwt_secret("short").is_err());
        assert!(validate_jwt_secret(&"a".repeat(31)).is_err());
    }

    #[test]
    fn accepts_a_long_random_jwt_secret() {
        assert!(validate_jwt_secret("A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8S9t0").is_ok());
    }

    #[test]
    fn leaves_urls_without_credentials_unchanged() {
        assert_eq!(
            redact_db_url("mysql://db.local:3306/insurance"),
            "mysql://db.local:3306/insurance"
        );
        assert_eq!(redact_db_url("not-a-url"), "not-a-url");
    }
}
