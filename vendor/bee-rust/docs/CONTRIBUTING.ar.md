# المساهمة

[简体中文](CONTRIBUTING.zh.md) · [English](CONTRIBUTING.md) · [한국어](CONTRIBUTING.ko.md) · [Русский](CONTRIBUTING.ru.md) · [Deutsch](CONTRIBUTING.de.md) · [Français](CONTRIBUTING.fr.md) · [Español](CONTRIBUTING.es.md) · [Português](CONTRIBUTING.pt.md) · [हिन्दी](CONTRIBUTING.hi.md) · [العربية](CONTRIBUTING.ar.md) · [বাংলা](CONTRIBUTING.bn.md) · [Bahasa Indonesia](CONTRIBUTING.id.md) · [日本語](CONTRIBUTING.ja.md)

## الإعداد

```bash
git clone https://github.com/erikwang2013/bee-rust.git
cd bee-rust
cargo build --workspace
cargo test --workspace
```

## قبل التقديم

- شغّل `cargo fmt --all` لتنسيق الكود
- شغّل `cargo clippy --workspace -- -D warnings` للتدقيق
- شغّل `cargo test --workspace` للتأكد من نجاح كل الاختبارات
- أبقِ الملفات أقل من 500 سطر

## هيكل المشروع

```
crates/
  bee_rust/         # Meta crate، re-export + feature flags
  bee_router/       # التوجيه + المتحكم + Context + سلسلة الفلاتر
  bee_orm/          # ORM — Model trait + QuerySet + Migration
  bee_kv/           # تجريد موحد KV/Cache
  bee_search/       # محرك بحث/تحليل
  bee_graph/        # قاعدة بيانات رسومية
  bee_tsdb/         # قاعدة بيانات سلاسل زمنية
  bee_config/       # إدارة التكوين + إعادة تحميل ساخنة
  bee_cache/        # تجريد التخزين المؤقت
  bee_session/      # إدارة الجلسات
  bee_logs/         # السجلات
  bee_template/     # تقديم القوالب
  bee_cli/          # أدوات CLI
```

## الترخيص

Apache-2.0
