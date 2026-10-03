# C006 实施计划

状态：`completed`。用户已采用开发全范围，按计划执行，无法执行的义务记录后延期。本方案涉及安全文件原语和原字节接口，采用两阶段交付；小型说明与真实使用按固定手册完成。共同规则见 [方案实施指南](../../../guides/proposal-implementation.md)。

阶段交接时，T01 作者在本 plan 的对应实现任务卡写 `**测试。**` 与已存在的真实测试名，手册链接同一清单；任务标题使用 `### Cnnn-Tnn` 供 check-tests 识别。尚未建立的用例只写“验收用例”，不提前声称测试已经存在。

## 1. 首次阅读与职责

先读 AGENTS/CONTEXT、specs 入口、本 package README/spec/design/validation、[工程规范](../../../engineering.md)与 C004 实际结果合同，再沿下表读源文件。仅需结果指针时不采用。

| 类型 / 入口 | 先理解什么 |
| --- | --- |
| `core/work/result.rs::result_view`、`runtime/service.rs::WorkService::result` | 最终资格、effects_pending、revision 和来源来自同一次可信读取 |
| `ManagedFs::open_regular`、`load.rs` | 原件路径经归属验证；摘要/大小从同一文件句柄取得 |
| `fsx.rs::{verify_tree_at,rename_tree_new,sync_managed_tree}` | 句柄身份、NOREPLACE 和 sync 各自保证与移动后失败 |
| `commands/mod.rs::dispatch`、`output.rs::print` | raw bytes 必须避开 String/JSON 打印；错误只去 stderr |
| 拟 `source.rs`、`target.rs` | 严格 DTO/子进程与安全目标原语，由复杂作者完整实现 |
| 拟 `export.rs`、`main.rs`、`Error`/输出状态 | 简单实现者按已定步骤组织整份复制，不能决定 OS 政策 |

复杂模型承担总体接口、平台原语与完整阶段 oracle；简单模型/初级开发者承担已冻结接口内的编排接线。安全难点在 T01 交付真实实现，不是空骨架。M1/M2 由未参与相应设计、代码、tests/oracle 的复杂模型独立审阅。

## 2. 任务与阶段

| ID | 状态 | Owner | 交付 | 依赖 |
| --- | --- | --- | --- | --- |
| C006-T00 | done | Codex /root；独立 Reviewer | 采用、前版归档与唯一实施入口 | 用户明确采用 |
| C006-T01 | done | 复杂模型架构、原语与测试作者 | 源合同、实际安全原语、窄骨架与阶段测试 | 人采用、C004 结果已验收 |
| C006-T04 | done | 复杂作者/root；独立Reviewer | checkpoint完整字节握手修复及冻结基准更新 | T02实际回归失败 |
| C006-M1 | done | 独立复杂模型 Reviewer | 安全及实现准备审查 | T01 |
| C006-T02 | done | 简单模型 / 初级开发者 | 固定原语内完成整份副本及正式接线 | M1 通过及骨架完整 SHA |
| C006-T05 | done | 复杂作者/later_plans；独立Reviewer | 未发布RC开发目标权威与生命周期治理修复 | T02完成、实验采用前 |
| C006-T03 | done | 手册执行者；复杂模型负责结论 | 临时构建说明、真实副本与最终证据 | T02 |
| C006-T06 | done | 复杂作者/root与later_plans；独立Reviewer | 补前读回与部分清单写入的真实kill窗口 | M2逐项核覆盖 |
| C006-M2 | done | 独立复杂模型 Reviewer | 源到副本、工程与真实结果审阅 | T03 |

阶段 1 入口为 C004 结果候选和副本需求；出口为 primitive green、普通行为回归通过、真实 feature caller 可编译且有意义 red、工具及引擎未激活未完成复制/raw 入口。阶段 2 入口为 M1 骨架完整 SHA、冻结 source/target 接口与命令；出口为全链行为、实际使用及 M2 采用义务完成。本轮复杂作者在 T01 完成安全原语及必要耦合的 library 编排/状态政策，T02 仅公开 raw/export 参数与入口；不为模型分工拆空任务。

