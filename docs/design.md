# 保险服务平台 — 设计文档（本地摘要）

> 本文档为**设计决策摘要与索引**，权威细节见 `docs/backend-architecture.md`（架构）与
> `docs/db-schema.md`（14 表 Schema 与 Rust models）。版本: 2026-10-30（v1.11.0）。

---

## 1. 分层架构

MVC 分层，axum 中间件承载横切关注点：

```
请求 → Trace → 限流 → JWT认证(RBAC) → Controller → Service → Db → 统一信封
```

| 层 | 职责 |
|----|------|
| 中间件链（axum） | trace_id / 限流 / 认证鉴权；无入站安全扫描器 |
| Controller | 参数提取 → 调用 Service → 组装响应（bee_router 分发） |
| Service | 业务规则、事务边界、审计落库 |
| Model | bee_orm `#[derive(Model)]` 行 → 结构体 |

服务仅提供 JSON 接口：`/api/v1/*` 与未匹配路径均返回统一 JSON 信封，不提供 HTML 页面。

## 2. 关键设计决策

| 决策 | 内容 | 权威出处 |
|------|------|----------|
| 统一信封 | `{code, message, data, trace_id}`，业务成功 code=0，业务错误 40000；内部错误 500 / 50000 通用文案 | backend-architecture.md §6 |
| 精确过期 | `validation.leeway = 0`，令牌 exp 精确执行 | src/middleware/auth.rs |
| 令牌类型隔离 | access / refresh 独立 `typ`；refresh 用 `JWT_REFRESH_EXPIRY`；`JWT_SECRET` ≥ 32 字节；无 `typ` 旧令牌拒绝 | backend-architecture.md §7 |
| RBAC | 角色 USER / AGENT / OPERATOR / ADMIN；axum 认证中间件校验 OPERATOR / ADMIN（无 `RequireRoleFilter`） | backend-architecture.md §5 |
| 身份来源 | 一律由令牌派生；客户端 `user_id` / `operator_user_id` 不作身份依据；详情按归属过滤 | backend-architecture.md §7 |
| 防注入 | 参数化查询；写事务 `with_tx` commit/rollback 闭环 | src/db.rs |
| 敏感加密 | `users.phone_enc` 服务端 AES-256-GCM 加密；报价 / 受益人 `*_enc` 列保存上游字节，服务端不重新加密 | db-schema.md §5 |
| 口令存储 | argon2 哈希，错误不区分账号/密码 | auth_service |
| 金额语义 | DECIMAL(14,2) 单位元，禁浮点；报价保费 = 保额 × 费率（服务端），订单优惠恒 0，客户端金额被忽略 | db-schema.md §1 |
| 状态机 | status 字符串列 + 枚举校验显式流转，禁任意跳变 | db-schema.md §1 |
| 软删除审计 | `deleted_at` + `audit_logs` 全量留痕（含 `trace_id`） | db-schema.md §1 |
| 搜索 | MySQL `LIKE`（仅 ON_SALE + search_enabled，LIKE 转义，分页兜底），无外部索引与异步同步 | src/services/search_service.rs |
| 支付 | 内联 Mock；回调需 `X-Callback-Token`；成功回调事务内提交支付 + 订单状态 | src/services/payment_service.rs |
| 保单 | 订单 PAID 后签发（幂等）；退保 / 受益人批改审计留痕；续保暂未开放 | src/services/policy_service.rs |
| 多端共用 | Flutter / 小程序 / 鸿蒙共用 REST API | backend-architecture.md §1 |

## 3. 数据模型要点（14 表）

- users / insurance_products / insurance_product_clauses / quotes / quotes_beneficiaries /
  orders / payments / policies / policy_beneficiaries / contracts / claims / audit_logs /
  claim_documents / quote_rates。
- 投保人 / 被保人 / 受益人可不同；一张保单多受益人，`share_percent` 合计 100%。
- 主键由应用层 snowflake 生成（idgen_rs 无锁算法，`crate::utils::idgen::next_id()`，worker_id 取
  `IDGEN_WORKER_ID` env，默认 0）。
- 业务号前缀：报价 `QT`、订单 `OD`、保单 `POL`、合同 `CT`、理赔 `CLM`、支付 `PAY`；
  独立生成并建唯一索引。

## 4. 文档导航

| 文档 | 职责 |
|------|------|
| `docs/backend-architecture.md` | 完整架构设计 |
| `docs/db-schema.md` | 库表 Schema 与 models 蓝本（install.sql 来源） |
| `docs/plan.md` | 分阶段规划与里程碑 |
| `docs/tasks.md` | 任务清单与完成判定 |
| `docs/flutter-app.md` / `docs/miniprogram-harmony.md` | 多端对接设计 |
| `README.md` | 项目总览（架构图 / 功能图 / 生命周期图 / 使用说明） |
