# insurance-service

保险服务平台后端服务（Rust 版）。基于 `bee_rust`（bee 管线 + 路由 + ORM 骨架）、`axum`、`tokio`
构建，目标覆盖保险业务从「商品展示 → 报价 → 订单 → 支付 → 保单 → 电子签约 → 理赔」的完整闭环。

当前版本 **v1.11.0**：已落地**用户认证、保险商品、MySQL LIKE 搜索、报价、理赔（含审核）**业务能力，并打通
**报价 → 订单 → 支付 → 保单签发 → 签约**交易闭环 API（支付 / 电子签当前为 Mock 渠道），
并提供**保单签发 / 退保**与**运营后台商品建档、上下架与按状态查询**接口。

服务仅提供 JSON 接口；`/api/v1/*` 与其他路径的未匹配请求都返回统一 JSON 信封，不提供 HTML 页面。

| 项目 | 值 |
|------|-----|
| 语言 / 版本 | Rust 2024 edition（rust-version ≥ 1.87） |
| 许可证 | 仅供个人学习；商业使用需授权（详见「项目授权 / Project License」） |
| 版本 | 1.11.0 |
| HTTP 框架 | axum 0.8 + bee_rust（bee_router / bee_orm，vendored 1.0.6） |
| 存储 | MySQL 8.4（业务库，表主键由应用层 idgen_rs snowflake 生成）· 内存缓存（限流 / 会话） |

---

## 项目架构

![项目架构](docs/architecture.svg)

分层职责：

- **客户端**：Flutter App、微信小程序、HarmonyOS App、运营管理后台。
- **接入层**：`bee_rust + axum` 提供 `/healthz` 健康检查与 `/api/v1/*` 业务路由。
- **中间件链（axum middleware）**：`Trace → 限流（进程内固定窗口，按对端 socket IP）→ JWT 认证（RBAC）→ Controller 分派`，
  拒绝即返回 `401 / 403 / 429` 统一信封，不进入业务层。受保护路由需
  `Authorization: Bearer <access token>`；`Trace` 位于最外层，404 / 401 / 429 响应同样带 `X-Trace-Id`。
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
- **保险商品**：公开列表（仅返回在售，分页）/ 精选位 / 详情 / 关联条款阅读；运营后台按状态查询、建档与上架 / 下架 / 停售（OPERATOR / ADMIN）。
- **报价**：费率表驱动（`quote_rates` 按产品 / 保障期 / 保额区间定价）；未配置费率行则拒绝报价，保费一律由服务端按 保额 × 费率 计算，不接受客户端保费。
- **全文搜索**：MySQL LIKE（带分页保护）。
- **交易闭环**：报价 → 订单 → 支付（预下单 / 支付 / 回调）API 已挂载；微信渠道当前为 Mock，真实渠道规划中。
- **保单与契约**：保单签发（订单须已支付，幂等）/ 列表 / 详情 / 退保（`POLICY_LAPSE` 审计留痕）、
  受益人批改（`POLICY_ENDORSE` 审计留痕）、电子合同内联 Mock 签署与签署回调 API 已挂载；
  续保暂未开放（需重新走报价 → 订单 → 支付流程）；外部电子签渠道未接入。
- **理赔**：报案（`CLM` 单号，校验保单归属）、我的理赔列表、审核（APPROVE / REJECT，需 OPERATOR / ADMIN）、资料上传 / 列表（`claim_documents` 元数据）已实现。
- **运营 / 审计**：运营后台 API、`audit_logs` 全量留痕 + 审计查询接口（`/api/v1/admin/audit-logs`，OPERATOR / ADMIN）。
- **横切能力**：JWT 认证（受保护路由需 `Authorization: Bearer`）/ RBAC、参数化查询、
  `users.phone_enc` 字段级 AES-256-GCM 加密与脱敏、令牌精确过期（leeway=0）、
  snowflake 分布式主键（idgen_rs 无锁生成，worker 位长 6 + 序列位长 6，
  worker_id 经 `IDGEN_WORKER_ID` 配置）、全链路 `trace_id`、统一响应信封（`{code, message, data}`，
  业务错误 `40000`；数据库 / 内部错误只记日志，响应统一 `50000` 通用文案）。

## 请求生命周期

![请求生命周期](docs/lifecycle.svg)

一次 `/api/v1/*` 请求自接入路由起，依次穿过 axum 中间件链、Controller / Service、数据访问与外部
Provider 适配层，最终以统一信封返回；全程透传 `trace_id`，审计点落库受理。

## 项目结构

