# C005 实施计划

状态：`proposed`；未采用，不执行本计划。任务全部 `todo`，实现与实验为 `not_run`。共用规则见[提案实施指南](../../../guides/proposal-implementation.md)；工程门禁与提交格式见[工程规范](../../../engineering.md)。

## 1. 先读项目，再选动作

首次接触项目的实现者先读 `CONTEXT.md`、`specs/README.md`、本 package 的 README/spec/design/validation，再读任务卡点名的合同与源文件。C004 完成候选是采用输入：先演示 `work handoff` 恢复原 running Attempt，再确认这次需要撤销旧提交资格，而不是仅缺交接材料。

强模型先定整体架构、格式变化和关键验收判据，再按里程碑搭建该阶段骨架与测试。简单模型只实现已冻结边界，不修改接口、断言、夹具期望或快照。新增缺口交回本阶段骨架作者；review 由未参与该阶段设计和实现的强模型完成。不会预先写完整 package 的全部测试。

所有拟新增路径均由阶段骨架作者创建。骨架提交后填写提交 hash；填空任务用此 hash 作为 `check-task.sh` 显式基准。`scripts/task.sh` 只用于已登记 Rust 用例的实现任务；骨架、文档、实际实验和独立 review 不因没有测试归属而强行运行它。

## 2. 任务与里程碑

| ID | 状态 | Owner | 依赖 | 交付 |
| --- | --- | --- | --- | --- |
| C005-T01 | todo | 强模型 / 架构与 M1 骨架 Owner | C004 完成，采用决定 | 上游合同、格式闭包、core 骨架与 M1 判据 |
| C005-T02 | todo | 简单模型 / core 实现 Owner | T01 | 纯状态替换、计数和投影 |
| C005-M1 | todo | 独立强模型 | T02 | core 契约审查；未解决阻断项不进 M2 |
| C005-T03 | todo | 强模型 / M2 骨架 Owner | M1 通过 | runtime 事务、快照与恢复骨架及测试 |
| C005-T04 | todo | 简单模型 / runtime 实现 Owner | T03 | 替换写链、真实输入观察、幂等恢复 |
| C005-M2 | todo | 独立强模型 | T04 | 真实 Store/caller/崩溃链审查 |
| C005-T05 | todo | 强模型 / M3 骨架 Owner | M2 通过 | CLI 骨架、真实场景与手工实验固定口径 |
| C005-T06 | todo | 简单模型 / CLI 实现 Owner | T05 | replace 命令、help、错误与 skill |
| C005-T07 | todo | 实验执行者；强模型负责证据复核 | T06 | 完整回归与真实接续原始证据 |
| C005-M3 | todo | 独立强模型 | T07 | 全链审查与采用范围验收结论 |

里程碑只审查和记录。review 文档提交使用审查开始时的固定候选作为 `check-task.sh` 显式基准，避免把先前实现与快照更新算作本次审查改动。审查发现需要改测试或代码时，交回对应骨架 Owner，按共用指南登记独立修复任务，同步 plan/tasks 并固定新的测试基准；审查者不得替实现者补代码再签署独立通过。计划任务与 `tasks.toml` 中的 T/M 条目一一对应。

## 3. M1：把替换定义为一个有界原子动作

### C005-T01：总体架构、合同与 core 骨架

**Owner / 依赖。** 强模型；C004 完成候选和人采用本方案。先确认是否有实际撤销需求。

**入口。** `ids.rs::AttemptId`、`flow/def.rs::NodeDef`、`flow/parse.rs::NodeDto`、`work/state.rs::AttemptStatus`、`work/command.rs::Command/Reply`、`work/decide.rs::decide`、`work/next.rs::legal_next`。路径均在 `crates/sheltie-core/src/`。跨层 caller 是 runtime 的 `service.rs`、`snapshot.rs`、`load.rs`、`effects.rs` 与 CLI 的 `cli.rs`、`output.rs`、`error_map.rs`；先用 `rg` 找出全部穷尽分支和字段引用。

**输入 → 输出。** 输入为 C004 的当前类型和冻结合同；输出为 spec §2–6 的精确上游合同、架构图、字段与错误表、可编译 core 骨架、M1 用例及其固定期望。整体接口在此定稿，后续阶段只按自己的 caller 补骨架。

**步骤。**

