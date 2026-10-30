# Mitwirken

[简体中文](CONTRIBUTING.zh.md) · [English](CONTRIBUTING.md) · [한국어](CONTRIBUTING.ko.md) · [Русский](CONTRIBUTING.ru.md) · [Deutsch](CONTRIBUTING.de.md) · [Français](CONTRIBUTING.fr.md) · [Español](CONTRIBUTING.es.md) · [Português](CONTRIBUTING.pt.md) · [हिन्दी](CONTRIBUTING.hi.md) · [العربية](CONTRIBUTING.ar.md) · [বাংলা](CONTRIBUTING.bn.md) · [Bahasa Indonesia](CONTRIBUTING.id.md) · [日本語](CONTRIBUTING.ja.md)

## Einrichtung

```bash
git clone https://github.com/erikwang2013/bee-rust.git
cd bee-rust
cargo build --workspace
cargo test --workspace
```

## Vor dem Einreichen

- Führe `cargo fmt --all` aus, um den Code zu formatieren
- Führe `cargo clippy --workspace -- -D warnings` aus, um zu linten
- Führe `cargo test --workspace` aus, um sicherzustellen, dass alle Tests bestehen
- Halte Dateien unter 500 Zeilen

## Projektstruktur

```
crates/
  bee_rust/         # Meta-Crate, re-export + feature flags
  bee_router/       # Routing + Controller + Context + Filterkette
  bee_orm/          # ORM — Model-Trait + QuerySet + Migration
  bee_kv/           # Vereinheitlichte KV/Cache-Abstraktion
  bee_search/       # Such-/Analyse-Engine
  bee_graph/        # Graphdatenbank
  bee_tsdb/         # Zeitreihendatenbank
  bee_config/       # Konfigurationsverwaltung + Hot-Reload
  bee_cache/        # Cache-Abstraktion
  bee_session/      # Session-Verwaltung
  bee_logs/         # Logging
  bee_template/     # Template-Rendering
  bee_cli/          # CLI-Werkzeuge
```

## Lizenz

Apache-2.0
