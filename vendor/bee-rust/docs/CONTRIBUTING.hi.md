# योगदान

[简体中文](CONTRIBUTING.zh.md) · [English](CONTRIBUTING.md) · [한국어](CONTRIBUTING.ko.md) · [Русский](CONTRIBUTING.ru.md) · [Deutsch](CONTRIBUTING.de.md) · [Français](CONTRIBUTING.fr.md) · [Español](CONTRIBUTING.es.md) · [Português](CONTRIBUTING.pt.md) · [हिन्दी](CONTRIBUTING.hi.md) · [العربية](CONTRIBUTING.ar.md) · [বাংলা](CONTRIBUTING.bn.md) · [Bahasa Indonesia](CONTRIBUTING.id.md) · [日本語](CONTRIBUTING.ja.md)

## सेटअप

```bash
git clone https://github.com/erikwang2013/bee-rust.git
cd bee-rust
cargo build --workspace
cargo test --workspace
```

## जमा करने से पहले

- कोड को फ़ॉर्मेट करने के लिए `cargo fmt --all` चलाएँ
- लिंट के लिए `cargo clippy --workspace -- -D warnings` चलाएँ
- यह सत्यापित करने के लिए कि सभी परीक्षण पास हुए, `cargo test --workspace` चलाएँ
- फ़ाइलें 500 पंक्तियों से कम रखें

## प्रोजेक्ट संरचना

```
crates/
  bee_rust/         # मेटा crate, re-export + feature flags
  bee_router/       # रूटिंग + कंट्रोलर + Context + फ़िल्टर चेन
  bee_orm/          # ORM — Model trait + QuerySet + Migration
  bee_kv/           # KV/Cache एकीकृत अमूर्तता
  bee_search/       # खोज/विश्लेषण इंजन
  bee_graph/        # ग्राफ़ डेटाबेस
  bee_tsdb/         # टाइम-सीरीज़ डेटाबेस
  bee_config/       # कॉन्फ़िगरेशन प्रबंधन + हॉट-रीलोड
  bee_cache/        # कैश अमूर्तता
  bee_session/      # सत्र प्रबंधन
  bee_logs/         # लॉगिंग
  bee_template/     # टेम्पलेट रेंडरिंग
  bee_cli/          # CLI उपकरण
```

## लाइसेंस

Apache-2.0
