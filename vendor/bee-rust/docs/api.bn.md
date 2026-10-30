<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Beerust API রেফারেন্স

[English](api.en.md) · [한국어](api.ko.md) · [Русский](api.ru.md) · [Deutsch](api.de.md) · [Français](api.fr.md) · [Español](api.es.md) · [Português](api.pt.md) · [हिन्दी](api.hi.md) · [العربية](api.ar.md) · [বাংলা](api.bn.md) · [Bahasa Indonesia](api.id.md) · [日本語](api.ja.md)

ফিরে যান [README](README.bn.md)।

এই ডকুমেন্টটি Beerust ফ্রেমওয়ার্কের API রেফারেন্স: প্রতিটি মডিউলের ব্যবহার, কোড উদাহরণ ও ইন্টারফেসের ব্যাখ্যা।

## ওয়েব কোর (bee_router)

### কন্ট্রোলার সংজ্ঞায়িত করা

```rust
use bee_rust::prelude::*;

// কন্ট্রোলার সংজ্ঞায়িত করা
struct UserController;

#[bee_router::async_trait]
impl Controller for UserController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        ctx.json(&serde_json::json!({"users": []}))
    }
}
```

### রাউট রেজিস্ট্রেশন

```rust
// রাউট রেজিস্ট্রেশন
let router = Router::new()
    .ns("/api/v1", |ns| {
        ns.get("/users")
          .post("/users");
    });
```

### Context API

**Context যা দেয়:**
- `ctx.json()` / `ctx.text()` / `ctx.html()` — রেসপন্স আউটপুট
- `ctx.redirect()` — রিডাইরেক্ট
- `ctx.abort()` — রিকোয়েস্ট বাধা
- `ctx.session` — সেশন অ্যাক্সেস
- `ctx.params` — পাথ প্যারামিটার

## নিরাপত্তা সনাক্তকরণ (`security` feature)

[security-rust](https://crates.io/crates/security-rust)-ভিত্তিক আক্রমণ শনাক্তকরণ ফিল্টার, XSS, SQL ইনজেকশন, কমান্ড ইনজেকশন, SSRF সহ ২৭ ধরনের আক্রমণ কভার করে:

```rust
use bee_rust::prelude::*;

let security = SecurityFilter::new();  // ২৭টি ডিটেক্টর চালু
```

`Cargo.toml`-এ চালু করুন:
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

// চেইনড কোয়েরি
let users = User::query()
    .filter("age > 18")
    .order_by("created_at DESC")
    .limit(20)
    .to_sql();
// → SELECT * FROM users WHERE age > 18 ORDER BY created_at DESC LIMIT 20
```

## কনফিগ ম্যানেজমেন্ট (bee_config)

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

## স্টোরেজ ইঞ্জিন

**KV/Cache:**
```rust
let kv = RedisStore::new("redis://localhost:6379").await?;
kv.set("key", b"value", Some(Duration::from_secs(60))).await?;
let val = kv.get("key").await?;
```

**সার্চ ইঞ্জিন (পরিকল্পনাধীন):**
```rust
// ড্রাইভার বাস্তবায়ন পরিকল্পনাধীন, বর্তমানে trait stub
let engine = ElasticsearchEngine::new("http://localhost:9200")?;
let result = engine.search("my_index", &SearchQuery {
    q: Some("keyword".into()), ..Default::default()
}).await?;
```

**গ্রাফ ডেটাবেস (পরিকল্পনাধীন):**
```rust
// ড্রাইভার বাস্তবায়ন পরিকল্পনাধীন, বর্তমানে trait stub
let db = Neo4jDB::new("bolt://localhost:7687").await?;
let vid = db.add_vertex("Person", &[("name", "Alice")]).await?;
```

**টাইম-সিরিজ ডেটাবেস (পরিকল্পনাধীন):**
```rust
// ড্রাইভার বাস্তবায়ন পরিকল্পনাধীন, বর্তমানে trait stub
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

## লগিং

```rust
Logger::new()
    .level(Level::INFO)
    .output(Output::MultiFile("logs/"))
    .async_()
    .init()?;
```

## টেমপ্লেট

```rust
let engine = TemplateEngine::new("views/")?;
let result = engine.render("hello.html", &context! { name: &"World" })?;
// → "Hello, World!"
```

## CLI টুল

```bash
# প্রজেক্ট তৈরি (রানযোগ্য স্ক্যাফোল্ড তৈরি: Cargo.toml + src/main.rs)
bee-rust new my-app

# কোড জেনারেট করুন
bee-rust generate controller user
bee-rust generate model user --fields "name:string,age:int"

# ডেভ রান (--watch src/ ফোল্ডারের পরিবর্তন দেখে অটো-রিস্টার্ট)
bee-rust run
bee-rust run --watch

# প্যাকেজিং ও ডিপ্লয় (cargo build --release + dist/-তে কপি)
bee-rust pack

# ডেটাবেস মাইগ্রেশন (বাস্তবায়িত হয়নি, পরিকল্পনাধীন)
bee-rust migrate up
```

> নোট: `pack --target` প্যারামিটারটি রিজার্ভড ইন্টারফেস; বর্তমান প্যাকেজিং প্রক্রিয়া টার্গেট প্ল্যাটফর্ম আলাদা করে না।
