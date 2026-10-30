<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Справочник API Beerust

[English](api.en.md) · [한국어](api.ko.md) · [Русский](api.ru.md) · [Deutsch](api.de.md) · [Français](api.fr.md) · [Español](api.es.md) · [Português](api.pt.md) · [हिन्दी](api.hi.md) · [العربية](api.ar.md) · [বাংলা](api.bn.md) · [Bahasa Indonesia](api.id.md) · [日本語](api.ja.md)

Назад к [README](README.ru.md).

Этот документ — справочник API фреймворка Beerust: использование модулей, примеры кода и описание интерфейсов.

## Веб-ядро (bee_router)

### Определение контроллера

```rust
use bee_rust::prelude::*;

// определяем контроллер
struct UserController;

#[bee_router::async_trait]
impl Controller for UserController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        ctx.json(&serde_json::json!({"users": []}))
    }
}
```

### Регистрация маршрутов

```rust
// регистрируем маршруты
let router = Router::new()
    .ns("/api/v1", |ns| {
        ns.get("/users")
          .post("/users");
    });
```

### Context API

**Context предоставляет:**
- `ctx.json()` / `ctx.text()` / `ctx.html()` — вывод ответа
- `ctx.redirect()` — перенаправление
- `ctx.abort()` — прерывание запроса
- `ctx.session` — доступ к сессии
- `ctx.params` — параметры пути

## Обнаружение атак (feature `security`)

Фильтр обнаружения атак на основе [security-rust](https://crates.io/crates/security-rust), покрывает 27 типов атак: XSS, SQL-инъекции, инъекции команд, SSRF и другие:

```rust
use bee_rust::prelude::*;

let security = SecurityFilter::new();  // все 27 детекторов включены
```

Включите в `Cargo.toml`:
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

// цепочный запрос
let users = User::query()
    .filter("age > 18")
    .order_by("created_at DESC")
    .limit(20)
    .to_sql();
// → SELECT * FROM users WHERE age > 18 ORDER BY created_at DESC LIMIT 20
```

## Управление конфигурацией (bee_config)

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

## Движки хранения

**KV/Кэш:**
```rust
let kv = RedisStore::new("redis://localhost:6379").await?;
kv.set("key", b"value", Some(Duration::from_secs(60))).await?;
let val = kv.get("key").await?;
```

**Поисковый движок (в планах):**
```rust
// реализация драйвера в планах, пока это trait-stub
let engine = ElasticsearchEngine::new("http://localhost:9200")?;
let result = engine.search("my_index", &SearchQuery {
    q: Some("keyword".into()), ..Default::default()
}).await?;
```

**Графовая БД (в планах):**
```rust
// реализация драйвера в планах, пока это trait-stub
let db = Neo4jDB::new("bolt://localhost:7687").await?;
let vid = db.add_vertex("Person", &[("name", "Alice")]).await?;
```

**БД временных рядов (в планах):**
```rust
// реализация драйвера в планах, пока это trait-stub
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

## Логирование

```rust
Logger::new()
    .level(Level::INFO)
    .output(Output::MultiFile("logs/"))
    .async_()
    .init()?;
```

## Шаблоны

```rust
let engine = TemplateEngine::new("views/")?;
let result = engine.render("hello.html", &context! { name: &"World" })?;
// → "Hello, World!"
```

## Инструменты CLI

```bash
# создание проекта (генерирует рабочий скаффолд: Cargo.toml + src/main.rs)
bee-rust new my-app

# генерация кода
bee-rust generate controller user
bee-rust generate model user --fields "name:string,age:int"

# запуск в разработке (--watch следит за изменениями в src/ и перезапускает автоматически)
bee-rust run
bee-rust run --watch

# упаковка для развёртывания (cargo build --release + копирование в dist/)
bee-rust pack

# миграции базы данных (не реализовано, в планах)
bee-rust migrate up
```

> Примечание: параметр `pack --target` — зарезервированный интерфейс; сейчас процесс упаковки не различает целевые платформы.
