# C005 实施计划

状态：`active`。用户已明确采用，按本计划实施；环境无法执行的义务记录后延期。共同规则见 [方案实施指南](../../../guides/proposal-implementation.md)，工程门禁见 [工程规范](../../../engineering.md)。本方案跨状态、协议与持久化，采用两阶段交付；产品范围只由 spec/design 定义。

阶段交接时，T01 作者在本 plan 的对应实现任务卡写 `**测试。**` 与已存在的真实测试名，手册链接同一清单；任务标题使用 `### Cnnn-Tnn` 供 check-tests 识别。尚未建立的用例只写“验收用例”，不提前声称测试已经存在。

## 1. 首次阅读与职责

先读 `AGENTS.md`、`CONTEXT.md`、`specs/README.md`、本 package 的 README/spec/design/validation，再读 `specs/engineering.md` 与当前任务点名源码。普通接续已经可用，采用证据须说明为什么需要撤销旧提交资格。

| 类型 / 入口 | 先理解什么 |
| --- | --- |
| `AttemptId`、`AttemptStatus`、`WorkState::validate_persisted` | 顺序号、真实失败和 superseded 是三个不同事实 |
| `decide`、`legal_next` | core 只从输入产生决定；替换及次序不由 CLI 猜 |
| `reply_status_matches`、`validate_command_owner` | 历史 fail 用截至该 Attempt 的失败前缀，而非当前总数 |
| `WorkService::run_command`、`RequestIntent`、`snapshot::check_data` | 查重、观察、事务、历史快照各自的输入和归属 |
| `effects.rs`、`recovery.rs`、`WorkLayout` | COMMIT 后按登记字节发布，不能从最新状态重建旧任务书 |
| `AttemptCmd`、`commands/attempt.rs::run` | CLI 仅解析和接线，不改变合同、恢复或资格政策 |

复杂模型作者负责总体接口、真实 caller 可行性、可信原语与阶段测试；简单模型或初级开发者负责固定边界内的纯状态变换和接线。真实使用任务按冻结手册执行，产品判断交复杂模型。两次里程碑由未参与其设计、测试或实现的复杂模型独立审阅。

## 2. 任务与阶段

| ID | 状态 | Owner | 交付 | 依赖 |
| --- | --- | --- | --- | --- |
| C005-T00 | done | Codex /root；独立 Reviewer | 采用、前版归档和唯一实施入口 | 用户明确采用 |
| C005-T01 | done | 复杂模型架构、原语与测试作者 | 全局合同、完整高风险原语、窄骨架和阶段测试 | 用户采用开发需求；真实撤销证据在 T03 单列 |
| C005-M1 | done | 独立复杂模型 Reviewer | 阶段实现准备审查 | T01 |
| C005-T02 | todo | 简单模型 / 初级开发者 | 固定接口内完成原子替换与正式接线 | M1 通过及完整骨架 SHA |
| C005-T03 | todo | 手册执行者；复杂模型负责结论 | 使用说明、真实接续与最终证据 | T02 |
| C005-M2 | todo | 独立复杂模型 Reviewer | 完整链、工程和真实结果审阅 | T03 |

阶段 1 的入口是当前采用候选；出口是普通行为仍正确、高风险原语通过、阶段新行为测试可执行且有意义地失败、CLI/next 尚未发布替换操作。阶段 2 的入口是 M1 审定的完整骨架提交和冻结命令；出口是正式行为、真实使用与 M2 采用义务完成。M1 不表示替换功能已交付。

T01 原语用例归属 T01，交接时为 green。新替换行为测试由 T01 创建，归属 T02，初始带 `#[ignore = "C005-T02"]`；运行它们记录真实 red，编译失败、零测试、仅占位 panic 或只测私有 stub 不算 red。已有正确 caller 保持原行为，不要求全部造红。T03 不凭文案生成镜像 Rust 测试；确有真实新行为用例时由复杂作者在 T01 提前冻结、归属 T03 并同步其 test_files 和命令；没有则 T03 测试表为空。

