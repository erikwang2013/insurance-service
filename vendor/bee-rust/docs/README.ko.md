<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Beerust

[简体中文](../README.md) · [English](README.en.md) · [한국어](README.ko.md) · [Русский](README.ru.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Português](README.pt.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [বাংলা](README.bn.md) · [Bahasa Indonesia](README.id.md) · [日本語](README.ja.md)

Beerust는 Go의 Beego 프레임워크에서 디자인 철학을 얻은, Rust 언어로 작성된 프로덕션급 웹 프레임워크입니다. Rust에 익숙한 trait, macro, 타입 시스템으로 재표현되었습니다.

## 프로젝트 마스코트: Rusty

<img src="rusty.svg" width="360" alt="앞모습으로 공중에 떠 있는 Rusty, 날개를 고리 모양으로 펼치고 앞발로 녹슨 주황색 공구 상자를 안고 있음">

### 캐릭터 카드

**이름** Rusty (러스티) — Rust의 녹슨 색, 그리고 작업복 바지에 묻은 그 철녹.

**모습**: 털북숭이 공학자 벌. 호박금빛 털이 동그란 배를 덮고 있고, 세 줄의 숯검정 줄무늬가 삐뚤게 맨 작업복 멜빵처럼 보입니다. 네 장의 투명한 날개는 빛이 비칠 만큼 얇아서, 공중에 떠 있을 때 윙윙거리는 잔상이 남습니다. 겹눈은 유리 구슬 같은 흑요석 두 알로 각각 하이라이트가 하나씩 박혀 있습니다. 앞발로는 닳아서 윤이 나는 녹슨 주황색 공구 상자를 안고 있는데, 그것은 꽃가루 바구니로, 꽃가루 대신 항상 의존성(dependency)을 담고 있습니다.

**성격**: 일벌레, 낙천주의자, 가벼운 강박증.

- 벌집을 고칠 때 더듬이로 한 칸씩 검사하며, lint를 돌리듯 꼼꼼하게 따진다
- 꽃가루 바구니의 의존성 버전을 철통같이 잠가 놓아서, 누가 건드리면 화를 낸다
- bug를 쫓을 때는 공중에 멈춰 서서, 날개가 너무 빨라 동그란 잔상만 남는다
- 새 버전을 배포하는 날에는 벌집 입구에서 벌의 8자 흔들기 춤을 춘다

### 시각 규격

| 요소 | 규격 |
|------|------|
| 본체 | 호박금 `#F5B301`, 배 털은 짧고 빽빽하며 가장자리에 하이라이트 한 겹 |
| 줄무늬 | 숯검정 `#1F1A17`, 세 줄, 배에만 |
| 날개 | 투명한 연한 파랑 `#CFE8F5`, 40% 불투명도, 맥(脈)이 그려져 있음 |
| 겹눈 | 짙은 갈색 검정 `#2A1E16`, 타원형, 왼쪽 위에 하이라이트 한 알 |
| 포인트 | 녹슨 주황 `#B7410E` (공구 상자, 나비넥타이), 녹슨 색이 시그니처 |

### 표정 모음

| 표정 | 그림 | 언제 |
|------|------|---------|
| 활력 충만 | `(^o^)` 털이 부풀고, 날개가 고주파로 진동 | 테스트가 전부 통과했을 때 |
| 공중 정지 디버깅 | `o(°▽°)o` 공중에 떠서 날개 잔상만 남김 | 까다로운 bug를 쫓을 때 |
| 가득 싣고 귀환 | `(≧▽≦)` 공구 상자가 통통하고, 비틀거리며 날아감 | 새 드라이버 연동 성공 |
| 졸고 있음 | `(-_-)zZ` 벌집 입구에 웅크리고, 더듬이가 축 처짐 | 한밤중에 트래픽이 없을 때 |
| 털이 곤두섬 | `(#°益°)` 꼬리 침이 곤두섬 | 보안 필터가 공격을 차단했을 때 |

### 시그니처 동작

1. **공중 정지 디버깅** —— bug를 발견하면 코드 위에 떠서, 날개가 윙윙거리며 잔상이 되도록 허공을 맴돌며, 고쳐질 때까지 지켜본다.
2. **8자 흔들기 춤** —— 새 버전을 배포할 때마다 벌집 주위를 한 바퀴 돌며 "꽃이 피었다"고 온 집안에 알린다. 진짜 벌들은 이것으로 꽃밭의 위치를 전달하고, Rusty는 이것으로 changelog를 전달한다.
3. **털 다듬기** —— 앞발로 더듬이를 반복해서 닦는다. 단정히 하려는 줄 알겠지만, 사실은 설정이 핫 리로드되었는지 확인하는 것이다.

### 어디에 등장하나

- **로고**: 정면으로 떠 있는 자세, 날개를 고리 모양으로 펼쳐 이름을 딱 둘러싼다.
- **404 페이지**: 텅 빈 꽃밭 위를 떠서 멍하니 빙글빙글 돈다 — "꿀은 어디 있지?"
- **CLI**: 시작 배너의 한 줄짜리 작은 벌, 테스트를 실행하는 동안 옆에 앉아 감독한다.
- **배포 공지**: 8자 춤을 추는 그 장면.

## 설계 목표

| 목표 | 지표 |
|------|------|
| **개발 경험** | `bee-rust new`부터 첫 요청까지 < 30초 |
| **성능** | 컨트롤러 계층 오버헤드 < 5% (순수 axum 대비), P99 라우팅 지연 < 100µs |
| **컴파일 속도** | 메타 crate 전체 컴파일 < 60s (release), 증분 컴파일 < 5s |
| **바이너리 크기** | 최소 애플리케이션 (router만) < 5MB (strip + LTO) |
| **안전성** | unsafe 비즈니스 코드 0개; 모든 FFI는 독립된 `*-sys` crate에 캡슐화 |
| **호환성** | Rust 1.80+ MSRV, stable 추적 |

## 설계 원칙

1. **Beego 철학, Rust로 표현** — MVC, 네임스페이스, 필터 체인을 trait + macro로 구현
2. **점진적 강화** — 최소 코어는 axum + tokio에만 의존하고, 나머지는 전부 feature gate
3. **암시보다 명시** — 라우팅 등록, 모델 매핑, 미들웨어 순서를 모두 코드에 명시적으로 선언
4. **제로 코스트 추상화** — trait 정적 디스패치, macro 컴파일 타임 확장, 가상 함수 오버헤드 없음
5. **스토리지 엔진 독립성** — 각 엔진의 trait 구현을 개별적으로 교체해도 상위 비즈니스에 영향 없음
6. **관측성 내장** — tracing + metrics 계측이 프레임워크 전반을 커버, 구조화 로그 기본 활성화

## 아키텍처 설계

### Crate 토폴로지

```
bee_rust/           # 메타 crate, re-export + feature flags
bee_router/         # 라우팅 + 컨트롤러 + Context + 필터 체인
bee_orm/            # ORM — Model trait + QuerySet + Migration + 관계 매핑
bee_kv/             # KV/Cache 통합 추상화 — Redis + Memcached
bee_search/         # 검색/분석 엔진 — Elasticsearch + OpenSearch + ClickHouse
bee_graph/          # 그래프 데이터베이스 — Neo4j + NebulaGraph + ArangoDB
bee_tsdb/           # 시계열 데이터베이스 — InfluxDB + Apache IoTDB + QuestDB
bee_config/         # 설정 관리 — INI/YAML/ENV + 핫 리로드
bee_cache/          # 캐시 추상화 — Memory/Redis/Memcache
bee_session/        # Session — Memory/Redis/Cookie/Database 백엔드
bee_logs/           # 로그 — 다중 레벨 로그 + tracing 통합
bee_template/       # 템플릿 렌더링 — tera 기반
bee_cli/            # CLI — 스캐폴딩/코드 생성/개발 실행/패키징 (마이그레이션 계획 중)
```

### 아키텍처 다이어그램

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

### Crate 의존 관계

```
bee_config (의존 없음)
bee_logs   (의존 없음)
bee_cache  → bee_config
bee_kv     → bee_config
bee_session → bee_cache, bee_config
bee_template (의존 없음)
bee_orm    → bee_config, bee_cache
bee_search → bee_config
bee_graph  → bee_config
bee_tsdb   → bee_config
bee_router → bee_session, bee_template, bee_config, bee_logs
bee_cli    → bee_router, bee_orm
bee_rust   → 위의 모든 crate (re-export)
```

### 지원되는 데이터베이스

| 카테고리 | 데이터베이스 | 해당 Crate | Feature Flag |
|------|--------|-----------|-------------|
| **관계형** | SQLite | `bee_orm` | `sqlite` |
| | PostgreSQL | `bee_orm` | `postgres` |
| | MySQL | `bee_orm` | `mysql` |
| | TiDB | `bee_orm` | `mysql` |
| **KV / 캐시** | Redis | `bee_kv` / `bee_cache` | `redis` |
| | Memcached | `bee_kv` / `bee_cache` | `memcache` |
| **검색 / 분석** | Elasticsearch | `bee_search` | `elasticsearch` |
| | OpenSearch | `bee_search` | `opensearch` |
| | ClickHouse | `bee_search` | `clickhouse` |
| **그래프 데이터베이스** | Neo4j | `bee_graph` | `neo4j` |
| | NebulaGraph | `bee_graph` | `nebulagraph` |
| | ArangoDB | `bee_graph` | `arangodb` |
| **시계열 데이터베이스** | InfluxDB | `bee_tsdb` | `influxdb` |
| | Apache IoTDB | `bee_tsdb` | `iotdb` |
| | QuestDB | `bee_tsdb` | `questdb` |

### 요청 필터 체인

```
요청 → [SecurityFilter 공격 감지] → [Session 복원] → [파라미터 검증] → [prepare 훅] → [handle 처리] → [finish 훅] → 응답
                  ↓ 어느 단계에서든 중단 가능 (Beego의 Abort와 유사)
```

## 기능 소개

기능 개요는 아래와 같으며, 각 모듈의 자세한 사용법, 코드 예제와 API 설명은 [API 참조](api.ko.md)에서 확인하세요.

### Web 핵심 (bee_router)

MVC 컨트롤러: `Controller` trait + `Context`, 라우팅 네임스페이스, RESTful 메서드 등록, 요청 필터 체인 지원.

- 응답 출력: `ctx.json()` / `ctx.text()` / `ctx.html()`
- 리다이렉트 / 중단: `ctx.redirect()` / `ctx.abort()`
- 세션과 파라미터: `ctx.session` / `ctx.params`

### 보안 감지 (`security` feature)

[security-rust](https://crates.io/crates/security-rust) 기반의 공격 감지 필터로, XSS, SQL 인젝션, 명령어 인젝션, SSRF 등 27가지 공격 유형을 다루며 한 줄로 켤 수 있습니다:

```rust
let security = SecurityFilter::new();  // 27개 탐지기 모두 켜짐
```

### ORM (bee_orm)

`#[derive(Model)]` 파생 매크로 + QuerySet 체이닝 쿼리 (filter / order_by / limit), SQLite, PostgreSQL, MySQL, TiDB 지원.

### 설정 관리 (bee_config)

`#[derive(Config)]` 파생 매크로, INI / YAML / ENV 로드와 핫 리로드 지원.

### 스토리지 엔진

KV / Cache (Redis + Memcached), 검색 엔진 (Elasticsearch / OpenSearch / ClickHouse), 그래프 데이터베이스 (Neo4j / NebulaGraph / ArangoDB), 시계열 데이터베이스 (InfluxDB / IoTDB / QuestDB)를 통일된 trait으로 추상화하며, 드라이버는 feature gate에 따라 컴파일됩니다.

### Session, 로그, 템플릿

- Session: Memory / Redis / Cookie / Database 다중 백엔드
- 로그: 다중 레벨 로그 + tracing 통합
- 템플릿: tera 기반 렌더링

### CLI 도구

```bash
bee-rust new my-app            # 프로젝트 스캐폴딩 생성
bee-rust generate controller user
bee-rust run --watch           # 개발 실행 (핫 리로드)
bee-rust pack                  # 패키징 배포
```

## 사용 방법

### 환경 요구 사항

- Rust 1.80+
- Cargo

### 설치

```bash
# 프로젝트 클론
git clone https://github.com/erikwang2013/bee-rust.git
cd bee-rust

# 컴파일
cargo build --workspace

# 테스트 실행
cargo test --workspace
```

### 빠른 시작

```bash
# CLI로 새 프로젝트 생성
cargo run -p bee_cli -- new hello
cd hello

# 개발 서버 실행
cargo run
```

### 프로젝트에서 사용

```toml
[dependencies]
bee_rust = { git = "https://github.com/erikwang2013/bee-rust", features = ["full"] }
```

## 기술 설명

### 기술 스택

| 계층 | 기술 |
|----|------|
| HTTP 기반 | axum 0.8 + tower 0.5 |
| 비동기 런타임 | tokio 1.x |
| 직렬화 | serde + serde_json |
| 템플릿 엔진 | tera 1.x |
| 로그 기반 | tracing + tracing-subscriber |
| CLI | clap 4 |
| 설정 파싱 | toml / serde_yaml / 자체 개발 INI |
| 오류 처리 | thiserror |
| 프로시저 매크로 | syn + quote + proc-macro2 |

### 디자인 패턴

| 패턴 | 적용 |
|------|------|
| **Builder** | Logger, Router, QuerySet |
| **Trait 추상화** | Cache, KvStore, SearchEngine, GraphDB, TimeSeriesDB |
| **파생 매크로** | `#[derive(Model)]`, `#[derive(Config)]` |
| **Feature Gate** | 드라이버 구현을 필요에 따라 컴파일 (redis, memcached, elasticsearch 등) |
| **Filter Chain** | 요청 필터 체인, Beego Filter에 대응 |

### Crate 목록

| Crate | 기능 | Beego 대응 |
|-------|------|-----------|
| `bee_rust` | 메타 crate, 통합 진입점 | — |
| `bee_router` | 라우팅 + 컨트롤러 + Context + 필터 | `server/web`, `context` |
| `bee_orm` | ORM + QuerySet + Migration | `client/orm` |
| `bee_kv` | KV/Cache 통합 추상화 | `client/cache` (확장) |
| `bee_search` | 검색/분석 엔진 | — (신규) |
| `bee_graph` | 그래프 데이터베이스 | — (신규) |
| `bee_tsdb` | 시계열 데이터베이스 | — (신규) |
| `bee_config` | 설정 관리 + 핫 리로드 | `client/config` |
| `bee_cache` | 캐시 추상화 | `client/cache` |
| `bee_session` | Session 관리 | `server/web/session` |
| `bee_logs` | 로그 | `logs` |
| `bee_template` | 템플릿 렌더링 | — (강화) |
| `bee_cli` | CLI 도구 | `bee` 도구 |

### 테스트 커버리지

전체 저장소의 68개 테스트 통과:

| Crate | 테스트 수 |
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

## 지원 환영

이 프로젝트가 도움이 되셨다면, QR 코드를 스캔하여 후원해 주시면 감사하겠습니다!

**위챗페이**

<img src="weixinpay.png" width="160" height="175" alt="위챗페이">

**알리페이**

<img src="alipay.png" width="160" height="175" alt="알리페이">

**글로벌 송금 (Bank Transfer)**

해외 사용자는 은행 송금으로 지원할 수 있습니다:

**수취인 정보**

| 항목 | 내용 |
|------|------|
| 수취인 이름 | WANG KEXUN |
| 수취인 계좌 번호 | 881015918251 |

**수취 은행**

| 항목 | 내용 |
|------|------|
| SWIFT Code | AABLHKHHXXX |
| 은행 이름 | ZA Bank Limited |
| 은행 번호 | 387 |
| 은행 주소 | Core F, Cyberport 3, 100 Cyberport Road, Hong Kong |

**해외 송금 중개 은행 (필요 시)**

> 참고: 이는 해외 송금 중개 은행(중간 은행) 정보로, 수취 은행 정보가 아닙니다. 송금 은행에 제공이 필요한지 문의하세요.

- **홍콩 달러, 위안화, 미국 달러 송금** (중개 은행은 Citibank):

| 항목 | 내용 |
|------|------|
| 은행 이름 | Citibank N.A. Hong Kong |
| SWIFT Code | CITIHKHXXXX |
| 은행 번호 | 006 |
| 지점 이름 | Hong Kong Branch |
| 지점 번호 | 391 |
| 은행 주소 | Citibank Tower, Citibank Plaza, 3 Garden Road, Central, Hong Kong |

- **기타 통화 송금** (중개 은행은 BNY Mellon):

| 항목 | 내용 |
|------|------|
| 은행 이름 | THE BANK OF NEW YORK MELLON |
| SWIFT Code | IRVTUS3NXXX |
| 은행 주소 | THE BANK OF NEW YORK MELLON, 240 GREENWICH STREET, NEW YORK, United States |

### 라이선스

Apache-2.0
