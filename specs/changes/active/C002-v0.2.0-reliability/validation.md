# C002 验证与历史索引

Candidate: `e3eea899877165f8573befee3774555598ec92bd`

本文件合并最终验收摘要和 51 行覆盖矩阵；产品状态只看 plan.md，审查结论见 review.md。所有运行都属于下列固定输入，文档收敛不构成新运行，也不把未执行项改为 PASS。

## 候选与正常门禁

| 身份 | 固定值 |
| --- | --- |
| 整体审查基准 | `a664e75a3ab3041d09cd0a1ab4d69336f2dcd055` |
| 产品源码候选 | `e3eea899877165f8573befee3774555598ec92bd` |
| M1 记录提交 | `9c933d8da38b2e7b1968de55343bf27e0dc11c25` |
| 变异冻结 clone | `95d78e0677f1ffba6abe5ad9c13430ea53957101` |
| 变异 source-input SHA256 | `6139c0648032211ef2a02bd60d4a05fd1e2898f80c7944868979f8769ea0aa13` |
| 当前门禁 input SHA256 | `2390ad8b68cb86b9e33acf07b9172484d50871d92e990585f7d0af5806a2c1ce` |
| 输入集合 | 167 项源码/配置/fixture；冻结 clone 另存 234 项治理输入 |
| 正常 Nextest run | `2e126f6a-1496-4b0b-8d71-583e22d4065e`；699/699、零跳过、1 slow |

fmt、check、Clippy、nextest、MSRV 1.85 locked、docs/specs/core-vocab/tests/skill、dist plan 及离线 deny 均 exit0。deny 只核本地缓存公告，不声称在线更新。完整 argv、环境、退出码、stdout SHA 位于快照的 `evidence/m1-2026-10-01/current-acceptance/gates/gate-results.json`；所有历史路径通过下方恢复方法读取。

当前 crash 补充运行 24/24 父测试，58 次真实终止：32 exit70、26 SIGKILL。20 次 Start/Add 有显式序列化 oracle，其余 38 次按父测试和对应源码断言核实，不扩大为全部独立序列化 oracle或断电证明。独立 Reviewer 核 51 行、103 个显式测试引用均对应当前 G 的实际 PASS。

任务记录门禁默认基准 a664e75 误计前序任务已提交的 6 项快照变更，失败原文保留；M1 记录采用脚本支持的 e3eea899 显式基准，通过且无源码/测试/快照改动。整体代码审查仍对 a664e75，不改 allow_test_changes 或门禁脚本。历史 gate 不能因当前 MIT 元数据或后续输入变化而自动复用为新候选 PASS。

## 变异核算与实际缺失

2514 个精确 ID 与 inventory 集合一致，无漏无重；核算通过与完整验证通过分开记录。

| 处分 | 数量 | 范围 |
| --- | ---: | --- |
| 首轮正式 CaughtMutant | 1812 | 实际 Build Success 后 Test Failure；不改 abort 为断言失败 |
| 编译器确认 Unviable | 328 | 原 Rust 诊断，不是测试捕获 |
| 限定静态处分 | 74 | 精确当前 consumer/producer、合同或平台；不是动态 PASS |
| 完整 workspace 捕获 | 33 | 只计完整终态阶段 |
| 能力组 CLI 捕获 | 30 | 共执行 181 项：30 caught、151 missed；不是完整 workspace 运行 |
| 独立补充检测 | 22 | 16 直接、6 受控观察；不改正式 MissedMutant |
| SK02 额外执行缺失 | 215 | 逐 ID 保留缺失，不是 caught/equivalent/PASS |
| 合计 | 2514 | 原首轮 missed 为 374，其中 159 有限定处分/检测，215 缺失 |

中断 29 项及取消 18 项 workspace 原文仅 WIP，不计完成。74 静态处分的一项解析仅证明接受集合/类型/结构化定位相同，错误文本不同；4 项非 Unix backend 仅当前平台不适用。受控 source-alias 仅证明提前跟随差异，两次最终 CLI 均拒绝；tree-swap 未检测保留。Workbook 重读只证明已测 request_id/replayed 条件，不外推任意外部 SQLite 编辑保护。所有证明保留各自 consumer/锁/平台边界。