### C005-T01：确定全局接口并实现可信基础

**Owner / 输入。** 复杂模型；采用决定、当前完整提交、撤销场景、schema/协议和冻结方法。

**范围。** design 的完整路径表与全部真实消费者；包括 number 改名影响的 ID/layout/render、命令与回复、state 载荷、failed/next/replay、runtime snapshot/load/request/effects/recovery、CLI begin/self 文案和当前格式 fixtures。先用全仓符号搜索核耦合，在本任务同步具体白名单。新增阶段资产为 `verification/commands.sh`、`experiments/runbook.md`；新行为测试放在拟 `crates/sheltie-runtime/tests/attempt_replace.rs`、`crates/sheltie-cli/tests/attempt_replace.rs`，既有 service/schema2_replay/crash/replay 和混合源码测试在本阶段完成耦合改动。

**测试。** `attempt_id_requires_number_and_rejects_retired_retry_field`、`attempt_number_allocation_checks_overflow_without_wraparound`、`replacement_atomically_ends_old_attempt_and_inherits_optional_frozen_inputs`、`second_replacement_rejects_without_removing_current_submit_or_fail_eligibility`、`replacement_does_not_consume_failure_budget_and_historical_fail_uses_original_prefix`、`zero_retries_still_allows_replacement_but_first_real_failure_blocks`、`replacement_quota_resets_for_new_occurrence_and_keeps_entered_from`、`superseded_submit_and_fail_reject_and_terminal_guard_takes_precedence`、`replacement_checks_target_and_qualification_before_reason_or_inputs`、`replacement_rejects_any_changed_frozen_observation_and_optional_rebinding`、`replacement_reason_accepts_exact_limit_and_rejects_one_more_byte_without_mutation`、`replacement_stats_are_fresh_post_state_bytes_and_old_binding_is_preserved`、`replacement_reason_field_is_required_nullable_and_unknown_fields_are_rejected`、`persisted_replacement_rejects_reason_combinations_number_gaps_and_second_superseded`、`replacement_records_one_atomic_pair_and_inherits_nonstats_refs_with_new_exact_stats`、`failed_history_uses_its_original_prefix_after_replacement_and_later_exhaustion`、`late_old_writes_and_second_replacement_are_rejected_without_new_records`、`replacement_replay_after_new_attempt_ends_preserves_reason_source_identity_and_bytes`、`qualification_precedes_reason_file_reads_and_modified_input_never_commits`、`replacement_and_submit_or_two_replacements_have_exactly_one_committed_winner`、`schema_three_and_missing_nullable_reason_are_rejected_without_rewriting_records`、`replacement_history_rejects_forged_reason_input_binding_and_snapshot_identity`、`replacement_complete_payloads_are_decoded_before_frozen_workbook_io`、`replacement_with_no_inputs_rejects_changed_entry_source_before_another_write`、`replacement_transaction_crash_boundaries_preserve_exact_committed_history`、`replacement_rejects_a_path_swap_after_opening_the_original_input`。

**执行步骤。**

1. 确认撤销需求与固定一次规则，更新上游 spec、architecture、protocol、storage、CONTEXT；选定唯一格式，不制作兼容层。
2. 逐一追踪 `AttemptId.retry` 的真实 caller，完成 number 接口和全部序列化/布局/fixture 改动，使既有命令完整可编译和可运行。
3. 完整实现 Superseded 载荷约束、失败计数及截至原 Attempt 的 failed 前缀校验。历史校验须通过真实 `validate_command_owner` caller，而非独立 helper 演示。
4. 完整实现冻结输入同句柄观察、严格 replacement 意图/响应/归属校验、精确任务书/stats 效果绑定和恢复原语。时间、主体、CAS、事务与文件政策由本任务作者解决。
5. 完整实现必要耦合的纯 Decision 和 runtime 库替换原语，固定参数、返回和可用原语；阶段二只接公开 CLI 与正常 next。CLI 不接受 replace，生产 next 不列 replace；不把未完成路径或假成功交给用户。
6. 写真实状态、runtime/CLI、请求重放、并发和 crash 用例。手写期望包含 replace→fail→begin→fail→重放首次 fail 的可达历史；T01 原语测试运行通过，T02 行为用例先禁用但能显式运行得到有意义 red。
7. 在 `verification/commands.sh` 固定 `primitives`、`future-red`、`feature`、`regression`、`gates` 的实际过滤器、测试数、失败签名和预算；在 runbook 固定 T03 任务、操作、计时、质量标准与停止条件。
8. 保持普通行为回归通过，短语义复核后保存原始输出与完整骨架 SHA，再交 M1。骨架中的 T02 占位只服务真实未来 caller，不加伪造状态、长期 dead_code 放行或额外状态库。

