# コントリビューション

[简体中文](CONTRIBUTING.zh.md) · [English](CONTRIBUTING.md) · [한국어](CONTRIBUTING.ko.md) · [Русский](CONTRIBUTING.ru.md) · [Deutsch](CONTRIBUTING.de.md) · [Français](CONTRIBUTING.fr.md) · [Español](CONTRIBUTING.es.md) · [Português](CONTRIBUTING.pt.md) · [हिन्दी](CONTRIBUTING.hi.md) · [العربية](CONTRIBUTING.ar.md) · [বাংলা](CONTRIBUTING.bn.md) · [Bahasa Indonesia](CONTRIBUTING.id.md) · [日本語](CONTRIBUTING.ja.md)

## セットアップ

```bash
git clone https://github.com/erikwang2013/bee-rust.git
cd bee-rust
cargo build --workspace
cargo test --workspace
```

## 提出前の確認

- コードを整形するには `cargo fmt --all` を実行
- lint するには `cargo clippy --workspace -- -D warnings` を実行
- すべてのテストがパスすることを確認するには `cargo test --workspace` を実行
- ファイルは 500 行以内に保つ

## プロジェクト構造

```
crates/
  bee_rust/         # メタ crate、re-export + feature flags
  bee_router/       # ルーティング + コントローラ + Context + フィルタチェーン
  bee_orm/          # ORM — Model trait + QuerySet + Migration
  bee_kv/           # KV/Cache 統一抽象
  bee_search/       # 検索/分析エンジン
  bee_graph/        # グラフデータベース
  bee_tsdb/         # 時系列データベース
  bee_config/       # 設定管理 + ホットリロード
  bee_cache/        # キャッシュ抽象
  bee_session/      # セッション管理
  bee_logs/         # ログ
  bee_template/     # テンプレートレンダリング
  bee_cli/          # CLI ツール
```

## ライセンス

Apache-2.0
