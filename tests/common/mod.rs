//! 集成测试共享辅助（任务 #5）
//!
//! - `test_db()`：按 `DATABASE_URL`（默认本地 insurance_service）建池并 ping；
//!   MySQL 不可用/未建库时返回 `None`，调用方打印 SKIP 后提前返回。
//! - `unique(prefix)`：生成带时间戳+计数器的唯一用户名/产品编码（VARCHAR(64) 内），
//!   避免并行测试互相冲突；测试结束用 `delete_user` / `delete_product` 清理。

use std::sync::atomic::{AtomicU64, Ordering};

use insurance_service::config::DbConfig;
use insurance_service::db::Db;
use mysql_async::prelude::Queryable;

/// 连接测试库；连接失败（MySQL 未启动 / 未执行 install.sql）返回 `None`。
pub async fn test_db() -> Option<Db> {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://root:@127.0.0.1:3306/insurance_service".to_string());
    let db = Db::new(&DbConfig { url }).ok()?;
    let mut conn = db.conn().await.ok()?;
    conn.query_drop("SELECT 1").await.ok()?;
    Some(db)
}

/// 生成唯一名称（前缀 + 纳秒时间戳 + 进程内计数器）。
pub fn unique(prefix: &str) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{prefix}_{nanos}_{n}")
}

/// 删除测试用户（按唯一用户名，仅命中本次测试插入的行）。
pub async fn delete_user(db: &Db, username: &str) {
    let _ = db
        .exec_drop("DELETE FROM users WHERE username = ?", vec![username])
        .await;
}

/// 删除测试产品（按唯一产品编码，仅命中本次测试插入的行）。
pub async fn delete_product(db: &Db, product_code: &str) {
    let _ = db
        .exec_drop(
            "DELETE FROM insurance_products WHERE product_code = ?",
            vec![product_code],
        )
        .await;
}

/// 测试专用 JWT 配置（固定密钥/签发者，过期时间可调）。
pub fn jwt_cfg(access_expiry: i64) -> insurance_service::config::JwtConfig {
    insurance_service::config::JwtConfig {
        secret: "integration-test-secret-0123456789".to_string(),
        issuer: "insurance-service".to_string(),
        access_expiry,
        refresh_expiry: 604800,
    }
}

/// 测试专用 CryptoService（32 字节固定密钥）。
pub fn crypto() -> insurance_service::crypto::CryptoService {
    insurance_service::crypto::CryptoService::from_key(&[7u8; 32]).expect("固定密钥构造")
}
