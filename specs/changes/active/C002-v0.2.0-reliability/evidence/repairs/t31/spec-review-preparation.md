# T31 独立 Spec 审查准备

状态：**WIP**。本报告准备 R01–R19、O01–O13、N01–N14 的最终审查入口，并记录独立源码增量结论；不标记 T31 完成或 M1 通过。Reviewer：`/root/spec_review`，未参与实现。本轮只读源码、合同、测试及已有原始日志，未运行 Cargo 或 Linux 测试；仅按 Owner 指定写本报告，不修改代码、plan 或其他实施者文件。

产品基线为 `b926789`；T31 尚未有产品提交。本矩阵初次准备使用普通 clone 临时测试提交 `6baee53376f8ba631d5c92777881ec82692576de`，旧输入闭包固定在 [原 candidate-input.txt](gate-source-6baee-before-public-contract-oracles/candidate-input.txt)。公共合同 oracle 候选 `2cfe8027cf60c3a2ba3eaa6ef5f92d09b8b21f6b` 的 core Stage1 与正常基线见 §2.1–2.2，现已因 R20 反例归入 superseded。R20 复核时的临时冻结候选 `0aee73e96cbbfe3610d25b5d62083e26640efa97` 也因临时 clone 被环境清理而归档；其独立结论见 §2.4。当前持久普通 clone 临时提交是 `da2bd13979df88dd987596d7ed726ef99bb89651`，输入和新执行计划见 §2.5；完整变异与 M1 仍未完成。表中的 **WT31** 指未提交实现，不把任一临时 clone commit 当作产品修复提交。

依据为 [repair-plan.md](../../../repair-plan.md)、[repair-validation.md](../../../repair-validation.md)、[原 M1 审查](../../../review-m1-2026-09-28.md)，以及上游产品、架构、storage/protocol/workbook 合同。以下矩阵用于后续独立 M1 逐行复核；任务状态、测试数量及源码静态结论都不单独构成最终关闭证据。

## 1. 当前输入和日志边界

- [原 nextest.stdout.txt](gate-source-6baee-before-public-contract-oracles/nextest.stdout.txt) 记录增补公共合同 oracle 前的工作树全仓 run `7c61730e-2eb5-4ea5-9642-1c8113b86e80`：642 passed、0 skipped、2 slow，162.061 秒；固定引用保留目录，避免后续 645 项新门禁覆盖根目录同名文件。
- [mutants/superseded-before-implementation-review-fixes/unmutated-workspace.metadata.json](mutants/superseded-before-implementation-review-fixes/unmutated-workspace.metadata.json) 与 [原文](mutants/superseded-before-implementation-review-fixes/unmutated-workspace.stdout.txt) 记录普通 clone 的完整未变异 workspace 基线：run `533d2ac5-5816-4156-bdb4-d7a7a70c9fa4`，exit 0，642 passed、0 skipped、3 slow，204.193 秒；使用独立 baseline target。
- [原 gate-results.json](gate-source-6baee-before-public-contract-oracles/gate-results.json) 记录 `6baee533` 输入的 fmt/check/Clippy/Nextest/task/MSRV/deny/docs/specs/core-vocab/tests/skill argv、exit 和耗时。这些是已执行记录的复核，本 Reviewer 没有自行执行这些门禁；不将旧门禁自动套用新增测试或 opt1 profile。
- [dist-plan.metadata.json](dist-plan.metadata.json) 已补 `6baee533` 输入的 `dist plan --output-format=json`、exit 0、exec session `78036` 和 stdout/stderr SHA；初次复核逐字节复算 [dist-plan.json](dist-plan.json) 与 [stderr](dist-plan.stderr.txt)，均匹配元数据。此前缺 argv/exit 的证据缺口已关闭；后续记录以各自 candidate 字段为准。该运行仅验证本地发布形状，不发布、不证明远端或四平台资产。
- [README.md](README.md) 已明确 WIP、临时候选、Linux `not_run`、完整变异未完成及 T16/T17 后继边界。Owner 通知的 M1 table 合并属于任务白名单文档修正，不据此声称变异源码、测试或产品能力改变。
- `2cfe8027` 的旧 inventory 为 2571 项（core 707、runtime 1864）；`0aee73e9` 与 `da2bd139` 各自的 inventory 均为 2573 项（core 709、runtime 1864），其完整 mutation 记录逐项相同，但旧执行结果不复用。两阶段完整结果与逐项存活体处置尚未结束；旧候选、错误分片和 superseded 运行不得计入当前通过。

Linux 原生运行由用户明确豁免，全程为 `not_run`，不从 macOS 推导跨平台 PASS。exit70、SIGKILL 与 sync 故障分别记录，不能据此声称断电持久性通过。真实 Host/agent 内容质量、usage 与发布由 T16/T17 的独立门槛决定。

## 2. 最新历史文件与系统别名增量

**结论：task-local Spec 静态复核 PASS，无本增量残余阻断。** 完整变异与 M1 仍为 WIP。

`crates/sheltie-runtime/src/effects.rs` 的 `WriteFile`（约 655 行）改用 `open_managed_optional`。`fsx.rs::open_regular_at`（约 2285 行）先以 `SYMLINK_NOFOLLOW` stat 核类型/nlink，再以 `NOFOLLOW | NONBLOCK` 打开并核 dev/inode；悬空链接、FIFO、硬链和目录均不能被当作缺失历史文件。`open_optional` 仅把真正 NotFound 转为 None；缺失父目录的恢复仍明确停止。

历史补写走 `write_new_atomic_file` → `ManagedFs::write_new_atomic`（约 900 行）→ 私有 `write_atomic_unlocked`：同目录临时文件使用 `EXCL | NOFOLLOW`，写入并 sync 后，以 `RenameFlags::NOREPLACE` 落位，再 sync 父目录。已有目标或恢复窗口新出现的目标不能被覆盖。`classify_committed_path_error` 将对象身份/类型、消失和 AlreadyExists 归为 `STORE_CORRUPT`，其他 I/O 保留原 cause；统一 recovery 继续按真实提交归属封装 `EFFECT_PENDING` 和原 snapshot。

独立 oracle 入口为 CLI `committed_history_and_attempt_directory_type_changes_are_integrity_errors_with_original_snapshot`：symlink、dangling symlink、hardlink、FIFO、outputs-file 分条件运行，核 committed、cause、original、published、snapshot 原 bytes 和根外 sentinel bytes/mode。底层 `new_atomic_history_publish_never_replaces_an_existing_regular_or_dangling_leaf` 核普通目标原 bytes、悬空链接目标和临时文件收尾。原失败与通过原文见 [dangling-history-red.stdout.txt](dangling-history-red.stdout.txt)、[dangling-history-green.stdout.txt](dangling-history-green.stdout.txt)，逐字节 raw 另存同名 `.raw.gz`。

