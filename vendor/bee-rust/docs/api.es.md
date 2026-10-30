<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Referencia de la API de Beerust

[English](api.en.md) · [한국어](api.ko.md) · [Русский](api.ru.md) · [Deutsch](api.de.md) · [Français](api.fr.md) · [Español](api.es.md) · [Português](api.pt.md) · [हिन्दी](api.hi.md) · [العربية](api.ar.md) · [বাংলা](api.bn.md) · [Bahasa Indonesia](api.id.md) · [日本語](api.ja.md)

Volver al [README](README.es.md).

Este documento es la referencia de la API del framework Beerust: el uso de cada módulo, los ejemplos de código y la descripción de las interfaces.

## Núcleo web (bee_router)

### Definir un controlador

```rust
use bee_rust::prelude::*;

// Definir el controlador
struct UserController;

#[bee_router::async_trait]
impl Controller for UserController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        ctx.json(&serde_json::json!({"users": []}))
    }
}
```

### Registro de rutas

```rust
// Registro de rutas
let router = Router::new()
    .ns("/api/v1", |ns| {
        ns.get("/users")
          .post("/users");
    });
```

### API de Context

**`Context` proporciona:**
- `ctx.json()` / `ctx.text()` / `ctx.html()` — salida de respuestas
- `ctx.redirect()` — redirección
- `ctx.abort()` — interrumpe la petición
- `ctx.session` — acceso a la sesión
- `ctx.params` — parámetros de ruta

## Detección de seguridad (feature `security`)

Filtro de detección de ataques basado en [security-rust](https://crates.io/crates/security-rust) que cubre 27 tipos de ataques, como XSS, inyección SQL, inyección de comandos y SSRF:

```rust
use bee_rust::prelude::*;

let security = SecurityFilter::new();  // Los 27 detectores activados
```

Para activarlo en `Cargo.toml`:
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

// Consulta encadenada
let users = User::query()
    .filter("age > 18")
    .order_by("created_at DESC")
    .limit(20)
    .to_sql();
// → SELECT * FROM users WHERE age > 18 ORDER BY created_at DESC LIMIT 20
```

## Gestión de configuración (bee_config)

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

## Motores de almacenamiento

**KV/Caché:**
```rust
let kv = RedisStore::new("redis://localhost:6379").await?;
kv.set("key", b"value", Some(Duration::from_secs(60))).await?;
let val = kv.get("key").await?;
```

**Motor de búsqueda (planificado):**
```rust
// La implementación del driver está planificada; actualmente es un trait stub
let engine = ElasticsearchEngine::new("http://localhost:9200")?;
let result = engine.search("my_index", &SearchQuery {
    q: Some("keyword".into()), ..Default::default()
}).await?;
```

**Base de datos de grafos (planificada):**
```rust
// La implementación del driver está planificada; actualmente es un trait stub
let db = Neo4jDB::new("bolt://localhost:7687").await?;
let vid = db.add_vertex("Person", &[("name", "Alice")]).await?;
```

**Base de datos de series temporales (planificada):**
```rust
// La implementación del driver está planificada; actualmente es un trait stub
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

## Registros

```rust
Logger::new()
    .level(Level::INFO)
    .output(Output::MultiFile("logs/"))
    .async_()
    .init()?;
```

## Plantillas

```rust
let engine = TemplateEngine::new("views/")?;
let result = engine.render("hello.html", &context! { name: &"World" })?;
// → "Hello, World!"
```

## Herramientas CLI

```bash
# Crea un proyecto (genera un andamiaje ejecutable: Cargo.toml + src/main.rs)
bee-rust new my-app

# Genera código
bee-rust generate controller user
bee-rust generate model user --fields "name:string,age:int"

# Ejecución en desarrollo (--watch observa src/ y reinicia automáticamente ante cambios)
bee-rust run
bee-rust run --watch

# Empaquetado y despliegue (cargo build --release + copia a dist/)
bee-rust pack

# Migración de base de datos (no implementada, planificada)
bee-rust migrate up
```

> Nota: el parámetro `pack --target` es una interfaz reservada; el proceso de empaquetado actual no distingue plataformas de destino.
