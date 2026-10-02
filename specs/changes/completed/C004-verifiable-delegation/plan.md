# C004 实施计划

状态：`completed`（本轮实际实现闭包）。用户已采用，原真实试用与环境义务按授权延期；采用范围见 [adoption](adoption.md)，验收与缺项见 [validation](validation.md)。

## 1. 首次读者与采用范围

实现者具备 Rust/CLI 基础，但第一次接触 Sheltie。先读 CONTEXT、specs 入口、本 package README/spec/design、[实施指南](../../../guides/proposal-implementation.md)，再按下表定位；不要求通读全部历史。

| 阅读入口 | 先理解什么 | 本任务用途 |
| --- | --- | --- |
| `core/flow/{def,parse,compile}.rs` | 声明、严格解析、合法图是三件事 | result 标记由定义校验，不从报告推断 |
| `core/work/{state,render,result}.rs` | state 事实与纯投影；result.rs 拟新增 | 明确终点选择、当前指针 |
| `runtime/{service,load}.rs`、`store/read.rs` | 可信装入、SQLite 读快照与实际 revision | T01 完整准备读取原语 |
| `cli/commands/{work,mod}.rs` | 参数、只读分派和 request-id 拒绝 | T02 只接已确定接口 |
| 本 package 的 stage-1.md（拟） | 实际符号、测试名、命令、允许改的函数体 | 初级实现者的唯一交接入口 |

T01 将每项真实需求映射到采用能力，同步裁剪 README/spec/design、本表、依赖、tasks 和 oracle。只需成果选择不连带接续，只需接续不连带结果语法或新方法；未选能力移出 active 范围并保留候选，不记 done/跳过/PASS。真实使用验证保留，但只核被采用能力。

负责人具备总体设计与实现能力，合并准备作者和实现者角色，避免不必要交接；保留 T01/M1/T02 的阶段边界、独立期望和未参与编写的 Reviewer。T01 可完整实现耦合的纯类型与读取原语，T02 才接公开 result 命令，不用假成功或人为占位制造红。

## 2. 两个阶段

| ID | 状态 | Owner | 依赖 | 可观察结果 |
| --- | --- | --- | --- | --- |
| C004-T00 | done | Codex /root；独立审阅者 | 用户明确采用 | 采用范围、授权与唯一实施入口 |
| C004-T01 | done | Codex /root 与 runtime 负责人 / 架构与测试作者 | 人采用、C007 或同等近期需求 | 总体合同、可编译窄骨架、可信读取原语、真实测试和交接手册 |
| C004-M1 | done | 未参与 T01 的复杂模型 | T01 | 阶段入口可实施；原语绿、新行为有意义红、现有调用者闭合 |
| C004-T02 | done | Codex /root / 完整行为实现者 | M1 | 冻结接口上的 status/result 完整真实链 |
| C004-T03 | done | Codex /root / 方法实现者 | T02 | 已审定方法与使用说明按固定材料接通 |
| C004-T04 | done | Codex /root；独立事实复核者 | T03、用户环境跳过授权 | 试用前提盘点与补验入口；实际新任务质量/成本/重开 not_run |
| C004-T05 | done | Codex /root；独立 Reviewer | T02 冻结用例的规范根路径期望修复 | M1、实际 alias 失败 |
| C004-M2 | done | 未参与被审设计/测试/实现的复杂模型 | T04 | 完整采用闭环、Rust 工程、方法和真实结果 |

阶段交接时，T01 作者在本 plan 的对应实现任务卡写 `**测试。**` 与已存在的真实测试名，手册链接同一清单；任务标题使用 `### Cnnn-Tnn` 供 check-tests 识别。尚未建立的用例只写“验收用例”，不提前声称测试已经存在。

骨架指完整的类型、接口、caller 与测试，不是让初级开发者设计剩余架构。原语和耦合接口确需完整实现时由 T01 作者承担。每任务先短语义复核；M1/M2 对相同闭包已核内容引用原结论，不重复审查或整套门禁。单任务提交仍按工程规范。

### C004-T00 采用与实施入口

**范围。** 移动 C004 package、修正受影响链接，更新规格与 change 入口。采用全部 C004 能力，按用户指示记录环境无法执行的义务为 `not_run` 并保留补验入口；不编造用户、真实任务、usage 或价值结论。仅采用本方案，C005–C008 依次执行。