`fsx.rs::canonical_external_root` 同时处理精确 `/tmp`、`/var` 和其后代，仅在核验系统链接确实指向 `/private/tmp`、`/private/var` 后规范化外部访问。RequestIntent 保持用户参数的词法规范化路径，不暗改旧 hash。真实同文件 alias/private 正例及任意用户 symlink 反例见 `exact_system_tmp_parent_alias_reads_the_same_at_file_and_rejects_user_symlinks`；历史独立探针见 [exact-alias-independent-probe.json](exact-alias-independent-probe.json)。

### 2.1 公共 core 合同 oracle 增量

**结论：3 项新增测试 task-local Spec 静态复核 PASS。** 唯一 fixture finding 已修正：`persisted_contracts.rs` 的合法 HostRequire.source 从非 URL 文本改为 `https://example.com/`（20 字节）加 492 个 `x`，恰 512 字节；source 负例从该值追加 1 个 ASCII 字节，恰 513 字节。version 的合法/反例分别为 32/33 字节。期望不依赖生产上限常量；Serde roundtrip 比较完整手写 JSON，而不只看 is_ok。

`validated_metadata_preserves_declared_values_for_public_consumers` 以手写 manifest/Flow 检查 version、name、description、flows/RelPath、requires、NodeDef.title 与 FlowDef.edges；期望不是调用另一个生产 getter 算出。`persisted_facts_reject_single_field_contradictions_and_accept_real_lifecycle_states` 经既有公共 Fixture 调用 core.decide 建立真实状态转换，再对每个合法基线独立 clone 和单字段改动；同 occurrence 不同 node、同 node 旧 occurrence 的审批正例能检验错误的跨审批匹配。没有改 testkit 或开放私有 API。

[public-contract-development-4.stdout.txt](public-contract-development-4.stdout.txt) 记录 run `4332d69c-94f4-4442-9ddd-725a9d7509a5`：3 项目标测试 passed、0 skipped。这是公共 core API 合同证据，不替代 Store/CLI 调用链或完整 workspace。Owner 后来固定普通 opt clone `2cfe8027cf60c3a2ba3eaa6ef5f92d09b8b21f6b` 并完成当时的完整 645 项基线（§2.2）；新增测试和 profile 属于不同输入，旧默认 profile 的部分结果没有复用为该候选通过。`2cfe8027` 又因 §2.3 反例成为 superseded，不复用其 2571 项变异结果给当前候选。

### 2.2 新候选 core Stage1 完整执行的独立复核

本次只读复核确认 `2cfe8027cf60c3a2ba3eaa6ef5f92d09b8b21f6b` 的 core Stage1 已完整执行；结论限本阶段，完整两阶段/runtime/M1 仍 WIP。profile 为 opt-level=1、debug=0、debug-assertions=true、overflow-checks=true，8 jobs、600 秒 timeout；四片 metadata 的 candidate/profile 均一致。

[旧 inventory.json](mutants/superseded-before-implementation-review-fixes/superseded-before-counter-integrity/inventory.json) 中 core 为 707 项。Reviewer 逐项比较当时四片 `raw/mutants.json` 的完整对象（含 diff）及 `raw/outcomes.json` 的 Mutant scenario 字段：listed=processed=707、各自 unique=707，missing/extra/duplicate 均为 0。四片分布及实际结果如下；shard 1 的 exit 2 来自未捕获变体，不是工具失败。

| 分片 | Listed / processed | Caught | Unviable | Missed | Timeout | 原始入口 |
| --- | --- | --- | --- | --- | --- | --- |
| 0/4 | 177 / 177 | 149 | 28 | 0 | 0 | [metadata](mutants/superseded-before-implementation-review-fixes/superseded-before-counter-integrity/stage1-sheltie-core-0-of-4/metadata.json)、[outcomes](mutants/superseded-before-implementation-review-fixes/superseded-before-counter-integrity/stage1-sheltie-core-0-of-4/raw/outcomes.json) |
| 1/4 | 177 / 177 | 125 | 49 | 3 | 0 | [metadata](mutants/superseded-before-implementation-review-fixes/superseded-before-counter-integrity/stage1-sheltie-core-1-of-4/metadata.json)、[outcomes](mutants/superseded-before-implementation-review-fixes/superseded-before-counter-integrity/stage1-sheltie-core-1-of-4/raw/outcomes.json) |
| 2/4 | 177 / 177 | 174 | 3 | 0 | 0 | [metadata](mutants/superseded-before-implementation-review-fixes/superseded-before-counter-integrity/stage1-sheltie-core-2-of-4/metadata.json)、[outcomes](mutants/superseded-before-implementation-review-fixes/superseded-before-counter-integrity/stage1-sheltie-core-2-of-4/raw/outcomes.json) |
| 3/4 | 176 / 176 | 158 | 18 | 0 | 0 | [metadata](mutants/superseded-before-implementation-review-fixes/superseded-before-counter-integrity/stage1-sheltie-core-3-of-4/metadata.json)、[outcomes](mutants/superseded-before-implementation-review-fixes/superseded-before-counter-integrity/stage1-sheltie-core-3-of-4/raw/outcomes.json) |
| 合计 | 707 / 707 | 606 | 98 | 3 | 0 | 只计当前 opt1 profile，不复用 superseded 默认 profile |

四个 baseline 的 Build/Test 均为 Success。Reviewer 读取全部 606 个 Caught 对应原始日志：均为 Build Success → Test Failure(100)，都有具体测试 FAIL 标记，未发现工具失败误计为 caught。98 个 Unviable 分列，不能当成测试 PASS 或 caught。旧原始日志随分片保留在上述 superseded 目录；不把它们计入 `0aee73e9` 候选。

对旧 47 个 missed 与新结果按完整 mutant name 比对，有 44 个现在 caught。逐一读取这 44 个原日志，确认捕获者正是新增公共合同 oracle，归属为 metadata 14、state 28、HostRequire 2；每条都有成功 build、实际测试失败及对应断言，不从正常 3/3 通过推算捕获结果。

