# 保险服务平台 — 后端架构

> 版本: 2026-10-30（v1.11.0）。本文档描述当前实现；历史规划中已移除的模块不再记录。

---

## 1. 总体架构

```
客户端（Flutter / 小程序 / HarmonyOS / 运营后台）
        │  HTTP / JSON
        ▼
axum + bee_router（/healthz + /api/v1/*）
        │
Trace → RateLimit → JWT 认证（RBAC）→ Controller
        │
Service（事务边界）→ Db（mysql_async 连接池）→ MySQL 8.4
        │
统一 ResponseEnvelope { code, message, data, trace_id }
```

- 服务只提供 JSON 接口；`/api/v1/*` 与未匹配路径均返回统一 JSON 信封，不提供 HTML 页面。
- 限流与会话缓存使用进程内实现。
- 支付与电子签约当前为内联 Mock 实现；微信 / 支付宝 / 易签等外部渠道未接入。
- 无 `SecurityFilter` / 入站 WAF / 内容扫描器；安全边界依赖 JWT + RBAC、参数化查询、
  2 MiB 请求体限制与限流。中间件均为 `src/routes.rs` 中的 axum middleware，不存在本地
  `Filter` / `RequestCtx` / `AuthFilter` / `RequireRoleFilter` 抽象。

## 2. 目录结构

```
insurance-service/
├── Cargo.toml
├── install.sql                 # MySQL 建库（14 表）
├── config/
│   ├── app.toml                # server / database / jwt / crypto / payment / log
│   └── bee.toml
├── docs/
├── vendor/bee-rust/            # bee-rust 1.0.6 vendored（源码快照 93dd5c57）
├── src/
│   ├── main.rs                 # 启动入口
│   ├── lib.rs
│   ├── config.rs               # AppConfig（环境变量 + app.toml）
│   ├── routes.rs               # build_bee_router()，41 业务端点 + /healthz + 中间件
│   ├── db.rs                   # 连接池 / 查询 / with_tx
│   ├── error.rs                # AppError
│   ├── response.rs             # ResponseEnvelope
│   ├── controllers/            # auth / product / search / quote / order /
│   │                           # payment / policy / contract / claim / admin
│   ├── middleware/             # trace / auth / rate_limit
│   ├── services/               # 各业务 Service（事务边界）
│   ├── models/                 # bee_orm Model
│   ├── providers/wechat.rs     # code2session
│   ├── search/                 # Searchable 序列化辅助
│   ├── crypto/                 # AES-256-GCM / Masker / argon2
│   └── utils/                  # validator / idgen（snowflake）
└── tests/
    ├── common/mod.rs           # 集成测试夹具（配置 / 用户 / 商品 / 费率 / 请求辅助）
    ├── trade_flow.rs           # 交易闭环、归属隔离、回调令牌（DB 门控）
    ├── public_surface.rs       # 公开目录 / 搜索 / 报价费率 / trace（部分 DB 门控）
    └── auth_tokens.rs          # access / refresh 类型隔离（1 项）
```

## 3. 依赖

- `axum` 0.8、`tokio`、`bee_rust`（bee_router / bee_orm；vendored 1.0.6）
- `mysql_async`、`serde` / `serde_json`
- `jsonwebtoken`、`argon2`、`aes-gcm`
- 开源版搜索为 MySQL `LIKE`，不含 OpenSearch / rust_scout。

## 4. 路由

`src/routes.rs::build_bee_router()` 注册 41 个业务端点（`/api/v1` 命名空间）+ `/healthz`。

| 模块 | 端点 | 访问级别 |
|------|------|----------|
| 认证 / 用户 | POST auth/register · auth/login · auth/wechat/login · auth/wechat/bind · auth/refresh · auth/logout；GET user/me；POST user/password · user/phone | 公开：register / login / wechat/login / refresh；其余需登录 |
| 商品 | GET products · products/{id} · products/{id}/clauses · products/featured | 公开（仅 ON_SALE） |
| 搜索 | GET search（`keyword` / `type` / `page` / `size`） | 公开（仅 ON_SALE） |
| 报价 | POST quotes · GET quotes/{id} | 登录 |
| 订单 | POST / GET orders · GET orders/{id} | 登录（归属校验） |
| 支付 | POST payments/{order_id}/prepay · payments/{order_id}/pay · payments/wechat/prepay · payments/callback/{provider} | 前三个需登录；回调公开但需 `X-Callback-Token` |
| 保单 | POST policies（签发）· GET policies · policies/{id} · POST policies/{id}/lapse · policies/{id}/beneficiaries | 登录（归属校验） |
| 合同 | GET contracts/{id} · POST contracts/{id}/sign · GET contracts/{id}/sign-url · POST contracts/callback/{provider} | 前三需登录；回调公开 |
| 理赔 | POST / GET claims · POST claims/{id}/review · POST / GET claims/{id}/documents | 登录；review 需 OPERATOR / ADMIN |
| 运营 | GET / POST admin/products · POST admin/products/{id}/status · GET admin/audit-logs | OPERATOR / ADMIN |

