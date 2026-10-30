<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Beerust

[简体中文](../README.md) · [English](README.en.md) · [한국어](README.ko.md) · [Русский](README.ru.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Português](README.pt.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [বাংলা](README.bn.md) · [Bahasa Indonesia](README.id.md) · [日本語](README.ja.md)

Beerust एक Rust भाषा का production-grade Web फ्रेमवर्क है, जिसका डिज़ाइन दर्शन Go के Beego फ्रेमवर्क से लिया गया है, और इसे Rust के विशिष्ट trait, macro और type system के साथ पुनः व्यक्त किया गया है।

## परियोजना शुभंकर: Rusty

<img src="rusty.svg" width="360" alt="Rusty सामने से मँडराता हुआ, पंख गोलाकार फैले हुए, अगले पंजों में जंग-नारंगी टूलबॉक्स पकड़े हुए">

### चरित्र कार्ड

**नाम** Rusty (जंगी) — Rust के जंगी रंग के नाम पर, और वही जंग जो इसके काम वाले पतलून पर सनी हुई है।

**रूप**: एक रोएँदार इंजीनियर मधुमक्खी। एम्बर-सुनहरा रोआँ गोल-गोल पेट को ढँके हुए है, तीन कोयला-काली धारियाँ टेढ़ी बंधी काम वाली पट्टियों जैसी; चार पारदर्शी पंख इतने पतले कि रोशनी उनमें से छन जाए, मँडराते समय धुंधली छाया गूँजती है; संयुक्त आँखें काँच के मोतियों जैसे दो काल-पाषाण (obsidian), हर एक पर एक हाइलाइट; अगले पंजों में एक घिसकर चमकता जंग-नारंगी टूलबॉक्स — यही इसकी पराग-टोकरी है, जिसमें कभी पराग नहीं, बल्कि डिपेंडेंसीज़ भरी रहती हैं।

**व्यक्तित्व**: काम-दीवाना, हँसमुख, हल्का-सा जुनूनी-बाध्यकारी (OCD)।

- छत्ते की मरम्मत करते समय एंटीना से हर कोशिका की जाँच करता है, जैसे lint चलाने जैसी पक्की सूझबूझ
- पराग-टोकरी में डिपेंडेंसी के संस्करण कसकर लॉक रखता है, कोई उन्हें छुए तो भिड़ जाता है
- bug का पीछा करते समय हवा में स्थिर मँडराता है, पंख इतनी तेज़ी से फड़फड़ाते हैं कि केवल एक धुंधला गोला दिखता है
- नया संस्करण जारी करने के दिन छत्ते के द्वार पर मधुमक्खी का 8-आकार का झूमता नृत्य करता है

### दृश्य विशिष्टताएँ

| तत्व | विशिष्टता |
|------|------|
| मुख्य शरीर | एम्बर-सुनहरा `#F5B301`, पेट का रोआँ छोटा और घना, किनारों पर हाइलाइट की परत |
| धारियाँ | कोयला-काला `#1F1A17`, तीन, केवल पेट पर |
| पंख | पारदर्शी हल्का नीला `#CFE8F5`, 40% अपारदर्शिता, शिराएँ दिखती हैं |
| संयुक्त आँखें | गहरा भूरा-काला `#2A1E16`, अंडाकार, ऊपर-बाएँ कोने में एक हाइलाइट |
| सजावट | जंग-नारंगी `#B7410E` (टूलबॉक्स, बो-टाई), जंग का रंग इसकी पहचान है |

### भाव-चिह्न सेट

| भाव | चित्र | कब |
|------|------|---------|
| ऊर्जा से भरपूर | `(^o^)` रोआँ खड़ा, पंख तेज़ आवृत्ति से काँपते | जब सभी परीक्षण पास होते हैं |
| मँडरा-डिबगिंग | `o(°▽°)o` हवा में मँडराता, केवल पंखों की धुंधली छाया | जब किसी पेचीदा bug का पीछा कर रहा हो |
| भरपूर वापसी | `(≧▽≦)` टूलबॉक्स उभरा हुआ, टेढ़ा-मेढ़ा उड़ता | जब नया ड्राइवर सफलतापूर्वक जुड़ता है |
| ऊँघना | `(-_-)zZ` छत्ते के द्वार पर बैठा, एंटीना झुके हुए | देर रात जब कोई ट्रैफ़िक नहीं होता |
| भड़कना | `(#°益°)` डंक खड़ा | जब सुरक्षा फ़िल्टर किसी हमले को रोकता है |

### विशिष्ट हरकतें

1. **मँडरा-डिबगिंग** — bug मिलते ही कोड के ऊपर मँडराता है, पंख गूँजती हुई धुंधली छाया बन जाती हैं, और तब तक घूरता रहता है जब तक तुम ठीक न कर दो।
2. **8-आकार का झूमता नृत्य** — हर नए संस्करण के जारी होने पर छत्ते के चारों ओर पूरा चक्कर लगाकर नाचता है, पूरे छत्ते को घोषित करता है "फूल खिल गए हैं"। असली मधुमक्खियाँ इससे फूलों के स्रोत की स्थिति बताती हैं, Rusty इससे changelog पहुँचाता है।
3. **संवारना** — अगले पंजों से बार-बार एंटीना साफ़ करता है। तुम सोचते हो यह सज-धज रहा है, असल में यह जाँच रहा है कि कॉन्फ़िगरेशन का हॉट-अपडेट हुआ या नहीं।

### यह कहाँ-कहाँ दिखाई देगा

- **Logo**: सामने से मँडराती मुद्रा, पंख गोलाकार फैले, बिल्कुल नाम को घेरते हुए।
- **404 पेज**: एक खाली फूलों के खेत पर मँडराता, हैरानी से चक्कर लगाता — "शहद कहाँ है?"
- **CLI**: स्टार्टअप बैनर पर एक छोटी मधुमक्खी, परीक्षण चलते समय पास बैठकर निगरानी करती।
- **रिलीज़ घोषणा**: वह तस्वीर जिसमें वह 8-आकार का नृत्य कर रहा है।

## डिज़ाइन लक्ष्य

| लक्ष्य | मानदंड |
|------|------|
| **डेवलपर अनुभव** | `bee-rust new` से पहली रिक्वेस्ट तक < 30 सेकंड |
| **प्रदर्शन** | कंट्रोलर परत का ओवरहेड < 5% (नंगे axum की तुलना में), P99 रूटिंग विलंबता < 100µs |
| **संकलन गति** | मेटा crate का पूर्ण संकलन < 60s (release), वृद्धिशील संकलन < 5s |
| **बाइनरी आकार** | न्यूनतम ऐप (केवल router) < 5MB (strip + LTO) |
| **सुरक्षा** | 0 unsafe बिज़नेस कोड; सभी FFI अलग `*-sys` crates में लिपटे हुए |
| **अनुकूलता** | Rust 1.80+ MSRV, stable का अनुसरण करता है |

## डिज़ाइन सिद्धांत

1. **Beego का दर्शन, Rust की अभिव्यक्ति** — MVC, नेमस्पेस, फ़िल्टर चेन, trait + macro से लागू
2. **क्रमिक वृद्धि** — न्यूनतम कोर केवल axum + tokio पर निर्भर, बाकी सब feature gate
3. **स्पष्टता, अंतर्निहितता से बेहतर** — रूट रजिस्ट्रेशन, मॉडल मैपिंग, मिडलवेयर क्रम — सब कोड में स्पष्ट घोषित
4. **शून्य-लागत अमूर्तता** — trait स्थिर डिस्पैच, macro संकलन-समय विस्तार, कोई वर्चुअल फ़ंक्शन ओवरहेड नहीं
5. **स्टोरेज इंजन स्वतंत्रता** — हर इंजन का trait अलग से बदला जा सकता है, ऊपरी बिज़नेस पर कोई असर नहीं
6. **अंतर्निहित अवलोकनीयता** — tracing + metrics पूरे फ्रेमवर्क में फैले, संरचित लॉग डिफ़ॉल्ट रूप से चालू

## आर्किटेक्चर डिज़ाइन

### Crate टोपोलॉजी

```
bee_rust/           # मेटा crate, re-export + feature flags
bee_router/         # रूटिंग + कंट्रोलर + Context + फ़िल्टर चेन
bee_orm/            # ORM — Model trait + QuerySet + Migration + संबंध मैपिंग
bee_kv/             # KV/Cache एकीकृत अमूर्तता — Redis + Memcached
bee_search/         # खोज/विश्लेषण इंजन — Elasticsearch + OpenSearch + ClickHouse
bee_graph/          # ग्राफ़ डेटाबेस — Neo4j + NebulaGraph + ArangoDB
bee_tsdb/           # टाइम-सीरीज़ डेटाबेस — InfluxDB + Apache IoTDB + QuestDB
bee_config/         # कॉन्फ़िगरेशन प्रबंधन — INI/YAML/ENV + हॉट-अपडेट
bee_cache/          # कैश अमूर्तता — Memory/Redis/Memcache
bee_session/        # Session — Memory/Redis/Cookie/Database बैकएंड
bee_logs/           # लॉगिंग — बहु-स्तरीय लॉग + tracing एकीकरण
bee_template/       # टेम्पलेट रेंडरिंग — tera पर आधारित
bee_cli/            # CLI — स्कैफोल्डिंग/कोड जनरेशन/डेव रन/पैकेजिंग (माइग्रेशन योजनाबद्ध)
```

### आर्किटेक्चर आरेख

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

### Crate निर्भरताएँ

```
bee_config (कोई निर्भरता नहीं)
bee_logs   (कोई निर्भरता नहीं)
bee_cache  → bee_config
bee_kv     → bee_config
bee_session → bee_cache, bee_config
bee_template (कोई निर्भरता नहीं)
bee_orm    → bee_config, bee_cache
bee_search → bee_config
bee_graph  → bee_config
bee_tsdb   → bee_config
bee_router → bee_session, bee_template, bee_config, bee_logs
bee_cli    → bee_router, bee_orm
bee_rust   → उपरोक्त सभी crates (re-export)
```

### समर्थित डेटाबेस

| श्रेणी | डेटाबेस | संबंधित Crate | Feature Flag |
|------|--------|-----------|-------------|
| **रिलेशनल** | SQLite | `bee_orm` | `sqlite` |
| | PostgreSQL | `bee_orm` | `postgres` |
| | MySQL | `bee_orm` | `mysql` |
| | TiDB | `bee_orm` | `mysql` |
| **KV / कैश** | Redis | `bee_kv` / `bee_cache` | `redis` |
| | Memcached | `bee_kv` / `bee_cache` | `memcache` |
| **खोज / विश्लेषण** | Elasticsearch | `bee_search` | `elasticsearch` |
| | OpenSearch | `bee_search` | `opensearch` |
| | ClickHouse | `bee_search` | `clickhouse` |
| **ग्राफ़ डेटाबेस** | Neo4j | `bee_graph` | `neo4j` |
| | NebulaGraph | `bee_graph` | `nebulagraph` |
| | ArangoDB | `bee_graph` | `arangodb` |
| **टाइम-सीरीज़ डेटाबेस** | InfluxDB | `bee_tsdb` | `influxdb` |
| | Apache IoTDB | `bee_tsdb` | `iotdb` |
| | QuestDB | `bee_tsdb` | `questdb` |

### अनुरोध फ़िल्टर चेन

```
अनुरोध → [SecurityFilter हमला-पता लगाना] → [Session पुनर्स्थापना] → [पैरामीटर सत्यापन] → [prepare हुक] → [handle प्रोसेसिंग] → [finish हुक] → प्रतिक्रिया
                  ↓ कोई भी चरण बाधित हो सकता है (Beego के Abort के समान)
```

## फ़ीचर परिचय

फ़ीचर अवलोकन नीचे दिया गया है; प्रत्येक मॉड्यूल के विस्तृत उपयोग, कोड उदाहरण और API विवरण के लिए [API संदर्भ](api.hi.md) देखें।

### Web कोर (bee_router)

MVC कंट्रोलर: `Controller` trait + `Context`, रूट नेमस्पेस, RESTful विधि रजिस्ट्रेशन और अनुरोध फ़िल्टर चेन का समर्थन करता है।

- प्रतिक्रिया आउटपुट: `ctx.json()` / `ctx.text()` / `ctx.html()`
- रीडायरेक्ट / रोकना: `ctx.redirect()` / `ctx.abort()`
- सत्र और पैरामीटर: `ctx.session` / `ctx.params`

### सुरक्षा जाँच (`security` feature)

[security-rust](https://crates.io/crates/security-rust) पर आधारित हमला-पता लगाने वाला फ़िल्टर, जो XSS, SQL इंजेक्शन, कमांड इंजेक्शन, SSRF आदि 27 प्रकार के हमलों को कवर करता है, एक पंक्ति में चालू:

```rust
let security = SecurityFilter::new();  // सभी 27 डिटेक्टर चालू
```

### ORM (bee_orm)

`#[derive(Model)]` डिराइव मैक्रो + QuerySet चेन क्वेरी (filter / order_by / limit), SQLite, PostgreSQL, MySQL, TiDB का समर्थन करता है।

### कॉन्फ़िगरेशन प्रबंधन (bee_config)

`#[derive(Config)]` डिराइव मैक्रो, INI / YAML / ENV लोडिंग और हॉट-अपडेट का समर्थन करता है।

### स्टोरेज इंजन

KV / Cache (Redis + Memcached), खोज इंजन (Elasticsearch / OpenSearch / ClickHouse), ग्राफ़ डेटाबेस (Neo4j / NebulaGraph / ArangoDB), टाइम-सीरीज़ डेटाबेस (InfluxDB / IoTDB / QuestDB) — सभी एकीकृत trait अमूर्तता, ड्राइवर feature gate के अनुसार संकलित होते हैं।

### Session, लॉग, टेम्पलेट

- Session: Memory / Redis / Cookie / Database कई बैकएंड
- लॉग: बहु-स्तरीय लॉग + tracing एकीकरण
- टेम्पलेट: tera पर आधारित रेंडरिंग

### CLI उपकरण

```bash
bee-rust new my-app            # प्रोजेक्ट स्कैफोल्ड बनाएँ
bee-rust generate controller user
bee-rust run --watch           # डेव रन (हॉट रीलोड)
bee-rust pack                  # पैकेज और डिप्लॉय
```

## उपयोग के चरण

### पर्यावरण आवश्यकताएँ

- Rust 1.80+
- Cargo

### स्थापना

```bash
# प्रोजेक्ट क्लोन करें
git clone https://github.com/erikwang2013/bee-rust.git
cd bee-rust

# संकलन करें
cargo build --workspace

# परीक्षण चलाएँ
cargo test --workspace
```

### त्वरित आरंभ

```bash
# CLI से नया प्रोजेक्ट बनाएँ
cargo run -p bee_cli -- new hello
cd hello

# डेवलपमेंट सर्वर चलाएँ
cargo run
```

### प्रोजेक्ट में उपयोग

```toml
[dependencies]
bee_rust = { git = "https://github.com/erikwang2013/bee-rust", features = ["full"] }
```

## तकनीकी विवरण

### तकनीकी स्टैक

| परत | तकनीक |
|----|------|
| HTTP आधार | axum 0.8 + tower 0.5 |
| एसिंक रनटाइम | tokio 1.x |
| सीरियलाइज़ेशन | serde + serde_json |
| टेम्पलेट इंजन | tera 1.x |
| लॉगिंग आधार | tracing + tracing-subscriber |
| CLI | clap 4 |
| कॉन्फ़िगरेशन पार्सिंग | toml / serde_yaml / स्व-निर्मित INI |
| एरर हैंडलिंग | thiserror |
| प्रोसीजर मैक्रो | syn + quote + proc-macro2 |

### डिज़ाइन पैटर्न

| पैटर्न | अनुप्रयोग |
|------|------|
| **Builder** | Logger, Router, QuerySet |
| **Trait अमूर्तता** | Cache, KvStore, SearchEngine, GraphDB, TimeSeriesDB |
| **डिराइव मैक्रो** | `#[derive(Model)]`, `#[derive(Config)]` |
| **Feature Gate** | ड्राइवर कार्यान्वयन आवश्यकतानुसार संकलित (redis, memcached, elasticsearch आदि) |
| **Filter Chain** | अनुरोध फ़िल्टर चेन, Beego Filter के समकक्ष |

### Crate सूची

| Crate | कार्य | Beego समकक्ष |
|-------|------|-----------|
| `bee_rust` | मेटा crate, एकीकृत प्रवेश बिंदु | — |
| `bee_router` | रूटिंग + कंट्रोलर + Context + फ़िल्टर | `server/web`, `context` |
| `bee_orm` | ORM + QuerySet + Migration | `client/orm` |
| `bee_kv` | KV/Cache एकीकृत अमूर्तता | `client/cache` (विस्तारित) |
| `bee_search` | खोज/विश्लेषण इंजन | — (नया) |
| `bee_graph` | ग्राफ़ डेटाबेस | — (नया) |
| `bee_tsdb` | टाइम-सीरीज़ डेटाबेस | — (नया) |
| `bee_config` | कॉन्फ़िगरेशन प्रबंधन + हॉट-अपडेट | `client/config` |
| `bee_cache` | कैश अमूर्तता | `client/cache` |
| `bee_session` | Session प्रबंधन | `server/web/session` |
| `bee_logs` | लॉगिंग | `logs` |
| `bee_template` | टेम्पलेट रेंडरिंग | — (वर्धित) |
| `bee_cli` | CLI उपकरण | `bee` उपकरण |

### परीक्षण कवरेज

पूरे रिपॉज़िटरी में 68 परीक्षण पास:

| Crate | परीक्षण संख्या |
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

## समर्थन का स्वागत है

अगर यह प्रोजेक्ट आपके काम आया है, तो क्यूआर कोड स्कैन करके दान के रूप में समर्थन करने का स्वागत है, धन्यवाद!

**WeChat Pay**

<img src="weixinpay.png" width="160" height="175" alt="WeChat Pay">

**Alipay**

<img src="alipay.png" width="160" height="175" alt="Alipay">

**वैश्विक बैंक ट्रांसफ़र (Bank Transfer)**

विदेशी उपयोगकर्ता बैंक ट्रांसफ़र से समर्थन कर सकते हैं:

**लाभार्थी जानकारी**

| फ़ील्ड | मान |
|------|------|
| लाभार्थी का नाम | WANG KEXUN |
| लाभार्थी खाता संख्या | 881015918251 |

**लाभार्थी बैंक**

| फ़ील्ड | मान |
|------|------|
| SWIFT Code | AABLHKHHXXX |
| बैंक का नाम | ZA Bank Limited |
| बैंक कोड | 387 |
| बैंक का पता | Core F, Cyberport 3, 100 Cyberport Road, Hong Kong |

**अंतर्राष्ट्रीय रेमिटेंस एजेंट बैंक (यदि आवश्यक हो)**

> नोट: यह अंतर्राष्ट्रीय रेमिटेंस के लिए एजेंट (इंटरमीडियरी) बैंक की जानकारी है, लाभार्थी बैंक की नहीं। कृपया अपने रेमिटेंस बैंक से पूछें कि क्या यह आवश्यक है।

- **HKD, RMB और USD में रेमिट करने के लिए** (एजेंट बैंक Citibank):

| फ़ील्ड | मान |
|------|------|
| बैंक का नाम | Citibank N.A. Hong Kong |
| SWIFT Code | CITIHKHXXXX |
| बैंक कोड | 006 |
| शाखा का नाम | Hong Kong Branch |
| शाखा कोड | 391 |
| बैंक का पता | Citibank Tower, Citibank Plaza, 3 Garden Road, Central, Hong Kong |

- **अन्य मुद्राओं में रेमिट करने के लिए** (एजेंट बैंक BNY Mellon):

| फ़ील्ड | मान |
|------|------|
| बैंक का नाम | THE BANK OF NEW YORK MELLON |
| SWIFT Code | IRVTUS3NXXX |
| बैंक का पता | THE BANK OF NEW YORK MELLON, 240 GREENWICH STREET, NEW YORK, United States |

### लाइसेंस

Apache-2.0