| 公共 oracle | 实际捕获数 | 独立断言及原始日志定位示例 |
| --- | --- | --- |
| `validated_metadata_preserves_declared_values_for_public_consumers` | 14 | NodeDef.title 空 getter：`persisted_contracts.rs:54` 比较 left=`""`、right=`"First node"` 失败；`stage1-sheltie-core-0-of-4/raw/log/crates__sheltie-core__src__flow__def.rs_line_181_col_9.log` |
| `persisted_facts_reject_single_field_contradictions_and_accept_real_lifecycle_states` | 28 | validate_persisted→Ok(())：`persisted_contracts.rs:90` 的单字段 name/work_id 矛盾反例被错误接受，测试以 `name does not match work id` 失败；`stage1-sheltie-core-3-of-4/raw/log/crates__sheltie-core__src__work__state.rs_line_304_col_9.log` |
| `host_requirement_snapshots_accept_exact_byte_limits_and_reject_one_extra_byte` | 2 | version `>`→`>=`：合法 32 字节 JSON 在 `persisted_contracts.rs:68` 被拒为 `requires.version 超过 32 字节`，正例 unwrap 失败；`stage1-sheltie-core-3-of-4/raw/log/crates__sheltie-core__src__workbook__manifest.rs_line_70_col_46_002.log` |

只剩 3 个 Stage1 missed：`parse_input_source` 的 `||`→`&&`，以及 `decide_approve:574` 的 `+=`→`-=`/`*=`。parse 的合同语义等价边界见 §7，仍须保留 workspace 结果；两个批准计数变体有已存在的真实 CLI Gate→NoLegalEdge oracle，最终捕获与否以 Stage2 实际运行记录为准，本轮不代填。

另已读取 [unmutated-workspace-opt645.metadata.json](mutants/superseded-before-implementation-review-fixes/unmutated-workspace-opt645.metadata.json) 与 [原文](mutants/superseded-before-implementation-review-fixes/unmutated-workspace-opt645.stdout.txt)：`2cfe8027` 的完整 opt1 workspace baseline run `cc20c27b-b4a2-410c-a824-e8cdd8379c17`，exit 0，645 passed、0 skipped、2 slow，136.055 秒。这只证明当时未变异的 opt1 workspace 基线成功，不代替 `0aee73e9` 的完整变异、默认 profile 公共门禁或 runtime 清单。

### 2.3 旧候选发现：不可能的累计计数导致真实批准命令 panic

**对 `2cfe8027` 的结论：需修改，阻断以该旧候选关闭 N03/N07 相关可信装入与事实计数义务。** 后续修复复核见 §2.4。Owner 指定独立 Reviewer 检查 `blocked_count` 装入边界。本轮在 `/private/tmp/sheltie-m1-spec-blocked-max-m8ph9ipp/` 创建独立控制/反例 Home，通过真实 CLI add/start/begin/submit 建立 Gate→NoLegalEdge；只在反例的 `works.state_json` 中把 `blocked_count` 从 1 改为 `u32::MAX`，其他持久业务字段保持原样。未改仓库源码或 plan。

控制例的 gate approve 成功，状态为 Blocked(NoLegalEdge)，随后 stats 的 blocked_count=2。反例 status/stats 都 exit 0，stats 把 4294967295 当事实呈现；新 request-id 的 gate approve 在 `decide.rs:574` 执行 `blocked_count += 1` 时 panic，exit 101、stdout 为空，stderr 为 `attempt to add with overflow`。works 的 revision/state 与 requests/audit 在此次调用前后完全相同，没有结构化 STORE_CORRUPT 响应。

先固定复制现成共享 target binary，SHA 为 `c3eb2a8ac3ca1450bb9bfaacea392a2babc6b046158ff1736a83e5ab5a81cf2e`；再从普通 `2cfe8027` clone 的 baseline `target/debug/sheltie` 固定复制并重复新 request-id，SHA 为 `72b6ecee9a6fad9c93ee9b8f5e220fe2fc4929278c427e04382a30cce97385d7`，同样 exit 101、panic 且数据库业务行不变。完整 argv/stdout/stderr/JSON、修改前后 SQL 和 binary 身份保存在临时 `results.json`，第二次结果在 `current_candidate_repeat`；Owner 需将这些原始证据纳入最终 package，不能仅复制本段结论。

源码搜索确认：`WorkState::validate_persisted` 不检查累计计数，runtime 的 Store decode/commit 仅调用这个入口，`load::validate_work_paths` 只核路径/绑定，没有其他计数上界。`GF-29` 与 protocol §work stats 要求受阻是状态转换发生时记录的累计事实；本例只有两次成功 Attempt、一次尚未批准的 Gate，MAX 不可能来自该历史。类型可反序列化不等于该字段是合法系统事实，有限 Flow 上限也排除了正常执行积累到 MAX。

本 finding 不要求认证任意协调改写的数据库，不引入 audit 第二事实源，也不要求还原每次历史状态。已按现有 WorkState Attempt/Approval 事实的安全计数界及状态转换的 checked 增量修复，独立静态复核见 §2.4。旧 645 正常回归和 core Stage1 成果不能否定这个单条件真实反例，也不能复用为修复后候选的变异结果；完整 M1 继续 WIP。

### 2.4 R20 当前修复的独立 task-local 复核

**结论：`0aee73e96cbbfe3610d25b5d62083e26640efa97` 的 R20 Spec 静态复核 PASS，完整 mutants/superseded-before-implementation-review-fixes/M1 仍 WIP。** `WorkState::validate_persisted` 在原有身份、Attempt、running、Approval 与 Gate 一致性检查之后，才核必要计数界：`approvals.len() + usize::from(matches!(status, WorkStatus::Blocked(_))) ≤ blocked_count ≤ attempts.len() + approvals.len()`。每个 Approval 必须先经历一次 Gate 阻断；当前 Blocked 又是独立发生的一次阻断。一次 submit/fail 阻断各归属一个 Attempt，gate approve 后若进入 NoLegalEdge，新增阻断归属这次 Approval。故这些是合法历史必满足的安全界，不是重新推算准确计数。取消/推进不减少计数；Gate→NoLegalEdge、同 node 回访、不同 node 同 occurrence 都落在界内。

`decide.rs` 将 submit、fail、approve 三处增量统一改为私有 `checked_add` helper，在纯 core API 中对不可信 MAX 返回现有 `InvalidRequest` 而非 panic，且输入 `WorkState` 不变。真实 runtime 的 Store `decode_row` 与 `commit` 仍把该持久状态校验失败映射为 `STORE_CORRUPT`，Work status/stats/list/approve 都经过该装入链。真 CLI 的 MAX、零、上界加一三个单条件反例均要求结构化拒绝、无新 request/audit、数据库和 Work 文件 bytes 不变。