**oracle。** Superseded 字段组合及一次上限；number 与 failed 独立；损坏历史身份、输入或状态拒绝；COMMIT 后 brief/stats 原字节恢复。替换测试经真实 WorkService/CLI 入口断言状态与文件，不只断言 helper 返回错误。完整 feature red 可以因当前 CLI 不支持 replace 或骨架尚未完成而失败，失败位置必须符合已冻结的用户行为预期。

**验证与交接。** T01 创建脚本后运行：

```bash
bash specs/changes/active/C005-executor-continuity/verification/commands.sh primitives
bash specs/changes/active/C005-executor-continuity/verification/commands.sh future-red
bash specs/changes/active/C005-executor-continuity/verification/commands.sh regression
scripts/check-tests.sh
scripts/check-docs.sh
scripts/check-specs.sh
scripts/check-task.sh C005-T01 <T01 开工完整提交> --staged
scripts/check-task.sh C005-T01 <同一完整提交>
```

primitives 至少执行一项 T01 测试并全部通过；future-red 至少执行一项 T02 行为测试、实际非零退出且失败签名吻合，不能把该失败写为 feature PASS。改动跨 crate/状态/格式，运行工程完整门禁；命令文件记录隔离 `CARGO_TARGET_DIR`、`RUSTC_WRAPPER=`、counts 和 run ID。交接骨架完整 SHA、字段/符号、所有测试归属、有效 red、原语 green、fixture closure、预算、未跑平台和冻结 runbook。

**停止。** caller 无法接通、现有正确行为被破坏、载荷/原件来源不明或恢复只能猜字节时，本复杂作者修正接口和测试后再交接。不得把困难原语留给简单实现者。

## C005-M1：审查实现准备

独立复杂模型核总体边界、普通行为、所有接口 caller、原语真实实现、严载荷、历史失败前缀、精确效果与可信恢复。确认普通用户尚不能调用未完成替换，测试可编译、有非零执行与有效 feature red，T02 只需纯变换/接线即可完成。核白名单混合源文件与 test_files 精确同路径；缺接口或 oracle 时交 T01 修复再独立复核。

保存完整审查候选、输入闭包、原 run ID 与未完成义务。可引用同闭包下 T01 原运行，不机械重跑完整工程门禁。记录通过、需修改或阻断；通过只授权进入已采用的阶段 2。运行文档/治理和 diff 检查；`scripts/check-task.sh C005-M1 <M1 开工完整提交> --staged`，提交后以同基准再检查。

### C005-T02：实现固定替换行为并接通入口

**Owner / 输入。** 简单模型或初级开发者；M1 审定骨架完整 SHA、冻结接口、测试和 `verification/commands.sh`。先按学习导航确认每个参数来自哪里，不扩展接口。

**范围。** 生产代码只改 `core/work/next.rs` 的 Running 资格列举，以及 CLI `cli.rs`、`commands/attempt.rs` 的参数与分发。复用 T01 完整实现的纯 Decision、runtime 库 wrapper、观察、事务、stats/brief 效果及恢复；不重写这些基础。测试文件只删除本任务 ignore 标记；混合源码的测试段、断言、fixtures 和 snapshots 保持冻结。不修改载荷、schema、snapshot 归属、OS/并发/恢复政策或上游合同。

