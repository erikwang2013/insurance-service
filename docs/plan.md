# 保险服务平台 — 规划文档（本地备份）

> 本文档为本地规划快照：与 [`backend-architecture.md`](./backend-architecture.md) §13 Roadmap、[db-schema.md](./db-schema.md)、[README.md](../README.md) 保持一致。
> 更新时请同步维护三处。版本: 2026-09-26（v1.9.0）。

---

## 总目标

Rust（`bee_rust` + `axum`）实现的保险服务平台后端，覆盖
「商品展示 → 报价 → 订单 → 支付 → 保单 → 电子签约 → 理赔」完整闭环，
面向 Flutter App / 微信小程序 / HarmonyOS App / 运营管理后台四端共用同一 REST API。

## 当前状态

- **阶段 0 完成 · 阶段 1 基本完成（v1.8.0）**：用户认证（含微信绑定 / 令牌吊销）、保险商品、全文搜索（LIKE 降级）、
  报价（`quote_rates` 费率表驱动）、理赔（报案 / 审核 / 资料）、运营后台与审计查询均已落地；
  **报价 → 订单 → 支付 → 保单 → 签约**交易闭环 API 已打通（支付 / 电子签为 Mock 渠道）。
  `src/routes.rs` 已挂载 39 个业务端点 + `/`、`/healthz`、`/favicon.svg` 根路径。
- **待办**：OpenSearch 真实接入、保单 PDF、微信支付 / e签宝真实渠道、Flutter / 小程序 / 鸿蒙端联调。
- 详细路线见下，逐项勾选进度同步自 `docs/tasks.md`（逐项实现状态以 `backend-architecture.md` §13 为准）。

## 分阶段路线（权威来源：backend-architecture.md §13）

### 阶段 0 — 后端骨架（✅ 已完成；逐项状态见 backend-architecture.md §13）
- 骨架激活：bee_rust 管线 + adopt 到真实 HTTP 服务器（axum）
- 配置 / 数据库 / 缓存 / 搜索接入
- SecurityFilter + JWT 认证 + RBAC + 统一 ResponseEnvelope
- 基础控制器（auth / product / search / healthz）
- CryptoService（AES-256-GCM + 脱敏）、SyncWorker 骨架

### 阶段 1 — 核心交易闭环（🔄 基本完成；保险单 PDF 待补）
- bee_orm MySQL 持久化（install.sql ↔ Rust models 对齐）
- 产品管理 + 分类 + 条款（含搜索索引同步）
- 报价（QuoteService + 保费计算）、订单 + 支付（PayProvider 抽象 + Mock，回调幂等）
- 保单生成、电子合同 + 签署（ElectronicSignature 抽象 + Mock）
- 理赔基础流程

### 阶段 2 — Flutter 主端对接
- 三端共用 REST API 联调（auth → 报价 → 订单 → 支付 → 保单 → 签署）
- 产品搜索 + 首页推荐

### 阶段 3 — 小程序 + 鸿蒙（微信登录 ✅ v1.6.0；微信支付真实适配待办）
- 微信登录 + 微信支付真实适配；鸿蒙 ArkTS 端联调

### 阶段 4 — 真实渠道与加固
- 微信支付（WechatPayProvider）、e签宝 / 法大大真实对接
- 短信、对象存储（保单/合同 PDF）、监控告警
- 等保合规、敏感数据密钥轮换（KMS）

---

## 里程碑判定

| 里程碑 | 判定标准 |
|--------|----------|
| 阶段 0 完成 ✅ | `cargo test` 全绿（v1.8.0：137 项 = 单元 22 + 集成 115，MySQL 缺失自动 SKIP）；`/healthz` 与 auth/product/search 路由可运行 |
| 阶段 1 完成 🔄 | 订单 → 支付（Mock 回调验签）→ 保单闭环走通 ✅；库表与 models 一致 ✅；待补保单 PDF |
| 阶段 2 完成 | Flutter 端完整业务链路联调通过（未开始） |