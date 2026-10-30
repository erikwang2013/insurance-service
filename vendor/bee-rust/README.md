<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Beerust

[简体中文](README.md) · [English](docs/README.en.md) · [한국어](docs/README.ko.md) · [Русский](docs/README.ru.md) · [Deutsch](docs/README.de.md) · [Français](docs/README.fr.md) · [Español](docs/README.es.md) · [Português](docs/README.pt.md) · [हिन्दी](docs/README.hi.md) · [العربية](docs/README.ar.md) · [বাংলা](docs/README.bn.md) · [Bahasa Indonesia](docs/README.id.md) · [日本語](docs/README.ja.md)

Beerust是一个 Rust 语言的生产级 Web 框架，设计哲学源自 Go 的 Beego框架，用 Rust 惯用的 trait、macro、类型系统重新表达。

## 项目吉祥物：Rusty

<img src="docs/rusty.svg" width="360" alt="Rusty 正面悬停，翅膀展开成环，前足抱着锈橙工具盒">

### 角色卡

**名字** Rusty（锈锈）——Rust 的锈色，也是它工装裤上蹭的那点铁锈。

**形象**：一只毛茸茸的工程师蜜蜂。琥珀金的绒毛覆满圆滚滚的腹部，三道炭黑条纹像系歪了的工装背带；四片透明的翅膀薄得能透光，悬停时嗡出残影；复眼是两颗玻璃珠似的黑曜石，各有一粒高光；前足抱着一只磨得发亮的锈橙色工具盒——那是它的花粉篮，里面从来不装花粉，装的是依赖。

**性格**：工作狂，乐天派，轻度强迫症。

- 修蜂巢时用触角逐格检查，像跑 lint 一样较真
- 花粉篮里的依赖锁版本锁得死死的，谁动它跟谁急
- 追 bug 时会悬停在半空不动，翅膀快得只剩一圈虚影
- 发布新版本那天，会在蜂巢门口跳蜜蜂的 8 字摇摆舞

### 视觉规范

| 元素 | 规范 |
|------|------|
| 主体 | 琥珀金 `#F5B301`，腹部绒毛短而密，边缘一圈高光 |
| 条纹 | 炭黑 `#1F1A17`，三道，只在腹部 |
| 翅膀 | 透明浅蓝 `#CFE8F5`，40% 不透明度，画得出脉络 |
| 复眼 | 深棕黑 `#2A1E16`，椭圆，左上角一粒高光 |
| 点缀 | 锈橙 `#B7410E`（工具盒、领结），铁锈色是它的签名 |

### 表情集

| 表情 | 画面 | 什么时候 |
|------|------|---------|
| 元气满满 | `(^o^)` 绒毛炸起，翅膀高频振动 | 测试全绿的时候 |
| 悬停调试 | `o(°▽°)o` 悬在半空，只剩翅膀残影 | 追一个难缠的 bug |
| 满载而归 | `(≧▽≦)` 工具盒鼓鼓的，飞得歪歪扭扭 | 新驱动接入成功 |
| 打瞌睡 | `(-_-)zZ` 蹲在蜂巢门口，触角耷拉 | 深夜没有流量 |
| 炸毛 | `(#°益°)` 尾针竖起来 | 安全过滤器拦到攻击 |

### 招牌动作

1. **悬停调试** —— 发现 bug 就悬停在代码上方，翅膀嗡成虚影，盯着看，直到你改完。
2. **8 字摇摆舞** —— 每次发布新版本，绕着蜂巢跳一整圈，向全窝宣告"花开好了"。真蜜蜂靠这个传递花源位置，Rusty 靠这个传递 changelog。
3. **梳毛** —— 前足反复擦触角。你以为它在爱美，其实它是在检查配置有没有热更新。

### 它会出现在哪

- **Logo**：正面悬停姿势，翅膀展开成圆环，正好圈住名字。
- **404 页**：悬停在一片空花田上，迷茫地转圈——"蜜呢？"
- **CLI**：启动横幅上的一行小蜜蜂，跑测试时蹲在旁边监工。
- **发布公告**：跳 8 字舞的那一张。

## 设计目标

