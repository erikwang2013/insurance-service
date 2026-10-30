# سجل التغييرات

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

## [1.0.6] — 2026-08-07

### أُضيف
- تنفيذات حقيقية لـ `bee_cli`: `new` (سقالات المشروع)، `generate controller/model`، `run` مع إعادة تحميل ساخنة `--watch`، `pack` (بناء release + نسخ إلى `dist/`)
- اختبارات وحدة CLI للسقالات وتوليد الكود (7 اختبارات جديدة)

### أُصلح
- `bee_rust::init()` أصبح الآن خلف feature `logs` — البناءات المختزلة الميزات (مثل `--no-default-features --features kv`) تُترجم مجددًا
- تحذير Clippy `unnecessary_map_or` في `bee_kv::InMemoryKvStore::exists`
- أُزيلت خيارات `rustfmt.toml` الخاصة بـ nightly التي كانت تُتجاهل بصمت على stable؛ المكدس الآن يجتاز `cargo fmt --all --check`
- ثنائي `bee_cli` الآن `doc = false` لإزالة تضارب اسم مخرجات rustdoc مع `bee_rust`
- منفذ مثال `hello` أصبح قابلاً للتكوين عبر متغير البيئة `PORT`

### تغيّر
- `bee-rust migrate` يبلغ "غير منفذ" ويخرج برمز خروج غير صفري (مخطط)
- تحديث README / README.en ليصفا سلوك CLI الفعلي

## [1.0.4] — 2026-07-29

### أُضيف
- فلتر كشف هجمات الأمان عبر `security-rust` (27 أداة كشف)
- `SecurityFilter` بتغطية XSS و SQL injection و command injection و path traversal
- feature flag `security` في `bee_rust` و `bee_router`

### تغيّر
- تحديث README بتوثيق ميزة الأمان
- تحديث README بقسم دعم الدفع (WeChat Pay / Alipay)

### أُصلح
- صيغة Tera للمعرّفات الأولية في `bee_template` لإصدار Rust 2024

## [1.0.3] — 2026-07-29

### أُضيف
- بنية workspace أولية بـ 13 crate
- توجيه MVC عبر `Controller` trait و `Router`
- ORM عبر منشئ `QuerySet` وماكرو اشتقاق `Model`
- تجريد traits لـ KV/Cache مع خلفيات Redis و Memory
- إدارة الجلسات مع خلفيات Memory/Redis
- إدارة التكوين بدعم INI/YAML/ENV وإعادة تحميل ساخنة
- تقديم القوالب عبر Tera
- السجلات بتكامل tracing
- سقالات CLI وتوليد الكود
- trait stubs لمحركات Search و Graph والسلاسل الزمنية (برامج التشغيل مخطط لها)

[1.0.4]: https://github.com/erikwang2013/bee-rust/compare/v1.0.3...v1.0.4
[1.0.3]: https://github.com/erikwang2013/bee-rust/releases/tag/v1.0.3
