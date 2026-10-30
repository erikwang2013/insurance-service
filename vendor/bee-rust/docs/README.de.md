<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Beerust

[简体中文](../README.md) · [English](README.en.md) · [한국어](README.ko.md) · [Русский](README.ru.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Português](README.pt.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [বাংলা](README.bn.md) · [Bahasa Indonesia](README.id.md) · [日本語](README.ja.md)

Beerust ist ein produktionsreifes Web-Framework in Rust. Seine Designphilosophie stammt vom Go-Framework Beego und wurde mit Rust-typischen Traits, Makros und dem Typsystem neu ausgedrückt.

## Projekt-Maskottchen: Rusty

<img src="rusty.svg" width="360" alt="Rusty von vorn schwebend, Flügel zu einem Ring ausgebreitet, in den Vorderpfoten einen rostorangen Werkzeugkasten haltend">

### Steckbrief

**Name** Rusty — die Rostfarbe von Rust, zugleich der Rostfleck, der an seiner Latzhose klebt.

**Aussehen**: Eine flauschige Ingenieursbiene. Ambergoldener Flaum bedeckt den runden Hinterleib, drei kohlschwarze Streifen wie schief getragene Latzträger; vier durchscheinende Flügel so dünn, dass Licht hindurchscheint, beim Schweben als Schlieren; die Komplexaugen sind zwei obsidianglatte Glaskugeln mit je einem Glanzlicht; in den Vorderpfoten hält sie einen blankpolierten rostorangen Werkzeugkasten — ihren Pollenkorb, der nie Pollen enthält, sondern Abhängigkeiten.

**Charakter**: Arbeitstier, Optimist, leicht zwanghaft.

- Beim Reparieren des Bienenstocks prüft sie mit den Fühlern Zelle für Zelle, so penibel wie ein Lint-Lauf
- Die Abhängigkeiten im Pollenkorb sind bis aufs Versions-Tag fest eingefroren; wer daran herumfummelt, bekommt Ärger
- Bei der Bug-Jagd schwebt sie regungslos in der Luft, die Flügel nur noch ein Schemen
- Am Tag der Veröffentlichung tanzt sie vor dem Stock den Achter-Schwänzeltanz der Bienen

### Visuelle Spezifikation

| Element | Spezifikation |
|------|------|
| Körper | Ambergold `#F5B301`, Hinterleibsflaum kurz und dicht, mit einem Glanzlichtring am Rand |
| Streifen | Kohleschwarz `#1F1A17`, drei Stück, nur am Hinterleib |
| Flügel | Transparentes Hellblau `#CFE8F5`, 40 % Deckkraft, Adern erkennbar |
| Komplexaugen | Dunkelbraunschwarz `#2A1E16`, oval, mit einem Glanzlicht oben links |
| Akzente | Rostorange `#B7410E` (Werkzeugkasten, Fliege) — die Rostfarbe ist ihr Markenzeichen |

### Ausdruckssammlung

| Ausdruck | Bild | Wann |
|------|------|---------|
| Voller Energie | `(^o^)` Flaum gesträubt, Flügel vibrieren mit hoher Frequenz | wenn alle Tests grün sind |
| Schwebend am Debuggen | `o(°▽°)o` schwebt in der Luft, nur noch Flügelschlieren | auf der Jagd nach einem hartnäckigen Bug |
| Beladen nach Hause | `(≧▽≦)` Werkzeugkasten prall gefüllt, fliegt etwas schief | wenn ein neuer Treiber erfolgreich angebunden wurde |
| Dösen | `(-_-)zZ` hockt vor dem Stock, Fühler hängen herab | nachts, wenn kein Traffic ist |
| Aufgeplustert | `(#°益°)` Stachel aufgerichtet | wenn der Sicherheitsfilter einen Angriff abfängt |

### Signature-Moves

1. **Schwebend debuggen** — Findet sie einen Bug, schwebt sie über dem Code, die Flügel zu einem Schemen, und starrt, bis du es behoben hast.
2. **Achter-Schwänzeltanz** — Bei jedem Release tanzt sie eine volle Runde um den Stock und verkündet der ganzen Kolonie: „Die Blüten sind offen." Echte Bienen übermitteln damit die Position der Trachtquelle; Rusty übermittelt damit das Changelog.
3. **Putzen** — Mit den Vorderpfoten wischt sie wiederholt über die Fühler. Du denkst, sie macht sich hübsch; in Wahrheit prüft sie, ob die Konfiguration heiß nachgeladen wurde.

### Wo sie auftaucht

- **Logo**: Schwebehaltung von vorn, die Flügel zu einem Ring ausgebreitet, der genau den Namen umschließt.
- **404-Seite**: Sie schwebt über einem leeren Blütenfeld und dreht sich ratlos im Kreis — „Wo ist der Honig?"
- **CLI**: Eine kleine Biene in der Startbanner-Zeile, die beim Testlauf danebensitzt und die Arbeit überwacht.
- **Release-Ankündigungen**: Das Bild mit dem Achter-Tanz.

## Designziele

| Ziel | Kennzahl |
|------|------|
| **Entwicklererfahrung** | von `bee-rust new` bis zum ersten Request < 30 Sekunden |
| **Leistung** | Controller-Overhead < 5 % (gegenüber nacktem axum), P99-Routing-Latenz < 100 µs |
| **Kompilierzeit** | Vollständige Kompilierung des Meta-Crates < 60 s (release), inkrementell < 5 s |
| **Binärgröße** | Minimale App (nur Router) < 5 MB (strip + LTO) |
| **Sicherheit** | 0 unsafe im Geschäftscode; sämtliches FFI in separaten `*-sys`-Crates gekapselt |
| **Kompatibilität** | Rust 1.80+ MSRV, folgt stable |

## Designprinzipien

1. **Beego-Philosophie, Rust-Ausdruck** — MVC, Namespaces, Filterketten, umgesetzt mit Trait + Makro
2. **Progressive Verbesserung** — Der minimale Kern hängt nur von axum + tokio ab, alles andere per Feature-Gate
3. **Explizit vor implizit** — Routenregistrierung, Modell-Mapping, Middleware-Reihenfolge: alles explizit im Code deklariert
4. **Nullkosten-Abstraktion** — Traits mit statischer Verteilung, Makros expandieren zur Kompilierzeit, keine virtuellen Funktionskosten
5. **Unabhängigkeit der Speicher-Engines** — Jeder Engine-Trait kann separat durch eine andere Implementierung ersetzt werden, ohne die darüberliegende Geschäftslogik zu beeinflussen
6. **Beobachtbarkeit eingebaut** — tracing + metrics-Instrumentierung über das gesamte Framework, strukturierte Logs standardmäßig aktiviert

## Architektur

### Crate-Topologie

```
bee_rust/           # Meta-Crate, re-export + feature flags
bee_router/         # Routing + Controller + Context + Filterkette
bee_orm/            # ORM — Model-Trait + QuerySet + Migration + Beziehungs-Mapping
bee_kv/             # Vereinheitlichte KV/Cache-Abstraktion — Redis + Memcached
bee_search/         # Such-/Analyse-Engine — Elasticsearch + OpenSearch + ClickHouse
bee_graph/          # Graphdatenbank — Neo4j + NebulaGraph + ArangoDB
bee_tsdb/           # Zeitreihendatenbank — InfluxDB + Apache IoTDB + QuestDB
bee_config/         # Konfigurationsverwaltung — INI/YAML/ENV + Hot-Reload
bee_cache/          # Cache-Abstraktion — Memory/Redis/Memcache
bee_session/        # Session — Memory/Redis/Cookie/Database-Backends
bee_logs/           # Logging — mehrstufige Logs + tracing-Integration
bee_template/       # Template-Rendering — basierend auf tera
bee_cli/            # CLI — Scaffolding/Codegenerierung/Entwicklungsbetrieb/Packaging (Migration geplant)
```

### Architekturdiagramm

```
                          ┌───────────────────────┐
                          │  bee_rust  (meta)      │
                          │  re-export + features  │
                          └───────────┬───────────┘
                                      │
              ┌───────────────────────┼───────────────────────┐
              │                       │                       │
     ┌────────▼────────┐    ┌────────▼────────┐    ┌────────▼────────┐
     │   Web-Ebene      │    │   Daten-Ebene   │    │   Werkzeug-Ebene │
     └────────┬────────┘    └────────┬────────┘    └────────┬────────┘
              │                       │                       │
  ┌───────────┼───────────┐  ┌───────┼───────┐  ┌───────────┼───────────┐
  │ bee_router            │  │ bee_orm        │  │ bee_cli               │
  │  - Routenregistrierung│  │  - Model/Query  │  │  - Scaffolding        │
  │  - Controller-Trait   │  │  - Migration   │  │  - Hot-Reload         │
  │  - Filterkette        │  │  - Verbindung   │  │  - Codegenerierung    │
  │  - Parameter-Extraktion │                 │  │                       │
  ├────────────────────────┤  ├────────────────┤  ├───────────────────────┤
  │ bee_template           │  │ bee_config     │  │ bee_logs              │
  │  - Template-Rendering  │  │  - INI/YAML/ENV│  │  - mehrstufige Logs   │
  │  - HTML/JSON           │  │  - Hot-Reload   │  │  - tracing-Integration│
  ├────────────────────────┤  ├────────────────┤  └───────────────────────┘
  │ bee_session            │  │ bee_cache      │
  │  - Session-Verwaltung  │  │  - Cache-Trait │
  │  - Multi-Backend       │  │  - Mem/Redis   │
  └────────────────────────┘  └────────────────┘

  ┌─────────────────────────────────────────────────────────┐
  │                Speicher-Engine-Ebene                     │
  ├──────────────────┬──────────────────────────────────────┤
  │ bee_kv           │  Redis + Memcached                   │
  │ bee_search       │  Elasticsearch + OpenSearch + ClickHouse │
  │ bee_graph        │  Neo4j + NebulaGraph + ArangoDB      │
  │ bee_tsdb         │  InfluxDB + Apache IoTDB + QuestDB   │
  └──────────────────┴──────────────────────────────────────┘
```

### Crate-Abhängigkeiten

```
bee_config (keine Abhängigkeiten)
bee_logs   (keine Abhängigkeiten)
bee_cache  → bee_config
bee_kv     → bee_config
bee_session → bee_cache, bee_config
bee_template (keine Abhängigkeiten)
bee_orm    → bee_config, bee_cache
bee_search → bee_config
bee_graph  → bee_config
bee_tsdb   → bee_config
bee_router → bee_session, bee_template, bee_config, bee_logs
bee_cli    → bee_router, bee_orm
bee_rust   → alle oben genannten Crates (re-export)
```

### Unterstützte Datenbanken

| Kategorie | Datenbank | Zugehöriges Crate | Feature-Flag |
|------|--------|-----------|-------------|
| **Relational** | SQLite | `bee_orm` | `sqlite` |
| | PostgreSQL | `bee_orm` | `postgres` |
| | MySQL | `bee_orm` | `mysql` |
| | TiDB | `bee_orm` | `mysql` |
| **KV / Cache** | Redis | `bee_kv` / `bee_cache` | `redis` |
| | Memcached | `bee_kv` / `bee_cache` | `memcache` |
| **Suche / Analyse** | Elasticsearch | `bee_search` | `elasticsearch` |
| | OpenSearch | `bee_search` | `opensearch` |
| | ClickHouse | `bee_search` | `clickhouse` |
| **Graphdatenbank** | Neo4j | `bee_graph` | `neo4j` |
| | NebulaGraph | `bee_graph` | `nebulagraph` |
| | ArangoDB | `bee_graph` | `arangodb` |
| **Zeitreihendatenbank** | InfluxDB | `bee_tsdb` | `influxdb` |
| | Apache IoTDB | `bee_tsdb` | `iotdb` |
| | QuestDB | `bee_tsdb` | `questdb` |

### Anfrage-Filterkette

```
Anfrage → [SecurityFilter Angriffserkennung] → [Session-Wiederherstellung] → [Parametervalidierung] → [prepare-Hook] → [handle-Verarbeitung] → [finish-Hook] → Antwort
                  ↓ jede Stufe kann unterbrochen werden (analog zu Beegos Abort)
```

## Funktionsübersicht

Die Funktionsübersicht folgt unten; detaillierte Verwendung, Codebeispiele und API-Beschreibungen der einzelnen Module finden sich in der [API-Referenz](api.de.md).

### Web-Kern (bee_router)

MVC-Controller: `Controller`-Trait + `Context`, mit Routing-Namespaces, Registrierung von RESTful-Methoden und Anfrage-Filterkette.

- Antwortausgabe: `ctx.json()` / `ctx.text()` / `ctx.html()`
- Weiterleitung / Abbruch: `ctx.redirect()` / `ctx.abort()`
- Session und Parameter: `ctx.session` / `ctx.params`

### Sicherheitserkennung (`security`-Feature)

Ein auf [security-rust](https://crates.io/crates/security-rust) basierender Angriffserkennungs-Filter, der 27 Angriffstypen wie XSS, SQL-Injection, Command-Injection und SSRF abdeckt — mit einer Zeile aktiviert:

```rust
let security = SecurityFilter::new();  // alle 27 Detektoren aktiv
```

### ORM (bee_orm)

`#[derive(Model)]`-Derivierungsmakro + QuerySet-Kettenabfrage (filter / order_by / limit), unterstützt SQLite, PostgreSQL, MySQL, TiDB.

### Konfigurationsverwaltung (bee_config)

`#[derive(Config)]`-Derivierungsmakro, unterstützt das Laden aus INI / YAML / ENV sowie Hot-Reload.

### Speicher-Engines

KV / Cache (Redis + Memcached), Suchmaschine (Elasticsearch / OpenSearch / ClickHouse), Graphdatenbank (Neo4j / NebulaGraph / ArangoDB), Zeitreihendatenbank (InfluxDB / IoTDB / QuestDB) — einheitliche Trait-Abstraktion, Treiber werden per Feature-Gate kompiliert.

### Session, Logging, Templates

- Session: Memory / Redis / Cookie / Database-Multi-Backend
- Logging: mehrstufige Logs + tracing-Integration
- Templates: Rendering basierend auf tera

### CLI-Werkzeuge

```bash
bee-rust new my-app            # erstellt das Projekt-Scaffolding
bee-rust generate controller user
bee-rust run --watch           # Entwicklungsbetrieb (Hot-Reload)
bee-rust pack                  # Build und Deploy-Paket
```

## Verwendung

### Voraussetzungen

- Rust 1.80+
- Cargo

### Installation

```bash
# Projekt klonen
git clone https://github.com/erikwang2013/bee-rust.git
cd bee-rust

# Kompilieren
cargo build --workspace

# Tests ausführen
cargo test --workspace
```

### Schnellstart

```bash
# Neues Projekt über die CLI erstellen
cargo run -p bee_cli -- new hello
cd hello

# Entwicklungsserver starten
cargo run
```

### Im eigenen Projekt verwenden

```toml
[dependencies]
bee_rust = { git = "https://github.com/erikwang2013/bee-rust", features = ["full"] }
```

## Technische Hinweise

### Technologie-Stack

| Ebene | Technologie |
|----|------|
| HTTP-Basis | axum 0.8 + tower 0.5 |
| Async-Runtime | tokio 1.x |
| Serialisierung | serde + serde_json |
| Template-Engine | tera 1.x |
| Logging-Unterbau | tracing + tracing-subscriber |
| CLI | clap 4 |
| Konfigurationsparsing | toml / serde_yaml / eigenes INI |
| Fehlerbehandlung | thiserror |
| Prozeduralmakros | syn + quote + proc-macro2 |

### Designmuster

| Muster | Anwendung |
|------|------|
| **Builder** | Logger, Router, QuerySet |
| **Trait-Abstraktion** | Cache, KvStore, SearchEngine, GraphDB, TimeSeriesDB |
| **Derivierungsmakro** | `#[derive(Model)]`, `#[derive(Config)]` |
| **Feature-Gate** | Treiberimplementierungen werden bei Bedarf kompiliert (redis, memcached, elasticsearch usw.) |
| **Filterkette** | Anfrage-Filterkette, analog zu Beego Filter |

### Crate-Übersicht

| Crate | Funktion | Beego-Pendant |
|-------|------|-----------|
| `bee_rust` | Meta-Crate, einheitlicher Einstiegspunkt | — |
| `bee_router` | Routing + Controller + Context + Filter | `server/web`, `context` |
| `bee_orm` | ORM + QuerySet + Migration | `client/orm` |
| `bee_kv` | Vereinheitlichte KV/Cache-Abstraktion | `client/cache` (erweitert) |
| `bee_search` | Such-/Analyse-Engine | — (neu) |
| `bee_graph` | Graphdatenbank | — (neu) |
| `bee_tsdb` | Zeitreihendatenbank | — (neu) |
| `bee_config` | Konfigurationsverwaltung + Hot-Reload | `client/config` |
| `bee_cache` | Cache-Abstraktion | `client/cache` |
| `bee_session` | Session-Verwaltung | `server/web/session` |
| `bee_logs` | Logging | `logs` |
| `bee_template` | Template-Rendering | — (verbessert) |
| `bee_cli` | CLI-Werkzeuge | `bee`-Werkzeug |

### Testabdeckung

Alle 68 Tests des Repositories bestehen:

| Crate | Anzahl der Tests |
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

## Unterstützung willkommen

Wenn dir dieses Projekt hilft, scanne gern den QR-Code und unterstütze es mit einer Spende. Danke!

**WeChat Pay**

<img src="weixinpay.png" width="160" height="175" alt="WeChat Pay">

**Alipay**

<img src="alipay.png" width="160" height="175" alt="Alipay">

**Globale Überweisung (Bank Transfer)**

Nutzer außerhalb Chinas können per Banküberweisung unterstützen:

**Empfängerinformationen**

| Feld | Wert |
|------|------|
| Name des Empfängers | WANG KEXUN |
| Kontonummer des Empfängers | 881015918251 |

**Empfängerbank**

| Feld | Wert |
|------|------|
| SWIFT-Code | AABLHKHHXXX |
| Bankname | ZA Bank Limited |
| Bankleitzahl | 387 |
| Bankadresse | Core F, Cyberport 3, 100 Cyberport Road, Hong Kong |

**Korrespondenzbank für grenzüberschreitende Überweisungen (falls erforderlich)**

> Hinweis: Dies sind Informationen der Korrespondenzbank (Zwischenbank) für grenzüberschreitende Überweisungen, nicht der Empfängerbank. Frage bei deiner überweisenden Bank nach, ob diese Angaben benötigt werden.

- **Einzahlungen in Hongkong-Dollar, Renminbi und US-Dollar** (Korrespondenzbank: Citibank):

| Feld | Wert |
|------|------|
| Bankname | Citibank N.A. Hong Kong |
| SWIFT-Code | CITIHKHXXXX |
| Bankleitzahl | 006 |
| Filialname | Hong Kong Branch |
| Filialnummer | 391 |
| Bankadresse | Citibank Tower, Citibank Plaza, 3 Garden Road, Central, Hong Kong |

- **Einzahlungen in anderen Währungen** (Korrespondenzbank: BNY Mellon):

| Feld | Wert |
|------|------|
| Bankname | THE BANK OF NEW YORK MELLON |
| SWIFT-Code | IRVTUS3NXXX |
| Bankadresse | THE BANK OF NEW YORK MELLON, 240 GREENWICH STREET, NEW YORK, United States |

### Lizenz

Apache-2.0