| 目标 | 指标 |
|------|------|
| **开发体验** | 从 `bee-rust new` 到首请求 < 30 秒 |
| **性能** | 控制器层开销 < 5%（对比裸 axum），P99 路由延迟 < 100µs |
| **编译速度** | 元 crate 全量编译 < 60s（release），增量编译 < 5s |
| **二进制体积** | 最小应用（仅 router）< 5MB（strip + LTO） |
| **安全性** | 0 unsafe 业务代码；所有 FFI 封装在独立 `*-sys` crate |
| **兼容性** | Rust 1.80+ MSRV，跟踪 stable |

## 设计原则

1. **Beego 哲学，Rust 表达** — MVC、命名空间、过滤器链，用 trait + macro 实现
2. **渐进增强** — 最小核心只依赖 axum + tokio，其他全部 feature gate
3. **显式优于隐式** — 路由注册、模型映射、中间件顺序全部代码显式声明
4. **零成本抽象** — trait 静态分发，macro 编译期展开，不引入虚函数开销
5. **存储引擎独立性** — 每个引擎 trait 可单独替换实现，不影响上层业务
6. **观测性内置** — tracing + metrics 埋点全框架覆盖，结构化日志默认开启

## 架构设计

### Crate 拓扑

```
bee_rust/           # 元 crate，re-export + feature flags
bee_router/         # 路由 + 控制器 + Context + 过滤器链
bee_orm/            # ORM — Model trait + QuerySet + Migration + 关系映射
bee_kv/             # KV/Cache 统一抽象 — Redis + Memcached
bee_search/         # 搜索/分析引擎 — Elasticsearch + OpenSearch + ClickHouse
bee_graph/          # 图数据库 — Neo4j + NebulaGraph + ArangoDB
bee_tsdb/           # 时序数据库 — InfluxDB + Apache IoTDB + QuestDB
bee_config/         # 配置管理 — INI/YAML/ENV + 热更新
bee_cache/          # 缓存抽象 — Memory/Redis/Memcache
bee_session/        # Session — Memory/Redis/Cookie/Database 后端
bee_logs/           # 日志 — 多级日志 + tracing 集成
bee_template/       # 模板渲染 — 基于 tera
bee_cli/            # CLI — 脚手架/代码生成/开发运行/打包（迁移规划中）
```

### 架构图

```
                          ┌───────────────────────┐
                          │  bee_rust  (meta)      │
                          │  re-export + features  │
                          └───────────┬───────────┘
                                      │
              ┌───────────────────────┼───────────────────────┐
              │                       │                       │
     ┌────────▼────────┐    ┌────────▼────────┐    ┌────────▼────────┐
     │    Web Layer     │    │   Data Layer    │    │   Tool Layer    │
     └────────┬────────┘    └────────┬────────┘    └────────┬────────┘
              │                       │                       │
  ┌───────────┼───────────┐  ┌───────┼───────┐  ┌───────────┼───────────┐
  │ bee_router            │  │ bee_orm        │  │ bee_cli               │
  │  - route register     │  │  - Model/Query  │  │  - scaffolding        │
  │  - controller trait   │  │  - Migration   │  │  - hot reload          │
  │  - filter chain       │  │  - Connection   │  │  - code generation    │
  │  - param extract      │  │                 │  │                       │
  ├────────────────────────┤  ├────────────────┤  ├───────────────────────┤
  │ bee_template           │  │ bee_config     │  │ bee_logs              │
  │  - template render     │  │  - INI/YAML/ENV│  │  - multi-level log    │
  │  - HTML/JSON           │  │  - hot reload   │  │  - tracing integrate  │
  ├────────────────────────┤  ├────────────────┤  └───────────────────────┘
  │ bee_session            │  │ bee_cache      │
  │  - session management  │  │  - cache trait  │
  │  - multi-backend       │  │  - Mem/Redis    │
  └────────────────────────┘  └────────────────┘

  ┌─────────────────────────────────────────────────────────┐
  │                   Storage Engine Layer                   │
  ├──────────────────┬──────────────────────────────────────┤
  │ bee_kv           │  Redis + Memcached                   │
  │ bee_search       │  Elasticsearch + OpenSearch + ClickHouse │
  │ bee_graph        │  Neo4j + NebulaGraph + ArangoDB      │
  │ bee_tsdb         │  InfluxDB + Apache IoTDB + QuestDB   │
  └──────────────────┴──────────────────────────────────────┘
```

### Crate 依赖关系

