# insurance-service

保险服务平台后端服务（Rust 版）。基于 `bee_rust`（bee 管线 + 路由 + ORM 骨架）、`axum`、`tokio`
构建，目标覆盖保险业务从「商品展示 → 报价 → 订单 → 支付 → 保单 → 电子签约 → 理赔」的完整闭环。

当前迭代处于 **阶段 0 → 1**：已落地**用户认证、保险商品、MySQL LIKE 搜索、报价、理赔（含审核）**业务能力，并打通
**报价 → 订单 → 支付 → 保单 → 签约**交易闭环 API（支付 / 电子签当前为 Mock 渠道），并已提供**运营后台商品建档与上下架**接口。

服务仅提供 JSON 接口；`/api/v1/*` 与其他路径的未匹配请求都返回统一 JSON 信封，不提供 HTML 页面。

| 项目 | 值 |
|------|-----|
| 语言 / 版本 | Rust 2024 edition（rust-version ≥ 1.87） |
| 许可证 | 仅供个人学习；商业使用需授权（详见「项目授权 / Project License」） |
| 版本 | 1.10.0 |
| HTTP 框架 | axum 0.8 + bee_rust（bee_router / bee_orm / 过滤器管线） |
| 存储 | MySQL 8.4（业务库，表主键由应用层 idgen_rs snowflake 生成）· 内存缓存（限流 / 会话） |

---

## 项目架构

![项目架构](docs/architecture.svg)

分层职责：

- **客户端**：Flutter App、微信小程序、HarmonyOS App、运营管理后台。
- **接入层**：`bee_rust + axum` 提供 `/healthz` 健康检查与 `/api/v1/*` 业务路由。
- **中间件过滤器链**：`SecurityFilter → Trace → JWT 认证（RBAC）→ Controller 分派`，
  任一层拒绝即返回 `401 / 403` 统一信封，不进入业务层。
- **Controller / Service**：auth / product / search / quote / order / payment / policy / contract / claim
  各业务 Controller 经 `bee_router` 分发；Service 为事务边界（`services/*_service.rs`）。
- **支付与签约**：当前为内联 Mock 实现（微信 / 支付宝 / 易签等外部渠道未接入）。
- **数据访问**：`Db`（mysql_async 连接池）——参数化查询防注入、写事务 `commit / rollback` 闭环、
  `bee_orm Model` 行 → 结构体映射。
- **基础设施**：MySQL 8.4（`install.sql` 建库）· 内存缓存与限流。

## 项目功能

![项目功能](docs/features.svg)

- **用户与认证**：注册（用户名唯一 + argon2 口令哈希）、登录（账号密码；微信 openid 绑定登录闭环——code2session 校验 + 绑定 + 按 openid 直登，凭据未配置时降级报错）、
  JWT 双令牌（access 短时效 + refresh 长时效，token_version 版本化——logout / 改密 / 换绑即吊销旧令牌）与 RBAC 角色（USER / AGENT / OPERATOR / ADMIN）。
- **保险商品**：列表分页 / 状态过滤 / 精选位、详情、关联条款阅读，运营后台建档与上架 / 下架 / 停售（OPERATOR / ADMIN）。
- **报价**：费率表驱动（`quote_rates` 按产品 / 保障期 / 保额区间定价，命中优先，未命中回退请求保费）。
- **全文搜索**：MySQL LIKE（带分页保护）。
- **交易闭环**：报价 → 订单 → 支付（预下单 / 支付 / 回调）API 已挂载；微信渠道当前为 Mock，真实渠道规划中。
- **保单与契约**：保单列表 / 详情、受益人批改（`POLICY_ENDORSE` 审计留痕）、电子合同内联 Mock 签署与签署回调 API 已挂载；外部电子签渠道未接入。
- **理赔**：报案（`CLM` 单号，校验保单归属）、我的理赔列表、审核（APPROVE / REJECT，需 OPERATOR / ADMIN）、资料上传 / 列表（`claim_documents` 元数据）已实现。
- **运营 / 审计**：运营后台 API、`audit_logs` 全量留痕 + 审计查询接口（`/api/v1/admin/audit-logs`，OPERATOR / ADMIN）。
- **横切能力**：入站安全扫描、参数化查询、AES-256-GCM 敏感字段加密、令牌精确过期（leeway=0）、
  snowflake 分布式主键（idgen_rs 无锁生成，worker 位长 6 + 序列位长 6，worker_id 经 `IDGEN_WORKER_ID` 配置）、
  全链路 `trace_id`、统一响应信封（`{code, message, data}`，业务错误 `40000`）。