T01 原语测试归 T01，交接时实际 green；T01 创建的新完整行为测试归 T02，初始带 `#[ignore = "C006-T02"]`，显式运行得到有意义 red。已有正确 metadata/读路径无需全红。T03 只执行实际手册；确有必要的新 Rust 行为用例时由复杂作者在 T01 提前冻结、归属 T03 并同步其 test_files 和命令；没有则 T03 测试表为空。

### C006-T01：交付可信源与安全目标原语

**Owner / 输入。** 复杂模型；C004 DTO、实际副本需求、当前完整提交、平台与锁定库 API。读取已有体验记录，不另建设平行价值实验。

**范围。** design 真实路径、全部正常/raw caller、拟 `crates/sheltie-export` 的 manifest/source/target/error/output 与阶段骨架、所有阶段测试、Cargo workspace/lock 和 dist 排除；上游 architecture/protocol、指南与工程清单。阶段资产为 `verification/commands.sh`、`experiments/runbook.md`。测试覆盖 runtime 的拟 `tests/result_artifact.rs`、已有 `tests/fs_boundary.rs`，CLI 的拟 `tests/result_artifact.rs`，exporter 的拟 `tests/{source,target,export,cli,crash}.rs` 和混合源码内私有原语测试；fixtures 与新 helpers 也由本阶段创建。

**测试。** `directory_authority_is_validated_before_the_source_binary_can_execute`、`report_error_variants_preserve_stable_status_exit_and_source_diagnostic`、`unconfirmed_report_only_names_a_target_whose_identity_was_confirmed`、`complete_report_is_one_json_line_with_exact_identity_and_no_residual_or_error`、`exporter_version_is_available_without_an_engine_or_management_root`、`source_strictly_reads_metadata_and_preserves_binary_bytes_with_direct_arguments`、`metadata_unknown_fields_identity_order_and_bounds_are_rejected`、`metadata_versions_terminal_binders_and_status_shapes_follow_the_full_contract`、`metadata_accepts_exact_one_mib_and_rejects_one_more_byte`、`receive_rejects_nonzero_exit_digest_size_and_target_writer_errors`、`receive_accepts_exact_32_mib_and_stops_growth_beyond_the_limit`、`source_passes_empty_unicode_and_shell_metacharacter_keys_as_literal_argv`、`receive_stops_a_direct_source_child_instead_of_waiting_for_a_blocked_producer`、`native_directory_publication_preserves_bytes_manifest_and_private_modes`、`atomic_noreplace_refuses_existing_directory_without_changing_competitor`、`target_rejects_links_nonexistent_relative_and_management_overlap_without_writes`、`artifact_requires_safe_leaf_contiguous_index_and_exclusive_single_file`、`target_refuses_injected_leaf_symlink_and_hardlink_without_touching_sentinel`、`parent_or_staging_path_replacement_stops_writes_to_moved_object`、`final_readback_detects_same_size_rewrite_and_preserves_failed_staging`、`publish_requires_all_finished_files_exact_source_mapping_and_no_extra_objects`、`repeated_exports_make_independent_new_directories_without_reusing_old_staging`、`artifact_byte_boundary_accepts_32_mib_and_rejects_one_more_declared_or_written_byte`、`empty_artifact_and_binary_bytes_are_read_back_without_text_conversion`、`target_declared_total_accepts_256_mib_and_rejects_the_next_file_before_creation`、`private_permission_drift_is_rejected_without_chmod_repair_or_publication`、`foreign_staging_writer_is_rejected_and_two_independent_exports_can_finish_concurrently`、`sync_failure_before_and_after_atomic_move_reports_distinct_real_visibility`、`racing_target_directory_wins_without_any_overwrite`、`moved_final_identity_is_unconfirmed_without_reporting_a_competitor_as_owned`、`raw_reader_preserves_binary_bytes_and_store_rows_without_taking_a_lock`、`raw_reader_rejects_revision_key_and_pending_effects_before_output`、`raw_reader_rejects_modified_truncated_and_unsafe_originals`、`raw_reader_rejects_corrupt_persisted_reference_before_reading_external_bytes`、`raw_reader_rejects_growth_truncation_and_parent_or_leaf_swaps_during_streaming`、`raw_reader_accepts_exact_32_mib_and_rejects_writer_failure`、`raw_reader_rechecks_identity_after_the_final_byte_was_proved`。

