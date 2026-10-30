<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Référence API Beerust

[English](api.en.md) · [한국어](api.ko.md) · [Русский](api.ru.md) · [Deutsch](api.de.md) · [Français](api.fr.md) · [Español](api.es.md) · [Português](api.pt.md) · [हिन्दी](api.hi.md) · [العربية](api.ar.md) · [বাংলা](api.bn.md) · [Bahasa Indonesia](api.id.md) · [日本語](api.ja.md)

Retour au [README](README.fr.md)。

Ce document est la référence API du framework Beerust : usages de chaque module, exemples de code et description des interfaces.

## Cœur Web (bee_router)

### Définir un contrôleur

```rust
use bee_rust::prelude::*;

// Définition du contrôleur
struct UserController;

#[bee_router::async_trait]
impl Controller for UserController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        ctx.json(&serde_json::json!({"users": []}))
    }
}
```

### Enregistrement des routes

```rust
// Enregistrement des routes
let router = Router::new()
    .ns("/api/v1", |ns| {
        ns.get("/users")
          .post("/users");
    });
```

### API de Context

**Context fournit :**
- `ctx.json()` / `ctx.text()` / `ctx.html()` — sortie de réponse
- `ctx.redirect()` — redirection
- `ctx.abort()` — interruption de la requête
- `ctx.session` — accès à la session
- `ctx.params` — paramètres de chemin

## Détection de sécurité (feature `security`)

Un filtre de détection d'attaques basé sur [security-rust](https://crates.io/crates/security-rust), couvrant 27 types d'attaques dont XSS, injection SQL, injection de commandes et SSRF :

```rust
use bee_rust::prelude::*;

let security = SecurityFilter::new();  // les 27 détecteurs sont tous activés
```

Pour l'activer dans `Cargo.toml` :
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

// Requête chaînée
let users = User::query()
    .filter("age > 18")
    .order_by("created_at DESC")
    .limit(20)
    .to_sql();
// → SELECT * FROM users WHERE age > 18 ORDER BY created_at DESC LIMIT 20
```

## Gestion de configuration (bee_config)

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

## Moteurs de stockage

**KV/Cache :**
```rust
let kv = RedisStore::new("redis://localhost:6379").await?;
kv.set("key", b"value", Some(Duration::from_secs(60))).await?;
let val = kv.get("key").await?;
```

**Moteur de recherche (prévu) :**
```rust
// Implémentation du driver prévue, actuellement un stub de trait
let engine = ElasticsearchEngine::new("http://localhost:9200")?;
let result = engine.search("my_index", &SearchQuery {
    q: Some("keyword".into()), ..Default::default()
}).await?;
```

**Base de données graphe (prévue) :**
```rust
// Implémentation du driver prévue, actuellement un stub de trait
let db = Neo4jDB::new("bolt://localhost:7687").await?;
let vid = db.add_vertex("Person", &[("name", "Alice")]).await?;
```

**Séries temporelles (prévu) :**
```rust
// Implémentation du driver prévue, actuellement un stub de trait
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

## Journalisation

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

## Outil CLI

```bash
# Crée un projet (génère un scaffolding exécutable : Cargo.toml + src/main.rs)
bee-rust new my-app

# Génération de code
bee-rust generate controller user
bee-rust generate model user --fields "name:string,age:int"

# Exécution de développement (--watch surveille src/ et redémarre automatiquement)
bee-rust run
bee-rust run --watch

# Empaquetage et déploiement (cargo build --release + copie dans dist/)
bee-rust pack

# Migration de base de données (non implémentée, prévue)
bee-rust migrate up
```

> Note : le paramètre `pack --target` est une interface réservée ; le processus d'empaquetage actuel ne distingue pas les plateformes cibles.