- 公开路由不做 `users` 表查询：`/api/v1/auth/{register,login,wechat/login,refresh}`、
  `/api/v1/products*`、`/api/v1/search`、`/api/v1/payments/callback/*`、`/api/v1/contracts/callback/*`。
- 其余 `/api/v1/*` 一律要求 access JWT；`/api/v1/admin/*` 与 `POST /api/v1/claims/{id}/review`
  额外要求 OPERATOR / ADMIN。
- 未匹配路径统一返回 HTTP 404 + `40400` JSON 信封。

## 5. 中间件链

```
Trace（最外层）→ RateLimit → JWT 认证（RBAC）→ Controller
```

- **Trace**：任务本地 `trace_id`；接受并校验入站 `X-Trace-Id`（1–64 个 ASCII 字母数字 / `-` / `_`），
  否则生成 UUID。响应写回 `X-Trace-Id`；404、401/403、429 与 `/healthz` 也带该头。
- **RateLimit**：进程内固定窗口，300 次 / 60 秒，key 为 `ConnectInfo` 中的对端 socket IP
  （不读取可伪造的 `X-Forwarded-For`，缺失回退 `unknown`）。窗口表达到 10,000 个 key 时清理过期项，
  清理后仍 ≥ 100,000 则 fail closed；超限返回 HTTP 429 + `42900`。
- **JWT / RBAC**：解析 `Authorization: Bearer <access token>`，校验签名 / issuer / exp（leeway=0）
  与 `typ == "access"`；随后从 `users` 表加载最新行，要求 `status = ACTIVE` 且
  `token_version` 与令牌一致，角色以数据库为准。公共路由跳过该查询；运营路由校验
  OPERATOR / ADMIN；成功后注入 `AuthUser` 供 Controller 使用。数据库故障返回 500，不伪装成 401。
- **审计**：`audit_logs` 记录关键操作（含 `trace_id`）；查询端点 `/api/v1/admin/audit-logs`。

## 6. 响应与错误

- 成功：HTTP 200 + `{ "code": 0, "message": "ok", "data": ... }`
- 业务错误：HTTP 200 + `{ "code": 40000, "message": "...", "data": null }`
- 认证 / 权限：HTTP 401 / 403 + 统一信封（`40100` / `40300`）。
- 限流：HTTP 429 + `42900`；未匹配：HTTP 404 + `40400`。
- 数据库 / 内部错误：HTTP 500 + `50000 "服务器内部错误"`；SQL 与 schema 细节只写日志，不返回客户端。

## 7. 认证与 RBAC

- 注册：用户名唯一 + argon2 口令哈希；手机号以 AES-256-GCM 加密存 `users.phone_enc`。
- 登录：账号密码；微信 `code2session` 绑定登录（凭据未配置时降级报错）；登录与微信登录均要求账号 ACTIVE。
- 双令牌：access 与 refresh 使用不同 `typ` 声明；access 用 `JWT_ACCESS_EXPIRY`（默认 7200 秒），
  refresh 用 `JWT_REFRESH_EXPIRY`（默认 604800 秒）。`JWT_SECRET` 必填且 ≥ 32 字节，否则启动失败。
  v1.11 之前签发、不带 `typ` 的令牌一律拒绝。
- 吊销：logout / 改密 / 换绑手机号会 bump `token_version`，旧 access 与 refresh 立即失效。
- 角色：USER / AGENT / OPERATOR / ADMIN；运营端点由中间件校验 OPERATOR / ADMIN。
- 身份一律由令牌派生：客户端提交的 `user_id` / `reviewer_id` / `operator_user_id` 不作为身份依据。
  订单 / 报价 / 保单 / 合同详情按归属过滤；理赔与资料为本人或管理员。

## 8. 交易与支付

### 8.1 报价 / 订单

- 报价：产品必须 `ON_SALE`，保额与保障期必须落在产品区间内，且 `quote_rates` 必须有命中费率行；
  保费恒由服务端按 `保额 × 费率` 四舍五入到 2 位小数，客户端 `premium` 被忽略；报价 7 天过期。
- 订单：`discount_amount` 恒为 0，客户端优惠被忽略；报价必须 PENDING 且未过期，事务内锁定并
  一次性置为 `CONVERTED`；应付金额等于报价保费。

### 8.2 支付

- 预支付仅接受 `CREATED` / `EXPIRED` 订单；渠道白名单 WECHAT / ALIPAY / UNIONPAY / BALANCE / MOCK，
  缺省 MOCK；本阶段 provider 恒为 MOCK，所有渠道回调都走 `/api/v1/payments/callback/mock`。