**执行步骤。**

1. 固定单一 work-result/v1、raw 参数/错误、manifest、大小限额、输出状态与权限；核 source.attempt 是终点绑定/封存身份。
2. 创建外围 package，明确 `publish=false`、`package.metadata.dist.dist=false`；核 workspace dist 覆盖和本地计划确实排除 exporter。
3. 完整实现 strict DTO、校验类型、子进程 argv/stdin/stdout/stderr/exit、资源上限及可信同句柄源读取原语，覆盖 Work/revision/key/final/effects 绑定。
4. 完整实现父目录逐段句柄、身份/管理根重叠、私有目录和独占文件、软硬链接拒绝、读回、sync、整目录 NOREPLACE 与移动后未确认错误。真实支持平台实测；不留系统调用政策给简单实现者。
5. 完整实现 raw stdout 路由所需原语和错误 stderr 路径；普通 metadata 回归仍通过。未完成的引擎 raw 模式和 exporter 编排不挂正常入口，窄骨架可编译且不返回假成功。
6. 写真实 source/target、runtime/CLI、并发/kill/sync oracle；T01 原语直接走实际 FS/runtime 边界取得 green，T02 完整新行为经两份 binary 入口可执行并得到有意义 red。模拟只替换外部错误边界，不代替真实成功链。
7. 固定 `verification/commands.sh` 的 primitives/future-red/feature/regression/gates/dist-plan 命令、测试数、失败签名、平台和预算；Cargo JSON 取 executable，不猜 target 路径。
8. 在 runbook 固定 T03 构建/操作/质量/计时/残留核查，保存原语 green、feature red、普通回归及完整骨架 SHA，交 M1。

**oracle。** 源实际 bytes/size/sha/exit；DTO 未知字段、过时 revision 和待完成效果；raw 非 UTF-8/零字节不转码；句柄路径替换、软硬链接与 NOREPLACE 冲突哨兵不变；移动后 sync 失败准确 unconfirmed。所有规范高风险义务在此有真实实现和独立预期。真实 feature red 可因 raw/工具入口未接通或编排未实现，且必须有真实 CLI 或持久消费者的行为断言失败，不能仅凭占位 panic。

**验证与交接。** 创建命令文件后运行：

```bash
bash specs/changes/completed/C006-result-delivery/verification/commands.sh primitives
bash specs/changes/completed/C006-result-delivery/verification/commands.sh future-red
bash specs/changes/completed/C006-result-delivery/verification/commands.sh regression
bash specs/changes/completed/C006-result-delivery/verification/commands.sh dist-plan
scripts/check-tests.sh
scripts/check-docs.sh
scripts/check-specs.sh
scripts/check-task.sh C006-T01 <T01 开工完整提交> --staged
scripts/check-task.sh C006-T01 <同一完整提交>
```

primitives 含 `scripts/task.sh C006-T01`，实际非零数量且通过；future-red 至少一项真实 T02 用例、非零退出和预定失败，不计 feature PASS。运行完整工程门禁与 `cargo deny check`，记录隔离 CARGO_TARGET_DIR/RUSTC_WRAPPER、两个 binary、counts、平台及原 run ID。交接所有参数/类型、可用安全原语、有效 red、平台与 dist 排除证据、fixture closure、骨架完整 SHA、runbook 和未跑项。

**停止。** 只需指针、C004 载荷不确定、safe API 或实机不能兑现目录原子不替换/同步、raw 需文字转码或已有正确行为受损时，由复杂作者修范围或接口；不交简单实现者降级保证。

## C006-M1：审查安全与实现准备

