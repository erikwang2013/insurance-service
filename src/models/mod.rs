//! 数据模型（对齐 docs/db-schema.md §6）
//!
//! 说明：规划文档使用 `bee_orm #[derive(Model)]` + `#[orm(...)]` 标注（feature `mysql`）。
//! bee-orm 当前无法在编译环境拉取，阶段 0 采用**标准 struct + serde** 承载字段，
//! 字段名/类型/可空性与 db-schema.md §6 严格一致。待 bee_orm 可用时：
//! 1. 为每个 struct 追加 `#[derive(Model)]`
//! 2. 为字段追加 `#[orm(primary_key)] / #[orm(size=N)] / #[orm(unique)] /
//!    #[orm(foreign_key="X::id", index)] / #[orm(create_time)] / #[orm(update_time)]` 标注
//! 3. `#[orm(...)]` 属性具体语法以 bee_orm 实际宏为准
//!
//! 说明 2：`users.phone_enc` 等密文字段用 `Vec<u8>` 映射 `VARBINARY`；金额用
//! `rust_decimal::Decimal`；时间用 `chrono::{DateTime<Utc>, NaiveDate}`。

pub mod audit_log;
pub mod claim;
pub mod contract;
pub mod contract_signer;
pub mod insurance_product;
pub mod insurance_product_category;
pub mod insurance_product_category_rel;
pub mod insurance_product_clause;
pub mod order;
pub mod payment;
pub mod policy;
pub mod policy_beneficiary;
pub mod policy_holder;
pub mod quote;
pub mod quote_beneficiary;
pub mod search_sync_log;
pub mod user;

pub use audit_log::AuditLog;
pub use claim::Claim;
pub use contract::Contract;
pub use contract_signer::ContractSigner;
pub use insurance_product::InsuranceProduct;
pub use insurance_product_category::InsuranceProductCategory;
pub use insurance_product_category_rel::InsuranceProductCategoryRel;
pub use insurance_product_clause::InsuranceProductClause;
pub use order::Order;
pub use payment::Payment;
pub use policy::Policy;
pub use policy_beneficiary::PolicyBeneficiary;
pub use policy_holder::PolicyHolder;
pub use quote::Quote;
pub use quote_beneficiary::QuoteBeneficiary;
pub use search_sync_log::SearchSyncLog;
pub use user::User;
