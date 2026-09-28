# C002-T22 同句柄输出观察与封存证据

基线：`543f9d2`（C002-T21）。平台：macOS arm64，Rust `1.98.1`，target `aarch64-apple-darwin`；MSRV `1.85.0` 检查通过。`Cargo.lock` SHA-256：`d9e4b72b4810337fa0c260223f2104907347a05da51f3ab146445ce04d295e4f`。所有管理根与文件均为任务临时目录。

Linux 原生运行由用户明确豁免，记 `not_run`。本证据不表示 Linux 或跨平台 PASS。

## 观察与封存链

`observe_output` 现在返回送 core 的 `ObservedFile` 和本调用持有的 `SafeFile`。`PreparedCommand` 把这组句柄留在提交调用栈中。`run_command` 只有在 `Store::commit` 成功后，才把同一组句柄传给 `execute_with_observed_outputs`；revision 冲突会丢弃本轮句柄并重新 load/observe。句柄和 fd 不序列化，也不进入 core、WorkState 或 effects JSON。

正常 submit 每条 `SealOutputs` 引用都必须找到匹配绝对路径的观察句柄，缺句柄会在任何重开前拒绝。执行器在同一句柄上重新核普通单链接文件类型、设备号/inode、字节数和 SHA-256，再对这个对象 `fchmod(0444)` 与 `fsync`。权限动作后，受管路径从 Home 锚定句柄重开核对仍绑定原观察对象。路径已换时，原观察对象完成封存，错误停止效果并保留 `published=0`；新目标不受影响。内容、链接数或类型不符时，先停止再 chmod。

恢复没有进程内句柄，因此从受检 `ArtifactRef` 逐段 no-follow 重开受管路径，重新核类型、nlink、bytes、SHA 后才封存。已完成 submit 重放使用 `publish=false`，不重复封存。`Error::Io` 在封存调用中保留结构化类型；路径丢失、对象身份/类型改变与摘要变化归完整性错误。最终 CLI `original/revision/cause` 封装仍由 T25 负责。

## V07/V08 反例与 oracle

- 三个真实 `WorkService::submit` 测试通过 failpoint feature rendezvous 停在 Store COMMIT 已成功、封存尚未开始的确定窗口；rendezvous 按 request-id 精确匹配，仅影响指定测试提交。父线程每次只改变一个条件，发出 release 后让真实 seal caller 继续。
- **只换路径**：把原 output inode rename 到保留路径，在原路径放外部哨兵软链。请求以 `EffectPending(committed=true, request_id=<本请求>)` 停止；SQL 核对 `works.revision = begin_revision + 1`、本请求 `published=0`、WorkId 正确。原已观察对象成为 `0444`，外部哨兵 bytes/mode 不变，原输出路径仍是链接。
- **只改同 inode bytes**：提交前固定 mode `0644`；COMMIT 后同路径原位写入等长 `tampered bytes!`。请求失败、revision 已提交、`published=0`；文件完整 permissions mode 与窗口开始相同，字节仍为被改内容。
- **只增加硬链接**：COMMIT 后为 output 建硬链接。请求失败、revision 已提交、`published=0`；原文件与别名仍保持提交前权限，未改对象权限。
- 三个窗口都从 SQLite 独立读取 `state_json` 和 `effects_json`，逐项确认原 `ArtifactRef` 与 `SealOutputs.refs` 的路径一致，摘要为手工向量 `bd9d61af8c2082b7b8073f3fe347543aefba3dbddb0d6de46638f85ae7040717`，长度 15；不以当前文件计算期望。
- **恢复窗口**分别在一次成功 submit 后制造 bytes、nlink 或叶类型变化，再置该请求 `published=0` 触发真实 `recover`；恢复拒绝、不修改新对象，`published` 留在 0。另一个重放用例确认已完成 submit 重放不重封也不重写后来内容。
- 正常 CLI submit 正例通过，产物字节不变且权限为 `0444`；原有缺失/超限/软链输出拒绝场景继续通过。

## 门禁

stdout/stderr 与退出码按 gate 分开保存。Rust 使用 `RUSTC_WRAPPER=`、`CARGO_TARGET_DIR=/private/tmp/sheltie-c002-t21-target`，避免写不可用的全局 target。

| Gate | 命令 | 结果 |
| --- | --- | --- |
| fmt | `cargo fmt --all -- --check` | PASS |
| check | `cargo check --offline --all-targets --all-features` | PASS |
| Clippy | `cargo clippy --offline --all-targets --all-features -- -D warnings` | PASS |
| 全仓 Nextest | `cargo nextest run --offline --workspace --all-features --no-tests=pass --no-fail-fast` | PASS；run `025d54a2-a67d-417e-aa7a-7b6002be84ee`；516 passed、0 skipped、3 leaky |
| T22 定向 | `scripts/task.sh C002-T22` | PASS；run `f3c1a700-8279-466a-b79f-6ab10a560d91`；14 passed、502 skipped |
| 暂存区白名单 | `scripts/check-task.sh C002-T22 --staged` | PASS；`check-task: C002-T22 OK` |
| 影响面 | runtime `fs_boundary/schema2_replay/service` 与 CLI `scenario_artifacts` | PASS；68 runtime passed、5 CLI passed，均 0 skipped |
| MSRV | `cargo +1.85.0 check --offline --workspace --all-targets --all-features --locked` | PASS |
| deny | `cargo deny --offline check` | PASS；本机缓存 advisory DB；既有 allowance/winnow 警告详见原始 stderr |
| 文档与治理 | docs/specs/core-vocab/tests/skill/typos 与 `git diff --check` | PASS |

全仓 Nextest 的 3 个 leaky 用例为既有 `release_governance::build_workflow_fetches_history_and_runs_msrv_gate`、`output_paths::add_rejects_ancestor_and_folded_alias_output_paths_at_cli`、`output_paths::worker_outputs_named_brief_and_stats_write_and_submit_via_real_chain`；均通过，没有失败。`cargo deny` 原始表格 stderr 以 `deny.stderr.raw.b64` 无损保存，可用 `base64 -D < deny.stderr.raw.b64` 解码。

## 独立 Review

- Spec `/root/spec_review`：PASS。核了真实 submit 窗口、两份原引用、Store revision/request/published 与外部哨兵 oracle。
- Rust Standards `/root/standards_review`：PASS。核了 fd 所有权、错误分类、哈希/类型/nlink 检查顺序、ReadOnly chmod/sync 同对象、恢复/重放分支与权限 oracle。
- 两位 Reviewer 未运行或修改代码。结论只覆盖 T22；T23–T31 与最终 M1 未关闭。Linux `not_run`。
