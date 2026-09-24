# Sheltie

本地运行的工作流引擎，给协调者 agent 用。人把做事方法写成 Workbook（TOML 图加自然语言说明），协调者 agent 按图派活，引擎记状态、发任务书、限定合法下一步、守门槛。引擎不判断内容好坏。

仓库当前是脚手架，尚未实现规格。目标与计划见 [specs/README.md](specs/README.md)；词汇见 [CONTEXT.md](CONTEXT.md)；agent 入口见 [AGENTS.md](AGENTS.md)。

## 快速开始（目标形态，随 MVP 落地生效）

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/<owner>/sheltie/releases/latest/download/sheltie-installer.sh | sh
export PATH="$HOME/.sheltie/bin:$PATH"

sheltie workbook add examples/two-step
sheltie work start --workbook two-step --flow default --input topic="给新人介绍 Sheltie"
sheltie work status 2026-09-24-001      # work_id 的唯一前缀即可
```

之后按状态卡里的「合法下一步」逐条执行，或在 Claude Code 里输入 `/sheltie` 让协调者代劳。升级用 `sheltie self update`，出问题 `sheltie self rollback`。

## 开发

```bash
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run --all-features --no-tests=pass
cargo deny check
scripts/check-docs.sh
```

工具链由 `rust-toolchain.toml` 固定为 stable（edition 2024，MSRV 1.85）。