旧「待批准 Gate 追加当前 Approval」反例如今也违反新低界，不能单靠它证明原 Gate 已批准保护仍生效。新用例从合法 `two_gates` 的第二节点 Blocked(Gate) 出发，只把第一节点 Approval.node 改成当前第二节点；计数仍在新界内，且当前节点已有成功 Attempt，必须由原 Gate 一致性检查拒绝。源代码顺序也已把新计数界移到原检查之后，保留原诊断优先级。

[blocked-count-green.stdout.txt](blocked-count-green.stdout.txt) 记录 R20 CLI 定向 run `97639eda`：3 passed；[counter-core-green.stdout.txt](counter-core-green.stdout.txt) 记录 core 定向 run `375922ef`：1 passed。新增单字段 Gate 反例在 run `1062e606` 的定向 2/2 通过：此轮结果见 [首轮诊断顺序记录](counter-diagnostic-order-failure.md)，原定向 stdout 未保存；最终 647 项完整原文包含该 core 测试。Reviewer 本轮未运行 Cargo。反实现把 checked helper 改回 `+=` 时，[原始失败](unchecked-counter-increment.stdout.txt) 的 core 单测在 `decide.rs:73` overflow panic、Nextest exit 100；[元数据](unchecked-counter-increment.metadata.json) 记录逐字节恢复 SHA `c0d1d5d305dba0cd0d9a66b756628108ef99dfcb4b3997fb1e3040237490fb8f`，本轮复算当前 `decide.rs` SHA 相同。失败原文另有同名 `.raw.gz`，不把反实现失败算作正常候选通过。

`0aee73e9` 的输入身份保留在 [归档 candidate](mutants/superseded-before-implementation-review-fixes/superseded-ephemeral-input/current-candidate.json) 与 [32 个变更文件 SHA](mutants/superseded-before-implementation-review-fixes/superseded-ephemeral-input/copied-candidate.json)；默认 profile 全仓 [原文](nextest.stdout.txt) run `6e03fa98-0535-4d68-bc1f-f4412a609ac8`：647 passed、0 skipped；独立 opt1 普通 clone [元数据](mutants/superseded-before-implementation-review-fixes/unmutated-workspace-opt647.metadata.json) 与 [原文](mutants/superseded-before-implementation-review-fixes/unmutated-workspace-opt647.stdout.txt) run `36cf2302-efc8-48f8-a1a4-1d5eb1770eb9`：647 passed、0 skipped。旧 `0aee73e9` 的 core 709 项及跨 workspace 复验已存 [归档状态](mutants/superseded-before-implementation-review-fixes/superseded-ephemeral-input/status.json)，runtime 第一片在环境清理前未完成；连同更早 `2cfe8027` 的旧运行，都不复用给新候选。该旧输入的静态 R20 结论仍可按当前代码/测试实际字节核查，不能由旧 mutation 结果推断当前运行通过。

### 2.5 持久 clone 新输入及 pipeline 清单检查

环境清理旧 `/private/tmp` clone 后，Owner 在 Git 忽略的 `target/t31-validation/source` 重建普通 clone，临时提交 `da2bd13979df88dd987596d7ed726ef99bb89651`。本轮只读检查该 clone 的 HEAD 与 [当前候选身份](mutants/superseded-before-implementation-review-fixes/current-candidate.json) 一致、Git 工作树干净；[candidate-input.txt](candidate-input.txt) 与 [mutation 闭包](mutants/superseded-before-implementation-review-fixes/closure.md) 记录 147 个源/配置/fixture/脚本 SHA 和 34 个改动文件 SHA。此前 `0aee73e9` 归档的 32 个改动文件中，当前仓库 29 个源码/测试等文件 SHA 一致，另 3 个差异仅在 `plan.md`、`repair-design.md`、`repair-validation.md`；仍须以 `da2bd139` 作为当前独立执行输入，而非复用旧 mutant 输出。

当前 [inventory.json](mutants/superseded-before-implementation-review-fixes/inventory.json) 为 2573 项、名字唯一：core 709、runtime 1864。与 `0aee73e9` 归档 inventory 的全项 mutation 记录（含 diff）相同，但仅清单相同不表示运行可复用。[新普通 clone baseline 元数据](mutants/superseded-before-implementation-review-fixes/unmutated-workspace-durable.metadata.json) 与 [原文](mutants/superseded-before-implementation-review-fixes/unmutated-workspace-durable.stdout.txt) 记录相同 opt1 profile、all-features workspace run `32ac9a76-3eeb-4e8e-a938-d7ed9b370680`：647 passed、0 skipped、exit 0。它是未变异基线，不是完整变异结论；旧 `0aee73e9` 647 运行仍以旧候选身份保留。

Reviewer 只读审阅 [pipeline.py](mutants/superseded-before-implementation-review-fixes/pipeline.py) 的集合逻辑：core 依次执行 0/4–3/4，runtime 依次执行 0/24–23/24；每包 Stage1 汇总后要求处理数、唯一数和名字集合精确等于该包 inventory。候选由该包所有 `MissedMutant` 与 `Timeout` 组成；core 一组、runtime 按 `candidates[index::8]` 分八组，无采样或遗漏。每组用逐字符转义并首尾锚定的 mutant name 正则选取完整 workspace、all-features 测试；Stage2 汇总后再次要求处理数、唯一数和名字集合精确等于候选。每片至少有成功 baseline，且 processed 数等于该片 listed；中断分片不被无声重跑，已有结果的候选 ID、argv、数量均需匹配。`scripts/mutants.sh` 设置相对 `CARGO_TARGET_DIR=target`，每个变异副本使用自身 binary。当前新完整分片仍在运行，本段仅确认选择算法，未据脚本打印值宣称通过。

一个自动停止条件仍需最终审计：`pipeline.py::run` 在 fresh/reuse 分支只核 baseline 与数量，没有限制 `metadata.exit` 为可接受的 0 或 2；若工具在写出完整 outcomes 后以异常退出码结束，脚本仍可能继续。最终证据须逐片核真实 exit、baseline Build/Test 及 Caught 的测试失败来源；若实际出现异常退出码，不能把 `all stages executed` 当作通过。Reviewer 未修改运行中的脚本或其他实施者文件。

## 3. 证据缩写

