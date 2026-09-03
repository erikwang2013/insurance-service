# insurance-service

保险服务平台后端服务（Rust 版）。基于 `bee_rust`（bee 管线 + 路由 + ORM 骨架）、`axum`、`tokio`
构建，目标覆盖保险业务从「商品展示 → 报价 → 订单 → 支付 → 保单 → 电子签约 → 理赔」的完整闭环。

当前迭代处于 **阶段 0 → 1**：已落地**用户认证、保险商品、全文搜索、报价、理赔（含审核）**业务能力，并打通
**报价 → 订单 → 支付 → 保单 → 签约**交易闭环 API（支付 / 电子签当前为 Mock 渠道），并已提供**运营后台商品建档与上下架**接口。

| 项目 | 值 |
|------|-----|
| 语言 / 版本 | Rust 2024 edition（rust-version ≥ 1.87） |
| 许可证 | Apache-2.0 |
| 版本 | 1.5.0 |
| HTTP 框架 | axum 0.8 + bee_rust（bee_router / bee_orm / 过滤器管线） |
| 存储 | MySQL 8.4（业务库）· Redis / 内存缓存（会话）· OpenSearch（搜索，可选） |

<p align="center">
  <img src="docs/mascot.svg" width="120" height="120" alt="吉祥物安安——守护熊猫" />
</p>

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
- **外部 Provider**：微信支付 / 支付宝 / 易签电子签约 / Mock 适配层（规划路由）。
- **数据访问**：`Db`（mysql_async 连接池）——参数化查询防注入、写事务 `commit / rollback` 闭环、
  `bee_orm Model` 行 → 结构体映射。
- **基础设施**：MySQL 8.4（`install.sql` 建库）、Redis 会话缓存（阶段 0 内存缓存暂代）、
  OpenSearch 全文索引（rust-scout 门面，未就绪时降级 MySQL LIKE 搜索 + 同步 Worker）。

## 项目功能

![项目功能](docs/features.svg)

- **用户与认证**：注册（用户名唯一 + argon2 口令哈希）、登录（账号密码；微信登录 code2session 客户端已配置化，凭据未配置时降级报错）、
  JWT 双令牌（access 短时效 + refresh 长时效）与 RBAC 角色（USER / AGENT / OPERATOR / ADMIN）。
