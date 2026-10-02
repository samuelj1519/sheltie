# C001 验证

Candidate: `SELF`（承载本 package 完成状态的提交）

| Requirement / risk | Mode | Input closure | Command / raw run ID | Result | Evidence |
| --- | --- | --- | --- | --- | --- |
| change、ADR、release 与 agent 入口闭合 | executed | 当前工作树 | `scripts/check-specs.sh` | PASS | 2 个 change，0 个 active |
| Markdown 链接与禁用措辞 | executed | 当前工作树 | `scripts/check-docs.sh` | PASS | 76 个 Markdown 文件 |
| 测试归属与 task 卡一致 | executed | 当前工作树 | `scripts/check-tests.sh` | PASS | 315 个测试，228 条任务卡 |
| skill 命令与协议一致 | executed | 当前工作树 | `scripts/check-skill.sh` | PASS | `check-skill: OK` |
| core 依赖与业务词汇边界 | executed | 当前工作树 | `scripts/check-core-vocab.sh` | PASS | `check-core-vocab: OK` |
| Bash 3.2 语法与测试归属脚本 | executed | 当前工作树 | `/bin/bash -n ...`、`/bin/bash scripts/check-tests.sh` | PASS | macOS Bash 3.2.57 |
| Rust 格式 | executed | 当前工作树 | `cargo fmt --all -- --check` | PASS | 无输出 |
| Rust 编译 | executed | 当前工作树 | `cargo check --all-targets --all-features` | PASS | dev profile 完成 |
| Rust Clippy | executed | 当前工作树 | `cargo clippy --all-targets --all-features -- -D warnings` | PASS | 零 warning |
| Rust 测试 | executed | 当前工作树 | `cargo nextest run --all-features --no-tests=pass` | PASS | run `a91b20be-445e-404d-8640-dcd4562208b1`，315 passed，0 skipped |
| 依赖许可、漏洞、来源 | executed | 当前工作树 | `cargo deny check` | PASS | advisories、bans、licenses、sources ok；既有 allow/duplicate warning 保留 |
| Diff 与拼写 | executed | 当前工作树 | `git diff --check`、`typos` | PASS | 无输出 |
| 独立文档 review | executed | 当前工作树 | `review.md` | PASS | Standards 与 Spec 均无 finding |

`SELF` 表示候选就是包含本文件的最终治理提交，避免在提交内容中保存无法自洽的自引用 commit hash。

完整 `pre-commit run --all-files` 中，文档、格式、typos、治理、skill、词汇和测试归属 hook 均通过；Cargo hooks 因沙箱不能写全局 `target`、`sccache` 与 advisory lock 失败。表中对应 Cargo 命令已使用隔离 `CARGO_TARGET_DIR`、禁用 `sccache` 和获批的 advisory lock 独立执行并通过；提交时使用相同环境重跑 hooks。
