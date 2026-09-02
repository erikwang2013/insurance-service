//! 数据库访问层：mysql_async 连接池 + 行→模型映射 + 执行/事务辅助（任务 #3）
//!
//! bee_orm 仅构建参数化 SELECT（不执行），此处以 mysql_async 补执行与事务；
//! 命名列读取（SELECT * 可移植），DATETIME → `DateTime<Utc>`，DECIMAL → `rust_decimal`。

use std::future::Future;
use std::pin::Pin;

use chrono::{DateTime, NaiveDateTime, Utc};
use rust_decimal::Decimal;

use bee_rust::bee_orm::Model;
use mysql_async::prelude::{FromRow, Queryable};
use mysql_async::{Conn, Params, Pool, Row, Transaction, TxOpts};

use crate::config::DbConfig;
use crate::error::AppError;
use crate::models::insurance_product::InsuranceProduct;
use crate::models::user::User;

/// MySQL 连接池封装（可 Clone，内部 Arc 共享）
#[derive(Clone)]
pub struct Db {
    pool: Pool,
}

impl Db {
    /// 以 DSN（mysql://user:pass@host:port/db）创建连接池；惰性建连，不立即连库。
    pub fn new(cfg: &DbConfig) -> Result<Self, AppError> {
        let pool = Pool::from_url(&cfg.url).map_err(db_error)?;
        Ok(Self { pool })
    }

    /// 取一条连接。
    pub async fn conn(&self) -> Result<Conn, AppError> {
        self.pool.get_conn().await.map_err(db_error)
    }

    /// 开启事务；未 commit 时 Drop 自动回滚。
    pub async fn tx(&self) -> Result<Transaction<'static>, AppError> {
        self.pool
            .start_transaction(TxOpts::default())
            .await
            .map_err(db_error)
    }

    /// 执行参数化查询，映射为模型列表。
    /// `sql` 仅允许含 `?` 占位符的受信语句；值一律经 `params` 绑定，杜绝注入。
    pub async fn query_all<T>(
        &self,
        sql: &str,
        params: impl Into<Params> + Send,
    ) -> Result<Vec<T>, AppError>
    where
        T: FromRow + Send + 'static,
    {
        let mut conn = self.conn().await?;
        conn.exec(sql, params).await.map_err(db_error)
    }

    /// 执行参数化查询，返回首行模型（无则 `None`）。
    pub async fn query_one<T>(
        &self,
        sql: &str,
        params: impl Into<Params> + Send,
    ) -> Result<Option<T>, AppError>
    where
        T: FromRow + Send + 'static,
    {
        let mut conn = self.conn().await?;
        conn.exec_first(sql, params).await.map_err(db_error)
    }

    /// 执行写操作（INSERT / UPDATE / DELETE），返回受影响行数。
    pub async fn exec_drop(
        &self,
        sql: &str,
        params: impl Into<Params> + Send,
    ) -> Result<u64, AppError> {
        let mut conn = self.conn().await?;
        conn.exec_drop(sql, params).await.map_err(db_error)?;
        Ok(conn.affected_rows())
    }

    /// 事务闭环：`f` 返回 `Ok` 则 COMMIT，`Err` 则 ROLLBACK。
    /// 调用方以 `|tx| Box::pin(async move { ... })` 提交闭包，事务内执行
    /// `tx.exec_*`（`Transaction` Deref 至 `Conn`，具备完整 `Queryable`）。
    pub async fn with_tx<T, F>(&self, f: F) -> Result<T, AppError>
    where
        F: for<'c> FnOnce(
                &'c mut Transaction<'static>,
            ) -> Pin<Box<dyn Future<Output = Result<T, AppError>> + Send + 'c>>,
    {
        let mut tx = self.tx().await?;
        match f(&mut tx).await {
            Ok(v) => {
                tx.commit().await.map_err(db_error)?;
                Ok(v)
            }
            Err(e) => {
                tx.rollback().await.map_err(db_error)?;
                Err(e)
            }
        }
    }
}

pub(crate) fn db_error(e: mysql_async::Error) -> AppError {
    AppError::db(e.to_string())
}

/// 读取非空时间列（DATETIME(3)）
fn dt(row: &Row, col: &str) -> DateTime<Utc> {
    row.get::<NaiveDateTime, &str>(col)
        .unwrap_or_default()
        .and_utc()
}

/// 读取可空时间列
fn dt_opt(row: &Row, col: &str) -> Option<DateTime<Utc>> {
    row.get::<Option<NaiveDateTime>, &str>(col)
        .flatten()
        .map(|d| d.and_utc())
}

/// 读取可空 DECIMAL 列（二进制协议以字符串到达）
fn dec_opt(row: &Row, col: &str) -> Option<Decimal> {
    row.get::<Option<String>, &str>(col)
        .flatten()
        .and_then(|s| s.parse().ok())
}

impl Model for User {}

impl FromRow for User {
    fn from_row_opt(row: Row) -> Result<Self, mysql_async::FromRowError> {
        Ok(User {
            id: row.get("id").unwrap_or_default(),
            username: row.get("username").unwrap_or_default(),
            phone_enc: row.get("phone_enc").flatten(),
            id_card_enc: row.get("id_card_enc").flatten(),
            phone_masked: row.get("phone_masked").flatten(),
            password_hash: row.get("password_hash").unwrap_or_default(),
            email: row.get("email").flatten(),
            nickname: row.get("nickname").flatten(),
            avatar_url: row.get("avatar_url").flatten(),
            role: row.get("role").unwrap_or_default(),
            status: row.get("status").unwrap_or_default(),
            last_login_at: dt_opt(&row, "last_login_at"),
            created_at: dt(&row, "created_at"),
            updated_at: dt(&row, "updated_at"),
            deleted_at: dt_opt(&row, "deleted_at"),
        })
    }
}

impl Model for InsuranceProduct {}

impl FromRow for InsuranceProduct {
    fn from_row_opt(row: Row) -> Result<Self, mysql_async::FromRowError> {
        Ok(InsuranceProduct {
            id: row.get("id").unwrap_or_default(),
            product_code: row.get("product_code").unwrap_or_default(),
            name: row.get("name").unwrap_or_default(),
            subtitle: row.get("subtitle").flatten(),
            description: row.get("description").flatten(),
            product_type: row.get("product_type").unwrap_or_default(),
            sale_channel: row.get("sale_channel").unwrap_or_default(),
            operator_user_id: row.get("operator_user_id").flatten(),
            insurer_name: row.get("insurer_name").flatten(),
            currency: row.get("currency").unwrap_or_default(),
            min_amount: dec_opt(&row, "min_amount"),
            max_amount: dec_opt(&row, "max_amount"),
            min_term_months: row.get("min_term_months").flatten(),
            max_term_months: row.get("max_term_months").flatten(),
            waiting_period_days: row.get("waiting_period_days").flatten(),
            is_featured: row.get("is_featured").unwrap_or_default(),
            cover_image_url: row.get("cover_image_url").flatten(),
            status: row.get("status").unwrap_or_default(),
            search_enabled: row.get("search_enabled").unwrap_or_default(),
            created_at: dt(&row, "created_at"),
            updated_at: dt(&row, "updated_at"),
            deleted_at: dt_opt(&row, "deleted_at"),
        })
    }
}