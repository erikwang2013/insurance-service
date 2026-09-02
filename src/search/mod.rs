//! rust-scout 搜索集成（对齐 backend-architecture.md §10 / db-schema.md §7）
//!
//! 说明：规划文档引用 `rust_scout`（0.3, features=["elasticsearch"]）。crates.io 当前
//! 可见最新为 0.1.0，且底层 Engine API 与文档描述可能有出入。为阶段 0 骨架可编译，
//! 此处定义与文档一致的 `Searchable` trait（§7.2），并以 `SearchService` 门面封装；
//! rust-scout 实际引擎注入在依赖可拉取后于 `services/search_service.rs` 内完成。

pub mod searchable_impl;
pub mod sync_worker;

/// 搜索操作类型（对齐 db-schema.md §7.2）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchOp {
    Upsert,
    Delete,
}

/// 可检索实体 trait（对齐 db-schema.md §7.2）
pub trait Searchable: Send + Sync {
    /// 索引名："insurance_products" / "clauses" / "policies"
    fn index_name(&self) -> &'static str;
    /// 索引文档 _id（取业务主键字符串）
    fn doc_id(&self) -> String;
    /// 序列化为待索引 JSON 文档（敏感字段只放脱敏值）
    fn to_doc(&self) -> serde_json::Value;
    /// 操作类型（Delete 时仅 doc_id 有效）
    fn op(&self) -> SearchOp;
}