下表的路径均相对本 package 的 `evidence/`；所有测试名保留源码原名，可用 `rg` 定位唯一函数。R/O/N 行中的 F/B 是历史映射入口，不代表 `da2bd139` 同闭包执行。当前新候选的未变异基线是 B3；完整变异和最终公共门禁仍需各自原文，不得用 F/B/B3 任一项直接关闭矩阵行。

| 缩写 | 目录或原文 | 用途 |
| --- | --- | --- |
| F | `repairs/t31/gate-source-6baee-before-public-contract-oracles/nextest.stdout.txt` | `6baee533` 输入的 642 项工作树全仓原文；输入见该保留目录 `candidate-input.txt`，不指向未来根目录的 645 项运行 |
| B | `repairs/t31/mutants/superseded-before-implementation-review-fixes/unmutated-workspace.*` | 旧 `6baee533` 普通 clone 的 642 项完整未变异 workspace 基线，run `533d2ac5`；不改写成新 profile 通过 |
| B2 | `repairs/t31/mutants/superseded-before-implementation-review-fixes/unmutated-workspace-opt645.*` | `2cfe8027` 的 645 项 opt1 完整未变异 workspace 基线，run `cc20c27b`；本次增量记录在 §2.2 |
| B3 | `repairs/t31/mutants/superseded-before-implementation-review-fixes/unmutated-workspace-durable.*` | 当前 `da2bd139` 的 647 项 opt1 完整未变异 workspace 基线，run `32ac9a76`；仍不是 mutant 或公共门禁结论 |
| P19 | `t19-managed-fs-2026-09-28/` 的 `task.*`、`nextest.*` | T19 历史修复与原语反例 |
| P20 | `t20-trusted-load-2026-09-28/` 的 `task.*`、`nextest.*` | T20 持久路径、原格式和效果闭包 |
| P21–P30 | `repairs/tNN/task.stdout.txt` 与最终 `nextest.stdout.txt` 或 `workspace-nextest.stdout.txt` | 对应修复任务原始运行 |
| Hnn | `tNN/nextest.txt` 与 README | 原 T02–T15 修复来源和历史证据；不代替当前 F/B |
| WT31 | 本报告 §1 固定的未提交实现/临时测试候选 | 待产品提交，不写成已提交修复 |

主要源码消费者为 runtime `service.rs`、`workbook_repo.rs`、`recovery.rs`、`effects.rs`、`fsx.rs`、`session.rs`、`store/{read,commit}.rs`，以及 CLI `commands/{work,workbook,self_cmd}.rs`。测试分别在 runtime `tests/{service,workbook_txn,workbook_repo,workbook_digest,write_session,selfmgmt,schema2_replay}.rs`、`src/store/tests.rs`，CLI `tests/{reliability_crash,replay,work,attempt,scenario_spec_dev,scenario_article_review,skill_delivery,release_governance,os_process}.rs`；core 事实视图在 `src/work/render.rs`。

## 4. R01–R19 准备矩阵

