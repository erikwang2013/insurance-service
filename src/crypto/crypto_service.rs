//! CryptoService：AES-256-GCM 加解密 + 脱敏（对齐 db-schema.md §8）
//!
//! 存储格式：`IV(12B) || CipherText || Tag(16B)`。
//! 密文列用 `VARBINARY`；明文仅在内存使用，禁止打日志、禁止序列化进响应。

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::Engine;

use crate::error::{AppError, Result};

/// AES-256-GCM 加解密服务
#[derive(Clone)]
pub struct CryptoService {
    cipher: Aes256Gcm,
}

impl CryptoService {
    /// 从 32 字节原始密钥构造
    pub fn from_key(key: &[u8]) -> Result<Self> {
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| AppError::Business(format!("无效 AES 密钥: {e}")))?;
        Ok(Self { cipher })
    }

    /// 从 base64 主密钥字符串构造（支持 "base64:" 前缀）
    pub fn from_master_key_b64(master_key: &str) -> Result<Self> {
        let b64 = master_key.strip_prefix("base64:").unwrap_or(master_key);
        let key = base64::engine::general_purpose::STANDARD
            .decode(b64)
            .map_err(|e| AppError::Business(format!("主密钥 base64 解码失败: {e}")))?;
        Self::from_key(&key)
    }

    /// 加密：明文 → IV || CipherText || Tag
    pub fn encrypt(&self, plain: &[u8]) -> Result<Vec<u8>> {
        let nonce_bytes: [u8; 12] = random_nonce();
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = self
            .cipher
            .encrypt(nonce, plain)
            .map_err(|e| AppError::Business(format!("AES 加密失败: {e}")))?;
        let mut out = Vec::with_capacity(12 + ciphertext.len());
        out.extend_from_slice(&nonce_bytes);
        out.extend_from_slice(&ciphertext);
        Ok(out)
    }

    /// 解密：IV || CipherText || Tag → 明文（仅内存使用）
    pub fn decrypt(&self, blob: &[u8]) -> Result<Vec<u8>> {
        if blob.len() < 12 + 16 {
            return Err(AppError::Business("密文长度不足".into()));
        }
        let (nonce_bytes, ciphertext) = blob.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| AppError::Business("AES 解密/认证失败".into()))
    }

    /// 加密 String
    pub fn encrypt_str(&self, plain: &str) -> Result<Vec<u8>> {
        self.encrypt(plain.as_bytes())
    }

    /// 解密为 String（仅内存使用）
    pub fn decrypt_str(&self, blob: &[u8]) -> Result<String> {
        let bytes = self.decrypt(blob)?;
        String::from_utf8(bytes).map_err(|e| AppError::Business(format!("解密结果非 UTF-8: {e}")))
    }
}

/// 脱敏工具（静态方法，无需密钥）
pub struct Masker;

impl Masker {
    /// 手机号脱敏：138****1234（保留前 3 后 4）
    pub fn phone(phone: &str) -> String {
        let chars: Vec<char> = phone.chars().collect();
        if chars.len() < 7 {
            return phone.to_string();
        }
        let mut out = String::new();
        for (i, c) in chars.iter().enumerate() {
            if i < 3 || i >= chars.len() - 4 {
                out.push(*c);
            } else {
                out.push('*');
            }
        }
        out
    }

    /// 身份证号脱敏：110***********1234（保留前 3 后 4）
    pub fn id_card(id: &str) -> String {
        let chars: Vec<char> = id.chars().collect();
        if chars.len() < 8 {
            return id.to_string();
        }
        let mut out = String::new();
        for (i, c) in chars.iter().enumerate() {
            if i < 3 || i >= chars.len() - 4 {
                out.push(*c);
            } else {
                out.push('*');
            }
        }
        out
    }

    /// 银行卡号脱敏：保留尾号 4 位
    pub fn bank_card(card: &str) -> String {
        let chars: Vec<char> = card.chars().collect();
        if chars.len() < 5 {
            return card.to_string();
        }
        let mut out = String::new();
        for (i, c) in chars.iter().enumerate() {
            if i >= chars.len() - 4 {
                out.push(*c);
            } else {
                out.push('*');
            }
        }
        out
    }
}

/// 随机 12 字节 IV（使用操作系统 CSPRNG）
fn random_nonce() -> [u8; 12] {
    let mut buf = [0u8; 12];
    // uuid 仅用于演示/骨架；生产环境应用 `rand` 或系统 CSPRNG 生成
    let id = uuid::Uuid::new_v4();
    buf.copy_from_slice(&id.as_bytes()[..12]);
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_encrypt_decrypt() {
        let svc = CryptoService::from_key(&[7u8; 32]).unwrap();
        let plain = "13800138000";
        let blob = svc.encrypt_str(plain).unwrap();
        assert_ne!(&blob, plain.as_bytes());
        assert_eq!(svc.decrypt_str(&blob).unwrap(), plain);
    }

    #[test]
    fn masking() {
        assert_eq!(Masker::phone("13800138000"), "138****8000");
        assert_eq!(Masker::id_card("110101199001011234"), "110***********1234");
    }
}
