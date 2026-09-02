//! 参数校验：身份证 / 手机号 / 金额（backend-architecture.md §2 utils/validator.rs）

use crate::error::{AppError, Result};

/// 中国大陆手机号（11 位，1 开头）
pub fn is_valid_phone(phone: &str) -> bool {
    let bytes = phone.as_bytes();
    bytes.len() == 11
        && bytes[0] == b'1'
        && bytes[1] >= b'3'
        && bytes[1] <= b'9'
        && bytes.iter().all(|b| b.is_ascii_digit())
}

/// 中国大陆身份证号（18 位，含校验位的宽松校验）
pub fn is_valid_id_card(id: &str) -> bool {
    let b = id.as_bytes();
    if b.len() != 18 {
        return false;
    }
    // 前 17 位数字
    if !b[..17].iter().all(|c| c.is_ascii_digit()) {
        return false;
    }
    // 末位为数字或 X/x
    let last = b[17];
    last.is_ascii_digit() || last == b'X' || last == b'x'
}

/// 金额校验：必须 > 0 且保留两位小数语义（rust_decimal）
pub fn is_valid_amount(amount: rust_decimal::Decimal) -> bool {
    amount > rust_decimal::Decimal::ZERO && !amount.is_sign_negative()
}

/// 校验手机号，失败返回 Validation 错误
pub fn check_phone(phone: &str) -> Result<()> {
    if is_valid_phone(phone) {
        Ok(())
    } else {
        Err(AppError::validation(format!("手机号非法: {phone}")))
    }
}

/// 校验身份证，失败返回 Validation 错误
pub fn check_id_card(id: &str) -> Result<()> {
    if is_valid_id_card(id) {
        Ok(())
    } else {
        Err(AppError::validation("身份证号非法".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phone() {
        assert!(is_valid_phone("13800138000"));
        assert!(!is_valid_phone("23800138000"));
        assert!(!is_valid_phone("1380013800"));
    }

    #[test]
    fn id_card() {
        assert!(is_valid_id_card("110101199001011234"));
        assert!(!is_valid_id_card("11010119900101123"));
        assert!(!is_valid_id_card("11010119900101123a"));
    }
}
