# Contribuer

[简体中文](CONTRIBUTING.zh.md) · [English](CONTRIBUTING.md) · [한국어](CONTRIBUTING.ko.md) · [Русский](CONTRIBUTING.ru.md) · [Deutsch](CONTRIBUTING.de.md) · [Français](CONTRIBUTING.fr.md) · [Español](CONTRIBUTING.es.md) · [Português](CONTRIBUTING.pt.md) · [हिन्दी](CONTRIBUTING.hi.md) · [العربية](CONTRIBUTING.ar.md) · [বাংলা](CONTRIBUTING.bn.md) · [Bahasa Indonesia](CONTRIBUTING.id.md) · [日本語](CONTRIBUTING.ja.md)

## Préparation

```bash
git clone https://github.com/erikwang2013/bee-rust.git
cd bee-rust
cargo build --workspace
cargo test --workspace
```

## Avant de soumettre

- Exécutez `cargo fmt --all` pour formater le code
- Exécutez `cargo clippy --workspace -- -D warnings` pour l'analyse statique (lint)
- Exécutez `cargo test --workspace` pour vérifier que tous les tests passent
- Gardez les fichiers sous 500 lignes

## Structure du projet

```
crates/
  bee_rust/         # Méta-crate, re-export + feature flags
  bee_router/       # Routage + contrôleur + Context + chaîne de filtres
  bee_orm/          # ORM — trait Model + QuerySet + Migration
  bee_kv/           # Abstraction unifiée KV/Cache
  bee_search/       # Moteur de recherche/analyse
  bee_graph/        # Base de données graphe
  bee_tsdb/         # Base de données de séries temporelles
  bee_config/       # Gestion de configuration + rechargement à chaud
  bee_cache/        # Abstraction de cache
  bee_session/      # Gestion de session
  bee_logs/         # Journalisation
  bee_template/     # Rendu de templates
  bee_cli/          # Outillage CLI
```

## Licence

Apache-2.0
