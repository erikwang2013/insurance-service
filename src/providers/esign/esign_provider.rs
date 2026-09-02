//! ElectronicSignature 抽象（对齐 backend-architecture.md §9.1）

use async_trait::async_trait;

use crate::error::Result;
use crate::models::{Contract, ContractSigner};

/// 创建签署流程结果
pub struct EsignCreateResult {
    /// 电子签平台流程 ID（存 contracts.sign_flow_id）
    pub sign_flow_id: String,
    /// (contract_signer.id, sign_url) 列表
    pub sign_urls: Vec<(i64, String)>,
}

/// 电子签抽象接口
#[async_trait]
pub trait ElectronicSignature: Send + Sync {
    /// 渠道名："MOCK" | "ESIGN"
    fn name(&self) -> &'static str;

    /// 创建签署流程，返回平台流程 ID 与各签署方签署链接
    async fn create_contract(
        &self,
        contract: &Contract,
        signers: &[ContractSigner],
    ) -> Result<EsignCreateResult>;

    /// 获取指定签署方的签署 URL
    async fn get_sign_url(&self, sign_flow_id: &str, signer: &ContractSigner) -> Result<String>;

    /// 校验签署是否全部完成
    async fn verify_completion(&self, sign_flow_id: &str) -> Result<bool>;
}
