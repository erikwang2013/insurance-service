//! 运营侧管理控制器：商品建档/更新、上下架/停售（POST）+ 审计日志查询（GET）
//!
//! 动作执行前由服务层校验操作人角色（OPERATOR/ADMIN，否则 40300）。
//!
//! TODO(lead): 接线 —— controllers/mod.rs 增 `pub mod admin;`、AppState 增
//! `pub admin: Arc<AdminController>` 与 `AdminController::new(db.clone())`、
//! `pub admin_handler` 适配器；routes.rs `build_bee_router` 注册
//! `POST /admin/products`、`POST /admin/products/{id}/status` 与
//! `GET /admin/audit-logs`（对齐 route_table() 中 admin 条目）。

use async_trait::async_trait;
use axum::http::StatusCode;
use axum::response::Response;
use chrono::{NaiveDate, NaiveDateTime};
use mysql_async::prelude::Queryable;
use mysql_async::{Row, Value};

use bee_rust::bee_router::context::RouterError;
use bee_rust::bee_router::{Context, Controller};

use crate::db::{db_error, Db};
use crate::error::{AppError, Result as AppResult};
use crate::models::audit_log::AuditLog;
use crate::models::user::User;
use crate::services::product_service::{
    admin_change_status, admin_upsert, AdminStatusReq, AdminUpsertReq,
};

use super::{error_response, json_envelope, ok_response, query_map, read_json};

/// 运营侧商品控制器（持有 Db，按请求路径分派动作）
pub struct AdminController {
    db: Db,
}

impl AdminController {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    async fn upsert(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let req: AdminUpsertReq = match read_json(ctx).await {
            Ok(r) => r,
            Err(resp) => return self.reply(ctx, resp).await,
        };
        match admin_upsert(&self.db, &req).await {
            Ok(p) => {
                let data = serde_json::to_value(&p)
                    .map_err(|e| RouterError::SerializeError(e.to_string()))?;
                self.reply(ctx, ok_response(data)).await
            }
            Err(e) => self.reply(ctx, error_response(&e)).await,
        }
    }

    async fn change_status(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let product_id = match ctx.param("id").and_then(|s| s.parse::<i64>().ok()) {
            Some(id) => id,
            None => {
                return self.reply(ctx, json_envelope(40000, "商品 id 参数无效")).await;
            }
        };
        let req: AdminStatusReq = match read_json(ctx).await {
            Ok(r) => r,
            Err(resp) => return self.reply(ctx, resp).await,
        };
        match admin_change_status(&self.db, product_id, &req).await {
            Ok(p) => {
                let data = serde_json::to_value(&p)
                    .map_err(|e| RouterError::SerializeError(e.to_string()))?;
                self.reply(ctx, ok_response(data)).await
            }
            Err(e) => self.reply(ctx, error_response(&e)).await,
        }
    }

