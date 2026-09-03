-- ============================================================
-- 保险服务平台 MySQL 建库脚本（install.sql）
-- 来源: docs/db-schema.md §4 完整 DDL（19 张表,主键均为应用层 snowflake 生成）
-- 约定: utf8mb4 / utf8mb4_unicode_ci / InnoDB / DATETIME(3) 毫秒级
-- ============================================================
CREATE DATABASE IF NOT EXISTS insurance_service
  DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
USE insurance_service;

-- ============================================================
-- 1. users 用户账户
-- ============================================================
CREATE TABLE users (
  id            BIGINT UNSIGNED NOT NULL PRIMARY KEY, -- snowflake 主键,应用层 idgen_rs 生成
  username      VARCHAR(64)  NOT NULL,
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
  openid        VARCHAR(64)   NULL,          -- 微信 openid（v1.6.0 绑定闭环）
  unionid       VARCHAR(64)   NULL,          -- 微信 unionid
  token_version INT           NOT NULL DEFAULT 0,  -- 令牌版本：logout/改密/换绑 +1 吊销旧 refresh
  UNIQUE KEY uk_username (username),
  UNIQUE KEY uk_openid (openid),
  KEY idx_phone_masked (phone_masked)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================================
-- 2. policy_holders 被保人档案
-- ============================================================
CREATE TABLE policy_holders (
  id            BIGINT UNSIGNED NOT NULL PRIMARY KEY, -- snowflake 主键,应用层 idgen_rs 生成
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
  id               BIGINT UNSIGNED NOT NULL PRIMARY KEY, -- snowflake 主键,应用层 idgen_rs 生成
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
  id          BIGINT UNSIGNED NOT NULL PRIMARY KEY, -- snowflake 主键,应用层 idgen_rs 生成
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
  id          BIGINT UNSIGNED NOT NULL PRIMARY KEY, -- snowflake 主键,应用层 idgen_rs 生成
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
  id          BIGINT UNSIGNED NOT NULL PRIMARY KEY, -- snowflake 主键,应用层 idgen_rs 生成
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

-- ============================================================
-- 7. quotes 报价 / 投保方案
-- ============================================================
CREATE TABLE quotes (
  id                 BIGINT UNSIGNED NOT NULL PRIMARY KEY, -- snowflake 主键,应用层 idgen_rs 生成
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
  id           BIGINT UNSIGNED NOT NULL PRIMARY KEY, -- snowflake 主键,应用层 idgen_rs 生成
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
  id            BIGINT UNSIGNED NOT NULL PRIMARY KEY, -- snowflake 主键,应用层 idgen_rs 生成
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
  id              BIGINT UNSIGNED NOT NULL PRIMARY KEY, -- snowflake 主键,应用层 idgen_rs 生成
  payment_no      VARCHAR(32)   NOT NULL,            -- 支付流水号
  order_id        BIGINT UNSIGNED NOT NULL,
  user_id         BIGINT UNSIGNED NOT NULL,
  amount          DECIMAL(14,2) NOT NULL,
  currency        VARCHAR(8)    NOT NULL DEFAULT 'CNY',
  channel         VARCHAR(32)   NOT NULL,            -- WECHAT/ALIPAY/UNIONPAY/BALANCE/MOCK
  provider        VARCHAR(32)   NOT NULL DEFAULT 'MOCK', -- PayProvider 实现名,预留 WECHAT
  provider_tx_id  VARCHAR(128)  NULL,                -- 支付渠道交易号
  status          VARCHAR(32)   NOT NULL DEFAULT 'CREATED',
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

-- ============================================================
-- 11. policies 保单
-- ============================================================
CREATE TABLE policies (
  id                  BIGINT UNSIGNED NOT NULL PRIMARY KEY, -- snowflake 主键,应用层 idgen_rs 生成
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
  id           BIGINT UNSIGNED NOT NULL PRIMARY KEY, -- snowflake 主键,应用层 idgen_rs 生成
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
  id            BIGINT UNSIGNED NOT NULL PRIMARY KEY, -- snowflake 主键,应用层 idgen_rs 生成
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
  id            BIGINT UNSIGNED NOT NULL PRIMARY KEY, -- snowflake 主键,应用层 idgen_rs 生成
  contract_id   BIGINT UNSIGNED NOT NULL,
  user_id       BIGINT UNSIGNED NULL,               -- 登录用户签署
  name          VARCHAR(64)   NOT NULL,             -- 签署人姓名
  signer_type   VARCHAR(32)   NOT NULL DEFAULT 'APPLICANT', -- APPLICANT/INSURED/BENEFICIARY/WITNESS
  sign_order    INT           NOT NULL DEFAULT 0,   -- 签署顺序
  status        VARCHAR(32)   NOT NULL DEFAULT 'PENDING',
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
  id              BIGINT UNSIGNED NOT NULL PRIMARY KEY, -- snowflake 主键,应用层 idgen_rs 生成
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

-- ============================================================
-- 16. search_sync_logs DB→OpenSearch 同步队列
-- ============================================================
CREATE TABLE search_sync_logs (
  id            BIGINT UNSIGNED NOT NULL PRIMARY KEY, -- snowflake 主键,应用层 idgen_rs 生成
  entity_type   VARCHAR(32)   NOT NULL,   -- PRODUCT / CLAUSE / POLICY
  entity_id     BIGINT UNSIGNED NOT NULL, -- 业务实体主键
  op            VARCHAR(16)   NOT NULL,   -- UPSERT / DELETE
  status        VARCHAR(16)   NOT NULL DEFAULT 'PENDING',
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
  id           BIGINT UNSIGNED NOT NULL PRIMARY KEY, -- snowflake 主键,应用层 idgen_rs 生成
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

-- ============================================================
-- 15. claim_documents 理赔资料（v1.6.0 C3）
-- ============================================================
CREATE TABLE claim_documents (
  id         BIGINT UNSIGNED NOT NULL PRIMARY KEY, -- snowflake 主键,应用层 idgen_rs 生成
  claim_id   BIGINT UNSIGNED NOT NULL,
  doc_type   VARCHAR(32)  NOT NULL,
  file_name  VARCHAR(255) NOT NULL,
  file_key   VARCHAR(255) NOT NULL,
  created_at DATETIME(3)  NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
  KEY idx_claim_doc_claim (claim_id),
  CONSTRAINT fk_claim_doc_claim FOREIGN KEY (claim_id) REFERENCES claims (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ============================================================
-- 16. quote_rates 报价费率表（v1.6.0 C4，整期保费系数 premium=保额×rate）
-- ============================================================
CREATE TABLE quote_rates (
  id          BIGINT UNSIGNED NOT NULL PRIMARY KEY, -- snowflake 主键,应用层 idgen_rs 生成
  product_id  BIGINT UNSIGNED NOT NULL,
  term_months INT           NOT NULL,              -- 保障期（月）匹配维度
  amount_min  DECIMAL(14,2) NOT NULL DEFAULT 0,    -- 保额下限（含）
  amount_max  DECIMAL(14,2) NULL,                  -- 保额上限（含），NULL=不限
  rate        DECIMAL(10,6) NOT NULL,              -- 整期保费系数
  created_at  DATETIME(3)   NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
  KEY idx_qrate_product (product_id, term_months),
  CONSTRAINT fk_qrate_product FOREIGN KEY (product_id)
    REFERENCES insurance_products (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
