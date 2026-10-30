# Berkontribusi

[简体中文](CONTRIBUTING.zh.md) · [English](CONTRIBUTING.md) · [한국어](CONTRIBUTING.ko.md) · [Русский](CONTRIBUTING.ru.md) · [Deutsch](CONTRIBUTING.de.md) · [Français](CONTRIBUTING.fr.md) · [Español](CONTRIBUTING.es.md) · [Português](CONTRIBUTING.pt.md) · [हिन्दी](CONTRIBUTING.hi.md) · [العربية](CONTRIBUTING.ar.md) · [বাংলা](CONTRIBUTING.bn.md) · [Bahasa Indonesia](CONTRIBUTING.id.md) · [日本語](CONTRIBUTING.ja.md)

## Persiapan

```bash
git clone https://github.com/erikwang2013/bee-rust.git
cd bee-rust
cargo build --workspace
cargo test --workspace
```

## Sebelum Mengirim

- Jalankan `cargo fmt --all` untuk memformat kode
- Jalankan `cargo clippy --workspace -- -D warnings` untuk lint
- Jalankan `cargo test --workspace` untuk memastikan semua pengujian lolos
- Jaga setiap berkas di bawah 500 baris

## Struktur Proyek

```
crates/
  bee_rust/         # Meta crate, re-export + flag fitur
  bee_router/       # Routing + Controller + Context + rantai filter
  bee_orm/          # ORM — trait Model + QuerySet + Migration
  bee_kv/           # Abstraksi terpadu KV/Cache
  bee_search/       # Mesin pencarian/analitik
  bee_graph/        # Basis data graf
  bee_tsdb/         # Basis data deret waktu
  bee_config/       # Manajemen konfigurasi + hot-reload
  bee_cache/        # Abstraksi cache
  bee_session/      # Manajemen sesi
  bee_logs/         # Logging
  bee_template/     # Rendering template
  bee_cli/          # Alat CLI
```

## Lisensi

Apache-2.0
