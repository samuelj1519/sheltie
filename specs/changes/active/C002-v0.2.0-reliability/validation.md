# C002 审查证据与修复验证状态

审查候选：`a664e75a3ab3041d09cd0a1ab4d69336f2dcd055`。开始时工作区干净。日期：2026-09-27。§1–§3 是审查与方案整合的证据（已完成）；§4 起是实施验证记录，未执行的任务保持 `not_run`，不填 PASS。

## 1. 本次代码审查验证

平台：macOS aarch64；`rustc 1.98.1`、`cargo 1.98.1`。Cargo 构建使用 `RUSTC_WRAPPER=`、独立 `CARGO_TARGET_DIR=/tmp/sheltie-review-a664e75/target`，避免写用户全局 target。默认构建曾被 sccache 沙箱权限拒绝，关闭 wrapper 后成功；该环境问题不计入产品 finding。

| Requirement / risk | Mode | Input closure | Command / raw run ID | Result | Evidence |
| --- | --- | --- | --- | --- | --- |
| 当前候选构建 | executed | HEAD + Cargo.lock + all features | `cargo build --all-features --message-format=json` | PASS | Cargo 报告的 executable 用于下列 probes |
| 格式 | executed | 同候选 | `cargo fmt --all -- --check` | PASS | [fmt.txt](evidence/2026-09-27/fmt.txt) |
| 编译 | executed | 全 targets/features | `cargo check --all-targets --all-features` | PASS | [check.txt](evidence/2026-09-27/check.txt) |
| Clippy | executed | 全 targets/features | `cargo clippy --all-targets --all-features -- -D warnings` | PASS | [clippy.txt](evidence/2026-09-27/clippy.txt) |
| 既有测试 | executed | 22 binaries、315 tests | nextest `aac13440-9f58-48d0-a91d-926dbc02c559` | PASS：315 passed，0 skipped | [nextest.txt](evidence/2026-09-27/nextest.txt)；测试执行 11.849 秒，不含编译 |
| 依赖检查 | executed | 原 deny 规则，仅 advisory 缓存路径改到临时副本 | `cargo deny --offline --locked --config /tmp/sheltie-review-a664e75/deny.toml check` | PASS，含告警 | [deny-isolated.txt](evidence/2026-09-27/deny-isolated.txt) |
| 当前文档与任务归属 | executed | 改文档前 HEAD | docs/specs/tests/core-vocab/skill scripts | PASS | [证据目录](evidence/2026-09-27/README.md) |
| 跨目标与重放 | executed | 真实 CLI、独立临时 home | request_probe | 缺陷确认：O02/O04/O05/N01 | [raw JSON](evidence/2026-09-27/request-probes.json) |
| 文件、Workbook、self | executed | 真实 CLI、假 HOME、临时哨兵 | boundary_probe | 缺陷确认：O01/O03/O06/O07/O08/N02/N05 | [raw JSON](evidence/2026-09-27/standards-probes.json) |
| 摘要边界碰撞 | executed | 两棵合法 Workbook | hash_probe | 缺陷确认：O07 | [raw JSON](evidence/2026-09-27/hash-framing.json) |
| 输出与视图 | executed | 真实 CLI、自造合法 Flow | output/status/stats probes | 缺陷确认：O12/O13/N07 | [输出](evidence/2026-09-27/output-probes.json)、[状态](evidence/2026-09-27/status-probes.json)、[统计](evidence/2026-09-27/stats-probes.json) |
| Workbook 与 Git 闭环 | executed | 合成输出走图；独立 Git 提交 | workbook/git probes | 缺陷确认：O09/O10/N08 | [图](evidence/2026-09-27/workbook-probes.json)、[Git](evidence/2026-09-27/git-probes.json) |
| CI 入口 | executed | 隔离浅克隆 HEAD | `scripts/check-specs.sh` | FAIL：历史 tag/commit 缺失 | [原始输出](evidence/2026-09-27/shallow-specs-check.txt) |
| 可移植复现脚本 | executed | 当前候选 binary + 8 个入库 probe scripts | 每个脚本独立临时目录 | PASS：脚本均 exit 0，代表能复现，不代表产品通过 | [结果](evidence/2026-09-27/portable-probe-results.json) |

