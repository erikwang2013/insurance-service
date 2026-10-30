<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Beerust

[简体中文](../README.md) · [English](README.en.md) · [한국어](README.ko.md) · [Русский](README.ru.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Português](README.pt.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [বাংলা](README.bn.md) · [Bahasa Indonesia](README.id.md) · [日本語](README.ja.md)

Beerust は Rust 言語によるプロダクショングレードの Web フレームワークです。設計哲学は Go の Beego フレームワークに由来し、Rust らしい trait、macro、型システムで再表現されています。

## プロジェクトマスコット：Rusty

<img src="rusty.svg" width="360" alt="正面でホバリングする Rusty。羽を環状に広げ、前足で錆オレンジ色の工具ボックスを抱えている">

### キャラクターカード

**名前** Rusty（サビサビ）——Rust の錆色であり、つなぎの作業着にこすりついた鉄錆でもあります。

**見た目**：ふわふわの毛に覆われたエンジニアのハチ。琥珀色の黄金の綿毛がまんまるのお腹を覆い、3 本の炭黒の縞模様はよじれた作業用サスペンダーのようです。4 枚の透明な羽は透けて見えるほど薄く、ホバリングするとブンブンと残像が揺れます。複眼はガラス玉のような黒曜石で、それぞれに一粒のハイライト。前足で磨き上げられた錆オレンジ色の工具ボックスを抱えています——それが花粉かごで、中に花粉が入ったことは一度もなく、入っているのは依存関係です。

**性格**：仕事人間、楽天家、軽度の完璧主義。

- 巣を修理するときは触角で一マスずつチェックし、lint を実行するかのように細かく確認する
- 花粉かごの中の依存関係はバージョンロックが厳重で、誰かが動かそうものなら怒る
- バグを追いかけるときは空中で静止してホバリングし、羽は速すぎて輪郭の残像だけになる
- 新バージョンのリリース日には、巣の入り口でハチの 8 の字ダンスを踊る

### ビジュアル規定

| 要素 | 規定 |
|------|------|
| 本体 | 琥珀色の金 `#F5B301`、腹部の綿毛は短く密、縁に一圈のハイライト |
| 縞模様 | 炭黒 `#1F1A17`、3 本、腹部のみ |
| 羽 | 透明な淡い青 `#CFE8F5`、不透明度 40%、脈が描き出せる |
| 複眼 | 深い茶黒 `#2A1E16`、楕円形、左上に一粒のハイライト |
| アクセント | 錆オレンジ `#B7410E`（工具ボックス、蝶ネクタイ）、鉄錆色が彼のトレードマーク |

### 表情集

| 表情 | 見た目 | どんなとき |
|------|------|---------|
| 元気いっぱい | `(^o^)` 綿毛が逆立ち、羽が高周波で振動 | テストが全緑のとき |
| ホバリングデバッグ | `o(°▽°)o` 空中に浮かび、羽の残像だけが残る | 手ごわいバグを追いかけているとき |
| 満載で帰還 | `(≧▽≦)` 工具ボックスがパンパンで、ふらふらと飛ぶ | 新しいドライバの接続に成功したとき |
| 居眠り | `(-_-)zZ` 巣の入り口にしゃがみ、触角がだらりと垂れる | 深夜でトラフィックがないとき |
| 毛を逆立てる | `(#°益°)` 尾針が突き立つ | セキュリティフィルタが攻撃をブロックしたとき |

### 看板アクション

1. **ホバリングデバッグ** —— バグを見つけるとコードの上にホバリングし、羽がブンブンと残像になり、あなたが直し終えるまでじっと見つめます。
2. **8 の字ダンス** —— 新バージョンをリリースするたびに、巣の周りを一周踊り、「花が咲いたよ」と巣全体に告げます。本物のハチはこれで花の位置を伝えますが、Rusty はこれで changelog を伝えます。
3. **毛づくろい** —— 前足で触角を何度もこすります。見た目を気にしていると思われがちですが、実は設定がホットリロードされていないかをチェックしているのです。

### どこに登場するか

- **ロゴ**：正面ホバリングの姿勢で、羽を環状に広げてちょうど名前を囲みます。
- **404 ページ**：空っぽの花畑の上にホバリングして、途方に暮れてぐるぐる回ります——「蜜はどこ？」
- **CLI**：起動バナーに並ぶ小さなハチたち。テストを実行すると、そばにしゃがんで監督します。
- **リリース告知**：8 の字ダンスを踊っているあの一枚。

## 設計目標

| 目標 | 指標 |
|------|------|
| **開発体験** | `bee-rust new` から最初のリクエストまで 30 秒未満 |
| **パフォーマンス** | コントローラ層のオーバーヘッド < 5%（素の axum との比較）、P99 ルーティング遅延 < 100µs |
| **コンパイル速度** | メタ crate のフルコンパイル < 60 秒（release）、インクリメンタルコンパイル < 5 秒 |
| **バイナリサイズ** | 最小アプリ（router のみ）< 5MB（strip + LTO） |
| **安全性** | 業務コードに unsafe ゼロ。すべての FFI は独立した `*-sys` crate にカプセル化 |
| **互換性** | Rust 1.80+ MSRV、stable を追従 |

## 設計原則

1. **Beego の哲学を Rust で表現** — MVC、ネームスペース、フィルタチェーンを trait + macro で実装
2. **段階的拡張** — 最小コアは axum + tokio のみに依存し、それ以外はすべて feature gate
3. **暗黙より明示** — ルート登録、モデルマッピング、ミドルウェアの順序をすべてコードで明示的に宣言
4. **ゼロコスト抽象** — trait の静的ディスパッチ、macro のコンパイル時展開により、仮想関数のオーバーヘッドを持ち込まない
5. **ストレージエンジンの独立性** — 各エンジンの trait は実装を個別に差し替え可能で、上位の業務に影響を与えない
6. **可観測性の組み込み** — tracing + metrics の計装がフレームワーク全体をカバーし、構造化ログはデフォルトで有効

## アーキテクチャ設計

### Crate トポロジー

```
bee_rust/           # メタ crate、re-export + feature flags
bee_router/         # ルーティング + コントローラ + Context + フィルタチェーン
bee_orm/            # ORM — Model trait + QuerySet + Migration + リレーションマッピング
bee_kv/             # KV/Cache 統一抽象 — Redis + Memcached
bee_search/         # 検索/分析エンジン — Elasticsearch + OpenSearch + ClickHouse
bee_graph/          # グラフデータベース — Neo4j + NebulaGraph + ArangoDB
bee_tsdb/           # 時系列データベース — InfluxDB + Apache IoTDB + QuestDB
bee_config/         # 設定管理 — INI/YAML/ENV + ホットリロード
bee_cache/          # キャッシュ抽象 — Memory/Redis/Memcache
bee_session/        # Session — Memory/Redis/Cookie/Database バックエンド
bee_logs/           # ログ — 多段階ログ + tracing 統合
bee_template/       # テンプレートレンダリング — tera ベース
bee_cli/            # CLI — スキャフォールディング/コード生成/開発実行/パッケージング（移行計画中）
```

### アーキテクチャ図

```
                          ┌───────────────────────┐
                          │  bee_rust  (meta)      │
                          │  re-export + features  │
                          └───────────┬───────────┘
                                      │
              ┌───────────────────────┼───────────────────────┐
              │                       │                       │
     ┌────────▼────────┐    ┌────────▼────────┐    ┌────────▼────────┐
     │    Web Layer     │    │   Data Layer    │    │   Tool Layer    │
     └────────┬────────┘    └────────┬────────┘    └────────┬────────┘
              │                       │                       │
  ┌───────────┼───────────┐  ┌───────┼───────┐  ┌───────────┼───────────┐
  │ bee_router            │  │ bee_orm        │  │ bee_cli               │
  │  - route register     │  │  - Model/Query  │  │  - scaffolding        │
  │  - controller trait   │  │  - Migration   │  │  - hot reload          │
  │  - filter chain       │  │  - Connection   │  │  - code generation    │
  │  - param extract      │  │                 │  │                       │
  ├────────────────────────┤  ├────────────────┤  ├───────────────────────┤
  │ bee_template           │  │ bee_config     │  │ bee_logs              │
  │  - template render     │  │  - INI/YAML/ENV│  │  - multi-level log    │
  │  - HTML/JSON           │  │  - hot reload   │  │  - tracing integrate  │
  ├────────────────────────┤  ├────────────────┤  └───────────────────────┘
  │ bee_session            │  │ bee_cache      │
  │  - session management  │  │  - cache trait  │
  │  - multi-backend       │  │  - Mem/Redis    │
  └────────────────────────┘  └────────────────┘

  ┌─────────────────────────────────────────────────────────┐
  │                   Storage Engine Layer                   │
  ├──────────────────┬──────────────────────────────────────┤
  │ bee_kv           │  Redis + Memcached                   │
  │ bee_search       │  Elasticsearch + OpenSearch + ClickHouse │
  │ bee_graph        │  Neo4j + NebulaGraph + ArangoDB      │
  │ bee_tsdb         │  InfluxDB + Apache IoTDB + QuestDB   │
  └──────────────────┴──────────────────────────────────────┘
```

### Crate 依存関係

```
bee_config (依存なし)
bee_logs   (依存なし)
bee_cache  → bee_config
bee_kv     → bee_config
bee_session → bee_cache, bee_config
bee_template (依存なし)
bee_orm    → bee_config, bee_cache
bee_search → bee_config
bee_graph  → bee_config
bee_tsdb   → bee_config
bee_router → bee_session, bee_template, bee_config, bee_logs
bee_cli    → bee_router, bee_orm
bee_rust   → 上記すべての crate (re-export)
```

### 対応データベース

| カテゴリ | データベース | 対応 Crate | Feature Flag |
|------|--------|-----------|-------------|
| **リレーショナル** | SQLite | `bee_orm` | `sqlite` |
| | PostgreSQL | `bee_orm` | `postgres` |
| | MySQL | `bee_orm` | `mysql` |
| | TiDB | `bee_orm` | `mysql` |
| **KV / キャッシュ** | Redis | `bee_kv` / `bee_cache` | `redis` |
| | Memcached | `bee_kv` / `bee_cache` | `memcache` |
| **検索 / 分析** | Elasticsearch | `bee_search` | `elasticsearch` |
| | OpenSearch | `bee_search` | `opensearch` |
| | ClickHouse | `bee_search` | `clickhouse` |
| **グラフデータベース** | Neo4j | `bee_graph` | `neo4j` |
| | NebulaGraph | `bee_graph` | `nebulagraph` |
| | ArangoDB | `bee_graph` | `arangodb` |
| **時系列データベース** | InfluxDB | `bee_tsdb` | `influxdb` |
| | Apache IoTDB | `bee_tsdb` | `iotdb` |
| | QuestDB | `bee_tsdb` | `questdb` |

### リクエストフィルタチェーン

```
リクエスト → [SecurityFilter 攻撃検知] → [Session 復元] → [パラメータ検証] → [prepare フック] → [handle 処理] → [finish フック] → レスポンス
                  ↓ どの段階でも中断可能（Beego の Abort に類似）
```

## 機能紹介

機能の概要は以下の通りです。各モジュールの詳細な使い方、コード例、API の説明は [API リファレンス](api.ja.md) を参照してください。

### Web コア（bee_router）

MVC コントローラ：`Controller` trait + `Context`。ルートネームスペース、RESTful メソッド登録、リクエストフィルタチェーンをサポート。

- レスポンス出力：`ctx.json()` / `ctx.text()` / `ctx.html()`
- リダイレクト / 中断：`ctx.redirect()` / `ctx.abort()`
- セッションとパラメータ：`ctx.session` / `ctx.params`

### セキュリティ検知（`security` feature）

[security-rust](https://crates.io/crates/security-rust) に基づく攻撃検知フィルタ。XSS、SQL インジェクション、コマンドインジェクション、SSRF など 27 種類の攻撃タイプをカバーし、一行で有効化できます：

```rust
let security = SecurityFilter::new();  // 27 個の検出器をすべて有効化
```

### ORM（bee_orm）

`#[derive(Model)]` 派生マクロ + QuerySet のチェーン式クエリ（filter / order_by / limit）。SQLite、PostgreSQL、MySQL、TiDB をサポート。

### 設定管理（bee_config）

`#[derive(Config)]` 派生マクロ。INI / YAML / ENV の読み込みとホットリロードをサポート。

### ストレージエンジン

KV / Cache（Redis + Memcached）、検索エンジン（Elasticsearch / OpenSearch / ClickHouse）、グラフデータベース（Neo4j / NebulaGraph / ArangoDB）、時系列データベース（InfluxDB / IoTDB / QuestDB）を統一 trait で抽象化し、ドライバは feature gate に応じてコンパイルされます。

### Session、ログ、テンプレート

- Session：Memory / Redis / Cookie / Database のマルチバックエンド
- ログ：多段階ログ + tracing 統合
- テンプレート：tera ベースのレンダリング

### CLI ツール

```bash
bee-rust new my-app            # プロジェクトのスキャフォールディングを作成
bee-rust generate controller user
bee-rust run --watch           # 開発実行（ホットリロード）
bee-rust pack                  # パッケージングしてデプロイ
```

## 使い方

### 環境要件

- Rust 1.80+
- Cargo

### インストール

```bash
# プロジェクトをクローン
git clone https://github.com/erikwang2013/bee-rust.git
cd bee-rust

# コンパイル
cargo build --workspace

# テストを実行
cargo test --workspace
```

### クイックスタート

```bash
# CLI で新しいプロジェクトを作成
cargo run -p bee_cli -- new hello
cd hello

# 開発サーバーを起動
cargo run
```

### プロジェクトでの利用

```toml
[dependencies]
bee_rust = { git = "https://github.com/erikwang2013/bee-rust", features = ["full"] }
```

## 技術詳細

### 技術スタック

| レイヤー | 技術 |
|----|------|
| HTTP 基盤 | axum 0.8 + tower 0.5 |
| 非同期ランタイム | tokio 1.x |
| シリアライズ | serde + serde_json |
| テンプレートエンジン | tera 1.x |
| ログ基盤 | tracing + tracing-subscriber |
| CLI | clap 4 |
| 設定パース | toml / serde_yaml / 自前の INI |
| エラーハンドリング | thiserror |
| 手続きマクロ | syn + quote + proc-macro2 |

### デザインパターン

| パターン | 適用 |
|------|------|
| **Builder** | Logger, Router, QuerySet |
| **Trait 抽象** | Cache, KvStore, SearchEngine, GraphDB, TimeSeriesDB |
| **派生マクロ** | `#[derive(Model)]`, `#[derive(Config)]` |
| **Feature Gate** | ドライバ実装を必要に応じてコンパイル（redis、memcached、elasticsearch など） |
| **Filter Chain** | リクエストフィルタチェーン、Beego Filter に相当 |

### Crate 一覧

| Crate | 機能 | Beego との対応 |
|-------|------|-----------|
| `bee_rust` | メタ crate、統合エントリポイント | — |
| `bee_router` | ルーティング + コントローラ + Context + フィルタ | `server/web`, `context` |
| `bee_orm` | ORM + QuerySet + Migration | `client/orm` |
| `bee_kv` | KV/Cache 統一抽象 | `client/cache`（拡張） |
| `bee_search` | 検索/分析エンジン | —（新規） |
| `bee_graph` | グラフデータベース | —（新規） |
| `bee_tsdb` | 時系列データベース | —（新規） |
| `bee_config` | 設定管理 + ホットリロード | `client/config` |
| `bee_cache` | キャッシュ抽象 | `client/cache` |
| `bee_session` | Session 管理 | `server/web/session` |
| `bee_logs` | ログ | `logs` |
| `bee_template` | テンプレートレンダリング | —（拡張） |
| `bee_cli` | CLI ツール | `bee` ツール |

### テストカバレッジ

リポジトリ全体で 68 個のテストがパス：

| Crate | テスト数 |
|-------|--------|
| bee_config | 4 |
| bee_cache | 4 |
| bee_template | 2 |
| bee_logs | 3 |
| bee_kv | 4 |
| bee_search | 6 |
| bee_graph | 5 |
| bee_tsdb | 5 |
| bee_orm | 7 |
| bee_session | 2 |
| bee_router | 9 |
| bee_cli | 16 |

## サポート歓迎

このプロジェクトがお役に立てたなら、QR コードをスキャンして投げ銭でサポートしていただけると嬉しいです。ありがとうございます！

**WeChat Pay**

<img src="weixinpay.png" width="160" height="175" alt="WeChat Pay">

**Alipay**

<img src="alipay.png" width="160" height="175" alt="Alipay">

**海外送金（Bank Transfer）**

海外のユーザーは銀行振込でサポートできます：

**受取人情報**

| 項目 | 内容 |
|------|------|
| 受取人名 | WANG KEXUN |
| 受取口座番号 | 881015918251 |

**受取銀行**

| 項目 | 内容 |
|------|------|
| SWIFT Code | AABLHKHHXXX |
| 銀行名 | ZA Bank Limited |
| 銀行番号 | 387 |
| 銀行住所 | Core F, Cyberport 3, 100 Cyberport Road, Hong Kong |

**クロスボーダー送金の代理銀行（必要な場合）**

> 注意：これはクロスボーダー送金の代理銀行（中継銀行）の情報であり、受取銀行の情報ではありません。あなたの送金銀行に提供が必要かどうかお問い合わせください。

- **香港ドル、人民元、米ドルの送金**（代理銀行は Citibank）：

| 項目 | 内容 |
|------|------|
| 銀行名 | Citibank N.A. Hong Kong |
| SWIFT Code | CITIHKHXXXX |
| 銀行番号 | 006 |
| 支店名 | Hong Kong Branch |
| 支店番号 | 391 |
| 銀行住所 | Citibank Tower, Citibank Plaza, 3 Garden Road, Central, Hong Kong |

- **その他の通貨の送金**（代理銀行は BNY Mellon）：

| 項目 | 内容 |
|------|------|
| 銀行名 | THE BANK OF NEW YORK MELLON |
| SWIFT Code | IRVTUS3NXXX |
| 銀行住所 | THE BANK OF NEW YORK MELLON, 240 GREENWICH STREET, NEW YORK, United States |

### ライセンス

Apache-2.0
