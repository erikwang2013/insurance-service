# Journal des modifications

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

## [1.0.6] — 2026-08-07

### Ajouts
- Implémentations réelles de `bee_cli` : `new` (scaffolding de projet), `generate controller/model`, `run` avec rechargement à chaud `--watch`, `pack` (build release + copie dans `dist/`)
- Tests unitaires du CLI pour le scaffolding et la génération de code (7 nouveaux tests)

### Corrections
- `bee_rust::init()` est désormais derrière la feature `logs` — les builds avec des features réduites (p. ex. `--no-default-features --features kv`) compilent à nouveau
- Lint Clippy `unnecessary_map_or` dans `bee_kv::InMemoryKvStore::exists`
- `rustfmt.toml` : suppression des options réservées à nightly, silencieusement ignorées sur stable ; le workspace passe désormais `cargo fmt --all --check`
- Binaire `bee_cli` : `doc = false` pour supprimer la collision de nom de fichier de sortie rustdoc avec `bee_rust`
- Le port de l'exemple `hello` est désormais configurable via la variable d'environnement `PORT`

### Modifications
- `bee-rust migrate` signale "not implemented" et se termine avec un code de sortie non nul (prévu)
- README / README.en mis à jour pour décrire le comportement réel du CLI

## [1.0.4] — 2026-07-29

### Ajouts
- Filtre de détection d'attaques via `security-rust` (27 détecteurs)
- `SecurityFilter` couvrant XSS, injection SQL, injection de commandes et traversée de chemins
- Feature flag `security` dans `bee_rust` et `bee_router`

### Modifications
- README mis à jour avec la documentation de la feature de sécurité
- README mis à jour avec la section de soutien par paiement (WeChat Pay / Alipay)

### Corrections
- Syntaxe des identifiants bruts Tera dans `bee_template` pour l'édition Rust 2024

## [1.0.3] — 2026-07-29

### Ajouts
- Structure initiale du workspace avec 13 crates
- Routage MVC avec le trait `Controller` et `Router`
- ORM avec le builder `QuerySet` et la macro dérivée `Model`
- Abstraction par traits KV/Cache avec les backends Redis et Memory
- Gestion de session avec les backends Memory/Redis
- Gestion de configuration avec prise en charge INI/YAML/ENV et rechargement à chaud
- Rendu de templates via Tera
- Journalisation avec intégration tracing
- Scaffolding et génération de code via le CLI
- Stubs de traits pour les moteurs de recherche, graphe et séries temporelles (drivers prévus)

[1.0.4]: https://github.com/erikwang2013/bee-rust/compare/v1.0.3...v1.0.4
[1.0.3]: https://github.com/erikwang2013/bee-rust/releases/tag/v1.0.3
