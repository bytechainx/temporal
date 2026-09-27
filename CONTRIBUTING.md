# 贡献约定

本仓只实现 [goal.md](goal.md) 与 [spec.md](spec.md) 归属 `temporal` 的合同。改变可观察语义时先修订规格，再修改代码、合同验证和 [CHANGELOG.md](CHANGELOG.md)。消费者适配、真实来源证据、存储及 `kernel` 迁移由各自仓库验收。

从仓库根目录运行：

```bash
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
RUSTDOCFLAGS=-Dwarnings cargo doc --locked --no-deps
cargo package --locked
```

只提交本仓任务文件。禁止生产依赖 `kernel`、工作区元仓库、存储驱动、Tokio、serde 或时区库。不得将文档结构检查、单仓测试或合成样例解释为真实消费方 PIT 验收。
