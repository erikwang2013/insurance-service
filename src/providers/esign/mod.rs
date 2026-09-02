//! 电子签抽象（对齐 backend-architecture.md §9）

pub mod escqian;
pub mod esign_provider;
pub mod mock;

pub use esign_provider::{ElectronicSignature, EsignCreateResult};
pub use mock::MockEsignProvider;
