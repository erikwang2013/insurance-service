# 保险服务平台 — 任务文档（本地备份）

> 同步自会话任务清单（TaskList）。更新任务时同步本表；**已完成任务保留历史行**（不删除），改状态为 ✅。
> 版本: 2026-09-02。

---

## 当前任务

### 阶段 0 — 后端骨架

| # | 任务 | 状态 |
|---|------|------|
| 1 | 激活 bee_rust 并接入真实服务器（bee 管线 → axum serve） | 🔄 进行中 |
| 2 | 启用 controllers 模块（注册到 bee 管线） | ⏳ 待办 |
| 6 | 添加 mysql_async 依赖 + db 模块（连接池 / with_tx） | 🔄 进行中 |

### 阶段 1 — 持久化与交易闭环交接

| # | 任务 | 状态 |
|---|------|------|
| 3 | bee_orm MySQL 持久化 + 交易闭环（register/login/order/payment 状态机） | ⏳ 待办 |
| 4 | 搜索服务 + 同步（OpenSearch 索引未就绪降级 MySQL LIKE） | ⏳ 待办 |
| 5 | 集成测试 tests/（auth / product / search / security） | ⏳ 待办 |

#### #3 分解（历史两批合并，以本表为准）

| # | 任务 | 状态 |
|---|------|------|
| 3a | 接入 AppState 与 controllers（AppState 接线 Db） | ⏳ 待办 |
| 3b | 实现 auth_service 持久化（注册 / 登录，unique 校验） | ⏳ 待办 |
| 3c | 实现 product_service 列表/详情查询（状态过滤 / 软删） | ⏳ 待办 |
| 3d | cargo check 验证 + 收尾（最终编译检查） | ⏳ 待办 |

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
- 集成测试 38 项：认证 6 / 商品 5 / 搜索 4 / 安全 18 / 单元 5。