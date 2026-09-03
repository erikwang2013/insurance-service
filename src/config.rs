//! 应用配置（对齐 backend-architecture.md §12.1）
//!
//! 说明：规划文档使用 `bee_config #[derive(Config)]`，但 bee-rust（含 bee_config）当前
//! 无法在编译环境拉取。阶段 0 采用「手写 env 读取」实现等价配置结构，字段名与
//! `.env.example` / `config/app.toml` 严格对齐，后续可平滑替换为 bee_config derive。

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

/// Redis 配置
#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

/// OpenSearch 搜索配置
#[derive(Debug, Clone, Deserialize)]
pub struct SearchConfig {
    pub url: String,
    pub username: String,
    pub password: String,
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

/// 应用总配置
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DbConfig,
    pub redis: RedisConfig,
    pub opensearch: SearchConfig,
    pub jwt: JwtConfig,
    pub crypto: CryptoConfig,
    pub log: LogConfig,
    pub wechat: WechatConfig,
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
        let redis = RedisConfig {
            url: env_or("REDIS_URL", "redis://127.0.0.1:6379"),
        };
        let opensearch = SearchConfig {
            url: env_or("OPENSEARCH_URL", "http://127.0.0.1:9200"),
            username: env_or("OPENSEARCH_USERNAME", "admin"),
            password: env_or("OPENSEARCH_PASSWORD", "changeme"),
        };
        let jwt = JwtConfig {
            secret: env_or("JWT_SECRET", "change-me-to-a-long-random-string"),
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
        Ok(AppConfig {
            server,
            database,
            redis,
            opensearch,
            jwt,
            crypto,
            log,
            wechat,
        })
    }

    /// 服务器监听地址
    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}

fn env_required(key: &str) -> Result<String, String> {
    std::env::var(key).map_err(|_| format!("缺少必需环境变量 {key}（见 .env.example）"))
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}
