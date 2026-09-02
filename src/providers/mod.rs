//! 可插拔第三方适配器（对齐 backend-architecture.md §8/§9）
//!
//! - `payment`：PayProvider（Mock / 预留 Wechat）
//! - `esign`：ElectronicSignature（Mock / 预留 ESignQian）

pub mod esign;
pub mod payment;