| R | 修复提交 | 真实 caller、独立 oracle、测试 | raw / 未完成边界 |
| --- | --- | --- | --- |
| R01 路径归属 | `ee78118`、`22942ee`、`5d2d051` | `WorkService::load/run_command`、`CheckedEffects`；单改 state 根、后续效果路径、ArtifactRef，根外 bytes/mode 不变；`tampered_work_dir_is_rejected_before_reading_outside_paths`、`invalid_later_effect_is_rejected_before_any_effect_runs`、`tampered_artifact_path_is_rejected_before_observing_external_file` | P19/P20/P24、F/B；最终产品 hash 待固定 |
| R02 seal | `ee78118`、`fd21a63`、`1d92bcd` | 真 submit COMMIT→seal rendezvous；原 ref 独立 SHA/bytes、SQLite revision/published、外部 sentinel mode；`real_submit_path_swap_after_commit_seals_original_and_preserves_external_target`、`real_submit_same_inode_byte_change_after_commit_stops_before_chmod`、`real_submit_hardlink_added_after_commit_stops_before_chmod` | P22/P25、F/B；Linux not_run |
| R03 self 文件链 | `ee78118`、`7e9178e` | `SelfManager::install/update/rollback/uninstall`；只改单一 tmp/bin 父链接、归档链接，旧 binary/prev、外部 bytes/mode；`install_rejects_tmp_parent_symlink_and_preserves_external_sentinel`、`rollback_rejects_bin_parent_symlink_and_preserves_external_binaries` | P23、F/B；远端资产 T17 not_run |
| R04 purge | `69710aa`、`7e9178e`、WT31 | 真 self uninstall --purge 和等待者；root/.lock dev/ino 保留、Store 最后删、部分失败；`purge_removes_frozen_work_and_binary_but_preserves_same_root_lock`、`purge_waiters_reach_the_failed_try_lock_before_initialization_or_old_work_rejection` | P23、F/B、T31 task raw；Linux not_run |
| R05 @file 重放 | `5d2d051` | CLI start/submit/fail；成功后删除源，同 rid 返回原响应，不新增序号；`start_request_replay_does_not_require_the_original_input_file`、`submit_request_replay_does_not_reread_a_deleted_summary_file`、`fail_request_replay_does_not_reread_a_deleted_reason_file` | P24、F/B；系统别名补强在 WT31 |
| R06 历史目标 | `5d2d051` | 所有 Work 写入口先取历史完整 WorkId；新增同前缀 Work、不匹配前缀，目标及快照不漂移；`historical_work_id_resolves_before_a_now_ambiguous_prefix`、`work_prefix_matching_is_case_sensitive_and_replays_with_the_same_selector` | P24、F/B |
| R07 恢复错误协议 | `1d92bcd`、WT31 | `recovery::before_write/finish_request` 和 CLI；自己 true/原响应、旧 A 阻 B false、mark 零行、坏 BLOB、历史父缺失；`post_commit_card_failure_returns_committed_response`、`old_effect_blocks_new_request_with_distinct_identities`、`mark_published_zero_rows_returns_committed_recovery_error`；新历史类型真实 CLI 用例 | P25、F/B；T31 `r07-*`、`dangling-history-*` 保存红绿和独立探针 |
| R08 Workbook 重放 | `1d92bcd`、`3dd224d` | 真 add/remove 同 rid；未发布先恢复，完成历史不绑定新生命周期；`workbook_replay_finishes_unpublished_effects`、`completed_old_remove_and_add_replays_preserve_new_lifecycle_bytes_and_row` | P25/P27、F/B |
| R09 pending 读 | `4526b7e` | Repo list/show/verify、Work status/start preflight；pending/final、owner、清理后 completed、真实 rename/mark 交错；`committed_pending_workbook_is_readable_without_recovery_and_is_marked_pending`、`real_work_status_retries_after_start_publish_renames_post_location`、`workbook_reader_accepts_publication_and_cleanup_after_its_reference_index` | P28、F/B；只读不恢复，SQLite 控制例外单列 |
| R10 跨入口卡刷新 | `1d92bcd` | Workbook add/remove 恢复旧 Work Start，而最新 begin 已提交；状态卡等于捕获的最新 bytes，历史回复不变；`workbook_write_recovers_work_status_card` | P25、F/B |
| R11 删除证明 | `3dd224d`、`4526b7e`、WT31 | remove 同对象移入/删、marker 同 rid/internal-id；未知结果停止，空替代根保留；`remove_marker_cannot_be_borrowed_from_another_internal_id`、`marker_replaced_after_validation_cannot_prove_deletion`、`remove_real_windows_preserve_unknown_results_and_do_not_touch_a_new_lifecycle` | P27/P28、F/B；kill 不等于断电 |
| R12 发布闭包 | `22942ee`、`5a9d430` | Start/Add pending/final：owner、manifest、digest_root、每个 start ref 实际 bytes/len；`publication_checks_each_start_input_reference_and_owner_field`、`final_only_publication_recovery_checks_start_input_bytes`、`workbook_publication_recovery_checks_owner_and_manifest_identity` | P20/P26、F/B |
| R13 cleanup | `4526b7e`、WT31 | 全 requests effects 索引；只清合法孤儿/完成空元数据，非法路径/坏 JSON 保留；`cleanup_index_rejects_a_decodable_effect_path_that_points_into_an_orphan`、`malformed_request_effects_stop_cleanup_before_any_unreferenced_object_is_removed`；same-rid 未提交残留清理失败不登记新请求 | P28、F/B、T31 `missing-same-rid-preparation.*` |
| R14 树装入/摘要 | `ee78118`、`543f9d2` | add source、installed load/verify、Start frozen copy、Work load、publish verify 共用受限快照；独立 Python framing 向量、碰撞对、exact/+1、root/parent/leaf 链接与 FIFO；`digest_matches_independent_vectors_on_disk`、`tree_reader_rejects_symlinked_parent_directory`、`total_limit_rejects_before_staging_or_registering_source_files` | P21、F/B；不声称性能量化 |
| R15 stats 快照 | `a4f1968` | 真 CLI stats 装入后暂停，writer begin 已完成再释放；手写旧 stats/旧 next 同一 revision；`stats_and_next_keep_one_snapshot_when_a_writer_begins_after_reader_load` | P29、F/B；`red.*` 和 `stop-route-mutation.*` |
| R16 replan | `b926789` | 真 CLI+临时 Git+只读绑定 JSON 的冷 worker；原字节镜像、累计前缀、原基线、Git 祖先、审批/原始 gate；`fresh_replanner_recovers_verified_task_and_original_baseline_only_from_bound_files`、`fresh_replanner_rejects_deep_dropped_rows_failed_reports_denied_approval_and_cycles` | P30、F/B；模拟 worker，不替代 T16 agent 质量 |
| R17 全验证闭包 | `b926789` 迁移＋WT31 | barrier/真实子进程窗口，exit70 与 signal9 分别核；反实现摘锁、删父 sync、删 same-rid preparation 必须失败；完整 core/runtime inventory 两阶段 | T31 task、负控制 raw、`mutants/superseded-before-implementation-review-fixes/`；**WIP：完整结果/逐存活处置未完成** |
| R18 必需 sync | `ee78118`、`5a9d430`、`3dd224d`、WT31 | pending/final-only 两格重复同一 fault；source/target parent sync 失败不 mark，重试 inode 不变；`publication_sync_failures_leave_effect_pending_and_retry_same_object`、`start_replay_sync_failure_preserves_own_commit_snapshot`、`moved_delete_recovery_resyncs_both_rename_parents_before_deleting_payload` | P26/P27、F/B、`missing-delete-parent-sync.*`；不声称掉电证明 |
| R19 锁内建库 | `5d2d051`、`edb86f1`、WT31 | WriteSession 持同根锁后 RW Store；旧 schema 无新锁、main/WAL 不变；真 add↔install 初始化，旧 Work 等 purge 后不 CREATE；`real_install_waits_for_a_concurrent_workbook_add_store_initializer`、`real_workbook_add_waits_for_a_concurrent_install_store_initializer`、`old_work_waiting_behind_purge_does_not_recreate_the_removed_store` | P24、`repairs/t24-followup/`、F/B |

## 5. O01–O13 准备矩阵