```
insurance-service/
├── Cargo.toml                 # 工作区 / 依赖 / 发布配置
├── install.sql                # MySQL 建库脚本（全部业务表）
├── config/
│   ├── app.toml               # 应用配置模板（server / database / jwt / crypto / payment / log）
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
│   ├── config.rs              # AppConfig：从环境变量加载（字段与 config/app.toml 对齐）
│   ├── routes.rs              # build_bee_router 路由表（41 个业务端点）
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
│   │   ├── mod.rs             # 模块声明与请求链说明
│   │   ├── auth.rs            # JwtService · Claims · Role · AuthUser
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
│   │   ├── policy_service.rs  # 保单（签发 / 退保 / 受益人批改）
│   │   ├── contract_service.rs # 电子合同（Mock 签署）
│   │   ├── claim_service/     # 理赔（报案 / 审核 / 资料；mod.rs + tests.rs）
│   ├── models/                # bee_orm Model（user / insurance_product / order / payment / policy / contract / claim…）
│   ├── crypto/                # AES-256-GCM · Masker · argon2
│   ├── providers/             # wechat（code2session）
│   ├── search/                # searchable_impl（搜索序列化）
│   └── utils/                 # validator · idgen（snowflake）
└── tests/
    ├── common/mod.rs          # 集成测试夹具：配置 / 用户 / 商品 / 费率 / 请求辅助
    ├── trade_flow.rs          # 交易闭环、归属隔离、回调令牌（DB 门控）
    ├── public_surface.rs      # 公开目录 / 搜索 / 报价费率 / trace（部分 DB 门控）
    └── auth_tokens.rs         # 令牌生命周期：access / refresh 类型隔离与版本（1 项）
```

## 使用说明

### 环境要求

- Rust toolchain ≥ 1.87（`cargo --version` 检查）
- MySQL 8.4（正式运行需要；单元测试不连库，DB 集成用例仅在设置 `TEST_DATABASE_URL` 时执行）

### 配置

配置通过环境变量注入，模板见 `.env.example`；`config/app.toml` 为字段对齐的示例（当前实现不解析 TOML），缺省值见代码：

| 变量 | 说明 | 缺省 |
|------|------|------|
| `SERVER_HOST` / `SERVER_PORT` | 监听地址 | `0.0.0.0` / `8080` |
| `DATABASE_URL` | MySQL 连接串（**必填**，如 `mysql://user:pass@127.0.0.1:3306/insurance_service`） | — |
| `JWT_SECRET` / `JWT_ISSUER` | JWT 签名密钥（**必填，至少 32 字节随机值**，如 `openssl rand -base64 48`）/ 签发者 | — |
| `JWT_ACCESS_EXPIRY` / `JWT_REFRESH_EXPIRY` | access / refresh 有效秒数 | `7200` / `604800` |
| `CRYPTO_MASTER_KEY` | AES-256-GCM 主密钥（口令 / 敏感字段加密） | — |
| `PAYMENT_CALLBACK_TOKEN` | 支付回调校验令牌（请求头 `X-Callback-Token`）；留空表示禁用回调 | 空（禁用） |
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

- `bee-rust` 1.0.6 已 vendored 至 `vendor/bee-rust`（源码快照 `93dd5c57`），构建不依赖 GitHub git；
  其余 registry 依赖在冷缓存下仍从 crates.io 下载。

### 测试

```bash
cargo test           # 单元 + 集成；DB 用例未设置 TEST_DATABASE_URL 时自动跳过（不计失败）
# 带数据库运行（需先执行 install.sql 建表；测试使用独立雪花 ID 行，可并行）：
TEST_DATABASE_URL='mysql://user:pass@127.0.0.1:3306/insurance_service' cargo test
```

- 当前测试规模：50 个测试 = 44 个单元测试（lib）+ 6 个集成测试（`tests/auth_tokens.rs` 1 项、
  `tests/trade_flow.rs` + `tests/public_surface.rs` 共 5 项，公共夹具在 `tests/common/mod.rs`）。
  其中 4 个 DB 用例由 `TEST_DATABASE_URL` 门控（未设置时直接返回）；覆盖交易闭环（报价 → 订单 →
  双支付回调 → 保单幂等 → 签约 → 理赔 → 退保）、跨用户归属隔离、公开目录与搜索可见性、
  回调令牌鉴权、404/429 的 trace_id 一致性。
  单元测试覆盖限流窗口、加解密与脱敏、身份证 / 手机号校验、snowflake 唯一性、搜索转义、
  报价试算、支付回调令牌比较、JWT 类型隔离、健康检查与统一 404 信封等，均不依赖 MySQL。