- **保险商品**：列表分页 / 状态过滤 / 精选位、详情、关联条款阅读，运营后台建档与上架 / 下架 / 停售（OPERATOR / ADMIN）。
- **全文搜索**：当前 MySQL LIKE（带分页保护），规划 OpenSearch 索引 + 同步 Worker，未就绪自动降级。
- **交易闭环**：报价 → 订单 → 支付（预下单 / 支付 / 回调）API 已挂载；微信渠道当前为 Mock，真实渠道规划中。
- **保单与契约**：保单列表 / 详情、电子合同 Mock 签署与签署回调 API 已挂载；易签等外部电子签渠道规划中。
- **理赔**：报案（`CLM` 单号，校验保单归属）、我的理赔列表、审核（APPROVE / REJECT，需 OPERATOR / ADMIN）已实现。
- **运营 / 审计（规划）**：运营后台 API、`audit_log` 全量留痕。
- **横切能力**：入站安全扫描、参数化查询、AES-256-GCM 敏感字段加密、令牌精确过期（leeway=0）、
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
│   ├── app.toml               # 应用配置模板（server / database / redis / opensearch / jwt / crypto / log）
│   └── bee.toml               # bee_rust 管线配置
├── docs/                      # 文档与架构图
│   ├── architecture.svg       # 系统架构图
│   ├── features.svg           # 功能总览图
│   ├── lifecycle.svg          # 请求生命周期图
│   ├── backend-architecture.md
│   └── db-schema.md
├── src/
│   ├── main.rs                # 启动入口：init → 配置 → AppState → 路由 → serve
│   ├── lib.rs                 # 库入口（集成测试依赖）
│   ├── config.rs              # AppConfig：从环境变量 + config/app.toml 加载
│   ├── routes.rs              # 数据驱动路由表（35 个业务端点）+ 已挂载 handler
│   ├── db.rs                  # mysql_async 连接池与查询 / 事务封装
│   ├── error.rs               # AppError（BadRequest / Unauthorized / NotFound / Business…）
│   ├── controllers/mod.rs     # AppState · bee 管线 run() · 统一信封
│   ├── middleware/mod.rs      # Filter trait · RequestCtx · trace_id
│   ├── middleware/auth.rs     # JwtService · AuthFilter · RequireRoleFilter
│   ├── middleware/security.rs # SecurityFilter
│   ├── services/
│   │   ├── mod.rs
│   │   ├── auth_service.rs    # 注册 / 登录 / 单点签发
│   │   ├── product_service.rs # 商品列表 / 详情 / 精选
│   │   ├── search_service.rs  # 搜索（LIKE 降级）
│   │   ├── quote_service.rs   # 报价
│   │   ├── order_service.rs   # 订单
│   │   ├── payment_service.rs # 支付（预下单 / 回调）
│   │   ├── policy_service.rs  # 保单
│   │   ├── contract_service.rs# 电子合同（Mock 签署）
│   │   └── claim_service.rs   # 理赔（报案 / 我的理赔）
│   ├── models/                # bee_orm Model（user / insurance_product / order / payment / policy / contract / claim…）
│   ├── crypto/                # AES-256-GCM · Masker · argon2
│   ├── providers/             # payment（wechat · mock）· esign（escqian · mock）
│   ├── search/                # searchable_impl · sync_worker
│   └── utils/                 # validator · id_generator
└── tests/
    ├── common/mod.rs           # 测试共享设施（测试库连接 / JWT 配置 / 唯一值 / 清理）
    ├── auth_service_test.rs    # 注册 / 登录 / 微信 stub（6 项）
    ├── product_service_test.rs # 商品增删改查 / 过滤 / 软删（5 项）
    ├── search_service_test.rs  # 搜索命中 / 无果 / 分页 / 索引路由（4 项）
    ├── quote_service_test.rs   # 报价试算 / 详情 / 鉴权（3 项）
    ├── security_test.rs        # JWT 校验 / 过期 / 角色 RBAC（18 项）
    ├── api_auth_test.rs        # API 层鉴权 E2E（8 项）
    ├── claim_service_test.rs   # 理赔报案 / 归属校验 / 分页（5 项）
    ├── claim_review_test.rs    # 理赔审核 APPROVE / REJECT（5 项）
    ├── admin_product_test.rs   # 商品上架 / 下架管理端（5 项）
    ├── auth_fix_test.rs        # 认证修复回归：#10 修复项（8 项）
    ├── product_fix_test.rs     # 产品模块修复回归：HTTP 层（4 项）
    └── contract_fix_test.rs    # 签约修复回归：sign-url Mock 真实化（4 项）
```

## 使用说明

### 环境要求

- Rust toolchain ≥ 1.87（`cargo --version` 检查）
- MySQL 8.4（可选，集成测试与正式运行需要）
- Redis / OpenSearch（可选，阶段 0 未就绪会自动降级）

### 配置

配置通过环境变量注入，模板见 `.env.example`，默认值见 `config/app.toml`：

| 变量 | 说明 | 缺省 |
|------|------|------|
| `SERVER_HOST` / `SERVER_PORT` | 监听地址 | `0.0.0.0` / `8080` |
| `DATABASE_URL` | MySQL 连接串（**必填**，如 `mysql://user:pass@127.0.0.1:3306/insurance_service`） | — |
| `REDIS_URL` | Redis 会话缓存 | 缺省走内存缓存 |
| `OPENSEARCH_URL` / `OPENSEARCH_INDEX_*` | 全文索引 | 缺省降级 LIKE |
| `JWT_SECRET` / `JWT_ISSUER` | JWT 签名密钥 / 签发者 | — |
| `JWT_ACCESS_EXPIRY` / `JWT_REFRESH_EXPIRY` | access / refresh 有效秒数 | `7200` / `604800` |
| `CRYPTO_MASTER_KEY` | AES-256-GCM 主密钥（口令 / 敏感字段加密） | — |
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

- 依赖 MySQL 的集成测试在未配置 `DATABASE_URL` 或未执行 `install.sql` 时会打印 `SKIP` 并跳过，
  保证无库环境 `cargo test` 不失败（v1.5.0 全量 107 项全绿：单元 13 + 集成 94）。集成测试覆盖：
  认证（含微信未配置降级）、API 鉴权 E2E、交易闭环（报价→订单→支付回调→保单签发）、
  保单生命周期（续保/退保）、限流、用户中心（改密/换绑）、运营统计、理赔、商品、修复回归等。
- 按项目约定（CLAUDE.md）：代码变更后**先跑测试、再提交**。

