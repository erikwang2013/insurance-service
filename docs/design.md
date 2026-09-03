# 保险服务平台 — 设计文档（本地摘要）

> 本文档为**设计决策摘要与索引**，权威细节见 `docs/backend-architecture.md`（架构）与
> `docs/db-schema.md`（19 表 Schema 与 Rust models）。设计正文不在此重复。
> 版本: 2026-09-02。

---

## 1. 分层架构

MVC 分层，过滤器链承载横切关注点：

```
请求 → SecurityFilter → Trace → JWT认证(RBAC) → Controller → Service → Db / Provider → 统一信封
```

| 层 | 职责 |
|----|------|
| Filter Chain | 安全扫描 / trace_id / 认证鉴权 / 审计 |
| Controller | 参数提取 → 调用 Service → 组装响应（bee_router 分发） |
| Service | 业务规则、事务边界 |
| Model | bee_orm `#[derive(Model)]` 行 → 结构体 |

## 2. 关键设计决策

| 决策 | 内容 | 权威出处 |
|------|------|----------|
| 统一信封 | `{code, message, data}`，业务成功 code=0，业务错误 40000 | backend-architecture.md §6 |
| 精确过期 | `validation.leeway = 0`，令牌 exp 精确执行 | src/middleware/auth.rs |
| RBAC | 角色 USER / AGENT / OPERATOR / ADMIN + RequireRoleFilter | backend-architecture.md §7 |
| 防注入 | 参数化查询；写事务 `with_tx` commit/rollback 闭环 | src/db.rs |
| 敏感加密 | 身份证 / 手机号 / 银行卡号字段级 AES-256-GCM，明文不落库 | db-schema.md §8 |
| 口令存储 | argon2 哈希，错误不区分账号/密码 | auth_service |
| 金额语义 | DECIMAL(14,2) 单位元，禁浮点；金额快照落库 | db-schema.md §1 |
| 状态机 | status 字符串列 + 枚举校验显式流转，禁任意跳变 | db-schema.md §1 |
| 软删除审计 | `deleted_at` + `audit_logs` 全量留痕 | db-schema.md §1 |
| 搜索解耦 | 业务只写 MySQL；OpenSearch 经 `search_sync_logs` 异步最终一致同步；未就绪降级 LIKE | db-schema.md §9 |
| 多端共用 | Flutter / 小程序 / 鸿蒙共用 REST API | backend-architecture.md §1 |

## 3. 外部 Provider 抽象

- 支付：`PayProvider` 接口 + 微信支付 / 支付宝 / Mock（预下单 + 回调验签，回调幂等）。
- 电子签：`ElectronicSignature` 接口 + 易签 / 法大大 / Mock（签署链接、签署回调、合同落库）。

## 4. 数据模型要点（19 表）

- users / insurance_products / clauses / quotes / orders / payments / policies（holder+beneficiary）/ contracts(+signer) / claims / audit_logs / search_sync_logs 等。
- 投保人 / 被保人 / 受益人可不同；一张保单多受益人，`beneficiary_share` 合计 100%。
- 主键由应用层 snowflake 生成（idgen_rs 0.2.0 无锁算法，`crate::utils::idgen::next_id()`，worker_id 取 `IDGEN_WORKER_ID` env，默认 0）；业务号（保单 O / 订单 P / 报价 Q / 理赔 CL）保留前缀独立生成并唯一索引。

## 5. 文档导航

| 文档 | 职责 |
|------|------|
| `docs/backend-architecture.md` | 完整架构设计 + §13 Roadmap |
| `docs/db-schema.md` | 库表 Schema 与 models 蓝本（install.sql 来源） |
| `docs/plan.md` | 分阶段规划与里程碑 |
| `docs/tasks.md` | 任务清单与完成判定 |
| `docs/flutter-app.md` / `docs/miniprogram-harmony.md` | 多端对接设计 |
| `README.md` | 项目总览（架构图 / 功能图 / 生命周期图 / 使用说明） |