**测试。** `replacement_is_atomic_and_business_failures_use_history_not_attempt_number`、`replacement_does_not_consume_zero_business_retries_or_approve_a_gate`、`replacement_replays_original_file_reason_and_rejects_conflicting_intent`、`replacement_reason_has_exact_limit_and_missing_identity_is_not_found`、`replacement_refuses_modified_frozen_input_without_revoking_the_running_attempt`、`replacement_crash_windows_preserve_atomic_state_and_exact_brief_and_stats`、`stats_and_next_keep_one_snapshot_when_a_writer_begins_after_reader_load`、`status_card_active_mid_flow`、`public_next_offers_current_replacement_once_and_restores_quota_on_new_occurrence`、`post_commit_card_failure_returns_committed_response`。

**执行步骤。**

1. 记录完整骨架或最新独立测试修订 SHA，检查 M1 结论及工作树归属。
2. 跑 future-red，核实际测试数与既定失败签名，不能接受编译失败或零测试。
3. 直接调用已完成的 `WorkService::replace(work, attempt, reason, request_id)`；不改 `Command::ReplaceAttempt`、资格、继承、计数或纯 Decision。
4. CLI 接受 work、--attempt、--reason，理由经既有 `parse_text_arg`；输出只用持久 Response 的 `AttemptReplaced` 与 data/next，不回读或重算 stats/brief。
5. Running 的 next 在 fail 后、cancel 前按 `!state.has_replacement_of(&state.current)` 列 `ReplaceAttempt`；正常 CLI 与 next 一起接通。
6. 逐项运行 feature 测试，全部满足后仅删除本任务 ignore，复核正式 CLI 与旧迟到拒绝。
7. 运行受影响普通消费者、工程门禁和语义短审，保存原字节恢复/并发证据后交 T03。

**oracle。** 一次替换、第二次拒绝但当前可继续；旧迟到 submit/fail 拒绝；并发至多一成功；原 reason 文件删除仍重放；历史首次 fail 在当前 blocked 后仍 active；原 begin/replace 重放及当前 status/next 分开。测试名称和过滤器由 T01 固定，不由实现者自行挑通过用例。

**验证。**

```bash
bash specs/changes/active/C005-executor-continuity/verification/commands.sh feature
bash specs/changes/active/C005-executor-continuity/verification/commands.sh regression
bash specs/changes/active/C005-executor-continuity/verification/commands.sh gates
scripts/check-task.sh C005-T02 <阶段骨架或最新独立测试修订完整提交> --staged
scripts/check-task.sh C005-T02 <同一完整提交>
```

feature 内含 `scripts/task.sh C005-T02`，至少一项实际执行、全部通过；正式验收无本任务 ignore。保存候选、counts、run ID、精确字节与未跑项。原语未变时引用 M1 闭包及原运行，不重复扩散验证。

**停止 / 交回。** 无法按接口完成、需改测试/fixture、严格载荷或恢复政策时停止相关实现，给复杂作者最小复现、真实 caller、缺数据和失败输出。复杂作者在同 package 明确修复任务/范围/oracle；修订测试后独立复核，更新完整测试基准。简单实现者不得改断言迁就代码。

### C005-T03：按手册完成真实接续与说明

T03 若只有说明和真实操作记录，范围基准用本任务开工完整提交；若 T01 已预制并归属 T03 的 Rust 场景测试，复杂作者先将实际文件加入 T03 test_files、固定 allow_test_changes=false，并像 T02 一样以 M1 骨架或最新独立测试修订完整 SHA 核冻结测试。手册必须写明实际采用哪一种，初级执行者不自行选择基准。

**Owner / 输入。** 手册执行者；T02 固定候选、T01 runbook、真实任务与旧进程停止/隔离条件。结论、标准变更或不明失败由复杂模型分析。