## 请求生命周期

![请求生命周期](docs/lifecycle.svg)

一次 `/api/v1/*` 请求自接入路由起，依次穿过过滤器链、Controller / Service、数据访问与外部
Provider 适配层，最终以统一信封返回；全程透传 `trace_id`，审计点落库受理。

## 项目结构

```
insurance-service/
├── Cargo.toml                 # 工作区 / 依赖 / 发布配置
├── install.sql                # MySQL 建库脚本（全部业务表）
├── config/
│   ├── app.toml               # 应用配置模板（server / database / jwt / crypto / log）
│   └── bee.toml               # bee_rust 管线配置
├── docs/                      # 文档与架构图
│   ├── social-preview.png     # GitHub Social Preview（1280×630）
│   ├── architecture.svg       # 系统架构图
│   ├── features.svg           # 功能总览图
│   ├── lifecycle.svg          # 请求生命周期图
│   ├── backend-architecture.md
│   ├── db-schema.md
│   └── design.md / plan.md / tasks.md / flutter-app.md / miniprogram-harmony.md
├── src/
│   ├── main.rs                # 启动入口：init → 配置 → AppState → 路由 → serve
│   ├── lib.rs                 # 库入口（集成测试依赖）
│   ├── config.rs              # AppConfig：从环境变量 + config/app.toml 加载
│   ├── routes.rs              # build_bee_router 路由表（38 个业务端点）
│   ├── db.rs                  # mysql_async 连接池与查询 / 事务封装
│   ├── error.rs               # AppError（BadRequest / Unauthorized / NotFound / Business…）
│   ├── response.rs            # 统一响应信封 ResponseEnvelope（{code, message, data}）
│   ├── controllers/           # 各业务 Controller（经 AppState 注入）
│   │   ├── mod.rs             #   AppState · bee 管线 run() · 统一信封
│   │   ├── auth.rs            #   认证 / 用户中心
│   │   ├── product.rs         #   商品 / 条款 / 精选
│   │   ├── search.rs          #   全文搜索
│   │   ├── quote.rs           #   报价
│   │   ├── order.rs           #   订单
│   │   ├── payment.rs         #   支付（预下单 / 回调）
│   │   ├── policy.rs          #   保单 / 受益人批改
│   │   ├── contract.rs        #   电子合同
│   │   ├── claim.rs           #   理赔（报案 / 审核 / 资料）
│   │   ├── admin.rs           #   运营后台（商品建档 / 上下架 / 审计查询）
│   ├── middleware/
│   │   ├── mod.rs             # Filter trait · RequestCtx
│   │   ├── auth.rs            # JwtService · AuthFilter · RequireRoleFilter
│   │   ├── security.rs        # SecurityFilter（入站安全扫描）
│   │   ├── rate_limit.rs      # 固定窗口限流器
│   │   └── trace.rs           # trace_id 生成 / 透传 + 请求日志
│   ├── services/
│   │   ├── mod.rs
│   │   ├── auth_service.rs    # 注册 / 登录 / 微信绑定 / 令牌吊销
│   │   ├── product_service.rs # 商品列表 / 详情 / 精选
│   │   ├── search_service.rs  # 搜索（MySQL LIKE）
│   │   ├── quote_service.rs   # 报价（费率表驱动）
│   │   ├── order_service.rs   # 订单
│   │   ├── payment_service.rs # 支付（预下单 / 回调）
│   │   ├── policy_service.rs  # 保单（续保 / 退保 / 批改）
│   │   ├── contract_service.rs # 电子合同（Mock 签署）
│   │   ├── claim_service.rs   # 理赔（报案 / 审核 / 资料）
│   ├── models/                # bee_orm Model（user / insurance_product / order / payment / policy / contract / claim…）
│   ├── crypto/                # AES-256-GCM · Masker · argon2
│   ├── providers/             # wechat（code2session）
│   ├── search/                # searchable_impl（搜索序列化）
│   └── utils/                 # validator · id_generator · idgen（snowflake）
└── tests/
    ├── common/mod.rs                # 测试共享设施（测试库连接 / JWT 配置 / 唯一值 / 清理）
    ├── auth_service_test.rs         # 注册 / 登录 / 微信 stub（6 项）
    ├── product_service_test.rs      # 商品增删改查 / 过滤 / 软删（5 项）
    ├── search_service_test.rs       # 搜索命中 / 无果 / 分页（4 项）
    ├── quote_service_test.rs        # 报价试算 / 详情 / 鉴权（3 项）
    ├── security_test.rs             # JWT 校验 / 过期 / 角色 RBAC（18 项）
    ├── api_auth_test.rs             # API 层鉴权 E2E（8 项）
    ├── claim_service_test.rs        # 理赔报案 / 归属校验 / 分页（5 项）
    ├── claim_review_test.rs         # 理赔审核 APPROVE / REJECT（5 项）
    ├── admin_product_test.rs        # 商品上架 / 下架管理端（5 项）
    ├── auth_fix_test.rs             # 认证修复回归：#10 修复项（8 项）
    ├── product_fix_test.rs          # 产品模块修复回归：HTTP 层（4 项）
    ├── contract_fix_test.rs         # 签约修复回归：sign-url Mock 真实化（4 项）
    ├── trade_flow_test.rs           # 交易闭环：报价 → 下单 → 支付回调 → 保单签发（4 项）
    ├── policy_lifecycle_test.rs     # 保单生命周期：续保 renew / 退保 lapse（9 项）
    ├── user_center_test.rs          # 用户中心：改密 / 换绑手机（3 项）
    ├── wechat_channel_test.rs       # 微信 code2session 渠道（2 项）
    ├── c1_auth_extend_test.rs       # C1 认证扩展（6 项）
    ├── c2_policy_endorse_test.rs    # C2 受益人批改（5 项）
    ├── c3_claim_docs_test.rs        # C3 理赔资料上传 / 列表（5 项）
    ├── c4_quote_rates_test.rs       # C4 费率表化报价（4 项）
    ├── c5_audit_query_test.rs       # C5 运营端审计日志查询（1 项）
```

