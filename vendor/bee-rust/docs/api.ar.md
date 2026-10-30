<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Beerust مرجع API

[English](api.en.md) · [한국어](api.ko.md) · [Русский](api.ru.md) · [Deutsch](api.de.md) · [Français](api.fr.md) · [Español](api.es.md) · [Português](api.pt.md) · [हिन्दी](api.hi.md) · [العربية](api.ar.md) · [বাংলা](api.bn.md) · [Bahasa Indonesia](api.id.md) · [日本語](api.ja.md)

العودة إلى [README](README.ar.md).

هذا المستند هو مرجع API لإطار Beerust: استخدام كل وحدة، وأمثلة الكود، وشرح الواجهات.

## نواة الويب (bee_router)

### تعريف متحكم

```rust
use bee_rust::prelude::*;

// تعريف متحكم
struct UserController;

#[bee_router::async_trait]
impl Controller for UserController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        ctx.json(&serde_json::json!({"users": []}))
    }
}
```

### تسجيل المسارات

```rust
// تسجيل المسارات
let router = Router::new()
    .ns("/api/v1", |ns| {
        ns.get("/users")
          .post("/users");
    });
```

### Context API

**يوفر Context:**
- `ctx.json()` / `ctx.text()` / `ctx.html()` — مخرجات الاستجابة
- `ctx.redirect()` — إعادة التوجيه
- `ctx.abort()` — مقاطعة الطلب
- `ctx.session` — الوصول إلى الجلسة
- `ctx.params` — معاملات المسار

## كشف الأمان (feature: `security`)

فلتر كشف الهجمات مبني على [security-rust](https://crates.io/crates/security-rust)، يغطي 27 نوعًا من الهجمات مثل XSS و SQL Injection و Command Injection و SSRF:

```rust
use bee_rust::prelude::*;

let security = SecurityFilter::new();  // تفعيل أدوات الكشف الـ 27 كلها
```

التمكين في `Cargo.toml`:
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

// استعلام متسلسل
let users = User::query()
    .filter("age > 18")
    .order_by("created_at DESC")
    .limit(20)
    .to_sql();
// → SELECT * FROM users WHERE age > 18 ORDER BY created_at DESC LIMIT 20
```

## إدارة التكوين (bee_config)

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

## محركات التخزين

**KV/Cache:**
```rust
let kv = RedisStore::new("redis://localhost:6379").await?;
kv.set("key", b"value", Some(Duration::from_secs(60))).await?;
let val = kv.get("key").await?;
```

**محرك البحث (مخطط):**
```rust
// تنفيذ برنامج التشغيل مخطط، حاليًا trait stub
let engine = ElasticsearchEngine::new("http://localhost:9200")?;
let result = engine.search("my_index", &SearchQuery {
    q: Some("keyword".into()), ..Default::default()
}).await?;
```

**قاعدة البيانات الرسومية (مخطط):**
```rust
// تنفيذ برنامج التشغيل مخطط، حاليًا trait stub
let db = Neo4jDB::new("bolt://localhost:7687").await?;
let vid = db.add_vertex("Person", &[("name", "Alice")]).await?;
```

**قاعدة بيانات السلاسل الزمنية (مخطط):**
```rust
// تنفيذ برنامج التشغيل مخطط، حاليًا trait stub
let tsdb = InfluxDB::new("http://localhost:8086").await?;
tsdb.write_point("cpu", &[("host", "srv1")], &[("value", 0.85)], Utc::now()).await?;
```

## الجلسات

```rust
let cache = Arc::new(MemoryCache::new());
let mut session = Session::new(cache, Duration::from_secs(3600));
session.set("user_id", &"123")?;
let uid: String = session.get("user_id")?.unwrap();
```

## السجلات

```rust
Logger::new()
    .level(Level::INFO)
    .output(Output::MultiFile("logs/"))
    .async_()
    .init()?;
```

## القوالب

```rust
let engine = TemplateEngine::new("views/")?;
let result = engine.render("hello.html", &context! { name: &"World" })?;
// → "Hello, World!"
```

## أدوات CLI

```bash
# إنشاء مشروع (يولد سقالة قابلة للتشغيل: Cargo.toml + src/main.rs)
bee-rust new my-app

# توليد الكود
bee-rust generate controller user
bee-rust generate model user --fields "name:string,age:int"

# تشغيل تطويري (--watch يراقب تغييرات src/ ويعيد التشغيل تلقائيًا)
bee-rust run
bee-rust run --watch

# تغليف ونشر (cargo build --release + نسخ إلى dist/)
bee-rust pack

# ترحيل قاعدة البيانات (غير منفذ، مخطط)
bee-rust migrate up
```

> ملاحظة: معامل `pack --target` واجهة محجوزة، وعملية التغليف الحالية لا تفرق منصات الهدف.
