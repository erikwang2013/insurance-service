# কন্ট্রিবিউশন

[简体中文](CONTRIBUTING.zh.md) · [English](CONTRIBUTING.md) · [한국어](CONTRIBUTING.ko.md) · [Русский](CONTRIBUTING.ru.md) · [Deutsch](CONTRIBUTING.de.md) · [Français](CONTRIBUTING.fr.md) · [Español](CONTRIBUTING.es.md) · [Português](CONTRIBUTING.pt.md) · [हिन्दी](CONTRIBUTING.hi.md) · [العربية](CONTRIBUTING.ar.md) · [বাংলা](CONTRIBUTING.bn.md) · [Bahasa Indonesia](CONTRIBUTING.id.md) · [日本語](CONTRIBUTING.ja.md)

## Setup

```bash
git clone https://github.com/erikwang2013/bee-rust.git
cd bee-rust
cargo build --workspace
cargo test --workspace
```

## জমা দেওয়ার আগে

- কোড ফরম্যাট করতে `cargo fmt --all` চালান
- lint করার জন্য `cargo clippy --workspace -- -D warnings` চালান
- সব টেস্ট পাস হচ্ছে কি না যাচাই করতে `cargo test --workspace` চালান
- ফাইল ৫০০ লাইনের নিচে রাখুন

## প্রজেক্ট কাঠামো

```
crates/
  bee_rust/         # মেটা crate, re-export + feature flags
  bee_router/       # রাউটিং + কন্ট্রোলার + Context + ফিল্টার চেইন
  bee_orm/          # ORM — Model trait + QuerySet + Migration
  bee_kv/           # KV/Cache একীভূত অ্যাবস্ট্রাকশন
  bee_search/       # সার্চ/অ্যানালিটিক্স ইঞ্জিন
  bee_graph/        # গ্রাফ ডেটাবেস
  bee_tsdb/         # টাইম-সিরিজ ডেটাবেস
  bee_config/       # কনফিগ ম্যানেজমেন্ট + হট-রিলোড
  bee_cache/        # ক্যাশ অ্যাবস্ট্রাকশন
  bee_session/      # সেশন ম্যানেজমেন্ট
  bee_logs/         # লগিং
  bee_template/     # টেমপ্লেট রেন্ডারিং
  bee_cli/          # CLI টুল
```

## লাইসেন্স

Apache-2.0
