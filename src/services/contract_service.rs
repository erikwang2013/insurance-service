//! 电子合同服务（交易闭环）
//!
//! 保单签发后生成合同（Mock 直签）：校验保单 ACTIVE → INSERT contract(COMPLETED) → 回读。

use chrono::{DateTime, NaiveDateTime, Utc};
use mysql_async::prelude::Queryable;
use mysql_async::Value;
use mysql_async::Row;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::db_error;
use crate::db::Db;
use crate::error::{AppError, Result};
use crate::models::contract::Contract;

#[derive(Debug, Deserialize)]
pub struct CreateContractReq {
    pub policy_id: i64,
    pub order_id: i64,
    pub user_id: i64,
    #[serde(default = "default_title")]
    pub title: String,
    #[serde(default = "default_type")]
    pub contract_type: String,
}

fn default_title() -> String { "保险电子合同".to_string() }
fn default_type() -> String { "POLICY".to_string() }

pub struct ContractService {
    db: Db,
}

/// sign-url 响应：Mock 签署地址（前端跳转进入 Mock 签署页）
#[derive(Debug, Serialize)]
pub struct ContractSignUrl {
    pub sign_url: String,
    pub provider: String,
}

impl ContractService {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    fn contract_no() -> String {
        format!("CT{}{:04x}", Utc::now().timestamp_millis(), Uuid::new_v4().as_u128() as u64 & 0xFFFF)
    }

    pub async fn sign(&self, req: CreateContractReq) -> Result<Contract> {
        let contract_no = Self::contract_no();

        let row: Option<Row> = self
            .db
            .with_tx(|tx| {
                Box::pin(async move {
                    let pol: Option<Row> = tx
                        .exec_first(
                            "SELECT id, status FROM policies WHERE id = ? AND user_id = ? AND deleted_at IS NULL LIMIT 1",
                            vec![req.policy_id, req.user_id],
                        )
                        .await
                        .map_err(db_error)?;
                    let pol = pol.ok_or_else(|| AppError::business("保单不存在"))?;
                    let pol_status: String = pol.get("status").unwrap_or_default();
                    if pol_status != PolicyStatus::ACTIVE {
                        return Err(AppError::business("保单状态不可签约"));
                    }

                    let now = Utc::now();
                    let dt = now.format("%Y-%m-%d %H:%M:%S").to_string();
                    let flow_id = format!("EFLOW{}", Uuid::new_v4());

                    // 主键由应用层 snowflake 预生成后显式插入（全库自增迁移，见 idgen）
                    let id = crate::utils::idgen::next_id();
                    tx.exec_drop(
                        "INSERT INTO contracts (id, contract_no, policy_id, order_id, title, \
                         contract_type, pdf_path, file_hash, sign_flow_id, provider, status, \
                         signed_at, created_at, updated_at) \
                         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                        vec![
                            id.into(),
                            Value::from(&contract_no),
                            Value::from(req.policy_id),
                            Value::from(req.order_id),
                            Value::from(&req.title),
                            Value::from(&req.contract_type),
                            Value::NULL,
                            Value::NULL,
                            Value::from(&flow_id),
                            Value::from("MOCK"),
                            Value::from(Contract::STATUS_COMPLETED.to_string()),
                            Value::from(&dt),
                            Value::from(&dt),
                            Value::from(&dt),
                        ],
                    )
                    .await
                    .map_err(db_error)?;

                    tx.exec_first("SELECT * FROM contracts WHERE id = ? LIMIT 1", vec![id])
                        .await
                        .map_err(db_error)
                })
            })
            .await?;

        row.map(|r| row_to_contract(&r)).transpose()?
            .ok_or_else(|| AppError::business("合同签发后回读失败"))
    }

    pub async fn by_id(&self, id: i64) -> Result<Contract> {
        let row: Option<Row> = self
            .db
            .conn()
            .await?
            .exec_first("SELECT * FROM contracts WHERE id = ? AND deleted_at IS NULL LIMIT 1", vec![id])
            .await
            .map_err(db_error)?;
        row.map(|r| row_to_contract(&r)).transpose()?.ok_or(AppError::NotFound)
    }

    /// Mock 签署 URL：合同存在且未进入终态（作废/过期/拒签）时返回可跳转的
    /// Mock 签署地址（provider='MOCK'）。真实验签渠道（e签宝）属规划，未接入。
    ///
    /// 地址与 MockEsignProvider 语义一致：`/sign/mock/{sign_flow_id}`；合同尚无
    /// 平台流程 ID 时按 MockEsignProvider::create_contract 对同合同的命名回退，
    /// 保证 URL 稳定可派生。
    pub async fn sign_url(&self, id: i64) -> Result<ContractSignUrl> {
        let c = self.by_id(id).await?;
        match c.status.as_str() {
            Contract::STATUS_VOID | Contract::STATUS_EXPIRED | Contract::STATUS_REJECTED => {
                return Err(AppError::business("合同已终止，不可签署"));
            }
            _ => {}
        }
        if c.provider != "MOCK" {
            return Err(AppError::business("电子签署渠道未接入（当前仅支持 MOCK）"));
        }
        let flow = c
            .sign_flow_id
            .unwrap_or_else(|| format!("MOCK-FLOW-{}", c.contract_no));
        Ok(ContractSignUrl {
            sign_url: format!("/sign/mock/{flow}"),
            provider: c.provider,
        })
    }

    pub async fn by_user(&self, user_id: i64, page: u32, size: u32) -> Result<Vec<Contract>> {
        let size = size.clamp(1, 100) as usize;
        let offset = ((page.max(1) as usize) - 1) * size;
        let rows: Vec<Row> = self
            .db
            .conn()
            .await?
            .exec(
                "SELECT c.* FROM contracts c JOIN policies p ON p.id = c.policy_id \
                 WHERE p.user_id = ? AND c.deleted_at IS NULL ORDER BY c.created_at DESC LIMIT ? OFFSET ?",
                vec![user_id, size as i64, offset as i64],
            )
            .await
            .map_err(db_error)?;
        Ok(rows.iter().map(|r| row_to_contract(r)).collect::<Result<Vec<_>>>()?)
    }
}

#[allow(non_snake_case)]
mod PolicyStatus {
    pub const ACTIVE: &str = "ACTIVE";
}

fn dt_row(row: &Row, col: &str) -> DateTime<Utc> {
    row.get::<NaiveDateTime, &str>(col).unwrap_or_default().and_utc()
}
fn dt_opt_row(row: &Row, col: &str) -> Option<DateTime<Utc>> {
    row.get::<Option<NaiveDateTime>, &str>(col).flatten().map(|d| d.and_utc())
}

fn row_to_contract(row: &Row) -> Result<Contract> {
    Ok(Contract {
        id: row.get("id").unwrap_or_default(),
        contract_no: row.get("contract_no").unwrap_or_default(),
        policy_id: row.get("policy_id").unwrap_or_default(),
        order_id: row.get("order_id").unwrap_or_default(),
        title: row.get("title").unwrap_or_default(),
        contract_type: row.get("contract_type").unwrap_or_default(),
        pdf_path: row.get("pdf_path").flatten(),
        file_hash: row.get("file_hash").flatten(),
        sign_flow_id: row.get("sign_flow_id").flatten(),
        provider: row.get("provider").unwrap_or_default(),
        status: row.get("status").unwrap_or_default(),
        signed_at: dt_opt_row(row, "signed_at"),
        created_at: dt_row(row, "created_at"),
        updated_at: dt_row(row, "updated_at"),
        deleted_at: dt_opt_row(row, "deleted_at"),
    })
}
