<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Beerust API 参考

[English](api.en.md) · [한국어](api.ko.md) · [Русский](api.ru.md) · [Deutsch](api.de.md) · [Français](api.fr.md) · [Español](api.es.md) · [Português](api.pt.md) · [हिन्दी](api.hi.md) · [العربية](api.ar.md) · [বাংলা](api.bn.md) · [Bahasa Indonesia](api.id.md) · [日本語](api.ja.md)

返回 [README](../README.md)。

本文档是 Beerust 框架的 API 参考：各模块的用法、代码示例与接口说明。

## Web 核心（bee_router）

### 定义控制器

```rust
use bee_rust::prelude::*;

// 定义控制器
struct UserController;

#[bee_router::async_trait]
impl Controller for UserController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        ctx.json(&serde_json::json!({"users": []}))
    }
}
```

### 路由注册

```rust
// 路由注册
let router = Router::new()
    .ns("/api/v1", |ns| {
        ns.get("/users")
          .post("/users");
    });
```

### Context API

**Context 提供：**
- `ctx.json()` / `ctx.text()` / `ctx.html()` — 响应输出
- `ctx.redirect()` — 重定向
- `ctx.abort()` — 中断请求
- `ctx.session` — 会话访问
- `ctx.params` — 路径参数

## 安全检测（`security` feature）

基于 [security-rust](https://crates.io/crates/security-rust) 的攻击检测过滤器，覆盖 XSS、SQL 注入、命令注入、SSRF 等 27 种攻击类型：

```rust
use bee_rust::prelude::*;

let security = SecurityFilter::new();  // 27 个检测器全开
```

在 `Cargo.toml` 中启用：
```toml
bee_rust = { features = ["security"] }
```

## ORM（bee_orm）

```rust
#[derive(Model)]
#[bee(table = "users")]
struct User {
    id:   i64,
    name: String,
    age:  i32,
}

// 链式查询
let users = User::query()
    .filter("age > 18")
    .order_by("created_at DESC")
    .limit(20)
    .to_sql();
// → SELECT * FROM users WHERE age > 18 ORDER BY created_at DESC LIMIT 20
```

## 配置管理（bee_config）

```rust
#[derive(Config)]
#[config(file = "conf/app.conf")]
struct AppConfig {
    app_name: String,
    http_port: u16,
    run_mode: String,
}

let cfg = AppConfig::load("conf/app.conf")?;
```

## 存储引擎

**KV/Cache：**
```rust
let kv = RedisStore::new("redis://localhost:6379").await?;
kv.set("key", b"value", Some(Duration::from_secs(60))).await?;
let val = kv.get("key").await?;
```

**搜索引擎（计划中）：**
```rust
// 驱动实现计划中，当前为 trait stub
let engine = ElasticsearchEngine::new("http://localhost:9200")?;
let result = engine.search("my_index", &SearchQuery {
    q: Some("keyword".into()), ..Default::default()
}).await?;
```

**图数据库（计划中）：**
```rust
// 驱动实现计划中，当前为 trait stub
let db = Neo4jDB::new("bolt://localhost:7687").await?;
let vid = db.add_vertex("Person", &[("name", "Alice")]).await?;
```

**时序数据库（计划中）：**
```rust
// 驱动实现计划中，当前为 trait stub
let tsdb = InfluxDB::new("http://localhost:8086").await?;
tsdb.write_point("cpu", &[("host", "srv1")], &[("value", 0.85)], Utc::now()).await?;
```

## Session

```rust
let cache = Arc::new(MemoryCache::new());
let mut session = Session::new(cache, Duration::from_secs(3600));
session.set("user_id", &"123")?;
let uid: String = session.get("user_id")?.unwrap();
```

## 日志

```rust
Logger::new()
    .level(Level::INFO)
    .output(Output::MultiFile("logs/"))
    .async_()
    .init()?;
```

## 模板

```rust
let engine = TemplateEngine::new("views/")?;
let result = engine.render("hello.html", &context! { name: &"World" })?;
// → "Hello, World!"
```

## CLI 工具

```bash
# 创建项目（生成可运行的脚手架：Cargo.toml + src/main.rs）
bee-rust new my-app

# 生成代码
bee-rust generate controller user
bee-rust generate model user --fields "name:string,age:int"

# 开发运行（--watch 监听 src/ 变化自动重启）
bee-rust run
bee-rust run --watch

# 打包部署（cargo build --release + 复制到 dist/）
bee-rust pack

# 数据库迁移（未实现，规划中）
bee-rust migrate up
```

> 说明：`pack --target` 参数为预留接口，当前打包流程不区分目标平台。
