# परिवर्तन-लॉग

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

## [1.0.6] — 2026-08-07

### जोड़ा गया
- `bee_cli` के वास्तविक कार्यान्वयन: `new` (प्रोजेक्ट स्कैफोल्डिंग), `generate controller/model`, `--watch` हॉट रीलोड के साथ `run`, `pack` (release बिल्ड + `dist/` में कॉपी)
- स्कैफोल्डिंग और कोड जनरेशन के लिए CLI यूनिट परीक्षण (7 नए परीक्षण)

### ठीक किया गया
- `bee_rust::init()` अब `logs` feature के पीछे गेट किया गया — घटे हुए feature बिल्ड (जैसे `--no-default-features --features kv`) फिर से संकलित होते हैं
- `bee_kv::InMemoryKvStore::exists` में Clippy `unnecessary_map_or` लिंट
- `rustfmt.toml` से केवल-nightly विकल्प हटाए गए जो stable पर चुपचाप अनदेखा किए जाते थे; वर्कस्पेस अब `cargo fmt --all --check` पास करता है
- `bee_cli` बाइनरी पर `doc = false` ताकि `bee_rust` के साथ rustdoc आउटपुट फ़ाइलनाम टकराव दूर हो
- `hello` उदाहरण का पोर्ट अब `PORT` env var से कॉन्फ़िगर किया जा सकता है

### बदला गया
- `bee-rust migrate` "not implemented" रिपोर्ट करता है और गैर-शून्य कोड के साथ बाहर निकलता है (योजनाबद्ध)
- README / README.en को वास्तविक CLI व्यवहार बताने के लिए अद्यतन किया गया

## [1.0.4] — 2026-07-29

### जोड़ा गया
- `security-rust` के माध्यम से सुरक्षा हमला-पता लगाने वाला फ़िल्टर (27 डिटेक्टर)
- XSS, SQL इंजेक्शन, कमांड इंजेक्शन, पाथ ट्रैवर्सल कवरेज के साथ `SecurityFilter`
- `bee_rust` और `bee_router` में `security` feature flag

### बदला गया
- सुरक्षा feature दस्तावेज़ीकरण के साथ README अद्यतन
- भुगतान समर्थन अनुभाग (WeChat Pay / Alipay) के साथ README अद्यतन

### ठीक किया गया
- Rust 2024 edition के लिए `bee_template` Tera रॉ आइडेंटिफ़ायर सिंटैक्स

## [1.0.3] — 2026-07-29

### जोड़ा गया
- 13 crates के साथ प्रारंभिक वर्कस्पेस संरचना
- `Controller` trait और `Router` के साथ MVC रूटिंग
- `QuerySet` बिल्डर और `Model` डिराइव मैक्रो के साथ ORM
- Redis और Memory बैकएंड के साथ KV/Cache trait अमूर्तता
- Memory/Redis बैकएंड के साथ सत्र प्रबंधन
- INI/YAML/ENV समर्थन और हॉट-रीलोड के साथ कॉन्फ़िगरेशन प्रबंधन
- Tera के माध्यम से टेम्पलेट रेंडरिंग
- tracing एकीकरण के साथ लॉगिंग
- CLI स्कैफोल्डिंग और कोड जनरेशन
- खोज, ग्राफ़, टाइम-सीरीज़ इंजन trait stubs (ड्राइवर योजनाबद्ध)

[1.0.4]: https://github.com/erikwang2013/bee-rust/compare/v1.0.3...v1.0.4
[1.0.3]: https://github.com/erikwang2013/bee-rust/releases/tag/v1.0.3
