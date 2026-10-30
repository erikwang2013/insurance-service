<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Beerust

[简体中文](../README.md) · [English](README.en.md) · [한국어](README.ko.md) · [Русский](README.ru.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Português](README.pt.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [বাংলা](README.bn.md) · [Bahasa Indonesia](README.id.md) · [日本語](README.ja.md)

Beerust adalah framework web tingkat produksi berbahasa Rust, dengan filosofi desain yang berasal dari framework Beego milik Go, diekspresikan ulang menggunakan trait, macro, dan sistem tipe khas Rust.

## Maskot Proyek: Rusty

<img src="rusty.svg" width="360" alt="Rusty melayang menghadap depan, sayap terbentang membentuk cincin, kaki depan memeluk kotak perkakas oranye karat">

### Kartu Karakter

**Nama** Rusty (karat) — warna karat dari Rust, juga karat yang menempel di overall-nya.

**Penampilan**: seekor lebah insinyur yang berbulu halus. Bulu berwarna emas amber menutupi perut bulatnya, tiga garis hitam arang seperti tali overall yang miring; empat sayap transparan setipis cahaya, berdengung meninggalkan jejak bayangan saat melayang; mata majemuknya adalah dua batu obsidian seperti manik kaca, masing-masing dengan satu titik kilau; kaki depannya memeluk kotak perkakas berwarna oranye karat yang mengilap karena sering dipakai — itu keranjang serbuk sarinya, yang tak pernah berisi serbuk sari, melainkan dependensi.

**Kepribadian**: gila kerja, optimis, dan sedikit perfeksionis.

- Saat memperbaiki sarang lebah, ia memeriksa sel demi sel dengan antenanya, seserius menjalankan lint
- Versi dependensi di keranjang serbuk sari dikunci mati-mati; siapa pun yang menyentuhnya akan dihadapi
- Saat memburu bug, ia melayang diam di udara, sayapnya bergetar begitu cepat hingga hanya tampak lingkaran bayangan
- Pada hari rilis versi baru, ia menari tarian goyang angka 8 khas lebah di depan pintu sarang

### Spesifikasi Visual

| Elemen | Spesifikasi |
|------|------|
| Tubuh | Emas amber `#F5B301`, bulu perut pendek dan lebat, dengan lingkaran kilau di tepinya |
| Garis | Hitam arang `#1F1A17`, tiga garis, hanya di perut |
| Sayap | Biru muda transparan `#CFE8F5`, opacity 40%, dengan urat yang terlihat |
| Mata majemuk | Cokelat kehitaman tua `#2A1E16`, berbentuk elips, satu titik kilau di pojok kiri atas |
| Aksen | Oranye karat `#B7410E` (kotak perkakas, dasi kupu-kupu), warna karat adalah ciri khasnya |

### Koleksi Ekspresi

| Ekspresi | Gambaran | Kapan |
|------|------|---------|
| Penuh semangat | `(^o^)` bulu mengembang, sayap bergetar frekuensi tinggi | saat semua pengujian hijau |
| Melayang debugging | `o(°▽°)o` melayang di udara, hanya tersisa jejak sayap | saat memburu bug yang sulit |
| Pulang dengan muatan penuh | `(≧▽≦)` kotak perkakas penuh berisi, terbang sempoyongan | saat driver baru berhasil diintegrasikan |
| Mengantuk | `(-_-)zZ` berjongkok di pintu sarang, antena terkulai | saat malam larut dan tidak ada lalu lintas |
| Bulu berdiri | `(#°益°)` sengat di ekor berdiri tegak | saat filter keamanan memblokir serangan |

### Gerakan Khas

1. **Melayang debugging** — begitu menemukan bug, ia melayang di atas kode, sayap berdengung menjadi bayangan, menatap tajam sampai kamu selesai memperbaikinya.
2. **Tarian goyang angka 8** — setiap kali merilis versi baru, ia menari satu putaran penuh mengelilingi sarang, mengumumkan ke seluruh koloni "bunga sudah mekar". Lebah sungguhan menggunakan ini untuk menyampaikan lokasi sumber bunga, Rusty menggunakannya untuk menyampaikan changelog.
3. **Merapikan bulu** — kaki depan mengusap antena berulang kali. Kamu mengira ia sedang berdandan, sebenarnya ia sedang memeriksa apakah konfigurasi sudah di-hot-reload.

### Di Mana Ia Muncul

- **Logo**: pose melayang menghadap depan, sayap terbentang membentuk cincin yang tepat mengelilingi nama.
- **Halaman 404**: melayang di atas ladang bunga kosong, berputar kebingungan — "Mananya madu?"
- **CLI**: deretan lebah kecil di banner saat startup, berjongkok di samping mengawasi saat pengujian berjalan.
- **Pengumuman rilis**: gambar saat menari tarian angka 8.

## Tujuan Desain

| Tujuan | Target |
|------|------|
| **Pengalaman pengembangan** | dari `bee-rust new` hingga permintaan pertama < 30 detik |
| **Performa** | overhead lapisan controller < 5% (dibandingkan axum polos), latensi routing P99 < 100µs |
| **Kecepatan kompilasi** | kompilasi penuh meta crate < 60 detik (release), kompilasi inkremental < 5 detik |
| **Ukuran biner** | aplikasi terkecil (hanya router) < 5MB (strip + LTO) |
| **Keamanan** | 0 kode bisnis `unsafe`; semua FFI dibungkus dalam crate `*-sys` terpisah |
| **Kompatibilitas** | Rust 1.80+ MSRV, mengikuti stable |

## Prinsip Desain

1. **Filosofi Beego, ekspresi Rust** — MVC, namespace, rantai filter, diimplementasikan dengan trait + macro
2. **Peningkatan progresif** — inti minimal hanya bergantung pada axum + tokio, sisanya diatur oleh feature gate
3. **Eksplisit lebih baik daripada implisit** — registrasi rute, pemetaan model, urutan middleware semuanya dideklarasikan eksplisit dalam kode
4. **Abstraksi tanpa biaya** — dispatch statis trait, ekspansi macro pada waktu kompilasi, tanpa overhead fungsi virtual
5. **Independensi mesin penyimpanan** — setiap trait engine dapat diganti implementasinya secara terpisah, tanpa memengaruhi bisnis di lapisan atas
6. **Observabilitas bawaan** — instrumentasi tracing + metrics mencakup seluruh framework, log terstruktur aktif secara default

## Desain Arsitektur

### Topologi Crate

```
bee_rust/           # Meta crate, re-export + feature flags
bee_router/         # Routing + Controller + Context + rantai filter
bee_orm/            # ORM — trait Model + QuerySet + Migration + pemetaan relasi
bee_kv/             # Abstraksi terpadu KV/Cache — Redis + Memcached
bee_search/         # Mesin pencarian/analitik — Elasticsearch + OpenSearch + ClickHouse
bee_graph/          # Basis data graf — Neo4j + NebulaGraph + ArangoDB
bee_tsdb/           # Basis data deret waktu — InfluxDB + Apache IoTDB + QuestDB
bee_config/         # Manajemen konfigurasi — INI/YAML/ENV + hot-reload
bee_cache/          # Abstraksi cache — Memory/Redis/Memcache
bee_session/        # Session — backend Memory/Redis/Cookie/Database
bee_logs/           # Logging — log bertingkat + integrasi tracing
bee_template/       # Rendering template — berbasis tera
bee_cli/            # CLI — scaffolding/generasi kode/run dev/pack (dalam rencana migrasi)
```

### Diagram Arsitektur

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

### Hubungan Ketergantungan Crate

```
bee_config (tanpa dependensi)
bee_logs   (tanpa dependensi)
bee_cache  → bee_config
bee_kv     → bee_config
bee_session → bee_cache, bee_config
bee_template (tanpa dependensi)
bee_orm    → bee_config, bee_cache
bee_search → bee_config
bee_graph  → bee_config
bee_tsdb   → bee_config
bee_router → bee_session, bee_template, bee_config, bee_logs
bee_cli    → bee_router, bee_orm
bee_rust   → semua crate di atas (re-export)
```

### Basis Data yang Didukung

| Kategori | Basis Data | Crate Terkait | Feature Flag |
|------|--------|-----------|-------------|
| **Relasional** | SQLite | `bee_orm` | `sqlite` |
| | PostgreSQL | `bee_orm` | `postgres` |
| | MySQL | `bee_orm` | `mysql` |
| | TiDB | `bee_orm` | `mysql` |
| **KV / Cache** | Redis | `bee_kv` / `bee_cache` | `redis` |
| | Memcached | `bee_kv` / `bee_cache` | `memcache` |
| **Pencarian / Analitik** | Elasticsearch | `bee_search` | `elasticsearch` |
| | OpenSearch | `bee_search` | `opensearch` |
| | ClickHouse | `bee_search` | `clickhouse` |
| **Basis data graf** | Neo4j | `bee_graph` | `neo4j` |
| | NebulaGraph | `bee_graph` | `nebulagraph` |
| | ArangoDB | `bee_graph` | `arangodb` |
| **Basis data deret waktu** | InfluxDB | `bee_tsdb` | `influxdb` |
| | Apache IoTDB | `bee_tsdb` | `iotdb` |
| | QuestDB | `bee_tsdb` | `questdb` |

### Rantai Filter Permintaan

```
Permintaan → [SecurityFilter deteksi serangan] → [Pemulihan Session] → [Validasi parameter] → [hook prepare] → [pemrosesan handle] → [hook finish] → Respons
                  ↓ Semua tahap dapat diinterupsi (mirip Abort milik Beego)
```

## Ringkasan Fitur

Rangkuman fitur ada di bawah; penggunaan detail, contoh kode, dan penjelasan API setiap modul ada di [Referensi API](api.id.md).

### Web Inti (bee_router)

Controller MVC: trait `Controller` + `Context`, mendukung namespace rute, registrasi metode RESTful, rantai filter permintaan.

- Output respons: `ctx.json()` / `ctx.text()` / `ctx.html()`
- Pengalihan / pembatalan: `ctx.redirect()` / `ctx.abort()`
- Sesi dan parameter: `ctx.session` / `ctx.params`

### Deteksi Keamanan (fitur `security`)

Filter deteksi serangan berbasis [security-rust](https://crates.io/crates/security-rust), mencakup 27 jenis serangan seperti XSS, injeksi SQL, injeksi perintah, SSRF, diaktifkan dengan satu baris:

```rust
let security = SecurityFilter::new();  // 27 detektor semuanya aktif
```

### ORM (bee_orm)

Macro turunan `#[derive(Model)]` + query berantai QuerySet (filter / order_by / limit), mendukung SQLite, PostgreSQL, MySQL, TiDB.

### Manajemen Konfigurasi (bee_config)

Macro turunan `#[derive(Config)]`, mendukung pemuatan INI / YAML / ENV dan hot-reload.

### Mesin Penyimpanan

Abstraksi trait terpadu untuk KV / Cache (Redis + Memcached), mesin pencarian (Elasticsearch / OpenSearch / ClickHouse), basis data graf (Neo4j / NebulaGraph / ArangoDB), basis data deret waktu (InfluxDB / IoTDB / QuestDB); driver dikompilasi sesuai feature gate.

### Session, Logging, Template

- Session: multi-backend Memory / Redis / Cookie / Database
- Logging: log bertingkat + integrasi tracing
- Template: rendering berbasis tera

### Alat CLI

```bash
bee-rust new my-app            # Membuat scaffolding proyek
bee-rust generate controller user
bee-rust run --watch           # Run development (hot reload)
bee-rust pack                  # Paket untuk deployment
```

## Langkah Penggunaan

### Persyaratan Lingkungan

- Rust 1.80+
- Cargo

### Instalasi

```bash
# Klon proyek
git clone https://github.com/erikwang2013/bee-rust.git
cd bee-rust

# Kompilasi
cargo build --workspace

# Jalankan pengujian
cargo test --workspace
```

### Memulai dengan Cepat

```bash
# Buat proyek baru menggunakan CLI
cargo run -p bee_cli -- new hello
cd hello

# Jalankan server development
cargo run
```

### Menggunakan dalam Proyek

```toml
[dependencies]
bee_rust = { git = "https://github.com/erikwang2013/bee-rust", features = ["full"] }
```

## Catatan Teknis

### Tumpukan Teknologi

| Lapisan | Teknologi |
|----|------|
| Basis HTTP | axum 0.8 + tower 0.5 |
| Runtime asinkron | tokio 1.x |
| Serialisasi | serde + serde_json |
| Mesin template | tera 1.x |
| Fondasi logging | tracing + tracing-subscriber |
| CLI | clap 4 |
| Penguraian konfigurasi | toml / serde_yaml / INI buatan sendiri |
| Penanganan kesalahan | thiserror |
| Macro prosedural | syn + quote + proc-macro2 |

### Pola Desain

| Pola | Penerapan |
|------|------|
| **Builder** | Logger, Router, QuerySet |
| **Abstraksi Trait** | Cache, KvStore, SearchEngine, GraphDB, TimeSeriesDB |
| **Macro turunan** | `#[derive(Model)]`, `#[derive(Config)]` |
| **Feature Gate** | implementasi driver dikompilasi sesuai kebutuhan (redis, memcached, elasticsearch, dll.) |
| **Filter Chain** | rantai filter permintaan, sebanding dengan Beego Filter |

### Daftar Crate

| Crate | Fungsi | Sebanding dengan Beego |
|-------|------|-----------|
| `bee_rust` | meta crate, pintu masuk terpadu | — |
| `bee_router` | Routing + Controller + Context + Filter | `server/web`, `context` |
| `bee_orm` | ORM + QuerySet + Migration | `client/orm` |
| `bee_kv` | Abstraksi terpadu KV/Cache | `client/cache` (perluasan) |
| `bee_search` | Mesin pencarian/analitik | — (baru) |
| `bee_graph` | Basis data graf | — (baru) |
| `bee_tsdb` | Basis data deret waktu | — (baru) |
| `bee_config` | Manajemen konfigurasi + hot-reload | `client/config` |
| `bee_cache` | Abstraksi cache | `client/cache` |
| `bee_session` | Manajemen Session | `server/web/session` |
| `bee_logs` | Logging | `logs` |
| `bee_template` | Rendering template | — (ditingkatkan) |
| `bee_cli` | Alat CLI | alat `bee` |

### Cakupan Pengujian

Seluruh 68 pengujian di repositori lolos:

| Crate | Jumlah Pengujian |
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

## Dukungan

Jika proyek ini bermanfaat untukmu, silakan pindai kode QR untuk memberi dukungan, terima kasih!

**WeChat Pay**

<img src="weixinpay.png" width="160" height="175" alt="WeChat Pay">

**Alipay**

<img src="alipay.png" width="160" height="175" alt="Alipay">

**Transfer Bank Global (Bank Transfer)**

Pengguna luar negeri dapat mendukung melalui transfer bank:

**Informasi Penerima**

| Item | Isi |
|------|------|
| Nama penerima | WANG KEXUN |
| Nomor rekening penerima | 881015918251 |

**Bank Penerima**

| Item | Isi |
|------|------|
| SWIFT Code | AABLHKHHXXX |
| Nama bank | ZA Bank Limited |
| Kode bank | 387 |
| Alamat bank | Core F, Cyberport 3, 100 Cyberport Road, Hong Kong |

**Bank Perantara Transfer Lintas Negara (Jika Diperlukan)**

> Catatan: Ini adalah informasi bank perantara (bank penerus) untuk transfer lintas negara, bukan informasi bank penerima. Silakan tanyakan ke bank pengirimmu apakah informasi ini diperlukan.

- **Untuk setoran dolar Hong Kong, RMB, dan dolar AS** (bank perantara: Citibank):

| Item | Isi |
|------|------|
| Nama bank | Citibank N.A. Hong Kong |
| SWIFT Code | CITIHKHXXXX |
| Kode bank | 006 |
| Nama cabang | Hong Kong Branch |
| Kode cabang | 391 |
| Alamat bank | Citibank Tower, Citibank Plaza, 3 Garden Road, Central, Hong Kong |

- **Untuk mata uang lainnya** (bank perantara: BNY Mellon):

| Item | Isi |
|------|------|
| Nama bank | THE BANK OF NEW YORK MELLON |
| SWIFT Code | IRVTUS3NXXX |
| Alamat bank | THE BANK OF NEW YORK MELLON, 240 GREENWICH STREET, NEW YORK, United States |

### Lisensi

Apache-2.0