**验证。** docs/specs/TOML、链接与独立事实复核。一个任务一个提交。

### C004-T01 总体架构、读取原语与阶段测试

**测试。** `all_read_payloads_reject_unknown_fields_before_frozen_workbook_io`、`final_work_without_selections_reports_empty_explicit_result`、`gate_blocks_final_result_and_missing_approval_is_corrupt`、`missing_selected_reference_is_corrupt_even_when_effects_are_pending`、`pending_effects_withhold_final_artifacts_without_changing_flow_status`、`pending_resource_loading_compares_logical_path_and_observed_bytes_separately`、`result_rejects_resource_reference_digest_or_size_drift_without_changing_originals`、`result_selection_defaults_false_and_accepts_explicit_true`、`result_selection_rejects_non_boolean_and_unknown_slot_fields`、`result_supports_start_resource_and_engine_stats_input_slots`、`result_uses_sorted_terminal_slots_and_preserves_complete_references`、`result_uses_specific_frozen_version_after_real_rework`、`resume_current_attempt_keeps_brief_and_inputs_after_end_but_removes_drafts`、`resume_running_attempt_has_frozen_inputs_and_declared_draft_paths`、`resume_without_current_attempt_is_null_in_text_and_json`、`schema_two_is_rejected_without_migrating_or_rewriting_original_bytes`、`selected_input_and_output_keys_must_be_unique`、`selected_input_must_match_frozen_source_path_digest_and_size`、`selected_optional_input_or_output_is_rejected`、`selected_output_must_belong_to_terminal_attempt_and_fit_declared_limit`、`selected_results_on_nonterminal_input_or_output_are_rejected`、`status_read_combines_revision_and_current_frozen_pointers_without_writing`、`status_read_rejects_incomplete_or_conflicting_request_audit_closure`、`status_read_reports_this_work_pending_effects_and_ignores_unrelated_corruption`、`status_read_uses_one_snapshot_when_a_new_attempt_commits_between_queries`、`succeeded_work_requires_successful_terminal_attempt`、`terminal_required_input_and_output_can_be_selected_results`、`uncompleted_and_cancelled_work_have_no_final_artifacts`。

**Owner/输入。** 复杂模型负责契约、格式、系统边界与独立期望。输入为采用记录、当前 C002 合同、真实消费者和需求证据。

**范围。** 上游合同与本 package；tasks 中的精确源码/测试/方法文件。所有拟新增位置在本任务创建，给 T02 留的是已有符号，不是路径猜测。

**步骤。**

1. 固定采用映射、DTO/错误码、唯一格式与成功终点算法，说明绑定者与生产者区别；判断哪些既有消费者和 fixture 会变化。
2. 完整实现 `Store::read_work_bundle` 的单一读事务、requests/audit 两边归属、Start 定位、严格 effects 和可信上下文；给 service/result helper 显式参数，不越过私有字段。原语归 T01，不能留给初级模型猜 SQLite 或文件信任规则。
3. 固定实时 StatusReadView 与磁盘卡的字段范围，解决 refresh→mark_published 顺序；完整处理受影响的类型/调用者。若接口变化不能独立保持可编译和正确回归，就由本任务完成必要耦合行为，不制造假成功或兼容路径。
4. 完整创建结果/接续纯接口与服务原语；CLI 接线形状在 stage-1.md 冻结，T02 才创建公开枚举项并接通。负责人具备总体实现能力，不为制造可编译骨架加入占位、假成功或公开的未完成命令。
5. 写 T01 原语绿例和 T02/T03 的合法、单条件拒绝、pending/终态、真实 CLI/SQLite/文件用例。新行为暂用对应任务 ignore；原本正确的回归保留通过，改变的合同/快照由复杂作者同步并独立核对。
6. 准备最小方法的确定图、task/project 输入、说明模板与终点选择；有 C007 confirmed 方法时只复制需要的材料。T03 不负责选择节点、上限或批准政策。
7. 实际运行编译/既有回归、原语用例和定向新行为测试，保存非零测试数与预定失败。编译失败、环境失败、零测试及只有占位 panic 不替代真实 caller 断言。
8. 写 stage-1.md：实际文件/函数、允许填的主体、不得改的字段/断言、真实测试名、精确命令/期望、原始 run、完整骨架提交、T02/T03 文件白名单和停止交接人。