**范围 / 操作。** 按 `experiments/runbook.md` 的具体命令先演示继续原 Attempt，再在真实需要时替换，完成原目标，记录输入/标准/gate 保持、额外解释、重做和总投入。完善 `skills/sheltie/SKILL.md` 的准确操作说明；涉及 skill 修改时读 writing-for-agents skill。故障注入不计真实撤销需求。

**oracle / 停止。** 新执行者使用原冻结输入与标准完成成果，旧正式提交被拒绝，普通重开不产生新 Attempt。无真实样本保留价值 not_run；不能人工制造额度问题。生产缺陷回到责任修复任务；不自行改合同、测试或成功标准。

**验证 / 交接。** 执行 runbook 冻结命令、`scripts/check-skill.sh`、文档检查和 diff 检查；有冻结 T03 Rust 行为用例才运行其 task.sh，否则用真实命令记录，不虚构零测试 PASS。范围检查为 `scripts/check-task.sh C005-T03 <runbook 已确定的完整范围基准> --staged`，提交后以同基准再检查。交接实际投入、候选、标准、原 run ID 与全部 not_run 项给 M2。

## C005-M2：独立完整验收

独立复杂模型核产品需求、完整 CLI→runtime→core、状态/计数、真实历史 caller、事务/恢复、next/结果、Rust 工程和首次读者真实接续结果。M1 未变化的原语/合同可引用其完整闭包与 run ID；实际新增行为和受影响消费者必须核新证据。机制、质量、成本、真实样本和平台分别结论，不把 M1、静态审查或 fixture PASS 当产品价值。修复交责任任务再复核；审查与证据按 M2 开工完整 SHA 记录，运行文档/治理和 diff 检查；`scripts/check-task.sh C005-M2 <M2 开工完整提交> --staged`，提交后同基准检查。不自行发布、安装或扩大范围。

### C005-T00 采用与单一实施入口

按用户明确顺序采用 C005 全范围，归档已独立验收的 C004 实现闭包并保留其授权延期。只激活 C005。目录、当前索引、受影响引用与角色准确；doc/spec/TOML/事实与独立短审通过再提交。原 C004 真实试用/工具版本/onlinefresh/未知LEAK边界不因归档消失。

本轮具备总体能力的作者承担准备及主体，保留 T01/M1/T02、独立 oracle、完整 caller 和未参与编写的 Reviewer。T01 可以完整实现必要耦合类型/高风险状态原语，T02 才接公开 replace/next；不制造假成功或正常入口的占位。实际接口、测试与阶段资产在 T01 固定。Schema 4、cli-result/v4 单一目标，旧 Store 1/2/3 原件保留并整体拒绝；Workbook/Flow/result/digest 版本不变。

## T01 阶段调整：受影响状态卡 oracle

原 T10 的 `status_card_active_mid_flow` 保留原真实场景，因正式 next 新增 replace 操作，在 T01 预制完整未来 snapshot 并转为 C005-T02 的 ignore 用例。T01 普通回归明确跳过该未来 oracle；显式 future-red 核它实际少 replace 行。T02 只删除 ignore，不修改场景、断言或 snapshot。新 public-next 用例同时覆盖原资格、额度耗尽、业务重试和新 Occurrence。此覆盖迁移由 M1 独立审查，不把尚未通过的历史场景报告为 PASS。

同一策略用于原 C002-T29 的 stats 单快照用例：旧 reader 仍手写 begin/cancel；新 running reader 的完整未来预期增加 replace。原场景和单快照断言不变，当前显式 red、T02 只删 ignore，暂跳事实单列。

原 C002-T25 的 `post_commit_card_failure_returns_committed_response` 同样保留完整提交错误场景，未来 next 新增 replace 后转为 C005-T02 冻结用例。三个既有场景的阶段 skip 与实际 future-red 一并记录，不算已通过。

完整 T01 骨架：`8d00a29e10810c79bb5b44bdc006dc021e26637b`。M1 通过后 T02 用此 SHA 核固定测试；M1 仅记录不改变源码或 oracle。
