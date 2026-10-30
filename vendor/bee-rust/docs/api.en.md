<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Beerust API Reference

[简体中文](api.md) · [한국어](api.ko.md) · [Русский](api.ru.md) · [Deutsch](api.de.md) · [Français](api.fr.md) · [Español](api.es.md) · [Português](api.pt.md) · [हिन्दी](api.hi.md) · [العربية](api.ar.md) · [বাংলা](api.bn.md) · [Bahasa Indonesia](api.id.md) · [日本語](api.ja.md)

Back to [README](README.en.md).

This document is the API reference for the Beerust framework: usage, code examples, and interface descriptions for each module.

## Web Core (bee_router)

### Defining a Controller

```rust
use bee_rust::prelude::*;

// Define a controller
struct UserController;

#[bee_router::async_trait]
impl Controller for UserController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        ctx.json(&serde_json::json!({"users": []}))
    }
}
```

### Route Registration

```rust
// Route registration
let router = Router::new()
    .ns("/api/v1", |ns| {
        ns.get("/users")
          .post("/users");
    });
```

### Context API

**Context provides:**
- `ctx.json()` / `ctx.text()` / `ctx.html()` — response output
- `ctx.redirect()` — redirection
- `ctx.abort()` — request interruption
- `ctx.session` — session access
- `ctx.params` — path parameters

## Security Detection (`security` feature)

Attack detection powered by [security-rust](https://crates.io/crates/security-rust), covering XSS, SQL injection, command injection, SSRF, and 23 other attack types via 27 detectors:

```rust
use bee_rust::prelude::*;

let security = SecurityFilter::new();  // all 27 detectors enabled
```

Enable in `Cargo.toml`:
```toml
bee_rust = { features = ["security"] }
```

## ORM (bee_orm)

```rust
#[derive(Model)]
#[bee(table = "users")]
struct User {
    id:   i64,
    name: String,
    age:  i32,
}

// Chainable queries
let users = User::query()
    .filter("age > 18")
    .order_by("created_at DESC")
    .limit(20)
    .to_sql();
// → SELECT * FROM users WHERE age > 18 ORDER BY created_at DESC LIMIT 20
```

## Config (bee_config)

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

## Storage Engines

**KV/Cache:**
```rust
let kv = RedisStore::new("redis://localhost:6379").await?;
kv.set("key", b"value", Some(Duration::from_secs(60))).await?;
let val = kv.get("key").await?;
```

**Search Engine (planned):**
```rust
// Driver implementation planned; currently a trait stub
let engine = ElasticsearchEngine::new("http://localhost:9200")?;
let result = engine.search("my_index", &SearchQuery {
    q: Some("keyword".into()), ..Default::default()
}).await?;
```

**Graph Database (planned):**
```rust
// Driver implementation planned; currently a trait stub
let db = Neo4jDB::new("bolt://localhost:7687").await?;
let vid = db.add_vertex("Person", &[("name", "Alice")]).await?;
```

**Time Series Database (planned):**
```rust
// Driver implementation planned; currently a trait stub
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

## Logging

```rust
Logger::new()
    .level(Level::INFO)
    .output(Output::MultiFile("logs/"))
    .async_()
    .init()?;
```

## Templates

```rust
let engine = TemplateEngine::new("views/")?;
let result = engine.render("hello.html", &context! { name: &"World" })?;
// → "Hello, World!"
```

## CLI Tool

```bash
# Scaffold a runnable project (Cargo.toml + src/main.rs)
bee-rust new my-app

# Code generation
bee-rust generate controller user
bee-rust generate model user --fields "name:string,age:int"

# Dev server (--watch restarts on src/ changes)
bee-rust run
bee-rust run --watch

# Packaging (cargo build --release + copy to dist/)
bee-rust pack

# Database migrations (not implemented yet — planned)
bee-rust migrate up
```

> Note: `pack --target` is a reserved argument; packaging does not currently
> distinguish target platforms.
