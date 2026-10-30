# 保险服务平台 — 任务文档

> 版本: 2026-10-30（v1.11.0）。已完成任务保留状态，不删除。

---

## 当前任务

| 任务 | 状态 |
|------|------|
| bee_rust + axum 服务骨架（`main.rs` → `axum::serve`） | ✅ 已完成 |
| 控制器注册（`build_bee_router()`，41 个业务端点 + `/healthz`） | ✅ 已完成 |
| MySQL 持久化（连接池 / `with_tx` / install.sql 14 表） | ✅ 已完成 |
| 认证（注册 / 登录 / 微信 code2session / 双令牌 / 令牌吊销） | ✅ 已完成 |
| 认证中间件（Bearer access JWT → RBAC；公共路由放行；运营端点 OPERATOR / ADMIN） | ✅ 已完成 |
| 身份取令牌 + 归属校验（订单 / 报价 / 保单 / 合同；理赔 / 资料本人或管理员） | ✅ 已完成 |
| 商品与条款（列表 / 详情 / 精选 / 条款；公开仅 ON_SALE） | ✅ 已完成 |
| 搜索（MySQL LIKE，ON_SALE + search_enabled，LIKE 转义，分页兜底） | ✅ 已完成 |
| 报价（`quote_rates` 费率行强制，服务端计费） | ✅ 已完成 |
| 订单（服务端金额，优惠恒 0，报价一次性转换） | ✅ 已完成 |
| 支付（预下单 / 回调令牌 / 事务幂等，Mock 渠道） | ✅ 已完成 |
| 保单（签发 / 退保 / 受益人批改） | ✅ 已完成 |
| 合同 Mock 签署闭环 | ✅ 已完成 |
| 理赔（报案 / 审核 / 资料上传） | ✅ 已完成 |
| 运营后台（商品列表 / 建档 / 上下架 / 审计查询） | ✅ 已完成 |
| 测试 50 项（44 单元 + 6 集成；4 个 DB 用例由 `TEST_DATABASE_URL` 门控） | ✅ 已完成 |
| 保单 / 合同 PDF | ⏳ 待办 |
| 续保流程 | ⏳ 待办 |
| 微信支付 / 支付宝 / 易签真实渠道 | ⏳ 待办 |
| Flutter / 小程序 / 鸿蒙端联调 | ⏳ 待办 |

## 完成判定

| 任务 | 完成标准 |
|------|----------|
| 服务骨架 | `cargo run` 后 `curl /healthz` 返回 200 统一信封 |
| 控制器 | auth / product / search 等 Controller 均被路由分发 |
| 持久化 | 注册重复用户名报 40000；登录正确/错误密码分流；写操作事务闭环 |
| 认证 | 受保护路由无 token 返回 HTTP 401 + 统一信封；运营端点非 OPERATOR / ADMIN 返回 403 |
| 搜索 | MySQL LIKE 可分页查询；公开结果仅 ON_SALE；无外部索引依赖 |
| 测试 | `cargo test` 50 项通过；DB 用例未设置 `TEST_DATABASE_URL` 时直接返回（不计失败） |

## 历史

- JWT 过期语义：`validation.leeway = 0`，令牌精确过期。
- JWT 类型隔离：access / refresh 独立 `typ`；refresh 用 `JWT_REFRESH_EXPIRY`，access 用
  `JWT_ACCESS_EXPIRY`；`JWT_SECRET` ≥ 32 字节，缺失或过短启动失败；无 `typ` 的旧令牌拒绝。
- 令牌吊销：logout / 改密 / 换绑手机号 bump `token_version`。
- JWT RBAC：角色 USER / AGENT / OPERATOR / ADMIN；由 axum 认证中间件守卫运营端点
  （已移除 `RequireRoleFilter` 抽象）。
- 身份由令牌派生：列表 / 详情不接受 `user_id` / `reviewer_id` / `operator_user_id` 作为身份参数。
- 支付回调：`X-Callback-Token` 与 `PAYMENT_CALLBACK_TOKEN` 常量时间比较；为空禁用回调；
  成功回调事务内完成支付 + 订单状态，重复回调回滚 `40900`。
- 保单：订单 PAID 后签发（幂等）；`POST /policies/{id}/lapse` 退保；续保暂未开放。
- 运营：`GET /admin/products` 支持状态过滤；商品建档 / 上下架 / 审计查询。
- 搜索加固：仅 `ON_SALE` + `search_enabled=1`；转义 `\ % _` 并 `ESCAPE '\'`；
  `COUNT(*) OVER()` + 空页 COUNT 兜底；查询参数百分号解码。
- 主键切换 snowflake（idgen_rs 无锁生成）。
- `bee-rust` 1.0.6 vendored 至 `vendor/bee-rust`（快照 `93dd5c57`），构建不依赖 GitHub git。
