# Contribución

[简体中文](CONTRIBUTING.zh.md) · [English](CONTRIBUTING.md) · [한국어](CONTRIBUTING.ko.md) · [Русский](CONTRIBUTING.ru.md) · [Deutsch](CONTRIBUTING.de.md) · [Français](CONTRIBUTING.fr.md) · [Español](CONTRIBUTING.es.md) · [Português](CONTRIBUTING.pt.md) · [हिन्दी](CONTRIBUTING.hi.md) · [العربية](CONTRIBUTING.ar.md) · [বাংলা](CONTRIBUTING.bn.md) · [Bahasa Indonesia](CONTRIBUTING.id.md) · [日本語](CONTRIBUTING.ja.md)

## Configuración

```bash
git clone https://github.com/erikwang2013/bee-rust.git
cd bee-rust
cargo build --workspace
cargo test --workspace
```

## Antes de enviar

- Ejecuta `cargo fmt --all` para formatear el código
- Ejecuta `cargo clippy --workspace -- -D warnings` para el linting
- Ejecuta `cargo test --workspace` para verificar que todos los tests pasan
- Mantén los archivos por debajo de 500 líneas

## Estructura del proyecto

```
crates/
  bee_rust/         # Meta-crate, re-export + feature flags
  bee_router/       # Enrutado + controlador + Context + cadena de filtros
  bee_orm/          # ORM — trait Model + QuerySet + Migration
  bee_kv/           # Abstracción unificada KV/Cache
  bee_search/       # Motor de búsqueda/análisis
  bee_graph/        # Base de datos de grafos
  bee_tsdb/         # Base de datos de series temporales
  bee_config/       # Gestión de configuración + recarga en caliente
  bee_cache/        # Abstracción de caché
  bee_session/      # Gestión de sesiones
  bee_logs/         # Registros
  bee_template/     # Renderizado de plantillas
  bee_cli/          # Herramientas CLI
```

## Licencia

Apache-2.0
