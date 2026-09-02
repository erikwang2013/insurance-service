//! MockEsignProvider（对齐 backend-architecture.md §9.2）

use async_trait::async_trait;

use super::esign_provider::{ElectronicSignature, EsignCreateResult};
use crate::error::Result;
use crate::models::{Contract, ContractSigner};

/// 模拟电子签渠道
pub struct MockEsignProvider;

#[async_trait]
impl ElectronicSignature for MockEsignProvider {
    fn name(&self) -> &'static str {
        "MOCK"
    }

    async fn create_contract(
        &self,
        contract: &Contract,
        signers: &[ContractSigner],
    ) -> Result<EsignCreateResult> {
        let sign_flow_id = format!("MOCK-FLOW-{}", contract.contract_no);
        let sign_urls = signers
            .iter()
            .map(|s| (s.id, format!("/sign/mock/{sign_flow_id}/{}", s.id)))
            .collect();
        Ok(EsignCreateResult {
            sign_flow_id,
            sign_urls,
        })
    }

    async fn get_sign_url(&self, sign_flow_id: &str, _signer: &ContractSigner) -> Result<String> {
        Ok(format!("/sign/mock/{sign_flow_id}"))
    }

    async fn verify_completion(&self, _sign_flow_id: &str) -> Result<bool> {
        // 模拟：回调被调用即视为完成
        Ok(true)
    }
}
