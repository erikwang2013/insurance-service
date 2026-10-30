# Log Perubahan

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

## [1.0.6] — 2026-08-07

### Ditambahkan
- Implementasi nyata `bee_cli`: `new` (scaffolding proyek), `generate controller/model`, `run` dengan hot reload `--watch`, `pack` (build release + salin ke `dist/`)
- Pengujian unit CLI untuk scaffolding dan generasi kode (7 pengujian baru)

### Diperbaiki
- `bee_rust::init()` kini dikurung di balik fitur `logs` — build dengan fitur tereduksi (mis. `--no-default-features --features kv`) dapat dikompilasi lagi
- Lint Clippy `unnecessary_map_or` di `bee_kv::InMemoryKvStore::exists`
- `rustfmt.toml` menghapus opsi khusus nightly yang sebelumnya diabaikan diam-diam di stable; workspace kini lolos `cargo fmt --all --check`
- Biner `bee_cli` diatur `doc = false` untuk menghilangkan bentrok nama berkas output rustdoc dengan `bee_rust`
- Port contoh `hello` kini dapat dikonfigurasi melalui variabel env `PORT`

### Diubah
- `bee-rust migrate` melaporkan "not implemented" dan keluar dengan kode non-nol (direncanakan)
- README / README.en diperbarui untuk menjelaskan perilaku CLI yang sebenarnya

## [1.0.4] — 2026-07-29

### Ditambahkan
- Filter deteksi serangan keamanan melalui `security-rust` (27 detektor)
- `SecurityFilter` dengan cakupan XSS, injeksi SQL, injeksi perintah, path traversal
- Flag fitur `security` di `bee_rust` dan `bee_router`

### Diubah
- README diperbarui dengan dokumentasi fitur keamanan
- README diperbarui dengan bagian dukungan donasi (WeChat Pay / Alipay)

### Diperbaiki
- Sintaks raw identifier Tera di `bee_template` untuk edisi Rust 2024

## [1.0.3] — 2026-07-29

### Ditambahkan
- Struktur workspace awal dengan 13 crate
- Routing MVC dengan trait `Controller` dan `Router`
- ORM dengan builder `QuerySet` dan macro turunan `Model`
- Abstraksi trait KV/Cache dengan backend Redis dan Memory
- Manajemen sesi dengan backend Memory/Redis
- Manajemen konfigurasi dengan dukungan INI/YAML/ENV dan hot-reload
- Rendering template melalui Tera
- Logging dengan integrasi tracing
- Scaffolding CLI dan generasi kode
- Trait stub untuk mesin pencarian, graf, dan deret waktu (driver direncanakan)

[1.0.4]: https://github.com/erikwang2013/bee-rust/compare/v1.0.3...v1.0.4
[1.0.3]: https://github.com/erikwang2013/bee-rust/releases/tag/v1.0.3