- 按项目约定（CLAUDE.md）：代码变更后**先跑测试、再提交**。

### 已实现 API 概览（v1.11.0）

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
| GET | `/api/v1/products` · `/api/v1/products/{id}` | 商品列表（公开，仅返回在售）/ 详情 |
| GET | `/api/v1/products/{id}/clauses` | 商品条款 |
| GET | `/api/v1/products/featured` | 精选商品 |
| GET | `/api/v1/search?keyword=&type=&page=&size=` | 全文搜索（`keyword` 必填） |
| POST | `/api/v1/quotes` | 报价试算（服务端按 `quote_rates` 计费；未配置费率则拒绝） |
| GET | `/api/v1/quotes/{id}` | 报价详情 |
| POST | `/api/v1/orders` · GET `/api/v1/orders` | 下单 / 我的订单 |
| GET | `/api/v1/orders/{id}` | 订单详情 |
| POST | `/api/v1/payments/{order_id}/prepay` · `/pay` | 预支付 / 支付（Mock） |
| POST | `/api/v1/payments/wechat/prepay` | 微信预支付（Mock 渠道） |
| POST | `/api/v1/payments/callback/{provider}` | 支付回调（需 `X-Callback-Token`；Mock 阶段 provider 须为 `mock`） |
| POST | `/api/v1/policies` | 保单签发（订单须已支付，幂等） |
| GET | `/api/v1/policies` · `/api/v1/policies/{id}` | 我的保单（token 身份）/ 保单详情 |
| POST | `/api/v1/policies/{id}/lapse` | 退保（ACTIVE → SURRENDERED，审计留痕） |
| POST | `/api/v1/policies/{id}/beneficiaries` | 保单批改：受益人整单替换（POLICY_ENDORSE 审计） |
| GET | `/api/v1/contracts/{id}` | 电子合同详情 |
| POST | `/api/v1/contracts/{id}/sign` · GET `/sign-url` | 合同签署 / 签署 URL |
| POST | `/api/v1/contracts/callback/{provider}` | 签署回调 |
| POST | `/api/v1/claims` | 理赔报案（校验保单归属） |
| GET | `/api/v1/claims?page=&size=` | 我的理赔（分页，身份取 token） |
| POST | `/api/v1/claims/{id}/review` | 理赔审核 APPROVE / REJECT（OPERATOR / ADMIN） |
| POST · GET | `/api/v1/claims/{id}/documents` | 理赔资料上传 / 列表（归属校验） |
| GET | `/api/v1/admin/products` | 运营商品列表（可按状态过滤，OPERATOR / ADMIN） |
| POST | `/api/v1/admin/products` · `/api/v1/admin/products/{id}/status` | 商品建档 / 上下架（OPERATOR / ADMIN） |
| GET | `/api/v1/admin/audit-logs` | 审计日志查询（OPERATOR / ADMIN，多条件过滤 + 分页） |

完整路由表（共 41 个业务端点，含 `/user/me`、`/user/password`、`/user/phone`、运营后台 `/admin/*`）
见 `src/routes.rs` `build_bee_router()`；库表设计见 `docs/db-schema.md` 与 `install.sql`。
列表 / 详情类接口的用户身份一律取自已认证 token，不接受 `user_id` / `operator_user_id` 身份参数；
唯一例外是运营审计查询 `GET /api/v1/admin/audit-logs` 的 `user_id`（审计筛选条件，不是请求者身份）。

### 已知限制与部署注意

- **MySQL 8.4 假设**：搜索分页使用窗口函数 `COUNT(*) OVER()`（MySQL 8.0+）；LIKE 的
  `ESCAPE '\\'` 依赖 `sql_mode` 未启用 `NO_BACKSLASH_ESCAPES`（MySQL 默认关闭）。
- **支付回调**：Mock 阶段预支付统一记录 `provider=MOCK`，所有渠道回调都必须走
  `/payments/callback/mock` 并携带 `X-Callback-Token`；`PAYMENT_CALLBACK_TOKEN` 为空时回调全部拒绝。
  重复预支付产生多笔支付单时，竞争回调会整笔回滚、支付单留在 `CREATED`，当前没有自动退款或对账路径。
- **限流**：进程内固定窗口、按对端 socket IP（不读取可伪造的 `X-Forwarded-For`）；部署在反向代理后
  时所有请求会共享代理 IP，需在代理层另做按客户端限流。
