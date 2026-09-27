# C002-T14 候选验证

Owner：Codex。独立 Reviewer：`/root/t14_standards`、`/root/t14_spec`；代码与合同复核均 PASS，具体边界见 [validation.md](../../validation.md)。基线：`2a37062dd885472edd36d114af87a0216b2c9773`。平台：Darwin 27.0.0 arm64、`rustc 1.98.1`、`cargo 1.98.1`。本任务改动 core/runtime/CLI 入口、测试及本 package 记录；`Cargo.toml`、`Cargo.lock`、examples、workbooks 未改，`Cargo.lock` sha256 为 `a1e0f195f63fba6dfead940ae736e1698faa05438d6b373cbe8e480ad8ba8097`。Cargo 使用 `RUSTC_WRAPPER=`、`CARGO_TARGET_DIR=/private/tmp/sheltie-c002-t14-target`，测试使用独立临时管理根。

| 检查 | 命令 | 结果 | 原始输出 |
| --- | --- | --- | --- |
| 格式 | `cargo fmt --all -- --check` | exit 0 | [fmt.txt](fmt.txt) |
| 编译 | `RUSTC_WRAPPER= CARGO_TARGET_DIR=/private/tmp/sheltie-c002-t14-target cargo check --all-targets --all-features` | exit 0 | [check.txt](check.txt) |
| Clippy | `RUSTC_WRAPPER= CARGO_TARGET_DIR=/private/tmp/sheltie-c002-t14-target cargo clippy --all-targets --all-features -- -D warnings` | exit 0 | [clippy.txt](clippy.txt) |
| 测试 | `RUSTC_WRAPPER= CARGO_TARGET_DIR=/private/tmp/sheltie-c002-t14-target cargo nextest run --all-features --no-tests=pass` | exit 0；run ID `88333664-74c1-43da-857a-16076175dba2`；325 passed、0 skipped | [nextest.txt](nextest.txt) |
| 公开 API 负例 | `RUSTC_WRAPPER= CARGO_TARGET_DIR=/private/tmp/sheltie-c002-t14-target cargo test -p sheltie-core --doc --all-features` | exit 0；5 个 `compile_fail` 通过 | [可读输出](doctest.txt)、[逐字节原始输出](doctest.txt.gz) |
| 文档 | `scripts/check-docs.sh` | exit 0 | [check-docs.txt](check-docs.txt) |
| 规格 | `scripts/check-specs.sh` | exit 0 | [check-specs.txt](check-specs.txt) |
| 测试归属 | `scripts/check-tests.sh` | exit 0 | [check-tests.txt](check-tests.txt) |
| 领域词汇 | `scripts/check-core-vocab.sh` | exit 0 | [check-core-vocab.txt](check-core-vocab.txt) |
| 任务范围 | `scripts/check-task.sh C002-T14 --staged` | exit 0 | [check-task.txt](check-task.txt) |
| 差异空白 | `git diff --check` | exit 0 | [diff-check.txt](diff-check.txt) |

合法例：当前样例、CLI show 与真实 start/submit/cancel 场景仍能编译并通过既有回归；`store_accepts_real_running_and_failed_attempt_states` 验证合法持久状态往返。反例：`load_rejects_row_identity_status_and_revision_mismatch` 每次只改一个行字段；`load_rejects_invalid_state_combination`、`load_rejects_running_attempt_with_end_time`、`load_rejects_running_attempt_on_different_current_occurrence` 各只改一处状态事实；`commit_rejects_revision_overflow_without_negative_row` 固定 SQLite 整数上限且证明无坏写；`remove_stops_on_corrupt_row_even_when_redundant_status_looks_terminal` 证明冗余 status 不能使坏引用行被跳过；`commit_rejects_mismatched_state_identity_without_writing` 核无业务行与请求记录。`host_require_snapshot_decode` 的合法/非法字段测试覆盖 Reply 快照仍需的反序列化。

5 个 `compile_fail` 中，3 个证明外部不能改 Manifest、FlowDef、NodeDef 字段，2 个证明 Manifest/FlowDef 不能绕过 parse 直接反序列化。doctest 文本为了通过仓库 EOF 门禁只删了最后一个空行，`.gz` 保存命令的逐字节原始输出。较早的 321 项开发运行曾标 1 项 `leaky`；因审查反例与 HostRequire 校验随后改变了代码和测试输入，才执行本次固定候选的 325 项运行，并非为消除标记重复运行同一候选。

T14 白名单增补 runtime service 与 CLI workbook show，因为两者直接读取被收紧的定义；若不迁移就无法编译。`check-task.sh` 仅自动允许 plan/tasks，未允许每任务必须维护的 progress/validation/evidence；本任务为 T02–T15 逐项补入这三条**本 package 内**的证据路径，没有扩展其产品代码路径。change 索引原有「当前任务」静态格会每次过期，本任务把它改为指向唯一状态权威 plan 的入口，后续任务不再重复改根索引。T14 只关闭定义接口与持久行验证，Workbook remove 的引用检查与删除同事务仍归 T08，schema 2 与恢复闭环仍归 T07/T08。最终待提交树的文档、规格与任务范围门禁通过；提交后 hash、trailer 和工作区仍须单独核对。