1. 固定采用起点 hash，核对本文真实路径；同步上游词汇、产品、架构、Workbook/协议/Store 合同和 tasks.toml。
2. 定 schema 与响应版本，明确旧库只读拒绝；字段重命名涉及的类型、序列化和 caller（包括 `work/layout.rs::attempt_dir`）在同一个提交闭合，不能让同一版本承载两种 retry 含义。
3. 定 ReplaceAttempt 参数、AttemptReplaced 回复、superseded 信息、max_replacements 范围及派生计数接口。补全部穷尽匹配，不用临时兜底假成功。
4. 搭建本阶段纯函数骨架，未实现行为用带 `C005-T02` 的占位；创建归属 T02 的正反例和固定期望。后续 runtime/CLI 行为测试到对应阶段再写。
5. 核对普通流程仍完整工作；尚未实现的替换不得通过公开 CLI 路径触发。若编译闭包必须提前完成跨层机械改动，明确记录这些改动与基准，不抢做其后行为。

**验收用例。** 拟新增、归属 T02：`replace_preserves_occurrence_and_frozen_inputs`、`replace_rejects_non_running_attempt`、`replace_at_limit_keeps_old_attempt_running`、`replacement_does_not_consume_business_retry`、`attempt_number_advances_after_failure_and_replacement`、`replace_rejects_changed_frozen_input`、`replacement_limit_accepts_32_and_rejects_33`、`replace_closes_open_observation_without_copying_completed_qualification`。位于 core 的相应 `#[cfg(test)]` 模块，测试名不带任务编号。用手写状态验证 2 次替换后第 1 次失败，不能只造默认 0。

**停止 / 验证 / 交接。** 输入身份或历史快照无法从参数核实时，先修接口；不得要求简单模型在实现里猜。运行完整编译门禁、未受影响回归和新用例的预期失败；仅新增未实现行为需要红，编译失败不算红。交接接口表、用例路径、预期红列表、骨架 commit 和下一任务白名单；T01 提交不要求 `task.sh C005-T01`。

### C005-T02：实现 core 替换、计数与投影

**Owner / 依赖。** 简单模型；T01 骨架、接口表与测试基准已冻结。

**入口 / 范围。** core 的 `ids.rs`、`error.rs`、`flow/{def,parse,compile,graph}.rs`、`work/{state,command,decide,next,layout,render,mod}.rs`。只填 T02 的实现占位与必要导出；测试只能删除本任务 ignore 标记。

**输入 → 输出。** 输入为固定 WorkState、图、冻结输入观察和 Context；输出为同一个 Decision 中的旧 superseded / 新 running、精确 brief/stats 效果或明确拒绝。

**步骤。** 校验目标状态和上限；继承输入并核对；检查分配顺序号；更新旧新 Attempt，并在同一 Decision 关闭旧 open 观察、保持 completed 历史及新观察为空；生成 stats 和任务书；将合法 next、统计、begin/fail 中的业务重试逻辑一起改为 failed 计数。不能以 number 判断业务上限。

**验收用例。** T01 创建的归属 T02 用例；骨架完成时本字段改为「测试」，列出实际存在的名字和路径。正例为允许的首次替换；反例只改一个条件：旧状态、输入摘要、上限或目标 Occurrence。旧 attempt submit/fail 应拒绝，新 attempt 成功后走原合法出边。

**停止 / 验证 / 交接。** 不得改快照或期望让实现通过；接口缺口交回 T01 Owner。运行 `scripts/task.sh C005-T02`、工程门禁与 `scripts/check-task.sh C005-T02 <T01骨架commit> --staged`。交接绿色用例、真实计数与新旧状态差异；一个任务一个提交。

### C005-M1：独立强模型 review

固定候选和测试输入闭包，逐条核 spec 的合法/拒绝规则，检查所有 begin/fail/next/render caller 都使用同一计数定义，特别是 max_retries=0、替换上限=0/32、输入修改和序号溢出。核上限拒绝仍允许原尝试完成，人工暂停没有隐式状态改变。证据和 findings 写 review/validation；未解决阻断项先返回 T01/T02。

## 4. M2：把状态转换接入真实持久链

### C005-T03：runtime 骨架与故障 oracle

**Owner / 依赖。** 强模型；M1 通过。