依赖检查首次因默认 advisory db.lock 位于只读路径而失败，见 [deny-offline.txt](evidence/2026-09-27/deny-offline.txt)。随后只复制已有 advisory 数据到临时目录，未改规则；缓存 commit 为 `e2111519ba6d14a5da59a7b2e5c8083ae8a37c01`，时间 `2026-09-25T19:51:57+02:00`。本次未联网刷新，不能宣称使用了当日最新公告。告警是重复 winnow 与未命中的许可 allowance，没有被隐藏或改为通过规则。

## 2. 静态确认与未执行边界

静态确认不伪装为动态故障实验：Workbook staging 并发删除、引用检查竞争、首次建库半结构、观察前全文读、self 固定版本选择缺口、release 质量依赖及已校验定义 public 构造面，均有对应源码定位，但本次没有对每项做 kill/并发/OOM/联网实验。

`cargo +1.85.0 check` 为 not_run，本机无 1.85 工具链；没有由此推断依赖必然不兼容。四平台发布、真正 Host、人审、usage、全量 mutants 和断电持久性验证均 not_run。当前用户 `~/.sheltie` 未检查或改动。旧 T26 的 prompt 冲突和用户目录叙述只保留来源边界。

## 3. 方案整合复核

主审完成整合后，两位独立审查者再次只读复核。首轮提出的修改已落实：

| 来源 | 问题 | 处理 |
| --- | --- | --- |
| Standards | N02 动态/静态边界表述不清 | 明确只动态复现发布失败残留；并发/kill 仍静态 |
| Standards、Spec | 新 outputs/brief.md/out 不应继续禁止 | 改为隔离后的合法例 |
| Standards | 写锁可能破坏失败 preflight/旧库拒绝的无写保证 | 只读预检 → 合法写建根/锁 → 锁内重验 |
| Standards | 提交后效果失败不能要求 Store 不变 | 明确 committed 与恢复响应 |
| Standards | pending rename 与只读查询竞争 | 同 effect 归属有限重读，不误报损坏、不任意 fallback |
| Standards | purge 与锁 inode 生命周期 | self 使用同一锁；等待者复核根/锁身份 |
| Spec | T06 新持久事实早于 T07 切换 | T06 仅准备纯单元，CLI 闭环移至 T07 |
| Spec | 已完成 submit 重放与未完成 seal 恢复冲突 | 完成封存不重做；未完成恢复错误携带原 snapshot 与 committed |

当时的最终复核：`spec_review` 与 `standards_review` 均确认“方案材料通过”。该结论只评价采用前的审查材料和当时的候选设计；不覆盖后来 T01 首次提交与本次勘误，也不是 C002-M1 产品修复 PASS。

整合后 `scripts/check-docs.sh`、`scripts/check-specs.sh` 与 `git diff --check` 均通过；改动范围仅本 package。生产代码未变化，未因文档修改重复全量 Rust 测试。最终脚本输出见 evidence 中 docs-final/specs-final/diff-final。

## 4. 后续实施证据要求

采用后每任务记录：候选 hash、输入闭包、执行命令及原始输出、正反例结果、实际 caller、独立 Reviewer、剩余问题。复用旧验证只在输入闭包相同时成立；缺原始运行标识或候选不同就重跑。M1 关闭 O01–O13/N01–N14；T16/T17 单独报告 Host、usage、产物与发布。未执行不填 PASS，合成输出不填真人批准。

重跑方法及原始证据映射见 [evidence/2026-09-27/README.md](evidence/2026-09-27/README.md)。只读审查结论不能作为采用、开始任务、迁移旧数据或对外发布的授权。

## 5. 实施验证记录（T01 起）

