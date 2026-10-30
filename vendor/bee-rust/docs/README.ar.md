<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Beerust

[简体中文](../README.md) · [English](README.en.md) · [한국어](README.ko.md) · [Русский](README.ru.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Português](README.pt.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [বাংলা](README.bn.md) · [Bahasa Indonesia](README.id.md) · [日本語](README.ja.md)

Beerust هو إطار ويب إنتاجي مكتوب بلغة Rust، فلسفة تصميمه مستمدة من إطار Beego بلغة Go، معاد التعبير عنها بأساليب Rust المألوفة: traits و macros وأنظمة الأنواع.

## تعويذة المشروع: Rusty

<img src="rusty.svg" width="360" alt="Rusty يطفو من الأمام، بجناحين ممدودين على شكل حلقة، يحمل صندوق أدوات برتقالي صدئ بقوائمه الأمامية">

### بطاقة الشخصية

**الاسم** Rusty (رستي) — لون صدأ Rust، وهو أيضًا الصدأ الملتصق بملابس عمله.

**المظهر**: نحلة مهندسة فروية. فرو كهرماني ذهبي يغطي بطنًا مستديرًا ممتلئًا، مع ثلاثة خطوط سوداء فحمية تشبه أحزمة ملابس عمل ملتوية؛ أربعة أجنحة شفافة رقيقة لدرجة أن الضوء يخترقها، تطن بآثار بصرية عند التحليق؛ عينان مركبتان تشبهان حبتي سبج زجاجيتين، في كل منهما لمعة؛ تحمل بقوائمها الأمامية صندوق أدوات برتقالي صدئ لامع — إنها سلة حبوب اللقاح الخاصة به، التي لا تحمل حبوب لقاح أبدًا، بل تحمل التبعيات.

**الشخصية**: مدمن عمل، متفائل، مصاب باضطراب قهري خفيف.

- عند إصلاح الخلية، يفحص خلية تلو الأخرى بقرنيه الاستشعاريين، بحرص مثل تشغيل lint
- يقفل إصدارات التبعيات في سلة حبوب اللقاح بإحكام شديد، ومن يلمسها يواجه غضبه
- عند مطاردة خطأ برمجي، يحوم في الهواء بلا حركة، وأجنحته سريعة لدرجة أنها لا تظهر إلا كظل دائري
- في يوم إصدار نسخة جديدة، يرقص رقصة النحل المتمايلة رقم 8 أمام باب الخلية

### المواصفات البصرية

| العنصر | المواصفة |
|------|------|
| الجسم | كهرماني ذهبي `#F5B301`، فراء البطن قصير وكثيف، مع دائرة لمعان على الحواف |
| الخطوط | أسود فحمي `#1F1A17`، ثلاثة خطوط على البطن فقط |
| الأجنحة | أزرق فاتح شفاف `#CFE8F5`، بشفافية 40%، تُرى فيها العروق |
| العيون المركبة | بني غامق يميل للسواد `#2A1E16`، بيضاوية، مع لمعة في الزاوية اليسرى العليا |
| اللمسات | برتقالي صدئ `#B7410E` (صندوق الأدوات، ربطة العنق)، اللون الصدئ هو توقيعه |

### مجموعة التعبيرات

| التعبير | الصورة | متى |
|------|------|---------|
| مفعم بالحيوية | `(^o^)` فراء منتفش، أجنحة تهتز بتردد عالٍ | عندما تجتاز كل الاختبارات |
| تحليق التصحيح | `o(°▽°)o` يحوم في الهواء، لا يظهر إلا ظل الأجنحة | عند مطاردة خطأ برمجي عنيد |
| عائد محمّل | `(≧▽≦)` صندوق الأدوات ممتلئ، يطير متمايلًا | عند نجاح توصيل محرك جديد |
| غفوة | `(-_-)zZ` جاثم عند باب الخلية، قرناه متدليان | في منتصف الليل دون حركة مرور |
| غضب | `(#°益°)` الإبرة الخلفية منتصبة | عندما يعترض فلتر الأمان هجومًا |

### الحركات المميزة

1. **تحليق التصحيح** — عند اكتشاف خطأ برمجي، يحوم فوق الكود، جناحاه يطنان كظل، ويحدق حتى تنتهي من التعديل.
2. **رقصة الـ 8 المتمايلة** — في كل إصدار جديد، يطوف حول الخلية دورة كاملة، معلنًا للخلية كلها "الأزهار تفتحت". ينقل النحل الحقيقي بهذا مواقع مصادر الرحيق، وRusty ينقل بهذا سجل التغييرات.
3. **تمشيط الفراء** — يمسح قرنيه بقوائمه الأمامية مرارًا. تظن أنه يعتني بمظهره، لكنه في الحقيقة يتحقق مما إذا كان التكوين حُدّث ساخنًا.

### أين سيظهر

- **الشعار**: وضعية التحليق الأمامية، أجنحة ممدودة على شكل حلقة تحيط بالاسم تمامًا.
- **صفحة 404**: يحوم فوق حقل أزهار فارغ، يدور في حيرة — "أين العسل؟"
- **CLI**: نحلة صغيرة في لافتة الإقلاع، تجلس بجانبه تراقب أثناء تشغيل الاختبارات.
- **إعلانات الإصدار**: صورة رقصة الـ 8.

## أهداف التصميم

| الهدف | المؤشر |
|------|------|
| **تجربة التطوير** | من `bee-rust new` إلى أول طلب < 30 ثانية |
| **الأداء** | حمل طبقة المتحكم < 5% (مقارنة بـ axum عارية)، زمن استجابة توجيه P99 < 100µs |
| **سرعة الترجمة** | ترجمة كاملة للـ meta crate < 60s (release)، ترجمة تزايدية < 5s |
| **حجم الثنائي** | أصغر تطبيق (router فقط) < 5MB (strip + LTO) |
| **الأمان** | 0 كود unsafe في منطق الأعمال؛ كل FFI مغلف في crates مستقلة `*-sys` |
| **التوافق** | Rust 1.80+ كحد أدنى، متابعة stable |

## مبادئ التصميم

1. **فلسفة Beego، تعبير Rust** — MVC ومساحات الأسماء وسلسلة الفلاتر، منفذة عبر trait + macro
2. **تحسين تدريجي** — النواة الأصغر تعتمد فقط على axum + tokio، والباقي كله خلف feature gates
3. **الصريح أفضل من الضمني** — تسجيل المسارات، تعيين النماذج، ترتيب الوسائط، كلها معلنة صراحة في الكود
4. **تجريدات صفرية التكلفة** — توزيع ثابت عبر traits، توسيع macros في زمن الترجمة، دون حمل افتراضية إضافي
5. **استقلالية محركات التخزين** — كل محرك يمكن استبدال تنفيذه عبر trait بشكل منفصل دون التأثير على الأعمال العلوية
6. **مراقبة مدمجة** — نقاط قياس tracing + metrics تغطي الإطار كله، وتسجيل هيكلي مفعل افتراضيًا

## التصميم المعماري

### طوبولوجيا Crate

```
bee_rust/           # meta crate، re-export + feature flags
bee_router/         # التوجيه + المتحكم + Context + سلسلة الفلاتر
bee_orm/            # ORM — Model trait + QuerySet + Migration + تعيين العلاقات
bee_kv/             # تجريد موحد KV/Cache — Redis + Memcached
bee_search/         # محرك بحث/تحليل — Elasticsearch + OpenSearch + ClickHouse
bee_graph/          # قاعدة بيانات رسومية — Neo4j + NebulaGraph + ArangoDB
bee_tsdb/           # قاعدة بيانات سلاسل زمنية — InfluxDB + Apache IoTDB + QuestDB
bee_config/         # إدارة التكوين — INI/YAML/ENV + تحديث ساخن
bee_cache/          # تجريد التخزين المؤقت — Memory/Redis/Memcache
bee_session/        # الجلسات — خلفيات Memory/Redis/Cookie/Database
bee_logs/           # السجلات — سجلات متعددة المستويات + تكامل tracing
bee_template/       # تقديم القوالب — مبني على tera
bee_cli/            # CLI — سقالات/توليد كود/تشغيل تطويري/تغليف (مخطط للترحيل)
```

### مخطط معماري

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

### تبعيات Crate

```
bee_config (بلا تبعيات)
bee_logs   (بلا تبعيات)
bee_cache  → bee_config
bee_kv     → bee_config
bee_session → bee_cache, bee_config
bee_template (بلا تبعيات)
bee_orm    → bee_config, bee_cache
bee_search → bee_config
bee_graph  → bee_config
bee_tsdb   → bee_config
bee_router → bee_session, bee_template, bee_config, bee_logs
bee_cli    → bee_router, bee_orm
bee_rust   → جميع الـ crates أعلاه (re-export)
```

### قواعد البيانات المدعومة

| الفئة | قاعدة البيانات | الـ Crate المقابل | Feature Flag |
|------|--------|-----------|-------------|
| **علائقية** | SQLite | `bee_orm` | `sqlite` |
| | PostgreSQL | `bee_orm` | `postgres` |
| | MySQL | `bee_orm` | `mysql` |
| | TiDB | `bee_orm` | `mysql` |
| **KV / تخزين مؤقت** | Redis | `bee_kv` / `bee_cache` | `redis` |
| | Memcached | `bee_kv` / `bee_cache` | `memcache` |
| **بحث / تحليل** | Elasticsearch | `bee_search` | `elasticsearch` |
| | OpenSearch | `bee_search` | `opensearch` |
| | ClickHouse | `bee_search` | `clickhouse` |
| **قاعدة بيانات رسومية** | Neo4j | `bee_graph` | `neo4j` |
| | NebulaGraph | `bee_graph` | `nebulagraph` |
| | ArangoDB | `bee_graph` | `arangodb` |
| **سلاسل زمنية** | InfluxDB | `bee_tsdb` | `influxdb` |
| | Apache IoTDB | `bee_tsdb` | `iotdb` |
| | QuestDB | `bee_tsdb` | `questdb` |

### سلسلة فلاتر الطلبات

```
طلب → [SecurityFilter كشف الهجمات] → [استعادة الجلسة] → [التحقق من المعاملات] → [خطاف prepare] → [معالجة handle] → [خطاف finish] → استجابة
                  ↓ أي مرحلة يمكن أن تنقطع (مشابه لـ Abort في Beego)
```

## مقدمة الميزات

نظرة عامة على الميزات أدناه، وللتفاصيل عن كل وحدة وأمثلة الكود وشرح الـ API، انظر [مرجع API](api.ar.md).

### نواة الويب (bee_router)

متحكم MVC: `Controller` trait + `Context`، يدعم مساحات أسماء المسارات، وتسجيل طرق RESTful، وسلسلة فلاتر الطلبات.

- مخرجات الاستجابة: `ctx.json()` / `ctx.text()` / `ctx.html()`
- إعادة التوجيه / المقاطعة: `ctx.redirect()` / `ctx.abort()`
- الجلسة والمعاملات: `ctx.session` / `ctx.params`

### كشف الأمان (feature: `security`)

فلتر كشف الهجمات مبني على [security-rust](https://crates.io/crates/security-rust)، يغطي 27 نوعًا من الهجمات مثل XSS و SQL Injection و Command Injection و SSRF، يُفعّل بسطر واحد:

```rust
let security = SecurityFilter::new();  // تفعيل أدوات الكشف الـ 27 كلها
```

### ORM (bee_orm)

ماكرو الاشتقاق `#[derive(Model)]` + استعلامات متسلسلة QuerySet (filter / order_by / limit)، يدعم SQLite و PostgreSQL و MySQL و TiDB.

### إدارة التكوين (bee_config)

ماكرو الاشتقاق `#[derive(Config)]`، يدعم تحميل INI / YAML / ENV والتحديث الساخن.

### محركات التخزين

تجريد موحد عبر traits لـ KV / Cache (Redis + Memcached)، ومحركات البحث (Elasticsearch / OpenSearch / ClickHouse)، وقواعد البيانات الرسومية (Neo4j / NebulaGraph / ArangoDB)، وقواعد بيانات السلاسل الزمنية (InfluxDB / IoTDB / QuestDB)، تُترجم برامج التشغيل حسب feature gates.

### الجلسات والسجلات والقوالب

- الجلسات: خلفيات متعددة Memory / Redis / Cookie / Database
- السجلات: سجلات متعددة المستويات + تكامل tracing
- القوالب: تقديم مبني على tera

### أدوات CLI

```bash
bee-rust new my-app            # إنشاء سقالات المشروع
bee-rust generate controller user
bee-rust run --watch           # تشغيل تطويري (إعادة تحميل ساخنة)
bee-rust pack                  # تغليف ونشر
```

## خطوات الاستخدام

### متطلبات البيئة

- Rust 1.80+
- Cargo

### التثبيت

```bash
# استنساخ المشروع
git clone https://github.com/erikwang2013/bee-rust.git
cd bee-rust

# الترجمة
cargo build --workspace

# تشغيل الاختبارات
cargo test --workspace
```

### بداية سريعة

```bash
# إنشاء مشروع جديد باستخدام CLI
cargo run -p bee_cli -- new hello
cd hello

# تشغيل خادم التطوير
cargo run
```

### الاستخدام في مشروعك

```toml
[dependencies]
bee_rust = { git = "https://github.com/erikwang2013/bee-rust", features = ["full"] }
```

## ملاحظات تقنية

### المكدس التقني

| الطبقة | التقنية |
|----|------|
| قاعدة HTTP | axum 0.8 + tower 0.5 |
| وقت تشغيل غير متزامن | tokio 1.x |
| التسلسل | serde + serde_json |
| محرك القوالب | tera 1.x |
| أساس السجلات | tracing + tracing-subscriber |
| CLI | clap 4 |
| تحليل التكوين | toml / serde_yaml / INI مخصص |
| معالجة الأخطاء | thiserror |
| ماكرو الإجراءات | syn + quote + proc-macro2 |

### أنماط التصميم

| النمط | التطبيق |
|------|------|
| **Builder** | Logger, Router, QuerySet |
| **تجريد Trait** | Cache, KvStore, SearchEngine, GraphDB, TimeSeriesDB |
| **ماكرو اشتقاق** | `#[derive(Model)]`, `#[derive(Config)]` |
| **Feature Gate** | ترجمة التطبيقات حسب الحاجة (redis, memcached, elasticsearch ...) |
| **سلسلة الفلاتر** | سلسلة فلاتر الطلبات، مقابل Beego Filter |

### قائمة Crate

| Crate | الوظيفة | مقابل Beego |
|-------|------|-----------|
| `bee_rust` | meta crate، مدخل موحد | — |
| `bee_router` | توجيه + متحكم + Context + فلاتر | `server/web`, `context` |
| `bee_orm` | ORM + QuerySet + Migration | `client/orm` |
| `bee_kv` | تجريد موحد KV/Cache | `client/cache` (موسع) |
| `bee_search` | محرك بحث/تحليل | — (جديد) |
| `bee_graph` | قاعدة بيانات رسومية | — (جديد) |
| `bee_tsdb` | قاعدة بيانات سلاسل زمنية | — (جديد) |
| `bee_config` | إدارة التكوين + تحديث ساخن | `client/config` |
| `bee_cache` | تجريد التخزين المؤقت | `client/cache` |
| `bee_session` | إدارة الجلسات | `server/web/session` |
| `bee_logs` | السجلات | `logs` |
| `bee_template` | تقديم القوالب | — (مُحسّن) |
| `bee_cli` | أدوات CLI | أداة `bee` |

### تغطية الاختبارات

68 اختبارًا ناجحًا في المستودع كله:

| Crate | عدد الاختبارات |
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

## نداء دعم

إذا كان هذا المشروع مفيدًا لك، فمرحبًا بك في مسح رمز الاستجابة السريعة للتبرع والدعم، شكرًا!

**WeChat Pay**

<img src="weixinpay.png" width="160" height="175" alt="WeChat Pay">

**Alipay**

<img src="alipay.png" width="160" height="175" alt="Alipay">

**التحويل المصرفي العالمي (Bank Transfer)**

يمكن للمستخدمين في الخارج تقديم الدعم عبر التحويل المصرفي:

**معلومات المستلم**

| الحقل | القيمة |
|------|------|
| اسم المستلم | WANG KEXUN |
| رقم حساب المستلم | 881015918251 |

**البنك المتلقي**

| الحقل | القيمة |
|------|------|
| SWIFT Code | AABLHKHHXXX |
| اسم البنك | ZA Bank Limited |
| رقم البنك | 387 |
| عنوان البنك | Core F, Cyberport 3, 100 Cyberport Road, Hong Kong |

**البنك الوكيل للتحويلات العابرة للحدود (عند الحاجة)**

> ملاحظة: هذه معلومات البنك الوكيل للتحويلات العابرة للحدود (البنك الوسيط)، وليست معلومات البنك المتلقي. استفسر من بنكك المرسِل عما إذا كان يتطلب تقديمها.

- **تحويل الدولار الهونغ كونغي واليوان الصيني والدولار الأمريكي** (البنك الوكيل: Citibank):

| الحقل | القيمة |
|------|------|
| اسم البنك | Citibank N.A. Hong Kong |
| SWIFT Code | CITIHKHXXXX |
| رقم البنك | 006 |
| اسم الفرع | Hong Kong Branch |
| رقم الفرع | 391 |
| عنوان البنك | Citibank Tower, Citibank Plaza, 3 Garden Road, Central, Hong Kong |

- **تحويل العملات الأخرى** (البنك الوكيل: BNY Mellon):

| الحقل | القيمة |
|------|------|
| اسم البنك | THE BANK OF NEW YORK MELLON |
| SWIFT Code | IRVTUS3NXXX |
| عنوان البنك | THE BANK OF NEW YORK MELLON, 240 GREENWICH STREET, NEW YORK, United States |

### الترخيص

Apache-2.0