**入口。** runtime 的 `WorkService::begin/run_command/replay`（`service.rs`）、`RequestIntent`（`request.rs`）、`snapshot.rs::check_data`、`load.rs`、`effects.rs`、`store/{schema,read,commit}.rs`、`failpoint.rs`；阅读 storage 合同的事务与恢复表。

**输入 → 输出。** 输入为已通过的 core Decision 合同；输出为 `WorkService::replace` 与意图/快照/效果骨架，以及拟新增 `crates/sheltie-runtime/tests/attempt_replace.rs` 的归属 T04 测试。只为真实缺失故障窗口补 failpoint。

**步骤。** 沿 begin 的完整写链定位锁、输入观察、提交、prepare_attempt、write_file、快照核对和恢复；为新命令逐入口补接口。用独立临时 SHELTIE_HOME、手写 reason 文件和固定时钟构造 oracle；校验主体来自系统 API，不信 USER 环境变量。测试不得直接把替换成功写进 Store。

**验收用例。** 拟新增归属 T04：`replace_replay_returns_original_attempt_and_brief_bytes`、`replace_rejects_changed_request_reason`、`replace_before_commit_leaves_old_attempt_running`、`replace_after_commit_recovers_one_new_attempt`、`late_submit_after_replace_is_rejected`、`submit_and_replace_accept_at_most_one`、`replacement_snapshot_rejects_mismatched_attempts`、`old_begin_replay_after_replace_keeps_historical_next`、`replace_interrupts_running_execution_and_rejects_late_completion`、`replace_rejects_prior_schema_without_mutation`。重放正例删掉原 reason 文件，另造新 request-id 负例；崩溃检查字节与摘要，不只检查文件存在。

**停止 / 验证 / 交接。** 不能独立核对历史替换响应时回合同 Owner；不放宽快照校验。运行编译门禁和普通持久链回归，记录新行为预期红。交接 T03 骨架 commit、故障点、固定快照与 T04 用例归属；不要求 `task.sh C005-T03`。

### C005-T04：实现事务、快照和幂等恢复

**Owner / 依赖。** 简单模型；T03 骨架冻结。

**入口 / 范围。** T03 点名的 runtime 源文件；`attempt_replace.rs` 测试只启用，不改断言。

**输入 → 输出。** 输入为 CLI 尚未接入的 replace 参数；输出为持久响应或无状态变化的拒绝，COMMIT 后失败准确显示 EFFECT_PENDING。

**步骤。** 构造完整目标与 reason 参数的意图；先查重放，再真实观察输入；从系统取得时间/主体；运行 core；在一次事务提交旧新尝试、审计、request，并将旧 open 原生执行记录关闭为 interrupted；登记并发布新目录/brief；严格校验新旧快照与原审计，恢复历史精确字节。并发负例不得靠自动 retry 隐藏 revision 冲突。

**验收用例。** T03 创建的归属 T04 用例；阶段骨架完成后登记实际名字。正例覆盖完整替换与同请求重放，反例覆盖 late submit、不同 reason、损坏快照、输入改变与 COMMIT 前后故障。

**停止 / 验证 / 交接。** 不改 oracle、不新增第二份计数/状态；缺 caller 交回 T03。运行 `scripts/task.sh C005-T04`、工程门禁与 `scripts/check-task.sh C005-T04 <T03骨架commit> --staged`。交接原始输出、事务表与效果字节；一个任务一个提交。

### C005-M2：独立强模型 review

固定候选；从 WorkService::replace 到 snapshot、Store commit、effects 和 replay 逐个追踪。复核真实输入观察在事务前、旧新状态同一事务、COMMIT 后不二次替换、旧请求历史资格与当前状态分离。核拒绝旧格式前未写数据；重新检查新增效果窗口所需故障测试，避免重复跑无关昂贵实验。未解决阻断项返回 T03/T04。

## 5. M3：形成可用入口并验证增量价值

### C005-T05：CLI 骨架、场景与真实实验配置

**Owner / 依赖。** 强模型；M2 通过。

**入口。** CLI 的 `cli.rs::AttemptCmd`、`commands/attempt.rs::run`、`commands/work.rs::resolve/next_lines`、`output.rs`、`error_map.rs`；C004 `work handoff` 的实际符号；`skills/sheltie/SKILL.md`。

