//! 支付抽象（对齐 backend-architecture.md §8）

pub mod mock;
pub mod pay_provider;
pub mod wechat;

pub use mock::MockPayProvider;
pub use pay_provider::{
    CallbackResult, PayProvider, PayStatus, PaymentProviderRegistry, PrepayResult,
};