**关键 oracle。** 同快照不会拼两版事实；request 工作索引被改、audit 关联缺 request 或 published=1 的非法 effects 均拒绝。无选择不同于损坏缺引用；另一 Work pending 不隐藏本 Work。源身份与 gate 不由正文产生。

**验证/交接。** T01 原语测试及受影响消费者；源码/跨 crate/协议变化按工程门禁，docs/specs/tests/skill 按闭包。原语测试归 T01，接口行为归 T02，方法场景归 T03，不用 M 编号归属。新增非代码原文仅保存一次；本机 nextest 0.9.140 与配置要求 0.9.145 的差异按 stage-1.md 的显式环境例外保留，补充实际测试不覆盖未执行版本门禁。向 M1 交完整实际材料；未就绪入口不交实现者。提交 `chore(work): 准备成果与接续架构测试`。

## C004-M1 复杂模型检查阶段就绪

固定 T01 候选，核采用范围、全部真实 caller、类型与参数、原语真实绿例、T02/T03 测试非零且有辨别力、未完成入口未激活、白名单测试冻结和 first-reader 手册。可在自有临时副本改一行断言，证明 check-task 拒绝，再恢复，不动真实候选。

M1 只证明交接可实施，不证明最终功能或用户收益。需修改回 T01 作者；Reviewer 不代写 oracle。通过后固定阶段骨架/测试修订完整 SHA，T02 不使用随意 HEAD 或初始 README 基线。

### C004-T02 按冻结接口完成通用行为

**测试。** `result_selects_terminal_bound_input_and_sealed_output_in_key_order`、`result_hides_artifacts_until_gate_and_file_effects_are_complete`、`result_distinguishes_cancelled_work_and_success_without_explicit_selection`、`result_lists_explicit_terminal_bindings_and_sealed_outputs_without_writes`、`result_is_empty_until_the_terminal_gate_is_approved`、`result_distinguishes_no_selection_from_cancelled_and_active_work`、`status_provides_the_current_brief_frozen_inputs_and_running_drafts`、`result_rejects_request_id_before_opening_a_store`、`result_returns_frozen_references_without_claiming_source_bytes_were_rechecked`、`result_rejects_a_published_request_with_invalid_effects_without_repairing_it`、`result_hides_committed_outputs_until_their_file_effects_finish`。

**输入/前提。** M1 通过、stage-1.md、骨架完整 SHA。先核相同输入闭包、允许文件和非零测试；不依口头补架构。

**步骤。**

1. 读本任务的纯函数注释、DTO 和测试，再跑 `scripts/task.sh C004-T02`，核预定失败信号。
2. 填终点选择与当前指针的已定主体，调用 T01 可信读取/投影接口；不自行新增查询、缓存或文件观察政策。
3. 按固定数据流接 parser/compiler、service、CLI 与只读分派；结果、next 和 revision 取同一上下文。
4. 覆盖 empty/nonfinal/pending/损坏的既定路径；文本与 JSON 同源，查询不恢复、刷卡或写业务行。
5. 运行冻结用例和真实消费者，移除本任务 ignore；不改签名、断言、快照、格式、依赖或上限。
6. 将开发二进制标识为 `0.3.0-rc.1`，同步 workspace/lock 与 Unreleased 说明，区别于已发布的 v0.2.0；不改变已冻结 schema/DTO 或发布外部资产。保存命令/退出码/测试数，短审影响链，通过适用门禁后核范围并提交。

**停止。** 缺必要接口、不能在白名单接通、必须改 oracle/格式或出现来源/原子性缺口时回 T01 作者，在本 package 新增修复任务并重固定测试基准。难度本身不算停止；不得改测试迁就实现。

**验证/交接。** 手册列真实测试名与命令；本任务测试及跨 crate 完整门禁。范围命令为 `scripts/check-task.sh C004-T02 <M1骨架或最新独立测试修订完整提交> --staged`，提交后同基准再核。交实际响应/错误和稳定候选给 T03；提交 `feat(work): 接通成果与接续读取`。

### C004-T03 按已审方法接通使用路径

**测试。** `code_change_rework_selects_the_exact_reports_bound_by_the_terminal_attempt`、`a_report_summary_cannot_bypass_the_code_change_review_edge`。

