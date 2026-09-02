//! 业务号生成：订单号 / 保单号 / 合同号（对齐 db-schema.md §1 业务号独立生成唯一索引）

use chrono::Utc;

/// 生成带业务前缀 + 时间戳 + 随机尾缀的编号
///
/// 格式：`{prefix}{yyyymmddHHMMSS}{随机6位}`
/// 示例：`P20260901123000AB12CD`（前缀 P=保单）
fn generate(prefix: &str) -> String {
    let ts = Utc::now().format("%Y%m%d%H%M%S");
    let rand = uuid::Uuid::new_v4();
    // 取 UUID 十六进制前 6 位作为随机尾缀
    let rand_hex: String = rand.simple().to_string();
    format!("{prefix}{ts}{}", &rand_hex[..6])
}

/// 订单号：`O` 前缀
pub fn order_no() -> String {
    generate("O")
}

/// 保单号：`P` 前缀
pub fn policy_no() -> String {
    generate("P")
}

/// 合同号：`C` 前缀
pub fn contract_no() -> String {
    generate("C")
}

/// 报价单号：`Q` 前缀
pub fn quote_no() -> String {
    generate("Q")
}

/// 理赔单号：`CL` 前缀
pub fn claim_no() -> String {
    generate("CL")
}

/// 支付流水号：`PAY` 前缀
pub fn payment_no() -> String {
    generate("PAY")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_and_uniqueness() {
        assert!(order_no().starts_with('O'));
        assert!(policy_no().starts_with('P'));
        assert!(contract_no().starts_with('C'));
        // 两次生成不同
        assert_ne!(order_no(), order_no());
    }
}
