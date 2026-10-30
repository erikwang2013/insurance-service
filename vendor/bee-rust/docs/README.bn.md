<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Beerust

[简体中文](../README.md) · [English](README.en.md) · [한국어](README.ko.md) · [Русский](README.ru.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Português](README.pt.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [বাংলা](README.bn.md) · [Bahasa Indonesia](README.id.md) · [日本語](README.ja.md)

Beerust হল Rust ভাষায় লেখা একটি প্রোডাকশন-গ্রেড ওয়েব ফ্রেমওয়ার্ক, যার নকশা দর্শন Go-এর Beego ফ্রেমওয়ার্ক থেকে নেওয়া এবং Rust-এর স্বাভাবিক trait, macro ও টাইপ সিস্টেম দিয়ে নতুন করে প্রকাশ করা হয়েছে।

## প্রজেক্ট মাসকট: Rusty

<img src="rusty.svg" width="360" alt="Rusty সামনের দিকে ঝুলে আছে, ডানা বৃত্তের মতো ছড়ানো, সামনের পা দিয়ে মরিচা-কমলা টুলবক্স জড়িয়ে ধরে আছে">

### ক্যারেক্টার কার্ড

**নাম** Rusty (রাস্টি) — Rust-এর মরিচার রং, আর এটাই তার ওয়ার্ক প্যান্টে লেগে থাকা সেই সামান্য মরিচা।

**চেহারা**: একটি লোমশ ইঞ্জিনিয়ার মৌমাছি। অ্যাম্বার-সোনালি লোমে ঢাকা গোলগোল পেট, তিনটি কাঠকয়লা-কালো ডোরা যেন বাঁকা করে পরা ওয়ার্ক সাসপেন্ডার; চারটি স্বচ্ছ ডানা এত পাতলা যে আলো ভেদ করে, ঝুলে থাকার সময় গুনগুন শব্দে অবশিষ্ট ছবি ফেলে; যৌগিক চোখ দুটি কাচের পুঁতির মতো অবসিডিয়ান, প্রতিটিতে একটি করে হাইলাইট; সামনের পা দিয়ে একটি ঘষে ঘষে চকচকে মরিচা-কমলা টুলবক্স জড়িয়ে ধরে — এটাই তার পরাগ ঝুড়ি, যাতে কখনো পরাগ থাকে না, থাকে ডিপেন্ডেন্সি।

**চরিত্র**: ওয়ার্কাহলিক, প্রাণবন্ত আশাবাদী, হালকা অবসেসিভ-কম্পালসিভ।

- মৌচাক মেরামতের সময় অ্যান্টেনা দিয়ে কোষে কোষে পরীক্ষা করে, ঠিক lint চালানোর মতো খুঁতখুঁতে
- পরাগ ঝুড়ির ডিপেন্ডেন্সির ভার্সন এত জোরালোভাবে লক করে রাখে যে, কেউ নাড়লে তার সাথে ঝগড়া
- বাগ তাড়া করার সময় বাতাসে স্থির হয়ে ঝুলে থাকে, ডানা এত দ্রুত চলে যে শুধু একটি গোল আভা দেখা যায়
- নতুন ভার্সন প্রকাশের দিন, মৌচাকের দরজায় মৌমাছির ৮-আকৃতির দোল-নাচ নাচে

### ভিজ্যুয়াল স্পেসিফিকেশন

| উপাদান | নির্দেশনা |
|------|------|
| মূল দেহ | অ্যাম্বার-সোনালি `#F5B301`, পেটের লোম ছোট ও ঘন, কিনারায় এক সারি হাইলাইট |
| ডোরা | কাঠকয়লা-কালো `#1F1A17`, তিনটি, শুধু পেটে |
| ডানা | স্বচ্ছ হালকা নীল `#CFE8F5`, 40% অস্বচ্ছতা, শিরাগুলো আঁকা যায় |
| যৌগিক চোখ | গাঢ় বাদামি-কালো `#2A1E16`, ডিম্বাকার, উপরের বাম কোণে একটি হাইলাইট |
| সাজসজ্জা | মরিচা-কমলা `#B7410E` (টুলবক্স, বো টাই), মরিচার রংই তার স্বাক্ষর |

### এক্সপ্রেশন সেট

| এক্সপ্রেশন | চিত্র | কখন |
|------|------|---------|
| প্রাণশক্তিতে ভরা | `(^o^)` লোম ফুলে ওঠা, ডানার উচ্চ-ফ্রিকোয়েন্সি কম্পন | যখন সব টেস্ট সবুজ (পাস) |
| ঝুলে থেকে ডিবাগ | `o(°▽°)o` বাতাসে ঝুলে থাকা, শুধু ডানার অবশিষ্ট ছবি | একটি দুরন্ত বাগ তাড়া করছে |
| ভরে ফেরা | `(≧▽≦)` টুলবক্স ফুলে ভরা, বাঁকা হয়ে উড়ছে | নতুন ড্রাইভার সফলভাবে সংযুক্ত হলে |
| তন্দ্রা | `(-_-)zZ` মৌচাকের দরজায় বসে আছে, অ্যান্টেনা ঝুলে পড়া | গভীর রাতে কোনো ট্রাফিক নেই |
| রেগে লোম ফুলানো | `(#°益°)` লেজের হুল খাড়া | সিকিউরিটি ফিল্টার একটি আক্রমণ আটকালে |

### স্বাক্ষর মুভ

1. **ঝুলে থেকে ডিবাগিং** —— বাগ পেলেই কোডের উপরে ঝুলে যায়, ডানা গুনগুন করে অস্পষ্ট হয়ে যায়, তাকিয়ে থাকে যতক্ষণ না তুমি ঠিক করো।
2. **৮-আকৃতির দোল-নাচ** —— প্রতিবার নতুন ভার্সন প্রকাশে মৌচাকের চারপাশে এক পূর্ণ বৃত্ত নাচে, গোটা চাকে জানিয়ে দেয় "ফুল ফুটেছে"। আসল মৌমাছিরা এভাবে ফুলের উৎসের অবস্থান জানায়, Rusty এতে changelog জানায়।
3. **লোম আঁচড়ানো** —— সামনের পা দিয়ে বারবার অ্যান্টেনা ঘষে। তুমি মনে করো সে সাজগোজ করছে, আসলে সে পরীক্ষা করছে কনফিগ হট-রিলোড হয়েছে কি না।

### সে কোথায় দেখা যাবে

- **Logo**: সামনের দিকে ঝুলে থাকা ভঙ্গি, ডানা বৃত্তের মতো ছড়ানো, ঠিক নামটাকে ঘিরে ধরে।
- **404 পেজ**: ফাঁকা ফুলের মাঠের উপর ঝুলে, হতভম্ব হয়ে ঘুরছে — "মধু কোথায়?"
- **CLI**: স্টার্টআপ ব্যানারের এক লাইনের ছোট মৌমাছি, টেস্ট চলাকালীন পাশে বসে তদারকি করে।
- **রিলিজ নোটিশ**: ৮-আকৃতির নাচ নাচার সেই ছবিটা।

## ডিজাইন লক্ষ্য

| লক্ষ্য | মাপকাঠি |
|------|------|
| **ডেভেলপার অভিজ্ঞতা** | `bee-rust new` থেকে প্রথম রিকোয়েস্ট পর্যন্ত < 30 সেকেন্ড |
| **পারফরম্যান্স** | কন্ট্রোলার স্তরের ওভারহেড < 5% (খালি axum-এর তুলনায়), P99 রাউটিং লেটেন্সি < 100µs |
| **কম্পাইল গতি** | মেটা crate-এর পূর্ণ কম্পাইল < 60s (release), ইনক্রিমেন্টাল কম্পাইল < 5s |
| **বাইনারি আকার** | ক্ষুদ্রতম অ্যাপ (শুধু router) < 5MB (strip + LTO) |
| **নিরাপত্তা** | 0 unsafe ব্যবসায়িক কোড; সব FFI আলাদা `*-sys` crate-এ মোড়ানো |
| **সামঞ্জস্যতা** | Rust 1.80+ MSRV, stable অনুসরণ করে |

## ডিজাইন নীতি

1. **Beego দর্শন, Rust-এ প্রকাশ** — MVC, নেমস্পেস, ফিল্টার চেইন, trait + macro দিয়ে বাস্তবায়িত
2. **প্রগতিশীল এনহ্যান্সমেন্ট** — ন্যূনতম কোর শুধু axum + tokio-র উপর নির্ভর করে, বাকি সব feature gate-এ
3. **স্পষ্টতা অন্তর্নিহিতের চেয়ে ভালো** — রুট রেজিস্ট্রেশন, মডেল ম্যাপিং, মিডলওয়্যার অর্ডার — সব কোডে স্পষ্টভাবে ঘোষিত
4. **শূন্য-খরচ অ্যাবস্ট্রাকশন** — trait স্ট্যাটিক ডিসপ্যাচ, macro কম্পাইল-টাইমে সম্প্রসারিত, ভার্চুয়াল ফাংশনের ওভারহেড নেই
5. **স্টোরেজ ইঞ্জিন স্বাধীনতা** — প্রতিটি ইঞ্জিন trait-এর বাস্তবায়ন আলাদাভাবে বদলানো যায়, উপরের স্তরের ব্যবসায়িক লজিকে প্রভাব না ফেলে
6. **বিল্ট-ইন অবজারভেবিলিটি** — tracing + metrics পয়েন্ট পুরো ফ্রেমওয়ার্কে ছড়ানো, স্ট্রাকচার্ড লগ ডিফল্টভাবে চালু

## আর্কিটেকচার ডিজাইন

### Crate টপোলজি

```
bee_rust/           # মেটা crate, re-export + feature flags
bee_router/         # রাউটিং + কন্ট্রোলার + Context + ফিল্টার চেইন
bee_orm/            # ORM — Model trait + QuerySet + Migration + রিলেশন ম্যাপিং
bee_kv/             # KV/Cache একীভূত অ্যাবস্ট্রাকশন — Redis + Memcached
bee_search/         # সার্চ/অ্যানালিটিক্স ইঞ্জিন — Elasticsearch + OpenSearch + ClickHouse
bee_graph/          # গ্রাফ ডেটাবেস — Neo4j + NebulaGraph + ArangoDB
bee_tsdb/           # টাইম-সিরিজ ডেটাবেস — InfluxDB + Apache IoTDB + QuestDB
bee_config/         # কনফিগ ম্যানেজমেন্ট — INI/YAML/ENV + হট-রিলোড
bee_cache/          # ক্যাশ অ্যাবস্ট্রাকশন — Memory/Redis/Memcache
bee_session/        # Session — Memory/Redis/Cookie/Database ব্যাকএন্ড
bee_logs/           # লগিং — মাল্টি-লেভেল লগ + tracing ইন্টিগ্রেশন
bee_template/       # টেমপ্লেট রেন্ডারিং — tera-ভিত্তিক
bee_cli/            # CLI — স্ক্যাফোল্ডিং/কোড জেনারেশন/ডেভ রান/প্যাকেজিং (মাইগ্রেশন পরিকল্পনাধীন)
```

### আর্কিটেকচার ডায়াগ্রাম

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

### Crate নির্ভরতা

```
bee_config (কোনো নির্ভরতা নেই)
bee_logs   (কোনো নির্ভরতা নেই)
bee_cache  → bee_config
bee_kv     → bee_config
bee_session → bee_cache, bee_config
bee_template (কোনো নির্ভরতা নেই)
bee_orm    → bee_config, bee_cache
bee_search → bee_config
bee_graph  → bee_config
bee_tsdb   → bee_config
bee_router → bee_session, bee_template, bee_config, bee_logs
bee_cli    → bee_router, bee_orm
bee_rust   → উপরের সব crate (re-export)
```

### সমর্থিত ডেটাবেস

| শ্রেণি | ডেটাবেস | সংশ্লিষ্ট Crate | Feature Flag |
|------|--------|-----------|-------------|
| **রিলেশনাল** | SQLite | `bee_orm` | `sqlite` |
| | PostgreSQL | `bee_orm` | `postgres` |
| | MySQL | `bee_orm` | `mysql` |
| | TiDB | `bee_orm` | `mysql` |
| **KV / ক্যাশ** | Redis | `bee_kv` / `bee_cache` | `redis` |
| | Memcached | `bee_kv` / `bee_cache` | `memcache` |
| **সার্চ / অ্যানালিটিক্স** | Elasticsearch | `bee_search` | `elasticsearch` |
| | OpenSearch | `bee_search` | `opensearch` |
| | ClickHouse | `bee_search` | `clickhouse` |
| **গ্রাফ ডেটাবেস** | Neo4j | `bee_graph` | `neo4j` |
| | NebulaGraph | `bee_graph` | `nebulagraph` |
| | ArangoDB | `bee_graph` | `arangodb` |
| **টাইম-সিরিজ ডেটাবেস** | InfluxDB | `bee_tsdb` | `influxdb` |
| | Apache IoTDB | `bee_tsdb` | `iotdb` |
| | QuestDB | `bee_tsdb` | `questdb` |

### রিকোয়েস্ট ফিল্টার চেইন

```
রিকোয়েস্ট → [SecurityFilter আক্রমণ শনাক্তকরণ] → [Session পুনরুদ্ধার] → [প্যারামিটার যাচাই] → [prepare হুক] → [handle প্রসেসিং] → [finish হুক] → রেসপন্স
                  ↓ যেকোনো ধাপে বাধা দেওয়া যায় (Beego-র Abort-এর মতো)
```

## ফিচার ওভারভিউ

নিচে ফিচারের সারসংক্ষেপ; প্রতিটি মডিউলের বিস্তারিত ব্যবহার, কোড উদাহরণ ও API ব্যাখ্যার জন্য দেখুন [API রেফারেন্স](api.bn.md)।

### ওয়েব কোর (bee_router)

MVC কন্ট্রোলার: `Controller` trait + `Context`, রাউট নেমস্পেস, RESTful মেথড রেজিস্ট্রেশন, রিকোয়েস্ট ফিল্টার চেইন সমর্থন করে।

- রেসপন্স আউটপুট: `ctx.json()` / `ctx.text()` / `ctx.html()`
- রিডাইরেক্ট / বাধা: `ctx.redirect()` / `ctx.abort()`
- সেশন ও প্যারামিটার: `ctx.session` / `ctx.params`

### নিরাপত্তা সনাক্তকরণ (`security` feature)

[security-rust](https://crates.io/crates/security-rust)-ভিত্তিক আক্রমণ শনাক্তকরণ ফিল্টার, XSS, SQL ইনজেকশন, কমান্ড ইনজেকশন, SSRF সহ ২৭ ধরনের আক্রমণ কভার করে, এক লাইনে চালু:

```rust
let security = SecurityFilter::new();  // ২৭টি ডিটেক্টর চালু
```

### ORM (bee_orm)

`#[derive(Model)]` ডেরাইভ ম্যাক্রো + QuerySet চেইনড কোয়েরি (filter / order_by / limit), SQLite, PostgreSQL, MySQL, TiDB সমর্থন করে।

### কনফিগ ম্যানেজমেন্ট (bee_config)

`#[derive(Config)]` ডেরাইভ ম্যাক্রো, INI / YAML / ENV লোডিং ও হট-রিলোড সমর্থন করে।

### স্টোরেজ ইঞ্জিন

KV / Cache (Redis + Memcached), সার্চ ইঞ্জিন (Elasticsearch / OpenSearch / ClickHouse), গ্রাফ ডেটাবেস (Neo4j / NebulaGraph / ArangoDB), টাইম-সিরিজ ডেটাবেস (InfluxDB / IoTDB / QuestDB) — সব একীভূত trait অ্যাবস্ট্রাকশন, ড্রাইভার feature gate অনুযায়ী কম্পাইল হয়।

### Session, লগ, টেমপ্লেট

- Session: Memory / Redis / Cookie / Database মাল্টি-ব্যাকএন্ড
- লগ: মাল্টি-লেভেল লগ + tracing ইন্টিগ্রেশন
- টেমপ্লেট: tera-ভিত্তিক রেন্ডারিং

### CLI টুল

```bash
bee-rust new my-app            # প্রজেক্ট স্ক্যাফোল্ড তৈরি
bee-rust generate controller user
bee-rust run --watch           # ডেভ রান (হট-রিলোড)
bee-rust pack                  # প্যাকেজিং ও ডিপ্লয়
```

## ব্যবহারের ধাপ

### পরিবেশের প্রয়োজনীয়তা

- Rust 1.80+
- Cargo

### ইনস্টলেশন

```bash
# প্রজেক্ট ক্লোন করুন
git clone https://github.com/erikwang2013/bee-rust.git
cd bee-rust

# কম্পাইল করুন
cargo build --workspace

# টেস্ট চালান
cargo test --workspace
```

### দ্রুত শুরু

```bash
# CLI দিয়ে নতুন প্রজেক্ট তৈরি করুন
cargo run -p bee_cli -- new hello
cd hello

# ডেভ সার্ভার চালান
cargo run
```

### প্রজেক্টে ব্যবহার

```toml
[dependencies]
bee_rust = { git = "https://github.com/erikwang2013/bee-rust", features = ["full"] }
```

## প্রযুক্তিগত নোট

### টেকনোলজি স্ট্যাক

| স্তর | প্রযুক্তি |
|----|------|
| HTTP ভিত্তি | axum 0.8 + tower 0.5 |
| অ্যাসিঙ্ক রানটাইম | tokio 1.x |
| সিরিয়ালাইজেশন | serde + serde_json |
| টেমপ্লেট ইঞ্জিন | tera 1.x |
| লগের নিচতলা | tracing + tracing-subscriber |
| CLI | clap 4 |
| কনফিগ পার্সিং | toml / serde_yaml / নিজস্ব INI |
| এরর হ্যান্ডলিং | thiserror |
| প্রসিডিউরাল ম্যাক্রো | syn + quote + proc-macro2 |

### ডিজাইন প্যাটার্ন

| প্যাটার্ন | প্রয়োগ |
|------|------|
| **Builder** | Logger, Router, QuerySet |
| **Trait অ্যাবস্ট্রাকশন** | Cache, KvStore, SearchEngine, GraphDB, TimeSeriesDB |
| **ডেরাইভ ম্যাক্রো** | `#[derive(Model)]`, `#[derive(Config)]` |
| **Feature Gate** | ড্রাইভার বাস্তবায়ন প্রয়োজন অনুযায়ী কম্পাইল হয় (redis, memcached, elasticsearch ইত্যাদি) |
| **Filter Chain** | রিকোয়েস্ট ফিল্টার চেইন, Beego Filter-এর সমতুল্য |

### Crate তালিকা

| Crate | কাজ | Beego-র সমতুল্য |
|-------|------|-----------|
| `bee_rust` | মেটা crate, একীভূত প্রবেশপথ | — |
| `bee_router` | রাউটিং + কন্ট্রোলার + Context + ফিল্টার | `server/web`, `context` |
| `bee_orm` | ORM + QuerySet + Migration | `client/orm` |
| `bee_kv` | KV/Cache একীভূত অ্যাবস্ট্রাকশন | `client/cache` (সম্প্রসারিত) |
| `bee_search` | সার্চ/অ্যানালিটিক্স ইঞ্জিন | — (নতুন) |
| `bee_graph` | গ্রাফ ডেটাবেস | — (নতুন) |
| `bee_tsdb` | টাইম-সিরিজ ডেটাবেস | — (নতুন) |
| `bee_config` | কনফিগ ম্যানেজমেন্ট + হট-রিলোড | `client/config` |
| `bee_cache` | ক্যাশ অ্যাবস্ট্রাকশন | `client/cache` |
| `bee_session` | Session ম্যানেজমেন্ট | `server/web/session` |
| `bee_logs` | লগিং | `logs` |
| `bee_template` | টেমপ্লেট রেন্ডারিং | — (উন্নত) |
| `bee_cli` | CLI টুল | `bee` টুল |

### টেস্ট কভারেজ

পুরো রিপোজিটরির ৬৮টি টেস্ট পাস:

| Crate | টেস্ট সংখ্যা |
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

## সমর্থন স্বাগত

এই প্রজেক্টটি যদি তোমার কাজে লাগে, QR কোড স্ক্যান করে দান করে সমর্থন করতে পারো, ধন্যবাদ!

**WeChat Pay**

<img src="weixinpay.png" width="160" height="175" alt="WeChat Pay">

**Alipay**

<img src="alipay.png" width="160" height="175" alt="Alipay">

**গ্লোবাল ট্রান্সফার (Bank Transfer)**

বিদেশি ব্যবহারকারীরা ব্যাংক ট্রান্সফারের মাধ্যমে সমর্থন করতে পারেন:

**প্রাপকের তথ্য**

| আইটেম | বিবরণ |
|------|------|
| প্রাপকের নাম | WANG KEXUN |
| প্রাপকের অ্যাকাউন্ট নম্বর | 881015918251 |

**প্রাপক ব্যাংক**

| আইটেম | বিবরণ |
|------|------|
| SWIFT Code | AABLHKHHXXX |
| ব্যাংকের নাম | ZA Bank Limited |
| ব্যাংক কোড | 387 |
| ব্যাংকের ঠিকানা | Core F, Cyberport 3, 100 Cyberport Road, Hong Kong |

**ক্রস-বর্ডার রেমিট্যান্স এজেন্ট ব্যাংক (প্রয়োজন হলে)**

> নোট: এটি ক্রস-বর্ডার রেমিট্যান্স এজেন্ট ব্যাংক (মধ্যস্থ ব্যাংক) এর তথ্য, প্রাপক ব্যাংকের তথ্য নয়। তোমার রেমিট্যান্স ব্যাংককে জিজ্ঞেস করো এটা দরকার কি না।

- **HKD, CNY ও USD-তে পাঠালে** (এজেন্ট ব্যাংক Citibank):

| আইটেম | বিবরণ |
|------|------|
| ব্যাংকের নাম | Citibank N.A. Hong Kong |
| SWIFT Code | CITIHKHXXXX |
| ব্যাংক কোড | 006 |
| শাখার নাম | Hong Kong Branch |
| শাখা কোড | 391 |
| ব্যাংকের ঠিকানা | Citibank Tower, Citibank Plaza, 3 Garden Road, Central, Hong Kong |

- **অন্যান্য মুদ্রায় পাঠালে** (এজেন্ট ব্যাংক BNY Mellon):

| আইটেম | বিবরণ |
|------|------|
| ব্যাংকের নাম | THE BANK OF NEW YORK MELLON |
| SWIFT Code | IRVTUS3NXXX |
| ব্যাংকের ঠিকানা | THE BANK OF NEW YORK MELLON, 240 GREENWICH STREET, NEW YORK, United States |

### লাইসেন্স

Apache-2.0
