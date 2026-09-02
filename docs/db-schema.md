# 保险服务平台 — MySQL 数据库 Schema 与领域模型规划

> 版本: v1.0 | 日期: 2026-09-01 | 状态: 待评审
> 适用: 后端 `bee-rust` + `bee_orm #[derive(Model)]`(feature `mysql`) + MySQL 8
> 搜索: OpenSearch + rust-scout(业务模型实现 `Searchable` trait 同步索引)
> 说明: 本文档为 `install.sql` 与 Rust `models/` 的编写蓝本;所有枚举采用**稳定大写字符串**(对齐前端已约定的 `order.status = "PENDING"|"PAID"|"CANCELLED"` 语义)。

---

## 目录

1. [总体设计原则](#1-总体设计原则)
2. [ER 关系总览](#2-er-关系总览)
3. [表清单](#3-表清单)
4. [DDL 建表 SQL(install.sql 蓝本)](#4-ddl-建表-sql)
5. [各表字段 / 索引 / 外键 / 状态机详解](#5-各表字段--索引--外键--状态机详解)
6. [bee_orm Rust 模型结构体](#6-bee_orm-rust-模型结构体)
7. [rust-scout Searchable 检索设计](#7-rust-scout-searchable-检索设计)
8. [敏感数据字段级加密](#8-敏感数据字段级加密)
9. [数据一致性:search_sync_logs 最终一致同步](#9-数据一致性search_sync_logs-最终一致同步)
10. [附录:枚举常量汇总](#10-附录枚举常量汇总)

---

## 1. 总体设计原则

1. **业务真实落地**:投保人 / 被保人 / 受益人三者可各不相同;一张保单可有**多个受益人**,各占比例合计 100%(`beneficiary_share`);被保人可独立于投保账户存在(`policy_holders` 表),以支持"为他人投保"。
2. **状态机显式化**:订单 / 保单 / 合同 / 支付 / 理赔均以 `status` 字符串列承载,业务逻辑用枚举常量校验合法流转,禁止任意跳变。
3. **金额语义**:保费、支付金额一律用 `DECIMAL(14,2)`,单位为**元**,禁止浮点。金额快照落库(订单价、报价、保单保费各自独立),不实时反查产品费率。
4. **软删除与审计**:核心业务表保留 `deleted_at`(软删除)+ `audit_logs` 全量操作审计。
5. **搜索解耦**:业务写入只落 MySQL;OpenSearch 索引通过 `search_sync_logs` 做**异步最终一致同步**(详见 §9),不阻塞主事务。
6. **敏感数据加密**:身份证号、手机号、银行卡号等做**字段级加密**存储(详见 §8),明文绝不直接落库。
7. **主键**:全部 `BIGINT UNSIGNED AUTO_INCREMENT`;业务号(保单号、订单号、合同号)独立生成并唯一索引,便于对外展示与搜索。

---

## 2. ER 关系总览

```
users 1──N insurance_products(产品运营/销售方)
users 1──N orders(下单人)
users 1──N quotes(投保请求发起人)
policy_holders 1──N quotes(被保人)

insurance_products 1──N insurance_product_clauses(条款)
insurance_products 1──N insurance_product_category_rel(分类多对多)
insurance_product_categories(类目表,parent_id 自关联树)

insurance_products 1──N quotes
quotes 1──N quotes_beneficiaries(报价期受益人快照)
quotes 1──1 orders
orders 1──N payments
orders 1──N policies(一单可拆多张保单)

policies 1──N policy_beneficiaries(受益人+占比)
policies 1──1 contracts(主合同,可扩展 1──N)
contracts 1──N contract_signers(签署方)

orders 1──N claims(理赔挂订单/保单)
policies 1──N claims

所有业务写入 → search_sync_logs(同步队列)
所有关键操作 → audit_logs(审计)
```

---

## 3. 表清单

| # | 表名 | 说明 | 关联主实体 |
|---|------|------|-----------|
| 1 | `users` | 用户 / 账户(投保人账户) | — |
| 2 | `policy_holders` | 被保人档案(可独立于用户) | users |
| 3 | `insurance_products` | 保险产品 | users |
| 4 | `insurance_product_clauses` | 产品条款 | products |
| 5 | `insurance_product_categories` | 产品分类(树) | — |
| 6 | `insurance_product_category_rel` | 产品-分类多对多 | products, categories |
| 7 | `quotes` | 报价 / 投保方案 | products, users, holders |
| 8 | `quotes_beneficiaries` | 报价期受益人快照 | quotes |
| 9 | `orders` | 订单 | quotes, users |
| 10 | `payments` | 支付流水 | orders |
| 11 | `policies` | 保单 | orders |
| 12 | `policy_beneficiaries` | 保单受益人(占比) | policies |
| 13 | `contracts` | 电子合同 | policies |
| 14 | `contract_signers` | 合同签署方 | contracts, users |
| 15 | `claims` | 理赔 | orders, policies |
| 16 | `search_sync_logs` | DB→OpenSearch 同步队列 | — |
| 17 | `audit_logs` | 操作审计 | — |

---

## 4. DDL 建表 SQL(install.sql 蓝本)

以下为完整可执行 DDL。统一约定:
- 字符集 `utf8mb4`、排序 `utf8mb4_unicode_ci`
- 引擎 `InnoDB`
- 时间列 `DATETIME(3)`(毫秒级),`created_at` 默认 `CURRENT_TIMESTAMP(3)`
- 业务号/状态等对外字段加唯一或普通索引

```sql
CREATE DATABASE IF NOT EXISTS insurance_service
  DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
USE insurance_service;

-- ============================================================
-- 1. users 用户账户
-- ============================================================
CREATE TABLE users (
  id            BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  username      VARCHAR(64)  NOT NULL,
  -- 以下两个敏感字段存储加密密文(见 §8),普通长度不足以放 AES 密文,故用 TEXT
  phone_enc     VARBINARY(512)  NULL,      -- 手机号 AES 密文
  id_card_enc   VARBINARY(1024) NULL,      -- 身份证号 AES 密文
  password_hash VARCHAR(128)  NOT NULL,    -- bcrypt/argon2
  email         VARCHAR(128)  NULL,
  nickname      VARCHAR(64)   NULL,
  avatar_url    VARCHAR(512)  NULL,
  role          VARCHAR(32)   NOT NULL DEFAULT 'USER',  -- USER / ADMIN / OPERATOR
  status        VARCHAR(32)   NOT NULL DEFAULT 'ACTIVE',-- ACTIVE / DISABLED / FROZEN
  phone_masked  VARCHAR(20)   NULL,        -- 脱敏展示值 138****1234
  last_login_at DATETIME(3)   NULL,
  created_at    DATETIME(3)   NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
  updated_at    DATETIME(3)   NOT NULL DEFAULT CURRENT_TIMESTAMP(3)
                           ON UPDATE CURRENT_TIMESTAMP(3),
  deleted_at    DATETIME(3)   NULL,
  UNIQUE KEY uk_username (username),
  KEY idx_phone_masked (phone_masked)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
```

```sql
-- ============================================================
-- 2. policy_holders 被保人档案
-- ============================================================
CREATE TABLE policy_holders (
  id            BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  user_id       BIGINT UNSIGNED NULL,        -- 关联投保人账户;为他人投保时为 NULL
  name          VARCHAR(64)   NOT NULL,      -- 被保人姓名
  id_card_enc   VARBINARY(1024) NULL,        -- 身份证密文
  id_type       VARCHAR(32)   NOT NULL DEFAULT 'ID_CARD', -- ID_CARD/PASSPORT/OTHER
  gender        VARCHAR(16)   NULL,          -- MALE / FEMALE / UNKNOWN
  birthday      DATE          NULL,
  phone_enc     VARBINARY(512) NULL,         -- 手机号密文
  email         VARCHAR(128)  NULL,
  address       VARCHAR(255)  NULL,
  relationship  VARCHAR(32)   NULL,          -- 与投保人关系 SELF/SPOUSE/CHILD/PARENT/OTHER
  status        VARCHAR(32)   NOT NULL DEFAULT 'ACTIVE', -- ACTIVE / DELETED
  created_at    DATETIME(3)   NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
  updated_at    DATETIME(3)   NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
  deleted_at    DATETIME(3)   NULL,
  KEY idx_holder_user (user_id),
  KEY idx_holder_name (name)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================================
-- 3. insurance_products 保险产品
-- ============================================================
CREATE TABLE insurance_products (
  id               BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  product_code     VARCHAR(64)   NOT NULL,        -- 产品编码,对外唯一
  name             VARCHAR(128)  NOT NULL,
  subtitle         VARCHAR(255)  NULL,            -- 副标题/卖点
  description      TEXT          NULL,            -- 产品介绍
  product_type     VARCHAR(32)   NOT NULL,        -- LIFE/HEALTH/ACCIDENT/TRAVEL/PROPERTY/...
  sale_channel     VARCHAR(32)   NOT NULL DEFAULT 'ONLINE', -- ONLINE/AGENT/BROKER/OFFLINE
  operator_user_id BIGINT UNSIGNED NULL,          -- 运营/销售方用户
  insurer_name     VARCHAR(128)  NULL,            -- 承保保险公司名称
  currency         VARCHAR(8)    NOT NULL DEFAULT 'CNY',
  min_amount       DECIMAL(14,2) NULL,            -- 最低保额(元)
  max_amount       DECIMAL(14,2) NULL,            -- 最高保额
  min_term_months  INT           NULL,            -- 最短保障期(月)
  max_term_months  INT           NULL,            -- 最长保障期
  waiting_period_days INT        NULL DEFAULT 0,  -- 等待期(天)
  is_featured      TINYINT(1)    NOT NULL DEFAULT 0, -- 首页推荐
  cover_image_url  VARCHAR(512)  NULL,
  status           VARCHAR(32)   NOT NULL DEFAULT 'DRAFT', -- DRAFT/ON_SALE/OFF_SHELF/DISCONTINUED
  search_enabled   TINYINT(1)    NOT NULL DEFAULT 1, -- 是否入 OpenSearch
  created_at       DATETIME(3)   NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
  updated_at       DATETIME(3)   NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
  deleted_at       DATETIME(3)   NULL,
  UNIQUE KEY uk_product_code (product_code),
  KEY idx_product_type (product_type),
  KEY idx_product_status (status),
  KEY idx_product_featured (is_featured),
  KEY idx_product_sale_channel (sale_channel),
  KEY idx_product_operator (operator_user_id),
  FULLTEXT KEY ft_product_name (name, subtitle)   -- 可选:MySQL 内置全文兜底
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================================
-- 4. insurance_product_clauses 产品条款
-- ============================================================
CREATE TABLE insurance_product_clauses (
  id          BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  product_id  BIGINT UNSIGNED NOT NULL,
  clause_type VARCHAR(32)   NOT NULL,   -- MAIN/EXCLUSION/WAIVER/RIDER/OBLIGATION
  title       VARCHAR(255)  NOT NULL,
  content     LONGTEXT      NOT NULL,   -- 条款正文(Markdown/HTML)
  sort_order  INT           NOT NULL DEFAULT 0,
  is_required TINYINT(1)    NOT NULL DEFAULT 1, -- 是否必须勾选阅读
  version     VARCHAR(32)   NOT NULL DEFAULT 'v1.0',
  status      VARCHAR(32)   NOT NULL DEFAULT 'ACTIVE', -- ACTIVE / DEPRECATED
  created_at  DATETIME(3)   NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
  updated_at  DATETIME(3)   NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
  deleted_at  DATETIME(3)   NULL,
  KEY idx_clause_product (product_id),
  KEY idx_clause_type (clause_type),
  CONSTRAINT fk_clause_product FOREIGN KEY (product_id)
    REFERENCES insurance_products (id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================================
-- 5. insurance_product_categories 产品分类(树)
-- ============================================================
CREATE TABLE insurance_product_categories (
  id          BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  parent_id   BIGINT UNSIGNED NULL,     -- 父分类,根为 NULL
  name        VARCHAR(64)   NOT NULL,
  slug        VARCHAR(64)   NOT NULL,
  sort_order  INT           NOT NULL DEFAULT 0,
  status      VARCHAR(32)   NOT NULL DEFAULT 'ACTIVE', -- ACTIVE / HIDDEN
  created_at  DATETIME(3)   NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
  updated_at  DATETIME(3)   NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
  deleted_at  DATETIME(3)   NULL,
  UNIQUE KEY uk_category_slug (slug),
  KEY idx_category_parent (parent_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================================
-- 6. insurance_product_category_rel 产品-分类(多对多)
-- ============================================================
CREATE TABLE insurance_product_category_rel (
  id          BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  product_id  BIGINT UNSIGNED NOT NULL,
  category_id BIGINT UNSIGNED NOT NULL,
  created_at  DATETIME(3)   NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
  UNIQUE KEY uk_prod_cat (product_id, category_id),
  KEY idx_cat_rel_category (category_id),
  CONSTRAINT fk_catrel_product FOREIGN KEY (product_id)
    REFERENCES insurance_products (id) ON DELETE CASCADE,
  CONSTRAINT fk_catrel_category FOREIGN KEY (category_id)
    REFERENCES insurance_product_categories (id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
```

```sql
-- ============================================================
-- 7. quotes 报价 / 投保方案
-- ============================================================
CREATE TABLE quotes (
  id                 BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  quote_no           VARCHAR(32)   NOT NULL,          -- 报价单号
  product_id         BIGINT UNSIGNED NOT NULL,
  user_id            BIGINT UNSIGNED NOT NULL,        -- 投保人账户
  holder_id          BIGINT UNSIGNED NULL,            -- 被保人档案(可空,内联信息)
  holder_name        VARCHAR(64)   NULL,              -- 被保人姓名快照
  holder_id_card_enc VARBINARY(1024) NULL,            -- 被保人身份证密文
  insurance_amount   DECIMAL(14,2) NOT NULL,          -- 保额
  term_months        INT           NOT NULL,          -- 保障期(月)
  premium            DECIMAL(14,2) NOT NULL,          -- 试算保费
  premium_detail     JSON          NULL,              -- 保费构成明细 {base, extra, discount, total}
  effective_date     DATE          NULL,              -- 期望生效日
  expire_date        DATE          NULL,              -- 期望到期日
  health_declaration JSON          NULL,              -- 健康告知问卷答案
  risk_score         INT           NULL,              -- 核保风险分(0-100)
  status             VARCHAR(32)   NOT NULL DEFAULT 'PENDING', -- PENDING/APPROVED/REJECTED/EXPIRED/CONVERTED/CANCELLED
  expires_at         DATETIME(3)   NOT NULL,          -- 报价有效期
  created_at         DATETIME(3)   NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
  updated_at         DATETIME(3)   NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
  deleted_at         DATETIME(3)   NULL,
  UNIQUE KEY uk_quote_no (quote_no),
  KEY idx_quote_product (product_id),
  KEY idx_quote_user (user_id),
  KEY idx_quote_holder (holder_id),
  KEY idx_quote_status (status),
  KEY idx_quote_expires (expires_at),
  CONSTRAINT fk_quote_product FOREIGN KEY (product_id)
    REFERENCES insurance_products (id),
  CONSTRAINT fk_quote_user FOREIGN KEY (user_id)
    REFERENCES users (id),
  CONSTRAINT fk_quote_holder FOREIGN KEY (holder_id)
    REFERENCES policy_holders (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================================
-- 8. quotes_beneficiaries 报价期受益人快照
-- ============================================================
CREATE TABLE quotes_beneficiaries (
  id           BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  quote_id     BIGINT UNSIGNED NOT NULL,
  name         VARCHAR(64)   NOT NULL,
  id_card_enc  VARBINARY(1024) NULL,
  relationship VARCHAR(32)   NULL,       -- SELF/SPOUSE/CHILD/PARENT/OTHER
  beneficiary_type VARCHAR(16) NOT NULL DEFAULT 'LEGAL', -- LEGAL(法定)/NAMED(指定)
  share_percent DECIMAL(5,2)  NULL,      -- 占比(0-100),指定受益人时使用,合计100
  sort_order   INT           NOT NULL DEFAULT 0,
  created_at   DATETIME(3)   NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
  KEY idx_qben_quote (quote_id),
  CONSTRAINT fk_qben_quote FOREIGN KEY (quote_id)
    REFERENCES quotes (id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================================
-- 9. orders 订单
-- ============================================================
CREATE TABLE orders (
  id            BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  order_no      VARCHAR(32)   NOT NULL,             -- 订单号
  quote_id      BIGINT UNSIGNED NOT NULL,
  user_id       BIGINT UNSIGNED NOT NULL,           -- 下单人
  product_id    BIGINT UNSIGNED NOT NULL,
  product_name  VARCHAR(128)  NOT NULL,             -- 产品名快照
  holder_name   VARCHAR(64)   NOT NULL,             -- 被保人快照
  insurance_amount DECIMAL(14,2) NOT NULL,
  term_months   INT           NOT NULL,
  total_amount  DECIMAL(14,2) NOT NULL,             -- 应付总额
  discount_amount DECIMAL(14,2) NOT NULL DEFAULT 0,
  payable_amount  DECIMAL(14,2) NOT NULL,           -- 实付(应付-优惠)
  currency      VARCHAR(8)    NOT NULL DEFAULT 'CNY',
  status        VARCHAR(32)   NOT NULL DEFAULT 'CREATED',
  -- 状态机: CREATED → PAID → POLICY_ISSUED → COMPLETED
  --                └→ CANCELLED / EXPIRED / REFUNDING → REFUNDED
  paid_at       DATETIME(3)   NULL,
  policy_issued_at DATETIME(3) NULL,
  cancelled_at  DATETIME(3)   NULL,
  remark        VARCHAR(255)  NULL,
  created_at    DATETIME(3)   NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
  updated_at    DATETIME(3)   NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
  deleted_at    DATETIME(3)   NULL,
  UNIQUE KEY uk_order_no (order_no),
  KEY idx_order_quote (quote_id),
  KEY idx_order_user (user_id),
  KEY idx_order_product (product_id),
  KEY idx_order_status (status),
  KEY idx_order_created (created_at),
  CONSTRAINT fk_order_quote FOREIGN KEY (quote_id) REFERENCES quotes (id),
  CONSTRAINT fk_order_user FOREIGN KEY (user_id) REFERENCES users (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================================
-- 10. payments 支付流水
-- ============================================================
CREATE TABLE payments (
  id              BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  payment_no      VARCHAR(32)   NOT NULL,            -- 支付流水号
  order_id        BIGINT UNSIGNED NOT NULL,
  user_id         BIGINT UNSIGNED NOT NULL,
  amount          DECIMAL(14,2) NOT NULL,
  currency        VARCHAR(8)    NOT NULL DEFAULT 'CNY',
  channel         VARCHAR(32)   NOT NULL,            -- WECHAT/ALIPAY/UNIONPAY/BALANCE/MOCK
  provider        VARCHAR(32)   NOT NULL DEFAULT 'MOCK', -- PayProvider 实现名,预留 WECHAT
  provider_tx_id  VARCHAR(128)  NULL,                -- 支付渠道交易号
  status          VARCHAR(32)   NOT NULL DEFAULT 'CREATED',
  -- 状态机: CREATED → PROCESSING → SUCCESS
  --                └→ FAILED / CANCELLED / REFUNDED
  prepay_payload  JSON          NULL,                -- 预支付参数(前端拉起收银台)
  callback_payload JSON         NULL,                -- 渠道回调原始报文(审计留痕)
  paid_at         DATETIME(3)   NULL,
  refunded_at     DATETIME(3)   NULL,
  created_at      DATETIME(3)   NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
  updated_at      DATETIME(3)   NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
  UNIQUE KEY uk_payment_no (payment_no),
  UNIQUE KEY uk_payment_tx (provider, provider_tx_id),
  KEY idx_payment_order (order_id),
  KEY idx_payment_user (user_id),
  KEY idx_payment_status (status),
  CONSTRAINT fk_payment_order FOREIGN KEY (order_id)
    REFERENCES orders (id) ON DELETE CASCADE,
  CONSTRAINT fk_payment_user FOREIGN KEY (user_id) REFERENCES users (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
```

```sql
-- ============================================================
-- 11. policies 保单
-- ============================================================
CREATE TABLE policies (
  id                  BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  policy_no           VARCHAR(32)   NOT NULL,          -- 保单号,对外展示
  order_id            BIGINT UNSIGNED NOT NULL,
  quote_id            BIGINT UNSIGNED NOT NULL,
  user_id             BIGINT UNSIGNED NOT NULL,        -- 投保人
  holder_id           BIGINT UNSIGNED NULL,            -- 被保人档案
  product_id          BIGINT UNSIGNED NOT NULL,
  product_name        VARCHAR(128)  NOT NULL,
  holder_name         VARCHAR(64)   NOT NULL,          -- 被保人姓名
  holder_id_card_enc  VARBINARY(1024) NULL,
  insurance_amount    DECIMAL(14,2) NOT NULL,          -- 保额
  premium             DECIMAL(14,2) NOT NULL,          -- 实缴保费
  term_months         INT           NOT NULL,
  effective_date      DATE          NOT NULL,          -- 保险起期
  expire_date         DATE          NOT NULL,          -- 保险止期
  status              VARCHAR(32)   NOT NULL DEFAULT 'PENDING_ISSUE',
  -- 状态机: PENDING_ISSUE → ACTIVE → EXPIRED
  --              └→ CANCELLED / SURRENDERED / LAPSED
  issue_type          VARCHAR(16)   NOT NULL DEFAULT 'NEW', -- NEW/RENEW
  is_renewable        TINYINT(1)    NOT NULL DEFAULT 1,
  pdf_path            VARCHAR(512)  NULL,              -- 电子保单 PDF 存储路径
  premium_detail      JSON          NULL,
  issued_at           DATETIME(3)   NULL,
  created_at          DATETIME(3)   NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
  updated_at          DATETIME(3)   NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
  deleted_at          DATETIME(3)   NULL,
  UNIQUE KEY uk_policy_no (policy_no),
  KEY idx_policy_order (order_id),
  KEY idx_policy_user (user_id),
  KEY idx_policy_holder (holder_id),
  KEY idx_policy_product (product_id),
  KEY idx_policy_status (status),
  KEY idx_policy_holder_name (holder_name),
  KEY idx_policy_expire (expire_date),
  CONSTRAINT fk_policy_order FOREIGN KEY (order_id) REFERENCES orders (id),
  CONSTRAINT fk_policy_user FOREIGN KEY (user_id) REFERENCES users (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================================
-- 12. policy_beneficiaries 保单受益人(占比)
-- ============================================================
CREATE TABLE policy_beneficiaries (
  id           BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  policy_id    BIGINT UNSIGNED NOT NULL,
  name         VARCHAR(64)   NOT NULL,
  id_card_enc  VARBINARY(1024) NULL,
  relationship VARCHAR(32)   NULL,        -- SELF/SPOUSE/CHILD/PARENT/OTHER
  beneficiary_type VARCHAR(16) NOT NULL DEFAULT 'LEGAL', -- LEGAL/NAMED
  share_percent DECIMAL(5,2)  NULL,       -- 占比(0-100),NAMED 时使用,同单合计=100
  sort_order   INT           NOT NULL DEFAULT 0,
  created_at   DATETIME(3)   NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
  updated_at   DATETIME(3)   NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
  KEY idx_pben_policy (policy_id),
  CONSTRAINT fk_pben_policy FOREIGN KEY (policy_id)
    REFERENCES policies (id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================================
-- 13. contracts 电子合同
-- ============================================================
CREATE TABLE contracts (
  id            BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  contract_no   VARCHAR(32)   NOT NULL,             -- 合同号
  policy_id     BIGINT UNSIGNED NOT NULL,
  order_id      BIGINT UNSIGNED NOT NULL,
  title         VARCHAR(128)  NOT NULL,             -- 合同标题
  contract_type VARCHAR(32)   NOT NULL DEFAULT 'POLICY', -- POLICY/ENDORSEMENT/RIDER
  pdf_path      VARCHAR(512)  NULL,                 -- 最终合同 PDF
  file_hash     VARCHAR(128)  NULL,                 -- 合同 PDF 防篡改摘要(SHA-256)
  sign_flow_id  VARCHAR(128)  NULL,                 -- 电子签服务端流程 ID(预留 e签宝)
  provider      VARCHAR(32)   NOT NULL DEFAULT 'MOCK', -- ElectronicSignature 实现名
  status        VARCHAR(32)   NOT NULL DEFAULT 'DRAFT',
  -- 状态机: DRAFT → PENDING_SIGN → SIGNING → COMPLETED
  --              └→ VOID / EXPIRED / REJECTED
  signed_at     DATETIME(3)   NULL,
  created_at    DATETIME(3)   NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
  updated_at    DATETIME(3)   NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
  deleted_at    DATETIME(3)   NULL,
  UNIQUE KEY uk_contract_no (contract_no),
  UNIQUE KEY uk_contract_policy (policy_id),
  KEY idx_contract_order (order_id),
  KEY idx_contract_status (status),
  CONSTRAINT fk_contract_policy FOREIGN KEY (policy_id)
    REFERENCES policies (id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================================
-- 14. contract_signers 合同签署方
-- ============================================================
CREATE TABLE contract_signers (
  id            BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  contract_id   BIGINT UNSIGNED NOT NULL,
  user_id       BIGINT UNSIGNED NULL,               -- 登录用户签署
  name          VARCHAR(64)   NOT NULL,             -- 签署人姓名
  signer_type   VARCHAR(32)   NOT NULL DEFAULT 'APPLICANT', -- APPLICANT/INSURED/BENEFICIARY/WITNESS
  sign_order    INT           NOT NULL DEFAULT 0,   -- 签署顺序
  status        VARCHAR(32)   NOT NULL DEFAULT 'PENDING',
  -- 状态机: PENDING → SIGNING → SIGNED → COMPLETED
  --              └→ REJECTED / ABANDONED
  sign_url      VARCHAR(512)  NULL,                 -- 签署链接(电子签平台)
  sign_token    VARCHAR(128)  NULL,                 -- 签署凭证
  signed_at     DATETIME(3)   NULL,
  sign_detail   JSON          NULL,                 -- 签署环境/IP/时间/落款坐标
  created_at    DATETIME(3)   NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
  updated_at    DATETIME(3)   NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
  KEY idx_csign_contract (contract_id),
  KEY idx_csign_user (user_id),
  KEY idx_csign_status (status),
  CONSTRAINT fk_csign_contract FOREIGN KEY (contract_id)
    REFERENCES contracts (id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================================
-- 15. claims 理赔
-- ============================================================
CREATE TABLE claims (
  id              BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  claim_no        VARCHAR(32)   NOT NULL,            -- 理赔单号
  policy_id       BIGINT UNSIGNED NOT NULL,
  order_id        BIGINT UNSIGNED NOT NULL,
  user_id         BIGINT UNSIGNED NOT NULL,          -- 报案人
  accident_date   DATE          NULL,                -- 出险日期
  accident_type   VARCHAR(64)   NULL,                -- 出险类型/原因
  accident_desc   TEXT          NULL,                -- 事故描述
  claim_amount    DECIMAL(14,2) NOT NULL,            -- 申请赔付金额
  approved_amount DECIMAL(14,2) NULL,                -- 核定赔付金额
  status          VARCHAR(32)   NOT NULL DEFAULT 'SUBMITTED',
  -- 状态机: SUBMITTED → UNDER_REVIEW → PENDING_INFO → REVIEWING
  --                 → APPROVED → PAID / REJECTED / CLOSED
  --                 └→ WITHDRAWN
  reviewer_id     BIGINT UNSIGNED NULL,
  review_remark   VARCHAR(255)  NULL,
  pay_ref         VARCHAR(64)   NULL,                -- 关联 payments.id 或渠道回执
  submitted_at    DATETIME(3)   NULL,
  paid_at         DATETIME(3)   NULL,
  created_at      DATETIME(3)   NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
  updated_at      DATETIME(3)   NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
  deleted_at      DATETIME(3)   NULL,
  UNIQUE KEY uk_claim_no (claim_no),
  KEY idx_claim_policy (policy_id),
  KEY idx_claim_order (order_id),
  KEY idx_claim_user (user_id),
  KEY idx_claim_status (status),
  CONSTRAINT fk_claim_policy FOREIGN KEY (policy_id) REFERENCES policies (id),
  CONSTRAINT fk_claim_order FOREIGN KEY (order_id) REFERENCES orders (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
```

```sql
-- ============================================================
-- 16. search_sync_logs DB→OpenSearch 同步队列
-- ============================================================
CREATE TABLE search_sync_logs (
  id            BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  entity_type   VARCHAR(32)   NOT NULL,   -- PRODUCT / CLAUSE / POLICY
  entity_id     BIGINT UNSIGNED NOT NULL, -- 业务实体主键
  op            VARCHAR(16)   NOT NULL,   -- UPSERT / DELETE
  status        VARCHAR(16)   NOT NULL DEFAULT 'PENDING',
  -- 状态机: PENDING → PROCESSING → SUCCESS
  --              └→ FAILED → RETRYING → SUCCESS/DEAD
  attempts      INT           NOT NULL DEFAULT 0,  -- 已重试次数
  max_attempts  INT           NOT NULL DEFAULT 5,
  next_retry_at DATETIME(3)   NULL,               -- 下次重试时间(指数退避)
  last_error    VARCHAR(512)  NULL,               -- 最近一次失败原因
  payload_json  JSON          NULL,               -- 待写入索引文档快照(幂等重放)
  created_at    DATETIME(3)   NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
  updated_at    DATETIME(3)   NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
  processed_at  DATETIME(3)   NULL,
  KEY idx_synclog_status (status, next_retry_at),  -- 扫描待处理任务
  KEY idx_synclog_entity (entity_type, entity_id)  -- 幂等去重/重放
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================================
-- 17. audit_logs 操作审计
-- ============================================================
CREATE TABLE audit_logs (
  id           BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  user_id      BIGINT UNSIGNED NULL,             -- 操作人
  action       VARCHAR(64)   NOT NULL,           -- ORDER_PAY / POLICY_ISSUE / CONTRACT_SIGN ...
  entity_type  VARCHAR(32)   NOT NULL,           -- ORDER / POLICY / CONTRACT / PAYMENT ...
  entity_id    BIGINT UNSIGNED NOT NULL,
  before_json  JSON          NULL,               -- 变更前快照
  after_json   JSON          NULL,               -- 变更后快照
  ip           VARCHAR(64)   NULL,
  user_agent   VARCHAR(255)  NULL,
  trace_id     VARCHAR(64)   NULL,               -- 与响应 ResponseEnvelope.trace_id 对齐
  created_at   DATETIME(3)   NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
  KEY idx_audit_entity (entity_type, entity_id),
  KEY idx_audit_user (user_id),
  KEY idx_audit_action (action),
  KEY idx_audit_created (created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
```

---

## 5. 各表字段 / 索引 / 外键 / 状态机详解

### 5.1 users — 用户账户

- **关键字段**:`username`(唯一)、`phone_enc`/`id_card_enc`(密文)、`password_hash`、`role`、`status`。
- **敏感处理**:手机号/身份证号存 AES 密文(`VARBINARY`),另存 `phone_masked`(脱敏串)供列表展示;明文仅内存中解密,不落库、不打日志。
- **索引**:`uk_username`、`idx_phone_masked`。
- **状态机** `status`: `ACTIVE → DISABLED / FROZEN`。

### 5.2 policy_holders — 被保人档案

- **作用**:支持"为他人投保"与"同一人多次投保"复用档案;`user_id` 可空。
- **字段**:`name`、`id_card_enc`、`id_type`、`gender`、`birthday`、`relationship`。
- **索引**:`idx_holder_user`、`idx_holder_name`。

### 5.3 insurance_products — 保险产品

- **字段**:`product_code`(唯一)、`product_type`、`sale_channel`(销售渠道)、`operator_user_id`、`insurer_name`、保额/期限区间、`is_featured`、`status`、`search_enabled`。
- **索引**:`uk_product_code` + `product_type/status/featured/sale_channel` + `FULLTEXT(name, subtitle)` 兜底。
- **状态机** `status`: `DRAFT → ON_SALE → OFF_SHELF / DISCONTINUED`。

### 5.4 insurance_product_clauses — 产品条款

- **字段**:`product_id`(FK→products)、`clause_type`、`title`、`content`(LONGTEXT)、`version`、`is_required`。
- **索引**:`idx_clause_product`、`idx_clause_type`;FK `ON DELETE CASCADE`。
- **状态机** `status`: `ACTIVE / DEPRECATED`。

### 5.5 insurance_product_categories — 分类树

- `parent_id` 自关联形成树;`slug` 唯一(URL 友好)。
- **状态机** `status`: `ACTIVE / HIDDEN`。

### 5.6 insurance_product_category_rel — 多对多

- 复合唯一 `uk_prod_cat(product_id, category_id)`,两级 FK CASCADE。

### 5.7 quotes — 报价

- **字段**:`quote_no`、`product_id`、`user_id`(投保人)、`holder_id`(被保人)、保额/期限、`premium` + `premium_detail`(JSON)、`health_declaration`、`risk_score`、`expires_at`(报价有效期)、`status`。
- **索引**:`uk_quote_no` + product/user/holder/status/expires。
- **状态机** `status`:
  - `PENDING`(待核保)→ `APPROVED`(通过)→ `CONVERTED`(已转订单)
  - `PENDING → REJECTED`(拒保)
  - `PENDING → EXPIRED`(超期未下单)
  - 任意未转换态 → `CANCELLED`。

### 5.8 quotes_beneficiaries — 报价期受益人

- 报价阶段即录入受益人快照(法定/指定+占比),下单后复制到 `policy_beneficiaries`,保证合同与报价一致。

### 5.9 orders — 订单

- **关键字段**:`order_no`、`quote_id`、`total_amount`/`discount_amount`/`payable_amount`(金额快照)、`status`、`paid_at`、`policy_issued_at`。
- **索引**:`uk_order_no` + quote/user/product/status/created。
- **状态机** `status`(核心):
  ```
  CREATED ──支付成功──▶ PAID ──保单生成──▶ POLICY_ISSUED ──完成──▶ COMPLETED
     │                    │
     │超时/用户取消         │退款
     ▼                    ▼
  CANCELLED / EXPIRED    REFUNDING ─▶ REFUNDED
  ```
  合法流转由 `OrderStatus` 枚举 + 校验函数保证,禁止跳变。

### 5.10 payments — 支付流水

- **字段**:`payment_no`、`order_id`、`channel`、`provider`(PayProvider 名,`MOCK`/预留 `WECHAT`)、`provider_tx_id`、`prepay_payload`、`callback_payload`(回调报文留痕)、`status`。
- **索引**:`uk_payment_no` + `uk_payment_tx(provider, provider_tx_id)`(幂等防重复回调)+ order/user/status。
- **状态机** `status`:
  ```
  CREATED → PROCESSING → SUCCESS
      │         │
      ▼         ▼
   CANCELLED   FAILED
   SUCCESS → REFUNDED
  ```

### 5.11 policies — 保单

- **字段**:`policy_no`(保单号,对外)、`order_id`、`holder_id`、`insurance_amount`、`premium`、`effective_date`/`expire_date`(保险期限)、`issue_type`、`pdf_path`、`status`。
- **索引**:`uk_policy_no` + order/user/holder/product/status/holder_name/expire。
- **状态机** `status`:
  ```
  PENDING_ISSUE → ACTIVE → EXPIRED
        │           │
        ▼           ├─▶ SURRENDERED(退保)
     CANCELLED      └─▶ LAPSED(失效)
  ```

### 5.12 policy_beneficiaries — 保单受益人

- **核心业务点**:一单多受益人,`beneficiary_type=NAMED` 时 `share_percent` 占比,同一保单指定受益人占比合计须 = 100(应用层校验);`LEGAL`(法定)可不填占比。
- **索引**:`idx_pben_policy`;FK `ON DELETE CASCADE`。

### 5.13 contracts — 电子合同

- **字段**:`contract_no`、`policy_id`(唯一)、`pdf_path`、`file_hash`(SHA-256 防篡改)、`sign_flow_id`、`provider`(ElectronicSignature 名,`MOCK`/预留 e签宝)、`status`。
- **索引**:`uk_contract_no` + `uk_contract_policy` + order/status。
- **状态机** `status`:
  ```
  DRAFT → PENDING_SIGN → SIGNING → COMPLETED
     │         │          │
     ▼         ▼          ▼
    VOID     EXPIRED    REJECTED
  ```

### 5.14 contract_signers — 合同签署方

- **字段**:`contract_id`、`user_id`(可空)、`signer_type`(投保人/被保人/受益人/见证人)、`sign_order`、`sign_url`、`sign_token`、`status`、`sign_detail`。
- **索引**:contract/user/status;FK CASCADE。
- **状态机** `status`: `PENDING → SIGNING → SIGNED → COMPLETED` / `REJECTED / ABANDONED`。

### 5.15 claims — 理赔

- **字段**:`claim_no`、`policy_id`、`accident_*`、`claim_amount`、`approved_amount`、`reviewer_id`、`pay_ref`、`status`。
- **索引**:`uk_claim_no` + policy/order/user/status。
- **状态机** `status`:
  ```
  SUBMITTED → UNDER_REVIEW → PENDING_INFO → REVIEWING
       │                                      │
       │                 ┌────────────────────┤
       ▼                 ▼                    ▼
    WITHDRAWN        APPROVED             REJECTED
                        │
                        ▼
                      PAID → CLOSED
  ```

### 5.16 search_sync_logs — 同步队列(详见 §9)

### 5.17 audit_logs — 操作审计

---

## 6. bee_orm Rust 模型结构体

以下结构体对应核心表,采用 bee_orm `#[derive(Model)]` + `QuerySet`(feature `mysql`)约定。字段标注 `#[orm(...)]` 描述主键/长度/唯一/外键/时间;`serde` 用于 JSON 序列化(API 响应与索引文档共用)。字符串枚举直接以 `String` 承载,配合类型化常量(见 §10)校验,避免过度类型化。

通用标注说明(对齐 bottle_orm 风格):
- `#[orm(primary_key)]` — 主键,`i64` 映射 `BIGINT`
- `#[orm(size = N)]` — `VARCHAR(N)`
- `#[orm(unique)]` / `#[orm(index)]` — 唯一/普通索引
- `#[orm(foreign_key = "Table::id")]` — 外键
- `#[orm(create_time)]` / `#[orm(update_time)]` — 时间列
- `#[serde(skip_serializing_if = "Option::is_none")]` — 可空字段 JSON 省略
- 密文字段仅内部可见,对外 DTO 返回 `*_masked` / 解密后明文(见 §8)

```rust
use bee_orm::{Model, QuerySet};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// ============================================================
// 1. users
// ============================================================
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[orm(primary_key)]
    pub id: i64,
    #[orm(size = 64, unique)]
    pub username: String,
    // 敏感字段:密文,仅后端解密,不直接参与 API JSON 输出
    #[serde(skip_serializing)]
    pub phone_enc: Option<Vec<u8>>,        // VARBINARY
    #[serde(skip_serializing)]
    pub id_card_enc: Option<Vec<u8>>,      // VARBINARY
    pub phone_masked: Option<String>,
    #[serde(skip_serializing)]
    pub password_hash: String,
    #[orm(size = 128)]
    pub email: Option<String>,
    #[orm(size = 64)]
    pub nickname: Option<String>,
    #[orm(size = 512)]
    pub avatar_url: Option<String>,
    #[orm(size = 32)]
    pub role: String,                      // "USER" | "ADMIN" | "OPERATOR"
    #[orm(size = 32)]
    pub status: String,                    // "ACTIVE" | "DISABLED" | "FROZEN"
    pub last_login_at: Option<DateTime<Utc>>,
    #[orm(create_time)]
    pub created_at: DateTime<Utc>,
    #[orm(update_time)]
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl User {
    pub const ROLE_USER: &'static str = "USER";
    pub const ROLE_ADMIN: &'static str = "ADMIN";
    pub const ROLE_OPERATOR: &'static str = "OPERATOR";
    pub const STATUS_ACTIVE: &'static str = "ACTIVE";
    pub const STATUS_DISABLED: &'static str = "DISABLED";
    pub const STATUS_FROZEN: &'static str = "FROZEN";
}

// ============================================================
// 2. policy_holders
// ============================================================
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
pub struct PolicyHolder {
    #[orm(primary_key)]
    pub id: i64,
    #[orm(foreign_key = "User::id", index)]
    pub user_id: Option<i64>,              // 可空:为他人投保
    #[orm(size = 64)]
    pub name: String,
    #[serde(skip_serializing)]
    pub id_card_enc: Option<Vec<u8>>,
    #[orm(size = 32)]
    pub id_type: String,                   // "ID_CARD" | "PASSPORT" | "OTHER"
    #[orm(size = 16)]
    pub gender: Option<String>,            // "MALE" | "FEMALE" | "UNKNOWN"
    pub birthday: Option<chrono::NaiveDate>,
    #[serde(skip_serializing)]
    pub phone_enc: Option<Vec<u8>>,
    #[orm(size = 128)]
    pub email: Option<String>,
    #[orm(size = 255)]
    pub address: Option<String>,
    #[orm(size = 32)]
    pub relationship: Option<String>,      // "SELF"|"SPOUSE"|"CHILD"|"PARENT"|"OTHER"
    #[orm(size = 32)]
    pub status: String,
    #[orm(create_time)]
    pub created_at: DateTime<Utc>,
    #[orm(update_time)]
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// ============================================================
// 3. insurance_products
// ============================================================
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
pub struct InsuranceProduct {
    #[orm(primary_key)]
    pub id: i64,
    #[orm(size = 64, unique)]
    pub product_code: String,
    #[orm(size = 128)]
    pub name: String,
    #[orm(size = 255)]
    pub subtitle: Option<String>,
    pub description: Option<String>,       // TEXT
    #[orm(size = 32)]
    pub product_type: String,              // "LIFE"|"HEALTH"|"ACCIDENT"|"TRAVEL"|"PROPERTY"
    #[orm(size = 32)]
    pub sale_channel: String,              // "ONLINE"|"AGENT"|"BROKER"|"OFFLINE"
    #[orm(foreign_key = "User::id", index)]
    pub operator_user_id: Option<i64>,
    #[orm(size = 128)]
    pub insurer_name: Option<String>,
    #[orm(size = 8)]
    pub currency: String,
    pub min_amount: Option<rust_decimal::Decimal>,   // DECIMAL(14,2)
    pub max_amount: Option<rust_decimal::Decimal>,
    pub min_term_months: Option<i32>,
    pub max_term_months: Option<i32>,
    pub waiting_period_days: Option<i32>,
    pub is_featured: bool,
    #[orm(size = 512)]
    pub cover_image_url: Option<String>,
    #[orm(size = 32)]
    pub status: String,                    // "DRAFT"|"ON_SALE"|"OFF_SHELF"|"DISCONTINUED"
    pub search_enabled: bool,
    #[orm(create_time)]
    pub created_at: DateTime<Utc>,
    #[orm(update_time)]
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl InsuranceProduct {
    pub const STATUS_DRAFT: &'static str = "DRAFT";
    pub const STATUS_ON_SALE: &'static str = "ON_SALE";
    pub const STATUS_OFF_SHELF: &'static str = "OFF_SHELF";
    pub const STATUS_DISCONTINUED: &'static str = "DISCONTINUED";
}

// ============================================================
// 4. insurance_product_clauses
// ============================================================
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
pub struct InsuranceProductClause {
    #[orm(primary_key)]
    pub id: i64,
    #[orm(foreign_key = "InsuranceProduct::id", index)]
    pub product_id: i64,
    #[orm(size = 32)]
    pub clause_type: String,               // "MAIN"|"EXCLUSION"|"WAIVER"|"RIDER"|"OBLIGATION"
    #[orm(size = 255)]
    pub title: String,
    pub content: String,                   // LONGTEXT
    pub sort_order: i32,
    pub is_required: bool,
    #[orm(size = 32)]
    pub version: String,
    #[orm(size = 32)]
    pub status: String,                    // "ACTIVE"|"DEPRECATED"
    #[orm(create_time)]
    pub created_at: DateTime<Utc>,
    #[orm(update_time)]
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// ============================================================
// 5. insurance_product_categories
// ============================================================
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
pub struct InsuranceProductCategory {
    #[orm(primary_key)]
    pub id: i64,
    pub parent_id: Option<i64>,            // 自关联
    #[orm(size = 64)]
    pub name: String,
    #[orm(size = 64, unique)]
    pub slug: String,
    pub sort_order: i32,
    #[orm(size = 32)]
    pub status: String,                    // "ACTIVE"|"HIDDEN"
    #[orm(create_time)]
    pub created_at: DateTime<Utc>,
    #[orm(update_time)]
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// ============================================================
// 6. insurance_product_category_rel
// ============================================================
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
pub struct InsuranceProductCategoryRel {
    #[orm(primary_key)]
    pub id: i64,
    #[orm(foreign_key = "InsuranceProduct::id", index)]
    pub product_id: i64,
    #[orm(foreign_key = "InsuranceProductCategory::id", index)]
    pub category_id: i64,
    #[orm(create_time)]
    pub created_at: DateTime<Utc>,
}
```

```rust
// ============================================================
// 7. quotes
// ============================================================
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    #[orm(primary_key)]
    pub id: i64,
    #[orm(size = 32, unique)]
    pub quote_no: String,
    #[orm(foreign_key = "InsuranceProduct::id", index)]
    pub product_id: i64,
    #[orm(foreign_key = "User::id", index)]
    pub user_id: i64,
    #[orm(foreign_key = "PolicyHolder::id", index)]
    pub holder_id: Option<i64>,
    #[orm(size = 64)]
    pub holder_name: Option<String>,
    #[serde(skip_serializing)]
    pub holder_id_card_enc: Option<Vec<u8>>,
    pub insurance_amount: rust_decimal::Decimal,
    pub term_months: i32,
    pub premium: rust_decimal::Decimal,
    pub premium_detail: Option<serde_json::Value>,   // JSON
    pub effective_date: Option<chrono::NaiveDate>,
    pub expire_date: Option<chrono::NaiveDate>,
    pub health_declaration: Option<serde_json::Value>,
    pub risk_score: Option<i32>,
    #[orm(size = 32)]
    pub status: String,    // "PENDING"|"APPROVED"|"REJECTED"|"EXPIRED"|"CONVERTED"|"CANCELLED"
    pub expires_at: DateTime<Utc>,
    #[orm(create_time)]
    pub created_at: DateTime<Utc>,
    #[orm(update_time)]
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// ============================================================
// 8. quotes_beneficiaries
// ============================================================
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
pub struct QuoteBeneficiary {
    #[orm(primary_key)]
    pub id: i64,
    #[orm(foreign_key = "Quote::id", index)]
    pub quote_id: i64,
    #[orm(size = 64)]
    pub name: String,
    #[serde(skip_serializing)]
    pub id_card_enc: Option<Vec<u8>>,
    #[orm(size = 32)]
    pub relationship: Option<String>,
    #[orm(size = 16)]
    pub beneficiary_type: String,   // "LEGAL"|"NAMED"
    pub share_percent: Option<rust_decimal::Decimal>, // DECIMAL(5,2) 0-100
    pub sort_order: i32,
    #[orm(create_time)]
    pub created_at: DateTime<Utc>,
}

// ============================================================
// 9. orders
// ============================================================
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    #[orm(primary_key)]
    pub id: i64,
    #[orm(size = 32, unique)]
    pub order_no: String,
    #[orm(foreign_key = "Quote::id", index)]
    pub quote_id: i64,
    #[orm(foreign_key = "User::id", index)]
    pub user_id: i64,
    #[orm(foreign_key = "InsuranceProduct::id", index)]
    pub product_id: i64,
    #[orm(size = 128)]
    pub product_name: String,
    #[orm(size = 64)]
    pub holder_name: String,
    pub insurance_amount: rust_decimal::Decimal,
    pub term_months: i32,
    pub total_amount: rust_decimal::Decimal,
    pub discount_amount: rust_decimal::Decimal,
    pub payable_amount: rust_decimal::Decimal,
    #[orm(size = 8)]
    pub currency: String,
    #[orm(size = 32)]
    pub status: String,  // "CREATED"|"PAID"|"POLICY_ISSUED"|"COMPLETED"|"CANCELLED"|"EXPIRED"|"REFUNDING"|"REFUNDED"
    pub paid_at: Option<DateTime<Utc>>,
    pub policy_issued_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    #[orm(size = 255)]
    pub remark: Option<String>,
    #[orm(create_time)]
    pub created_at: DateTime<Utc>,
    #[orm(update_time)]
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Order {
    pub const STATUS_CREATED: &'static str = "CREATED";
    pub const STATUS_PAID: &'static str = "PAID";
    pub const STATUS_POLICY_ISSUED: &'static str = "POLICY_ISSUED";
    pub const STATUS_COMPLETED: &'static str = "COMPLETED";
    pub const STATUS_CANCELLED: &'static str = "CANCELLED";
    pub const STATUS_EXPIRED: &'static str = "EXPIRED";
    pub const STATUS_REFUNDING: &'static str = "REFUNDING";
    pub const STATUS_REFUNDED: &'static str = "REFUNDED";
}

// ============================================================
// 10. payments
// ============================================================
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    #[orm(primary_key)]
    pub id: i64,
    #[orm(size = 32, unique)]
    pub payment_no: String,
    #[orm(foreign_key = "Order::id", index)]
    pub order_id: i64,
    #[orm(foreign_key = "User::id", index)]
    pub user_id: i64,
    pub amount: rust_decimal::Decimal,
    #[orm(size = 8)]
    pub currency: String,
    #[orm(size = 32)]
    pub channel: String,   // "WECHAT"|"ALIPAY"|"UNIONPAY"|"BALANCE"|"MOCK"
    #[orm(size = 32)]
    pub provider: String,  // "MOCK" | "WECHAT"(PayProvider 实现名)
    #[orm(size = 128)]
    pub provider_tx_id: Option<String>,
    #[orm(size = 32)]
    pub status: String,    // "CREATED"|"PROCESSING"|"SUCCESS"|"FAILED"|"CANCELLED"|"REFUNDED"
    pub prepay_payload: Option<serde_json::Value>,
    pub callback_payload: Option<serde_json::Value>,
    pub paid_at: Option<DateTime<Utc>>,
    pub refunded_at: Option<DateTime<Utc>>,
    #[orm(create_time)]
    pub created_at: DateTime<Utc>,
    #[orm(update_time)]
    pub updated_at: DateTime<Utc>,
}

impl Payment {
    pub const STATUS_CREATED: &'static str = "CREATED";
    pub const STATUS_PROCESSING: &'static str = "PROCESSING";
    pub const STATUS_SUCCESS: &'static str = "SUCCESS";
    pub const STATUS_FAILED: &'static str = "FAILED";
    pub const STATUS_CANCELLED: &'static str = "CANCELLED";
    pub const STATUS_REFUNDED: &'static str = "REFUNDED";
    pub const CHANNEL_WECHAT: &'static str = "WECHAT";
    pub const CHANNEL_ALIPAY: &'static str = "ALIPAY";
    pub const CHANNEL_MOCK: &'static str = "MOCK";
}
```

```rust
// ============================================================
// 11. policies
// ============================================================
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    #[orm(primary_key)]
    pub id: i64,
    #[orm(size = 32, unique)]
    pub policy_no: String,
    #[orm(foreign_key = "Order::id", index)]
    pub order_id: i64,
    #[orm(foreign_key = "Quote::id", index)]
    pub quote_id: i64,
    #[orm(foreign_key = "User::id", index)]
    pub user_id: i64,
    #[orm(foreign_key = "PolicyHolder::id", index)]
    pub holder_id: Option<i64>,
    #[orm(foreign_key = "InsuranceProduct::id", index)]
    pub product_id: i64,
    #[orm(size = 128)]
    pub product_name: String,
    #[orm(size = 64)]
    pub holder_name: String,
    #[serde(skip_serializing)]
    pub holder_id_card_enc: Option<Vec<u8>>,
    pub insurance_amount: rust_decimal::Decimal,
    pub premium: rust_decimal::Decimal,
    pub term_months: i32,
    pub effective_date: chrono::NaiveDate,
    pub expire_date: chrono::NaiveDate,
    #[orm(size = 32)]
    pub status: String, // "PENDING_ISSUE"|"ACTIVE"|"EXPIRED"|"CANCELLED"|"SURRENDERED"|"LAPSED"
    #[orm(size = 16)]
    pub issue_type: String,  // "NEW"|"RENEW"
    pub is_renewable: bool,
    #[orm(size = 512)]
    pub pdf_path: Option<String>,
    pub premium_detail: Option<serde_json::Value>,
    pub issued_at: Option<DateTime<Utc>>,
    #[orm(create_time)]
    pub created_at: DateTime<Utc>,
    #[orm(update_time)]
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Policy {
    pub const STATUS_PENDING_ISSUE: &'static str = "PENDING_ISSUE";
    pub const STATUS_ACTIVE: &'static str = "ACTIVE";
    pub const STATUS_EXPIRED: &'static str = "EXPIRED";
    pub const STATUS_CANCELLED: &'static str = "CANCELLED";
    pub const STATUS_SURRENDERED: &'static str = "SURRENDERED";
    pub const STATUS_LAPSED: &'static str = "LAPSED";
}

// ============================================================
// 12. policy_beneficiaries
// ============================================================
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
pub struct PolicyBeneficiary {
    #[orm(primary_key)]
    pub id: i64,
    #[orm(foreign_key = "Policy::id", index)]
    pub policy_id: i64,
    #[orm(size = 64)]
    pub name: String,
    #[serde(skip_serializing)]
    pub id_card_enc: Option<Vec<u8>>,
    #[orm(size = 32)]
    pub relationship: Option<String>,
    #[orm(size = 16)]
    pub beneficiary_type: String,   // "LEGAL"|"NAMED"
    pub share_percent: Option<rust_decimal::Decimal>, // 0-100, NAMED 合计=100
    pub sort_order: i32,
    #[orm(create_time)]
    pub created_at: DateTime<Utc>,
    #[orm(update_time)]
    pub updated_at: DateTime<Utc>,
}

// ============================================================
// 13. contracts
// ============================================================
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    #[orm(primary_key)]
    pub id: i64,
    #[orm(size = 32, unique)]
    pub contract_no: String,
    #[orm(foreign_key = "Policy::id", unique)]
    pub policy_id: i64,
    #[orm(foreign_key = "Order::id", index)]
    pub order_id: i64,
    #[orm(size = 128)]
    pub title: String,
    #[orm(size = 32)]
    pub contract_type: String,   // "POLICY"|"ENDORSEMENT"|"RIDER"
    #[orm(size = 512)]
    pub pdf_path: Option<String>,
    #[orm(size = 128)]
    pub file_hash: Option<String>,       // SHA-256 防篡改
    #[orm(size = 128)]
    pub sign_flow_id: Option<String>,    // 电子签平台流程 ID
    #[orm(size = 32)]
    pub provider: String,                // "MOCK" | "ESIGN"(ElectronicSignature 实现名)
    #[orm(size = 32)]
    pub status: String,  // "DRAFT"|"PENDING_SIGN"|"SIGNING"|"COMPLETED"|"VOID"|"EXPIRED"|"REJECTED"
    pub signed_at: Option<DateTime<Utc>>,
    #[orm(create_time)]
    pub created_at: DateTime<Utc>,
    #[orm(update_time)]
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Contract {
    pub const STATUS_DRAFT: &'static str = "DRAFT";
    pub const STATUS_PENDING_SIGN: &'static str = "PENDING_SIGN";
    pub const STATUS_SIGNING: &'static str = "SIGNING";
    pub const STATUS_COMPLETED: &'static str = "COMPLETED";
    pub const STATUS_VOID: &'static str = "VOID";
    pub const STATUS_EXPIRED: &'static str = "EXPIRED";
    pub const STATUS_REJECTED: &'static str = "REJECTED";
}

// ============================================================
// 14. contract_signers
// ============================================================
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
pub struct ContractSigner {
    #[orm(primary_key)]
    pub id: i64,
    #[orm(foreign_key = "Contract::id", index)]
    pub contract_id: i64,
    #[orm(foreign_key = "User::id", index)]
    pub user_id: Option<i64>,
    #[orm(size = 64)]
    pub name: String,
    #[orm(size = 32)]
    pub signer_type: String,  // "APPLICANT"|"INSURED"|"BENEFICIARY"|"WITNESS"
    pub sign_order: i32,
    #[orm(size = 32)]
    pub status: String,  // "PENDING"|"SIGNING"|"SIGNED"|"COMPLETED"|"REJECTED"|"ABANDONED"
    #[orm(size = 512)]
    pub sign_url: Option<String>,
    #[orm(size = 128)]
    pub sign_token: Option<String>,
    pub signed_at: Option<DateTime<Utc>>,
    pub sign_detail: Option<serde_json::Value>,
    #[orm(create_time)]
    pub created_at: DateTime<Utc>,
    #[orm(update_time)]
    pub updated_at: DateTime<Utc>,
}

// ============================================================
// 15. claims
// ============================================================
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    #[orm(primary_key)]
    pub id: i64,
    #[orm(size = 32, unique)]
    pub claim_no: String,
    #[orm(foreign_key = "Policy::id", index)]
    pub policy_id: i64,
    #[orm(foreign_key = "Order::id", index)]
    pub order_id: i64,
    #[orm(foreign_key = "User::id", index)]
    pub user_id: i64,
    pub accident_date: Option<chrono::NaiveDate>,
    #[orm(size = 64)]
    pub accident_type: Option<String>,
    pub accident_desc: Option<String>,
    pub claim_amount: rust_decimal::Decimal,
    pub approved_amount: Option<rust_decimal::Decimal>,
    #[orm(size = 32)]
    pub status: String, // "SUBMITTED"|"UNDER_REVIEW"|"PENDING_INFO"|"REVIEWING"|"APPROVED"|"PAID"|"REJECTED"|"CLOSED"|"WITHDRAWN"
    pub reviewer_id: Option<i64>,
    #[orm(size = 255)]
    pub review_remark: Option<String>,
    #[orm(size = 64)]
    pub pay_ref: Option<String>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
    #[orm(create_time)]
    pub created_at: DateTime<Utc>,
    #[orm(update_time)]
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Claim {
    pub const STATUS_SUBMITTED: &'static str = "SUBMITTED";
    pub const STATUS_UNDER_REVIEW: &'static str = "UNDER_REVIEW";
    pub const STATUS_PENDING_INFO: &'static str = "PENDING_INFO";
    pub const STATUS_REVIEWING: &'static str = "REVIEWING";
    pub const STATUS_APPROVED: &'static str = "APPROVED";
    pub const STATUS_PAID: &'static str = "PAID";
    pub const STATUS_REJECTED: &'static str = "REJECTED";
    pub const STATUS_CLOSED: &'static str = "CLOSED";
    pub const STATUS_WITHDRAWN: &'static str = "WITHDRAWN";
}

// ============================================================
// 16. search_sync_logs
// ============================================================
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
pub struct SearchSyncLog {
    #[orm(primary_key)]
    pub id: i64,
    #[orm(size = 32)]
    pub entity_type: String,   // "PRODUCT"|"CLAUSE"|"POLICY"
    pub entity_id: i64,
    #[orm(size = 16)]
    pub op: String,            // "UPSERT"|"DELETE"
    #[orm(size = 16)]
    pub status: String,        // "PENDING"|"PROCESSING"|"SUCCESS"|"FAILED"|"RETRYING"|"DEAD"
    pub attempts: i32,
    pub max_attempts: i32,
    pub next_retry_at: Option<DateTime<Utc>>,
    #[orm(size = 512)]
    pub last_error: Option<String>,
    pub payload_json: Option<serde_json::Value>,
    #[orm(create_time)]
    pub created_at: DateTime<Utc>,
    #[orm(update_time)]
    pub updated_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
}

impl SearchSyncLog {
    pub const OP_UPSERT: &'static str = "UPSERT";
    pub const OP_DELETE: &'static str = "DELETE";
    pub const STATUS_PENDING: &'static str = "PENDING";
    pub const STATUS_PROCESSING: &'static str = "PROCESSING";
    pub const STATUS_SUCCESS: &'static str = "SUCCESS";
    pub const STATUS_FAILED: &'static str = "FAILED";
    pub const STATUS_RETRYING: &'static str = "RETRYING";
    pub const STATUS_DEAD: &'static str = "DEAD";
}

// ============================================================
// 17. audit_logs
// ============================================================
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    #[orm(primary_key)]
    pub id: i64,
    #[orm(foreign_key = "User::id", index)]
    pub user_id: Option<i64>,
    #[orm(size = 64)]
    pub action: String,        // "ORDER_PAY"|"POLICY_ISSUE"|"CONTRACT_SIGN"|...
    #[orm(size = 32)]
    pub entity_type: String,   // "ORDER"|"POLICY"|"CONTRACT"|"PAYMENT"|...
    pub entity_id: i64,
    pub before_json: Option<serde_json::Value>,
    pub after_json: Option<serde_json::Value>,
    #[orm(size = 64)]
    pub ip: Option<String>,
    #[orm(size = 255)]
    pub user_agent: Option<String>,
    #[orm(size = 64)]
    pub trace_id: Option<String>,
    #[orm(create_time)]
    pub created_at: DateTime<Utc>,
}
```

---

## 7. rust-scout Searchable 检索设计

### 7.1 哪些实体实现 `Searchable` trait

| 实体 | 是否可检索 | 说明 |
|------|-----------|------|
| `InsuranceProduct` | **是** | 产品检索首页/搜索主入口 |
| `InsuranceProductClause` | **是** | 条款检索,便于核保/客服快速定位 |
| `Policy` | **是** | 保单检索(内部/管理员,含被保人姓名、保单号) |
| `User` | 否 | 用户不开放全局搜索,仅内部精确查 |
| `Order/Payment/Quote/Contract/Claim` | 否 | 有结构化筛选即可,无需全文索引 |

`InsuranceProduct` 实现 `Searchable`,同步 OpenSearch;`InsuranceProductClause` 与 `Policy` 同构实现。三者均通过 `search_sync_logs` 异步同步。

### 7.2 Searchable trait 接口约定

```rust
pub trait Searchable {
    fn index_name(&self) -> &'static str;      // "insurance_products" / "clauses" / "policies"
    fn doc_id(&self) -> String;                // 索引文档 _id(取业务主键字符串)
    fn to_doc(&self) -> serde_json::Value;     // 序列化为待索引 JSON 文档
    fn op(&self) -> SearchOp;                  // Upsert / Delete(删除时仅 doc_id 有效)
}
```

### 7.3 三类索引文档 JSON 结构

**a) 产品索引 `insurance_products`(检索词:名称/副标题/描述/分类/类型)**

```json
{
  "id": 1001,
  "product_code": "PA-LIFE-001",
  "name": "安心定期寿险",
  "subtitle": "高杠杆家庭保障",
  "description": "覆盖身故/全残,保障家庭责任",
  "product_type": "LIFE",
  "sale_channel": "ONLINE",
  "insurer_name": "示例人寿",
  "currency": "CNY",
  "min_amount": 100000.00,
  "max_amount": 5000000.00,
  "min_term_months": 12,
  "max_term_months": 360,
  "waiting_period_days": 90,
  "category_slugs": ["life", "term"],
  "category_names": ["人寿保险", "定期寿险"],
  "is_featured": true,
  "status": "ON_SALE",
  "created_at": "2026-09-01T00:00:00Z"
}
```

**b) 条款索引 `clauses`(检索词:标题/正文)**

```json
{
  "id": 5001,
  "product_id": 1001,
  "product_name": "安心定期寿险",
  "clause_type": "EXCLUSION",
  "title": "责任免除条款",
  "content": "因下列情形导致被保险人身故的,保险人不承担给付保险金责任:...",
  "version": "v1.0",
  "status": "ACTIVE",
  "updated_at": "2026-09-01T00:00:00Z"
}
```

**c) 保单索引 `policies`(内部检索:保单号/被保人/产品;含敏感字段索引需脱敏)**

```json
{
  "id": 90001,
  "policy_no": "P2026090100001",
  "order_id": 80001,
  "user_id": 101,
  "product_id": 1001,
  "product_name": "安心定期寿险",
  "holder_name": "张三",              // 被保人姓名(非敏感,可索引)
  "holder_id_card_masked": "110***********1234",  // 身份证脱敏
  "insurance_amount": 1000000.00,
  "premium": 2200.00,
  "effective_date": "2026-09-02",
  "expire_date": "2036-09-02",
  "status": "ACTIVE",
  "created_at": "2026-09-01T00:00:00Z"
}
```

### 7.4 搜索路由映射

- `GET /api/v1/search?keyword=&type=product|clause|policy` 命中对应索引;
- `type` 省略时多索引联合查询并 RRF 融合排序;
- 索引文档的 `status` 字段用于过滤下架/失效内容,保证只检索在售/有效数据。

---

## 8. 敏感数据字段级加密

### 8.1 加密字段清单

| 字段 | 算法/建议 | 存储列 | 展示策略 |
|------|-----------|--------|----------|
| 身份证号 `id_card` | **AES-256-GCM**(带随机 IV + 认证标签),主密钥由 KMS/环境变量托管 | `VARBINARY(1024)` | 返回脱敏 `110***********1234` |
| 手机号 `phone` | **AES-256-GCM** | `VARBINARY(512)` | 返回脱敏 `138****1234` |
| 银行卡号 `bank_card`(预留) | **AES-256-GCM** | `VARBINARY(1024)` | 返回尾号 4 位 |
| 密码 | bcrypt/argon2(单向哈希,不可逆) | `VARCHAR(128)` | 永不返回 |

### 8.2 设计要点

1. **字段级加密**:仅对确属敏感的字段加密,不整库加密(保持可索引/可排序的常规字段正常)。
2. **AES-256-GCM**:提供认证加密,防篡改;随机 IV 每次不同,相同明文产生不同密文。
3. **密钥管理**:主密钥存 KMS 或安全环境变量;业务代码经 `CryptoService` 封装加解密,`decrypt` 仅内存使用,禁止打日志、禁止序列化进响应。
4. **密文列用 `VARBINARY`** 而非 `VARCHAR`:二进制安全、无字符集问题;长度按明文上限 + IV(12B)+ tag(16B) 预留。
5. **密文不入索引**:OpenSearch 文档仅放脱敏值(`holder_id_card_masked`),绝不放密文或明文。
6. **脱敏列**:`users.phone_masked` 冗余存储脱敏串,供列表与索引直读,避免每次解密。

### 8.3 加解密流程

```
写入: 明文 → CryptoService.encrypt(key, plain) → IV||CipherText||Tag → VARBINARY 落库
读取: VARBINARY → CryptoService.decrypt(key, blob) → 内存明文 → 仅本请求使用
脱敏: CryptoService.mask(plain) → 138****1234 → 响应 / 索引 / phone_masked
```

---

## 9. 数据一致性:search_sync_logs 最终一致同步

### 9.1 目标

业务写入只落 MySQL,通过异步任务把产品/条款/保单同步到 OpenSearch。`search_sync_logs` 作为**持久化同步队列**,保证 **DB→OpenSearch 最终一致**、可重试、可幂等重放。

### 9.2 写路径(业务流程内不阻塞)

1. 业务事务内写主表(如 `INSERT/UPDATE insurance_products`)。
2. 同一事务内插入一条 `search_sync_logs` 记录:
   - `entity_type='PRODUCT'`, `entity_id=1001`, `op='UPSERT'`, `status='PENDING'`, `attempts=0`, `max_attempts=5`, `next_retry_at=NOW()`。
   - 同时把待同步文档快照写入 `payload_json`(幂等重放的基础,见 9.4)。
   - 事务原子性保证:主表与同步记录**要么都提交、要么都回滚**,不会漏记。
3. 主事务提交后返回响应,同步在后台进行。

### 9.3 消费路径(后台同步 Worker)

```
轮询/定时: SELECT * FROM search_sync_logs
           WHERE status IN ('PENDING','FAILED','RETRYING')
             AND next_retry_at <= NOW()
           ORDER BY id LIMIT N  FOR UPDATE SKIP LOCKED;

处理: 标记 status='PROCESSING',attempts+=1
      按 entity_type 加载主表 → to_doc() → rust-scout/OpenSearch 写入(或删除)
      成功 → status='SUCCESS', processed_at=NOW()
      失败 → attempts < max_attempts ? status='FAILED'(next_retry_at=指数退避)
            : status='DEAD'(记录 last_error,人工告警)
```

- **`idx_synclog_status(status, next_retry_at)`** 支撑高效扫描待处理任务;
- 多实例部署时用 `FOR UPDATE SKIP LOCKED` 避免重复消费同一行。

### 9.4 幂等与重放保证

- **文档快照落库**:`payload_json` 存待写文档 JSON;重试时可直接用快照重建文档,即使主表已变化也不影响补偿(或选择重新 `to_doc()` 覆盖)。
- **`uk 业务唯一`**:OpenSearch 文档 `_id = entity_id` 字符串;`UPSERT` 幂等(同 _id 覆盖),`DELETE` 幂等(不存在也视为成功)。
- **失败重试**:指数退避(`next_retry_at = NOW() + 2^attempts` 秒),超 `max_attempts` 转 `DEAD`,由运维人工或补偿任务处理。

### 9.5 删除与一致性窗口

- 业务软删除(`deleted_at`)→ 同步 `op='DELETE'` 删除索引文档;若需彻底物理删除,先在索引 DELETE 成功后再物理删主表,保证索引不残留孤儿文档。
- 允许**短暂不一致窗口**(秒级):搜索可能滞后于 DB 写入,属最终一致可接受范围;对强实时性诉求,可对单条查询走 DB 兜底。

### 9.6 时序图

```
业务线程                    同步 Worker              OpenSearch
  │ 写主表 + INSERT sync_log(PENDING)
  │ (同事务,原子)
  │──commit──▶
  │                          │ 轮询到 PENDING 任务
  │                          │──PROCESSING→to_doc()──▶ 写入/删除
  │                          │◀──200 OK───────────────
  │                          │──UPDATE status=SUCCESS
  │◀──响应返回────────────
```

---

## 10. 附录:枚举常量汇总

| 实体 | 字段 | 枚举值 |
|------|------|--------|
| users | role | `USER` / `ADMIN` / `OPERATOR` |
| users | status | `ACTIVE` / `DISABLED` / `FROZEN` |
| policy_holders | id_type | `ID_CARD` / `PASSPORT` / `OTHER` |
| policy_holders | gender | `MALE` / `FEMALE` / `UNKNOWN` |
| policy_holders | relationship | `SELF` / `SPOUSE` / `CHILD` / `PARENT` / `OTHER` |
| insurance_products | product_type | `LIFE` / `HEALTH` / `ACCIDENT` / `TRAVEL` / `PROPERTY` |
| insurance_products | sale_channel | `ONLINE` / `AGENT` / `BROKER` / `OFFLINE` |
| insurance_products | status | `DRAFT` / `ON_SALE` / `OFF_SHELF` / `DISCONTINUED` |
| insurance_product_clauses | clause_type | `MAIN` / `EXCLUSION` / `WAIVER` / `RIDER` / `OBLIGATION` |
| insurance_product_clauses | status | `ACTIVE` / `DEPRECATED` |
| insurance_product_categories | status | `ACTIVE` / `HIDDEN` |
| quotes | status | `PENDING` / `APPROVED` / `REJECTED` / `EXPIRED` / `CONVERTED` / `CANCELLED` |
| quotes_beneficiaries | beneficiary_type | `LEGAL` / `NAMED` |
| orders | status | `CREATED` / `PAID` / `POLICY_ISSUED` / `COMPLETED` / `CANCELLED` / `EXPIRED` / `REFUNDING` / `REFUNDED` |
| payments | channel | `WECHAT` / `ALIPAY` / `UNIONPAY` / `BALANCE` / `MOCK` |
| payments | provider | `MOCK` / `WECHAT`(预留) |
| payments | status | `CREATED` / `PROCESSING` / `SUCCESS` / `FAILED` / `CANCELLED` / `REFUNDED` |
| policies | status | `PENDING_ISSUE` / `ACTIVE` / `EXPIRED` / `CANCELLED` / `SURRENDERED` / `LAPSED` |
| policies | issue_type | `NEW` / `RENEW` |
| policy_beneficiaries | beneficiary_type | `LEGAL` / `NAMED` |
| contracts | contract_type | `POLICY` / `ENDORSEMENT` / `RIDER` |
| contracts | provider | `MOCK` / `ESIGN`(预留 e签宝) |
| contracts | status | `DRAFT` / `PENDING_SIGN` / `SIGNING` / `COMPLETED` / `VOID` / `EXPIRED` / `REJECTED` |
| contract_signers | signer_type | `APPLICANT` / `INSURED` / `BENEFICIARY` / `WITNESS` |
| contract_signers | status | `PENDING` / `SIGNING` / `SIGNED` / `COMPLETED` / `REJECTED` / `ABANDONED` |
| claims | status | `SUBMITTED` / `UNDER_REVIEW` / `PENDING_INFO` / `REVIEWING` / `APPROVED` / `PAID` / `REJECTED` / `CLOSED` / `WITHDRAWN` |
| search_sync_logs | entity_type | `PRODUCT` / `CLAUSE` / `POLICY` |
| search_sync_logs | op | `UPSERT` / `DELETE` |
| search_sync_logs | status | `PENDING` / `PROCESSING` / `SUCCESS` / `FAILED` / `RETRYING` / `DEAD` |

---

## 结语

本文档提供可直接落地 `install.sql`(§4 完整 DDL)与 Rust `models/`(§6 结构体)的完整规划。后续实现时:
1. 金额统一 `rust_decimal::Decimal`,禁止 `f64`;
2. 状态流转收敛到各 Model 的 `status` 常量 + 校验函数;
3. 受益人占比应用层校验(NAMED 合计=100);
4. 搜索同步走 `search_sync_logs` 异步队列,不阻塞主事务;
5. 敏感字段一律 AES-256-GCM 密文 + 脱敏展示。