每个候选一行：提交或待提交候选、四条 Rust 门禁的原始运行位置、任务附加验证、独立 review、覆盖的 finding。失败或未执行如实记录，修复候选另起一行，不覆盖旧结果。具体正反例、CLI 命令、临时管理根、输入闭包和停止条件按 [plan.md](plan.md) 记录；下表只给总览。

| 任务 | 候选 | 四条 Rust 门禁 | 附加验证 | 独立 review / findings |
| --- | --- | --- | --- | --- |
| C002-T01 首次提交 | `ceadc465fc2c57aaa52f910e78733da010f890b7` | `not_run`；提交说明沿用旧基线，不能证明本候选 | 该提交原样复跑 `scripts/check-specs.sh` FAIL：`check-specs: 缺 specs/changes/proposed`；`scripts/check-task.sh C002-T01` PASS | 需修改：重放顺序、目录发布、删除引用、效果失败、安全 API 与交接状态；T01 尚未关闭 |
| C002-T01 勘误 | Owner Codex 的本次纠正提交；基线 `ceadc465` | fmt/check/Clippy/nextest exit 0；nextest 315 passed（1 leaky）；[原始运行](evidence/t01-correction/README.md) | 最终待提交树 docs/specs/check-task 见同目录门禁记录 | `/root/t01_standards` 独立 Standards PASS、`/root/t01_spec` 独立 Spec PASS；仅证明 T01 文档合同一致，后续实现仍 not_run |
| C002-T14 | not_run | not_run | not_run | not_run |
| C002-T02 | not_run | not_run | not_run | not_run |
| C002-T03 | not_run | not_run | not_run | not_run |
| C002-T04 | not_run | not_run | not_run | not_run |
| C002-T09 | not_run | not_run | not_run | not_run |
| C002-T05 | not_run | not_run | not_run | not_run |
| C002-T06 | not_run | not_run | not_run | not_run |
| C002-T07 | not_run | not_run | not_run | not_run |
| C002-T08 | not_run | not_run | not_run | not_run |
| C002-T10 | not_run | not_run | not_run | not_run |
| C002-T11 | not_run | not_run | not_run | not_run |
| C002-T12 | not_run | not_run | not_run | not_run |
| C002-T13 | not_run | not_run | not_run | not_run |
| C002-T15 | not_run | not_run | not_run | not_run |
| C002-M1 | not_run | not_run | not_run | not_run |
| C002-T16 | not_run | not_run | not_run | not_run |
| C002-T17 | not_run | not_run | not_run | not_run |

T01 勘误的 Rust 门禁与工作树静态检查原始输出见 [evidence/t01-correction/](evidence/t01-correction/README.md)。Rust 源码、fixtures 与 Cargo 输入闭包未改变，后续验证记录文字改动不使这四条运行失效。独立审查首轮发现并关闭：侧车持久顺序、历史 `write_file` 恢复、A 阻断 B 的 `committed = false`、删除结果不明、T09 对 T04 的依赖，以及未发布 Workbook 的只读入口。最终规格复核 PASS 仅覆盖本次文档一致性；Standards 复核 PASS 仅覆盖范围与证据。原始审查问题不从历史记录删除，T02–M1 的代码行为仍 `not_run`。

每个实施候选在本表下另起一个小节，至少填写以下字段；未执行项写 `not_run`，失败项保留原始输出和后续修复候选，不覆盖旧记录：

| 字段 | 填写内容 |
| --- | --- |
| 候选与输入闭包 | task、Owner、Reviewer、基准/候选 commit 或待提交树、`Cargo.lock`、特性、平台、工作区原状、临时管理根 |
| 改动与依据 | 文件清单、对应 finding、根规格/合同条款、影响到的实际 caller |
| 正反例 | 命令、唯一改变的条件、独立期望、实际响应/Store/文件字节/退出码、原始输出路径 |
| 门禁 | 四条 Rust 命令及附加 gate 的完整命令、运行标识、退出码、原始输出路径；复用时证明输入闭包相同 |
| 审查与交接 | Reviewer 的独立结论、逐条问题处置、剩余 `not_run`、最终提交 hash 与下一任务入口 |
