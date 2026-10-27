# 保险服务平台 — 规划文档

> 版本: 2026-10-27（v1.10.0）。与 `backend-architecture.md`、`db-schema.md`、`README.md` 同步维护。

---

## 总目标

Rust（`bee_rust` + `axum`）实现的保险服务平台后端，覆盖
「商品展示 → 报价 → 订单 → 支付 → 保单 → 电子签约 → 理赔」完整闭环，
面向 Flutter App / 微信小程序 / HarmonyOS App / 运营管理后台共用同一 REST API。

## 当前状态

- 用户认证（含微信绑定 / 令牌吊销）、保险商品、MySQL LIKE 搜索、报价（`quote_rates` 费率表驱动）、
  理赔（报案 / 审核 / 资料）、运营后台与审计查询均已落地。
- **报价 → 订单 → 支付 → 保单 → 签约**交易闭环 API 已打通；支付与签约当前为内联 Mock 实现。
- `src/routes.rs` 的 `build_bee_router()` 挂载 38 个业务端点 + `/healthz`；未匹配路径统一返回 JSON 信封。
- 存储只依赖 MySQL 8.4；限流与会话使用进程内缓存。

## 待办

- 保单 / 合同 PDF 生成。
- 微信支付 / 支付宝 / 易签等外部渠道真实接入（替换内联 Mock）。
- Flutter / 小程序 / 鸿蒙端联调。

## 分阶段路线

### 阶段 0 — 后端骨架（✅ 已完成）
- bee_rust 管线 + axum HTTP 服务
- 配置 / MySQL / 内存限流
- SecurityFilter + JWT 认证 + RBAC + 统一 ResponseEnvelope
- 基础控制器（auth / product / search / healthz）
- CryptoService（AES-256-GCM + 脱敏）

### 阶段 1 — 核心交易闭环（🔄 基本完成）
- bee_orm MySQL 持久化（install.sql ↔ Rust models 对齐，14 表）
- 产品 + 条款
- 报价、订单、支付（内联 Mock，回调幂等）
- 保单生成、合同与签署（内联 Mock）
- 理赔基础流程

### 阶段 2 — Flutter 主端对接
- 三端共用 REST API 联调（auth → 报价 → 订单 → 支付 → 保单 → 签署）
- 产品搜索 + 首页推荐

### 阶段 3 — 小程序 + 鸿蒙
- 微信登录已接入；微信支付真实适配待办；鸿蒙 ArkTS 端联调

### 阶段 4 — 真实渠道与加固
- 微信支付 / 支付宝 / 易签 / 法大大真实对接
- 短信、对象存储（保单 / 合同 PDF）、监控告警
- 等保合规、敏感数据密钥轮换（KMS）

---

## 里程碑判定

| 里程碑 | 判定标准 |
|--------|----------|
| 阶段 0 完成 ✅ | `/healthz` 与 auth / product / search 路由可运行；`cargo test` 通过 |
| 阶段 1 完成 🔄 | 订单 → 支付（Mock 回调验签）→ 保单闭环走通；库表与 models 一致；待补保单 PDF |
| 阶段 2 完成 | Flutter 端完整业务链路联调通过（未开始） |