    /// GET /api/v1/admin/audit-logs：运营端审计日志查询（须 OPERATOR/ADMIN）
    ///
    /// query 参数：`operator_user_id` 必填（操作人身份，用于角色校验）；
    /// `user_id`/`action`/`entity_type`/`entity_id` 精确过滤（可选）；
    /// `created_from`/`created_to` 时间范围（可选，支持 `YYYY-MM-DD` 或
    /// `YYYY-MM-DD HH:MM:SS`，闭区间）；`page` 从 1 起、`size` 1..=100
    /// （缺省 1/20）。按 `created_at DESC, id DESC` 排序，返回行 + 过滤后总数。
    async fn audit_logs(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let q = query_map(ctx.request.uri().query());
        let operator_id: i64 = match q.get("operator_user_id").and_then(|s| s.parse().ok()) {
            Some(id) => id,
            None => return self.reply(ctx, json_envelope(40000, "缺少 operator_user_id")).await,
        };
        let page: u32 = q.get("page").and_then(|s| s.parse().ok()).unwrap_or(1);
        let size = q.get("size").and_then(|s| s.parse().ok()).unwrap_or(20u32);
        let size = size.clamp(1, 100) as usize;
        let offset = ((page.max(1) as usize) - 1) * size;

        // 逐参数拼过滤条件（列名白名单硬编码，无注入面）
        let mut conds: Vec<String> = Vec::new();
        let mut args: Vec<Value> = Vec::new();
        for (key, col) in [("user_id", "user_id"), ("entity_id", "entity_id")] {
            if let Some(v) = q.get(key) {
                let n: i64 = match v.parse() {
                    Ok(n) => n,
                    Err(_) => {
                        return self
                            .reply(ctx, json_envelope(40000, format!("{key} 参数无效")))
                            .await;
                    }
                };
                conds.push(format!("{col} = ?"));
                args.push(Value::from(n));
            }
        }
        for (key, col) in [("action", "action"), ("entity_type", "entity_type")] {
            if let Some(v) = q.get(key) {
                if v.is_empty() {
                    return self
                        .reply(ctx, json_envelope(40000, format!("{key} 参数无效")))
                        .await;
                }
                conds.push(format!("{col} = ?"));
                args.push(Value::from(v.as_str()));
            }
        }
        if let Some(v) = q.get("created_from") {
            // 客户端常以 %20 编码空格（query_map 阶段 0 不解码），先还原再解析
            let v = v.replace("%20", " ");
            let Some(dt) = parse_dt_start(&v) else {
                return self
                    .reply(ctx, json_envelope(
                        40000,
                        "created_from 时间格式无效（支持 YYYY-MM-DD 或 YYYY-MM-DD HH:MM:SS）",
                    ))
                    .await;
            };
            conds.push("created_at >= ?".to_string());
            args.push(Value::from(dt.format("%Y-%m-%d %H:%M:%S").to_string()));
        }
        if let Some(v) = q.get("created_to") {
            // 客户端常以 %20 编码空格（query_map 阶段 0 不解码），先还原再解析
            let v = v.replace("%20", " ");
            let Some(dt) = parse_dt_end(&v) else {
                return self
                    .reply(ctx, json_envelope(
                        40000,
                        "created_to 时间格式无效（支持 YYYY-MM-DD 或 YYYY-MM-DD HH:MM:SS）",
                    ))
                    .await;
            };
            conds.push("created_at <= ?".to_string());
            args.push(Value::from(dt.format("%Y-%m-%d %H:%M:%S").to_string()));
        }

        match query_audit_logs(&self.db, operator_id, conds, args, size, offset).await {
            Ok((list, total)) => {
                let data = serde_json::json!({
                    "list": list,
                    "total": total,
                    "page": page,
                    "size": size as u32,
                });
                self.reply(ctx, ok_response(data)).await
            }
            Err(e) => self.reply(ctx, error_response(&e)).await,
        }
    }

    /// 将响应写入 Context（与 claim/product 控制器同模式）
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
impl Controller for AdminController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        let path = ctx.request.uri().path().to_string();
        let method = ctx.request.method().clone();
        if method == axum::http::Method::POST && path.ends_with("/status") {
            self.change_status(ctx).await
        } else if method == axum::http::Method::POST && path.ends_with("/admin/products") {
            self.upsert(ctx).await
        } else if method == axum::http::Method::GET && path.ends_with("/admin/audit-logs") {
            self.audit_logs(ctx).await
        } else {
            ctx.abort(StatusCode::NOT_FOUND, "接口不存在");
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// 审计日志查询辅助（控制器内私有的服务逻辑）
// ---------------------------------------------------------------------------

/// 校验操作人为运营/管理员（users.role IN OPERATOR/ADMIN 且未软删），否则 Forbidden
/// （与 product_service::ensure_operator 同模式）。
async fn ensure_operator(db: &Db, user_id: i64) -> AppResult<()> {
    let role: Option<String> = db
        .conn()
        .await?
        .exec_first(
            "SELECT role FROM users WHERE id = ? AND deleted_at IS NULL LIMIT 1",
            vec![user_id],
        )
        .await
        .map_err(db_error)?;
    match role.as_deref() {
        Some(User::ROLE_OPERATOR) | Some(User::ROLE_ADMIN) => Ok(()),
        _ => Err(AppError::Forbidden),
    }
}

/// 解析 `YYYY-MM-DD HH:MM:SS` 或 `YYYY-MM-DD` 为时间下界（date-only 取当日 00:00:00）。
fn parse_dt_start(s: &str) -> Option<NaiveDateTime> {
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Some(dt);
    }
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()?.and_hms_opt(0, 0, 0)
}

/// 时间上界解析：date-only 取当日 23:59:59（该日整日落入闭区间）。
fn parse_dt_end(s: &str) -> Option<NaiveDateTime> {
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Some(dt);
    }
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .ok()?
        .and_hms_opt(23, 59, 59)
}

