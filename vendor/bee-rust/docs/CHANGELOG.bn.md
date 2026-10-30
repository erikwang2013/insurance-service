# Changelog

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

## [1.0.6] — 2026-08-07

### যা যোগ হয়েছে
- `bee_cli`-এর বাস্তব বাস্তবায়ন: `new` (প্রজেক্ট স্ক্যাফোল্ডিং), `generate controller/model`, `--watch` হট-রিলোডসহ `run`, `pack` (release বিল্ড + `dist/`-তে কপি)
- স্ক্যাফোল্ডিং ও কোড জেনারেশনের জন্য CLI ইউনিট টেস্ট (৭টি নতুন টেস্ট)

### যা ঠিক করা হয়েছে
- `bee_rust::init()` এখন `logs` feature-এর পেছনে গেট করা হয়েছে — কমানো feature বিল্ড (যেমন `--no-default-features --features kv`) আবার কম্পাইল হয়
- `bee_kv::InMemoryKvStore::exists`-এ Clippy `unnecessary_map_or` lint
- `rustfmt.toml` থেকে নাইটলি-অনলি অপশন সরানো হয়েছে যেগুলো stable-এ নীরবে উপেক্ষা করা হতো; ওয়ার্কস্পেস এখন `cargo fmt --all --check` পাস করে
- `bee_cli` বাইনারিতে `doc = false` যাতে `bee_rust`-এর সাথে rustdoc আউটপুট ফাইলনেম সংঘর্ষ না হয়
- `hello` উদাহরণের পোর্ট এখন `PORT` env ভেরিয়েবলের মাধ্যমে কনফিগারযোগ্য

### যা পরিবর্তন করা হয়েছে
- `bee-rust migrate` "not implemented" রিপোর্ট করে এবং নন-জিরো দিয়ে বেরিয়ে যায় (পরিকল্পনাধীন)
- README / README.en আপডেট করে প্রকৃত CLI আচরণ বর্ণনা করা হয়েছে

## [1.0.4] — 2026-07-29

### যা যোগ হয়েছে
- `security-rust`-এর মাধ্যমে নিরাপত্তা আক্রমণ শনাক্তকরণ ফিল্টার (২৭টি ডিটেক্টর)
- XSS, SQL ইনজেকশন, কমান্ড ইনজেকশন, পাথ ট্রাভার্সাল কভারেজসহ `SecurityFilter`
- `bee_rust` ও `bee_router`-এ `security` feature flag

### যা পরিবর্তন করা হয়েছে
- README আপডেট করে নিরাপত্তা ফিচারের ডকুমেন্টেশন যোগ করা হয়েছে
- README আপডেট করে পেমেন্ট সাপোর্ট সেকশন যোগ করা হয়েছে (WeChat Pay / Alipay)

### যা ঠিক করা হয়েছে
- Rust 2024 এডিশনের জন্য `bee_template`-এ Tera র-আইডেন্টিফায়ার সিনট্যাক্স

## [1.0.3] — 2026-07-29

### যা যোগ হয়েছে
- ১৩টি crate-সহ প্রাথমিক ওয়ার্কস্পেস কাঠামো
- `Controller` trait ও `Router`-সহ MVC রাউটিং
- `QuerySet` বিল্ডার ও `Model` ডেরাইভ ম্যাক্রোসহ ORM
- Redis ও Memory ব্যাকএন্ডসহ KV/Cache trait অ্যাবস্ট্রাকশন
- Memory/Redis ব্যাকএন্ডসহ সেশন ম্যানেজমেন্ট
- INI/YAML/ENV সাপোর্ট ও হট-রিলোডসহ কনফিগ ম্যানেজমেন্ট
- Tera-র মাধ্যমে টেমপ্লেট রেন্ডারিং
- tracing ইন্টিগ্রেশনসহ লগিং
- CLI স্ক্যাফোল্ডিং ও কোড জেনারেশন
- সার্চ, গ্রাফ, টাইম-সিরিজ ইঞ্জিন trait stub (ড্রাইভার পরিকল্পনাধীন)

[1.0.4]: https://github.com/erikwang2013/bee-rust/compare/v1.0.3...v1.0.4
[1.0.3]: https://github.com/erikwang2013/bee-rust/releases/tag/v1.0.3