```
bee_config (无依赖)
bee_logs   (无依赖)
bee_cache  → bee_config
bee_kv     → bee_config
bee_session → bee_cache, bee_config
bee_template (无依赖)
bee_orm    → bee_config, bee_cache
bee_search → bee_config
bee_graph  → bee_config
bee_tsdb   → bee_config
bee_router → bee_session, bee_template, bee_config, bee_logs
bee_cli    → bee_router, bee_orm
bee_rust   → 全部上述 crate (re-export)
```

### 支持的数据库

| 类别 | 数据库 | 对应 Crate | Feature Flag |
|------|--------|-----------|-------------|
| **关系型** | SQLite | `bee_orm` | `sqlite` |
| | PostgreSQL | `bee_orm` | `postgres` |
| | MySQL | `bee_orm` | `mysql` |
| | TiDB | `bee_orm` | `mysql` |
| **KV / 缓存** | Redis | `bee_kv` / `bee_cache` | `redis` |
| | Memcached | `bee_kv` / `bee_cache` | `memcache` |
| **搜索 / 分析** | Elasticsearch | `bee_search` | `elasticsearch` |
| | OpenSearch | `bee_search` | `opensearch` |
| | ClickHouse | `bee_search` | `clickhouse` |
| **图数据库** | Neo4j | `bee_graph` | `neo4j` |
| | NebulaGraph | `bee_graph` | `nebulagraph` |
| | ArangoDB | `bee_graph` | `arangodb` |
| **时序数据库** | InfluxDB | `bee_tsdb` | `influxdb` |
| | Apache IoTDB | `bee_tsdb` | `iotdb` |
| | QuestDB | `bee_tsdb` | `questdb` |

### 请求过滤器链

```
请求 → [SecurityFilter 攻击检测] → [Session 恢复] → [参数验证] → [prepare 钩子] → [handle 处理] → [finish 钩子] → 响应
                  ↓ 任何环节可中断（类似 Beego 的 Abort）
```

## 功能介绍

功能概览见下，各模块的详细用法、代码示例与 API 说明见 [API 参考](docs/api.md)。

### Web 核心（bee_router）

MVC 控制器：`Controller` trait + `Context`，支持路由命名空间、RESTful 方法注册、请求过滤器链。

- 响应输出：`ctx.json()` / `ctx.text()` / `ctx.html()`
- 重定向 / 中断：`ctx.redirect()` / `ctx.abort()`
- 会话与参数：`ctx.session` / `ctx.params`

### 安全检测（`security` feature）

