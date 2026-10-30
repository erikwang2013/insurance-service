# 保险服务平台 — 数据库 Schema

> 版本: 2026-10-30（v1.11.0）。以 `install.sql` 为准，共 14 张业务表。

---

## 1. 总体设计原则

- 主键：`BIGINT UNSIGNED`，由应用层 snowflake 生成（`idgen_rs`），非自增。
- 业务号：报价 QT / 订单 OD / 保单 POL / 合同 CT / 理赔 CLM / 支付 PAY，独立生成并建唯一索引。
- 金额：`DECIMAL(14,2)`，单位元；禁止浮点。报价保费由服务端按保额 × 费率计算；
  订单 `discount_amount` 恒为 0，应付 = 报价保费。
- 时间：`DATETIME(3)`；业务状态用 `VARCHAR` 显式枚举，禁止任意跳变。
- 软删除：`deleted_at`；关键操作写入 `audit_logs`（含 `trace_id`）。
- 敏感字段：`users.phone_enc` 等用户敏感列由服务端 AES-256-GCM 加密；
  报价 / 保单受益人表的 `*_enc` 列保存上游传入字节，服务端不解密、不重新加密。
- 搜索：MySQL `LIKE`，仅 `ON_SALE` + `search_enabled=1`；不依赖外部索引或异步同步队列。

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
| 8 | `policies` | 保单（签发 / 退保状态；续保暂未开放） |
| 9 | `policy_beneficiaries` | 保单受益人（占比合计 100%） |
| 10 | `contracts` | 电子合同（Mock 签署状态） |
| 11 | `claims` | 理赔（报案 / 审核 / 赔付金额） |
| 12 | `claim_documents` | 理赔资料元数据 |
| 13 | `audit_logs` | 操作审计 |
| 14 | `quote_rates` | 报价费率表（产品 / 保障期 / 保额区间） |

## 4. 关键字段与约束

### users
`username` 唯一、`openid` 唯一、`password_hash`（argon2）、`phone_enc`（服务端 AES-256-GCM 密文）、
`phone_masked`、`role`（USER/AGENT/OPERATOR/ADMIN）、`token_version`（令牌吊销）、`status`。

### insurance_products
`product_code` 唯一；`product_type`（LIFE/HEALTH/ACCIDENT/TRAVEL/PROPERTY…）；`sale_channel`（ONLINE/AGENT/BROKER/OFFLINE）；
`status`（DRAFT/ON_SALE/OFF_SHELF/DISCONTINUED）；`is_featured`；
`search_enabled`（搜索开关：公开搜索仅命中 `ON_SALE` 且开启该开关的商品）。

### quotes
`quote_no` 唯一（`QT` 前缀）；`product_id` / `user_id` 外键；`holder_id`（可空，内联被保人信息时为空）；
`insurance_amount`、`premium`、`premium_detail` JSON；`status`（PENDING/APPROVED/REJECTED/EXPIRED/CONVERTED/CANCELLED）；
`expires_at`。保费恒由服务端按 `quote_rates` 计算（保额 × 费率），客户端 `premium` 被忽略；有效期 7 天。

### orders
`order_no` 唯一（`OD` 前缀）；`quote_id` / `user_id` / `product_id`；产品名与被保人快照；
`discount_amount` 恒为 0，`payable_amount` = 报价保费；报价一次性转为 `CONVERTED`。

### payments
`payment_no` 唯一（`PAY` 前缀）；`provider`（本阶段恒为 `MOCK`）+ `provider_tx_id` 唯一约束保证回调幂等；
`amount`；`status`（CREATED/SUCCESS/FAILED…）；回调需 `X-Callback-Token`，成功回调事务内同时更新订单。

### policies
`policy_no` 唯一（`POL` 前缀）；`order_id` 外键；`issue_type`（NEW/RENEW 保留，续保暂未开放）；
`status`（PENDING_ISSUE/ACTIVE/LAPSED/SURRENDERED…）；保额与保费快照；订单 PAID 后签发（幂等）。

### contracts
`contract_no` 唯一（`CT` 前缀）；`policy_id` 唯一；`contract_type`（POLICY/ENDORSEMENT/RIDER）；
`provider`（默认 `MOCK`）；`status`（DRAFT/…）；签署 URL / 回调由 `contract_service.rs` 内联 Mock 处理。

### claims / claim_documents
`claim_no` 唯一（`CLM` 前缀）；`policy_id` / `order_id` 外键；`status`（SUBMITTED/…）；
`claim_amount` / `approved_amount`；事故日期须在保单有效期内，金额不超过保额；资料元数据表按 claim 归属校验。

### quote_rates
产品 / 保障期 / 保额区间 → 费率；报价必须命中费率行，未命中直接拒绝（不再回退客户端保费）。

### audit_logs
记录操作人、来源端、目标、结果与 `trace_id`；保单退保 / 受益人批改在审计插入时写入当前请求 trace_id。

## 5. 敏感数据加密

- `users.phone_enc`：注册 / 换绑时由 `CryptoService`（AES-256-GCM）加密写入，并另存 `phone_masked` 供展示。
- `users.id_card_enc` 与报价 / 保单受益人的 `holder_id_card_enc` / `id_card_enc` 为 `VARBINARY` 列；
  其中报价、受益人相关列直接保存上游传入字节，服务端不解密、不重新加密。
- 密文不参与搜索 / 列表展示；展示走脱敏（`Masker`）。

## 6. 枚举与状态

状态枚举以 `install.sql` 列注释为准；业务层在写入前校验合法流转，禁止跨状态跳变。审计表 `audit_logs` 记录操作人、来源端、目标与结果，供运营端查询。
