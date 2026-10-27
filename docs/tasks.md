# 保险服务平台 — 任务文档

> 版本: 2026-10-27（v1.10.0）。已完成任务保留状态，不删除。

---

## 当前任务

| 任务 | 状态 |
|------|------|
| bee_rust + axum 服务骨架（`main.rs` → `axum::serve`） | ✅ 已完成 |
| 控制器注册（`build_bee_router()`，38 个业务端点 + `/healthz`） | ✅ 已完成 |
| MySQL 持久化（连接池 / `with_tx` / install.sql 14 表） | ✅ 已完成 |
| 认证（注册 / 登录 / 微信 code2session / 双令牌 / 令牌吊销） | ✅ 已完成 |
| 商品与条款（列表 / 详情 / 精选 / 条款） | ✅ 已完成 |
| 搜索（MySQL LIKE，分页保护） | ✅ 已完成 |
| 报价（`quote_rates` 费率表驱动） | ✅ 已完成 |
| 订单 → 支付 → 保单 → 合同 Mock 闭环 | ✅ 已完成 |
| 理赔（报案 / 审核 / 资料上传） | ✅ 已完成 |
| 运营后台（商品建档 / 上下架 / 审计查询） | ✅ 已完成 |
| 保单 / 合同 PDF | ⏳ 待办 |
| 微信支付 / 支付宝 / 易签真实渠道 | ⏳ 待办 |
| Flutter / 小程序 / 鸿蒙端联调 | ⏳ 待办 |

## 完成判定

| 任务 | 完成标准 |
|------|----------|
| 服务骨架 | `cargo run` 后 `curl /healthz` 返回 200 统一信封 |
| 控制器 | auth / product / search 等 Controller 均被路由分发 |
| 持久化 | 注册重复用户名报 40000；登录正确/错误密码分流；写操作事务闭环 |
| 搜索 | MySQL LIKE 可分页查询；无外部索引依赖 |
| 测试 | `cargo test` 通过；未配置 `DATABASE_URL` 时集成测试按项目约定处理 |

## 历史

- JWT 过期语义：`validation.leeway = 0`，令牌精确过期。
- JWT RBAC：角色 USER / AGENT / OPERATOR / ADMIN，`RequireRoleFilter` 守卫。
- 主键切换 snowflake（idgen_rs 无锁生成）。
