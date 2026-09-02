//! PayProvider 抽象（对齐 backend-architecture.md §8）

use std::collections::HashMap;

use async_trait::async_trait;
use rust_decimal::Decimal;

use crate::error::{AppError, Result};
use crate::models::Order;

/// 预支付结果（存 payments.prepay_payload）
pub struct PrepayResult {
    pub provider_tx_id: String,
    /// 前端拉起收银台所需参数
    pub pay_params: serde_json::Value,
}

/// 支付状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PayStatus {
    Success,
    Failed,
    Pending,
    Refunded,
}

/// 渠道异步回调结果
pub struct CallbackResult {
    pub provider_tx_id: String,
    pub status: PayStatus,
    /// 原文留痕（payments.callback_payload）
    pub raw_payload: serde_json::Value,
}

/// 支付渠道抽象接口
#[async_trait]
pub trait PayProvider: Send + Sync {
    /// 渠道名："MOCK" | "WECHAT"
    fn name(&self) -> &'static str;

    /// 创建预支付，返回可拉起的收银台参数（存 payments.prepay_payload）
    async fn create_payment(&self, order: &Order, amount: Decimal) -> Result<PrepayResult>;

    /// 主动查询支付状态（兜底/对账）
    async fn query_status(&self, provider_tx_id: &str) -> Result<PayStatus>;

    /// 处理渠道异步回调报文，验签后返回结果
    async fn handle_callback(&self, provider: &str, payload: &[u8]) -> Result<CallbackResult>;
}

/// 渠道注册与分发（backend-architecture.md §8.3）
pub struct PaymentProviderRegistry {
    providers: HashMap<&'static str, Box<dyn PayProvider>>,
}

impl PaymentProviderRegistry {
    pub fn new() -> Self {
        let mut m: HashMap<&'static str, Box<dyn PayProvider>> = HashMap::new();
        m.insert(
            "MOCK",
            Box::new(crate::providers::payment::MockPayProvider) as Box<dyn PayProvider>,
        );
        // 预留: m.insert("WECHAT", Box::new(WechatPayProvider::new(cfg)));
        Self { providers: m }
    }

    pub fn get(&self, name: &str) -> Result<&dyn PayProvider> {
        self.providers
            .get(name)
            .map(|p| p.as_ref())
            .ok_or_else(|| AppError::Payment(format!("未知渠道 {name}")))
    }
}

impl Default for PaymentProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
