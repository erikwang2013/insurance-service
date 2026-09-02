//! MockPayProvider（对齐 backend-architecture.md §8.2）

use async_trait::async_trait;
use rust_decimal::Decimal;

use super::pay_provider::{CallbackResult, PayProvider, PayStatus, PrepayResult};
use crate::error::{AppError, Result};
use crate::models::Order;

/// 模拟支付渠道
pub struct MockPayProvider;

#[async_trait]
impl PayProvider for MockPayProvider {
    fn name(&self) -> &'static str {
        "MOCK"
    }

    async fn create_payment(&self, order: &Order, _amount: Decimal) -> Result<PrepayResult> {
        let tx_id = format!("MOCK-{}-{}", order.order_no, uuid::Uuid::new_v4());
        Ok(PrepayResult {
            provider_tx_id: tx_id.clone(),
            pay_params: serde_json::json!({ "mock_url": format!("/pay/mock/{tx_id}") }),
        })
    }

    async fn query_status(&self, _provider_tx_id: &str) -> Result<PayStatus> {
        // 简单模拟：仅当后端主动调用 payments/{orderId}/pay 时才置为成功
        Ok(PayStatus::Pending)
    }

    async fn handle_callback(&self, _provider: &str, payload: &[u8]) -> Result<CallbackResult> {
        let raw: serde_json::Value =
            serde_json::from_slice(payload).map_err(|e| AppError::Business(e.to_string()))?;
        Ok(CallbackResult {
            provider_tx_id: raw["tx_id"].as_str().unwrap_or_default().to_string(),
            status: PayStatus::Success,
            raw_payload: raw,
        })
    }
}
