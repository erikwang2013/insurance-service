<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Beerust API 참조

[English](api.en.md) · [한국어](api.ko.md) · [Русский](api.ru.md) · [Deutsch](api.de.md) · [Français](api.fr.md) · [Español](api.es.md) · [Português](api.pt.md) · [हिन्दी](api.hi.md) · [العربية](api.ar.md) · [বাংলা](api.bn.md) · [Bahasa Indonesia](api.id.md) · [日本語](api.ja.md)

[README](README.ko.md)로 돌아가기.

이 문서는 Beerust 프레임워크의 API 참조입니다: 각 모듈의 사용법, 코드 예제와 인터페이스 설명.

## Web 핵심 (bee_router)

### 컨트롤러 정의

```rust
use bee_rust::prelude::*;

// 컨트롤러 정의
struct UserController;

#[bee_router::async_trait]
impl Controller for UserController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        ctx.json(&serde_json::json!({"users": []}))
    }
}
```

### 라우팅 등록

```rust
// 라우팅 등록
let router = Router::new()
    .ns("/api/v1", |ns| {
        ns.get("/users")
          .post("/users");
    });
```

### Context API

**Context 제공:**
- `ctx.json()` / `ctx.text()` / `ctx.html()` — 응답 출력
- `ctx.redirect()` — 리다이렉트
- `ctx.abort()` — 요청 중단
- `ctx.session` — 세션 접근
- `ctx.params` — 경로 파라미터

## 보안 감지 (`security` feature)

[security-rust](https://crates.io/crates/security-rust) 기반의 공격 감지 필터로, XSS, SQL 인젝션, 명령어 인젝션, SSRF 등 27가지 공격 유형을 다룹니다:

```rust
use bee_rust::prelude::*;

let security = SecurityFilter::new();  // 27개 탐지기 모두 켜짐
```

`Cargo.toml`에서 활성화:
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

// 체이닝 쿼리
let users = User::query()
    .filter("age > 18")
    .order_by("created_at DESC")
    .limit(20)
    .to_sql();
// → SELECT * FROM users WHERE age > 18 ORDER BY created_at DESC LIMIT 20
```

## 설정 관리 (bee_config)

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

## 스토리지 엔진

**KV/Cache:**
```rust
let kv = RedisStore::new("redis://localhost:6379").await?;
kv.set("key", b"value", Some(Duration::from_secs(60))).await?;
let val = kv.get("key").await?;
```

**검색 엔진 (계획 중):**
```rust
// 드라이버 구현 예정, 현재는 trait stub
let engine = ElasticsearchEngine::new("http://localhost:9200")?;
let result = engine.search("my_index", &SearchQuery {
    q: Some("keyword".into()), ..Default::default()
}).await?;
```

**그래프 데이터베이스 (계획 중):**
```rust
// 드라이버 구현 예정, 현재는 trait stub
let db = Neo4jDB::new("bolt://localhost:7687").await?;
let vid = db.add_vertex("Person", &[("name", "Alice")]).await?;
```

**시계열 데이터베이스 (계획 중):**
```rust
// 드라이버 구현 예정, 현재는 trait stub
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

## 로그

```rust
Logger::new()
    .level(Level::INFO)
    .output(Output::MultiFile("logs/"))
    .async_()
    .init()?;
```

## 템플릿

```rust
let engine = TemplateEngine::new("views/")?;
let result = engine.render("hello.html", &context! { name: &"World" })?;
// → "Hello, World!"
```

## CLI 도구

```bash
# 프로젝트 생성 (실행 가능한 스캐폴딩 생성: Cargo.toml + src/main.rs)
bee-rust new my-app

# 코드 생성
bee-rust generate controller user
bee-rust generate model user --fields "name:string,age:int"

# 개발 실행 (--watch는 src/ 변경을 감지해 자동 재시작)
bee-rust run
bee-rust run --watch

# 패키징 배포 (cargo build --release + dist/로 복사)
bee-rust pack

# 데이터베이스 마이그레이션 (미구현, 계획 중)
bee-rust migrate up
```

> 참고: `pack --target` 파라미터는 예약된 인터페이스이며, 현재 패키징 프로세스는 대상 플랫폼을 구분하지 않습니다.