- **安全边界**：服务不含 `SecurityFilter` 或入站 WAF / 内容扫描；安全依赖 JWT + RBAC、参数化查询、
  请求体大小限制（2 MiB）与上述限流，公网部署前请按需补充网关层防护。
- **敏感字段**：`users.phone_enc` 由服务端 AES-256-GCM 加密；报价 / 受益人等 `*_enc` 列直接保存
  上游传入的字节，服务端不解密、不重新加密。

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
| 代码规模 | 55 个 Rust 源文件 / 7,747 行 | 74 个 / 8,616 行 |
| 路由 | 41 条业务 + `/healthz` = 42 | 42 条业务（含 `/admin/stats`）+ `/healthz` + 3 条浏览器路由 |
| 控制器 / 模型 / 服务 | 11 / 13 / 10 | 12 / 18 / 11 |
| 测试 | 50 个测试（44 单元 + 6 集成；4 个 DB 用例由 `TEST_DATABASE_URL` 门控） | 商业版当前快照未包含测试（以交付版本为准）；无 CI 配置 |

### 多维度对比 / Multi-dimensional Comparison

| 维度 | 开源版 | 商业版 |
|------|--------|--------|
| 功能覆盖 | 41 条业务路由 + `/healthz`；11 控制器 / 13 模型 / 10 服务 / 14 表；核心投保交易闭环 | 42 条业务路由（含 `/admin/stats`）+ 浏览器路由/静态站点；12 / 18 / 11 / 19；多 5 张扩展表 |
| 安全防护 | JWT access/refresh + RBAC（Bearer 鉴权）；固定窗口限流 300 次/60s/IP（进程内、按对端 IP）；trace_id；AES-256-GCM + argon2；validator；审计日志；内部错误不外泄 | 安全中间件、限流、加密、校验源码与开源版一致；新增公开 HTML/宠物/favicon 面与运营统计接口（OPERATOR/ADMIN），暴露面略大 |
| 性能与规模 | axum + tokio；mysql_async 连接池；进程内 MemoryCache；进程内固定窗口限流；无后台任务 | 运行时基本相同；配置新增 Redis/OpenSearch，但 AppState 仍用 MemoryCache，SyncWorker 仅定义未启动；多实例共享缓存/限流未落地 |
| 数据与检索 | 14 表；搜索为 MySQL LIKE；无全文索引/同步 | 19 表（+产品分类/关系、被保人、合同签署方、搜索同步日志）；OpenSearch 同步 Worker 与 rust_scout 依赖为预留，SearchService 仍走 MySQL LIKE |
| 测试与质量 | 50 个测试（44 单元 + 6 集成；4 个 DB 用例由 `TEST_DATABASE_URL` 门控）；无 CI 配置 | 商业版当前快照未包含测试（以交付版本为准）；无 CI 配置 |
| 部署与运维 | 无 Dockerfile/compose/CI；`config/app.toml` + `bee.toml`、`install.sql`、`/healthz`、`scripts/release.sh`；单进程 | 同左；新增 Redis/OpenSearch 可选环境变量；仍无 compose、迁移 runner、CI |
| 多端与国际化 | 纯 JSON API；无 HTML 面、无客户端源码、无 i18n | 增加 HTML 落地页、宠物表情、favicon、HTML 错误页与 `site/` 静态站；无 Flutter/小程序源码；无 i18n |
| 扩展性与二次开发 | 仅微信 code2session provider；支付/电子签为内联 Mock；路由/控制器结构简单 | `PayProvider` / `ElectronicSignature` trait + 微信/易签/Mock 适配器；统计 service；搜索同步 Worker 预留；路由/控制器结构同源；无 OpenAPI 文件 |
| 授权与商业支持 | 仅个人学习/研究，禁止商业使用；无商业支持 | 商业授权；交付与支持条款联系 **erik@erik.xyz** |
| 适用场景 | 学习、研究、原型、二次开发；核心交易闭环足够 | 商业部署/交付；需要落地页、运营统计、支付/电子签扩展点；真实支付/电子签与 OpenSearch 仍需配置，部分模块为脚手架 |

**English**

