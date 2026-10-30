<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Beerust API リファレンス

[English](api.en.md) · [한국어](api.ko.md) · [Русский](api.ru.md) · [Deutsch](api.de.md) · [Français](api.fr.md) · [Español](api.es.md) · [Português](api.pt.md) · [हिन्दी](api.hi.md) · [العربية](api.ar.md) · [বাংলা](api.bn.md) · [Bahasa Indonesia](api.id.md) · [日本語](api.ja.md)

README に戻る：[README](README.ja.md)。

このドキュメントは Beerust フレームワークの API リファレンスです：各モジュールの使い方、コード例、インターフェースの説明。

## Web コア（bee_router）

### コントローラの定義

```rust
use bee_rust::prelude::*;

// コントローラを定義
struct UserController;

#[bee_router::async_trait]
impl Controller for UserController {
    async fn handle(&self, ctx: &mut Context) -> Result<(), RouterError> {
        ctx.json(&serde_json::json!({"users": []}))
    }
}
```

### ルートの登録

```rust
// ルートの登録
let router = Router::new()
    .ns("/api/v1", |ns| {
        ns.get("/users")
          .post("/users");
    });
```

### Context API

**Context が提供するもの：**
- `ctx.json()` / `ctx.text()` / `ctx.html()` — レスポンス出力
- `ctx.redirect()` — リダイレクト
- `ctx.abort()` — リクエストの中断
- `ctx.session` — セッションへのアクセス
- `ctx.params` — パスパラメータ

## セキュリティ検知（`security` feature）

[security-rust](https://crates.io/crates/security-rust) に基づく攻撃検知フィルタ。XSS、SQL インジェクション、コマンドインジェクション、SSRF など 27 種類の攻撃タイプをカバーします：

```rust
use bee_rust::prelude::*;

let security = SecurityFilter::new();  // 27 個の検出器をすべて有効化
```

`Cargo.toml` で有効化：
```toml
bee_rust = { features = ["security"] }
```

## ORM（bee_orm）

```rust
#[derive(Model)]
#[bee(table = "users")]
struct User {
    id:   i64,
    name: String,
    age:  i32,
}

// チェーン式クエリ
let users = User::query()
    .filter("age > 18")
    .order_by("created_at DESC")
    .limit(20)
    .to_sql();
// → SELECT * FROM users WHERE age > 18 ORDER BY created_at DESC LIMIT 20
```

## 設定管理（bee_config）

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

## ストレージエンジン

**KV/Cache：**
```rust
let kv = RedisStore::new("redis://localhost:6379").await?;
kv.set("key", b"value", Some(Duration::from_secs(60))).await?;
let val = kv.get("key").await?;
```

**検索エンジン（計画中）：**
```rust
// ドライバ実装は計画中、現在は trait のスタブ
let engine = ElasticsearchEngine::new("http://localhost:9200")?;
let result = engine.search("my_index", &SearchQuery {
    q: Some("keyword".into()), ..Default::default()
}).await?;
```

**グラフデータベース（計画中）：**
```rust
// ドライバ実装は計画中、現在は trait のスタブ
let db = Neo4jDB::new("bolt://localhost:7687").await?;
let vid = db.add_vertex("Person", &[("name", "Alice")]).await?;
```

**時系列データベース（計画中）：**
```rust
// ドライバ実装は計画中、現在は trait のスタブ
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

## ログ

```rust
Logger::new()
    .level(Level::INFO)
    .output(Output::MultiFile("logs/"))
    .async_()
    .init()?;
```

## テンプレート

```rust
let engine = TemplateEngine::new("views/")?;
let result = engine.render("hello.html", &context! { name: &"World" })?;
// → "Hello, World!"
```

## CLI ツール

```bash
# プロジェクトを作成（実行可能なスキャフォールディングを生成：Cargo.toml + src/main.rs）
bee-rust new my-app

# コードを生成
bee-rust generate controller user
bee-rust generate model user --fields "name:string,age:int"

# 開発実行（--watch は src/ の変更を監視して自動再起動）
bee-rust run
bee-rust run --watch

# パッケージングしてデプロイ（cargo build --release + dist/ にコピー）
bee-rust pack

# データベースマイグレーション（未実装、計画中）
bee-rust migrate up
```

> 注記：`pack --target` パラメータは予約済みのインターフェースで、現在のパッケージング処理はターゲットプラットフォームを区別しません。
