# Contribuindo

[简体中文](CONTRIBUTING.zh.md) · [English](CONTRIBUTING.md) · [한국어](CONTRIBUTING.ko.md) · [Русский](CONTRIBUTING.ru.md) · [Deutsch](CONTRIBUTING.de.md) · [Français](CONTRIBUTING.fr.md) · [Español](CONTRIBUTING.es.md) · [Português](CONTRIBUTING.pt.md) · [हिन्दी](CONTRIBUTING.hi.md) · [العربية](CONTRIBUTING.ar.md) · [বাংলা](CONTRIBUTING.bn.md) · [Bahasa Indonesia](CONTRIBUTING.id.md) · [日本語](CONTRIBUTING.ja.md)

## Configuração

```bash
git clone https://github.com/erikwang2013/bee-rust.git
cd bee-rust
cargo build --workspace
cargo test --workspace
```

## Antes de Enviar

- Execute `cargo fmt --all` para formatar o código
- Execute `cargo clippy --workspace -- -D warnings` para o lint
- Execute `cargo test --workspace` para verificar se todos os testes passam
- Mantenha os arquivos abaixo de 500 linhas

## Estrutura do Projeto

```
crates/
  bee_rust/         # Meta crate, re-export + feature flags
  bee_router/       # Roteamento + Controlador + Context + cadeia de filtros
  bee_orm/          # ORM — trait Model + QuerySet + Migration
  bee_kv/           # Abstração unificada KV/Cache
  bee_search/       # Mecanismo de busca/análise
  bee_graph/        # Banco de dados de grafos
  bee_tsdb/         # Banco de dados de séries temporais
  bee_config/       # Gerenciamento de configuração + hot reload
  bee_cache/        # Abstração de cache
  bee_session/      # Gerenciamento de sessão
  bee_logs/         # Logs
  bee_template/     # Renderização de templates
  bee_cli/          # Ferramentas de CLI
```

## Licença

Apache-2.0
