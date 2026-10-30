# 変更履歴

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

## [1.0.6] — 2026-08-07

### 追加
- `bee_cli` の実装：`new`（プロジェクトのスキャフォールディング）、`generate controller/model`、`--watch` によるホットリロード付き `run`、`pack`（release ビルド + `dist/` へのコピー）
- スキャフォールディングとコード生成の CLI ユニットテスト（新規 7 テスト）

### 修正
- `bee_rust::init()` を `logs` feature の背後にゲート——feature を絞ったビルド（例：`--no-default-features --features kv`）が再びコンパイル可能に
- `bee_kv::InMemoryKvStore::exists` の Clippy `unnecessary_map_or` lint
- `rustfmt.toml` から stable で黙って無視されていた nightly 専用オプションを削除。ワークスペースは `cargo fmt --all --check` をパスするように
- `bee_cli` バイナリに `doc = false` を設定し、`bee_rust` との rustdoc 出力ファイル名の衝突を解消
- `hello` サンプルのポートが `PORT` 環境変数で設定可能に

### 変更
- `bee-rust migrate` が「未実装」を報告し、非ゼロで終了するように（計画中）
- README / README.en を実際の CLI の動作に合わせて更新

## [1.0.4] — 2026-07-29

### 追加
- `security-rust` によるセキュリティ攻撃検知フィルタ（27 検出器）
- XSS、SQL インジェクション、コマンドインジェクション、パストラバーサルをカバーする `SecurityFilter`
- `bee_rust` と `bee_router` に `security` feature フラグ

### 変更
- README にセキュリティ feature のドキュメントを追加
- README に投げ銭サポートのセクションを追加（WeChat Pay / Alipay）

### 修正
- Rust 2024 edition 向けの `bee_template` の Tera 生識別子構文

## [1.0.3] — 2026-07-29

### 追加
- 13 個の crate からなる初期ワークスペース構造
- `Controller` trait と `Router` による MVC ルーティング
- `QuerySet` ビルダーと `Model` 派生マクロを備えた ORM
- Redis と Memory バックエンドを備えた KV/Cache の trait 抽象
- Memory/Redis バックエンドによるセッション管理
- INI/YAML/ENV 対応とホットリロードを備えた設定管理
- Tera によるテンプレートレンダリング
- tracing 統合によるログ
- CLI のスキャフォールディングとコード生成
- 検索、グラフ、時系列エンジンの trait スタブ（ドライバは計画中）

[1.0.4]: https://github.com/erikwang2013/bee-rust/compare/v1.0.3...v1.0.4
[1.0.3]: https://github.com/erikwang2013/bee-rust/releases/tag/v1.0.3
