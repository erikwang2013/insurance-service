//! 搜索集成（对齐 backend-architecture.md §10 / db-schema.md §7）
//!
//! 当前搜索由 `services/search_service.rs` 以 MySQL LIKE 实现；此处只保留
//! `Searchable` trait（§7.2）与实体序列化。

pub mod searchable_impl;

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
