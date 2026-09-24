# AGENTS.md

给在本仓库工作的 agent 与开发者。Claude Code 通过 `CLAUDE.md` 引用本文；Codex、Cursor 直接读本文。改这里，不要另开一份。

## 项目一句话

Sheltie 是给协调者 agent 用的本地工作流引擎。人把做事方法写成 Workbook，引擎记状态、发任务书、限定合法下一步、守门槛，不判断内容好坏。Rust，三个 crate，单二进制 `sheltie`。

## 开工前读（按顺序，约一小时）

1. `CONTEXT.md`：词汇。同一个东西只有一个名字。
2. `specs/README.md`：文档地图与权威顺序。
3. `specs/plan.md`：找到第一个 `todo` 的任务。§0.2 是实现者的十条规则。
4. `specs/engineering.md`：怎么写代码、测试、提交。

## 当前阶段

代码是脚手架，规格已定稿。下一步是 `specs/plan.md` 的 T01（强模型搭骨架）。T02 起是填空任务。

## 硬边界

- 产品语义以 `specs/spec.md` 为准；不变式以 `specs/constitution.md` 为准；字段与命令以 `specs/contracts/` 为准。冲突时先改上游文档再改代码。
- 代码进度只看 `specs/plan.md` 状态列。文档描述目标，不得把计划中的能力写成已实现。
- `sheltie-core` 不做 I/O；`sheltie-runtime` 不做业务判断；引擎不读自然语言下结论（`INV-1`、`INV-2`）。
- 引擎只写 `~/.sheltie`，不写宿主配置、不安装任何东西到 agent（`INV-3`）。
- Package 只指 Cargo 包；业务方法只有 Workbook（`INV-4`）。
- 填空任务：不改签名、不改测试、不改快照、不加依赖、不新建文件。做不到就停下，把任务状态改 `blocked`，写清原因。不猜。

## 命令

```bash
cargo fmt --all -- --check
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run --all-features --no-tests=pass
cargo deny check
scripts/check-docs.sh
scripts/task.sh Tnn          # 只跑某任务的测试（T01 之后可用）
scripts/check-task.sh Tnn    # 提交前核对白名单与残留（T01 之后可用）
```

## 提交

一个任务一个提交，提交前上面六条全绿。格式见 `specs/engineering.md` §4：`type(scope): 中文摘要`，正文写为什么与怎么验证，末尾 `Task:`、`Agent:` 两行。

## 遇到缺口

产品问题改 `specs/spec.md` 并在 `specs/decisions.md` 追加一条；机制问题改对应合同；顺序问题改 `specs/plan.md`。不为了让当前任务编译通过发明第二套状态、兼容层或「先放这里」。
