# 参与贡献

[English](CONTRIBUTING.md) · [한국어](CONTRIBUTING.ko.md) · [Русский](CONTRIBUTING.ru.md) · [Deutsch](CONTRIBUTING.de.md) · [Français](CONTRIBUTING.fr.md) · [Español](CONTRIBUTING.es.md) · [Português](CONTRIBUTING.pt.md) · [हिन्दी](CONTRIBUTING.hi.md) · [العربية](CONTRIBUTING.ar.md) · [বাংলা](CONTRIBUTING.bn.md) · [Bahasa Indonesia](CONTRIBUTING.id.md) · [日本語](CONTRIBUTING.ja.md)

## 环境搭建

```bash
git clone https://github.com/erikwang2013/bee-rust.git
cd bee-rust
cargo build --workspace
cargo test --workspace
```

## 提交之前

- 运行 `cargo fmt --all` 格式化代码
- 运行 `cargo clippy --workspace -- -D warnings` 检查 lint
- 运行 `cargo test --workspace` 确认所有测试通过
- 保持文件不超过 500 行

## 项目结构

```
crates/
  bee_rust/         # 元 crate，re-export + feature flags
  bee_router/       # 路由 + 控制器 + Context + 过滤器链
  bee_orm/          # ORM — Model trait + QuerySet + Migration
  bee_kv/           # KV/Cache 统一抽象
  bee_search/       # 搜索/分析引擎
  bee_graph/        # 图数据库
  bee_tsdb/         # 时序数据库
  bee_config/       # 配置管理 + 热更新
  bee_cache/        # 缓存抽象
  bee_session/      # Session 管理
  bee_logs/         # 日志
  bee_template/     # 模板渲染
  bee_cli/          # CLI 工具
```

## 许可证

Apache-2.0
