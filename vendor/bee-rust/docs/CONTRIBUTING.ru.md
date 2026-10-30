# Вклад в проект

[简体中文](CONTRIBUTING.zh.md) · [English](CONTRIBUTING.md) · [한국어](CONTRIBUTING.ko.md) · [Русский](CONTRIBUTING.ru.md) · [Deutsch](CONTRIBUTING.de.md) · [Français](CONTRIBUTING.fr.md) · [Español](CONTRIBUTING.es.md) · [Português](CONTRIBUTING.pt.md) · [हिन्दी](CONTRIBUTING.hi.md) · [العربية](CONTRIBUTING.ar.md) · [বাংলা](CONTRIBUTING.bn.md) · [Bahasa Indonesia](CONTRIBUTING.id.md) · [日本語](CONTRIBUTING.ja.md)

## Настройка окружения

```bash
git clone https://github.com/erikwang2013/bee-rust.git
cd bee-rust
cargo build --workspace
cargo test --workspace
```

## Перед отправкой изменений

- Запустите `cargo fmt --all`, чтобы отформатировать код
- Запустите `cargo clippy --workspace -- -D warnings` для линтовки
- Запустите `cargo test --workspace`, чтобы убедиться, что все тесты проходят
- Держите файлы не длиннее 500 строк

## Структура проекта

```
crates/
  bee_rust/         # Мета-crate, re-export + feature-флаги
  bee_router/       # Маршрутизация + Контроллер + Context + цепочка фильтров
  bee_orm/          # ORM — trait Model + QuerySet + Migration
  bee_kv/           # Единая абстракция KV/кэша
  bee_search/       # Поисковый/аналитический движок
  bee_graph/        # Графовая база данных
  bee_tsdb/         # База данных временных рядов
  bee_config/       # Управление конфигурацией + горячая перезагрузка
  bee_cache/        # Абстракция кэша
  bee_session/      # Управление сессиями
  bee_logs/         # Логирование
  bee_template/     # Рендеринг шаблонов
  bee_cli/          # Инструменты CLI
```

## Лицензия

Apache-2.0
