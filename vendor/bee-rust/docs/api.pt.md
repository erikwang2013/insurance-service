<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Referência da API Beerust

[English](api.en.md) · [한국어](api.ko.md) · [Русский](api.ru.md) · [Deutsch](api.de.md) · [Français](api.fr.md) · [Español](api.es.md) · [Português](api.pt.md) · [हिन्दी](api.hi.md) · [العربية](api.ar.md) · [বাংলা](api.bn.md) · [Bahasa Indonesia](api.id.md) · [日本語](api.ja.md)

Voltar ao [README](README.pt.md).

Este documento é a referência da API do framework Beerust: o uso de cada módulo, exemplos de código e descrição das interfaces.

## Núcleo Web (bee_router)

### Definindo um controlador

```rust
use bee_rust::prelude::*;

// Define um controlador
struct UserController;

#[bee_router::async_trait]
impl Controller for UserController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        ctx.json(&serde_json::json!({"users": []}))
    }
}
```

### Registro de rotas

```rust
// Registro de rotas
let router = Router::new()
    .ns("/api/v1", |ns| {
        ns.get("/users")
          .post("/users");
    });
```

### API do Context

**O Context oferece:**
- `ctx.json()` / `ctx.text()` / `ctx.html()` — saída de resposta
- `ctx.redirect()` — redirecionamento
- `ctx.abort()` — interromper a requisição
- `ctx.session` — acesso à sessão
- `ctx.params` — parâmetros de caminho

## Detecção de segurança (feature `security`)

Filtro de detecção de ataques baseado em [security-rust](https://crates.io/crates/security-rust), cobrindo 27 tipos de ataque, incluindo XSS, injeção de SQL, injeção de comandos e SSRF:

```rust
use bee_rust::prelude::*;

let security = SecurityFilter::new();  // todos os 27 detectores ativados
```

Habilite no `Cargo.toml`:
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

// Consulta encadeada
let users = User::query()
    .filter("age > 18")
    .order_by("created_at DESC")
    .limit(20)
    .to_sql();
// → SELECT * FROM users WHERE age > 18 ORDER BY created_at DESC LIMIT 20
```

## Gerenciamento de configuração (bee_config)

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

## Mecanismos de armazenamento

**KV/Cache:**
```rust
let kv = RedisStore::new("redis://localhost:6379").await?;
kv.set("key", b"value", Some(Duration::from_secs(60))).await?;
let val = kv.get("key").await?;
```

**Mecanismo de busca (planejado):**
```rust
// Implementação do driver planejada; atualmente é um stub de trait
let engine = ElasticsearchEngine::new("http://localhost:9200")?;
let result = engine.search("my_index", &SearchQuery {
    q: Some("keyword".into()), ..Default::default()
}).await?;
```

**Banco de dados de grafos (planejado):**
```rust
// Implementação do driver planejada; atualmente é um stub de trait
let db = Neo4jDB::new("bolt://localhost:7687").await?;
let vid = db.add_vertex("Person", &[("name", "Alice")]).await?;
```

**Banco de dados de séries temporais (planejado):**
```rust
// Implementação do driver planejada; atualmente é um stub de trait
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

## Logs

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

## Ferramenta CLI

```bash
# Cria um projeto (gera um scaffolding executável: Cargo.toml + src/main.rs)
bee-rust new my-app

# Gera código
bee-rust generate controller user
bee-rust generate model user --fields "name:string,age:int"

# Execução de desenvolvimento (--watch monitora mudanças em src/ e reinicia automaticamente)
bee-rust run
bee-rust run --watch

# Empacotamento para implantação (cargo build --release + cópia para dist/)
bee-rust pack

# Migração de banco de dados (não implementada, planejada)
bee-rust migrate up
```

> Nota: o parâmetro `pack --target` é uma interface reservada; o processo de empacotamento atual não distingue a plataforma de destino.
