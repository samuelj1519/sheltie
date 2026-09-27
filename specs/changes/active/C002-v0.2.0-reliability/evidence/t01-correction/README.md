# C002-T01 勘误候选门禁

Owner：Codex。独立 Reviewer：`/root/t01_standards`（工程规范）、`/root/t01_spec`（上游合同），结论见 [validation.md](../../validation.md)。平台：Darwin 27.0.0 arm64；`rustc 1.98.1`、`cargo 1.98.1`。

基线提交：`ceadc465fc2c57aaa52f910e78733da010f890b7`。运行时工作树只有 `CONTEXT.md` 与 `specs/` 文档改动，没有 `crates/`、`examples/`、`workbooks/`、Cargo 配置或 `Cargo.lock` 改动；`Cargo.lock` sha256 为 `a1e0f195f63fba6dfead940ae736e1698faa05438d6b373cbe8e480ad8ba8097`。Rust 门禁的编译与测试输入闭包未因后续验证记录文字变化而改变。Cargo 使用 `RUSTC_WRAPPER=` 与独立 `CARGO_TARGET_DIR=/private/tmp/sheltie-c002-t01-target`，未使用真实 `~/.sheltie`。

| 检查 | 完整命令 | 结果 | 原始输出 |
| --- | --- | --- | --- |
| 格式 | `RUSTC_WRAPPER= CARGO_TARGET_DIR=/private/tmp/sheltie-c002-t01-target cargo fmt --all -- --check` | exit 0 | [fmt.txt](fmt.txt) |
| 编译 | `RUSTC_WRAPPER= CARGO_TARGET_DIR=/private/tmp/sheltie-c002-t01-target cargo check --all-targets --all-features` | exit 0 | [check.txt](check.txt) |
| Clippy | `RUSTC_WRAPPER= CARGO_TARGET_DIR=/private/tmp/sheltie-c002-t01-target cargo clippy --all-targets --all-features -- -D warnings` | exit 0 | [clippy.txt](clippy.txt) |
| 测试 | `RUSTC_WRAPPER= CARGO_TARGET_DIR=/private/tmp/sheltie-c002-t01-target cargo nextest run --all-features --no-tests=pass` | exit 0；run ID `f89afde5-9961-4469-8efc-b11938563527`；315 passed，1 leaky，0 skipped | [nextest.txt](nextest.txt) |
| 文档 | `scripts/check-docs.sh` | exit 0 | [check-docs.txt](check-docs.txt) |
| 规格 | `scripts/check-specs.sh` | exit 0 | [check-specs.txt](check-specs.txt) |
| 任务范围 | `scripts/check-task.sh C002-T01 --staged` | exit 0 | [check-task.txt](check-task.txt) |
| 差异空白 | `git diff --check HEAD` | exit 0 | [diff-check.txt](diff-check.txt) |

`leaky` 是 `sheltie-cli::scenario_article_review next_after_review_offers_both_main_and_back_with_kinds` 的 nextest 标记；本次没有修改测试或产品代码，不能据此声称缺陷已修复。文档、规格与任务范围门禁在 T01 标 `done` 后的待提交树上执行；独立 Reviewer 结论见 [validation.md](../../validation.md)。提交后的 hash 与工作区核对须另行完成，不能由这些门禁代替。
