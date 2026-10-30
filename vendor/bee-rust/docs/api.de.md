<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Beerust API-Referenz

[English](api.en.md) · [한국어](api.ko.md) · [Русский](api.ru.md) · [Deutsch](api.de.md) · [Français](api.fr.md) · [Español](api.es.md) · [Português](api.pt.md) · [हिन्दी](api.hi.md) · [العربية](api.ar.md) · [বাংলা](api.bn.md) · [Bahasa Indonesia](api.id.md) · [日本語](api.ja.md)

Zurück zur [README](README.de.md).

Dieses Dokument ist die API-Referenz des Beerust-Frameworks: Verwendung der einzelnen Module, Codebeispiele und Schnittstellenbeschreibungen.

## Web-Kern (bee_router)

### Controller definieren

```rust
use bee_rust::prelude::*;

// Controller definieren
struct UserController;

#[bee_router::async_trait]
impl Controller for UserController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        ctx.json(&serde_json::json!({"users": []}))
    }
}
```

### Routenregistrierung

```rust
// Routenregistrierung
let router = Router::new()
    .ns("/api/v1", |ns| {
        ns.get("/users")
          .post("/users");
    });
```

### Context-API

**Context bietet:**
- `ctx.json()` / `ctx.text()` / `ctx.html()` — Antwortausgabe
- `ctx.redirect()` — Weiterleitung
- `ctx.abort()` — Anfrage abbrechen
- `ctx.session` — Session-Zugriff
- `ctx.params` — Pfadparameter

## Sicherheitserkennung (`security`-Feature)

Ein auf [security-rust](https://crates.io/crates/security-rust) basierender Angriffserkennungs-Filter, der 27 Angriffstypen wie XSS, SQL-Injection, Command-Injection und SSRF abdeckt:

```rust
use bee_rust::prelude::*;

let security = SecurityFilter::new();  // alle 27 Detektoren aktiv
```

In `Cargo.toml` aktivieren:
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

// Kettenabfrage
let users = User::query()
    .filter("age > 18")
    .order_by("created_at DESC")
    .limit(20)
    .to_sql();
// → SELECT * FROM users WHERE age > 18 ORDER BY created_at DESC LIMIT 20
```

## Konfigurationsverwaltung (bee_config)

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

## Speicher-Engines

**KV/Cache:**
```rust
let kv = RedisStore::new("redis://localhost:6379").await?;
kv.set("key", b"value", Some(Duration::from_secs(60))).await?;
let val = kv.get("key").await?;
```

**Suchmaschine (geplant):**
```rust
// Treiberimplementierung geplant, derzeit Trait-Stub
let engine = ElasticsearchEngine::new("http://localhost:9200")?;
let result = engine.search("my_index", &SearchQuery {
    q: Some("keyword".into()), ..Default::default()
}).await?;
```

**Graphdatenbank (geplant):**
```rust
// Treiberimplementierung geplant, derzeit Trait-Stub
let db = Neo4jDB::new("bolt://localhost:7687").await?;
let vid = db.add_vertex("Person", &[("name", "Alice")]).await?;
```

**Zeitreihendatenbank (geplant):**
```rust
// Treiberimplementierung geplant, derzeit Trait-Stub
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

## CLI-Werkzeuge

```bash
# Projekt erstellen (erzeugt lauffähiges Scaffolding: Cargo.toml + src/main.rs)
bee-rust new my-app

# Code generieren
bee-rust generate controller user
bee-rust generate model user --fields "name:string,age:int"

# Entwicklungsbetrieb (--watch überwacht src/ und startet bei Änderungen automatisch neu)
bee-rust run
bee-rust run --watch

# Build und Paketierung (cargo build --release + Kopieren nach dist/)
bee-rust pack

# Datenbankmigration (nicht implementiert, geplant)
bee-rust migrate up
```

> Hinweis: Der Parameter `pack --target` ist eine reservierte Schnittstelle; der aktuelle Paketierungsprozess unterscheidet nicht zwischen Zielplattformen.