独立复杂模型实查 source、target、raw 原语已完成而非空骨架；核真实平台、资源上限、身份/链接、sync/未确认分支、严格来源与 dist 排除。核实际 green、非零完整行为 red、普通回归、测试/混合白名单、冻结命令与初级开发者能照接的接口。未完成 raw/export 路径不得作为正常用户能力出现。缺原语、fixture 或 oracle 交 T01 修复，再独立复核。

保存审查候选、closure、原 run ID 与平台未跑项；未变化的 T01 证据可引用，不重跑机械全套。通过仅表示阶段 2 准备完成，不是复制、用户价值或发行 PASS。运行文档/治理和 diff 检查；`scripts/check-task.sh C006-M1 <M1 开工完整提交> --staged`，提交后以同基准再检查。

### C006-T02：用固定原语完成整份副本

**Owner / 输入。** 简单模型或初级开发者；M1 骨架或最新独立测试修订完整 SHA、冻结 source/target 接口、测试和命令文件。

**范围。** exporter `main.rs` 的参数与分发（复用冻结library编排）、引擎 `cli.rs`、`commands/work.rs/mod.rs` 的正式接线。source/target/strict DTO/error、大小、OS、资源回收和发布政策已由 T01 实现，不修改；测试只删本任务 ignore。正常入口在全链满足后激活，不能阶段中暴露半成品。

**测试。** `result_artifact_stdout_preserves_binary_empty_and_text_bytes_without_json_or_extra_newline`、`result_artifact_rejects_json_request_ids_and_unpaired_parameters_before_creating_a_home`、`result_artifact_rejects_revision_key_and_source_modification_without_business_writes`、`result_artifact_accepts_empty_hyphen_unicode_and_literal_metacharacter_slot_keys`、`exporter_rejects_an_overlapping_or_symlinked_parent_without_modifying_the_target`、`exporter_rejects_a_nonfinal_work_before_creating_staging`、`kill_after_staging_creation_preserves_owned_scene_without_a_published_copy`、`kill_during_actual_receive_preserves_partial_scene_without_a_published_copy`、`kill_after_file_sync_preserves_owned_scene_without_a_published_copy`、`kill_after_independent_readback_preserves_owned_scene_without_a_published_copy`、`kill_after_manifest_write_preserves_owned_scene_without_a_published_copy`、`kill_after_tree_sync_before_rename_preserves_staging_without_a_published_copy`、`kill_after_rename_before_parent_sync_leaves_a_whole_visible_copy_without_a_response`、`kill_after_parent_sync_before_response_leaves_a_whole_copy_and_rerun_creates_another`、`sync_failure_before_publication_keeps_staging_and_rerun_preserves_the_original_scene`、`sync_failure_after_rename_reports_unconfirmed_and_keeps_the_whole_visible_copy`、`export_publishes_all_final_bytes_and_deterministic_provenance_without_changing_work`、`export_rerun_creates_a_new_copy_and_preserves_edits_in_the_previous_copy`、`export_source_integrity_failure_preserves_owned_staging_without_publishing_a_final_directory`。

**执行步骤。**

1. 核 M1、骨架 SHA、白名单和工作树，读学习导航及固定命令。
2. 跑 future-red，核真实行为用例数和失败签名；编译错误或零测试退回准备阶段。
3. 按固定 DTO 取得最终结果；通过 source 原语接收每项字节到独占私有目标，逐项核 size/sha/exit。
4. 按 target 原语读回全部成果、写固定清单、同步树，用已完成 NOREPLACE 原语发布，准确映射 complete/unconfirmed。
5. 接正式 raw CLI 和 exporter 参数分派，直接使用 T01 原字节路由；只读路径不触发写后清理或业务写回。
6. 逐条跑 feature 及 crash/竞态，全部满足后只删本任务 ignore，核无残留半接线。
7. 跑普通消费者回归、工程门禁与短语义复核，保存实际文件/哨兵/Store 前后证据，交 T03。

**oracle。** 两份真实 binary 的 C004 metadata→固定 revision/key raw→独立 size/sha/exit→全体暂存→清单→NOREPLACE→sync；所有规范拒绝/kill 窗口按冻结矩阵。源非零、缺 bytes、链接、目标竞争、父身份变化不得发表完成；移动后未确认不得删除补偿。只复制明确最终 Artifacts，业务状态不变。

