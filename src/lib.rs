//! 保险服务平台后端 — bee-rust 阶段 0 骨架
//!
//! MVC 分层：Controller / Service / Model，过滤器链承载横切关注点。
//! 详细设计见 `docs/backend-architecture.md`，表结构与模型见 `docs/db-schema.md`。

pub mod config;
pub mod controllers;
pub mod db;
pub mod crypto;
pub mod error;
pub mod middleware;
pub mod models;
pub mod providers;
pub mod response;
pub mod routes;
pub mod search;
pub mod services;
pub mod utils;
