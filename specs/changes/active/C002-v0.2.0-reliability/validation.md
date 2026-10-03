# C002 验证与历史索引

Candidate: `8455aed2bcd9ac1739852be3987091a0975e7ac3`

本文件合并最终验收摘要和 51 行覆盖矩阵；产品状态只看 plan.md，审查结论见 review.md。所有运行属于各自阶段的固定输入，文档收敛不构成新运行，也不把未执行项改为 PASS。

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


T37仅提交执行效率复盘和通用建议。Rust四门禁exit0；Nextest run `466e6640-7ca7-4486-8bcf-3b0fc259d62c`，699/699、零跳过，原文临时保存于 `/private/tmp/c002-t37-gates/`。docs/specs和显式基准a31824b的任务范围检查通过；独立Spec/Standards复核只针对文档，原M1输入、SK01/SK02和未采用门禁建议不变。

## C002-T38 精简验证

基准：`4b86279a863f941cc282fa6cb772f7359fda585a`。初始工作区干净，候选为本次未提交工作区。生产 diff 由 code-simplifier 编写，Codex逐处复核。改动与分析见 review.md 的 T38 节，行为缺口见 findings.md 的 F38 项；原 M1 候选、变异与 SK01/SK02 不转记为新候选证据。

本次命令和原始 stdout/stderr 保存在 `/private/tmp/sheltie-t38-gates/`，结构化结果为 `results.json`，输入文件清单为 `input-files.json`。使用独立 `CARGO_TARGET_DIR=/private/tmp/sheltie-simplify-t38-target`、`RUSTC_WRAPPER=`、`CARGO_NET_OFFLINE=true`、Nextest线程2；Nextest取仓库既有 `target/t31-validation/tools/cargo-nextest`（0.9.146）。在线公告未刷新，deny仅核本地缓存。工具入口首次使用PATH的Nextest 0.9.140，未满足required 0.9.145，exit92且未执行测试；原文保存在 `/private/tmp/sheltie-t38-simplifier-tests.log`，不计测试FAIL或PASS。

### 候选与结果

- 生产 Rust 改动 10 文件、74 行新增、204 行删除，净减少 130 行。`production.patch` SHA256：`ef5a89085fa3ce5f1333234ee7b8dbed0366494830fb2e239d0f4b4cf39f9349`。
- 166 项源码/构建配置/fixture 输入清单 SHA256：`bdc6cfc92b4bb45b43d48c0cbec5ddf75db3929460105049da8b5947a69181f0`；门禁前后无输入漂移。`integrity.json` 核对 75 个测试文件、快照或内联测试区域，原字节全部相同。首次辅助检查将 Store 的测试专用 helper 之后的生产方法也算为测试，纠正为该 helper 自身边界后确认未改测试；不是测试执行失败。
- fmt、check、Clippy、Nextest、MSRV 1.85 locked、docs、specs、core-vocab、tests、skill、dist plan 全部 exit 0。Nextest run `11272072-a410-4024-8cb4-39bad529546b`：41 binaries，699/699、0 skipped、1 slow；测试阶段 164.258 秒，完整命令 174.233 秒。
- 初次离线 deny 因默认公告库锁位于沙箱只读目录而 exit 1，保留 `deny.stderr`。将同一缓存公告库复制到独立临时目录，仅覆盖 `advisories.db-path` 后重新执行 exit 0；不修改仓库 deny 策略。缓存 commit `db663534ae858abb3fbad408a041ce04209c377f`，策略及覆盖记录见 `deny-cache.json`，仍保留重复依赖/许可警告，不声称在线漏洞公告已刷新。
- CLI 产物通过 `cargo build -p sheltie-cli --all-features --message-format=json` 获取，exit 0，原文为 `cli-artifacts.jsonl`/`cli-build.stderr`。被测 binary SHA256：`c8cccc8ebded6df26a64c9a4ab713f0ef7cbcce23f052fec7965646751be5567`。
- code-simplifier 局部补充 `cargo test -p sheltie-core --all-features work::decide::`：49/49 PASS，原文 `/private/tmp/sheltie-t38-simplifier-core-tests.log`；不替代上述全仓运行。

### 新问题的取证

以下探针都在独立临时管理根，未触碰真实用户数据，也未修改仓库测试。结果确认的是现有缺口，不是正向合同 PASS；相关定义、装入与 self 源码相对基准未变，未由本次精简引入。

| Finding | 原文与可重跑入口 | 实际结果 |
| --- | --- | --- |
| F38-01 tmp 过期维护 | 全生产源码调用链检查及 `rg -n 'modified\(|cleanup.*tmp|tmp.*cleanup|86_400|86400' crates/sheltie-runtime/src` | 静态确认只有当前 self tmp 清理，未接入跨操作 mtime 维护；未执行过期清理动态 oracle |
| F38-02 嵌套未知字段 | `/private/tmp/sheltie-t38-nested-fields-cli-probe.py` 与同名 `.json`（SHA256 `b95215e7033d52922ed6c458bb3253d0f04332508e3c59355cc1b822d2ceea1b`） | control exit 0；只增顶层字段 exit 1/STORE_CORRUPT；只增 AttemptId 或 WorkStatus 字段均 exit 0，响应与 control 相等。每例 state_json 原字节、revision/request/audit 和业务文件摘要不变 |
| F38-03 重复 Flow id | `/private/tmp/sheltie-t38-duplicate-flow-cli-probe.py` 与同名 `.json`（SHA256 `9d6883c3eb5031efb52a93b31538efffce81e5c5537641e016f02250e97c58dd`） | 首次 add committed=true/EFFECT_PENDING，workbooks/requests/audit 各一行、published=0、final 不存在；同请求重放仍失败，无关新 add committed=false 且 pending_request_id 指向该坏请求。后续请求未新增行 |

Codex 对全部生产 diff 复核通过，code-simplifier 对架构/协议文案勘误复核确认与既有源码及 caller 一致。行为缺口保持 OPEN；本次没有重跑全量变异或重新作 M1 验收。最终 docs/specs 和 `scripts/check-task.sh C002-T38 4b86279 --staged` 均 exit 0；原文在本机临时目录，不属于固定 Git 历史归档，本次未提交或发布。

## C002-T39 修复验证

HEAD：`4b86279a863f941cc282fa6cb772f7359fda585a`；候选为保留 T38 精简的未提交工作区。开工 tracked 输入及差异在 `/private/tmp/sheltie-t39-evidence/input-hashes.json`、`input-worktree.patch`，本次修复与上轮精简分别归属。三个新增 CLI 测试文件和 core/fsx 新用例均标 `Task: C002-T39`。

先红后绿的原文：Flow 为 `/private/tmp/sheltie-t39-flow-{red,green}.log` 及 `sheltie-t39-flow-report.md`；嵌套解码为 `/private/tmp/sheltie-f39-decode-{cli,core}-{red,green}.log`；tmp 为本目录下 `tmp-red.log`、`tmp-green-initial.log`、`tmp-green-final.log`、`tmp-green-reviewed.log`、`tmp-exact.log`。Flow 修前 1 PASS/2 FAIL，最终新增 4/4 PASS；嵌套 CLI 修前 4 FAIL/修后 4 PASS，core 修前 2 FAIL/修后 2 PASS；tmp 修前 1 PASS/3 FAIL，最终 CLI 7/7 PASS，真实私有文件清理边界 1/1 PASS。过滤后零用例的其他 binary 不计为额外验证通过。

实现迭代中 FIFO fixture 尝试 rustix mknodat/mkfifoat 时 macOS 不导出该 API，发生编译失败；改用仓库既有 `mkfifo` 测试方式，无新依赖或生产 API。这不是行为红。首个补例编译诊断保留在会话原文，第二次原文为 `tmp-green.log`，最终成功原文另存，不重写为最初通过。

最终全仓门禁均exit0，原始 stdout/stderr 与结构化命令/退出码位于 `/private/tmp/sheltie-t39-evidence/gates/`。输入包括三份未跟踪的新测试；固定清单 SHA256 `311099bf1e295f7a06d61f9e97991ca90ff58f0482a0aa6e3e0a53a6a82b09d1`。`RUSTC_WRAPPER=`、独立 target、离线 Cargo、Nextest 0.9.146/线程2与本机 macOS；deny 复用原缓存公告内容并仅改临时 db-path，实际重新执行，不复用 T38 退出码，不声称在线刷新。

