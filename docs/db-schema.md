# 保险服务平台 — 数据库 Schema

> 版本: 2026-10-27（v1.10.0）。以 `install.sql` 为准，共 14 张业务表。

---

## 1. 总体设计原则

- 主键：`BIGINT UNSIGNED`，由应用层 snowflake 生成（`idgen_rs`），非自增。
- 业务号：报价 Q / 订单 P / 保单 O / 理赔 CL，独立生成并建唯一索引。
- 金额：`DECIMAL(14,2)`，单位元；禁止浮点。
- 时间：`DATETIME(3)`；业务状态用 `VARCHAR` 显式枚举，禁止任意跳变。
- 软删除：`deleted_at`；关键操作写入 `audit_logs`。
- 敏感字段：身份证 / 手机号 / 银行卡号等使用 AES-256-GCM 密文列（`*_enc`）。
- 搜索：MySQL `LIKE`；不依赖外部索引或异步同步队列。

## 2. ER 关系

```
users ──< quotes ──< quotes_beneficiaries
  │         │
  │         └──< orders ──< payments
  │                 │
  │                 ├──< policies ──< policy_beneficiaries
  │                 │        │
  │                 │        └──< contracts
  │                 │        └──< claims ──< claim_documents
  │                 └──< quote_rates
  └──< audit_logs
```

## 3. 表清单

| # | 表 | 用途 |
|---|---|---|
| 1 | `users` | 用户账户、微信 openid、角色、令牌版本 |
| 2 | `insurance_products` | 保险产品（编码唯一、状态、保额/保障期范围） |
| 3 | `insurance_product_clauses` | 产品条款（主条款 / 免责 / 等待期 / 附加） |
| 4 | `quotes` | 报价 / 投保方案（保额、保费、健康告知、有效期） |
| 5 | `quotes_beneficiaries` | 报价期受益人快照 |
| 6 | `orders` | 订单（报价转订单、产品/被保人快照、金额） |
| 7 | `payments` | 支付流水（provider + provider_tx_id 幂等） |
| 8 | `policies` | 保单（签发 / 续保 / 退保状态） |
| 9 | `policy_beneficiaries` | 保单受益人（占比合计 100%） |
| 10 | `contracts` | 电子合同（Mock 签署状态） |
| 11 | `claims` | 理赔（报案 / 审核 / 赔付金额） |
| 12 | `claim_documents` | 理赔资料元数据 |
| 13 | `audit_logs` | 操作审计 |
| 14 | `quote_rates` | 报价费率表（产品 / 保障期 / 保额区间） |

## 4. 关键字段与约束

### users
`username` 唯一、`openid` 唯一、`password_hash`（argon2）、`role`（USER/AGENT/OPERATOR/ADMIN）、`token_version`（令牌吊销）、`status`。

### insurance_products
`product_code` 唯一；`product_type`（LIFE/HEALTH/ACCIDENT/TRAVEL/PROPERTY…）；`sale_channel`（ONLINE/AGENT/BROKER/OFFLINE）；`status`（DRAFT/ON_SALE/OFF_SHELF/DISCONTINUED）；`is_featured`；`search_enabled`（保留字段，当前搜索走 MySQL LIKE）。

### quotes
`quote_no` 唯一；`product_id` / `user_id` 外键；`holder_id`（可空，内联被保人信息时为空）；`insurance_amount`、`premium`、`premium_detail` JSON；`status`（PENDING/APPROVED/REJECTED/EXPIRED/CONVERTED/CANCELLED）；`expires_at`。

### orders
`order_no` 唯一；`quote_id` / `user_id` / `product_id`；产品名与被保人快照；`status` 与金额；`order_no` 独立业务号。

### payments
`payment_no` 唯一；`provider`（默认 `MOCK`）+ `provider_tx_id` 唯一约束保证回调幂等；`amount`；`status`（CREATED/…）。

### policies
`policy_no` 唯一；`order_id` 外键；`issue_type`（NEW/RENEW）；`status`（PENDING_ISSUE/ACTIVE/LAPSED/…）；保额与保费快照。

### contracts
`contract_no` 唯一；`policy_id` 唯一；`contract_type`（POLICY/ENDORSEMENT/RIDER）；`provider`（默认 `MOCK`）；`status`（DRAFT/…）；签署 URL / 回调由 `contract_service.rs` 内联 Mock 处理。

### claims / claim_documents
`claim_no` 唯一；`policy_id` / `order_id` 外键；`status`（SUBMITTED/…）；`claim_amount` / `approved_amount`；资料元数据表按 claim 归属校验。

### quote_rates
产品 / 保障期 / 保额区间 → 费率；报价命中优先，未命中回退请求保费。

## 5. 敏感数据加密

- 身份证：`holder_id_card_enc`、受益人 `id_card_enc` 等 `VARBINARY(1024)` 密文列。
- 手机号 / 银行卡号：对应 `*_enc` 列；读写经 `CryptoService`（AES-256-GCM）。
- 密文不参与搜索 / 列表展示；展示走脱敏（`Masker`）。

## 6. 枚举与状态

状态枚举以 `install.sql` 列注释为准；业务层在写入前校验合法流转，禁止跨状态跳变。审计表 `audit_logs` 记录操作人、来源端、目标与结果，供运营端查询。
