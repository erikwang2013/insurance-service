//! 电子合同控制器：详情 / 签署 / 签署 URL / 回调

use async_trait::async_trait;
use axum::http::StatusCode;
use axum::response::Response;

use bee_rust::bee_router::context::RouterError;
use bee_rust::bee_router::{Context, Controller};

use crate::services::contract_service::{CreateContractReq, ContractService};

use super::{error_response, json_envelope, ok_response, read_json};

/// 电子合同控制器（持有 ContractService，按请求路径与方法分派动作）
pub struct ContractController {
    service: ContractService,
}

impl ContractController {
    pub fn new(service: ContractService) -> Self {
        Self { service }
    }

    async fn detail(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let id = match ctx.param("id").and_then(|s| s.parse::<i64>().ok()) {
            Some(id) => id,
            None => return self.reply(ctx, json_envelope(40000, "合同 id 参数无效")).await,
        };
        match self.service.by_id(id).await {
            Ok(c) => {
                let data = serde_json::to_value(&c)
                    .map_err(|e| RouterError::SerializeError(e.to_string()))?;
                self.reply(ctx, ok_response(data)).await
            }
            Err(e) => self.reply(ctx, error_response(&e)).await,
        }
    }

    async fn sign(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let _id = match ctx.param("id").and_then(|s| s.parse::<i64>().ok()) {
            Some(id) => id,
            None => return self.reply(ctx, json_envelope(40000, "合同 id 参数无效")).await,
        };
        let req: CreateContractReq = match read_json(ctx).await {
            Ok(r) => r,
            Err(resp) => return self.reply(ctx, resp).await,
        };
        match self.service.sign(req).await {
            Ok(c) => {
                let data = serde_json::to_value(&c)
                    .map_err(|e| RouterError::SerializeError(e.to_string()))?;
                self.reply(ctx, ok_response(data)).await
            }
            Err(e) => self.reply(ctx, error_response(&e)).await,
        }
    }

    async fn sign_url(&self, ctx: &mut Context) -> Result<(), RouterError> {
        // Mock 签署：已建合同返回可跳转的 Mock 签署 URL（provider='MOCK'，
        // 形如 /sign/mock/{flow}），前端凭该地址进入 Mock 签署；合同不存在
        // 或已终止按统一错误语义返回。真实验签渠道（e签宝）属规划，不在此实现。
        let id = match ctx.param("id").and_then(|s| s.parse::<i64>().ok()) {
            Some(id) => id,
            None => return self.reply(ctx, json_envelope(40000, "合同 id 参数无效")).await,
        };
        match self.service.sign_url(id).await {
            Ok(u) => {
                let data = serde_json::to_value(&u)
                    .map_err(|e| RouterError::SerializeError(e.to_string()))?;
                self.reply(ctx, ok_response(data)).await
            }
            Err(e) => self.reply(ctx, error_response(&e)).await,
        }
    }

    async fn callback(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let _provider = ctx.param("provider").unwrap_or("mock");
        self.reply(ctx, ok_response(serde_json::json!({"ok": true}))).await
    }

    async fn reply(&self, ctx: &mut Context, resp: Response) -> Result<(), RouterError> {
        let bytes = axum::body::to_bytes(resp.into_body(), 2 * 1024 * 1024)
            .await
            .map_err(|e| RouterError::Internal(e.to_string()))?;
        let value: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|e| RouterError::SerializeError(e.to_string()))?;
        ctx.json(&value)?;
        Ok(())
    }
}

#[async_trait]
impl Controller for ContractController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let path = ctx.request.uri().path().to_string();
        let method = ctx.request.method().clone();
        if path.contains("/sign-url") {
            self.sign_url(ctx).await
        } else if path.contains("/callback") {
            self.callback(ctx).await
        } else if path.contains("/sign") && method == axum::http::Method::POST {
            self.sign(ctx).await
        } else if method == axum::http::Method::GET && ctx.param("id").is_some() {
            self.detail(ctx).await
        } else {
            ctx.abort(StatusCode::NOT_FOUND, "接口不存在");
            Ok(())
        }
    }
}