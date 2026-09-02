//! Product / Clause / Policy 实现 Searchable（对齐 db-schema.md §7.3）

use super::{SearchOp, Searchable};
use crate::models::{InsuranceProduct, InsuranceProductClause, Policy};

impl Searchable for InsuranceProduct {
    fn index_name(&self) -> &'static str {
        "insurance_products"
    }

    fn doc_id(&self) -> String {
        self.id.to_string()
    }

    fn to_doc(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "product_code": self.product_code,
            "name": self.name,
            "subtitle": self.subtitle,
            "description": self.description,
            "product_type": self.product_type,
            "sale_channel": self.sale_channel,
            "insurer_name": self.insurer_name,
            "currency": self.currency,
            "min_amount": self.min_amount,
            "max_amount": self.max_amount,
            "min_term_months": self.min_term_months,
            "max_term_months": self.max_term_months,
            "waiting_period_days": self.waiting_period_days,
            "is_featured": self.is_featured,
            "status": self.status,
            "created_at": self.created_at,
        })
    }

    fn op(&self) -> SearchOp {
        SearchOp::Upsert
    }
}

impl Searchable for InsuranceProductClause {
    fn index_name(&self) -> &'static str {
        "clauses"
    }

    fn doc_id(&self) -> String {
        self.id.to_string()
    }

    fn to_doc(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "product_id": self.product_id,
            "clause_type": self.clause_type,
            "title": self.title,
            "content": self.content,
            "version": self.version,
            "status": self.status,
            "updated_at": self.updated_at,
        })
    }

    fn op(&self) -> SearchOp {
        SearchOp::Upsert
    }
}

impl Searchable for Policy {
    fn index_name(&self) -> &'static str {
        "policies"
    }

    fn doc_id(&self) -> String {
        self.id.to_string()
    }

    /// 保单索引（内部检索）：含被保人姓名/保单号；身份证只放脱敏值。
    fn to_doc(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "policy_no": self.policy_no,
            "order_id": self.order_id,
            "user_id": self.user_id,
            "product_id": self.product_id,
            "product_name": self.product_name,
            "holder_name": self.holder_name,
            "insurance_amount": self.insurance_amount,
            "premium": self.premium,
            "effective_date": self.effective_date,
            "expire_date": self.expire_date,
            "status": self.status,
            "created_at": self.created_at,
        })
    }

    fn op(&self) -> SearchOp {
        SearchOp::Upsert
    }
}
