# 保险服务平台 — 任务文档（本地备份）

> 同步自会话任务清单（TaskList）。更新任务时同步本表；**已完成任务保留历史行**（不删除），改状态为 ✅。
> 版本: 2026-09-26（v1.9.0）。

---

## 当前任务

### 阶段 0 — 后端骨架

| # | 任务 | 状态 |
|---|------|------|
| 1 | 激活 bee_rust 并接入真实服务器（bee 管线 → axum serve） | ✅ 已完成（`main.rs` → `axum::serve`） |
| 2 | 启用 controllers 模块（注册到 bee 管线） | ✅ 已完成（`routes.rs` 挂载 39 业务端点） |
| 6 | 添加 mysql_async 依赖 + db 模块（连接池 / with_tx） | ✅ 已完成（`src/db.rs`） |

### 阶段 1 — 持久化与交易闭环交接

| # | 任务 | 状态 |
|---|------|------|
| 3 | bee_orm MySQL 持久化 + 交易闭环（register/login/order/payment 状态机） | ✅ 已完成（v1.5.0 交易闭环测试通过） |
| 4 | 搜索服务 + 同步（OpenSearch 索引未就绪降级 MySQL LIKE） | ✅ 已完成（降级路径；OpenSearch 真实接入待办） |
| 5 | 集成测试 tests/（auth / product / search / security） | ✅ 已完成（集成 115 项） |

#### #3 分解（历史两批合并，以本表为准）

| # | 任务 | 状态 |
|---|------|------|
| 3a | 接入 AppState 与 controllers（AppState 接线 Db） | ✅ 已完成 |
| 3b | 实现 auth_service 持久化（注册 / 登录，unique 校验） | ✅ 已完成 |
| 3c | 实现 product_service 列表/详情查询（状态过滤 / 软删） | ✅ 已完成 |
| 3d | cargo check 验证 + 收尾（最终编译检查） | ✅ 已完成 |

### v1.5.0 → v1.8.0 追加任务（已完成，留档）

| 任务 | 版本 | 状态 |
|------|------|------|
| A1–A5 交易闭环扩展（报价 / 订单 / 支付回调 / 保单签发 / 理赔） | v1.5.0 | ✅ |
| B 微信 code2session 渠道接线（可配置骨架） | v1.5.0 | ✅ |
| 认证令牌吊销（token_version）+ 微信绑定闭环 | v1.6.0 | ✅ |
| 保单批改 / 费率表报价 / 理赔资料 / 审计查询（C1–C5） | v1.6.0 | ✅ |
| 全库主键切换 snowflake（idgen_rs 无锁生成） | v1.7.0 | ✅ |
| 吉祥物安安（SVG）+ 落地页 / 错误页 HTML 面 + 文档体系（架构 / 功能 / 生命周期图） | v1.8.0 | ✅ |
| 吉祥物换代：雪豹幼崽霜霜（尾尖雪花结晶 / 怀抱统一信封）替换守护熊猫安安，全站同名同源 | v1.9.0 | ✅ |

---

## 完成判定（承接 docs/plan.md）

| 任务 | 完成标准 |
|------|----------|
| #1 | `cargo run` 后 `curl /healthz` 返回 200 统一信封 |
| #2 | auth / product / search 三个 Controller 均被管线分发 |
| #3 | 注册重复用户名报 40000；登录正确/错误密码分流；写操作事务闭环 |
| #4 | OpenSearch 不可用时搜索仍可用（LIKE 降级 + 分页保护） |
| #5 | `cargo test` 全绿；未配置 DATABASE_URL 时集成测试打印 SKIP 不失败 |

## 历史（已完成，保留留档）

- JWT 过期语义：`jsonwebtoken` 默认 leeway 60s → 改为 `validation.leeway = 0`，令牌精确过期（已提交）。
- JWT RBAC：角色 USER / AGENT / OPERATOR / ADMIN，`RequireRoleFilter` 守卫。
- 集成测试 38 项（阶段 0 时点）：认证 6 / 商品 5 / 搜索 4 / 安全 18 / 单元 5。
- 测试规模现状（v1.8.0）：**137 项 = 单元 22 + 集成 115**，`cargo test` 全绿（无库环境集成测试自动 SKIP）。