### 已实现 API 概览（阶段 0 → 1）

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/healthz` | 健康检查 |
| POST | `/api/v1/auth/register` | 注册（返回双令牌） |
| POST | `/api/v1/auth/login` | 账号密码登录 |
| POST | `/api/v1/auth/wechat/login` | 微信登录（code2session 客户端已配置化，未配置凭据时降级报错；openid 绑定期待阶段 3） |
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
| GET | `/api/v1/contracts/{id}` | 电子合同详情 |
| POST | `/api/v1/contracts/{id}/sign` · GET `/sign-url` | 合同签署 / 签署 URL |
| POST | `/api/v1/contracts/callback/{provider}` | 签署回调 |
| POST | `/api/v1/claims` | 理赔报案（校验保单归属） |
| GET | `/api/v1/claims?user_id=&page=&size=` | 我的理赔（分页） |
| POST | `/api/v1/claims/{id}/review` | 理赔审核 APPROVE / REJECT（OPERATOR / ADMIN） |
| POST | `/api/v1/admin/products` · `/api/v1/admin/products/{id}/status` | 商品建档 / 上下架（OPERATOR / ADMIN） |

完整路由表（共 35 个业务端点，含 `/user/me`、`/user/password`、`/user/phone`、运营后台 `/admin/*`）
见 `src/routes.rs` `route_table()`；库表设计见 `docs/db-schema.md` 与 `install.sql`。

---

## 支持与打赏

如果本项目对您有帮助，欢迎打赏支持（一分也是爱 💖）。

### 微信支付 / 支付宝

| 微信支付 | 支付宝 |
|----------|--------|
| <img src="docs/weixinpay.png" width="130" height="130" alt="微信支付收款码"/> | <img src="docs/alipay.png" width="130" height="130" alt="支付宝收款码"/> |

打开对应 App 扫码即可；转账时欢迎备注昵称，感谢支持！

### 虚拟币打赏

| 币种 | 主网 | 二维码 | 钱包地址 |
|------|------|--------|----------|
| BNB | BNB Smart Chain (BEP20) | <img src="docs/coin/1.jpg" width="130" height="130" alt="BNB Smart Chain (BEP20) 收款码"/> | `0x355d429f97511897ccb4e271ec888205f9ab6629` |
| TRX | Tron (TRC20) | <img src="docs/coin/2.jpg" width="130" height="130" alt="Tron (TRC20) 收款码"/> | `TEdDHWLajt1XvqtPDWmQctdrJaC3pzZZzz` |
| ETH | Ethereum (ERC20) | <img src="docs/coin/3.jpg" width="130" height="130" alt="Ethereum (ERC20) 收款码"/> | `0x355d429f97511897ccb4e271ec888205f9ab6629` |
| APT | Aptos | <img src="docs/coin/4.jpg" width="130" height="130" alt="Aptos 收款码"/> | `0x836e3780edfc3f7b2372b39e2a1a3a5d7adfaccd96c726f21cfde1b50dd68030` |
| — | Plasma | <img src="docs/coin/5.jpg" width="130" height="130" alt="Plasma 收款码"/> | `0x355d429f97511897ccb4e271ec888205f9ab6629` |
| MATIC | Polygon POS | <img src="docs/coin/6.jpg" width="130" height="130" alt="Polygon POS 收款码"/> | `0x355d429f97511897ccb4e271ec888205f9ab6629` |
| SOL | Solana | <img src="docs/coin/7.jpg" width="130" height="130" alt="Solana 收款码"/> | `2hfhboHdmdrYsY25XfQSsEWxq5ip4EQsR7f4AzSRMUyr` |
| TON | The Open Network (TON) | <img src="docs/coin/8.jpg" width="130" height="130" alt="The Open Network (TON) 收款码"/> | `UQB9kFQohzmXUir9QSSZq01iwl9aQZIDdBpNmDklljRtCoGK` |
| ARB | Arbitrum One | <img src="docs/coin/9.jpg" width="130" height="130" alt="Arbitrum One 收款码"/> | `0x355d429f97511897ccb4e271ec888205f9ab6629` |
| AVAX | AVAX C-Chain | <img src="docs/coin/10.jpg" width="130" height="130" alt="AVAX C-Chain 收款码"/> | `0x355d429f97511897ccb4e271ec888205f9ab6629` |

> 虚拟币打赏按 `docs/coin/1.jpg ~ 10.jpg` 顺序对应上方 10 条主网排列；如有出入请指出，即时更正。