**验证。**

```bash
bash specs/changes/completed/C006-result-delivery/verification/commands.sh feature
bash specs/changes/completed/C006-result-delivery/verification/commands.sh regression
bash specs/changes/completed/C006-result-delivery/verification/commands.sh gates
scripts/check-task.sh C006-T02 <阶段骨架或最新独立测试修订完整提交> --staged
scripts/check-task.sh C006-T02 <同一完整提交>
```

feature 含 `scripts/task.sh C006-T02`，实际非零数量、全部通过且无本任务 ignore。原语未变化时引用 M1 closure/原 run；新增真实 binary 链必须有新运行。保存 binary hash、counts、byte 比对、故障结果、未跑项和下一动作。

**停止 / 交回。** 需修改 source/target、DTO、OS/恢复/限额政策、测试或 fixture 时停止相关实现。把最小 caller 复现与输出交复杂作者；在同 package 分配修复任务、范围和 oracle，独立审查后更新完整基准。简单实现者不得根据 stderr 自然语言猜成功或改断言。

### C006-T03：按手册完成首次使用与副本证据

T03 若只有说明和真实操作记录，范围基准用本任务开工完整提交；若 T01 已预制并归属 T03 的 Rust 场景测试，复杂作者先将实际文件加入 T03 test_files、固定 allow_test_changes=false，并像 T02 一样以 M1 骨架或最新独立测试修订完整 SHA 核冻结测试。手册必须写明实际采用哪一种，初级执行者不自行选择基准。

**Owner / 输入。** 手册执行者；T02 候选、T01 runbook、已明确副本用途。复杂模型负责产品结论与不明失败分析，不把判断留给初级执行者。

**范围 / 操作。** 按 `experiments/runbook.md` 的实际构建与二进制路径，用同一最终结果执行手工复制与工具复制，分开记录找结果、接收、核对和失败处置。补准确 `specs/guides/result-export.md`，核首次读者能取得正确可编辑副本。同步CHANGELOG未发布段的schema4/cli-resultv4与已实现功能；不改发布历史。机制可在临时目录跑；真实用户位置须已有授权。

**oracle / 停止。** 字节与清单一致、不会覆盖、残留/unconfirmed 能准确核查；用户只需指针或无收益也能如实结论。无真实用途保持价值 not_run；缺授权只停相关真实位置写入。缺陷交修复任务，不改合同、oracle 或成功标准。

**验证 / 交接。** 执行冻结 runbook、文档/diff 检查；仅有提前冻结的 T03 Rust 行为用例才跑 task.sh，否则记录真实操作，不用零测试验收。范围检查为 `scripts/check-task.sh C006-T03 <runbook 已确定的完整范围基准> --staged`，提交后以同基准再检查。交接总投入、失败处置、实际指南、候选与全部 not_run 给 M2；不发布或正式安装。

## C006-M2：独立完整验收

独立复杂模型核来源到整份副本的真实用户链、所有错误/竞态/crash、平台、根外哨兵、业务状态、Rust 工程、dist 排除及首次读者真实结果。对 M1 后未改变的安全原语/合同引用原 closure 和 run ID，新增编排和正常入口核新证据；不重复运行未受影响全套。机制、质量、人工成本、样本和发布范围分别结论。修复交责任任务后复核；运行文档/治理和 diff 检查；`scripts/check-task.sh C006-M2 <M2 开工完整提交> --staged`，提交后同基准检查。审查不授权外部发行。

### C006-T00：采用与单一实施入口

开工候选 `8d27348d995a434ae3dbf8255e1110973dec1381`。归档已限定独立完成的C005，保留全部raw字节/未知/授权延期；只激活C006，更新当前索引和引用。用户明确开发需求允许实施，没有编造真实复制成本或可编辑副本用途。代码尚未改，T01同步source/target合同后再实现；docs/specs/tests/TOML/diff与独立短审通过再提交。