| O | 修复/保留提交 | 真实 caller、独立 oracle、测试 | raw / 边界 |
| --- | --- | --- | --- |
| O01 根隔离 | `ee78118`、`22942ee`、`543f9d2`、`fd21a63`、`7e9178e` | add/start/submit/self 全链；单路径/父链接/叶换绑，外部 bytes/mode；R01–R03/R14 上述测试 | P19–P24、F/B；同账户搬对象边界按合同 |
| O02 跨 Work 重放 | `bcfa0e6`、`5d2d051` | 同 rid cancel A 后 cancel B，B 保持 Active；`cross_work_request_id_is_request_conflict_and_target_untouched` | H07/P24、F/B |
| O03 Workbook 身份 | `d38b6e1`、`429cb1d`、`22942ee`、`543f9d2`、`4526b7e` | Repo load/verify/remove：行/manifest/digest/publisher；`load_rejects_tampered_registered_digest`、`completed_workbook_final_requires_its_current_successful_publisher` | H05/H09、P20/P21/P28、F/B |
| O04 历史回复/观察 | `bcfa0e6`、`5d2d051`、`1d92bcd` | submit→cancel→旧 submit，回复逐字段、旧源消失；`old_submit_replay_after_cancel_returns_original_reply`、三个 @file replay 用例 | H07/P24/P25、F/B |
| O05 历史 engine.stats | `bcfa0e6`、`1d92bcd`、WT31 | begin→推进→删旧 stats→重放 begin；登记 content 原 bytes、当前卡不回退；`begin_replay_regenerates_missing_stats_json`、`replay_does_not_rewrite_status_card_to_old_revision` | H07/P25、F/B；类型/悬空链接补强 raw 在 T31 |
| O06 clean home | `d38b6e1`、`2320020`、`5d2d051`、`edb86f1` | 真 install/add、two-step、缺失只读 Home；`self_install_on_new_home_creates_root_store_and_bin`、`clean_home_install_then_update_two_step` | H05/H15/P24/follow-up、F/B |
| O07 摘要碰撞 | `429cb1d`、`543f9d2` | 显式 framing 独立 SHA、碰撞对、插入顺序；`digest_matches_independent_vectors_on_disk`、`framing_collision_pair_now_yields_two_different_digests` | H09/P21、F/B |
| O08 Workbook/self 幂等 | `766dbe2`、`2320020`、`1d92bcd`、`3dd224d` | add/remove 历史快照与新生命周期；self rid 拒绝；`add_and_remove_replay_return_original_snapshots`、`request_id_rejected_for_readonly_and_self_commands` | H08/H15/P25/P27、F/B |
| O09 每任务 Git 范围 | `c1a530e`、`b926789` | 真临时 Git diff --name-only 对手写白名单；`two_task_loops_scope_each_task_diff_to_its_own_files`、`out_of_whitelist_file_in_second_task_is_flagged` | H12/P30、F/B |
| O10 人工意见/继续 | `c1a530e`、`b926789` | CLI 输入路径、审批摘要、escalate 回程、明确更正；`conditional_approval_binds_decision_into_scaffold_implement_verify`、`fresh_replanner_accepts_explicit_human_approval_correction_and_rejects_stale_version` | H12/P30、F/B；T16 真人/agent 实测 not_run |
| O11 skill 自包含 | `4960e9f` | 真打包/复制后删源码；文件集合与链接、missing/stale/escape 反例；`installed_delivery_stays_self_contained_after_source_tree_is_removed`、`check_skill_fails_when_delivery_link_escapes` | H13、F/B；真实 Host 发现/执行 T16 not_run |
| O12 输出命名空间 | `6b40abd`、`bcfa0e6`、`fd21a63` | 真 worker outputs brief/stats 与 engine 分离；祖先/大小写别名拒绝；`worker_outputs_named_brief_and_stats_write_and_submit_via_real_chain`、`add_rejects_ancestor_and_folded_alias_output_paths_at_cli` | H03/H07/P22、F/B |
| O13 完整事实视图 | `85ada77`、`bcfa0e6`、`4526b7e`、`a4f1968` | 文本/JSON reason、完整 ref、同形 next、手写非零状态；`text_and_json_agree_on_hand_written_state`、`status_json_carries_reason_and_full_artifact_refs`；真 pending/stats caller | H06/H07/P28/P29、F/B |

## 6. N01–N14 准备矩阵

| N | 修复/保留提交 | 真实 caller、独立 oracle、测试 | raw / 边界 |
| --- | --- | --- | --- |
| N01 start 预检 | `df0e120`、`5d2d051` | 真 CLI 缺/多键、名、Workbook/Flow、坏 @file；Home 目录快照、SQL rows/sequence 无变化；`start_deterministic_rejections_leave_home_unchanged_and_do_not_burn_seq`、`failed_start_on_new_home_creates_nothing` | H02/P24、F/B；分配后合法空号不回收 |
| N02 Workbook 生命周期 | `d38b6e1`、`766dbe2`、`1d92bcd`、`3dd224d`、`4526b7e` | add/remove 活动引用、pending 读/恢复、未知删除、新生命周期；`remove_rejects_active_reference_and_rolls_back_row`、`completed_old_remove_and_add_replays_preserve_new_lifecycle_bytes_and_row`，R08–R13 反例 | H05/H08/P25–P28、F/B |
| N03 schema/持久验证 | `63d4d48`、`22942ee`、`5d2d051`、`edb86f1` | Store 真 load/list/commit；schema1 main/WAL 原 bytes、行 identity/status/revision 单改、合法 Running/Failed；`schema1_store_rejected_without_touching_file`、`load_rejects_row_identity_status_and_revision_mismatch`、`store_accepts_real_running_and_failed_attempt_states` | H14/P20/P24/follow-up、F/B；state mutant 的完整处置归 R17 |
| N04 OS 主体/授权 | `d38b6e1`、`a5b2a05`、`b926789` 迁移 | 真子进程伪造 USER 与 id -un 独立期望；`audit_principal_ignores_spoofed_user_env`、`principal_matches_id_un_oracle` | H05/H10/P30、F/B；不声称独立真人认证，T16 not_run |
| N05 modify-path/JSON | `2320020`、`7e9178e`、`b926789` 迁移 | 真 CLI flag 拒绝、宿主 rc 原 bytes、单 JSON stdout；`install_modify_path_flag_is_rejected`、`self_install_json_stdout_is_single_document` | H15/P23/P30、F/B |
| N06 固定 release/tag | `2320020`、`7e9178e` | 本地 release fixture，固定 tag、latest 毒字节、manifest 错配、rollback 原 bytes；`update_pinned_version_installs_only_that_tag`、`update_rejects_manifest_version_not_matching_tag`、`update_pinned_version_then_rollback_restores_previous` | H15/P23、F/B；远端/四平台 T17 not_run |
| N07 stats 累计/edge | `85ada77`、`bcfa0e6`、`a4f1968`；WT31 补 oracle | 手写来源 node/edge、NoLegalEdge→cancel，Gate→NoLegalEdge 应 1→2；`entered_via_distinguishes_edges_and_entry`、`blocked_count_survives_cancel_after_no_legal_edge`、`gate_approval_without_a_remaining_edge_adds_a_second_blocked_event` | H06/P29、F/B；approve 变体待 stage2 结果 |
| N08 最终审查基线 | `c1a530e`、`b926789` | CLI 冷 worker、真实 Git、递归来源/累计前缀；`fresh_replanner_recovers_verified_task_and_original_baseline_only_from_bound_files`、`fresh_replanner_stops_on_reset_baseline_missing_fields_or_dropped_verified_task` | H12/P30、F/B；模型履行 T16 not_run |
| N09 article-review 意见 | `b3f138b` | 真 back 回环：首次空、后绑前轮 verdict、重复回环取最新 occurrence；`first_draft_marks_review_input_absent_without_body`、`back_to_draft_binds_review_verdict_path_with_source_occurrence` | H11、F/B；内容好坏由 worker，T16 not_run |
| N10 冻结根/元数据 | `d38b6e1`、`543f9d2`、`7e9178e` | 目录0555/文件0444、DS_Store 拒绝、改 bytes digest 停止、删仓库仍从冻结副本读；`readonly_bits_reduce_accidents_but_digest_is_the_guard`、`ds_store_rejected_by_name_at_add`、`work_readable_after_workbook_removed` | H05/P21/P23、F/B；不是同账户防篡改 |
| N11 治理历史/版本 | `2320020` | 真临时浅/完整 Git、active/RC、越目标、缺 changelog/tag；`check_specs_shallow_clone_reports_missing_history`、`check_specs_accepts_active_target_without_tag`、`check_specs_rejects_version_outside_active_target` | H15、F/B；远端 CI not_run |
| N12 同 SHA 质量/MSRV | `2320020`、WT31 门禁 | 工作流删 needs/if 单条件反例、MSRV locked 实编译、dist plan 形状；`release_workflow_gates_announce_on_quality_job`、`release_workflow_quality_failure_blocks_announce` | H15、F/B、T31 `msrv.stdout.txt`、`dist-plan.metadata.json`/JSON；dist argv/exit 缺口已关闭，T17 not_run |
| N13 自证/假并发/窗口 | `b926789` 迁移＋WT31 | barrier 两参与者、真 exit70/SIGKILL、3 条反实现失败、独立 bytes/SQL/Git；全部 reliability_crash 窗口和全 inventory | T31 task、负控制 metadata/stdout、`mutants/superseded-before-implementation-review-fixes/`；**完整两阶段与存活处置 WIP** |
| N14 限额/重复扫描 | `5196cb1`、`429cb1d`、`543f9d2`、`a4f1968` | exact/+1 file/tree、先核总量、单次快照编译+digest、stats 真 writer 交错；`file_size_limit_is_exactly_32_mib`、`total_size_limit_is_exactly_256_mib`、`total_limit_rejects_before_staging_or_registering_source_files`、T29 stats 测试 | H04/H09/P21/P29、F/B；不宣称 OOM/性能收益量化 |

