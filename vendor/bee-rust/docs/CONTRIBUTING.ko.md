# 기여하기 (Contributing)

[简体中文](CONTRIBUTING.zh.md) · [English](CONTRIBUTING.md) · [한국어](CONTRIBUTING.ko.md) · [Русский](CONTRIBUTING.ru.md) · [Deutsch](CONTRIBUTING.de.md) · [Français](CONTRIBUTING.fr.md) · [Español](CONTRIBUTING.es.md) · [Português](CONTRIBUTING.pt.md) · [हिन्दी](CONTRIBUTING.hi.md) · [العربية](CONTRIBUTING.ar.md) · [বাংলা](CONTRIBUTING.bn.md) · [Bahasa Indonesia](CONTRIBUTING.id.md) · [日本語](CONTRIBUTING.ja.md)

## 설정

```bash
git clone https://github.com/erikwang2013/bee-rust.git
cd bee-rust
cargo build --workspace
cargo test --workspace
```

## 제출 전에

- `cargo fmt --all`을 실행하여 코드를 포맷합니다
- `cargo clippy --workspace -- -D warnings`을 실행하여 린트합니다
- `cargo test --workspace`을 실행하여 모든 테스트가 통과하는지 확인합니다
- 파일을 500줄 미만으로 유지합니다

## 프로젝트 구조

```
crates/
  bee_rust/         # 메타 crate, re-export + feature flags
  bee_router/       # 라우팅 + 컨트롤러 + Context + 필터 체인
  bee_orm/          # ORM — Model trait + QuerySet + Migration
  bee_kv/           # KV/Cache 통합 추상화
  bee_search/       # 검색/분석 엔진
  bee_graph/        # 그래프 데이터베이스
  bee_tsdb/         # 시계열 데이터베이스
  bee_config/       # 설정 관리 + 핫 리로드
  bee_cache/        # 캐시 추상화
  bee_session/      # Session 관리
  bee_logs/         # 로깅
  bee_template/     # 템플릿 렌더링
  bee_cli/          # CLI 도구
```

## 라이선스

Apache-2.0