/// 读取 CAST 为 CHAR 的 JSON 列并解析（NULL → None；解析失败按 None 容错）。
fn json_opt_row(row: &Row, col: &str) -> Option<serde_json::Value> {
    match row.get::<Option<String>, _>(col) {
        Some(Some(s)) => serde_json::from_str(&s).ok(),
        _ => None,
    }
}

/// 行 → AuditLog（created_at DATETIME(3) 经 chrono feature 读出 NaiveDateTime 后转 UTC）。
/// 注意：Row::get 的外层 Option 语义是「列缺失」，SQL NULL 需再套一层
/// `Option<T>` 读取（裸 get 会按字段类型推断成内层 T，NULL 时 panic）。
fn row_to_audit(row: &Row) -> AuditLog {
    AuditLog {
        id: row.get("id").unwrap_or_default(),
        user_id: row.get::<Option<i64>, _>("user_id").flatten(),
        action: row.get("action").unwrap_or_default(),
        entity_type: row.get("entity_type").unwrap_or_default(),
        entity_id: row.get("entity_id").unwrap_or_default(),
        before_json: json_opt_row(row, "before_json"),
        after_json: json_opt_row(row, "after_json"),
        ip: row.get::<Option<String>, _>("ip").flatten(),
        user_agent: row.get::<Option<String>, _>("user_agent").flatten(),
        trace_id: row.get::<Option<String>, _>("trace_id").flatten(),
        created_at: row
            .get::<NaiveDateTime, _>("created_at")
            .unwrap_or_default()
            .and_utc(),
    }
}

/// 查询审计日志：先校验操作人角色；conds/args 拼 WHERE（列名调用方白名单硬编码），
/// 返回（行列表按 created_at DESC, id DESC, 过滤后总数）。JSON 列以
/// `CAST(... AS CHAR)` 读出（mysql_async 无 serde_json 编解码时最稳）。
async fn query_audit_logs(
    db: &Db,
    operator_id: i64,
    conds: Vec<String>,
    args: Vec<Value>,
    size: usize,
    offset: usize,
) -> AppResult<(Vec<AuditLog>, i64)> {
    ensure_operator(db, operator_id).await?;
    let mut conn = db.conn().await?;
    let mut sql = String::from("FROM audit_logs");
    if !conds.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conds.join(" AND "));
    }
    let total: i64 = conn
        .exec_first(&format!("SELECT COUNT(*) {sql}"), args.clone())
        .await
        .map_err(db_error)?
        .unwrap_or(0);
    // LIMIT/OFFSET 以 Value 追加进同一参数表（与全项目 exec 传参风格一致）
    let mut list_args = args;
    list_args.push(Value::from(size as i64));
    list_args.push(Value::from(offset as i64));
    let rows: Vec<Row> = conn
        .exec(
            &format!(
                "SELECT id, user_id, action, entity_type, entity_id, \
                 CAST(before_json AS CHAR) AS before_json, CAST(after_json AS CHAR) AS after_json, \
                 ip, user_agent, trace_id, created_at {sql} \
                 ORDER BY created_at DESC, id DESC LIMIT ? OFFSET ?"
            ),
            list_args,
        )
        .await
        .map_err(db_error)?;
    Ok((rows.iter().map(row_to_audit).collect(), total))
}
