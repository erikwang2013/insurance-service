# 保险服务平台 — 后端 bee-rust 架构规划

> 版本: v1.0 → v1.7 | 日期: 2026-09-01（v1.6/v1.7 实现事实已同步标注）| 状态: 规划蓝图（对照实现核对版）
> 框架: `bee-rust`（Beerust，Rust Web 框架，对标 Go Beego）— git 依赖 `features=["full"]`
> 搜索: OpenSearch（`bee_search`）+ rust-scout（业务层门面）
> 数据库: MySQL 8（`bee_orm` feature `mysql`）；缓存/会话: Redis（`bee_kv`/`bee_session`）
> 安全: security-rust（27 检测器，`bee_rust` security feature 内置 `SecurityFilter`）
> 多端: Flutter / 原生微信小程序 / 鸿蒙 ArkTS 共用同一 REST API
>
> 关联文档: [db-schema.md](./db-schema.md)（19 表 Schema 与 Rust models）、[flutter-app.md](./flutter-app.md)、[miniprogram-harmony.md](./miniprogram-harmony.md)

---

## 目录

1. [总体架构](#1-总体架构)
2. [Workspace 目录结构](#2-workspace-目录结构)
3. [Cargo.toml 依赖配置](#3-cargotoml-依赖配置)
4. [路由注册方案](#4-路由注册方案)
5. [中间件 / 过滤器链](#5-中间件--过滤器链)
6. [统一响应与错误处理](#6-统一响应与错误处理)
7. [认证与鉴权（RBAC）](#7-认证与鉴权rbac)
8. [支付抽象 PayProvider](#8-支付抽象-payprovider)
9. [电子签抽象 ElectronicSignature](#9-电子签抽象-electronic-signature)
10. [搜索服务层与同步](#10-搜索服务层与同步)
11. [关键控制器签名](#11-关键控制器签名)
12. [配置与部署](#12-配置与部署)
13. [分阶段实现 Roadmap](#13-分阶段实现-roadmap)

---

## 1. 总体架构

采用 **MVC 分层**，严格遵循 bee-rust（Beego 哲学）的 Controller / Service / Model 结构，并以过滤器链承载横切关注点。

```
请求 ──► SecurityFilter ──► Session恢复 ──► JWT认证 ──► 参数校验
          (security-rust)                          │
                                                   ▼
                                        Controller(Context)
                                                   │
                                                   ▼
                                        Service(业务规则)
                                                   │
                                    ┌──────────────┼──────────────┐
                                    ▼              ▼              ▼
                                bee_orm        bee_kv        bee_search
                                (MySQL)       (Redis)      (OpenSearch)
```

### 分层职责

| 层 | 职责 | 说明 |
|----|------|------|
| **Filter Chain** | 横切关注点（安全/会话/认证/审计） | bee_router 过滤器链，对标 Beego Filter |
| **Controller** | 参数提取、调用 Service、组装响应 | `Controller` trait + `Context` |
| **Service** | 业务规则、事务、状态机流转 | 领域逻辑唯一入口 |
| **Model** | 数据访问 | `bee_orm #[derive(Model)]` + QuerySet |
| **SearchService** | 全文检索门面 | rust-scout Engine 封装 |

### 关键设计决策

1. **Service 承载事务**：Controller 不做业务，只做编排；事务边界在 Service。
2. **金额一律 `rust_decimal::Decimal`**：禁止浮点（对齐 db-schema.md）。
3. **状态机收敛到 Model 常量 + 校验函数**：订单/保单/合同/支付各状态流转禁止跳变。
4. **写库只落 MySQL，搜索走异步同步**：通过 `search_sync_logs` 最终一致。
5. **敏感字段 AES-256-GCM 密文 + 脱敏**：密文不入 API 响应、不入索引。
6. **三端共用统一 `ResponseEnvelope`**：`{ code, message, data, trace_id }`。

---

## 2. Workspace 目录结构

采用 Cargo workspace，单主 crate（`src/`），业务按 controller / service / model / middleware / search 分层。目录结构对齐 bee-rust 规范（Bee CLI 风格）。

```
insurance-service/
├── Cargo.toml                    # workspace 根
├── .gitignore
├── .env.example                  # 环境变量模板（不含密钥）
├── config/
│   ├── app.toml                  # bee_config 配置（MySQL/Redis/OpenSearch/JWT）
│   └── bee.toml                  # bee-rust 运行配置
├── install.sql                   # 建库脚本（取自 db-schema.md §4）
├── src/
│   ├── main.rs                   # 入口：加载配置、装配过滤器链、注册路由、启动
│   ├── lib.rs
│   ├── config.rs                 # 配置结构（bee_config #[derive(Config)]）
│   ├── routes.rs                 # 路由注册总表
│   ├── error.rs                  # 统一错误枚举 + thiserror
│   ├── response.rs               # ResponseEnvelope + 成功/失败构造
│   ├── middleware/
│   │   ├── mod.rs
│   │   ├── security.rs           # SecurityFilter 装配（security-rust 27 检测器）
│   │   ├── auth.rs               # JWT 认证 + RBAC 角色守卫
│   │   ├── trace.rs              # trace_id 生成/透传 + 请求日志
│   │   ├── audit.rs              # audit_logs 操作审计
│   │   └── rate_limit.rs         # Redis 限流（IP/用户维度）
│   ├── controllers/
│   │   ├── mod.rs
│   │   ├── auth_controller.rs
│   │   ├── product_controller.rs
│   │   ├── quote_controller.rs
│   │   ├── order_controller.rs
│   │   ├── payment_controller.rs
│   │   ├── policy_controller.rs
│   │   ├── contract_controller.rs
│   │   ├── claim_controller.rs
│   │   ├── user_controller.rs
│   │   └── search_controller.rs
│   ├── services/
│   │   ├── mod.rs
│   │   ├── auth_service.rs
│   │   ├── product_service.rs
│   │   ├── quote_service.rs      # 报价 + 保费计算
│   │   ├── pricing_service.rs    # 费率引擎（可扩展）
│   │   ├── order_service.rs
│   │   ├── payment_service.rs    # 依赖 PayProvider
│   │   ├── policy_service.rs     # 保单生成
│   │   ├── contract_service.rs   # 依赖 ElectronicSignature
│   │   ├── claim_service.rs
│   │   ├── user_service.rs
│   │   └── search_service.rs     # rust-scout 封装
│   ├── models/                   # bee_orm #[derive(Model)]（对齐 db-schema.md §6）
│   │   ├── mod.rs
│   │   ├── user.rs
│   │   ├── policy_holder.rs
│   │   ├── insurance_product.rs
│   │   ├── insurance_product_clause.rs
│   │   ├── insurance_product_category.rs
│   │   ├── insurance_product_category_rel.rs
│   │   ├── quote.rs
│   │   ├── quote_beneficiary.rs
│   │   ├── order.rs
│   │   ├── payment.rs
│   │   ├── policy.rs
│   │   ├── policy_beneficiary.rs
│   │   ├── contract.rs
│   │   ├── contract_signer.rs
│   │   ├── claim.rs
│   │   ├── search_sync_log.rs
│   │   └── audit_log.rs
│   ├── search/                   # rust-scout 集成
│   │   ├── mod.rs
│   │   ├── searchable_impl.rs    # Product/Clause/Policy 实现 Searchable
│   │   └── sync_worker.rs        # DB→OpenSearch 后台同步 Worker
│   ├── providers/                # 可插拔第三方适配器
│   │   ├── mod.rs
│   │   ├── payment/
│   │   │   ├── mod.rs
│   │   │   ├── pay_provider.rs   # PayProvider trait
│   │   │   ├── mock.rs           # MockPayProvider
│   │   │   └── wechat.rs         # WechatPayProvider（预留 stub）
│   │   └── esign/
│   │       ├── mod.rs
│   │       ├── esign_provider.rs # ElectronicSignature trait
│   │       ├── mock.rs           # MockEsignProvider
│   │       └── escqian.rs        # ESignQianProvider（预留 stub）
│   ├── crypto/                   # AES-256-GCM 加解密 + 脱敏
│   │   ├── mod.rs
│   │   └── crypto_service.rs
│   └── utils/
│       ├── id_generator.rs       # 订单号/保单号/合同号生成
│       └── validator.rs          # 参数校验（身份证/手机号/金额）
├── migrations/                   # bee_orm Migration（可选，或纯 install.sql）
├── tests/                        # 集成测试
│   ├── api_auth_test.rs
│   ├── api_product_test.rs
│   └── api_policy_flow_test.rs
├── app/                          # Flutter 客户端（见 flutter-app.md）
├── miniprogram/                  # 原生微信小程序（见 miniprogram-harmony.md）
└── harmony/                      # 鸿蒙 ArkTS（见 miniprogram-harmony.md）
```

---

## 3. Cargo.toml 依赖配置

workspace 根 `Cargo.toml`：

```toml
[workspace]
resolver = "2"
members = ["src"]            # 单 crate；后续可按需拆分 crates/

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "Apache-2.0"

[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tracing = "0.1"
tracing-subscriber = "0.3"
async-trait = "0.1"
chrono = { version = "0.4", features = ["serde"] }
rust_decimal = { version = "1", features = ["serde"] }
jsonwebtoken = "9"
regex = "1"
base64 = "0.22"
aes-gcm = "0.10"
argon2 = "0.5"               # 密码哈希
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }

# bee-rust（git 依赖，锁定 rev 保证可复现）
bee_rust = { git = "https://github.com/erikwang2013/bee-rust", features = ["full"] }

# rust-scout 全文搜索门面（OpenSearch 走 elasticsearch feature）
rust_scout = { version = "0.3", features = ["elasticsearch"] }

# security-rust 已被 bee_rust 间接引用，此处显式声明便于直接使用 Scanner
security_rust = "1"

[package]
name = "insurance-service"
version = "0.1.0"
edition = "2024"

[dependencies]
bee_rust = { workspace = true }
rust_scout = { workspace = true }
security_rust = { workspace = true }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
async-trait = { workspace = true }
chrono = { workspace = true }
rust_decimal = { workspace = true }
jsonwebtoken = { workspace = true }
regex = { workspace = true }
base64 = { workspace = true }
aes-gcm = { workspace = true }
argon2 = { workspace = true }
reqwest = { workspace = true }
```

> **可复现性建议**：bee-rust 尚未发布 crates.io，git 依赖应锁定 `rev = "<commit>"`（如 `bee_rust = { git = "...", rev = "abc123", features = ["full"] }`），避免上游变更破坏构建。

---

## 4. 路由注册方案

RESTful 命名空间 `/api/v1`。三端（Flutter/小程序/鸿蒙）共用同一路由，通过 `X-Client-Platform` 头区分（auth 登录、payment 支付两处按平台分流）。

| 命名空间 | 方法 | 路径 | 控制器动作 | 鉴权 |
|---------|------|------|-----------|------|
| auth | POST | `/api/v1/auth/register` | 注册 | 公开 |
| auth | POST | `/api/v1/auth/login` | 登录（密码/验证码） | 公开 |
| auth | POST | `/api/v1/auth/wechat/login` | 微信登录：code2session → openid 直登 / 未绑定提示 / 未配置降级（v1.6.0 已实现） | 公开 |
| auth | POST | `/api/v1/auth/wechat/bind` | 微信绑定：code2session 校验 + 写 users.openid，openid 冲突返回 40900（v1.6.0） | 需认证 |
| auth | POST | `/api/v1/auth/refresh` | 刷新令牌 | 公开（refresh token） |
| auth | POST | `/api/v1/auth/logout` | 登出 | 需认证 |
| products | GET | `/api/v1/products` | 产品列表（分页/筛选） | 公开 |
| products | GET | `/api/v1/products/{id}` | 产品详情+条款 | 公开 |
| products | GET | `/api/v1/products/{id}/clauses` | 产品条款 | 公开 |
| products | GET | `/api/v1/products/featured` | 首页推荐 | 公开 |
| quotes | POST | `/api/v1/quotes` | 创建报价/试算 | 需认证 |
| quotes | GET | `/api/v1/quotes/{id}` | 报价详情 | 需认证（属主） |
| orders | POST | `/api/v1/orders` | 由报价创建订单 | 需认证 |
| orders | GET | `/api/v1/orders` | 我的订单 | 需认证 |
| orders | GET | `/api/v1/orders/{id}` | 订单详情 | 需认证（属主） |
| payments | POST | `/api/v1/payments/{orderId}/prepay` | 创建支付 | 需认证 |
| payments | POST | `/api/v1/payments/{orderId}/pay` | 发起支付（Mock 直付） | 需认证 |
| payments | POST | `/api/v1/payments/wechat/prepay` | 微信统一下单 | 需认证 |
| payments | POST | `/api/v1/payments/callback/{provider}` | 支付渠道回调 | 公开（验签） |
| policies | GET | `/api/v1/policies` | 我的保单 | 需认证 |
| policies | GET | `/api/v1/policies/{id}` | 保单详情 | 需认证（属主） |
| policies | POST | `/api/v1/policies/{id}/beneficiaries` | 受益人批改（audit_logs action=POLICY_ENDORSE 快照，v1.6.0） | 需认证（属主） |
| contracts | GET | `/api/v1/contracts/{id}` | 合同详情 | 需认证（属主） |
| contracts | POST | `/api/v1/contracts/{id}/sign` | 发起签署 | 需认证 |
| contracts | GET | `/api/v1/contracts/{id}/sign-url` | 获取签署链接 | 需认证 |
| contracts | POST | `/api/v1/contracts/callback/{provider}` | 电子签回调 | 公开（验签） |
| search | GET | `/api/v1/search` | 全文搜索 | 公开 |
| claims | POST | `/api/v1/claims` | 报案 | 需认证 |
| claims | GET | `/api/v1/claims` | 我的理赔 | 需认证 |
| claims | POST/GET | `/api/v1/claims/{id}/documents` | 理赔材料上传/查询（claim_documents 元数据，v1.6.0） | 需认证（属主） |
| user | GET | `/api/v1/user/me` | 当前用户 | 需认证 |
| admin | * | `/api/v1/admin/**` | 管理端（产品上架/审核） | 需 ADMIN/OPERATOR |
| admin | GET | `/api/v1/admin/audit-logs` | 审计日志（OPERATOR/ADMIN 过滤分页，v1.6.0） | 需 ADMIN/OPERATOR |

> **实现核对**：本表为规划蓝图；现网实现以 `src/routes.rs` 的 `route_table()` 为准（共 39 个业务端点）。v1.6.0 新增 `auth/wechat/bind`、`policies/{id}/beneficiaries`、`claims/{id}/documents`、`admin/audit-logs` 已并入上表。费率计算（v1.6.0）：`quote_rates` 命中 → premium = 保额 × rate；未命中或费率表缺失（ERRNO 1146）→ 回退使用请求保费。

### 路由注册示例（bee_router 风格）

```rust
use bee_router::{Router, controller::Controller};

pub fn routes() -> Router {
    Router::new()
        .namespace("/api/v1", |api| {
            api.namespace("/auth", |r| {
                r.post("/register", AuthController::register)
                    .post("/login", AuthController::login)
                    .post("/wechat/login", AuthController::wechat_login)
                    .post("/refresh", AuthController::refresh);
            })
            .namespace("/products", |r| {
                r.get("/", ProductController::list)
                    .get("/featured", ProductController::featured)
                    .get("/{id}", ProductController::detail)
                    .get("/{id}/clauses", ProductController::clauses);
            })
            .namespace("/quotes", |r| {
                r.post("/", QuoteController::create)
                    .get("/{id}", QuoteController::detail);
            })
            .namespace("/orders", |r| {
                r.post("/", OrderController::create)
                    .get("/", OrderController::my_orders)
                    .get("/{id}", OrderController::detail);
            })
            .namespace("/payments", |r| {
                r.post("/{order_id}/prepay", PaymentController::prepay)
                    .post("/{order_id}/pay", PaymentController::pay)
                    .post("/wechat/prepay", PaymentController::wechat_prepay)
                    .post("/callback/{provider}", PaymentController::callback);
            })
            .namespace("/policies", |r| {
                r.get("/", PolicyController::my_policies)
                    .get("/{id}", PolicyController::detail);
            })
            .namespace("/contracts", |r| {
                r.get("/{id}", ContractController::detail)
                    .post("/{id}/sign", ContractController::sign)
                    .get("/{id}/sign-url", ContractController::sign_url)
                    .post("/callback/{provider}", ContractController::callback);
            })
            .namespace("/search", |r| {
                r.get("/", SearchController::search);
            })
            .namespace("/claims", |r| {
                r.post("/", ClaimController::create)
                    .get("/", ClaimController::my_claims);
            })
            .namespace("/user", |r| {
                r.get("/me", UserController::me);
            })
        })
}
```

---

## 5. 中间件 / 过滤器链

过滤器链装配顺序（对标 Beego FilterChain），任何环节可中断（Abort）：

```
请求 ──► [1] SecurityFilter ──► [2] Trace ──► [3] Session恢复 ──► [4] JWT认证(RBAC)
          (security-rust)                                      │
                                                               ▼
                                      [5] 参数校验 ──► Controller ──► [6] 审计(写audit_logs)
                                                               │
                                                               ▼
                                                            响应(ResponseEnvelope)
```

### 5.1 SecurityFilter（security-rust）

```rust
use bee_router::filter::Filter;
use security_rust::Scanner;

pub struct SecurityFilter { scanner: Scanner }

impl SecurityFilter {
    pub fn new() -> Self {
        Self { scanner: Scanner::default() }  // 27 个检测器全开
    }
}

impl Filter for SecurityFilter {
    fn name(&self) -> &'static str { "security" }

    // 对 url + query + body 逐段扫描
    fn before(&self, ctx: &mut Context) -> Result<(), Error> {
        let input = format!("{}?{} {}", ctx.path(), ctx.query_string(), ctx.body_text());
        if let Some(hit) = self.scanner.scan(&input).first() {
            // 记录 + 中断请求（Abort）
            return Err(Error::SecurityRejected(hit.attack_type.clone()));
        }
        Ok(())
    }
}
```

> 复用 `bee_rust` 的 `security` feature 也可一行装配 `SecurityFilter::new()`（bee-rust 已内置对 security-rust 的封装）。此处保留显式 `Scanner` 以便按需定制（如忽略某些路径）。

### 5.2 Trace（trace_id 透传）

- 入口生成 `trace_id`（UUID），注入请求上下文，写入响应头 `X-Trace-Id`；
- 与 `audit_logs.trace_id` 对齐（见 db-schema.md §5.17），贯穿整条链路日志。

### 5.3 JWT 认证 + RBAC

```rust
pub enum Role { User, Agent, Admin, Operator }

// 认证过滤器：解析 Authorization: Bearer <jwt>，注入 ctx.current_user
impl Filter for AuthFilter {
    fn before(&self, ctx: &mut Context) -> Result<(), Error> {
        let token = ctx.header("Authorization")?;
        let claims = auth_service::verify_token(token)?;  // jsonwebtoken
        ctx.set("current_user", claims);
        Ok(())
    }
}

// 角色守卫：在需要特定角色的路由上附加
pub fn require_role(roles: &[Role]) -> impl Filter { /* 校验 ctx.current_user.role ∈ roles */ }
```

### 5.4 审计过滤器

- 对写操作（POST/PUT/PATCH/DELETE），事务内写入 `audit_logs`（before_json / after_json / ip / trace_id）；
- 使用 `after` 钩子捕获响应。

### 5.5 限流过滤器

- Redis 滑动窗口/令牌桶：IP 维度（公开接口）与 user_id 维度（需认证接口）；
- 超限返回 `429 Too Many Requests`。

---

## 6. 统一响应与错误处理

### 6.1 ResponseEnvelope

所有接口返回统一结构（对齐三端约定）：

```rust
#[derive(Serialize, Deserialize)]
pub struct ResponseEnvelope<T> {
    pub code: i32,          // 0 = 成功；非 0 = 业务错误码
    pub message: String,    // 人类可读信息
    pub data: Option<T>,    // 业务数据
    pub trace_id: String,   // 链路追踪
}

impl<T> ResponseEnvelope<T> {
    pub fn ok(data: T) -> Self { Self { code: 0, message: "ok".into(), data: Some(data), trace_id: current_trace() } }
    pub fn err(code: i32, msg: impl Into<String>) -> Self { Self { code, message: msg.into(), data: None, trace_id: current_trace() } }
}
```

### 6.2 错误枚举（error.rs）

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("参数校验失败: {0}")]
    Validation(String),
    #[error("未认证")]
    Unauthorized,
    #[error("无权限")]
    Forbidden,
    #[error("资源不存在")]
    NotFound,
    #[error("状态冲突: {0}")]       // 非法状态流转
    StateConflict(String),
    #[error("业务错误: {0}")]
    Business(String),
    #[error("安全检测拦截: {0}")]
    SecurityRejected(String),
    #[error("支付失败: {0}")]
    Payment(String),
    #[error("电子签失败: {0}")]
    Esign(String),
    #[error("搜索失败: {0}")]
    Search(String),
    #[error("数据库错误: {0}")]
    Db(#[from] bee_orm::Error),
    #[error("内部错误")]
    Internal(#[source] anyhow::Error),
}
```

### 6.3 错误 → HTTP 状态映射

| AppError | HTTP | ResponseEnvelope.code |
|----------|------|----------------------|
| Validation | 400 | 40000 |
| Unauthorized | 401 | 40100 |
| Forbidden | 403 | 40300 |
| NotFound | 404 | 40400 |
| StateConflict | 409 | 40900 |
| SecurityRejected | 403 | 40301 |
| Payment/Esign/Search | 422 | 422xx |
| Db/Internal | 500 | 50000 |

> 所有错误经统一 handler 转为 `ResponseEnvelope`，Controller 只 `return Err(...)`，由框架层兜底序列化。

---

## 7. 认证与鉴权（RBAC）

### 7.1 令牌模型

- **Access Token**（JWT）：短时效（如 2h），无状态，携带 `sub`（user_id）、`role`、`platform`；
- **Refresh Token**：长时效（如 7d），存 Redis（`refresh:{user_id}`），支持吊销；
- 登录 → 签发双令牌；刷新接口用 refresh token 换新 access token。
- **吊销机制（v1.6.0 已实现）**：logout / 改密 / 换绑手机 → `users.token_version + 1`，旧 refresh token 立即失效；JWT Claims 携带 `token_version`（`#[serde(default)]` 兼容历史令牌）。

### 7.2 三种登录渠道

| 渠道 | 流程 |
|------|------|
| 密码登录 | 手机号/用户名 + 密码（argon2 校验）→ 签发 JWT |
| 验证码登录 | 短信验证码（Redis 存验证码）→ 签发 JWT |
| 微信登录 | `wx.login` code → 后端 code2session → 按 openid 查用户：已绑定 → 直登签发 JWT；未绑定 → 提示先登录既有账号再调 wechat/bind 绑定；微信未配置 → 降级（v1.6.0 已实现） |

### 7.3 RBAC 角色

| 角色 | 权限 |
|------|------|
| `USER` | 投保人：浏览/报价/下单/支付/保单/合同/理赔 |
| `AGENT`（预留） | 经纪人：管理名下客户保单 |
| `OPERATOR` | 运营：产品上下架、审核 |
| `ADMIN` | 系统管理：用户/权限/审计 |

### 7.4 对象级越权防护

- 属主校验：`quote.user_id == ctx.current_user.id`，否则 `Forbidden`；
- 敏感数据（身份证/手机号）仅返回脱敏值，明文仅经 `CryptoService.decrypt` 在服务端使用；
- 管理端接口强制 `ADMIN/OPERATOR` 角色守卫。

---

## 8. 支付抽象 PayProvider

### 8.1 Trait 定义

```rust
use async_trait::async_trait;
use rust_decimal::Decimal;

#[async_trait]
pub trait PayProvider: Send + Sync {
    fn name(&self) -> &'static str;                       // "MOCK" | "WECHAT"

    /// 创建预支付，返回可拉起的收银台参数（存 payments.prepay_payload）
    async fn create_payment(&self, order: &Order, amount: Decimal) -> Result<PrepayResult>;

    /// 主动查询支付状态（兜底/对账）
    async fn query_status(&self, provider_tx_id: &str) -> Result<PayStatus>;

    /// 处理渠道异步回调报文，验签后返回结果（支付服务据此更新状态）
    async fn handle_callback(&self, provider: &str, payload: &[u8]) -> Result<CallbackResult>;
}

pub struct PrepayResult {
    pub provider_tx_id: String,
    pub pay_params: serde_json::Value,   // 前端拉起收银台所需参数
}

#[derive(Debug, Clone, PartialEq)]
pub enum PayStatus { Success, Failed, Pending, Refunded }

pub struct CallbackResult {
    pub provider_tx_id: String,
    pub status: PayStatus,
    pub raw_payload: serde_json::Value,  // 原文留痕（payments.callback_payload）
}
```

### 8.2 Mock 实现

```rust
pub struct MockPayProvider;

#[async_trait]
impl PayProvider for MockPayProvider {
    fn name(&self) -> &'static str { "MOCK" }

    async fn create_payment(&self, order: &Order, _amount: Decimal) -> Result<PrepayResult> {
        let tx_id = format!("MOCK-{}-{}", order.order_no, uuid::Uuid::new_v4());
        Ok(PrepayResult {
            provider_tx_id: tx_id.clone(),
            pay_params: serde_json::json!({ "mock_url": format!("/pay/mock/{tx_id}") }),
        })
    }

    async fn query_status(&self, provider_tx_id: &str) -> Result<PayStatus> {
        // 简单模拟：仅当后端主动调用 payments/{orderId}/pay 时才置为成功
        Ok(PayStatus::Pending)
    }

    async fn handle_callback(&self, _provider: &str, payload: &[u8]) -> Result<CallbackResult> {
        let raw: serde_json::Value = serde_json::from_slice(payload)?;
        Ok(CallbackResult {
            provider_tx_id: raw["tx_id"].as_str().unwrap_or_default().into(),
            status: PayStatus::Success,
            raw_payload: raw,
        })
    }
}
```

### 8.3 渠道注册与分发

```rust
pub struct PaymentProviderRegistry {
    providers: HashMap<&'static str, Box<dyn PayProvider>>,
}

impl PaymentProviderRegistry {
    pub fn new() -> Self {
        let mut m = HashMap::new();
        m.insert("MOCK", Box::new(MockPayProvider) as Box<dyn PayProvider>);
        // 预留: m.insert("WECHAT", Box::new(WechatPayProvider::new(cfg)));
        Self { providers: m }
    }

    pub fn get(&self, name: &str) -> Result<&dyn PayProvider> {
        self.providers.get(name).map(|p| p.as_ref()).ok_or_else(|| AppError::Payment(format!("未知渠道 {name}")))
    }
}
```

### 8.4 支付状态机（对齐 db-schema.md §5.10）

```
payments.status: CREATED → PROCESSING → SUCCESS
                    │         │
                    ▼         ▼
                 CANCELLED   FAILED
                 SUCCESS → REFUNDED
```

支付回调处理要点：
- **幂等**：`uk_payment_tx(provider, provider_tx_id)` 唯一约束防重复回调；
- **金额校验**：回调金额与订单 `payable_amount` 比对，不符即告警拒绝；
- **回调安全**：验签失败丢弃；`callback_payload` 原文留痕；
- **成功回调** → 事务内 `payment SUCCESS` + `order PAID` + 写 audit_log。

---

## 9. 电子签抽象 ElectronicSignature

### 9.1 Trait 定义

```rust
#[async_trait]
pub trait ElectronicSignature: Send + Sync {
    fn name(&self) -> &'static str;                       // "MOCK" | "ESIGN"

    /// 创建签署流程，返回平台流程 ID 与各签署方签署链接
    async fn create_contract(&self, contract: &Contract, signers: &[ContractSigner]) -> Result<EsignCreateResult>;

    /// 获取指定签署方的签署 URL
    async fn get_sign_url(&self, sign_flow_id: &str, signer: &ContractSigner) -> Result<String>;

    /// 校验签署是否全部完成
    async fn verify_completion(&self, sign_flow_id: &str) -> Result<bool>;
}

pub struct EsignCreateResult {
    pub sign_flow_id: String,          // 存 contracts.sign_flow_id
    pub sign_urls: Vec<(i64, String)>, // (contract_signer.id, url)
}
```

### 9.2 Mock 实现

```rust
pub struct MockEsignProvider;

#[async_trait]
impl ElectronicSignature for MockEsignProvider {
    fn name(&self) -> &'static str { "MOCK" }

    async fn create_contract(&self, contract: &Contract, signers: &[ContractSigner]) -> Result<EsignCreateResult> {
        let sign_flow_id = format!("MOCK-FLOW-{}", contract.contract_no);
        let sign_urls = signers.iter().map(|s| (s.id, format!("/sign/mock/{sign_flow_id}/{}", s.id))).collect();
        Ok(EsignCreateResult { sign_flow_id, sign_urls })
    }

    async fn get_sign_url(&self, sign_flow_id: &str, _signer: &ContractSigner) -> Result<String> {
        Ok(format!("/sign/mock/{sign_flow_id}"))
    }

    async fn verify_completion(&self, sign_flow_id: &str) -> Result<bool> {
        // 模拟：回调被调用即视为完成
        Ok(true)
    }
}
```

### 9.3 合同签署流程

```
保单生成 → 创建合同(DRAFT) → create_contract → 各签署方生成 sign_url
→ 合同 PENDING_SIGN → 前端 WebView 打开 sign_url 签署
→ 电子签回调 → 校验全部签完 → 合同 COMPLETED + 更新 contract_signers.status
```

- 合同 PDF 生成后计算 `file_hash`（SHA-256）防篡改；
- 回调验签 + 幂等（`contract_no` 或 `sign_flow_id` 唯一）；
- 预留 `ESignQianProvider`（e签宝）实现同一 trait。

---

## 10. 搜索服务层与同步

### 10.1 搜索门面（SearchService）

业务 Controller 通过 `SearchService` 访问搜索，不直接接触 rust-scout 底层：

```rust
use rust_scout::{Engine, EngineManager, ScoutConfig, SearchBuilder};

pub struct SearchService {
    engine: Box<dyn Engine>,   // EngineManager 解析配置得到
}

impl SearchService {
    pub fn new(cfg: ScoutConfig) -> Result<Self> {
        let engine = EngineManager::new(cfg).engine()?;
        Ok(Self { engine })
    }

    /// 全文搜索：keyword 命中指定索引
    pub async fn search(&self, index: &str, keyword: &str, status: &str, page: u32, size: u32) -> Result<SearchResult> {
        let builder = SearchBuilder::new(keyword)
            .within(index)
            .where_field("status", status)
            .order_by("created_at", true)
            .take(size as usize)
            .skip(((page - 1) * size) as usize);
        self.engine.search(builder).await.map_err(AppError::Search)
    }
}
```

### 10.2 Searchable 实现（对齐 db-schema.md §7）

Product / Clause / Policy 三个实体实现 `Searchable`，`to_doc()` 返回索引文档 JSON（敏感字段只放脱敏值）：

```rust
use rust_scout::{Searchable, SearchDocument};

impl Searchable for InsuranceProduct {
    fn index_name(&self) -> &'static str { "insurance_products" }
    fn doc_id(&self) -> String { self.id.to_string() }

    fn to_doc(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "product_code": self.product_code,
            "name": self.name,
            "subtitle": self.subtitle,
            "description": self.description,
            "product_type": self.product_type,
            "sale_channel": self.sale_channel,
            "insurer_name": self.insurer_name,
            "currency": self.currency,
            "min_amount": self.min_amount,
            "max_amount": self.max_amount,
            "min_term_months": self.min_term_months,
            "max_term_months": self.max_term_months,
            "waiting_period_days": self.waiting_period_days,
            "category_slugs": /* 联表取分类 slug */,
            "is_featured": self.is_featured,
            "status": self.status,
            "created_at": self.created_at,
        })
    }
}
```

### 10.3 DB → OpenSearch 同步（对齐 db-schema.md §9）

**写路径**（业务事务内不阻塞）：

```rust
// order_service 内示例：支付成功后生成保单并登记同步
let policy = policy_service::issue(&order).await?;      // 写 policies 表

// 同一事务登记同步任务（主表 + search_sync_logs 原子提交）
let doc = policy.to_doc();
sync_repo::enqueue(SearchSyncLog {
    entity_type: "POLICY".into(),
    entity_id: policy.id,
    op: "UPSERT".into(),
    status: "PENDING".into(),
    payload_json: Some(doc),
    ..Default::default()
}).await?;
```

**消费路径**（后台 SyncWorker）：

```rust
pub async fn run(engine: &dyn Engine, pool: &DbPool) {
    loop {
        let rows = sync_repo::claim_pending(pool, 50).await?;  // FOR UPDATE SKIP LOCKED
        for row in rows {
            let result = match row.op.as_str() {
                "UPSERT" => engine.update(&[SearchDocument::from_json(&row.doc_id, row.payload_json)?]).await,
                "DELETE" => engine.delete(&[row.doc_id]).await,
                _ => continue,
            };
            match result {
                Ok(_) => sync_repo::mark_success(pool, row.id).await?,
                Err(e) => {
                    if row.attempts + 1 >= row.max_attempts {
                        sync_repo::mark_dead(pool, row.id, &e.to_string()).await?;   // DEAD，人工告警
                    } else {
                        sync_repo::mark_retry(pool, row.id, &e.to_string()).await?;  // 指数退避
                    }
                }
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
```

**要点**：
- 幂等：索引 `_id = entity_id`，UPSERT 覆盖、DELETE 幂等；
- 重试：指数退避 `next_retry_at = NOW() + 2^attempts`，超 `max_attempts` 转 `DEAD`；
- 多实例：`FOR UPDATE SKIP LOCKED` 防重复消费；
- 一致性窗口：允许秒级最终一致，强实时场景可走 DB 兜底。

---

## 11. 关键控制器签名

bee_router `Controller` trait + `Context`。以下为各控制器核心动作签名（对齐 §4 路由）：

```rust
use bee_router::{controller::Controller, Context};

// ---- Auth ----
pub struct AuthController;

impl Controller for AuthController {
    type Error = AppError;

    async fn register(&self, ctx: &mut Context) -> Result<(), AppError> {
        let req: RegisterReq = ctx.json()?;                    // 参数反序列化
        let tokens = auth_service::register(req).await?;        // 建用户 + 签发
        ctx.json(ResponseEnvelope::ok(tokens))?;
        Ok(())
    }

    async fn login(&self, ctx: &mut Context) -> Result<(), AppError> {
        let req: LoginReq = ctx.json()?;
        let tokens = auth_service::login(req).await?;
        ctx.json(ResponseEnvelope::ok(tokens))?;
        Ok(())
    }

    async fn wechat_login(&self, ctx: &mut Context) -> Result<(), AppError> {
        let req: WechatLoginReq = ctx.json()?;                 // { code }
        let tokens = auth_service::wechat_login(req.code).await?;
        ctx.json(ResponseEnvelope::ok(tokens))?;
        Ok(())
    }
}

// ---- Product ----
pub struct ProductController;

impl Controller for ProductController {
    type Error = AppError;

    async fn list(&self, ctx: &mut Context) -> Result<(), AppError> {
        let page = ctx.query("page").unwrap_or(1);
        let size = ctx.query("size").unwrap_or(20);
        let status = ctx.query("status").unwrap_or("ON_SALE");
        let products = product_service::list(status, page, size).await?;
        ctx.json(ResponseEnvelope::ok(products))?;
        Ok(())
    }

    async fn detail(&self, ctx: &mut Context) -> Result<(), AppError> {
        let id: i64 = ctx.param("id")?.parse()?;
        let product = product_service::detail(id).await?;
        ctx.json(ResponseEnvelope::ok(product))?;
        Ok(())
    }
}

// ---- Search ----
pub struct SearchController;

impl Controller for SearchController {
    type Error = AppError;

    async fn search(&self, ctx: &mut Context) -> Result<(), AppError> {
        let keyword = ctx.query("keyword").unwrap_or_default();
        let type_: Option<String> = ctx.query("type");          // product|clause|policy
        let page = ctx.query("page").unwrap_or(1);
        let size = ctx.query("size").unwrap_or(20);
        let result = search_service::search(keyword, type_.as_deref(), page, size).await?;
        ctx.json(ResponseEnvelope::ok(result))?;
        Ok(())
    }
}

// ---- Policy（示例：保单生成走 Order 支付回调后触发）----
pub struct PolicyController;

impl Controller for PolicyController {
    type Error = AppError;

    async fn my_policies(&self, ctx: &mut Context) -> Result<(), AppError> {
        let user = ctx.current_user()?;                          // JWT 注入
        let policies = policy_service::list_by_user(user.id).await?;
        ctx.json(ResponseEnvelope::ok(policies))?;
        Ok(())
    }

    async fn detail(&self, ctx: &mut Context) -> Result<(), AppError> {
        let id: i64 = ctx.param("id")?.parse()?;
        let user = ctx.current_user()?;
        let policy = policy_service::detail_owned(id, user.id).await?;   // 属主校验
        ctx.json(ResponseEnvelope::ok(policy))?;
        Ok(())
    }
}
```

### 请求/响应 DTO 示例

```rust
#[derive(Deserialize)]
pub struct RegisterReq {
    pub username: String,
    pub password: String,        // argon2 哈希后存 password_hash
    pub phone: String,           // AES 加密存 phone_enc，另存 phone_masked
}

#[derive(Deserialize)]
pub struct LoginReq { pub username: String, pub password: String }

#[derive(Deserialize)]
pub struct WechatLoginReq { pub code: String }
```

---

## 12. 配置与部署

### 12.1 配置结构（config.rs，bee_config #[derive(Config)]）

```rust
#[derive(Config)]
pub struct AppConfig {
    pub server: ServerConfig,      // host/port
    pub database: DbConfig,        // MySQL 连接串
    pub redis: RedisConfig,        // Redis 连接串
    pub opensearch: SearchConfig,  // OpenSearch 地址/认证
    pub jwt: JwtConfig,            // secret/issuer/expiry
    pub crypto: CryptoConfig,      // AES 主密钥
}
```

### 12.2 `.env.example`

```env
# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# MySQL
DATABASE_URL=mysql://insurance:password@127.0.0.1:3306/insurance_service

# Redis
REDIS_URL=redis://127.0.0.1:6379

# OpenSearch
OPENSEARCH_URL=http://127.0.0.1:9200
OPENSEARCH_USERNAME=admin
OPENSEARCH_PASSWORD=changeme

# JWT
JWT_SECRET=change-me-to-a-long-random-string
JWT_ISSUER=insurance-service
JWT_ACCESS_EXPIRY=7200
JWT_REFRESH_EXPIRY=604800

# AES 主密钥（32 字节 base64）
CRYPTO_MASTER_KEY=base64:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=

# 微信（预留）
WECHAT_APPID=
WECHAT_SECRET=
WECHAT_MCH_ID=
WECHAT_API_V3_KEY=
```

### 12.3 `config/app.toml`

```toml
[server]
host = "0.0.0.0"
port = 8080

[database]
url = "${DATABASE_URL}"

[redis]
url = "${REDIS_URL}"

[opensearch]
url = "${OPENSEARCH_URL}"
username = "${OPENSEARCH_USERNAME}"
password = "${OPENSEARCH_PASSWORD}"

[jwt]
secret = "${JWT_SECRET}"
issuer = "${JWT_ISSUER}"
access_expiry = 7200
refresh_expiry = 604800

[crypto]
master_key = "${CRYPTO_MASTER_KEY}"
```

### 12.4 部署拓扑

```
                ┌──────────────┐
   Flutter ────►│              │
   小程序 ─────►│  Nginx/LB    │── HTTPS ──► bee-rust 应用(多实例)
   鸿蒙 ──────►│ (TLS+WAF)    │
                └──────────────┘
                        │
       ┌────────────────┼──────────────────┐
       ▼                ▼                  ▼
   MySQL(主从)      Redis(缓存/会话)    OpenSearch(集群)
       │                                    ▲
       └────────── SyncWorker 同步 ──────────┘
```

- **多实例**：无状态（JWT 无状态、会话存 Redis），横向扩展；
- **SyncWorker**：独立进程或单实例内单任务运行，避免重复消费；
- **CI/CD**：`cargo fmt` → `cargo clippy -D warnings` → `cargo test` → 构建 release → 容器化（Docker）部署；
- **HTTPS**：Nginx/LB 终结 TLS，WAF 前置（与 security-rust 互补）。

---

## 13. 分阶段实现 Roadmap

### 阶段 0 — 后端骨架（先行）
- [ ] Cargo workspace + bee-rust git 依赖 + 配置加载（bee_config）
- [ ] `install.sql`（取自 db-schema.md §4）+ bee_orm Model 结构体（§6 蓝本）
- [ ] MySQL 连接（bee_orm mysql feature）、Redis（bee_kv）
- [ ] OpenSearch 连接（bee_search opensearch feature）+ rust-scout 封装
- [ ] SecurityFilter + JWT 认证 + RBAC + 统一 ResponseEnvelope
- [ ] 基础控制器（auth / user / product / search）+ 健康检查 `/healthz`
- [ ] CryptoService（AES-256-GCM + 脱敏）
- [ ] SyncWorker 骨架 + search_sync_logs 消费

### 阶段 1 — 核心交易闭环
- [ ] 产品管理 + 分类 + 条款（含搜索索引同步）
- [ ] 报价（QuoteService + 保费计算 PricingService）
- [ ] 订单 + 支付（PayProvider 接口 + Mock，支付回调幂等）
- [ ] 保单生成（PolicyService：保单号、PDF、受益人）
- [ ] 电子合同 + 签署（ElectronicSignature 接口 + Mock）
- [ ] 理赔基础流程

### 阶段 2 — Flutter 主端对接
- [ ] 三端共用 REST API 联调（auth → 报价 → 订单 → 支付 → 保单 → 签署）
- [ ] 产品搜索 + 首页推荐

### 阶段 3 — 小程序 + 鸿蒙
- [x] 微信登录（v1.6.0 已实现：wechat/login openid 直登 + wechat/bind 绑定闭环）
- [ ] 微信支付真实适配
- [ ] 鸿蒙 ArkTS 端联调

### 阶段 4 — 真实渠道与加固
- [ ] 微信支付（WechatPayProvider）真实对接
- [ ] e签宝 / 法大大（ESignQianProvider / FaDaDaProvider）真实对接
- [ ] 短信服务、对象存储（保单/合同 PDF）
- [ ] 监控告警（tracing + metrics）、性能优化、安全加固
- [ ] 等保合规审计、敏感数据密钥轮换（KMS）

---

## 结语

本文档为后端实现提供完整蓝本：MVC 分层、bee-rust 过滤器链装配、统一响应/错误处理、可插拔支付与电子签抽象、rust-scout 搜索门面与 DB→OpenSearch 异步同步。所有设计与 [db-schema.md](./db-schema.md) 的状态机、字段、枚举严格对齐，可直接进入阶段 0 骨架搭建。