基于 [security-rust](https://crates.io/crates/security-rust) 的攻击检测过滤器，覆盖 XSS、SQL 注入、命令注入、SSRF 等 27 种攻击类型，一行开启：

```rust
let security = SecurityFilter::new();  // 27 个检测器全开
```

### ORM（bee_orm）

`#[derive(Model)]` 派生宏 + QuerySet 链式查询（filter / order_by / limit），支持 SQLite、PostgreSQL、MySQL、TiDB。

### 配置管理（bee_config）

`#[derive(Config)]` 派生宏，支持 INI / YAML / ENV 加载与热更新。

### 存储引擎

KV / Cache（Redis + Memcached）、搜索引擎（Elasticsearch / OpenSearch / ClickHouse）、图数据库（Neo4j / NebulaGraph / ArangoDB）、时序数据库（InfluxDB / IoTDB / QuestDB）统一 trait 抽象，驱动按 feature gate 编译。

### Session、日志、模板

- Session：Memory / Redis / Cookie / Database 多后端
- 日志：多级日志 + tracing 集成
- 模板：基于 tera 渲染

### CLI 工具

```bash
bee-rust new my-app            # 创建项目脚手架
bee-rust generate controller user
bee-rust run --watch           # 开发运行（热重载）
bee-rust pack                  # 打包部署
```

## 使用步骤

### 环境要求

- Rust 1.80+
- Cargo

### 安装

```bash
# 克隆项目
git clone https://github.com/erikwang2013/bee-rust.git
cd bee-rust

# 编译
cargo build --workspace

# 运行测试
cargo test --workspace
```

### 快速开始

```bash
# 使用 CLI 创建新项目
cargo run -p bee_cli -- new hello
cd hello

# 运行开发服务器
cargo run
```

### 在项目中使用

```toml
[dependencies]
bee_rust = { git = "https://github.com/erikwang2013/bee-rust", features = ["full"] }
```

## 技术说明

### 技术栈

| 层 | 技术 |
|----|------|
| HTTP 基座 | axum 0.8 + tower 0.5 |
| 异步运行时 | tokio 1.x |
| 序列化 | serde + serde_json |
| 模板引擎 | tera 1.x |
| 日志底层 | tracing + tracing-subscriber |
| CLI | clap 4 |
| 配置解析 | toml / serde_yaml / 自研 INI |
| 错误处理 | thiserror |
| 过程宏 | syn + quote + proc-macro2 |

### 设计模式

| 模式 | 应用 |
|------|------|
| **Builder** | Logger, Router, QuerySet |
| **Trait 抽象** | Cache, KvStore, SearchEngine, GraphDB, TimeSeriesDB |
| **派生宏** | `#[derive(Model)]`, `#[derive(Config)]` |
| **Feature Gate** | 驱动实现按需编译（redis, memcached, elasticsearch 等） |
| **Filter Chain** | 请求过滤器链，对标 Beego Filter |

### Crate 清单

| Crate | 功能 | 对标 Beego |
|-------|------|-----------|
| `bee_rust` | 元 crate，统一入口 | — |
| `bee_router` | 路由 + 控制器 + Context + 过滤器 | `server/web`, `context` |
| `bee_orm` | ORM + QuerySet + Migration | `client/orm` |
| `bee_kv` | KV/Cache 统一抽象 | `client/cache`（扩展） |
| `bee_search` | 搜索/分析引擎 | —（新增） |
| `bee_graph` | 图数据库 | —（新增） |
| `bee_tsdb` | 时序数据库 | —（新增） |
| `bee_config` | 配置管理 + 热更新 | `client/config` |
| `bee_cache` | 缓存抽象 | `client/cache` |
| `bee_session` | Session 管理 | `server/web/session` |
| `bee_logs` | 日志 | `logs` |
| `bee_template` | 模板渲染 | —（增强） |
| `bee_cli` | CLI 工具 | `bee` 工具 |

### 测试覆盖

全仓 68 个测试通过：

| Crate | 测试数 |
|-------|--------|
| bee_config | 4 |
| bee_cache | 4 |
| bee_template | 2 |
| bee_logs | 3 |
| bee_kv | 4 |
| bee_search | 6 |
| bee_graph | 5 |
| bee_tsdb | 5 |
| bee_orm | 7 |
| bee_session | 2 |
| bee_router | 9 |
| bee_cli | 16 |

## 欢迎支持

如果这个项目对你有帮助，欢迎扫描二维码打赏支持，谢谢！

**微信支付**

<img src="docs/weixinpay.png" width="160" height="175" alt="微信支付">

**支付宝**

<img src="docs/alipay.png" width="160" height="175" alt="支付宝">

**全球转账（Bank Transfer）**

海外用户可通过银行转账支持：

**收款人信息**

| 项目 | 内容 |
|------|------|
| 收款人姓名 | WANG KEXUN |
| 收款账户号码 | 881015918251 |

**收款银行**

| 项目 | 内容 |
|------|------|
| SWIFT Code | AABLHKHHXXX |
| 银行名称 | ZA Bank Limited |
| 银行编号 | 387 |
| 银行地址 | Core F, Cyberport 3, 100 Cyberport Road, Hong Kong |

**跨境汇款代理银行（如需）**

> 注意：此为跨境汇款代理银行（中转银行）信息，非收款银行信息。请向你的汇款银行查询是否需要提供。

- **汇入港元、人民币及美元**（代理银行为 Citibank）：

| 项目 | 内容 |
|------|------|
| 银行名称 | Citibank N.A. Hong Kong |
| SWIFT Code | CITIHKHXXXX |
| 银行编号 | 006 |
| 分行名称 | Hong Kong Branch |
| 分行编号 | 391 |
| 银行地址 | Citibank Tower, Citibank Plaza, 3 Garden Road, Central, Hong Kong |

- **汇入其他币种**（代理银行为 BNY Mellon）：

| 项目 | 内容 |
|------|------|
| 银行名称 | THE BANK OF NEW YORK MELLON |
| SWIFT Code | IRVTUS3NXXX |
| 银行地址 | THE BANK OF NEW YORK MELLON, 240 GREENWICH STREET, NEW YORK, United States |

### 许可证

Apache-2.0
