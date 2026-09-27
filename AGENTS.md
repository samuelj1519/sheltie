# AGENTS.md

给在本仓库工作的 agent 与开发者。Claude Code 通过 `CLAUDE.md` 引用本文；Codex、Cursor 直接读本文。

## 项目一句话

Sheltie 是给协调者 agent 用的本地工作流引擎。人把做事方法写成 Workbook，引擎记状态、发任务书、限定合法下一步、守门槛，不判断内容好坏。Rust，三个 crate，单二进制 `sheltie`。

## 开工入口

1. 读 `CONTEXT.md`，使用项目词汇。
2. 读 `specs/README.md`，确认当前 release、active change 与文档权威。
3. 只有存在 active change 时，读 `specs/changes/README.md` 指向的 package；按 package plan 找任务和验证入口。
4. 写代码、测试或提交前，读 `specs/engineering.md`。

没有 active change 时，不从 `proposed/` 自行选择方案实施。提案采用、任务开始和对外发布都由人决定。

## 硬边界

- 产品语义以 `specs/spec.md` 为准；不变式以 `specs/constitution.md` 为准；字段与命令以 `specs/contracts/` 为准。冲突时先改上游文档再改代码。
- 当前实施进度只看 active change 的 `plan.md`。proposal、review、提交存在或测试通过都不等于产品能力完成。
- `sheltie-core` 不做 I/O；`sheltie-runtime` 不做业务判断；引擎不读自然语言下结论（`INV-1`、`INV-2`）。
- 引擎只写 `~/.sheltie`，不写宿主配置、不安装任何东西到 agent（`INV-3`）。
- Package 只指 Cargo 包；业务方法只有 Workbook（`INV-4`）。
- 一个 change package 只在自己的目录内记录 finding、设计、计划、进度、验证和审查；不要把执行流水追加到根规格或决定记录。

## 命令

```bash
cargo fmt --all -- --check
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run --all-features --no-tests=pass
cargo deny check
scripts/check-docs.sh
scripts/check-specs.sh
scripts/task.sh <task>          # package plan 要求时运行
scripts/check-task.sh <task>    # package plan 要求时运行
```

## 提交

一个任务一个提交，提交前运行 package plan 要求的门禁。格式见 `specs/engineering.md` §4：`type(scope): 中文摘要`，正文写为什么与怎么验证。新 change 使用 `Change:`、`Task:`、`Agent:` trailer；MVP 的 T01–T26 保留原格式。

## 遇到缺口

产品问题改 `specs/spec.md`；机制问题改架构或合同；跨任务的重要选择新增 `specs/decisions/D-*.md`；任务顺序改 active package plan。问题尚未采用时留在 `changes/proposed/`，不得写成当前行为。不要为了让任务编译通过发明第二套状态、兼容层或临时事实来源。
