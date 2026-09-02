//! WechatPayProvider（预留 stub，阶段 4 实现真实对接）

use async_trait::async_trait;
use rust_decimal::Decimal;

use super::pay_provider::{CallbackResult, PayProvider, PayStatus, PrepayResult};
use crate::error::Result;
use crate::models::Order;

/// 微信支付（预留，未实现）
pub struct WechatPayProvider;

#[async_trait]
impl PayProvider for WechatPayProvider {
    fn name(&self) -> &'static str {
        "WECHAT"
    }

    async fn create_payment(&self, _order: &Order, _amount: Decimal) -> Result<PrepayResult> {
        todo!("阶段 4：微信统一下单 API 对接")
    }

    async fn query_status(&self, _provider_tx_id: &str) -> Result<PayStatus> {
        todo!("阶段 4：微信订单查询 API 对接")
    }

    async fn handle_callback(&self, _provider: &str, _payload: &[u8]) -> Result<CallbackResult> {
        todo!("阶段 4：微信支付回调验签")
    }
}
