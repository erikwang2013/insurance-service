# Changelog

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

## [1.0.6] — 2026-08-07

### Neu hinzugefügt
- `bee_cli` echte Implementierungen: `new` (Projekt-Scaffolding), `generate controller/model`, `run` mit `--watch`-Hot-Reload, `pack` (Release-Build + Kopieren nach `dist/`)
- CLI-Unit-Tests für Scaffolding und Codegenerierung (7 neue Tests)

### Behoben
- `bee_rust::init()` ist jetzt hinter dem `logs`-Feature abgesperrt — reduzierte Feature-Builds (z. B. `--no-default-features --features kv`) kompilieren wieder
- Clippy-Lint `unnecessary_map_or` in `bee_kv::InMemoryKvStore::exists`
- `rustfmt.toml`: nur-Nightly-Optionen entfernt, die auf stable stillschweigend ignoriert wurden; der Workspace besteht jetzt `cargo fmt --all --check`
- `bee_cli`-Binary mit `doc = false`, um die Kollision der rustdoc-Ausgabedatei mit `bee_rust` zu beheben
- Der Port des `hello`-Beispiels ist jetzt über die Umgebungsvariable `PORT` konfigurierbar

### Geändert
- `bee-rust migrate` meldet „nicht implementiert" und endet mit einem Exit-Code ungleich null (geplant)
- README / README.en aktualisiert, um das tatsächliche CLI-Verhalten zu beschreiben

## [1.0.4] — 2026-07-29

### Neu hinzugefügt
- Sicherheitsfilter zur Angriffserkennung über `security-rust` (27 Detektoren)
- `SecurityFilter` mit Abdeckung von XSS, SQL-Injection, Command-Injection und Path-Traversal
- `security`-Feature-Flag in `bee_rust` und `bee_router`

### Geändert
- README um die Dokumentation des Sicherheitsfeatures erweitert
- README um einen Abschnitt zur Zahlungsunterstützung erweitert (WeChat Pay / Alipay)

### Behoben
- Tera-Syntax für rohe Bezeichner in `bee_template` für die Rust-2024-Edition

## [1.0.3] — 2026-07-29

### Neu hinzugefügt
- Initiale Workspace-Struktur mit 13 Crates
- MVC-Routing mit `Controller`-Trait und `Router`
- ORM mit `QuerySet`-Builder und `Model`-Derivierungsmakro
- Trait-Abstraktion für KV/Cache mit Redis- und Memory-Backends
- Session-Verwaltung mit Memory/Redis-Backends
- Konfigurationsverwaltung mit INI/YAML/ENV-Unterstützung und Hot-Reload
- Template-Rendering über Tera
- Logging mit tracing-Integration
- CLI-Scaffolding und Codegenerierung
- Trait-Stubs für Such-, Graph- und Zeitreihen-Engines (Treiber geplant)

[1.0.4]: https://github.com/erikwang2013/bee-rust/compare/v1.0.3...v1.0.4
[1.0.3]: https://github.com/erikwang2013/bee-rust/releases/tag/v1.0.3
