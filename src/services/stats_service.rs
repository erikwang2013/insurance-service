//! 运营统计服务（A2：OPERATOR/ADMIN 全局运营数据汇总）
//!
//! 单次请求聚合，仅 SQL COUNT/SUM，不新建表、不改状态机。
//! 鉴权语义对齐 claim_service::review / product_service admin 动作：请求携带
//! 操作人 id，服务层校验 `users.role ∈ {OPERATOR, ADMIN}` 且未软删，否则 40300。
//! 所有统计在同一连接上顺序执行，近似单点快照，避免跨连接读取错位。

use std::collections::HashMap;

use mysql_async::prelude::Queryable;
use mysql_async::{Conn, Row};
use serde::{Deserialize, Serialize};

use crate::db::db_error;
use crate::db::Db;
use crate::error::{AppError, Result};
use crate::models::user::User;

/// 统计请求体（POST /api/v1/admin/stats）
#[derive(Debug, Clone, Deserialize)]
pub struct StatsReq {
    /// 操作人 id（须 OPERATOR / ADMIN）
    pub operator_user_id: i64,
}

/// 商品统计：总数 / 在售（ON_SALE）/ 其他（非 ON_SALE）
#[derive(Debug, Serialize)]
pub struct ProductStats {
    pub total: i64,
    pub on_sale: i64,
    pub others: i64,
}

/// 订单统计：总数 / 成交（PAID）单数 / 成交总额（SUM payable_amount，金额串保持精度）
#[derive(Debug, Serialize)]
pub struct OrderStats {
    pub total: i64,
    pub paid: i64,
    pub paid_amount: String,
}

/// 支付统计：支付成功总额（payments.status = SUCCESS 的 SUM amount）
#[derive(Debug, Serialize)]
pub struct PaymentStats {
    pub success_amount: String,
}

/// 按状态分组的通用结构（理赔 / 保单），by_status 键为状态原值
#[derive(Debug, Serialize)]
pub struct StatusStats {
    pub total: i64,
    pub by_status: HashMap<String, i64>,
}

/// 运营总览（data 载荷）
#[derive(Debug, Serialize)]
pub struct Overview {
    /// 用户总数（未软删）
    pub users: i64,
    pub products: ProductStats,
    pub orders: OrderStats,
    pub payments: PaymentStats,
    /// 理赔总数 + 按状态计数
    pub claims: StatusStats,
    /// 保单总数 + 按状态计数
    pub policies: StatusStats,
}

/// 运营统计服务
pub struct StatsService {
    db: Db,
}

impl StatsService {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// 运营总览：先校验操作人角色，再聚合全局统计（同一连接，近似单点快照）。
    pub async fn overview(&self, req: StatsReq) -> Result<Overview> {
        let mut conn = self.db.conn().await?;

        // 1) 操作人须为 OPERATOR / ADMIN（对齐理赔审核的角色校验）
        let role: Option<String> = conn
            .exec_first(
                "SELECT role FROM users WHERE id = ? AND deleted_at IS NULL LIMIT 1",
                vec![req.operator_user_id],
            )
            .await
            .map_err(db_error)?;
        if !matches!(
            role.as_deref(),
            Some(User::ROLE_OPERATOR) | Some(User::ROLE_ADMIN)
        ) {
            return Err(AppError::Forbidden);
        }

        // 2) 各表全局聚合（金额 SUM 经 CAST AS CHAR 输出，避免浮点/位宽歧义）
        let users = count(&mut conn, "SELECT COUNT(*) AS v FROM users WHERE deleted_at IS NULL").await?;

        let products_total =
            count(&mut conn, "SELECT COUNT(*) AS v FROM insurance_products WHERE deleted_at IS NULL").await?;
        let products_on_sale = count(
            &mut conn,
            "SELECT COUNT(*) AS v FROM insurance_products \
             WHERE deleted_at IS NULL AND status = 'ON_SALE'",
        )
        .await?;

        let orders_total =
            count(&mut conn, "SELECT COUNT(*) AS v FROM orders WHERE deleted_at IS NULL").await?;
        let orders_paid = count(
            &mut conn,
            "SELECT COUNT(*) AS v FROM orders WHERE deleted_at IS NULL AND status = 'PAID'",
        )
        .await?;
        let orders_paid_amount = sum_money(
            &mut conn,
            "SELECT CAST(COALESCE(SUM(payable_amount), 0) AS CHAR) AS v \
             FROM orders WHERE deleted_at IS NULL AND status = 'PAID'",
        )
        .await?;

        // payments 无软删列
        let payments_success_amount = sum_money(
            &mut conn,
            "SELECT CAST(COALESCE(SUM(amount), 0) AS CHAR) AS v \
             FROM payments WHERE status = 'SUCCESS'",
        )
        .await?;

        let claims_total =
            count(&mut conn, "SELECT COUNT(*) AS v FROM claims WHERE deleted_at IS NULL").await?;
        let claims_by_status = status_counts(
            &mut conn,
            "SELECT status, COUNT(*) AS c FROM claims WHERE deleted_at IS NULL GROUP BY status",
        )
        .await?;

        let policies_total =
            count(&mut conn, "SELECT COUNT(*) AS v FROM policies WHERE deleted_at IS NULL").await?;
        let policies_by_status = status_counts(
            &mut conn,
            "SELECT status, COUNT(*) AS c FROM policies WHERE deleted_at IS NULL GROUP BY status",
        )
        .await?;

        Ok(Overview {
            users,
            products: ProductStats {
                total: products_total,
                on_sale: products_on_sale,
                others: products_total - products_on_sale,
            },
            orders: OrderStats {
                total: orders_total,
                paid: orders_paid,
                paid_amount: orders_paid_amount,
            },
            payments: PaymentStats {
                success_amount: payments_success_amount,
            },
            claims: StatusStats {
                total: claims_total,
                by_status: claims_by_status,
            },
            policies: StatusStats {
                total: policies_total,
                by_status: policies_by_status,
            },
        })
    }
}

// ---------- 行映射 helpers（对齐 db.rs 的 Option<Option<T>> 解码习惯） ----------

/// COUNT(*) 计数（恒非 NULL；BIGINT 有符号 → i64）
async fn count(conn: &mut Conn, sql: &str) -> Result<i64> {
    let row: Option<Row> = conn.exec_first(sql, ()).await.map_err(db_error)?;
    Ok(row
        .and_then(|r| r.get::<Option<i64>, &str>("v"))
        .flatten()
        .unwrap_or(0))
}

/// SUM(...) AS v 金额（COALESCE 兜 NULL，CAST AS CHAR 保精度）
async fn sum_money(conn: &mut Conn, sql: &str) -> Result<String> {
    let row: Option<Row> = conn.exec_first(sql, ()).await.map_err(db_error)?;
    Ok(row
        .and_then(|r| r.get::<Option<String>, &str>("v"))
        .flatten()
        .unwrap_or_else(|| "0".to_string()))
}

/// GROUP BY status 计数表 → {status: count}
async fn status_counts(conn: &mut Conn, sql: &str) -> Result<HashMap<String, i64>> {
    let rows: Vec<(String, i64)> = conn.exec(sql, ()).await.map_err(db_error)?;
    Ok(rows.into_iter().collect())
}