最终结果：fmt/check/Clippy/nextest、离线deny、MSRV 1.85 locked、docs/specs/core-vocab/tests/skill与dist plan均exit0。Nextest run `3284ed1d-b9a5-4546-bce5-dbd06ea6f14d`：44 binaries，717/717、0 skipped、1 slow，测试阶段166.013秒（全命令172.366秒）。相对T38增加18个测试，所有新增用例实际运行。169项构建输入在收尾复核无漂移；Cargo.lock、原fixture、快照与既有T38生产简化保持，本次只改6个生产文件并新增3个CLI测试文件及core/fsx单元回归。

未参与实施的 `/root/review_repairs` 三项审查均“通过”，完整原文 `/private/tmp/sheltie-t39-independent-review.md`（SHA256 `ab77a2d7f4a2e31da5a9efd058d33e07fdee2ee06c4f6e856c217d55dfb0dcd7`），涵盖9个冻结源码/测试hash、真实红绿和目录替换调用链。code-simplifier只读复核确认没有需追加的抽象。`integrity.json`记录实际输入和本次增量，最终docs/specs与 `scripts/check-task.sh C002-T39 4b86279 --staged` 均exit0，完整记录为 `final-checks.json`。

F38-03/F38-02/F38-01按本次修复、正反例、真实caller与独立审查关闭；不是仅凭717全绿关闭。原文在本机临时目录，不属于固定Git历史归档。本次未提交、推送或发布；未修写已有坏Store，M1原例外、Linux/T16/T17状态不变。


## C002-T40 测试精简验证

HEAD为`4b86279a863f941cc282fa6cb772f7359fda585a`，候选是保留T38/T39的未提交工作区。本次初始文件字节、diff、717项名单在`/private/tmp/sheltie-t40-evidence/`；`input-tree/`为逐哈希核验的原始镜像。最初Git archive/diff未包含三个未跟踪T39测试，随后从固定旧原文恢复并逐SHA验证，计量基准完整，不把缺失文件误记为新增测试。

| 局部证据 | 实际范围 |
| --- | --- |
| core | 205/205及5/5 compile_fail通过；`/private/tmp/sheltie-t40-core-test.log` |
| runtime | run `f3672b0e-2496-4268-aa38-809827aeb6ff`，103/103通过、204显式scope排除；runtime test/summary文件见scope report |
| CLI | 12个受影响binary 118/118通过；后续8/8 skill及1/1 symlink，均各绑定实际输入，不相加称全仓PASS |
| schema定向负控 | 正控2/2；只在私有副本禁表形状比较后2 FAIL，已正常编译，原文`schema-control.log`/`schema-negative.log` |
| 提交前观察定向负控 | 正控1/1；仅注入revision+1违规写后1 FAIL，精确全表不变断言失败，原文`observation-control.log`/`observation-negative.log` |
| CLI叶软链定向负控 | 正控1/1；只在私有副本open_regular_at跟随stat并移除NOFOLLOW后1 FAIL，原文`cli-symlink-control.log`/`cli-symlink-negative.log` |

三类负控的命令、退出码与原文SHA为`negative-controls.json`；试验只在独立镜像和临时Home，源文件复原，不涉及真实用户管理根，不是完整mutants运行。

默认非测试支持源码的52文件去测试AST前后相同，token SHA256 `8cdf7bcd5e65df18148fd00b2f5b37dca628f7e7b84851f3076b9066f71e0461`。7个快照、44项fixture/config/依赖字节不变；验证器、实际输入和指标在`integrity.json`、`fingerprint-command.json`、`line-metrics.json`。仅testkit feature支持API收口/公开已有构造器有变化，不计默认生产行为变化。

717条原test identity的`coverage-map.json`逐条绑定保留/退休/合并，31消失名字均有真实当前replacement；2新名字为参数化改名与schema2合法控制。三份scope dispositions保存case/oracle/window、原/现函数或hash，最终独立Reviewer全文核验，不只按测试数判断覆盖。37个Rust文件净少896行，仍保留688个入口和5个compile_fail；统计包含新shared支持文件且不含文档。

完整冻结候选门禁全部exit0。命令/环境/退出码/原文位于`gates/results.json`及逐命令stdout/stderr；离线Cargo、RUSTC_WRAPPER空、build jobs2、Nextest0.9.146/threads2、独立target。deny用原公告缓存独立db-path重新执行，不声称在线刷新。新src/test/支持文件纳入输入闭包，不沿用717原run作为当前688运行证明。

独立审查三scope“通过”，原文`/private/tmp/sheltie-t40-independent-review.md`；21条legacy注记与checker原字节另核。当前改动未提交；普通门禁与本次定向负控不扩为完整M1/变异/Host/发布PASS，既有SK01/SK02和Linux/T16/T17限制不变。


T40最终Nextest run `8c8a31ba-5bb2-4b0e-a679-13c4e4ad63f4`：44 binaries，688/688、0 skipped、1 slow；core205/runtime307/CLI176。测试阶段162.308秒，全命令178.959秒；core doctest5/5。fmt/check/Clippy、core-doc、离线deny、MSRV1.85 locked、docs/specs/core-vocab/tests/skill与dist plan全部exit0。

171项构建/测试/配置/fixture输入清单SHA256 `678e58742b852744f015e08f0d494dbb2f64d239a5c3622b37da0af67ddb23b5`，收尾无字节漂移。独立审查原文SHA256 `e52572428afe7215218ea5a3e6435df0290496ca5576ec21196a4189e4b65379`；处置与真实语义审查通过不代替实际普通门禁，两类结果分别保存。最终docs/specs/tests与 `scripts/check-task.sh C002-T40 4b86279 --staged` 均exit0，完整收尾结果为 `final-checks.json`；原文为本机临时证据。

单次重型add限额测试仍69.847秒，和T39的69.877秒接近；总测试阶段也接近T39的166.013秒。输入和case集合已变，未经重复受控基线不将差异解释为可靠提速。本次结果是测试维护代码减少、覆盖集中且弱判定修正，不承诺执行耗时大幅下降。


### C002-T40 提交授权与证据保存

本任务的上述“未提交/不自动提交”是此前工作区交接事实。用户随后明确授权提交；本提交保留该任务实际源字节和对应原运行，原文已按README索引压缩归档，不以当前最终运行替代早期候选证据。

恢复方式：从本package运行 `tar -xzf evidence/submissions/C002-T40.tar.gz -C <临时目录>`；根目录为 `C002-T40/`，`contents.json`逐文件给出SHA256与大小。原文中的本机绝对路径保留为历史来源，归档内的相对文件及该清单是提交后的恢复入口。此次归档逐项核字节后才纳入提交，不声称仅hash就是备份。


### 分阶段提交的构建缓存复核

首次T39钩子正常执行，但共享target因历史文件mtime早于既有产物而复用旧core/runtime，选择714项，12项已运行中1项失败、702未运行；该失败不记PASS，原输出保留会话，结构化原状态收进T40档案的submission目录。重建文件清单及mtime处理已修正；各阶段采用独立target，T38补核699/699，T39钩子717/717，零跳过，源字节与166/169原清单相同。原门禁、该失败、后续fresh结果分别记录，不混为一次run。T40钩子使用单独target，并核171项源输入，不复用错误产物。

## C002-T16 当前宿主回归

2026-10-02 用户要求继续 C002；开工工作区干净，源码候选 `82c6c55159592db5cdb485549e933960863de185`。未改 Rust 生产代码、测试或 Cargo 版本；后续 F16-01 修复源码 Workbook 三个文件并升版，单列其输入与验证。隔离目录 `/private/tmp/sheltie-c002-t16-82c6c55`；`candidate.json` 保存原始 tracked 文件 SHA256、平台与 Rust 版本，`commands/` 每次调用分别保存 argv、预生成 request-id、stdout、stderr、退出码和耗时。本轮准备原文归档为 `evidence/submissions/C002-T16-preparation.tar.gz`，内容与限制见下文；可恢复实际输出，不以摘要替代日志。

### 构建与工程门禁

macOS `aarch64-apple-darwin`，Rust `1.98.1`；release/default-features 二进制通过 `cargo build --release --locked -p sheltie-cli --message-format=json` 构建，从 Cargo 的 `executable` 获取路径，复制到 `delivery/sheltie-bin`。SHA256 为 `eba587d3a6e84578317bd5a0c0b6f677e6632a54924ae06891f86dd92095752d`；实际版本 `sheltie 0.1.0`，不是具有 v0.2.0 版本身份的 rc。`self install` 在独立 home 完成，已装 bytes 与该二进制相同，schema 为 2。