## 使用说明

### 环境要求

- Rust toolchain ≥ 1.87（`cargo --version` 检查）
- MySQL 8.4（可选，集成测试与正式运行需要）

### 配置

配置通过环境变量注入，模板见 `.env.example`，默认值见 `config/app.toml`：

| 变量 | 说明 | 缺省 |
|------|------|------|
| `SERVER_HOST` / `SERVER_PORT` | 监听地址 | `0.0.0.0` / `8080` |
| `DATABASE_URL` | MySQL 连接串（**必填**，如 `mysql://user:pass@127.0.0.1:3306/insurance_service`） | — |
| `JWT_SECRET` / `JWT_ISSUER` | JWT 签名密钥 / 签发者 | — |
| `JWT_ACCESS_EXPIRY` / `JWT_REFRESH_EXPIRY` | access / refresh 有效秒数 | `7200` / `604800` |
| `CRYPTO_MASTER_KEY` | AES-256-GCM 主密钥（口令 / 敏感字段加密） | — |
| `IDGEN_WORKER_ID` | snowflake 主键 worker id（多实例各分配唯一 0-63） | `0` |
| `RUST_LOG` | 日志级别 | `info` |

### 初始化数据库

```bash
mysql -h127.0.0.1 -uroot -p < install.sql     # 建库建表（含权限脚本示例）
```

### 构建与运行

```bash
cargo build          # 编译
cargo run            # 启动，监听 SERVER_HOST:SERVER_PORT（默认 0.0.0.0:8080）
curl http://127.0.0.1:8080/healthz   # 健康检查
```

### 测试

```bash
cargo test           # 全部测试（单元 + 集成）
```

- 集成测试需要可用的 MySQL（`DATABASE_URL` 且已执行 `install.sql`）；未配置时**不会自动跳过，而是失败**。
  测试规模以 `cargo test` 输出为准。集成测试覆盖：
  认证（微信绑定/未配置降级/令牌吊销）、API 鉴权 E2E、交易闭环（报价→订单→支付回调→保单签发）、
  保单生命周期（续保/退保/受益人批改）、限流、用户中心（改密/换绑）、费率报价、
  理赔（资料上传）、审计查询、商品、修复回归等；单元测试覆盖限流窗口、加解密与脱敏、
  身份证 / 手机号校验、snowflake 唯一性、微信响应解析、健康检查与统一 404 信封（均不依赖 DB）。