## 7. Core 第一阶段 missed 的独立预分析

增补公共合同 oracle 前读取完整 core 四片时，707 项为：stage1 missed 47、timeout 0；47 是旧第一阶段未捕获数，不是最终存活数。逐项旧输入现保留在 `mutants/superseded-before-implementation-review-fixes/superseded-default-profile-before-public-contract-oracles/stage1-sheltie-core-{0..3}-of-4/raw/outcomes.json` 与 `missed.txt`。下面是当时的预分析；新测试/opt1 profile 的完整 inventory 从头执行，不能把 superseded 结果当作当前通过。本报告不建议删公开 API。

| 组 / 数量 | 当前合同与真实 consumer | 最终处置所需证据 |
| --- | --- | --- |
| `WorkState::validate_persisted` / 28 | `store/read.rs::decode_row`（约 383 行）及 `store/commit.rs::commit`（约 80 行）真实调用；涉及 id/name/day、occurrence/visits、Attempt status/结果组合、running/current、Work status、approval 及已批准 Gate 一致性 | 从合法 CLI/Fixture 状态每次改单字段，核 load/list 精确 STORE_CORRUPT 或 commit 零写；保留合法 Running/Failed/Succeeded/Gate/Cancelled-running 正例捕获过严变体；不能以 core 本身未消费校验认 dead API |
| `HostRequire::deserialize` 长度边界 / 2 | `manifest.rs` 70/77 的 `>`→`>=`；合同允许 version 恰 32/source 恰 512 字节；Reply::AttemptBegun.requires 经 PersistedResponse/recovery 解码 | 旧 snapshot tests 只含短合法和 +1 非法；§2.1 已增补恰好上限的合法 URL/版本快照 roundtrip 与单字节越限反例。仍需记录新完整变异是否实际捕获，不能从 3 例正常通过推算 mutant 结果 |
| Getter / 14 | `RelPath::as_str`×2 → Repo load 的 read_tree_utf8；`NodeDef::title`×2 与 `FlowDef::edges`×1 → CLI show；`Manifest::version`×2、name×2、description×3、flows×1、requires×1 → Repo add/load 与 CLI show | 当前都有可观察 consumer；核非空/非默认实际字段、声明 Flow、requires，以及真 CLI JSON/text 独立固定期望；不能仅因 core 私有字段消费者绕过 getter 而等价化 |
| `parse_input_source` / 1 | `parse.rs` 140 的 `||`→`&&`；空/多点继续被 validate_id 拒绝，但 InvalidFlow.reason 从结构错误变为 ID 原因；真实 add/verify 经 parse_flow | 后续独立合同复核允许记为当前合同语义等价：reserved 分支不变，合法域/拒绝域与 FLOW_INVALID、rule=parse、path 不变；protocol 未承诺 reason 固定字面值，两种诊断均准确。不是完整 Error/JSON 字节等价，也不是测试 PASS；保留逻辑证明、变异 diff 与最终 workspace 结果 |
| `decide_approve` / 2 | `decide.rs` 574 的 `+=`→`-=`/`*=`；Gate→NoLegalEdge 是合法可达转换，protocol 累计受阻应分别记 Gate 和 NoLegalEdge | 新增真 CLI `gate_approval_without_a_remaining_edge_adds_a_second_blocked_event` 已在 F/B；其合法图有可达终点分支，批准后 blocked_count 1→2；须核 stage2 实际捕获，不能仅凭普通 Gate→Active 正例处置 |

公开 `legal_next` retry 边界的早期 missed 已由 core 新 oracle 捕获，保留当前完整分片结果及旧 superseded 记录；这不意味着其余 state 或 getter 项自动通过。stage2 最终统计须明确每项 caught 所在阶段，仍 missed/timeout/unviable 各自记录，不把等价或不适用写成测试 PASS。

## 8. 最终关闭前仍需完成

1. 完成当前 core/runtime 全 inventory 的两阶段结果，核无漏无重、正确变异 binary、全部 missed/无法判断 timeout 的 workspace 复验，以及逐存活体 Reviewer 处置。
2. 作者完成 T31 的结果汇总和逐条答复，固定产品提交及完整输入闭包；WT31 再映射实际产品修复 commit。文档更新与 mutation 源码/fixture 更新需分别记录是否使已有运行失效。
3. 独立 M1 Reviewer 在固定候选按本矩阵与原合同逐行复核，确认公共门禁 executed/reused 来源和同闭包证据。旧 `6baee533` 的 dist argv/exit 与普通 clone baseline 元数据缺口已补；新测试/profile 的候选门禁与完整基线仍须按 §2.1 记录，不把旧输入执行结果改写成新候选通过。
4. 全程保留 Linux not_run、同账户文件搬移边界、历史 LEAK 未逐次定位、SIGKILL 非断电，以及 T16/T17 后继门槛。不得把本报告的增量静态 PASS 或当前正常回归运行写成完整 M1 通过。
