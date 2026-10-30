# 변경 로그 (Changelog)

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

## [1.0.6] — 2026-08-07

### 추가됨
- `bee_cli` 실제 구현: `new` (프로젝트 스캐폴딩), `generate controller/model`, `--watch` 핫 리로드가 포함된 `run`, `pack` (release 빌드 + `dist/`로 복사)
- 스캐폴딩 및 코드 생성을 위한 CLI 단위 테스트 (신규 테스트 7개)

### 수정됨
- `bee_rust::init()`이 이제 `logs` feature 뒤로 게이트됨 — 축소된 feature 빌드 (예: `--no-default-features --features kv`)가 다시 컴파일됨
- `bee_kv::InMemoryKvStore::exists`의 Clippy `unnecessary_map_or` 린트
- `rustfmt.toml`에서 stable에서 조용히 무시되던 nightly 전용 옵션 제거; 이제 워크스페이스가 `cargo fmt --all --check`를 통과함
- `bee_cli` 바이너리에 `doc = false`를 적용하여 `bee_rust`와의 rustdoc 출력 파일명 충돌 제거
- `hello` 예제의 포트를 이제 `PORT` 환경 변수로 설정 가능

### 변경됨
- `bee-rust migrate`가 "not implemented"를 보고하고 0이 아닌 코드로 종료함 (계획됨)
- 실제 CLI 동작을 설명하도록 README / README.en 업데이트

## [1.0.4] — 2026-07-29

### 추가됨
- `security-rust`를 통한 보안 공격 감지 필터 (탐지기 27개)
- XSS, SQL 인젝션, 명령어 인젝션, 경로 탐색을 다루는 `SecurityFilter`
- `bee_rust`와 `bee_router`의 `security` feature 플래그

### 변경됨
- 보안 feature 문서를 포함하도록 README 업데이트
- 결제 지원 섹션 (WeChat Pay / Alipay)을 포함하도록 README 업데이트

### 수정됨
- Rust 2024 에디션을 위한 `bee_template` Tera 원시 식별자(raw identifier) 구문

## [1.0.3] — 2026-07-29

### 추가됨
- 13개 crate로 구성된 초기 워크스페이스 구조
- `Controller` trait과 `Router`를 사용한 MVC 라우팅
- `QuerySet` 빌더와 `Model` 파생 매크로를 사용한 ORM
- Redis 및 Memory 백엔드를 지원하는 KV/Cache trait 추상화
- Memory/Redis 백엔드를 지원하는 Session 관리
- INI/YAML/ENV 지원 및 핫 리로드가 포함된 설정 관리
- Tera를 통한 템플릿 렌더링
- tracing 통합 로깅
- CLI 스캐폴딩 및 코드 생성
- 검색, 그래프, 시계열 엔진 trait 스텁 (드라이버는 계획 중)

[1.0.4]: https://github.com/erikwang2013/bee-rust/compare/v1.0.3...v1.0.4
[1.0.3]: https://github.com/erikwang2013/bee-rust/releases/tag/v1.0.3
