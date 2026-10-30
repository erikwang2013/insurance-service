<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Beerust

[简体中文](../README.md) · [English](README.en.md) · [한국어](README.ko.md) · [Русский](README.ru.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Português](README.pt.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [বাংলা](README.bn.md) · [Bahasa Indonesia](README.id.md) · [日本語](README.ja.md)

O Beerust é um framework web de produção escrito em Rust, cuja filosofia de design é originária do framework Beego de Go, reexpressa com os traits, macros e o sistema de tipos idiomáticos de Rust.

## Mascote do projeto: Rusty

<img src="rusty.svg" width="360" alt="Rusty pairando de frente, asas abertas em círculo, patas dianteiras segurando a caixa de ferramentas cor de ferrugem laranja">

### Ficha do personagem

**Nome** Rusty (Ferrugento) — a cor da ferrugem de Rust, e também a ferrugem esfregada no seu macacão de trabalho.

**Aparência**: uma abelha engenheira peluda. A pelagem âmbar-dourada cobre o abdômen redondo e volumoso, com três listras pretas como carvão que parecem suspensórios de macacão tortos; quatro asas transparentes tão finas que deixam passar a luz, zumbindo em rastro fantasma ao pairar; os olhos compostos são duas contas de vidro de obsidiana, cada uma com um pontinho de brilho; as patas dianteiras seguram uma caixa de ferramentas cor de ferrugem laranja, bem polida pelo uso — essa é a sua cesta de pólen, que nunca carrega pólen, e sim dependências.

**Personalidade**: workaholic, otimista, levemente compulsiva.

- Ao consertar a colmeia, verifica célula por célula com as antenas, tão rigorosa quanto rodar um lint
- Trava firmemente as versões das dependências na cesta de pólen; quem mexer nelas, enfrenta a fúria dela
- Ao caçar bugs, fica imóvel pairando no ar, com as asas tão rápidas que resta apenas um círculo de rastro
- No dia do lançamento de uma nova versão, dança a dança do oito das abelhas na entrada da colmeia

### Especificações visuais

| Elemento | Especificação |
|------|------|
| Corpo | Âmbar-dourado `#F5B301`, pelo abdominal curto e denso, com um contorno de brilho na borda |
| Listras | Preto-carvão `#1F1A17`, três, apenas no abdômen |
| Asas | Azul-claro transparente `#CFE8F5`, 40% de opacidade, com nervuras visíveis |
| Olhos compostos | Marrom-escuro-preto `#2A1E16`, ovais, com um pontinho de brilho no canto superior esquerdo |
| Detalhes | Ferrugem-laranja `#B7410E` (caixa de ferramentas, gravata-borboleta); o tom ferrugem é a sua assinatura |

### Catálogo de expressões

| Expressão | Visual | Quando |
|------|------|---------|
| Cheia de energia | `(^o^)` pelagem eriçada, asas vibrando em alta frequência | quando todos os testes passam |
| Debug pairando | `o(°▽°)o` pairando no ar, restando apenas o rastro das asas | ao caçar um bug difícil |
| Retorno com carga total | `(≧▽≦)` caixa de ferramentas recheada, voando torta | quando um novo driver é integrado com sucesso |
| Cochilando | `(-_-)zZ` agachada na entrada da colmeia, antenas caídas | tarde da noite, sem tráfego |
| Irritada | `(#°益°)` ferrão eriçado | quando o filtro de segurança bloqueia um ataque |

### Movimentos de assinatura

1. **Debug pairando** — ao encontrar um bug, paira sobre o código, com as asas zumbindo em rastro, olhando fixamente até você corrigir.
2. **Dança do oito** — a cada lançamento de nova versão, dá uma volta completa ao redor da colmeia, anunciando para toda a colmeia: "as flores desabrocharam". As abelhas de verdade usam isso para transmitir a localização da fonte de flores; a Rusty usa isso para transmitir o changelog.
3. **Alisar a pelagem** — esfrega as antenas repetidamente com as patas dianteiras. Você pensa que é vaidade; na verdade ela está verificando se a configuração foi recarregada a quente.

### Onde ela aparece

- **Logo**: posição de pairar de frente, asas abertas em anel, circundando exatamente o nome.
- **Página 404**: pairando sobre um campo de flores vazio, girando confusa — "e o mel?"
- **CLI**: uma pequena abelha na linha do banner de inicialização, agachada ao lado supervisionando enquanto os testes rodam.
- **Anúncio de lançamento**: aquela em que dança o oito.

## Objetivos de design

| Objetivo | Métrica |
|------|------|
| **Experiência de desenvolvimento** | De `bee-rust new` à primeira requisição em < 30 s |
| **Desempenho** | Sobrecarga da camada de controladores < 5% (comparado ao axum puro), latência de roteamento P99 < 100 µs |
| **Velocidade de compilação** | Compilação completa do meta crate < 60 s (release), compilação incremental < 5 s |
| **Tamanho do binário** | Aplicação mínima (apenas router) < 5 MB (strip + LTO) |
| **Segurança** | 0 código de negócio com unsafe; todo FFI encapsulado em crates `*-sys` separados |
| **Compatibilidade** | MSRV Rust 1.80+, acompanhando o canal stable |

## Princípios de design

1. **Filosofia do Beego, expressão em Rust** — MVC, namespaces, cadeia de filtros, implementados com traits + macros
2. **Aprimoramento progressivo** — o núcleo mínimo depende apenas de axum + tokio; todo o resto fica atrás de feature gates
3. **Explícito melhor que implícito** — registro de rotas, mapeamento de modelos e ordem de middlewares são todos declarados explicitamente no código
4. **Abstrações de custo zero** — despacho estático via traits, expansão em tempo de compilação via macros, sem sobrecarga de funções virtuais
5. **Independência dos mecanismos de armazenamento** — cada trait de mecanismo pode ter sua implementação trocada isoladamente, sem afetar a lógica de negócio acima
6. **Observabilidade embutida** — instrumentação de tracing + metrics em todo o framework, logs estruturados ativados por padrão

## Arquitetura

### Topologia de crates

```
bee_rust/           # Meta crate, re-export + feature flags
bee_router/         # Roteamento + controladores + Context + cadeia de filtros
bee_orm/            # ORM — trait Model + QuerySet + Migration + mapeamento de relações
bee_kv/             # Abstração unificada KV/Cache — Redis + Memcached
bee_search/         # Mecanismo de busca/análise — Elasticsearch + OpenSearch + ClickHouse
bee_graph/          # Banco de dados de grafos — Neo4j + NebulaGraph + ArangoDB
bee_tsdb/           # Banco de dados de séries temporais — InfluxDB + Apache IoTDB + QuestDB
bee_config/         # Gerenciamento de configuração — INI/YAML/ENV + hot reload
bee_cache/          # Abstração de cache — Memory/Redis/Memcache
bee_session/        # Session — backends Memory/Redis/Cookie/Database
bee_logs/           # Logs — log em múltiplos níveis + integração com tracing
bee_template/       # Renderização de templates — baseado em tera
bee_cli/            # CLI — scaffolding/geração de código/execução de desenvolvimento/empacotamento (migração planejada)
```

### Diagrama de arquitetura

```
                          ┌───────────────────────┐
                          │  bee_rust  (meta)      │
                          │  re-export + features  │
                          └───────────┬───────────┘
                                      │
              ┌───────────────────────┼───────────────────────┐
              │                       │                       │
     ┌────────▼────────┐    ┌────────▼────────┐    ┌────────▼────────┐
     │    Web Layer     │    │   Data Layer    │    │   Tool Layer    │
     └────────┬────────┘    └────────┬────────┘    └────────┬────────┘
              │                       │                       │
  ┌───────────┼───────────┐  ┌───────┼───────┐  ┌───────────┼───────────┐
  │ bee_router            │  │ bee_orm        │  │ bee_cli               │
  │  - route register     │  │  - Model/Query  │  │  - scaffolding        │
  │  - controller trait   │  │  - Migration   │  │  - hot reload          │
  │  - filter chain       │  │  - Connection   │  │  - code generation    │
  │  - param extract      │  │                 │  │                       │
  ├────────────────────────┤  ├────────────────┤  ├───────────────────────┤
  │ bee_template           │  │ bee_config     │  │ bee_logs              │
  │  - template render     │  │  - INI/YAML/ENV│  │  - multi-level log    │
  │  - HTML/JSON           │  │  - hot reload   │  │  - tracing integrate  │
  ├────────────────────────┤  ├────────────────┤  └───────────────────────┘
  │ bee_session            │  │ bee_cache      │
  │  - session management  │  │  - cache trait  │
  │  - multi-backend       │  │  - Mem/Redis    │
  └────────────────────────┘  └────────────────┘

  ┌─────────────────────────────────────────────────────────┐
  │                   Storage Engine Layer                   │
  ├──────────────────┬──────────────────────────────────────┤
  │ bee_kv           │  Redis + Memcached                   │
  │ bee_search       │  Elasticsearch + OpenSearch + ClickHouse │
  │ bee_graph        │  Neo4j + NebulaGraph + ArangoDB      │
  │ bee_tsdb         │  InfluxDB + Apache IoTDB + QuestDB   │
  └──────────────────┴──────────────────────────────────────┘
```

### Dependências entre crates

```
bee_config (sem dependências)
bee_logs   (sem dependências)
bee_cache  → bee_config
bee_kv     → bee_config
bee_session → bee_cache, bee_config
bee_template (sem dependências)
bee_orm    → bee_config, bee_cache
bee_search → bee_config
bee_graph  → bee_config
bee_tsdb   → bee_config
bee_router → bee_session, bee_template, bee_config, bee_logs
bee_cli    → bee_router, bee_orm
bee_rust   → todos os crates acima (re-export)
```

### Bancos de dados suportados

| Categoria | Banco de dados | Crate correspondente | Feature Flag |
|------|--------|-----------|-------------|
| **Relacional** | SQLite | `bee_orm` | `sqlite` |
| | PostgreSQL | `bee_orm` | `postgres` |
| | MySQL | `bee_orm` | `mysql` |
| | TiDB | `bee_orm` | `mysql` |
| **KV / Cache** | Redis | `bee_kv` / `bee_cache` | `redis` |
| | Memcached | `bee_kv` / `bee_cache` | `memcache` |
| **Busca / Análise** | Elasticsearch | `bee_search` | `elasticsearch` |
| | OpenSearch | `bee_search` | `opensearch` |
| | ClickHouse | `bee_search` | `clickhouse` |
| **Banco de grafos** | Neo4j | `bee_graph` | `neo4j` |
| | NebulaGraph | `bee_graph` | `nebulagraph` |
| | ArangoDB | `bee_graph` | `arangodb` |
| **Séries temporais** | InfluxDB | `bee_tsdb` | `influxdb` |
| | Apache IoTDB | `bee_tsdb` | `iotdb` |
| | QuestDB | `bee_tsdb` | `questdb` |

### Cadeia de filtros de requisição

```
Requisição → [SecurityFilter detecção de ataques] → [Restauração de Session] → [Validação de parâmetros] → [gancho prepare] → [tratamento handle] → [gancho finish] → Resposta
                  ↓ Qualquer etapa pode ser interrompida (como o Abort do Beego)
```

## Recursos

A visão geral dos recursos está abaixo; para o uso detalhado de cada módulo, exemplos de código e descrições da API, consulte a [Referência da API](api.pt.md).

### Núcleo Web (bee_router)

Controladores MVC: o trait `Controller` + `Context`, com suporte a namespaces de rotas, registro de métodos RESTful e cadeia de filtros de requisição.

- Saída de resposta: `ctx.json()` / `ctx.text()` / `ctx.html()`
- Redirecionamento / interrupção: `ctx.redirect()` / `ctx.abort()`
- Sessão e parâmetros: `ctx.session` / `ctx.params`

### Detecção de segurança (feature `security`)

Filtro de detecção de ataques baseado em [security-rust](https://crates.io/crates/security-rust), cobrindo 27 tipos de ataque, incluindo XSS, injeção de SQL, injeção de comandos e SSRF, ativado com uma linha:

```rust
let security = SecurityFilter::new();  // todos os 27 detectores ativados
```

### ORM (bee_orm)

Macro derivada `#[derive(Model)]` + consulta encadeada com QuerySet (filter / order_by / limit), com suporte a SQLite, PostgreSQL, MySQL e TiDB.

### Gerenciamento de configuração (bee_config)

Macro derivada `#[derive(Config)]`, com suporte a carregamento de INI / YAML / ENV e hot reload.

### Mecanismos de armazenamento

KV / Cache (Redis + Memcached), mecanismo de busca (Elasticsearch / OpenSearch / ClickHouse), banco de dados de grafos (Neo4j / NebulaGraph / ArangoDB), banco de dados de séries temporais (InfluxDB / IoTDB / QuestDB) — todos com abstração unificada via trait, drivers compilados sob feature gate.

### Session, logs e templates

- Session: múltiplos backends — Memory / Redis / Cookie / Database
- Logs: log em múltiplos níveis + integração com tracing
- Templates: renderização baseada em tera

### Ferramenta CLI

```bash
bee-rust new my-app            # cria o scaffolding do projeto
bee-rust generate controller user
bee-rust run --watch           # execução de desenvolvimento (hot reload)
bee-rust pack                  # empacotamento para implantação
```

## Como usar

### Requisitos de ambiente

- Rust 1.80+
- Cargo

### Instalação

```bash
# Clonar o projeto
git clone https://github.com/erikwang2013/bee-rust.git
cd bee-rust

# Compilar
cargo build --workspace

# Executar os testes
cargo test --workspace
```

### Início rápido

```bash
# Criar um novo projeto com a CLI
cargo run -p bee_cli -- new hello
cd hello

# Executar o servidor de desenvolvimento
cargo run
```

### Usando no seu projeto

```toml
[dependencies]
bee_rust = { git = "https://github.com/erikwang2013/bee-rust", features = ["full"] }
```

## Notas técnicas

### Stack de tecnologia

| Camada | Tecnologia |
|----|------|
| Base HTTP | axum 0.8 + tower 0.5 |
| Runtime assíncrono | tokio 1.x |
| Serialização | serde + serde_json |
| Mecanismo de templates | tera 1.x |
| Camada de logs | tracing + tracing-subscriber |
| CLI | clap 4 |
| Parsing de configuração | toml / serde_yaml / INI próprio |
| Tratamento de erros | thiserror |
| Macros processuais | syn + quote + proc-macro2 |

### Padrões de design

| Padrão | Aplicação |
|------|------|
| **Builder** | Logger, Router, QuerySet |
| **Abstração por trait** | Cache, KvStore, SearchEngine, GraphDB, TimeSeriesDB |
| **Macros derivadas** | `#[derive(Model)]`, `#[derive(Config)]` |
| **Feature Gate** | implementações de drivers compiladas sob demanda (redis, memcached, elasticsearch, etc.) |
| **Filter Chain** | cadeia de filtros de requisição, equivalente ao Filter do Beego |

### Lista de crates

| Crate | Função | Equivalente no Beego |
|-------|------|-----------|
| `bee_rust` | Meta crate, ponto de entrada unificado | — |
| `bee_router` | Roteamento + controladores + Context + filtros | `server/web`, `context` |
| `bee_orm` | ORM + QuerySet + Migration | `client/orm` |
| `bee_kv` | Abstração unificada KV/Cache | `client/cache` (estendido) |
| `bee_search` | Mecanismo de busca/análise | — (novo) |
| `bee_graph` | Banco de dados de grafos | — (novo) |
| `bee_tsdb` | Banco de dados de séries temporais | — (novo) |
| `bee_config` | Gerenciamento de configuração + hot reload | `client/config` |
| `bee_cache` | Abstração de cache | `client/cache` |
| `bee_session` | Gerenciamento de Session | `server/web/session` |
| `bee_logs` | Logs | `logs` |
| `bee_template` | Renderização de templates | — (aprimorado) |
| `bee_cli` | Ferramenta CLI | ferramenta `bee` |

### Cobertura de testes

Os 68 testes de todo o repositório passam:

| Crate | Número de testes |
|-------|--------|
| bee_config | 4 |
| bee_cache | 4 |
| bee_template | 2 |
| bee_logs | 3 |
| bee_kv | 4 |
| bee_search | 6 |
| bee_graph | 5 |
| bee_tsdb | 5 |
| bee_orm | 7 |
| bee_session | 2 |
| bee_router | 9 |
| bee_cli | 16 |

## Apoie o projeto

Se este projeto foi útil para você, fique à vontade para escanear o QR code e fazer uma doação de apoio. Obrigado!

**WeChat Pay**

<img src="weixinpay.png" width="160" height="175" alt="WeChat Pay">

**Alipay**

<img src="alipay.png" width="160" height="175" alt="Alipay">

**Transferência bancária internacional (Bank Transfer)**

Usuários no exterior podem apoiar o projeto por meio de transferência bancária:

**Informações do beneficiário**

| Item | Conteúdo |
|------|------|
| Nome do beneficiário | WANG KEXUN |
| Número da conta do beneficiário | 881015918251 |

**Banco beneficiário**

| Item | Conteúdo |
|------|------|
| Código SWIFT | AABLHKHHXXX |
| Nome do banco | ZA Bank Limited |
| Número do banco | 387 |
| Endereço do banco | Core F, Cyberport 3, 100 Cyberport Road, Hong Kong |

**Banco correspondente para remessas internacionais (se necessário)**

> Observação: estas são informações do banco correspondente (intermediário) para remessas internacionais, e não do banco beneficiário. Consulte o seu banco remetente para saber se é necessário fornecê-las.

- **Para depósitos em HKD, CNY e USD** (banco correspondente: Citibank):

| Item | Conteúdo |
|------|------|
| Nome do banco | Citibank N.A. Hong Kong |
| Código SWIFT | CITIHKHXXXX |
| Número do banco | 006 |
| Nome da agência | Hong Kong Branch |
| Número da agência | 391 |
| Endereço do banco | Citibank Tower, Citibank Plaza, 3 Garden Road, Central, Hong Kong |

- **Para outras moedas** (banco correspondente: BNY Mellon):

| Item | Conteúdo |
|------|------|
| Nome do banco | THE BANK OF NEW YORK MELLON |
| Código SWIFT | IRVTUS3NXXX |
| Endereço do banco | THE BANK OF NEW YORK MELLON, 240 GREENWICH STREET, NEW YORK, United States |

### Licença

Apache-2.0
