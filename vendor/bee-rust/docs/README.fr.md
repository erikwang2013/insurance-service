<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Beerust

[简体中文](../README.md) · [English](README.en.md) · [한국어](README.ko.md) · [Русский](README.ru.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Português](README.pt.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [বাংলা](README.bn.md) · [Bahasa Indonesia](README.id.md) · [日本語](README.ja.md)

Beerust est un framework Web de niveau production écrit en Rust, dont la philosophie de conception s'inspire du framework Beego de Go, réexprimée avec les idiomes Rust : traits, macros et système de types.

## La mascotte du projet : Rusty

<img src="rusty.svg" width="360" alt="Rusty en vol stationnaire de face, ailes déployées en anneau, les pattes avant serrant sa caisse à outils rouille-orange">

### Fiche du personnage

**Nom** Rusty — la couleur rouille de Rust, et aussi la trace de rouille accrochée à sa salopette.

**Apparence** : une abeille ingénieur toute pelucheuse. Un pelage ambre doré recouvre son abdomen rondouillard, traversé de trois rayures noir de charbon comme des bretelles de salopette mises de travers ; quatre ailes translucides si fines qu'elles laissent passer la lumière, laissant une traînée fantôme en vol stationnaire ; les yeux composés sont deux billes d'obsidienne comme des billes de verre, chacune portant un reflet ; ses pattes avant serrent une caisse à outils rouille-orange usée et brillante — c'est sa corbeille à pollen, qui ne contient jamais de pollen mais des dépendances.

**Caractère** : bourreau de travail, optimiste, légèrement obsessionnel.

- Quand il répare la ruche, il inspecte cellule par cellule avec ses antennes, aussi pointilleux qu'un lint
- Les dépendances de sa corbeille à pollen sont verrouillées version par version ; malheur à qui y touche
- Quand il traque un bug, il reste suspendu immobile dans les airs, ses ailes si rapides qu'elles ne forment plus qu'un halo
- Le jour d'une nouvelle release, il exécute la danse frétillante du 8 devant la porte de la ruche

### Charte visuelle

| Élément | Spécification |
|------|------|
| Corps | Ambre doré `#F5B301`, pelage abdominal court et dense, liseré lumineux sur les bords |
| Rayures | Noir de charbon `#1F1A17`, trois, uniquement sur l'abdomen |
| Ailes | Bleu clair translucide `#CFE8F5`, opacité 40 %, nervures dessinées |
| Yeux composés | Brun-noir profond `#2A1E16`, elliptiques, un reflet dans le coin supérieur gauche |
| Touches | Orange rouille `#B7410E` (caisse à outils, nœud papillon), la rouille est sa signature |

### Expressions

| Expression | Visuel | Quand |
|------|------|---------|
| Plein d'énergie | `(^o^)` pelage hérissé, ailes vibrant à haute fréquence | quand tous les tests passent |
| Debug en vol stationnaire | `o(°▽°)o` suspendu dans les airs, seules des traînées d'ailes subsistent | quand il traque un bug tenace |
| Retour chargé | `(≧▽≦)` caisse à outils bien pleine, vol en zigzag | quand un nouveau driver est intégré avec succès |
| Somnolent | `(-_-)zZ` accroupi devant la ruche, antennes affaissées | quand le trafic est nul tard dans la nuit |
| Hérissé | `(#°益°)` dard dressé | quand le filtre de sécurité bloque une attaque |

### Gestes signatures

1. **Debug en vol stationnaire** — dès qu'il repère un bug, il se suspend au-dessus du code, ailes vrombissant en un halo, et il fixe le code jusqu'à ce que tu aies corrigé.
2. **Danse frétillante du 8** — à chaque nouvelle release, il fait un tour complet autour de la ruche pour annoncer à toute la colonie « la fleur est en fleur ». Les vraies abeilles s'en servent pour transmettre l'emplacement des sources de nectar ; Rusty s'en sert pour transmettre le changelog.
3. **Toilettage** — il frotte ses antennes avec ses pattes avant. Tu crois qu'il fait son coquet ; en réalité il vérifie si la configuration a été rechargée à chaud.

### Là où on le trouve

- **Logo** : posture de vol stationnaire de face, ailes déployées en anneau, entourant exactement le nom.
- **Page 404** : suspendu au-dessus d'un champ de fleurs vide, tournant en rond, perdu — « Et le miel ? »
- **CLI** : une rangée de petites abeilles sur la bannière de démarrage, accroupies à côté pour surveiller pendant les tests.
- **Annonces de release** : celle qui exécute la danse du 8.

## Objectifs de conception

| Objectif | Indicateur |
|------|------|
| **Expérience de développement** | de `bee-rust new` à la première requête en moins de 30 s |
| **Performance** | surcoût de la couche contrôleur < 5 % (par rapport à axum nu), latence P99 des routes < 100 µs |
| **Vitesse de compilation** | compilation complète du méta-crate < 60 s (release), compilation incrémentale < 5 s |
| **Taille du binaire** | application minimale (router seul) < 5 Mo (strip + LTO) |
| **Sécurité** | zéro code métier `unsafe` ; tout le FFI encapsulé dans des crates `*-sys` dédiés |
| **Compatibilité** | MSRV Rust 1.80+, suivi de la branche stable |

## Principes de conception

1. **Philosophie Beego, expression Rust** — MVC, espaces de noms, chaîne de filtres, implémentés avec des traits + macros
2. **Amélioration progressive** — le cœur minimal ne dépend que d'axum + tokio, tout le reste est derrière des feature gates
3. **L'explicite plutôt que l'implicite** — enregistrement des routes, mappage des modèles, ordre des middlewares : tout est déclaré explicitement dans le code
4. **Abstractions à coût zéro** — dispatch statique des traits, expansion des macros à la compilation, sans surcoût de fonctions virtuelles
5. **Indépendance des moteurs de stockage** — l'implémentation de chaque trait de moteur peut être remplacée indépendamment sans affecter la couche métier
6. **Observabilité intégrée** — instrumentation tracing + metrics couvrant tout le framework, logs structurés activés par défaut

## Architecture

### Topologie des crates

```
bee_rust/           # méta-crate, re-export + feature flags
bee_router/         # routage + contrôleur + Context + chaîne de filtres
bee_orm/            # ORM — trait Model + QuerySet + Migration + mappage des relations
bee_kv/             # abstraction unifiée KV/Cache — Redis + Memcached
bee_search/         # moteur de recherche/analyse — Elasticsearch + OpenSearch + ClickHouse
bee_graph/          # base de données graphe — Neo4j + NebulaGraph + ArangoDB
bee_tsdb/           # base de données de séries temporelles — InfluxDB + Apache IoTDB + QuestDB
bee_config/         # gestion de configuration — INI/YAML/ENV + rechargement à chaud
bee_cache/          # abstraction de cache — Memory/Redis/Memcache
bee_session/        # Session — backends Memory/Redis/Cookie/Database
bee_logs/           # journalisation — logs multi-niveaux + intégration tracing
bee_template/       # rendu de templates — basé sur tera
bee_cli/            # CLI — scaffolding/génération de code/exécution de dev/empaquetage (migration prévue)
```

### Schéma d'architecture

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

### Dépendances entre crates

```
bee_config (aucune dépendance)
bee_logs   (aucune dépendance)
bee_cache  → bee_config
bee_kv     → bee_config
bee_session → bee_cache, bee_config
bee_template (aucune dépendance)
bee_orm    → bee_config, bee_cache
bee_search → bee_config
bee_graph  → bee_config
bee_tsdb   → bee_config
bee_router → bee_session, bee_template, bee_config, bee_logs
bee_cli    → bee_router, bee_orm
bee_rust   → tous les crates ci-dessus (re-export)
```

### Bases de données prises en charge

| Catégorie | Base de données | Crate correspondant | Feature Flag |
|------|--------|-----------|-------------|
| **Relationnelles** | SQLite | `bee_orm` | `sqlite` |
| | PostgreSQL | `bee_orm` | `postgres` |
| | MySQL | `bee_orm` | `mysql` |
| | TiDB | `bee_orm` | `mysql` |
| **KV / cache** | Redis | `bee_kv` / `bee_cache` | `redis` |
| | Memcached | `bee_kv` / `bee_cache` | `memcache` |
| **Recherche / analyse** | Elasticsearch | `bee_search` | `elasticsearch` |
| | OpenSearch | `bee_search` | `opensearch` |
| | ClickHouse | `bee_search` | `clickhouse` |
| **Bases de données graphe** | Neo4j | `bee_graph` | `neo4j` |
| | NebulaGraph | `bee_graph` | `nebulagraph` |
| | ArangoDB | `bee_graph` | `arangodb` |
| **Séries temporelles** | InfluxDB | `bee_tsdb` | `influxdb` |
| | Apache IoTDB | `bee_tsdb` | `iotdb` |
| | QuestDB | `bee_tsdb` | `questdb` |

### Chaîne de filtres de requêtes

```
Requête → [SecurityFilter détection d'attaque] → [Restauration de session] → [Validation des paramètres] → [hook prepare] → [traitement handle] → [hook finish] → Réponse
                  ↓ Interruptible à tout moment (comme l'Abort de Beego)
```

## Fonctionnalités

Un aperçu des fonctionnalités suit ; pour les usages détaillés de chaque module, les exemples de code et les explications de l'API, voir la [Référence API](api.fr.md).

### Cœur Web (bee_router)

Contrôleurs MVC : le trait `Controller` + `Context`, avec espaces de noms de routes, enregistrement de méthodes RESTful et chaîne de filtres de requêtes.

- Sortie de réponse : `ctx.json()` / `ctx.text()` / `ctx.html()`
- Redirection / interruption : `ctx.redirect()` / `ctx.abort()`
- Session et paramètres : `ctx.session` / `ctx.params`

### Détection de sécurité (feature `security`)

Un filtre de détection d'attaques basé sur [security-rust](https://crates.io/crates/security-rust), couvrant 27 types d'attaques dont XSS, injection SQL, injection de commandes et SSRF, activé en une ligne :

```rust
let security = SecurityFilter::new();  // les 27 détecteurs sont tous activés
```

### ORM (bee_orm)

Macro dérivée `#[derive(Model)]` + requêtes chaînées QuerySet (filter / order_by / limit), avec prise en charge de SQLite, PostgreSQL, MySQL et TiDB.

### Gestion de configuration (bee_config)

Macro dérivée `#[derive(Config)]`, prenant en charge le chargement INI / YAML / ENV et le rechargement à chaud.

### Moteurs de stockage

Abstractions par traits unifiés pour KV / Cache (Redis + Memcached), moteurs de recherche (Elasticsearch / OpenSearch / ClickHouse), bases de données graphe (Neo4j / NebulaGraph / ArangoDB) et séries temporelles (InfluxDB / IoTDB / QuestDB) ; les drivers sont compilés selon les feature gates.

### Session, journalisation, templates

- Session : backends multiples Memory / Redis / Cookie / Database
- Journalisation : logs multi-niveaux + intégration tracing
- Templates : rendu basé sur tera

### Outil CLI

```bash
bee-rust new my-app            # crée le scaffolding du projet
bee-rust generate controller user
bee-rust run --watch           # exécution de développement (rechargement à chaud)
bee-rust pack                  # empaquetage et déploiement
```

## Mise en route

### Prérequis

- Rust 1.80+
- Cargo

### Installation

```bash
# Clonez le dépôt
git clone https://github.com/erikwang2013/bee-rust.git
cd bee-rust

# Compilation
cargo build --workspace

# Exécution des tests
cargo test --workspace
```

### Démarrage rapide

```bash
# Créez un nouveau projet avec le CLI
cargo run -p bee_cli -- new hello
cd hello

# Lancez le serveur de développement
cargo run
```

### Utilisation dans un projet

```toml
[dependencies]
bee_rust = { git = "https://github.com/erikwang2013/bee-rust", features = ["full"] }
```

## Notes techniques

### Pile technique

| Couche | Technologie |
|----|------|
| Base HTTP | axum 0.8 + tower 0.5 |
| Runtime asynchrone | tokio 1.x |
| Sérialisation | serde + serde_json |
| Moteur de templates | tera 1.x |
| Sous-système de journalisation | tracing + tracing-subscriber |
| CLI | clap 4 |
| Analyse de configuration | toml / serde_yaml / INI maison |
| Gestion des erreurs | thiserror |
| Macros procédurales | syn + quote + proc-macro2 |

### Design patterns

| Pattern | Application |
|------|------|
| **Builder** | Logger, Router, QuerySet |
| **Abstraction par traits** | Cache, KvStore, SearchEngine, GraphDB, TimeSeriesDB |
| **Macros dérivées** | `#[derive(Model)]`, `#[derive(Config)]` |
| **Feature Gate** | implémentations des drivers compilées à la demande (redis, memcached, elasticsearch, etc.) |
| **Chaîne de filtres** | chaîne de filtres de requêtes, équivalent du Beego Filter |

### Inventaire des crates

| Crate | Fonction | Équivalent Beego |
|-------|------|-----------|
| `bee_rust` | méta-crate, point d'entrée unifié | — |
| `bee_router` | routage + contrôleur + Context + filtres | `server/web`, `context` |
| `bee_orm` | ORM + QuerySet + Migration | `client/orm` |
| `bee_kv` | abstraction unifiée KV/Cache | `client/cache` (extension) |
| `bee_search` | moteur de recherche/analyse | — (nouveau) |
| `bee_graph` | base de données graphe | — (nouveau) |
| `bee_tsdb` | base de données de séries temporelles | — (nouveau) |
| `bee_config` | gestion de configuration + rechargement à chaud | `client/config` |
| `bee_cache` | abstraction de cache | `client/cache` |
| `bee_session` | gestion de session | `server/web/session` |
| `bee_logs` | journalisation | `logs` |
| `bee_template` | rendu de templates | — (amélioré) |
| `bee_cli` | outil CLI | outil `bee` |

### Couverture de tests

Les 68 tests de l'ensemble du dépôt passent :

| Crate | Nombre de tests |
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

## Soutenir le projet

Si ce projet vous est utile, n'hésitez pas à scanner le code QR pour nous soutenir, merci !

**WeChat Pay**

<img src="weixinpay.png" width="160" height="175" alt="WeChat Pay">

**Alipay**

<img src="alipay.png" width="160" height="175" alt="Alipay">

**Virement international (Bank Transfer)**

Les utilisateurs à l'étranger peuvent soutenir le projet par virement bancaire :

**Informations du bénéficiaire**

| Élément | Valeur |
|------|------|
| Nom du bénéficiaire | WANG KEXUN |
| Numéro de compte du bénéficiaire | 881015918251 |

**Banque du bénéficiaire**

| Élément | Valeur |
|------|------|
| SWIFT Code | AABLHKHHXXX |
| Nom de la banque | ZA Bank Limited |
| Code banque | 387 |
| Adresse de la banque | Core F, Cyberport 3, 100 Cyberport Road, Hong Kong |

**Banque correspondante pour les virements internationaux (si nécessaire)**

> Note : il s'agit des informations de la banque correspondante (banque intermédiaire) pour les virements internationaux, et non de la banque du bénéficiaire. Renseignez-vous auprès de votre banque émettrice pour savoir si ces informations sont requises.

- **Pour les virements en dollars de Hong Kong, yuans et dollars américains** (banque correspondante : Citibank) :

| Élément | Valeur |
|------|------|
| Nom de la banque | Citibank N.A. Hong Kong |
| SWIFT Code | CITIHKHXXXX |
| Code banque | 006 |
| Nom de la succursale | Hong Kong Branch |
| Code de la succursale | 391 |
| Adresse de la banque | Citibank Tower, Citibank Plaza, 3 Garden Road, Central, Hong Kong |

- **Pour les virements dans d'autres devises** (banque correspondante : BNY Mellon) :

| Élément | Valeur |
|------|------|
| Nom de la banque | THE BANK OF NEW YORK MELLON |
| SWIFT Code | IRVTUS3NXXX |
| Adresse de la banque | THE BANK OF NEW YORK MELLON, 240 GREENWICH STREET, NEW YORK, United States |

### Licence

Apache-2.0
