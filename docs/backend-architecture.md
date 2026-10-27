# 保险服务平台 — 后端架构

> 版本: 2026-10-27（v1.10.0）。本文档描述当前实现；历史规划中已移除的模块不再记录。

---

## 1. 总体架构

```
客户端（Flutter / 小程序 / HarmonyOS / 运营后台）
        │  HTTP / JSON
        ▼
axum + bee_router（/healthz + /api/v1/*）
        │
SecurityFilter → Trace → JWT 认证（RBAC）→ RateLimit → Controller
        │
Service（事务边界）→ Db（mysql_async 连接池）→ MySQL 8.4
        │
统一 ResponseEnvelope { code, message, data }
```

- 服务只提供 JSON 接口；`/api/v1/*` 与未匹配路径均返回统一 JSON 信封，不提供 HTML 页面。
- 限流与会话缓存使用进程内实现。
- 支付与电子签约当前为内联 Mock 实现；微信 / 支付宝 / 易签等外部渠道未接入。

## 2. 目录结构

```
insurance-service/
├── Cargo.toml
├── install.sql                 # MySQL 建库（14 表）
├── config/
│   ├── app.toml                # server / database / jwt / crypto / log
│   └── bee.toml
├── docs/
├── src/
│   ├── main.rs                 # 启动入口
│   ├── lib.rs
│   ├── config.rs               # AppConfig（环境变量 + app.toml）
│   ├── routes.rs               # build_bee_router()，38 业务端点 + /healthz
│   ├── db.rs                   # 连接池 / 查询 / with_tx
│   ├── error.rs                # AppError
│   ├── response.rs             # ResponseEnvelope
│   ├── controllers/            # auth / product / search / quote / order /
│   │                           # payment / policy / contract / claim / admin
│   ├── middleware/             # security / trace / auth / rate_limit
│   ├── services/               # 各业务 Service（事务边界）
│   ├── models/                 # bee_orm Model
│   ├── providers/wechat.rs     # code2session
│   ├── search/                 # Searchable 序列化辅助
│   ├── crypto/                 # AES-256-GCM / Masker / argon2
│   └── utils/                  # validator / idgen（snowflake）
└── tests/
```

## 3. 依赖

- `axum` 0.8、`tokio`、`bee_rust`（bee_router / bee_orm / 过滤器）
- `mysql_async`、`serde` / `serde_json`
- `jsonwebtoken`、`argon2`、`aes-gcm`

## 4. 路由

`src/routes.rs::build_bee_router()` 注册 38 个业务端点（`/api/v1` 命名空间）+ `/healthz`。

| 模块 | 端点 |
|------|------|
| 认证 / 用户 | register / login / wechat/login / wechat/bind / refresh / logout / user/me / user/password / user/phone |
| 商品 | products / products/{id} / products/{id}/clauses / products/featured |
| 搜索 | GET /search |
| 报价 | POST /quotes / GET /quotes/{id} |
| 订单 | POST/GET /orders / GET /orders/{id} |
| 支付 | /payments/{order_id}/prepay /pay /payments/wechat/prepay /payments/callback/{provider}（Mock） |
| 保单 | GET /policies /policies/{id} / POST /policies/{id}/beneficiaries |
| 合同 | GET /contracts/{id} / POST /contracts/{id}/sign / GET /contracts/{id}/sign-url / POST /contracts/callback/{provider}（Mock） |
| 理赔 | POST/GET /claims / POST /claims/{id}/review / POST/GET /claims/{id}/documents |
| 运营 | POST /admin/products /admin/products/{id}/status / GET /admin/audit-logs |

未匹配路径统一返回 `40400` JSON 信封。

## 5. 中间件 / 过滤器链

```
SecurityFilter → Trace → JWT 认证（RBAC）→ RateLimit → Controller
```

- **SecurityFilter**：入站安全扫描（SQL 注入 / XSS / 路径遍历等）。
- **Trace**：生成并透传 `trace_id`。
- **JWT**：`Authorization: Bearer`，`validation.leeway = 0`；`token_version` 支持吊销。
- **RateLimit**：进程内固定窗口，按 `X-Forwarded-For`（缺失回退全局桶）。
- **审计**：`audit_logs` 记录关键操作；查询端点 `/api/v1/admin/audit-logs`。

## 6. 响应与错误

- 成功：`{ "code": 0, "message": "ok", "data": ... }`
- 业务错误：`{ "code": 40000, "message": "...", "data": null }`
- 认证 / 权限：HTTP 401 / 403 + 统一信封。
- 未匹配：HTTP 404 + `40400`。

## 7. 认证与 RBAC

- 注册：用户名唯一 + argon2 口令哈希。
- 登录：账号密码；微信 `code2session` 绑定登录（凭据未配置时降级报错）。
- 双令牌：access 短时效、refresh 长时效；`token_version` 版本化，logout / 改密 / 换绑吊销旧令牌。
- 角色：USER / AGENT / OPERATOR / ADMIN；`RequireRoleFilter` 守卫运营端点。
- 对象级越权防护：保单 / 理赔 / 合同按当前用户归属校验。

## 8. 支付与合同

- 支付：`payment_service.rs` 内联 Mock 预下单 / 支付 / 回调；回调幂等由 `uk_payment_tx` 保证。
- 合同：`contract_service.rs` 内联 Mock 签署 / 签署 URL / 回调；合同落库。
- 外部渠道（微信支付 / 支付宝 / 易签等）真实接入仍待办。

## 9. 搜索

- `search_service.rs` 使用 MySQL `LIKE`，带分页保护。
- `search/searchable_impl.rs` 仅提供搜索序列化辅助。

## 10. 配置

环境变量（模板见 `.env.example`）：

| 变量 | 说明 |
|------|------|
| `SERVER_HOST` / `SERVER_PORT` | 监听地址 / 端口 |
| `DATABASE_URL` | MySQL 连接串（必填） |
| `JWT_SECRET` / `JWT_ISSUER` | JWT 签名密钥 / 签发者 |
| `JWT_ACCESS_EXPIRY` / `JWT_REFRESH_EXPIRY` | access / refresh 有效秒数 |
| `CRYPTO_MASTER_KEY` | AES-256-GCM 主密钥 |
| `IDGEN_WORKER_ID` | snowflake worker id（0-63） |
| `RUST_LOG` | 日志级别 |

`config/app.toml` 仅保留 server / database / jwt / crypto / log 配置段。

## 11. 部署

```bash
cargo build --release
DATABASE_URL=... JWT_SECRET=... CRYPTO_MASTER_KEY=... ./target/release/insurance-service
```

- 依赖：MySQL 8.4。
- 健康检查：`GET /healthz`。
- 集成测试需要可用 MySQL 且已执行 `install.sql`。

## 12. Roadmap

- 保单 / 合同 PDF 生成。
- 微信支付 / 支付宝 / 易签等外部渠道真实接入。
- Flutter / 小程序 / 鸿蒙端联调。