- 回调要求 `PAYMENT_CALLBACK_TOKEN` 非空且请求头 `X-Callback-Token` 常量时间匹配；token 为空时回调全部拒绝。
  回调必须显式声明成功、provider 路径与支付单一致、交易号必填且按 provider 唯一。
- 成功回调在同一事务内提交支付 `SUCCESS` + 订单 `PAID`；已支付订单的第二次回调整笔回滚并返回 `40900`，
  其支付单保持 `CREATED`。失败回调从不写 `paid_at`。
- 重复预支付产生多笔支付单时，竞争回调的败者保持 `CREATED`；当前没有自动退款或对账路径。

### 8.3 保单

- 签发：`POST /api/v1/policies { order_id }` 要求订单已 `PAID`，事务内锁定订单；幂等（重试返回同一保单）。
- 退保：`POST /api/v1/policies/{id}/lapse`，ACTIVE → SURRENDERED，写 `POLICY_LAPSE` 审计。
- 续保暂未开放：需重新走报价 → 订单 → 支付流程；`is_renewable` 列保留。
- 受益人批改：`POST /api/v1/policies/{id}/beneficiaries` 整单替换，写 `POLICY_ENDORSE` 审计。

### 8.4 合同

- `contract_service.rs` 内联 Mock 签署 / 签署 URL / 回调；合同落库。
- 外部渠道（易签等）真实接入仍待办。

### 8.5 理赔

- 报案：要求保单 ACTIVE；事故日期（缺省今天）落在生效日..失效日之间；
  `0 < claim_amount <= insurance_amount`。
- 审核（OPERATOR / ADMIN）：`0 < approved <= claim_amount <= insurance_amount`；
  UPDATE 以 `status = 'SUBMITTED'` 为条件，重复审核返回 `40900`。

## 9. 搜索

- `search_service.rs` 使用 MySQL `LIKE`，仅检索 `status = 'ON_SALE' AND search_enabled = 1` 的商品。
- 关键字中的 LIKE 元字符 `\`、`%`、`_` 被转义，SQL 显式 `ESCAPE '\'`；参数全部绑定。
- 分页使用 `COUNT(*) OVER()`，空页时回退一次 `COUNT(*)`；查询参数经 `serde_urlencoded` 解码（支持百分号编码）。
- `search/searchable_impl.rs` 仅提供搜索序列化辅助；开源版无 OpenSearch / rust_scout。

## 10. 配置

环境变量（模板见 `.env.example`）：

| 变量 | 说明 |
|------|------|
| `SERVER_HOST` / `SERVER_PORT` | 监听地址 / 端口 |
| `DATABASE_URL` | MySQL 连接串（必填） |
| `JWT_SECRET` / `JWT_ISSUER` | JWT 签名密钥（必填，≥ 32 字节随机值）/ 签发者 |
| `JWT_ACCESS_EXPIRY` / `JWT_REFRESH_EXPIRY` | access / refresh 有效秒数（默认 7200 / 604800） |
| `CRYPTO_MASTER_KEY` | AES-256-GCM 主密钥（必填） |
| `WECHAT_APPID` / `WECHAT_SECRET` | 微信 code2session 凭据（未配置时微信登录降级报错） |
| `PAYMENT_CALLBACK_TOKEN` | 支付回调令牌（请求头 `X-Callback-Token`）；留空禁用回调 |
| `IDGEN_WORKER_ID` | snowflake worker id（0-63） |
| `RUST_LOG` | 日志级别 |

`config/app.toml` 仅保留 server / database / jwt / crypto / payment / log 配置段。

## 11. 部署与测试

```bash
cargo build --release
DATABASE_URL=... JWT_SECRET=... CRYPTO_MASTER_KEY=... ./target/release/insurance-service
```

- 依赖：MySQL 8.4；`bee-rust` 1.0.6 随仓库 vendored（`vendor/bee-rust`，快照 `93dd5c57`），无需 GitHub git。
- 健康检查：`GET /healthz`。
- 测试：`cargo test` 共 50 项（44 单元 + 6 集成）。DB 用例由 `TEST_DATABASE_URL` 门控，
  未设置时直接返回；带库运行需先执行 `install.sql`：
  `TEST_DATABASE_URL=mysql://... cargo test`。
- `cargo check --all-targets` 除外部 `proc-macro-error2` 的 future-incompat 提示外干净；
  仍有 4 个既有 clippy `result_large_err` 警告。
- 反向代理部署时，限流看到的都是代理 IP，需在代理层另做按客户端限流。

## 12. Roadmap

- 保单 / 合同 PDF 生成、续保流程。
- 微信支付 / 支付宝 / 易签等外部渠道真实接入。
- 多实例共享限流 / 缓存、对账与退款。
- Flutter / 小程序 / 鸿蒙端联调。
