<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Beerust API संदर्भ

[English](api.en.md) · [한국어](api.ko.md) · [Русский](api.ru.md) · [Deutsch](api.de.md) · [Français](api.fr.md) · [Español](api.es.md) · [Português](api.pt.md) · [हिन्दी](api.hi.md) · [العربية](api.ar.md) · [বাংলা](api.bn.md) · [Bahasa Indonesia](api.id.md) · [日本語](api.ja.md)

वापस [README](README.hi.md) पर जाएँ।

यह दस्तावेज़ Beerust फ्रेमवर्क का API संदर्भ है: प्रत्येक मॉड्यूल का उपयोग, कोड उदाहरण और इंटरफ़ेस विवरण।

## Web कोर (bee_router)

### कंट्रोलर परिभाषित करना

```rust
use bee_rust::prelude::*;

// कंट्रोलर परिभाषित करें
struct UserController;

#[bee_router::async_trait]
impl Controller for UserController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        ctx.json(&serde_json::json!({"users": []}))
    }
}
```

### रूट रजिस्ट्रेशन

```rust
// रूट रजिस्ट्रेशन
let router = Router::new()
    .ns("/api/v1", |ns| {
        ns.get("/users")
          .post("/users");
    });
```

### Context API

**Context प्रदान करता है:**
- `ctx.json()` / `ctx.text()` / `ctx.html()` — प्रतिक्रिया आउटपुट
- `ctx.redirect()` — रीडायरेक्ट
- `ctx.abort()` — अनुरोध रोकना
- `ctx.session` — सत्र तक पहुँच
- `ctx.params` — पाथ पैरामीटर

## सुरक्षा जाँच (`security` feature)

[security-rust](https://crates.io/crates/security-rust) पर आधारित हमला-पता लगाने वाला फ़िल्टर, जो XSS, SQL इंजेक्शन, कमांड इंजेक्शन, SSRF आदि 27 प्रकार के हमलों को कवर करता है:

```rust
use bee_rust::prelude::*;

let security = SecurityFilter::new();  // सभी 27 डिटेक्टर चालू
```

`Cargo.toml` में सक्षम करें:
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

// चेन क्वेरी
let users = User::query()
    .filter("age > 18")
    .order_by("created_at DESC")
    .limit(20)
    .to_sql();
// → SELECT * FROM users WHERE age > 18 ORDER BY created_at DESC LIMIT 20
```

## कॉन्फ़िगरेशन प्रबंधन (bee_config)

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

## स्टोरेज इंजन

**KV/Cache:**
```rust
let kv = RedisStore::new("redis://localhost:6379").await?;
kv.set("key", b"value", Some(Duration::from_secs(60))).await?;
let val = kv.get("key").await?;
```

**खोज इंजन (योजनाबद्ध):**
```rust
// ड्राइवर कार्यान्वयन योजनाबद्ध है, फ़िलहाल trait stub है
let engine = ElasticsearchEngine::new("http://localhost:9200")?;
let result = engine.search("my_index", &SearchQuery {
    q: Some("keyword".into()), ..Default::default()
}).await?;
```

**ग्राफ़ डेटाबेस (योजनाबद्ध):**
```rust
// ड्राइवर कार्यान्वयन योजनाबद्ध है, फ़िलहाल trait stub है
let db = Neo4jDB::new("bolt://localhost:7687").await?;
let vid = db.add_vertex("Person", &[("name", "Alice")]).await?;
```

**टाइम-सीरीज़ डेटाबेस (योजनाबद्ध):**
```rust
// ड्राइवर कार्यान्वयन योजनाबद्ध है, फ़िलहाल trait stub है
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

## लॉगिंग

```rust
Logger::new()
    .level(Level::INFO)
    .output(Output::MultiFile("logs/"))
    .async_()
    .init()?;
```

## टेम्पलेट

```rust
let engine = TemplateEngine::new("views/")?;
let result = engine.render("hello.html", &context! { name: &"World" })?;
// → "Hello, World!"
```

## CLI उपकरण

```bash
# प्रोजेक्ट बनाएँ (चलने योग्य स्कैफोल्ड उत्पन्न करता है: Cargo.toml + src/main.rs)
bee-rust new my-app

# कोड जनरेट करें
bee-rust generate controller user
bee-rust generate model user --fields "name:string,age:int"

# डेवलपमेंट रन (--watch src/ में बदलाव देखकर स्वतः रीस्टार्ट करता है)
bee-rust run
bee-rust run --watch

# पैकेज और डिप्लॉय (cargo build --release + dist/ में कॉपी)
bee-rust pack

# डेटाबेस माइग्रेशन (लागू नहीं, योजनाबद्ध)
bee-rust migrate up
```

> नोट: `pack --target` पैरामीटर एक आरक्षित इंटरफ़ेस है; वर्तमान पैकेजिंग प्रक्रिया लक्ष्य प्लेटफ़ॉर्म में भेद नहीं करती।