- 按项目约定（CLAUDE.md）：代码变更后**先跑测试、再提交**。

### 已实现 API 概览（阶段 0 → 1）

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/healthz` | 健康检查（JSON 信封） |
| — | 未匹配路径 | 统一返回 JSON 信封 `40400` |
| POST | `/api/v1/auth/register` | 注册（返回双令牌） |
| POST | `/api/v1/auth/login` | 账号密码登录 |
| POST | `/api/v1/auth/wechat/login` | 微信登录（code2session 校验，按 openid 直登；未绑定提示；未配置凭据降级报错） |
| POST | `/api/v1/auth/wechat/bind` | 微信绑定（登录态 + code → 写入 openid，冲突 409） |
| POST | `/api/v1/auth/refresh` · `/api/v1/auth/logout` | 令牌刷新 / 注销 |
| GET | `/api/v1/user/me` | 当前用户资料（Bearer Token） |
| GET | `/api/v1/products` · `/api/v1/products/{id}` | 商品列表 / 详情 |
| GET | `/api/v1/products/{id}/clauses` | 商品条款 |
| GET | `/api/v1/products/featured` | 精选商品 |
| GET | `/api/v1/search?q=&index=&page=&size=` | 全文搜索 |
| POST | `/api/v1/quotes` | 报价试算 |
| GET | `/api/v1/quotes/{id}` | 报价详情 |
| POST | `/api/v1/orders` · GET `/api/v1/orders` | 下单 / 我的订单 |
| GET | `/api/v1/orders/{id}` | 订单详情 |
| POST | `/api/v1/payments/{order_id}/prepay` · `/pay` | 预支付 / 支付（Mock） |
| POST | `/api/v1/payments/wechat/prepay` | 微信预支付（Mock 渠道） |
| POST | `/api/v1/payments/callback/{provider}` | 支付回调 |
| GET | `/api/v1/policies` · `/api/v1/policies/{id}` | 我的保单 / 保单详情 |
| POST | `/api/v1/policies/{id}/beneficiaries` | 保单批改：受益人整单替换（POLICY_ENDORSE 审计） |
| GET | `/api/v1/contracts/{id}` | 电子合同详情 |
| POST | `/api/v1/contracts/{id}/sign` · GET `/sign-url` | 合同签署 / 签署 URL |
| POST | `/api/v1/contracts/callback/{provider}` | 签署回调 |
| POST | `/api/v1/claims` | 理赔报案（校验保单归属） |
| GET | `/api/v1/claims?user_id=&page=&size=` | 我的理赔（分页） |
| POST | `/api/v1/claims/{id}/review` | 理赔审核 APPROVE / REJECT（OPERATOR / ADMIN） |
| POST · GET | `/api/v1/claims/{id}/documents` | 理赔资料上传 / 列表（归属校验） |
| POST | `/api/v1/admin/products` · `/api/v1/admin/products/{id}/status` | 商品建档 / 上下架（OPERATOR / ADMIN） |
| GET | `/api/v1/admin/audit-logs` | 审计日志查询（OPERATOR / ADMIN，多条件过滤 + 分页） |

完整路由表（共 38 个业务端点，含 `/user/me`、`/user/password`、`/user/phone`、运营后台 `/admin/*`）
见 `src/routes.rs` `build_bee_router()`；库表设计见 `docs/db-schema.md` 与 `install.sql`。

---

## 支持与打赏

如果本项目对您有帮助，欢迎打赏支持（一分也是爱 💖）。

### 微信支付 / 支付宝

| 微信支付 | 支付宝 |
|----------|--------|
| ![微信支付收款码](docs/weixinpay.png) | ![支付宝收款码](docs/alipay.png) |

打开对应 App 扫码即可；转账时欢迎备注昵称，感谢支持！

## 视觉资产授权 / Visual Assets License

本项目的图片、图标（包括 `docs/` 下的 SVG / PNG）**不是开源资产**，版权归作者所有。
未经书面许可不得复制、修改、再分发或用于商业用途；商业使用需取得商业授权，联系
**erik@erik.xyz**。详见 [ASSETS-LICENSE.md](ASSETS-LICENSE.md)。

> 代码部分仍遵循仓库原有 `LICENSE`；第三方素材（如明确标注的 Go Gopher 等）仍遵循其原许可证。


## 开源版 vs 商业版

> 商业版由作者单独提供，与开源版同源；商业版是当前开源版的严格超集（无开源版独有的文件、表或路由）。

| 维度 | 开源版 | 商业版 |
|------|--------|--------|
| 授权 | 仅个人学习 / 研究，禁止商业使用 | 需商业授权，联系 erik@erik.xyz |
| 核心交易闭环 | 认证 / RBAC / 商品 / 报价 / 订单 / 支付（Mock）/ 保单 / 合同（Mock）/ 理赔 / 审计 | 同左，全部保留 |
| 支付与电子签 | 内联 Mock | 保留 Provider 抽象与渠道适配骨架（微信支付 / 易签 + Mock；真实渠道需配置） |
| 全文搜索 | MySQL LIKE | OpenSearch + rust_scout 同步 Worker（未就绪自动降级 LIKE） |
| 运营统计 | 无 | `POST /api/v1/admin/stats` |
| 浏览器展示面 | 纯 JSON，无 HTML | `GET /` 落地页、`/favicon.svg`、`/pet/{slug}`、HTML 错误页、吉祥物素材 |
| 扩展数据模型 | 14 张表 | 19 张表：+ 产品分类 / 分类关系 / 被保人 / 合同签署方 / 搜索同步日志 |
| 代码规模 | 56 个 Rust 源文件 / 7,059 行 | 74 个 / 8,616 行 |
| 路由 | 40 条（38 业务） | 44 条（39 业务 + 浏览器面） |
| 控制器 / 模型 / 服务 | 11 / 13 / 10 | 12 / 18 / 11 |
| 测试 | 以仓库 HEAD 为准 | 商业版 README 标注 26 单元 + 115 集成（以交付版本为准） |

### 商业版独有能力

- **浏览器访问面**：落地页、吉祥物/宠物表情、favicon、HTML 错误页。
- **支付/电子签 Provider 层**：`providers/payment`（微信 / Mock）与 `providers/esign`（易签 / Mock）；开源版已剥离为内联 Mock。
- **OpenSearch 搜索同步**：`search/sync_worker.rs`、`search_sync_logs` 表、`rust_scout` 依赖、Redis/OpenSearch 配置。
- **运营统计**：`/api/v1/admin/stats` 及对应 service。
- **扩展业务模型**：产品分类、分类关系、被保人、合同签署方。

### 开源版保留

- 用户认证 / JWT / RBAC、保险商品、报价、订单、支付回调（Mock）、保单、电子合同（Mock）、理赔、审计、MySQL LIKE 搜索、AES-256-GCM 加密、限流与安全扫描。
- 适合个人学习、研究、二次开发原型；生产或商业部署需取得商业授权。

**商业版授权与交付条款请联系：erik@erik.xyz**

**English summary:** The open edition is for personal study and research and retains the core insurance transaction loop (auth/RBAC, products, quotes, orders, mock payment, policies, mock e-sign, claims, audit, MySQL LIKE search). The commercial edition is a strict superset: browser landing/mascot pages, external payment/e-sign provider layer, OpenSearch sync worker, admin ops stats, and product-category/policy-holder/contract-signer schema (19 tables vs 14). Commercial licensing and delivery: erik@erik.xyz.


## 项目授权 / Project License

Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

- 本项目仅供个人学习、研究使用。
- 未经作者书面授权，禁止任何形式的商业使用（包括但不限于商业部署、付费服务、二次销售、SaaS 运营及以本项目为基础提供商业服务）。
- 商业授权请联系 **erik@erik.xyz**。
- 未经授权的商业使用，作者将在全球范围内依法追究法律责任，并保留终身追责的权利。
- 第三方组件仍遵循其各自原有的许可证；视觉资产适用仓库内的 ASSETS-LICENSE.md（如适用）。

**English:** This project is provided for personal study and research only. Commercial use of any kind requires prior written authorization from the author (erik@erik.xyz). Unauthorized commercial use will be pursued for legal liability worldwide, and the author reserves the right to pursue such liability for life. Third-party components remain under their original licenses; visual assets are governed by ASSETS-LICENSE.md where applicable.
