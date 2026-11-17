# temporal

`temporal` 是独立 Rust crate，提供绝对时刻、时间角色、精度转换，以及基于显式证据的历史可见性与事实版本选择。包版本从 `0.1.0` 开始。

行为合同见 [goal.md](goal.md) 与 [spec.md](spec.md)。本仓不依赖 `kernel`、数据源、存储驱动或运行时。库只校验证据的结构、身份关联、上下文绑定与确定性规则；证据字符串所指向内容的真实性由调用方受信适配器验收（信任边界见 [docs/adr/ADR-0001-evidence-trust-boundary.md](docs/adr/ADR-0001-evidence-trust-boundary.md)），调用方同时负责授权、持久化和业务映射。依赖本仓时使用固定 Git commit，并在消费者清单中同时声明 `path` 与 `version`（本地组合）。本仓设置 `publish = false`，不执行 `cargo publish`。

本地门禁：

```bash
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo doc --locked --no-deps
cargo package --locked
```

`SourcePublishedAsOf` 与 `SystemAsKnown` 使用不同证据和快照，不可互换。没有完整候选覆盖或可信证据时，查询会拒绝给出确定的历史结论。