自包含 skill 通过 `scripts/skill-delivery.sh pack/verify/tar`；`delivery/sheltie-skill.tar.gz` SHA256 为 `d1aa737e2bbf1a6742fdad275572c2a41257506673034884c5cb508d314be06a`，SKILL.md 为 `a7ee7b9e4d4b0d89576d49a3b827fe4cab169fbd2b27a6679e1eb582062146b4`。三份交付文件的逐字节摘要见 `delivery/manifest.json`。本次手动读取并执行交付规则，未向宿主安装或验证新会话自动发现。

门禁使用 `RUSTC_WRAPPER=`、`CARGO_NET_OFFLINE=true`、`CARGO_BUILD_JOBS=2`、`NEXTEST_TEST_THREADS=2`、独立 target `/private/tmp/sheltie-simplify-t38-target`。原文及每条命令的 hash/退出码在 `gates/results.json`。默认 Nextest `0.9.140` 不满足配置，首次退出 92，未执行测试；保留该记录，改用已有 `target/t31-validation/tools/cargo-nextest` `0.9.146` 后执行完整运行，未绕过版本检查。

| 验证 | 实际结果 |
| --- | --- |
| fmt；check/clippy 全 targets/features、locked | exit 0 |
| Nextest 全 features、locked | 688/688，0 skipped，164.367 秒，exit 0 |
| core doctest 全 features、locked | 5/5 compile-fail，exit 0 |
| deny | 同本地公告缓存、独立配置 db-path，offline exit 0；未在线刷新 |
| MSRV `1.85.0` workspace/all-targets/all-features/locked | exit 0 |
| dist plan JSON | exit 0；生成的是当前 `0.1.0` 的计划，不是四平台资产构建 |
| docs/specs/core-vocab/tests/skill | exit 0；文档变更后另行复核 |

本轮没有重跑完整变异或 Linux。旧 M1、SK01/SK02、699/717 的闭包与结论不改写为本次运行；即使源字节相同，也不推定历史缺失验证已通过。

### 当前会话场景

下表与准备档案固定于收到人工决定前。后续已授权执行另记在「人工决定后的继续执行」，不覆盖此快照及其独立复核。

Host 为当前 Codex 会话，coordinator `Codex /root`；worker 使用真实 `collaboration.spawn_agent`，只得到 brief 与绑定文件路径。身份、职责和返回结果可从本会话工具记录复核。应用版本/build 未取得；usage 没有可用计量，记 `null`，不记 0。`host.json` 保存这些缺口；逐 CLI/工程耗时与 Work stats 不当作模型成本或质量评分。

| 场景 | 实际观察 | 未完成项 |
| --- | --- | --- |
| 发现与输入 | missing home/Workbook 返回 NOT_FOUND；四份 Workbook 经 show 获取 start_inputs，已给输入直接用于 start | 未指定 Workbook、缺输入的真实人机往返尚未验证；CLI 错误返回不等于交互通过 |
| two-step | outline/summary 由不同真实 worker 生成并提交，Work succeeded；摘要 395 字、四段覆盖提纲，stats total 113 秒 | 宿主 usage、真实重开会话缺失 |
| 请求与错误恢复 | 完成后重放旧 outline submit，除 replayed 外响应与原文完全相同，历史 revision 3；当前终态不回退；新 begin 返回 WORK_TERMINAL，状态逐字段不变 | 不从历史 next 续接；真实会话关闭/重开未执行 |
| gated-release | notes worker 生成演练发布说明，提交后 blocked(gate)，next 只有 approve/cancel；已向用户展示产物并请求明确批准 | 未收到批准，未 approve 或执行 archive；不是对外发布 |
| article-review | writer 首稿，未参与写作的 reviewer 按冻结 checklist 判通过；人工 publish Attempt 已开始 | 本次未走 back；人工 final 文件未生成，不能代人完成 |
| spec-dev | 独立临时 Git 项目，原始基线 `2f88d59b0d214967b64b285e96d764f26bc71c30`；worker 生成 spec/plan/tasks，人工 plan-review Attempt 已开始 | 未收到人工审核决定/条件，未重规划或实施任务；T04 纯文档属于 task-rules §4 的明确例外，已说明检查方式 |
| Finder 与完整性 | 真实 Finder 新窗口打开 two-step 冻结 Workbook，显示 flows/instructions/workbook.toml 和 not editable；前后四文件 hash/mode 相同，无 `.DS_Store`；installed Workbook verify 全 ok | 一次访问不代表所有 Finder 操作；UI 原文在本会话工具响应，未导出截图 |

`replay-oracle.json` 以原响应字节解析后的完整结构为期望，`finder-before/after.json` 保存访问前后的实际字节摘要与权限。`artifact-oracle.json` 用 Python hashlib/实际长度/stat 独立核对 8 个已提交 ArtifactRef 和 0444 权限，以及安装二进制的 bytes。没有用 Sheltie helper 计算期望，没有改封存产物或真实用户 home。