原 T31 的 2499 项结果与当时 269 项暂缓是历史记录，不套当前 2514 输入。恢复映射为 261 当前相同片段（13 跨函数）、2 退役接口、6 替换实现；映射本身不是执行或等价证明。

| 项目 | 实际暂停任务 | 未完成义务 |
| --- | --- | --- |
| SK01 / M1-safety-skip-001 | T34 独立 Spec 最终续审，/root/m1_spec；2026-10-01 15:23:42 UTC | 没有最终 Spec 批准；此前发现/方向意见只作历史证据 |
| SK02 / M1-safety-skip-002 | M1 新共享事件补充 oracle，/root/m1_evidence；2026-10-02 | 215 个精确 ID 缺额外执行，不虚构每个 ID 各触发提示 |
| Linux | 用户明确豁免的原生运行 | not_run，不记跨平台 PASS |
| 非 UTF-8 物理目录 fixture | 本机创建被 EPERM 拒绝 | 环境限制；不等于非法环境值未测试 |
| T16 / T17 | 真实 Host / 发布 | not_run，保留独立责任与授权门槛 |

两次实际提示原文、暂停阶段与缺失任务均在快照的 `evidence/m1-2026-10-01/safety-skips.json`，未重试、改写提示或转交替代执行。M1 在用户授权例外内完成；`disposition_complete=false`、`full_validation_pass=false`、`security_validation_pass=false`、`final_spec_approval=false` 保持不变。

## 历史证据恢复

归档快照：`e54dcd41d8f1e186007b62b47583063cb19a4b66`，本地引用 `refs/archive/C002-before-consolidation-20261002`。父提交 `9c933d8da38b2e7b1968de55343bf27e0dc11c25` 保存原 M1 记录；归档额外补入 392 个被忽略的 .log（28431150 字节）。当前移出的 4253 个 evidence 文件和 8 个阶段/repair 文档均已核对快照中的 Git blob 字节，不丢原文、不重写历史。工作树清理不缩减 Git 历史体积。

归档仅在本地建立，普通分支推送不会自动推送 refs/archive。恢复或迁移需要保留该引用或完整 Git 对象；缺 hash 时应报告缺历史，不声称原文可用。正式分支和暂存区未为归档切换；此前 MIT 改动没有混入归档。

路径都相对于 `specs/changes/active/C002-v0.2.0-reliability/`：

| 材料 | 快照内路径 |
| --- | --- |
| 旧审查与答复 | `review.md`、`review-m1-2026-09-28.md`、`review-m1-2026-10-01.md`、`review-m1-2026-10-02.md`、`review-implementation-2026-09-30.md`、`review-response-implementation-2026-09-30.md` |
| 完整实施手册与 V01–V33 | `repair-design.md`、`repair-plan.md`、`repair-validation.md` |
| 各任务和早期失败 | `evidence/tNN/`、`evidence/repairs/tNN/`、`evidence/m1-2026-09-28/`；具体原路径见快照 validation.md |
| 51 行完整矩阵与当前门禁 | `evidence/m1-2026-10-01/coverage-matrix.md`、`evidence/m1-2026-10-01/current-acceptance/` |
| inventory/输入闭包/原运行 | `evidence/m1-2026-10-01/mutants/`，完整 stage 的 raw.tar.gz、成员 SHA、loose index 均保留 |
| 逐 ID 最终账本 | `evidence/m1-2026-10-01/mutants/adaptive-validation-2026-10-02/final-ledger.json` |
| 215 项缺失映射 | `evidence/m1-2026-10-01/mutants/adaptive-validation-2026-10-02/SK02-missing-execution-map.json` |
| 核算与 Reviewer 原文 | `evidence/m1-2026-10-01/mutants/adaptive-validation-2026-10-02/independent-final-accounting.json`、`evidence/m1-2026-10-01/mutants/reviewer-records/` |
| 实际跳过/验收范围 | `evidence/m1-2026-10-01/safety-skips.json`、`evidence/m1-2026-10-01/acceptance-scope.json` |

读取单个原文，不改变当前工作区：

```bash
git show e54dcd41d8f1e186007b62b47583063cb19a4b66:specs/changes/active/C002-v0.2.0-reliability/review-m1-2026-10-02.md
```

导出完整历史 package 到独立目录，不覆盖当前文件：