**前提。** T02 可用接口；T01 已写定方法和 T03 场景。文件为 examples/code-change、skill 和实际方法消费者；实现者不从零设计图。

**步骤。** 核模板和候选；填写已允许的说明与输入位置，按固定图绑定成果；说明先查询当前 status、继续原 Attempt、执行失败重试与内容返工。以真实 CLI 走完成/返工/必要 gate，读实际路径；同步受影响说明，跑 `scripts/task.sh C004-T03` 和手册命令。已有正确场景不强制造红，测试/快照仍冻结。

**正反。** implement→review→back→复核→deliver，具体终点引用正确；摘要说通过不形成非法边；未批准 gate 不给 final；普通报告的 commit 不变为引擎事实。

**停止/交接。** 方法须改图、上限、结果合同或约束时回架构作者；保持用户与其他人的修改。范围基准同 M1 或最新测试修订 SHA，交 actual CLI trace 与 first-use 指南给 T04；提交 `feat(workbook): 接通已审代码变更方法`。

### C004-T04 按固定规程核试用前提与真实增量

**本轮授权范围。** 用户允许环境无法执行的操作记录后跳过。实际参与者/新配对材料/宿主会话控制不可用时，本轮交付前提盘点和准确 not_run/补验入口，不把原真实增量义务改成通过。真实 run 仍按以下原规程后续执行。

**前提。** T03 稳定候选；复杂作者在 stage-1.md 预先确定新任务选择原则、质量/成本口径、最大投入、重开步骤与停止规则，实际样本在运行前由该作者确认。

简单模型执行冻结步骤、保存原文、记活动和未完成；真实用户/授权和独立质量不能由模型模拟替代。复杂模型分析是否改善具体摩擦。采用 C007 方法/指标，已解决任务只作机制回归；新收益比较用新等价任务。至少实际关闭重开一次，usage 未知为 null。

质量下降、预算耗尽、输入漂移或缺真实前提时停止相应 run 并保留缺失，不重跑凑绿。纯记录跑文档/计算核对，无关 Rust 不重跑。交候选、原文、质量与成本给 M2；提交 `docs(specs): 记录成果与接续使用证据`。

## C004-M2 复杂模型审完整采用闭环

固定候选与输入闭包，核产品承诺、INV、真实来源、终点/gate、当前/历史、实时/磁盘卡、全部消费者、只读副作用、方法与用户质量/总投入。基础阶段结果只有相同闭包才引用；不能把 M1 就绪当 M2 PASS。

先语义短反馈，再在稳定候选运行完整工程门禁及有预算的定向突变，包含实际 CLI 消费者。未执行、timeout、平台和用户缺项分别报告；修复交明确作者并增量复核。全部实际采用义务通过才可 completed；发布另行决定。

历史 M1 基准：`0cedb8c1f3ee7ba8d4041151a5382d3b9e8abb4e`。Reviewer `/root/independent_review` 通过阶段准备；当前 T02/T03 采用末尾的最新独立测试修订基准。公开能力、真实价值与 M2 未完成。

### C004-T05 修复冻结用例的规范根期望

**Owner。** 复杂作者 Codex /root；独立 Reviewer `/root/independent_review`。仅修 `work_result.rs` 的 expected 管理根路径：临时目录可经系统别名 `/var` 创建，真实 Home 按规范解析为 `/private/var`。用 std canonicalize 独立确定已存在夹具 Work 目录，再拼固定 `start-inputs/topic`。摘要、大小、来源与只读判据不变，不用被测 result 生成答案。

**验证。** 实际 T02 draft 先复现 11 用例中 10 PASS/1 FAIL（路径别名）；复杂作者修期望后相同真实消费者 19/19 PASS（含 11 T02 和 8 T01 runtime），独立 Reviewer 核期望和规范根合同通过。此次提交保留所有 T02 ignore，生产接线与开发版本变更仍归 T02，不把 draft 功能验收写成 T05 产品 PASS。完整修复 SHA 作为 T02/T03 最新独立测试基准。

T02/T03 最新冻结测试修订：`2914e8564047809659c4610d151f9ea6ceef7476`，取代其测试检查基准；M1 原始候选仍保留，T05 只修规范根 oracle。