初次交接误将 T04 纯文档任务判为违规；独立 Reviewer 核实冻结 task-rules §4 明确允许纯文档任务写检查方式，§9 要求文档收尾，T04 已满足。已修正本记录和发给用户的审核问题；不把此误判记为产品 finding 或要求 planner 修复。归档中的早期文档 patch 仅保留纠正前的历史输入，不作为最终结论。独立报告 `independent-review.md` 同时保留初次「需修改」和纠正后有限「通过」；范围见 [review.md](review.md#t16-部分证据与交接复核)。

归档 SHA256 以 [README](README.md) 的 T16 行为准。T16 档案顶层直接包含 `candidate.json`、`commands/`、`gates/`、`home/`、`project/` 与交付文本，和 T38–T40 的布局不同。恢复到新目录只用于核证据；Store/任务书保留原绝对路径，不能把恢复目录直接作为新的管理根运行。二进制保留在原临时目录，未收进此证据档案。恢复示例：

```bash
t16_restore_dir="$(mktemp -d /private/tmp/sheltie-t16-restore.XXXXXX)"
tar -xzf specs/changes/active/C002-v0.2.0-reliability/evidence/submissions/C002-T16-preparation.tar.gz -C "$t16_restore_dir"
```

### 准备阶段交接与发布边界（初次归档）

T16 保持 `doing`，T17 保持 `not_run`。逐 Work 当前查询和人工动作入口见 [progress.md](progress.md#t16-当前交接)。门槛批准、人工定稿、方案修改与条件需要真实用户输入；不能用当前「继续 C002」补造对尚未展示产物的批准。真实关会话再续接、Host 版本与至少两个 spec-dev 已验证任务仍须补证据。审批原文尚不存在，未记 PASS。

本 package 文档变更后，docs/specs 与 `git diff --check` 均 exit 0；171 项生产、测试、fixture、合同交付及门禁输入保持候选字节不变，未提交路径均在 T16 白名单内。`scripts/check-task.sh C002-T16 82c6c55` 实际 exit 1，原因为 T16 状态不是 done；这项完成门禁尚未通过，未放宽 checker 或提交。具体输入路径及命令原文以 `gates/documentation-results.json` 为准。

T17 的本机候选、skill 与 quality 原文已可复核；dist plan 仅列四平台的 `0.1.0` 资产名，其生成成功不证明资产存在。四平台 v0.2.0 manifest/checksum/同 SHA quality、指定版本远端更新与 rollback 均未运行；版本、CHANGELOG、release record 和发布生命周期未更改。完成 T16 后固定最终版本候选，再准备 release record 与 CHANGELOG，并在具体发布材料齐备后申请单独发布授权。

### 人工决定后的继续执行

用户随后明确选择当前 Codex 会话并批准本地演练归档。方案问题先收到「修改方案并附上述条件」，再对替代问题明确答复「通过方案，附上述条件」；以后者为准批准当前四任务方案，条件为「错误信息不得回显输入正文」，前一答复记录为已覆盖，不算一次真实重规划。用户回复「我会手动复制并告知」只表示待执行，不能当成人工定稿完成。

`human-decisions.json` 保存各 questionItemId、真实答复、覆盖关系和记录者。Codex 按实际决定机械写方案版本摘要、原字节镜像并调用 CLI，决定来自用户；CLI 主体仍记 `shushu`，不声称独立真人认证。

`gated-release` 实际 approve 后派 archive worker；`archive-copy-oracle.json` 独立核正文与已批准 notes 逐字节一致。Work succeeded，revision 6，stats approvals=1、blocked_count=1；完整请求/响应在 `commands/approve-notes`、`begin-archive`、`submit-archive`、`gated-final-status/stats`。

`spec-dev` 已提交实际审核决定和两份被审镜像。骨架提交 `92e2d3f3fb14d077aac3cba9d10ec2b7df53f540`，原始基线不变；T01 候选 `65c9163a4ef2b45892f3cc2f87e2d16ea6eedc3c`、T02 候选 `6773f932765895735a5786220653440afcba2d8f` 均获未参与实现的独立 verifier 实跑、范围与审批核验通过。任务测试分别 4/4、5/5；完整 unittest 均收集 21 项，其中未来任务分别 17/12 项仍禁用，不冒充全产品通过。两份 report 保存原始基线和累计前缀，仅追加 T01/T02，原文在 `task-runs/implement-1/2`、`verify-1/2`。当前已进入 escalate 等实际重规划决定；CLI 条件尚待后续任务验收。

第一份文章由用户实际手动复制并回复「已复制」，`manual-final-oracle.json` 独立核普通文件、单链接及与已审文章逐字节相同，提交后 Work succeeded，revision 7。此前「我会手动复制并告知」未被提前算作完成。

另开原始 article-review Workbook 的受控 back Work `2026-10-02-005-t16-back`：真实 writer 首稿故意只有 186 汉字、一段、一个例子；独立 reviewer 按冻结清单判不通过。协调者选择 back，新的冷读 writer 只从绑定 topic/review 逐条修订，另一个 reviewer 判通过。`back-oracle.json` 独立核 draft#2 实际绑定 review#1 路径及摘要、stats 的 review(back)×1、原 Workbook 字节不变。本用例证明真实 worker 的受控回环，不用于估计自然写作质量；当前复审已提交但第二份人工 final 未完成，未声称整 Work 成功。

宿主元数据补录为应用显示名 ChatGPT、运行 bundle `com.openai.codex`、版本 `26.928.31416`、build `12553`。`host-app-metadata.json` 保存本机 Info.plist hash 与可见应用清单来源；读取本宿主窗口被工具以 safety reasons 拒绝，未绕过，窗口匹配及真实重开仍未核。元数据不证明自动发现 skill、模型 usage 或完整 Host 资格。

准备档案最终收尾复核另见 `evidence/submissions/C002-T16-preparation.audit.json`；其 SHA 和 364 文件核验限定于收到决定前的固定快照，临时管理根随后正常前进不改写它。后续阶段的独立报告见 `continuation-review.md`，有限结论为「通过」；只核本段实际决定、产物、两任务和受控回环，不替代完整 T16 验收。

### 实际重规划与完整开发闭环

用户随后明确同意 T03 读取、T04 CLI/README 的重排，再批准展示的具体新版并保留原条件。真实 escalation、两版 decision 与审核镜像分别封存；`replan-oracle.json` 独立核整体基线行和 T01/T02 原始两行逐字一致。冷读 planner 核递归历史来源后生成新方案，未取当前 HEAD 重设基线。初稿把已给条件误列为开放问题，经引用绑定审批明文纠正后才封存，未重复问用户或改旧规格。

新骨架 `ba56d3fc3a25a9ca170183fd2315beb0ff37db5b` 保留九项既有测试与实现，新增五读取正反例、原十二 CLI 测试仅重标 T04。T03 `71821061f32d9c6496c2d621e673527bc785e4f6`、T04 `0ff54fa35002477c033f306e46089cfda738e3d1` 获独立 verify；最终四行累计事实完整、26/26 零 skip、compileall 通过。T04 红阶段真实为十一行为红、一既有标准库静态绿；未制造失败或改断言。

`task-runs/verify-4/` 与 `whole-review/` 保存实际空 venv（without-pip、禁止系统 site-packages）、隔离 Python 路径、继承执行上下文的 network-disabled/seatbelt 标记、真实外联 EPERM、CLI 全部正反例、chmod0 后 PermissionError 前置与原文件 bytes。私有 sandbox_check 返回值无公开语义，只作辅助观察；nested sandbox_apply exit71 不计成功。环境结论限定这些实际 Python 执行闭包，不是整台 Host、任意代码或全协议的安全资格。

整体 Reviewer 未参与实现，从原始基线到当前 HEAD 核四文件完整 diff、十条原始验收及审批条件，另独立跑 26/26、compileall、空 venv CLI 成功失败；结论通过，零阻断、零建议。未安装变异工具，因此此项目 mutation 为 not_run；不补旧 M1 215 项或最终 Spec 批准。交付说明保存六提交与遗留；反思仅提一条有证据的 F16-01，不凑三条。开发 Work 当前 blocked(gate: retro)、revision 39，尚未批准；stats total_seconds=6541 是 Work 跨度，含人和协调者等待，不是模型用量或成本。

第二份文章已由用户实际复制并告知，`back-manual-final-oracle.json` 核与复审稿逐字节相同，提交后 back Work succeeded、revision 11。此受控回环完整，仍与普通首稿质量分开记录。

### F16-01 修复闭包

按已有 WorkLayout 合同把 retro 报告定位改成 `attempts/<node>/occurrence-<NNN>/attempt-<NNN>/outputs/`，两个维度独立补零；Workbook 升为 0.2.1，README 记录，先扩本任务三文件白名单。真实 add/show/verify 新版本通过；`retro-layout-fix/oracle.json` 核新的真实 BEGIN brief 逐字包含新说明，旧 0.2.0 冻结指令 SHA256 `3bf38deb3c77162f7fea8275fffa992fd2412bb47708d4546ef366484dd17407` 与原候选字节相同。该独立管理根中的合成产物只证明文案 producer，不算模型或真人回归。

`fix-gates/input-files.json` 与 source-worktree.patch 固定新 fixture 闭包，`fix-gates/results.json` 保存实际 argv/退出码/原文 hash；fmt/check/clippy/Nextest/doctest/deny/MSRV/dist/docs/specs/core-vocab/tests/skill 全 exit0。Nextest 688/688、零跳过、167.925秒；doctest 5/5。不是把旧运行改写为新版本 PASS。独立最终阶段报告 `final-review.md` 复核 F16 原因、白名单与新旧 bytes/brief、真实模型/人回归边界；初次报告未预判当时尚运行的门禁，门禁完成后另核实际结果。

T16 仍 doing，T17 仍 not_run。新的 Workbook/输入缺失真实交互已由用户实际选择 `two-step / default` 并给出 `topic=本地工作流如何跨会话续接`；得到数据后从 show 的 start_inputs 直接 start，不反复问已给信息、不以失败 start 探 key。Work `2026-10-02-006-t16-续接` revision 2，outline worker 已写五点提纲但未 submit，留作真实续接点；`resume-checkpoint.json` 独立核实际 topic、running 状态、未提交文件 bytes/hash，并要求重开后以新 CLI status 为准。

上述是重开前的阶段交接；本轮真实续接见下节。完成状态门禁仍不可放行，不因开发示例完成或修复闭环而标整个 C002 完成。源码修复及所有未提交记录仅在 T16 修订白名单内；最终 gate、T16 状态及提交留待具体批准与独立复核完成后处理。

### 真实重开后的续接

用户本轮直接陈述「我已真实关闭并重开会话」。本会话先读取 progress 与交付 `delivery/sheltie/SKILL.md`，再通过原候选 CLI 的 `work list/status` 发现 Work006 和 Work003；没有重放历史响应取 next。该陈述和来源保存在 `reopened-session/session-source.json`，不声称独立窗口或进程认证、skill 自动发现或模型 usage 可用。

`commands/reopened-work006-status` 确认 outline#1.0 running；协调者读取历史 BEGIN 返回的任务书与输出定位，独立核原提纲 983 bytes、SHA256 `db1dd9ca5730f74dd2329e6afd6fc0ebb509c3e251c2fb48798d90c3a19c1ca1` 不变，再按当前 next 提交。新 BEGIN 返回 summary 的真实任务书和绑定输入，`Codex /root/t16_resume_summary` worker 按它生成 417 字（含标点，不含空白）、五段、1260 bytes 的摘要，顺序覆盖五点且未新增观点；协调者核内容和输入 hash 后提交。摘要 SHA256 为 `e1c7b576cf0cdec6657e0f6fc0c7f870b11d531681e9bd5fd05147201ab5c752`，独立计算原文在 `reopened-session/summary-oracle.json`。

三个写操作均先生成并告知 request-id，再保存实际 argv 与请求，分别为 `473e8f1b-a9bf-4e68-aeee-d04d550db4d6`、`07e1121d-54f7-4680-8fbc-57aaa3b072a5`、`119a4efd-83e0-4dfb-8b86-86f1b30056aa`。`commands/reopened-submit-outline`、`commands/reopened-begin-summary`、`commands/reopened-submit-summary`，其 stdout/stderr/result 保存原响应、退出码与耗时；最终 `reopened-work006-final-status` 为 succeeded、next 为空，submit revision 5。stats total_seconds=919、outline=854、summary=65 是实际 Work/Attempt 跨度，含会话关闭及等待，不是模型用量或成本。

`reopened-session/artifact-oracle.json` 独立核当前 39 个唯一 ArtifactRef 的完整字节、长度、SHA256、单链接与 0444 权限，均一致。Work003 新查询仍 blocked(gate: retro)，保持 revision 39；本轮已读并展示 delivery/lessons，再请求最终门槛的具体用户批准，未提前调用 gate。T16 保持 doing，T17 保持 not_run，usage 缺失和旧 M1/SK/Linux 限制保留。

独立 Reviewer `Codex /root/t16_reopen_review` 未参与本轮运行与修改，当前阶段结论通过，完整 T16 资格仍阻断于具体批准及实际 gate。`reopened-session/checks.json` 保存 docs/specs/diff-check exit 0 与正式 `scripts/check-task.sh C002-T16 82c6c55` exit 1（状态不是 done）的实际 argv、原文和摘要；未改 checker。重开续接档案的 69 个普通文件逐字回读一致，文件清单与来源只保存证据，不作为第二套状态；路径与 SHA256 见 [README](README.md)。该快照保留批准前事实，后续批准应另记真实原话和 CLI 响应，不改旧快照。

### 最终 retro 批准

展示实际 delivery/lessons 后，用户答复「批准 Work003 最终 retro gate」；`reopened-session/work003-gate-approval.json` 保存题目、答复、questionItemId `call_9adT9sBMUpCj2TLfgsz6HIRz/0` 及仅结束本地演练的范围。协调者重新查询当前 status，再以已告知的 request-id `43e1101f-7a1b-4862-b776-2c505c9e15fb` 调用 gate approve。`commands/reopened-work003-before-approve`、`commands/reopened-work003-approve-retro`、`commands/reopened-work003-final-status`、`commands/reopened-work003-final-stats`，四项均 exit 0；approve revision 40，CLI 主体 shushu、Work succeeded、next 为空，不声称独立真人认证。最终 stats approvals=1、blocked_count=1、total_seconds=8896；时长包括 gate 等待，不作模型成本。

批准前档案与独立报告保持原样；最终批准、当前终态与 T16 完成资格另作补核。用户本轮批准不授权提交、对外发布、安装宿主、清理管理根或补跑历史验证；T17 保持 not_run，已有 M1/SK/Linux/完整变异及 usage 限制不改。

最终独立报告 `reopened-session/final-review.md` 结论通过，限定 plan §5 的本机实际场景；Work003 批准前阻断已消除，T16 收尾为 done。最终 `work list` 六个 Work 均 succeeded；未走的 fix 分支不作为新的 next 或失败。`fix-gate-input-recheck.json` 独立核修复后门禁的 277 个非 C002 package 输入 SHA256 全匹配，含 Rust、测试、依赖、配置、Workbook、skill 和合同，不因纯会话续接重跑完整工程测试。usage=null、引擎 0.1.0 身份、手动 skill、Host 窗口访问限制与旧 M1/SK/Linux 缺失仍保留；本轮不提交或发布。

更新状态后的 `reopened-session/final-checks.json` 保存 docs/specs/diff-check 和正式 `scripts/check-task.sh C002-T16 82c6c55` 全部 exit 0；原批准前 exit 1 不改。没有暂存提交，因此此处是工作区任务门禁，不冒充 `--staged` 或提交成功。

最终阶段档案包含 104 个普通文件，逐字回读一致；SHA256 与入口见 [README](README.md)。其原文保留批准前/后两份独立报告、全部 reopened CLI、实际答复及最终门禁，不覆盖原待批准快照。临时管理根、演练项目和二进制保留用于只读核验，本轮不执行 purge。

### T16 提交收尾

用户随后要求继续执行直到 C002 全部完成，本地任务按计划进入提交循环。T16 的代码、测试、依赖、配置、Workbook、skill 与合同输入仍匹配修复后原运行；复用 `fix-gates/results.json` 的 fmt/check/Clippy、688/688、5/5 及附加工程门禁，依据是最终独立 audit 核实的 277 项实际 SHA256，而不是沿用旧 HEAD 的测试标签。提交前重新运行文档、规格、diff 与 staged task gate；实际结果及提交由下次交接与 Git trailer 定位。既有「未提交」保留为各阶段时点事实，不改写原快照。

## C002-T17 候选准备

T16 已提交 `785552be4fddc8fbbbcbffef7bcf69e65588ff8f`，其历史0.1.0身份与原输入保持。T17 独立目录 `/private/tmp/sheltie-c002-t17-785552b` 保存候选 patch、tracked 输入SHA256、逐命令 argv/环境/UTC/stdout/stderr/退出码与耗时；Cargo 升0.2.0，lock仅三个本地crate版本变化，无依赖升级。

PR 从plan改upload；六个job统一PR head SHA或tag SHA，不混用merge SHA。publishing仅tag push，announce仍需同候选quality与host成功，非发布PR只构建。quality逐门禁原文和工具/源输入以quality-<SHA>独立资产保存，不混入发布assets。完整真实plan给出四平台矩阵；本机实际生成macOS aarch64 tar.xz，尚无其它三平台实物，不把plan当构建PASS。

本地固定输入执行 fmt/check/Clippy、Nextest 700/700（165.357秒，零skip）、core doctest 5/5、MSRV1.85、offline deny及docs/specs/skill/core-vocab/tests均exit0。实际argv与输出摘要见commands；依赖公告复用同本地缓存，未在线刷新。治理测试原10项保留，新增12个独立正反例；旧checker的历史表/零active缺陷及独立审查发现的空字段/列数绕过保留红阶段，最终focused22/22。原文另存 `/private/tmp/sheltie-t17-governance-p2/`，提交证据时一并归档。

本机release binary版本0.2.0/schema2，actualCargoJSON定位后安装到独立home，安装bytes核对。真实dist包sha256 `d55acd1acbc57df9362f2272b686a9ae66d4241033927aadc5a0abfac9c4771d` 用于本地完整manifest镜像的指定版本update/rollback，旧T16候选→0.2.0→旧候选bytes一致、Store不变；这是同schema本地演练，不是远端或正式v0.1.0旧schema更新。

另下载已发布v0.1.0本机包，sha256 `c37ab135f7388910b854cd37a70240da1cfaef23cd313c1641112e0767fd64ad` 与其真实manifest相同。旧self install对不存在home失败，记录原文后按其既有前置条件预建空fixture目录，仅由旧CLI生成schema1。新binary拒绝该Store、旧发布binary拒绝新schema2 Store，均为STORE_SCHEMA_MISMATCH，既有Store字节不变。早期oracle误把允许创建的WAL/SHM控制文件当业务改动，保留失败并按D-039纠正；新增控制文件只为零字节WAL/共享内存，不改旧主库或迁移。`schema-boundary-oracle.json`保存初始hash、实际反例与边界。未操作真实用户home。

独立Reviewer `Codex /root/t16_reopen_review` 未参与T17改动，candidate-review.md保存Standards/Spec及F17-01原红例与修复后复核，候选范围通过；code-simplifier只读认为无需结构性改写，清理一段已失实的CI注释。四平台CI、远端指定版本更新、最终外部发布及生命周期均尚未执行，T17 doing，不新增完整变异、SK01/SK02或全Host PASS。release record草稿仅存临时目录，不预登记released。

本地准备原文已封存155文件逐字回读，档案路径与SHA256见README。完整700测试之后只改package进度与候选CHANGELOG说明；治理测试由fixture自行生成CHANGELOG，不读其live正文。相关docs/specs重新通过，实际dist重新打包说明文件；原更新演练仍绑定原包，不把后一次pack的SHA替换旧CLI日志。候选提交仅固定上述已验证输入供非发布CI，正式task gate仍因doing失败，不冒充任务完成。

### 首次真实非发布CI与Linux修复

用户本轮答复「批准，继续完成」，范围为新验证分支与draft PR的非发布四平台CI。SSH原入口传输等待；HTTPS被全局insteadOf改写，后在仅本次忽略全局设置后完成上传但远端ref更新拒绝。OAuth缺workflow scope与该症状相符，完整失败stderr未单独封存，不把推断写成精确远端错误。已有samuelj1519 SSH身份通过官方443入口严格主机密钥核验，最终push exit0，210.05秒；原argv/结果/完整progress在ci/ssh443-push。无全局配置修改或新权限申请。首次PR调用早于push完成被GraphQL拒绝，无PR生成；确认push成功后才创建[draft PR #1](https://github.com/samuelj1519/sheltie/pull/1)。远端head精确7f42e3b。

Release CI `37029608597`与build CI `37029608698`均pull_request事件，head SHA `7f42e3b294473ac72e6e472480431db397670806`。Release两个Mac包success，两个Linux包、quality失败，global/host/announce跳过；build的Linux/MSRV同错。原status.json与failed.log保存实际job结果，不把skip当PASS。失败根因是fsx.rs2775/3178/3206把u16传入Linux RawMode=u32；Mac原700无法证明此目标编译通过。

先扩T17的fsx白名单再修三处目标推断转换，保留原Mode::from_raw_mode、S_IFMT/bitflag处理和所有API/权限值/句柄/同步次序。锁定rustix1.1.4 primary源码核Darwin u16、Linux u32；不改为另一constructor或新cfg兼容层。独立报告linux-mode-review.md与worker的Mac fmt/check/Clippy/MSRV原文保存，内容通过；新Linux入口及新完整候选结果须实际执行后才记录。外部发布授权仍未取得，T17 doing；首次失败和旧M1/SK/usage边界保持。

三处修复后新完整Mac运行700/700，零skip，Nextest165.194秒；core doctest5/5，docs/specs exit0。worker的fmt/check/Clippy/MSRV原argv/env/hash亦被独立核实，Rust代码输入sha a2d1efce26f523cb1838327fb40ce48e4f2b81251a4114f13541b94d4e64f83a。原失败与修复闭包37文件另档案封存，入口和摘要见README；候选将更新同一获批验证分支，继续非发布CI，当前没有发布授权或tag。

### 本版本 Linux 排除授权

新候选33b08bc的Release CI37032589688已经实际构建四平台/global，MSRV与Mac build-rust成功；Linuxquality/build-rust在fsx.rs1307/1381/1447/2373因同类型st_dev as u64触发Clippy失败。两个失败log原样保存，不以实际四个包存在替代quality。用户随后明确「此版本可以忽略Linux平台」，本版收尾范围改为两macOS架构；Linux记excluded_by_user并保留原FAIL，不继续为本版改Linux lint，也不上传本版Linux包。

Cargo dist当前目标和发布质量/工程矩阵按本版范围改为macOS，原全平台Rust源与三处已完成compile修复保留，不加按版本分支或永久兼容层。新候选必须取得同SHA的两个Mac包/manifest/checksum及原生Mac完整quality/MSRV，才能申请发布授权与完成T17。该例外仅来自本轮实际用户陈述，来源与文字保存于ci/linux-exclusion-authorization.json；不回写旧CI或M1结果。

Mac限定输入的新完整运行700/700、零skip、165.985秒；doctest5/5、fmt/check/Clippy/MSRV/docs/specs全exit0，真实distplan只含aarch64/x86_64 Apple两目标。独立报告macos-scope-review.md核scope/新旧失败/接线及现README说明，限定通过。43文件新档案摘要见README，当前T17仍doing；用户的Linux排除不是外部发布批准。

### 最终单平台范围

用户进一步明确「可以仅发布 macOS aarch64。其余平台以后有需求再增加」，本版只保留aarch64-apple-darwin；其他平台excluded_by_user，原两次LinuxFAIL和曾成功构建的其他包仍留历史，不混入最终发布。Cargo dist与build矩阵各移除Intel目标，其余同SHA质量/MSRV/发布guards保持。Rust生产、测试、依赖、特性、锁和fsx修复均不变；新目标plan与真实consumer治理tests单独核验，复用此前同Rust输入700/700和5/5，不冒充新CI同SHA质量或实物。

本轮「可以仅发布 macOS aarch64」同时明确了该版本单平台发布许可，保存于single-platform-authorization.json。当前仍须先完成实物、同SHA质量与独立核验；条件满足后按此已给许可发布，不重复请求相同授权。此前未授权描述保留为其时点事实。

单目标改动不涉及Rust/test/lock/toolchain/features/config，`git diff --quiet 6bb8b5a -- crates Cargo.lock rust-toolchain.toml .config`实际exit0；Cargo只删dist的Intel目标。真实22/22治理consumer和新单目标plan均exit0，复用此前700/700与5/5及工程结果按sameRust输入记录；新候选仍须真实同SHA完整MacCI。范围11文件档案入口/SHA见README，不声称其中含当时尚未生成的独立报告。

## 最终验收闭包

当前Candidate指向真正发布source8455，历史M1 source e3eea899877165f8573befee3774555598ec92bd与上表/原档案仍保留。正式release为 [v0.2.0](https://github.com/samuelj1519/sheltie/releases/tag/v0.2.0)，publishedAt `2026-10-02T17:03:04Z`，target8455；不是draft或prerelease。tag Release CI37037558064与build37037558052同SHA全部success。原生ARM quality实际700/700、零skip、156.541秒，全部完整工程与MSRV1.85通过；不沿用PR或旧M1证明tag实物。

公开aarch64包1779600bytes，SHA256 `fc63c9d0c7132796cdacd050be102ed5501f637f2db0f19b279da6b4744899e4`；binary `a220309ab437a5de52202acb8b8f933370a45969e2027c55964930818e315130`，完整manifest `8540728b9db73a31a74e7f762128bebed90210ecfb2ec24845786db02a02354b`。tag重建包与PR压缩字节不同，重新核公开包/独立checksum/globalsum/manifest/GitHubdigest，binary与PR相同。8个公开资产只含ARM和通用source/installer/skill，skill独立从8455源生成三文件逐字一致、references无deadlink。

真实公开binary fresh install/version为0.2.0/schema2，installed bytes等于公开binary。另用T16 C002 schema2 updater（显示0.1.0，不冒充旧发布schema1）在独立home默认GitHub来源、SHELTIE_RELEASE_BASE未设，明确版本0.2.0真实下载替换；updated bytes=公开binary、prev=原binary、全部Store文件hash不变。新binary调用rollback后恢复原binary、prev移除、Store仍不变。不是旧Store迁移、不是回退schema，未触及真实用户home。

独立发布前/后报告均由未参与执行的Reviewer核全部raw/hash与源码/asset/CLI一致性，产品与发布事实限定通过；原SK/完整安全变异缺失、usage null、手动skill/用户重开陈述与被排除平台不扩为执行PASS。最终执行335文件档案与README摘要逐字一致，正式lifecycle/staged验证和派生audit在收尾后另存。

| Requirement / risk | Mode | Input closure | Command / raw run ID | Result | Evidence |
| --- | --- | --- | --- | --- | --- |
| 本版原生ARM完整质量与MSRV | executed | source 8455aed | Release 37037558064 与 build 37037558052 | PASS | 同SHA quality logs / final job JSON |
| 单平台公开实物、manifest、checksum、skill | executed | tag v0.2.0 source 8455aed | published-artifact-oracle / 独立postpublication-review | PASS | 8公开assets与逐字来源核验 |
| 正式安装及真实远端指定update/rollback | executed | 公开ARMbinary及独立home | commands/published-self 与 remote-* | PASS | CLI原文 / binary及Store before-after SHA |
| T16真实宿主必需场景 | executed | T16闭包785552be | 六Work / 重开与实际批准 / 独立最终audit | PASS | T16固定档案与原范围保留 |
| 旧格式拒绝与原数据保持 | executed | 真实v0.1.0 schema1 / 新schema2独立home | schema-boundary-oracle | PASS | 两端STORE_SCHEMA_MISMATCH及既有字节核验 |
| 授权范围与缺失披露保真 | evidence_review | 本版ARM许可 / M1 SK例外 | 实际用户原话与各阶段独立审查 | PASS | excluded_by_user与原FAIL/not_run保留，不当执行通过 |

最终目录与记录由未参与编写的Reviewer独立复核，26项检查通过。当前41个T/M均done，0active；12个旧tar与3个旧audit只改目录、SHA保持，最终335文件archive逐字一致。实际docs/specs/tests/check-task C002-T17 785552b --staged/git cached diff check全exit0，原argv/raw/hash在lifecycle-gates与派生audit。没有产品/测试/依赖/配置输入改动，复用同source8455正式tag质量700/700与同Rust闭包5/5；不新增staged之外或未知运行的PASS。审计入口见README，源码tag与后续验收提交分离。

## 2026-10-03 恢复完整验收入口

T41 基线 `9ee0f0114a8e407257864175eb354037e38a8451`。按用户恢复所有跳过步骤的授权，移动本 package 为唯一 active；旧 task/M1/release 的限定结论与原结果保持原记录。独立 Reviewer `/root/completion_spec_review` 核 17 份证据、4,383,945 字节与基线逐字一致，见 [保真原文](evidence/completion-20261003/independent-t41-integrity.json)。docs/specs/tests/diff/TOML 独立复核通过；源码与测试未变。C001/C003 的治理入口由当前 docs/specs/tests 复核，未发现另外欠项，不将它们历史注释中的后续 not_run 当自身未实现任务。

T42 的当前完整工程与在线公告仍是独立补验任务；T44 已发现 command/data 的纯严格解码顺序缺口，未获最终 Spec 批准。此恢复动作不解除 SK02 的 215 个执行缺失、不关闭真实用户/平台义务、不发起发布。

## C002-T46 历史 Command/data 解码顺序

基线 `8167165a5a707bbb51a29fd1aa1047f387c9e9d3`，实际输入与所有命令/输出 SHA 见 [最终门禁](evidence/completion-20261003/t46-final-gates.json)。独立 Reviewer 在恢复 Spec 时确认 F-SPEC-01：同 request-id 写重放读取冻结 Workbook 后才完整解码 Command/data；真实 CLI 锁定二进制反例及完整行/字节保留见 [初审](evidence/completion-20261003/independent-spec-review.md)。

新增回归真实红 `e423d628-cba5-40e8-9c7e-0cbe544c66f0`，先报缺冻结副本而未拒绝未知字段；修复后 Command/data 在冻结读前严格解码，复用 CheckedData 给业务资格检查，删无其余 caller 的包装。效果资格合同不变；工程规范只勘误快照读取和效果动作的分层。扩展同 rid 重放/旧 A 阻新 B 四情境后，5/5严格回归、独立5/5与4/4效果资格通过，见 [独立复核](evidence/completion-20261003/independent-t46-review.md)。

完整稳定输入 Nextest `843/843`、零 skip、零 LEAK、2 slow，run `62efeb8a-8680-4a38-9958-8ac5376e800b`（完整 ID 以 [原文](evidence/completion-20261003/t46-final-nextest.txt)为准）；fmt/check/Clippy 和五项 compile-fail doctest 均 exit0，源码哈希前后相同。源码变化后的完整回归不复用修前输入。独立审查当前关键 Spec 合同通过，旧 e3eea89/SK01 的原暂停仍为历史事实；215项/平台/真实价值不随本任务关闭。

首次环境运行是842项中841 PASS/1FAIL、零skip/零LEAK，原文 [nextest](evidence/completion-20261003/nextest.txt)。失败发生于 CLI `.output()` 的 ENOENT，执行者同时在共享 target 构建了另一个 feature 组合的 CLI；该调度冲突保留为验证失败，不改记产品 PASS。后续 stable 输入运行禁止并发改建同一 target。历史 C004–C006 LEAK 因果尚未逐次证明；官方 nextest0.9.145 修复的捕获管道继承问题和当前零LEAK只能作为限定证据，不反写旧run。

## C002-T47 存活条件的独立 oracle

基线 `06e5a91`，仅改selfmgmt的cfg(test)与store/tests测试模块，生产前缀逐字不变，见 [保真记录](evidence/completion-20261003/t47-production-preservation.json)。五条真实Store/WriteSession/child及纯资产路径合同用例：WAL在首次RW初始化已设置且行不变；0持久revision先报损坏而非CAS冲突；真实SQLite非constraint INSERT失败保留准确原因和整事务；单文件名保留/拒绝；恰好stdout/stderr限额时关闭管道、仍等待显式释放的child不得被提前kill。

原分组25/40/40基线通过，11项中10项Missed。新增用例后指定10项全部Caught；独立复核指出夹具释放前的panic清理缺口，改为RAII release、scoped join和有限producer后，新5/5控制及受影响4项再次Caught。前6项Store/asset源码/oracle未变，保留原具体执行范围，不说整组相同闭包重跑。所有Build Success→真实Test Failure、diff、argv、run与选择原文集中在 [oracle档案](evidence/completion-20261003/t47-oracles.tar.gz)，[逐成员校验](evidence/completion-20261003/t47-oracle-archive.json)全部回读一致。旧正式Missed标签不改写。

首次新WAL用例假设手写fixture五表为空而失败；该fixture本来含控制行，已改为构造前后的独立完整行比较。旧失败原文保留，不算产品red。初审与修后报告分别 [初审](evidence/completion-20261003/independent-t47-review.md)、[增量](evidence/completion-20261003/independent-t47-review-after-cleanup.md)；独立Reviewer `/root/oracle_review` 未参与用例设计/编写。修后限定PASS，无剩余T47finding。producer轮询有限但不是严格3秒墙钟保证。

稳定完整工程848/848、零skip/零LEAK，run `fbb2e15c-7dd1-4910-8ab4-93147c66057a`；fmt/check/Clippy及5compile-fail均exit0，见 [完整输入/结果](evidence/completion-20261003/t47-final-gates.json)。此前误指T46输出的验证已停止，四份旧原文按Git对象逐字恢复，校验 [恢复记录](evidence/completion-20261003/t47-misdirected-run-restoration.json)；其partial没有计PASS。当前差异不含任何旧T46原文变更。T47关闭这10项的判定缺口；T43此前另有schema两项与readonly初检一项实际捕获，合计13项，另外202项仍在T43，不扩大到完整变异或全产品验收。

## C002-T42 工具与环境补验

独立 Reviewer `/root/completion_spec_review` 审查通过，见 [原文](evidence/completion-20261003/independent-environment-review.md) 与 [逐项核验](evidence/completion-20261003/independent-environment-audit.json)。原MSRV缺来源的初审保留；已补06e5a91当前MSRV source/argv/env/exit0/hash，再补1218361新测试闭包的实际1.85全targets/features编译，[当前MSRV](evidence/completion-20261003/msrv-121.json)。所有实际命令/输出与影响闭包见 [运行清单](evidence/completion-20261003/environment-results.json)，不是首次nextest失败后的旧流水线PASS。

要求 nextest0.9.145 与 cargo-dist0.32.0 的官方asset digest逐字节校验，仅临时目录使用，没有宿主安装。默认nextest配置无override，现行产品848/848、零skip、零LEAK及5compile-fail保持T47具体run。实际dist plan为6项sheltie-cli资产、没有exporter发布资产；它不执行构建/发布。fresh deny原政策仅db-path变为独立目录，Cargo图/lock/许可输入在T46/T47不变；advisories/bans/licenses/sources全部ok，获取数据库HEAD `f8dee89e1b2f2f1eaf548312df7655fe5202a302`，取证输出结束时点在运行清单，不声称永远最新。

[执行器对照](evidence/completion-20261003/runner-pipe-comparison.json)与 [可恢复原文](evidence/completion-20261003/runner-pipe-probe.tar.gz)：同一4096个纯测试、同target/source、64线程，旧0.9.140出现3leaky，新0.9.145为0leaky，均4096PASS/0skip。该结果复现官方修复的Apple并发捕获管道继承问题，只用于执行器诊断，不加入产品测试数。历史Sheltie每次LEAK因果仍不追溯造证，原unknown/FAIL保持。

T42结束要求版本、编译、在线公告与实际计划的补验；其他平台/跨设备/真实actor/215项属于各自任务，不随本结论关闭。

## C002-T44 SK01 历史最终补审与现行结论

按本次恢复授权，独立 Reviewer `/root/completion_spec_review` 完成固定 `e3eea899877165f8573befee3774555598ec92bd` 的原T34最终Spec增量及原M1整体判断，见 [正式历史结论](evidence/completion-20261003/independent-sk01-historical-final.md)、[原合同/源码输入](evidence/completion-20261003/independent-sk01-historical-inputs.json)。`review_execution_complete=true`，但 `old_candidate_final_spec_approval=false`：旧候选需修改，重复Flow先COMMIT后拒而阻断、嵌套未知字段接受、24h tmp维护缺caller三条违反当时合同。旧R22/R23/R24条件自身静态成立，不足以批准其完整资格基础。

这三项已由T39 `84bafa37` 后续修复，现行写重放顺序另由T46 `06e5a91` 修复；对应已提交原文与实际新版测试分别保存。当前关键合同审查/增量PASS明确属于修复后的输入，不给旧输入造批准。本轮没有重跑旧699或变异，不覆盖SK01原暂停；215项恢复在T43，发布历史仍绑定8455及其既有独审。T44结束的是原缺失审核执行及准确结论，不表示旧候选合格或全部方案完成。

## C002-T48 G01共享oracle与冻结重验

仅追加测试：两条真实CLI有效形状/坏业务绑定与读取先后，窄/完整请求metadata的一致性纯契约，真实Start生成的发布字段纯契约，以及真实开始/执行/提交状态的namespace与required输入纯契约。纯helper期望是手写的Namespace/字段/输入义务，不称全部当前callee已越过前序guard；CLI坏Attempt使用另一真实成功/失败且同历史status的事实，避免被状态拒绝抢先截获。所有合法控制实际通过；CLI比较五表SQLite原值及works/workbooks/pending全树type/mode/bytes（包括retained freeze），控制文件例外单列。

原G01 94测试基线绿色，19对应体全部Missed。初次17体补测19之外load测试在仓库添加导致source_unchanged=false；冻结scratch原17捕获只按原局部输入保留，不作为整体现行PASS。文件oracle补齐、fixture纠正并冻结后，全部同19集合重新执行，19/19 Caught、0missed/timeout、exit0、source_unchanged=true；每项实际失败/argv/diff集中于 [G01档案](evidence/completion-20261003/t48-g01-oracles.tar.gz)，[逐成员回读](evidence/completion-20261003/t48-oracle-archive.json)。G02错误binary过滤exit94/0变异执行也保存，不记成功。

Publication测试初次调用参数不符是编译错误；load首次可选producer绑定required consumer被规则5拒，是错误fixture，不算产品red。最终沿原合法必需Flow，从真实已提交状态造一致的坏绑定纯参数，两个load guard均实际捕获。独立初审/修后意见分别保留；[增量报告](evidence/completion-20261003/independent-t48-review-after-files.md)、[完整核验](evidence/completion-20261003/independent-t48-final-audit.json)限定PASS。

三个runtime生产前缀相同，见 [字节核对](evidence/completion-20261003/t48-production-preservation.json)。完整fmt/check/Clippy/Nextest853/853、零skip/零LEAK，run `aec12b7c-fead-400c-a1e0-eaf665c12648`，5compile-fail均通过，[具体输入/命令](evidence/completion-20261003/t48-final-gates.json)。G01补齐19，加此前13，当前32/215有实际检测，183仍待T43，不改旧正式Missed、SK或全产品状态。本轮用户已明确自动推进/macARM-only，后续执行范围见plan，不等待人动作。


## C002-T49 Workbook 资格与发布观察补验

范围为 G02 的 47 个历史 ID，不改历史 `MissedMutant`。只新增 2 个 CLI、8 个 runtime 共享消费者测试，生产源码及原测试正文保持，见 [字节核对](evidence/completion-20261003/t49-production-preservation.json)。真实 add/replay、A→remove→B、Start、实体 manifest 与 owner/index/audit 观察窗口提供材料；五表、业务树 bytes/mode/inode、原响应资格分别判定。独立 digest/v2 帧来自 storage §5.1，不调用生产摘要 helper。

首批整体 1200 秒预算截断，42 项完成（12 Caught、30 Missed），2 项仅完成 build、Test 未运行，3 项未派发；不是五个 mutation timeout。随后 13 项冻结短样本 exit2：11 Caught、2 Missed，source_unchanged=true。其中 3 项只检测错误诊断来源，8 项检测坏状态被实际接受。后续 oracle 增量及最终批次另记，不把旧 scratch 成果扩成新候选 PASS。

独立准备者对 30 个 Missed 给出 10 项限定静态证明，4 项保留诊断差异；另对索引 785/786 给出 2 项 producer 闭包证明，787/788 仍需动态判据。[首批处分](evidence/completion-20261003/g02-static-disposition.md)、[索引补充](evidence/completion-20261003/g02-index-boundary-static.md) 不修改原 mutation 标签。最终 [精确选择](evidence/completion-20261003/g02-final-selection.json) 分为 35 动态、12 静态，尚待最终执行与未参与准备的 Reviewer 复核。

owner 正例用 SQL 重构 published=false 的原已提交请求，再由真实 writer 重放发布和清理；只允许 published 转 true、pending 维护清理和实际 status-card 原子换 inode，卡的 bytes/mode 保持。false 未发布及五项 immutable 元数据漂移在同一 owner 窗口拒绝；reader 释放后全部五表和业务树不变。Start 复制竞态先在锁内 Workbook 资格之后等待，再在复制源文件句柄观察点同步改同 inode、同长度的 manifest；只允许序号消耗与自有 staging，旧 workbooks/works/requests/audit 和源新字节/inode 保留。没有把人工故障注入写成真实用户动作。

[首次独审](evidence/completion-20261003/t49-independent-review.md) 发现 feature 门控、独立返回事实和同窗口负对照缺项，已修；[增量独审](evidence/completion-20261003/t49-independent-review-final.md) 与 [全部 oracle 独审](evidence/completion-20261003/t49-all-oracles-independent-review.md) 限定通过。冻结 10 测试基线 run `0ab9e52e-288f-4cab-8a1c-89b5967a934d` 全通过。早期 fixture 编译错误、误触同步点而终止自有测试、过早修改源命中 WorkbookTampered、错误 WriteFile 字段及 writer status-card inode 初始期望失败均保留原文，不算产品红。简化仅提取私有 helper、展开 SQL/JSON；helper 上的 Task 注释导致 check-tests 868/863 失败，已移除，863 测试/204 任务卡通过。precomment 完整 863/863、零 skip/LEAK 和 5 compile-fail 的结果绑定原输入；修正后最终工程闭包正在重验，不提前标 T49 done 或 M2 完成。


T49 最终动态批次 A/B/C 分别 12/12、12/12、11/11 Caught，exit0，source/fixture/config 闭包均未漂移，耗时 259.252/179.984/162.935 秒。35 项全部为 Build 成功、目标 Test 失败；11 项为准确诊断来源检测，24 项为行为或原响应资格检测，未以编译失败或 timeout 代替。12 项静态处分单列，其中 4 项仅证明拒绝集合/停止边界，保留错误文本差异。[逐 ID 账本](evidence/completion-20261003/t49-g02-dispositions.json) 的 35+12 恰好覆盖原 G02 47 项，历史 stage1 Missed 原样保留。

[最终工程](evidence/completion-20261003/t49-final-gates.json) source_unchanged=true：fmt/check/Clippy/Nextest/doctest 全部 exit0；run `d9c118ef-a227-4725-bd47-259128d33306` 为 863/863、零 skip/零 LEAK，5 项 compile-fail 通过。[Rust 1.85](evidence/completion-20261003/t49-final-msrv.txt)、[默认特性编译](evidence/completion-20261003/t49-final-default-check.txt)、[测试治理](evidence/completion-20261003/t49-final-check-tests.txt) 及 docs/specs/skill/core-vocab 均通过。依赖/deny 政策未变，T42 的 fresh 输入结论不扩大到新的 DB 时点。

原始运行、失败、脚本、精确选择、diff/log、整理前与最终测试原文集中于 [G02 原文档案](evidence/completion-20261003/t49-g02-oracles.tar.gz)，[逐成员 SHA 与回读](evidence/completion-20261003/t49-oracle-archive.json) 全部一致。当前累计 67/215 项有动态检测、12/215 项有限定静态处分，剩余 136 项继续 T43；不得记成 79 Caught、旧 M1 通过或产品完成。G03 已有独立分组准备，T50 仅列下一任务，尚未实施。

[最终独立验收](evidence/completion-20261003/t49-final-qualification-review.md) 逐原 log 复核全部 35 项目标失败及 12 项限定证明，核原旧 G02 精确集合、190 项 mutation 输入及其与工程输入的共同闭包、生产和旧测试字节及阶段边界，限定 T49/G02 通过；T49 收尾，不批准旧候选、其他组、全部 215 或 C002-M2。

提交钩子首次补齐索引静态报告可读副本的末尾换行，原作者字节保存在档案；[格式化记录](evidence/completion-20261003/t49-review-formatting.json) 校验仅增加一个换行，不改正文或 runtime 输入。