```bash
mkdir -p /private/tmp/sheltie-c002-history-e54dcd41
git archive e54dcd41d8f1e186007b62b47583063cb19a4b66 specs/changes/active/C002-v0.2.0-reliability | tar -x -C /private/tmp/sheltie-c002-history-e54dcd41
```

## 验收矩阵

保留原 51 行提交、真实入口、正反例、独立 oracle 与结果范围。G 为上述当前 699 项 run；H 为历史 ca6d92f 的 675 项运行；Hnn 指快照 `evidence/tnn/`，P19/P20 指 `evidence/t19-managed-fs-2026-09-28/` 和 `evidence/t20-trusted-load-2026-09-28/`，P21–P30 指 `evidence/repairs/tnn/`。引用均由历史快照读取，不依赖已移出的工作树路径。R17/N13 的授权例外交接不代表完整验证通过。

| Finding | 修复提交 | 真实入口、正反例与独立 oracle | 运行结果与限制 |
| --- | --- | --- | --- |
| R01 路径归属 | `ee78118`、`22942ee`、`5d2d051` | `WorkService::load/run_command`、`CheckedEffects`；单改 state 根、后续效果路径、ArtifactRef，根外 bytes/mode 不变；`tampered_work_dir_is_rejected_before_reading_outside_paths`、`invalid_later_effect_is_rejected_before_any_effect_runs`、`tampered_artifact_path_is_rejected_before_observing_external_file` | 列明源码与离线oracle通过；P19/P20/P24、当前候选全仓原文 G；历史原文 H；原实现候选ca6d92f |
| R02 seal | `ee78118`、`fd21a63`、`1d92bcd` | 真 submit COMMIT→seal rendezvous；原 ref 独立 SHA/bytes、SQLite revision/published、外部 sentinel mode；`real_submit_path_swap_after_commit_seals_original_and_preserves_external_target`、`real_submit_same_inode_byte_change_after_commit_stops_before_chmod`、`real_submit_hardlink_added_after_commit_stops_before_chmod` | 列明源码与离线oracle通过；P22/P25、当前候选全仓原文 G；历史原文 H；Linux not_run |
| R03 self 文件链 | `ee78118`、`7e9178e` | `SelfManager::install/update/rollback/uninstall`；只改单一 tmp/bin 父链接、归档链接，旧 binary/prev、外部 bytes/mode；`install_rejects_tmp_parent_symlink_and_preserves_external_sentinel`、`rollback_rejects_bin_parent_symlink_and_preserves_external_binaries` | 列明源码与离线oracle通过；P23、当前候选全仓原文 G；历史原文 H；远端资产 T17 not_run |
| R04 purge | `69710aa`、`7e9178e`、`ca6d92f` | 真 self uninstall --purge 和等待者；root/.lock dev/ino 保留、Store 最后删、部分失败；`purge_removes_frozen_work_and_binary_but_preserves_same_root_lock`、`purge_waiters_reach_the_failed_try_lock_before_initialization_or_old_work_rejection` | 列明源码与离线oracle通过；P23、当前候选全仓原文 G；历史原文 H、T31 task raw；Linux not_run |
| R05 @file 重放 | `5d2d051` | CLI start/submit/fail；成功后删除源，同 rid 返回原响应，不新增序号；`start_request_replay_does_not_require_the_original_input_file`、`submit_request_replay_does_not_reread_a_deleted_summary_file`、`fail_request_replay_does_not_reread_a_deleted_reason_file` | 列明源码与离线oracle通过；P24、当前候选全仓原文 G；历史原文 H；系统别名补强在 `ca6d92f` |
| R06 历史目标 | `5d2d051` | 所有 Work 写入口先取历史完整 WorkId；新增同前缀 Work、不匹配前缀，目标及快照不漂移；`historical_work_id_resolves_before_a_now_ambiguous_prefix`、`work_prefix_matching_is_case_sensitive_and_replays_with_the_same_selector` | 列明源码与离线oracle通过；P24、当前候选全仓原文 G；历史原文 H |
| R07 恢复错误协议 | `1d92bcd`、`ca6d92f` | `recovery::before_write/finish_request` 和 CLI；自己 true/原响应、旧 A 阻 B false、mark 零行、坏 BLOB、历史父缺失；`post_commit_card_failure_returns_committed_response`、`old_effect_blocks_new_request_with_distinct_identities`、`mark_published_zero_rows_returns_committed_recovery_error`；新历史类型真实 CLI 用例 | 列明源码与离线oracle通过；P25、当前候选全仓原文 G；历史原文 H；T31 `r07-*`、`dangling-history-*` 保存红绿和独立探针 |
| R08 Workbook 重放 | `1d92bcd`、`3dd224d` | 真 add/remove 同 rid；未发布先恢复，完成历史不绑定新生命周期；`workbook_replay_finishes_unpublished_effects`、`completed_old_remove_and_add_replays_preserve_new_lifecycle_bytes_and_row` | 列明源码与离线oracle通过；P25/P27、当前候选全仓原文 G；历史原文 H |
| R09 pending 读 | `4526b7e` | Repo list/show/verify、Work status/start preflight；pending/final、owner、清理后 completed、真实 rename/mark 交错；`committed_pending_workbook_is_readable_without_recovery_and_is_marked_pending`、`real_work_status_retries_after_start_publish_renames_post_location`、`workbook_reader_accepts_publication_and_cleanup_after_its_reference_index` | 列明源码与离线oracle通过；P28、当前候选全仓原文 G；历史原文 H；只读不恢复，SQLite 控制例外单列 |
| R10 跨入口卡刷新 | `1d92bcd` | Workbook add/remove 恢复旧 Work Start，而最新 begin 已提交；状态卡等于捕获的最新 bytes，历史回复不变；`workbook_write_recovers_work_status_card` | 列明源码与离线oracle通过；P25、当前候选全仓原文 G；历史原文 H |
| R11 删除证明 | `3dd224d`、`4526b7e`、`ca6d92f` | remove 同对象移入/删、marker 同 rid/internal-id；未知结果停止，空替代根保留；`remove_marker_cannot_be_borrowed_from_another_internal_id`、`marker_replaced_after_validation_cannot_prove_deletion`、`remove_real_windows_preserve_unknown_results_and_do_not_touch_a_new_lifecycle` | 列明源码与离线oracle通过；P27/P28、当前候选全仓原文 G；历史原文 H；kill 不等于断电 |
| R12 发布闭包 | `22942ee`、`5a9d430` | Start/Add pending/final：owner、manifest、digest_root、每个 start ref 实际 bytes/len；`publication_checks_each_start_input_reference_and_owner_field`、`final_only_publication_recovery_checks_start_input_bytes`、`workbook_publication_recovery_checks_owner_and_manifest_identity` | 列明源码与离线oracle通过；P20/P26、当前候选全仓原文 G；历史原文 H |
| R13 cleanup | `4526b7e`、`ca6d92f` | 全 requests effects 索引；只清合法孤儿/完成空元数据，非法路径/坏 JSON 保留；`cleanup_index_rejects_a_decodable_effect_path_that_points_into_an_orphan`、`malformed_request_effects_stop_cleanup_before_any_unreferenced_object_is_removed`；same-rid 未提交残留清理失败不登记新请求 | 列明源码与离线oracle通过；P28、当前候选全仓原文 G；历史原文 H、T31 `missing-same-rid-preparation.*` |
| R14 树装入/摘要 | `ee78118`、`543f9d2` | add source、installed load/verify、Start frozen copy、Work load、publish verify 共用受限快照；独立 Python framing 向量、碰撞对、exact/+1、root/parent/leaf 链接与 FIFO；`digest_matches_independent_vectors_on_disk`、`tree_reader_rejects_symlinked_parent_directory`、`total_limit_rejects_before_staging_or_registering_source_files` | 列明源码与离线oracle通过；P21、当前候选全仓原文 G；历史原文 H；不声称性能量化 |
| R15 stats 快照 | `a4f1968` | 真 CLI stats 装入后暂停，writer begin 已完成再释放；手写旧 stats/旧 next 同一 revision；`stats_and_next_keep_one_snapshot_when_a_writer_begins_after_reader_load` | 列明源码与离线oracle通过；P29、当前候选全仓原文 G；历史原文 H；`red.*` 和 `stop-route-mutation.*` |
| R16 replan | `b926789` | 真 CLI+临时 Git+只读绑定 JSON 的冷 worker；原字节镜像、累计前缀、原基线、Git 祖先、审批/原始 gate；`fresh_replanner_recovers_verified_task_and_original_baseline_only_from_bound_files`、`fresh_replanner_rejects_deep_dropped_rows_failed_reports_denied_approval_and_cycles` | 列明源码与离线oracle通过；P30、当前候选全仓原文 G；历史原文 H；模拟 worker，不替代 T16 agent 质量 |
| R17 全验证闭包 | `b926789` 迁移＋`ca6d92f` | barrier/真实子进程窗口，exit70 与 signal9 分别核；反实现摘锁、删父 sync、删 same-rid preparation 必须失败；完整 core/runtime inventory 两阶段 | M1授权例外内交接完成；T31负控制原文保留，2514项精确核算及既有能力验证完成；215项新额外执行按实际SK02暂停任务跳过，完整验证未通过；SK01缺最终Spec批准 |
| R18 必需 sync | `ee78118`、`5a9d430`、`3dd224d`、`ca6d92f` | pending/final-only 两格重复同一 fault；source/target parent sync 失败不 mark，重试 inode 不变；`publication_sync_failures_leave_effect_pending_and_retry_same_object`、`start_replay_sync_failure_preserves_own_commit_snapshot`、`moved_delete_recovery_resyncs_both_rename_parents_before_deleting_payload` | 列明源码与离线oracle通过；P26/P27、当前候选全仓原文 G；历史原文 H、`missing-delete-parent-sync.*`；不声称掉电证明 |
| R19 锁内建库 | `5d2d051`、`edb86f1`、`ca6d92f` | WriteSession 持同根锁后 RW Store；旧 schema 无新锁、main/WAL 不变；真 add↔install 初始化，旧 Work 等 purge 后不 CREATE；`real_install_waits_for_a_concurrent_workbook_add_store_initializer`、`real_workbook_add_waits_for_a_concurrent_install_store_initializer`、`old_work_waiting_behind_purge_does_not_recreate_the_removed_store` | 列明源码与离线oracle通过；P24、`repairs/t24-followup/`、当前候选全仓原文 G；历史原文 H |
| O01 根隔离 | `ee78118`、`22942ee`、`543f9d2`、`fd21a63`、`7e9178e` | add/start/submit/self 全链；单路径/父链接/叶换绑，外部 bytes/mode；R01–R03/R14 上述测试 | 列明源码与离线oracle通过；P19–P24、当前候选全仓原文 G；历史原文 H；同账户搬对象边界按合同 |
| O02 跨 Work 重放 | `bcfa0e6`、`5d2d051` | 同 rid cancel A 后 cancel B，B 保持 Active；`cross_work_request_id_is_request_conflict_and_target_untouched` | 列明源码与离线oracle通过；H07/P24、当前候选全仓原文 G；历史原文 H |
| O03 Workbook 身份 | `d38b6e1`、`429cb1d`、`22942ee`、`543f9d2`、`4526b7e` | Repo load/verify/remove：行/manifest/digest/publisher；`load_rejects_tampered_registered_digest`、`completed_workbook_final_requires_its_current_successful_publisher` | 列明源码与离线oracle通过；H05/H09、P20/P21/P28、当前候选全仓原文 G；历史原文 H |
| O04 历史回复/观察 | `bcfa0e6`、`5d2d051`、`1d92bcd` | submit→cancel→旧 submit，回复逐字段、旧源消失；`old_submit_replay_after_cancel_returns_original_reply`、三个 @file replay 用例 | 列明源码与离线oracle通过；H07/P24/P25、当前候选全仓原文 G；历史原文 H |
| O05 历史 engine.stats | `bcfa0e6`、`1d92bcd`、`ca6d92f` | begin→推进→删旧 stats→重放 begin；登记 content 原 bytes、当前卡不回退；`begin_replay_regenerates_missing_stats_json`、`replay_does_not_rewrite_status_card_to_old_revision` | 列明源码与离线oracle通过；H07/P25、当前候选全仓原文 G；历史原文 H；类型/悬空链接补强 raw 在 T31 |
| O06 clean home | `d38b6e1`、`2320020`、`5d2d051`、`edb86f1` | 真 install/add、two-step、缺失只读 Home；`self_install_on_new_home_creates_root_store_and_bin`、`clean_home_install_then_update_two_step` | 列明源码与离线oracle通过；H05/H15/P24/follow-up、当前候选全仓原文 G；历史原文 H |
| O07 摘要碰撞 | `429cb1d`、`543f9d2` | 显式 framing 独立 SHA、碰撞对、插入顺序；`digest_matches_independent_vectors_on_disk`、`framing_collision_pair_now_yields_two_different_digests` | 列明源码与离线oracle通过；H09/P21、当前候选全仓原文 G；历史原文 H |
| O08 Workbook/self 幂等 | `766dbe2`、`2320020`、`1d92bcd`、`3dd224d` | add/remove 历史快照与新生命周期；self rid 拒绝；`add_and_remove_replay_return_original_snapshots`、`request_id_rejected_for_readonly_and_self_commands` | 列明源码与离线oracle通过；H08/H15/P25/P27、当前候选全仓原文 G；历史原文 H |
| O09 每任务 Git 范围 | `c1a530e`、`b926789` | 真临时 Git diff --name-only 对手写白名单；`two_task_loops_scope_each_task_diff_to_its_own_files`、`out_of_whitelist_file_in_second_task_is_flagged` | 列明源码与离线oracle通过；H12/P30、当前候选全仓原文 G；历史原文 H |
| O10 人工意见/继续 | `c1a530e`、`b926789` | CLI 输入路径、审批摘要、escalate 回程、明确更正；`conditional_approval_binds_decision_into_scaffold_implement_verify`、`fresh_replanner_accepts_explicit_human_approval_correction_and_rejects_stale_version` | 列明源码与离线oracle通过；H12/P30、当前候选全仓原文 G；历史原文 H；T16 真人/agent 实测 not_run |
| O11 skill 自包含 | `4960e9f` | 真打包/复制后删源码；文件集合与链接、missing/stale/escape 反例；`installed_delivery_stays_self_contained_after_source_tree_is_removed`、`check_skill_fails_when_delivery_link_escapes` | 列明源码与离线oracle通过；H13、当前候选全仓原文 G；历史原文 H；真实 Host 发现/执行 T16 not_run |
| O12 输出命名空间 | `6b40abd`、`bcfa0e6`、`fd21a63` | 真 worker outputs brief/stats 与 engine 分离；祖先/大小写别名拒绝；`worker_outputs_named_brief_and_stats_write_and_submit_via_real_chain`、`add_rejects_ancestor_and_folded_alias_output_paths_at_cli` | 列明源码与离线oracle通过；H03/H07/P22、当前候选全仓原文 G；历史原文 H |
| O13 完整事实视图 | `85ada77`、`bcfa0e6`、`4526b7e`、`a4f1968` | 文本/JSON reason、完整 ref、同形 next、手写非零状态；`text_and_json_agree_on_hand_written_state`、`status_json_carries_reason_and_full_artifact_refs`；真 pending/stats caller | 列明源码与离线oracle通过；H06/H07/P28/P29、当前候选全仓原文 G；历史原文 H |
| N01 start 预检 | `df0e120`、`5d2d051` | 真 CLI 缺/多键、名、Workbook/Flow、坏 @file；Home 目录快照、SQL rows/sequence 无变化；`start_deterministic_rejections_leave_home_unchanged_and_do_not_burn_seq`、`failed_start_on_new_home_creates_nothing` | 列明源码与离线oracle通过；H02/P24、当前候选全仓原文 G；历史原文 H；分配后合法空号不回收 |
| N02 Workbook 生命周期 | `d38b6e1`、`766dbe2`、`1d92bcd`、`3dd224d`、`4526b7e` | add/remove 活动引用、pending 读/恢复、未知删除、新生命周期；`remove_rejects_active_reference_and_rolls_back_row`、`completed_old_remove_and_add_replays_preserve_new_lifecycle_bytes_and_row`，R08–R13 反例 | 列明源码与离线oracle通过；H05/H08/P25–P28、当前候选全仓原文 G；历史原文 H |
| N03 schema/持久验证 | `63d4d48`、`22942ee`、`5d2d051`、`edb86f1` | Store 真 load/list/commit；schema1 main/WAL 原 bytes、行 identity/status/revision 单改、合法 Running/Failed；`schema1_store_rejected_without_touching_file`、`load_rejects_row_identity_status_and_revision_mismatch`、`store_accepts_real_running_and_failed_attempt_states` | 列明源码与离线oracle通过；H14/P20/P24/follow-up、当前候选全仓原文 G；历史原文 H；state mutant 的完整处置归 R17 |
| N04 OS 主体/授权 | `d38b6e1`、`a5b2a05`、`b926789` 迁移 | 真子进程伪造 USER 与 id -un 独立期望；`audit_principal_ignores_spoofed_user_env`、`principal_matches_id_un_oracle` | 列明源码与离线oracle通过；H05/H10/P30、当前候选全仓原文 G；历史原文 H；不声称独立真人认证，T16 not_run |
| N05 modify-path/JSON | `2320020`、`7e9178e`、`b926789` 迁移 | 真 CLI flag 拒绝、宿主 rc 原 bytes、单 JSON stdout；`install_modify_path_flag_is_rejected`、`self_install_json_stdout_is_single_document` | 列明源码与离线oracle通过；H15/P23/P30、当前候选全仓原文 G；历史原文 H |
| N06 固定 release/tag | `2320020`、`7e9178e` | 本地 release fixture，固定 tag、latest 毒字节、manifest 错配、rollback 原 bytes；`update_pinned_version_installs_only_that_tag`、`update_rejects_manifest_version_not_matching_tag`、`update_pinned_version_then_rollback_restores_previous` | 列明源码与离线oracle通过；H15/P23、当前候选全仓原文 G；历史原文 H；远端/四平台 T17 not_run |
| N07 stats 累计/edge | `85ada77`、`bcfa0e6`、`a4f1968`；`ca6d92f` 补 oracle | 手写来源 node/edge、NoLegalEdge→cancel，Gate→NoLegalEdge 应 1→2；`entered_via_distinguishes_edges_and_entry`、`blocked_count_survives_cancel_after_no_legal_edge`、`gate_approval_without_a_remaining_edge_adds_a_second_blocked_event` | 列明源码与离线oracle通过；H06/P29、当前候选全仓原文 G；历史原文 H；完整变异结论仍归R17，不从普通测试推定 |
| N08 最终审查基线 | `c1a530e`、`b926789` | CLI 冷 worker、真实 Git、递归来源/累计前缀；`fresh_replanner_recovers_verified_task_and_original_baseline_only_from_bound_files`、`fresh_replanner_stops_on_reset_baseline_missing_fields_or_dropped_verified_task` | 列明源码与离线oracle通过；H12/P30、当前候选全仓原文 G；历史原文 H；模型履行 T16 not_run |
| N09 article-review 意见 | `b3f138b` | 真 back 回环：首次空、后绑前轮 verdict、重复回环取最新 occurrence；`first_draft_marks_review_input_absent_without_body`、`back_to_draft_binds_review_verdict_path_with_source_occurrence` | 列明源码与离线oracle通过；H11、当前候选全仓原文 G；历史原文 H；内容好坏由 worker，T16 not_run |
| N10 冻结根/元数据 | `d38b6e1`、`543f9d2`、`7e9178e` | 目录0555/文件0444、DS_Store 拒绝、改 bytes digest 停止、删仓库仍从冻结副本读；`readonly_bits_reduce_accidents_but_digest_is_the_guard`、`ds_store_rejected_by_name_at_add`、`work_readable_after_workbook_removed` | 列明源码与离线oracle通过；H05/P21/P23、当前候选全仓原文 G；历史原文 H；不是同账户防篡改 |
| N11 治理历史/版本 | `2320020` | 真临时浅/完整 Git、active/RC、越目标、缺 changelog/tag；`check_specs_shallow_clone_reports_missing_history`、`check_specs_accepts_active_target_without_tag`、`check_specs_rejects_version_outside_active_target` | 列明源码与离线oracle通过；H15、当前候选全仓原文 G；历史原文 H；远端 CI not_run |
| N12 同 SHA 质量/MSRV | `2320020`、`ca6d92f` 门禁 | 工作流删 needs/if 单条件反例、MSRV locked 实编译、dist plan 形状；`release_workflow_gates_announce_on_quality_job`、`release_workflow_quality_failure_blocks_announce` | 列明源码与离线oracle通过；H15、当前候选全仓原文 G；历史原文 H、T31 `msrv.stdout.txt`、`dist-plan.metadata.json`/JSON；dist argv/exit 缺口已关闭，T17 not_run |
| N13 自证/假并发/窗口 | `b926789` 迁移＋`ca6d92f` | barrier 两参与者、真 exit70/SIGKILL、3 条反实现失败、独立 bytes/SQL/Git；全部 reliability_crash 窗口和全 inventory | M1授权例外内交接完成；T31负控制metadata/stdout保留，2514项核算及既有窗口/能力验证完成；215项新额外执行按实际SK02暂停任务跳过，完整验证未通过；SK01缺最终Spec批准 |
| N14 限额/重复扫描 | `5196cb1`、`429cb1d`、`543f9d2`、`a4f1968` | exact/+1 file/tree、先核总量、单次快照编译+digest、stats 真 writer 交错；`file_size_limit_is_exactly_32_mib`、`total_size_limit_is_exactly_256_mib`、`total_limit_rejects_before_staging_or_registering_source_files`、T29 stats 测试 | 列明源码与离线oracle通过；H04/H09/P21/P29、当前候选全仓原文 G；历史原文 H；不宣称 OOM/性能收益量化 |
| R20 持久计数 | `ca6d92f` | V33：合法Gate count=1、approve后NoLegalEdge count=2；MAX/0/上界+1单字段反例，真实status/stats/list/write均返回STORE_CORRUPT、原state和业务行不变；纯core checked增量返回错误而非panic | 列明源码与离线oracle通过；G/H；T31 blocked-count/counter-core/counter-diagnostic-order原文及独立探针；计数必要界不宣称重建全部历史 |
| R21 根解析错误 | `06c3af3` | 真实Home::resolve/confine权限反例、非法环境值及显式CLI优先序、真实CLI子进程cwd消失；I/O只对NotFound退让，其他错误保留路径/原因，state/seq/requests不变；独立原bytes及错误码oracle | T33权限红绿、confine/home/lexical-cwd原文，当前G；独立Spec/Standards增量通过；物理非UTF8目录fixture在本机创建EPERM，保留环境限制，严格转换仅静态/API复核；Linux not_run |
| R22 原响应业务绑定 | `e3eea89` | 真实CLI同rid，合法id/version/digest单字段漂移；本人original和旧A的pending_original拒绝伪成功，合法后效果错误保留捕获原响应，Store/业务bytes不变；`corrupt_remove_snapshot_cannot_be_projected_as_a_successful_original`、`corrupt_pending_remove_snapshot_cannot_be_projected_as_a_blockers_original`、`add_snapshot_target_is_bound_before_an_original_response_is_released` | T34红色probe/CLI回归、G、target-projection-controls；Standards通过；Spec后续SK01实际暂停按授权跳过，不记Spec通过 |
| R23 历史状态 | `e3eea89` | 真start/begin/fail，首次active、末次blocked；随后cancel仍可重放历史，非法status单字段拒绝且原响应不可核；`fail_snapshot_status_matches_the_original_retry_even_after_later_progress` | T34原实现probe、G；core共享规则不重建历史visits；SK01及Linux边界 |
| R24 节点资源 | `e3eea89` | 真begin及同rid合法对照；Reply/data一致但与冻结节点不符的requires拒绝，业务行/原bytes不变；`begin_snapshot_requires_match_the_frozen_node_even_when_reply_and_data_agree` | T34原实现probe、G；core生成/验证同一节点资源函数；SK01及Linux边界 |


## 收敛后的独立维护提交

T35仅将项目许可统一为MIT，提交 `7a584a3c1dc0a0589f6839eef23c755cf53320c6`；三个Cargo包metadata及package --list核MIT正文且无项目Apache正文，第三方声明保留。Rust提交门禁及Spec/Standards独立复核通过。T36只收敛文档/引用及历史证据，独立提交；原M1候选和运行不因此变成新候选PASS。

T36工作树Rust四门禁exit0；Nextest run `ad1e5b10-625d-4c55-b513-2a8bcf3f06d6`，699/699、零跳过，测试168.880秒。此次未重跑mutation。docs/specs和显式基准 `7a584a3` 的Task36范围检查通过；临时命令原文位于 `/private/tmp/c002-t36-gates/`，不是归档M1输入。