本轮继续由同等能力作者实现必要耦合原语，保留T01/M1/T02和未参与oracle/代码的独立Reviewer。用户明确真实无链接的绝对home/to；key保留C004字符串（含空名/非ASCII），NUL因argv不可表达明确拒绝。complete表示核验及规定OS文件/树/父fsync成功，不增加断电物理持久承诺。实际平台API仍须T01实测。

### C006-T04：修复真实checkpoint通知握手

T02完整回归4954967d为829PASS/1FAIL，reached文件已存在但尚为空；writer create_new与write_all之间允许reader读取。隔离自有T02草稿stash `ef7d9bdfe49697a1aa858935c4ac0040ab28b117`，不改失败历史。复杂作者仅修crash.rs的checkpoint消费等待：完整预期字节才宣称到达，空/合法前缀继续等待；错误同步点仍拒绝，12秒上限、child退出和原artifact/kill/重跑断言不变。

**测试。** `boundary_waits_for_complete_marker_bytes_before_claiming_the_checkpoint`、`boundary_rejects_bytes_from_another_checkpoint`。

先用真实文件与活子进程构造已存在空/部分通知取得有效red，再修等待取green；不调整业务政策、fault生产端或原成果断言。两项独立期望、fmt/check/clippy、独立review和治理/范围检查完成后提交T04；该完整SHA是T02新冻结基准。恢复自有草稿仅保留公共接线与19ignore删除，其他部分字节保真。RC生命周期治理另列后续任务。

### C006-T05：保留未发布开发候选的明确版本权威

产品package归档后，active实验没有数值产品目标；现checker会误拒绝同一未发布RC。修复前用隔离真实治理夹具取得非产品active/无active的实际red。开发目标只由specs/README首屏唯一字段给出；数值产品active必须相符，明确不进入产品release的实验及无active不替换该权威。未知实验目标、缺失/重复/非法开发目标、Cargo错误RC或数值active冲突均拒绝；原release/tag/缺历史/CHANGELOG检查和原断言保持。

**测试。** `check_specs_accepts_authorized_rc_during_nonproduct_experiment`、`check_specs_accepts_authorized_rc_after_product_package_is_completed`、`check_specs_rejects_missing_duplicate_or_invalid_development_authority`、`check_specs_requires_development_authority_in_first_screen`、`check_specs_rejects_numeric_active_conflict_even_when_cargo_version_is_released`、`check_specs_rejects_unknown_nonproduct_target_even_when_cargo_version_is_released`、`check_specs_rejects_rc_outside_authority_during_nonproduct_experiment`、`check_specs_authorized_rc_without_active_still_requires_changelog`。

Owner分开：later_plans负责check-specs、README字段与release_governance测试；root先写工程权威、D-043、任务范围和记录。验收用例先建立再登记实际名称。check/clippy/fmt、release_governance全部真实consumer、独立review、docs/spec/tests/范围通过再提交；引擎source/Cargo/格式及832运行输入未变部分按消费者引用，不重跑无关完整引擎链。未发布目标不是发布授权。

### C006-T06：补齐两个原定真实崩溃窗口

M2逐项对照发现原19caller只在AfterReadback/AfterManifestWrite暂停，不能等同原要求的全部成果读回前/清单写入中。已有8checkpoint证据保持原范围；复杂作者新增BeforeReadback和DuringManifestWrite外部边界，后者在真实manifest部分字节写入后、其余写入前暂停。正常路径仍写同一完整清单并读回/同步后整目录发布；移动/权限/OS/大小政策不改。

**测试。** `kill_before_independent_readback_preserves_artifacts_without_publication`、`kill_during_manifest_write_preserves_partial_manifest_without_publication`。

Root新建两个真实CLI消费者先取缺checkpoint的red；later_plans只改Target/failpoint、保留旧points。新oracle核3文件实字节、partial JSON不完整/读回前无manifest、实际SIGKILL/无响应/无final、business status/result不变、重跑只新建完整副本并保留旧场景。原10crash断言不改。2新green、原Target/crash/publicconsumer、真实固定binary、完整工程门禁与MSRV按实际变更执行；独立审查后提交T06再M2。不能据静态分支或相邻点宣称原窗口通过。
