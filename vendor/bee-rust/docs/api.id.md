<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Referensi API Beerust

[English](api.en.md) · [한국어](api.ko.md) · [Русский](api.ru.md) · [Deutsch](api.de.md) · [Français](api.fr.md) · [Español](api.es.md) · [Português](api.pt.md) · [हिन्दी](api.hi.md) · [العربية](api.ar.md) · [বাংলা](api.bn.md) · [Bahasa Indonesia](api.id.md) · [日本語](api.ja.md)

Kembali ke [README](README.id.md).

Dokumen ini adalah referensi API framework Beerust: penggunaan, contoh kode, dan penjelasan antarmuka setiap modul.

## Web Inti (bee_router)

### Mendefinisikan Controller

```rust
use bee_rust::prelude::*;

// Mendefinisikan controller
struct UserController;

#[bee_router::async_trait]
impl Controller for UserController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        ctx.json(&serde_json::json!({"users": []}))
    }
}
```

### Registrasi Rute

```rust
// Registrasi rute
let router = Router::new()
    .ns("/api/v1", |ns| {
        ns.get("/users")
          .post("/users");
    });
```

### Context API

**Context menyediakan:**
- `ctx.json()` / `ctx.text()` / `ctx.html()` — output respons
- `ctx.redirect()` — pengalihan
- `ctx.abort()` — membatalkan permintaan
- `ctx.session` — akses sesi
- `ctx.params` — parameter jalur

## Deteksi Keamanan (fitur `security`)

Filter deteksi serangan berbasis [security-rust](https://crates.io/crates/security-rust), mencakup 27 jenis serangan seperti XSS, injeksi SQL, injeksi perintah, SSRF:

```rust
use bee_rust::prelude::*;

let security = SecurityFilter::new();  // 27 detektor semuanya aktif
```

Aktifkan di `Cargo.toml`:
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

// Query berantai
let users = User::query()
    .filter("age > 18")
    .order_by("created_at DESC")
    .limit(20)
    .to_sql();
// → SELECT * FROM users WHERE age > 18 ORDER BY created_at DESC LIMIT 20
```

## Manajemen Konfigurasi (bee_config)

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

## Mesin Penyimpanan

**KV/Cache:**
```rust
let kv = RedisStore::new("redis://localhost:6379").await?;
kv.set("key", b"value", Some(Duration::from_secs(60))).await?;
let val = kv.get("key").await?;
```

**Mesin pencarian (direncanakan):**
```rust
// Implementasi driver direncanakan, saat ini berupa trait stub
let engine = ElasticsearchEngine::new("http://localhost:9200")?;
let result = engine.search("my_index", &SearchQuery {
    q: Some("keyword".into()), ..Default::default()
}).await?;
```

**Basis data graf (direncanakan):**
```rust
// Implementasi driver direncanakan, saat ini berupa trait stub
let db = Neo4jDB::new("bolt://localhost:7687").await?;
let vid = db.add_vertex("Person", &[("name", "Alice")]).await?;
```

**Basis data deret waktu (direncanakan):**
```rust
// Implementasi driver direncanakan, saat ini berupa trait stub
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

## Template

```rust
let engine = TemplateEngine::new("views/")?;
let result = engine.render("hello.html", &context! { name: &"World" })?;
// → "Hello, World!"
```

## Alat CLI

```bash
# Buat proyek (menghasilkan scaffolding yang dapat dijalankan: Cargo.toml + src/main.rs)
bee-rust new my-app

# Generasi kode
bee-rust generate controller user
bee-rust generate model user --fields "name:string,age:int"

# Run development (--watch memantau perubahan di src/ dan restart otomatis)
bee-rust run
bee-rust run --watch

# Paket untuk deployment (cargo build --release + salin ke dist/)
bee-rust pack

# Migrasi basis data (belum diimplementasikan, direncanakan)
bee-rust migrate up
```

> Catatan: parameter `pack --target` adalah antarmuka cadangan; proses pengemasan saat ini tidak membedakan platform target.
