# Changelog

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

## [1.0.6] — 2026-08-07

### Adicionado
- Implementações reais do `bee_cli`: `new` (scaffolding de projetos), `generate controller/model`, `run` com hot reload via `--watch`, `pack` (build de release + cópia para `dist/`)
- Testes unitários da CLI para scaffolding e geração de código (7 novos testes)

### Corrigido
- `bee_rust::init()` agora fica atrás da feature `logs` — builds com features reduzidas (ex.: `--no-default-features --features kv`) voltam a compilar
- Lint `unnecessary_map_or` do Clippy em `bee_kv::InMemoryKvStore::exists`
- `rustfmt.toml` removeu opções exclusivas do nightly que eram silenciosamente ignoradas no stable; o workspace agora passa em `cargo fmt --all --check`
- `doc = false` no binário `bee_cli` para eliminar a colisão do nome do arquivo de saída do rustdoc com o `bee_rust`
- A porta do exemplo `hello` agora é configurável pela variável de ambiente `PORT`

### Alterado
- `bee-rust migrate` reporta "not implemented" e sai com código de erro diferente de zero (planejado)
- README / README.en atualizados para descrever o comportamento real da CLI

## [1.0.4] — 2026-07-29

### Adicionado
- Filtro de detecção de ataques de segurança via `security-rust` (27 detectores)
- `SecurityFilter` com cobertura para XSS, injeção de SQL, injeção de comandos e path traversal
- Feature flag `security` em `bee_rust` e `bee_router`

### Alterado
- README atualizado com a documentação do recurso de segurança
- README atualizado com a seção de apoio por pagamento (WeChat Pay / Alipay)

### Corrigido
- Sintaxe de identificador raw do Tera em `bee_template` para a edição 2024 do Rust

## [1.0.3] — 2026-07-29

### Adicionado
- Estrutura inicial do workspace com 13 crates
- Roteamento MVC com o trait `Controller` e o `Router`
- ORM com o builder `QuerySet` e a macro derivada `Model`
- Abstração de trait KV/Cache com os backends Redis e Memory
- Gerenciamento de sessão com os backends Memory/Redis
- Gerenciamento de configuração com suporte a INI/YAML/ENV e hot reload
- Renderização de templates via Tera
- Logs com integração com tracing
- Scaffolding de CLI e geração de código
- Stubs de traits para os mecanismos de busca, grafos e séries temporais (drivers planejados)

[1.0.4]: https://github.com/erikwang2013/bee-rust/compare/v1.0.3...v1.0.4
[1.0.3]: https://github.com/erikwang2013/bee-rust/releases/tag/v1.0.3