| Dimension | Open edition | Commercial edition |
|---|---|---|
| Functional coverage | 41 business routes + `/healthz`; 11 controllers / 13 models / 10 services / 14 tables; core insurance transaction loop | 42 business routes (including `/admin/stats`) + browser routes/static site; 12 / 18 / 11 / 19; 5 extra tables |
| Security | JWT access/refresh + RBAC (Bearer auth); fixed-window 300 req/60s/IP (in-process, peer IP); trace ID; AES-256-GCM + argon2; validator; audit logs; internal errors not exposed | Same middleware, rate-limit, crypto and validation source; larger public HTML/pet/favicon surface plus stats endpoint (OPERATOR/ADMIN) |
| Performance / scale | axum + tokio; mysql_async pool; in-process MemoryCache and rate limiter; no background jobs | Effectively the same runtime; Redis/OpenSearch config exists, but AppState still uses MemoryCache and SyncWorker is not spawned; no shared multi-instance cache/limit |
| Data / search | 14 tables; MySQL LIKE search | 19 tables (+ product categories/relation, policy holder, contract signer, search sync log); OpenSearch sync Worker and rust_scout are scaffolding; SearchService still uses MySQL LIKE |
| Tests / quality | 50 tests (44 unit + 6 integration; 4 DB cases gated by `TEST_DATABASE_URL`); no CI | Commercial snapshot ships without tests (delivery-dependent); no CI |
| Deployment / ops | No Dockerfile/compose/CI; app.toml + bee.toml, install.sql, `/healthz`, release.sh; single process | Same; adds optional Redis/OpenSearch env vars; still no compose, migration runner or CI |
| Clients / i18n | JSON API only; no HTML surface, client source or i18n | Adds HTML landing/pet/favicon/error pages and `site/` static site; no Flutter/mini-program source; no i18n |
| Extensibility | WeChat code2session provider only; inline mock payment/e-sign | `PayProvider` / `ElectronicSignature` traits + WeChat/Escqian/Mock adapters; stats service; search sync Worker scaffolding; same route/controller structure; no OpenAPI file |
| License / support | Personal study/research only; commercial use prohibited | Commercial license; delivery/support terms via **erik@erik.xyz** |
| Use cases | Learning, research, prototypes, secondary development; core transaction loop is sufficient | Commercial deployment/delivery needing landing page, ops stats and payment/e-sign extension points; real payment/e-sign and OpenSearch still require configuration, some modules are scaffolding |

**商业版授权联系：erik@erik.xyz** / **Commercial licensing: erik@erik.xyz**

### 商业版独有能力

- **浏览器访问面**：落地页、吉祥物/宠物表情、favicon、HTML 错误页。
- **支付/电子签 Provider 层**：`providers/payment`（微信 / Mock）与 `providers/esign`（易签 / Mock）；开源版已剥离为内联 Mock。
- **OpenSearch 搜索同步**：`search/sync_worker.rs`、`search_sync_logs` 表、`rust_scout` 依赖、Redis/OpenSearch 配置。
- **运营统计**：`/api/v1/admin/stats` 及对应 service。
- **扩展业务模型**：产品分类、分类关系、被保人、合同签署方。

### 开源版保留

- 用户认证 / JWT / RBAC、保险商品、报价、订单、支付回调（Mock）、保单、电子合同（Mock）、理赔、审计、MySQL LIKE 搜索、AES-256-GCM 加密与限流。
- 适合个人学习、研究、二次开发原型；生产或商业部署需取得商业授权。

**商业版授权与交付条款请联系：erik@erik.xyz**

**English summary:** The open edition is for personal study and research and retains the core insurance transaction loop (auth/RBAC, products, quotes, orders, mock payment, policies, mock e-sign, claims, audit, MySQL LIKE search). The commercial edition is a strict superset: browser landing/mascot pages, external payment/e-sign provider layer, OpenSearch sync worker, admin ops stats, and product-category/policy-holder/contract-signer schema (19 tables vs 14). Commercial licensing and delivery: erik@erik.xyz.


## 项目授权 / Project License

Copyright (c) 2026 erik erik@erik.xyz — https://erik.xyz

- 本项目仅供个人学习、研究使用。
- 未经作者书面授权，禁止任何形式的商业使用（包括但不限于商业部署、付费服务、二次销售、SaaS 运营及以本项目为基础提供商业服务）。
- 商业授权请联系 **erik@erik.xyz**。
- 未经授权的商业使用，作者将在全球范围内依法追究法律责任，并保留终身追责的权利。
- 第三方组件仍遵循其各自原有的许可证；视觉资产适用仓库内的 ASSETS-LICENSE.md（如适用）。

**English:** This project is provided for personal study and research only. Commercial use of any kind requires prior written authorization from the author (erik@erik.xyz). Unauthorized commercial use will be pursued for legal liability worldwide, and the author reserves the right to pursue such liability for life. Third-party components remain under their original licenses; visual assets are governed by ASSETS-LICENSE.md where applicable.