**输入 → 输出。** 输入为完整 runtime API；输出为 replace 参数/输出骨架、拟新增 `crates/sheltie-cli/tests/attempt_replace.rs` 的归属 T06 测试，以及 validation 中固定的真实接续任务与对照。

**步骤。** 写严格 clap 参数与 help；固定 JSON 用同一 Response 输出旧新身份、不回读当前 Store；场景用真实子进程和临时目录，经 begin→replace→迟到拒绝→新 submit→gate/结束完整调用。添加普通恢复场景证明没有命令时仍可原 submit。根据真实任务配置上限、工作区和停止旧进程的人工步骤。

**验收用例。** 拟新增归属 T06：`replace_cli_returns_new_brief_and_original_inputs`、`replace_cli_replay_preserves_original_json`、`replace_limit_error_keeps_old_submit_available`、`reopened_session_continues_original_attempt_without_replace`、`replacement_preserves_candidate_and_gate_requirements`。已有完整场景可能已经通过，记录其回归结果，不强求全部先红。

**停止 / 验证 / 交接。** 无真实替换需要时价值实验保持 not_run，不人为耗尽额度；不据此扩大宿主适配范围。运行编译、help/参数 oracle 与既有场景回归；交接 T05 commit、CLI 用例、实验输入闭包和对照材料。

### C005-T06：接入 CLI 与使用说明

**Owner / 依赖。** 简单模型；T05 骨架与测试冻结。

**入口 / 范围。** `cli.rs`、`commands/attempt.rs`、`output.rs`、`error_map.rs`、`skills/sheltie/SKILL.md`；测试只启用归属 T06 的用例。

**输入 → 输出。** 参数解析成功后调用真实 WorkService::replace；JSON 与文本准确给出旧新 Attempt、任务书和 next。上限拒绝指出原 Attempt 仍能按C004已有收集/提交资格继续；不能提示 Work 已不能恢复。

**步骤。** 填参数分发、runtime 调用、reply 检查、错误映射和原快照输出；skill 说明先读交接视图、按需替换、历史 next 的范围与共享工作区人工责任。不能在 skill 内建立状态文件。

**验收用例。** T05 创建的归属 T06 用例，实际名字在骨架提交时登记。正例走新尝试提交，反例走旧迟到提交与上限拒绝；同时运行普通单 agent、门槛、终态回归。

**停止 / 验证 / 交接。** 发现缺数据字段回 T05，不从当前 Store 拼补历史响应。运行 `scripts/task.sh C005-T06`、工程门禁、`scripts/check-skill.sh` 与 `scripts/check-task.sh C005-T06 <T05骨架commit> --staged`。一个任务一个提交。

### C005-T07：完整回归与真实接续证据

**Owner / 依赖。** 实验执行者；强模型独立核输入与口径。T06 完成，不承担代码修复。

**入口。** validation 的机制矩阵与真实接续实验、C004 handoff/result、已实现 replace CLI、package 的 review/progress。

**输入 → 输出。** 输入为固定候选、Workbook 摘要、任务、原 AttemptId、宿主版本与已确认停止条件；输出为原始命令/日志、接续结果、人工投入、重做与约束保持证据。

**步骤。** 完成机制矩阵；对同一类任务分别记录原恢复与显式替换结果；核上限拒绝后仍能继续原 Attempt；在实际需要撤销的任务上使用替换并完成原目标。只统计真实时间，不把 fixtures 的故障注入计成产品收益。

**验收用例。** validation 预先冻结的机制与真实任务表；本任务不另创建 Rust 用例，不要求 `task.sh C005-T07`。

**停止 / 验证 / 交接。** 失败返回相应实施任务修复；缺真实样本保留 not_run 并记录原因，不伪造 PASS。运行本任务实际实验命令、文档门禁与 `git diff --check`，按工程规范完成提交检查，并运行 `scripts/check-task.sh C005-T07 <T06完成候选> --staged`。交接完整输入闭包、原始 run ID、候选与所有未完成项。

### C005-M3：独立强模型全链 review

在固定候选上核产品目标、设计边界、真实 caller、故障恢复、输入/候选/证据保持和 junior 交接可执行性。分别报告机制结论与真实价值结论；静态/fixture PASS 不能替代真实接续结果。确认普通恢复不被强制替换、独立审查不被自动放宽、没有偷偷引入额度预算或驱动器。人决定是否完成本 change 及版本发布，review 不自动发布。
