//! ESignQianProvider（e签宝，预留 stub，阶段 4 实现真实对接）

use async_trait::async_trait;

use super::esign_provider::{ElectronicSignature, EsignCreateResult};
use crate::error::Result;
use crate::models::{Contract, ContractSigner};

/// e签宝电子签（预留，未实现）
pub struct ESignQianProvider;

#[async_trait]
impl ElectronicSignature for ESignQianProvider {
    fn name(&self) -> &'static str {
        "ESIGN"
    }

    async fn create_contract(
        &self,
        _contract: &Contract,
        _signers: &[ContractSigner],
    ) -> Result<EsignCreateResult> {
        todo!("阶段 4：e签宝创建签署流程")
    }

    async fn get_sign_url(&self, _sign_flow_id: &str, _signer: &ContractSigner) -> Result<String> {
        todo!("阶段 4：e签宝获取签署链接")
    }

    async fn verify_completion(&self, _sign_flow_id: &str) -> Result<bool> {
        todo!("阶段 4：e签宝校验签署完成")
    }
}
