# C002-T15 证据

- Owner：Claude（mimo 会话）。基准 `4960e9fc2295d02d9abdd624375d82340dcba7d4`（T13 提交），候选是本次任务提交前的待提交树。
- 输入闭包：`Cargo.lock` 未变（deny 仍跑）；`Cargo.toml` 只加 dist 的 `allow-dirty = ["ci"]`；特性全开（`--all-features`，含 `failpoint`）；平台 macOS aarch64、`rustc 1.98.1`，MSRV `cargo +1.85.0` 实跑（不是 stable 代替）。self 安装、更新、rollback 与发布 fixture 全部在临时管理根/临时发布目录；治理夹具是临时 Git 仓库；不写真实 `~/.sheltie`、`~/.claude`，不写任何 shell rc。
- 改动：`crates/sheltie-runtime/src/selfmgmt.rs`（固定 tag 解析、路径安全拒绝、失败窗口清 tmp、update/rollback/install 全程持锁）；`crates/sheltie-runtime/src/home.rs`（锁身份复核 + 建锁窗口 NotFound 整体重试）；`crates/sheltie-cli/src/cli.rs`、`crates/sheltie-cli/src/commands/self_cmd.rs`（删 `--modify-path`、`--json` 纯协议、self 拒 request-id、schema 2 提示）；`scripts/check-specs.sh`（已发布/开发中分流、缺历史诊断）；`.github/workflows/build.yml`（docs 拉全历史、MSRV job）、`.github/workflows/release.yml`（同 SHA `quality` job，announce 挂它）；`README.md`（rollback 不降级 Store）；测试 `crates/sheltie-runtime/tests/{selfmgmt,crash,fs_boundary,workbook_identity}.rs`、`crates/sheltie-cli/tests/self_cmd.rs`、新文件 `crates/sheltie-cli/tests/release_governance.rs`。

## 卡片四条落在哪

| 卡片要求 | 落点 | 由谁证明 |
| --- | --- | --- |
| 删 `--modify-path` 写 shell rc 的入口 | `cli.rs` 没有该参数；install/update 只写管理根内路径 | `install_modify_path_flag_is_rejected`：真实二进制带 `--modify-path` 退出码 2（clap 用法错误），另 `install_prints_path_hint_and_does_not_touch_rc_by_default`（T20）复验不碰 rc |
| `--json` stdout 只输出协议 JSON | schema 2 提示只进文本模式的 SCHEMA_NOTE；`--json` 只有响应封装（data 里的 `installed_to`/`path_hint` 等字段），stdout 是单文档 | `self_install_json_stdout_is_single_document`、`self_update_json_stdout_is_single_document`：`serde_json::from_slice` 解析整个 stdout，前后缀文本直接失败 |
| self 带 request-id 退出码 2 | self 是无 request-id 命令组 | `request_id_rejected_for_readonly_and_self_commands`（T10，本次全量复验绿） |
| 按固定 tag 解析 `--version`、manifest 与资产（N06 两份 latest 漂移） | `resolve_tag` 一次固定：无 `--version` 只读 `latest/` 清单学版本再固定 `v{V}`；有 `--version` 固定 `v<version>`；清单与资产同 tag | `update_pinned_version_installs_only_that_tag`、`self_update_version_flag_pins_tag`、`update_latest_drift_does_not_mix_manifest_and_assets`、`update_rejects_manifest_version_not_matching_tag` |
| 下载、摘要、解包、替换和 rollback 分别核失败窗口 | 每个窗口失败：旧二进制不变、`sheltie.prev` 不乱建、tmp/<uuid>/ 清空（存储合同 §9）；替换崩溃窗口用 `update_between_renames` 真注入 | 见下表 |
| 所有 managed self 路径用 T04 边界与同一写锁 | `install`/`update`/`rollback`/`uninstall` 都 `acquire_lock`；路径经 `checked_asset_name`/`checked_version` + `fsx` 边界 | `update_rejects_forged_asset_name`、`update_rejects_path_like_version_argument`、`fs_boundary` 全量绿 |
| purge 持锁；等待者复核根与 `.lock` 的 dev/inode，不符释放旧锁整体重试 | `HomeLock::identity_still_valid` + `acquire_lock` 重试 16 次（根被 purge 删除时 NotFound 同样整体重试） | `home_lock_identity_detects_replaced_lock_file`、`home_lock_identity_detects_replaced_root`、`purge_waiter_reacquires_on_fresh_root`、`self_install_and_work_writes_serialize_under_home_lock` |
| 安装/更新提示明确 Store schema 2 与旧数据保留，不能误导只换二进制降级 | `SCHEMA_NOTE` 进 install/update 文本提示；README 同口径 | `self_install_text_prompt_explains_schema2_and_rollback_limit`、`self_update_text_prompt_explains_schema2_and_rollback_limit` |
| 治理 job 获取历史 tags/commits | build.yml docs job 与 release.yml `quality` job 都 `fetch-depth: 0` | `build_workflow_fetches_history_and_runs_msrv_gate`（静态断言）；浅克隆诊断由 `check_specs_shallow_clone_reports_missing_history` 实测 |
| check-specs 分开验证已发布 release 与 active target/RC | 有 release record 按已发布核 tag/Release commit；开发中版本 = active 目标或其 RC 不要求 tag | `check_specs_accepts_active_target_without_tag`、`check_specs_accepts_rc_of_active_target`、`check_specs_rejects_version_outside_active_target`、`check_specs_dev_version_requires_changelog_section`、`check_specs_full_history_resolves_release_commits` |
| release 依赖同一 SHA 的质量 job；质量失败不能发布 | `quality` job 验 `git rev-parse HEAD = $GITHUB_SHA` 后跑全套门禁；announce `needs: quality` 且 `if` 要 `needs.quality.result == 'success'` | `release_workflow_gates_announce_on_quality_job`、`release_workflow_quality_failure_blocks_announce`（两个单条件反例） |
| 实际运行 MSRV 1.85 locked gate | build.yml `msrv` job 与 release.yml `quality` job 各跑 `cargo +1.85.0 check --workspace --all-targets --all-features --locked`；本树本地实跑 exit 0（见门禁表 msrv.txt） | `build_workflow_fetches_history_and_runs_msrv_gate`（静态断言）+ 本地真实运行 |
| 本地 release fixture 测指定版本、latest、rollback、clean home two-step 与 `cargo dist plan` 真实发布形状 | `SHELTIE_RELEASE_BASE` 本地 tag 目录；plan 形状夹具照 [dist-plan.json](dist-plan.json) 的真实结构 | `update_pinned_version_installs_only_that_tag`、`update_latest_drift_does_not_mix_manifest_and_assets`、`update_pinned_version_then_rollback_restores_previous`、`clean_home_install_then_update_two_step`、`update_adapts_cargo_dist_plan_real_manifest_shape` |
| 四平台已发布资产取证留给 T17 | 本任务不取 | `not_run`（边界段） |

## 五个失败窗口

| 窗口 | 唯一改变的条件 | 独立期望 | 测试 | 结果 |
| --- | --- | --- | --- | --- |
| 下载 | 清单声明的资产不在 tag 目录里 | `UpdateUnavailable`；旧二进制字节不变；无 `.prev`；tmp/ 空 | `update_failed_download_leaves_old_binary_and_cleans_tmp` | PASS |
| 摘要 | 资产下载得到，清单摘要记的是别的字节的哈希 | `UpdateChecksumMismatch`；旧二进制字节不变；无 `.prev`；tmp/ 空 | `update_failed_digest_leaves_old_binary_and_cleans_tmp`（T20 的 `update_rejects_checksum_mismatch_and_leaves_binary_intact` 同口径复验绿） | PASS |
| 解包 | 资产是坏压缩包但摘要自洽 | 解包错误；旧二进制字节不变；无 `.prev`；tmp/ 空 | `update_failed_extract_leaves_old_binary_and_cleans_tmp` | PASS |
| 替换 | 真实进程在 rename 之间被 `update_between_renames` 注入退出（exit 70） | `bin/sheltie` 不存在、`.prev` 与替换前的旧二进制逐字节相同 | T23 `kill_between_update_renames_leaves_prev_and_rollback_recovers`（本次把它的发布夹具改成 `latest/` + `v<version>/` 后复验绿；`.prev` 字节相等断言本轮补上） | PASS |
| rollback | 从替换崩溃窗口恢复（`bin/sheltie` 缺失时 `.prev` 挪回） | 恢复后目标是旧字节、`.prev` 消失；无 `.prev` 时再 rollback 以 `NotFound` 失败（API 层，不是进程退出码） | 同上 T23 用例（恢复腿，含恢复后字节相等）+ T20 `rollback_swaps_prev_back`（`NotFound` 腿）、`rollback_recovers_when_current_missing` | PASS |

## 发布形状与 dist plan

`dist plan --output-format=json` 在本树实跑（exit 0，stderr 空），原始输出存 [dist-plan.json](dist-plan.json)。测试 `update_adapts_cargo_dist_plan_real_manifest_shape` 的夹具按真实形状写死：`artifacts` 按产物名索引；`checksum` 是**校验文件名**（`X.tar.xz.sha256`），真哈希在 `checksums.sha256` 字段里；已建产物（本机 aarch64-apple-darwin tarball 与 `source.tar.gz`）带 `checksums`，未建的 executable-zip **缺 `checksums` 键**（不是 null），适配器必须跳过而不是拿文件名当哈希；顶层 `assets` 是 exe 条目字典（无 `kind`、`name` 是 `sheltie`），不能当发布资产取——夹具在 tag 目录另放同名毒字节，谁取谁翻车。本次实跑的 plan 与该形状一致（12 个 artifacts，带 `checksums` 的是已建的本机 tarball 与 `source.tar.gz` 两条）。

## 治理夹具

测试在 `crates/sheltie-cli/tests/release_governance.rs`，全部 `// Task: C002-T15`。夹具把仓库治理树（`scripts/`、`specs/`、`.github/` 与四个根文件）拷进临时目录自建 Git：commit1 是 record 记的发布 commit，commit2 装 record 对齐改写（40 位 commit 全改写成 commit1，短 hash 不动），`v0.1.0` tag 指 commit1。浅克隆 tip（`git clone --depth 1 file://…`）只剩 commit2，tag 与 commit1 都取不到。git 与 check-specs 都剥 `GIT_DIR` 类变量、`GIT_CONFIG_GLOBAL`/`SYSTEM` 指 `/dev/null`（T12 的教训）。

| 测试 | 唯一改变的条件 | 独立期望 | 结果 |
| --- | --- | --- | --- |
| `check_specs_full_history_resolves_release_commits` | 无（正例：版本 0.1.0 有 record） | exit 0 | PASS |
| `check_specs_accepts_active_target_without_tag` | 版本改 0.2.0（active 目标）+ CHANGELOG 段；**没有** v0.2.0 tag | exit 0（不要求开发中的 tag） | PASS |
| `check_specs_accepts_rc_of_active_target` | 版本改 0.2.0-rc.1；无 RC tag | exit 0 | PASS |
| `check_specs_rejects_version_outside_active_target` | 只改版本为 9.9.9（照样有 CHANGELOG 段） | 恰好一条失败且点名「active 目标版本」 | PASS |
| `check_specs_dev_version_requires_changelog_section` | 相对正例只去掉 CHANGELOG 版本段 | 恰好一条失败且点名「CHANGELOG」 | PASS |
| `check_specs_shallow_clone_reports_missing_history` | 只把检出换成浅克隆 | 失败行全部含「缺历史」，输出含「浅克隆」 | PASS |
| `check_specs_missing_tag_without_shallow_reports_not_created` | 只删 tag（历史完整） | 失败行全部含「未创建」，且不含「浅克隆」 | PASS |
| `release_workflow_gates_announce_on_quality_job` | 无（正例：读真实工作流） | `quality` 有 `fetch-depth: 0`、同 SHA 校验、MSRV locked；announce 挂 quality | PASS |
| `release_workflow_quality_failure_blocks_announce` | 两个反例各改一个条件：needs 去掉 quality；if 去掉 quality 成功条件 | 每个单条件反例都让「被挡住」谓词翻转 | PASS |
| `build_workflow_fetches_history_and_runs_msrv_gate` | 无（正例） | docs job `fetch-depth: 0`；msrv 与 release `quality` 都跑 MSRV locked；announce 用 `github.sha` | PASS |

## 测试总表（C002-T15 归属 32 例 = 31 新增 + 1 改写）

- self 生命周期与并发（runtime 17）：`install_modify_path_flag_is_rejected`、`update_pinned_version_installs_only_that_tag`、`update_latest_drift_does_not_mix_manifest_and_assets`、`update_rejects_forged_asset_name`、`update_rejects_path_like_version_argument`、`update_missing_tag_directory_reports_missing_tag`、`update_rejects_manifest_version_not_matching_tag`、`update_failed_download_leaves_old_binary_and_cleans_tmp`、`update_failed_digest_leaves_old_binary_and_cleans_tmp`、`update_failed_extract_leaves_old_binary_and_cleans_tmp`、`clean_home_install_then_update_two_step`、`update_pinned_version_then_rollback_restores_previous`、`update_adapts_cargo_dist_plan_real_manifest_shape`、`home_lock_identity_detects_replaced_lock_file`、`home_lock_identity_detects_replaced_root`、`purge_waiter_reacquires_on_fresh_root`、`self_install_and_work_writes_serialize_under_home_lock`。
- CLI 协议与提示（5）：`self_install_json_stdout_is_single_document`、`self_update_json_stdout_is_single_document`、`self_install_text_prompt_explains_schema2_and_rollback_limit`、`self_update_text_prompt_explains_schema2_and_rollback_limit`、`self_update_version_flag_pins_tag`。
- 治理与工作流（10）：见上两表。

全量 `cargo nextest run --all-features --no-tests=pass`：464 passed、0 skipped、1 leaky（`scenario_article_review::human_executor_node_is_begun_and_submitted_like_agent`）（T13 时 433；本任务 31 新增 + 1 改写——`install_modify_path_appends_export_line_to_shell_rc`（T20）改写成 `install_modify_path_flag_is_rejected`，T23 夹具改编不新增）。leaky 归因随运行漂移：本记录的最终运行标 1 项（见上），更早一次全量标 0 项，独立 Reviewer 复跑两次各标 1 项（不同用例）——同 T12/T13 的计时漂移口径。

## 处置记

- `--modify-path` 从 CLI 删除后，未知参数走 clap 用法错误退出码 2，测试按协议断言 `Some(2)`，不猜 clap 内部分类。
- `acquire_lock` 除了身份复核，把**建根/建 `.lock` 窗口里被 purge 删根**的 NotFound 也当「根已删」整体重试（存储合同 §2.2）：不重试的话并发 purge 会被报成普通 I/O 失败。`purge_waiter_reacquires_on_fresh_root` 是这个分支的动态用例（主线程持锁、purge 删根、等待者接手新根）。
- `install()` 在取锁**之后**才 `Store::open`，等待者在锁上排队，不会碰 purge 到一半的库。
- `update` 全程不碰 `store.db`（存储合同 §9）：`clean_home_install_then_update_two_step` 与 `update_pinned_version_then_rollback_restores_previous` 在 update（后者连同 rollback）前后比较 `store.db` 字节，逐字节不变。
- `checked_asset_name`/`checked_version` 同一套拒法：空、`.`、`..`、`/`、`\`、NUL（版本另拒前后缀形态）；伪造名反例四个形状（`../../secret`、`sub/secret`、`..\\secret`、`x\0y`）都验哨兵文件不被碰、旧二进制不变、tmp 清空，版本反例八个形状（含 `x\0y`）全部拒绝。
- latest 漂移反例的夹具设计：`latest/` 自洽但带毒（清单与资产都是毒字节），`v9.9.9/` 正确——只要混用两处来源，装上的要么是毒字节要么报摘要错，断言「装上的 == 正确字节」两种混法都杀掉。
- release.yml 的 `quality` job 与 skill 打包一步都是 dist 不生成的自定义步骤，注释写明重新 `dist init` 会丢掉、需照此补回；`[workspace.metadata.dist]` 加 `allow-dirty = ["ci"]`，否则 dist 校验会把整份工作流当过期内容拒绝。
- announce 的 `needs`/`if` 双条件是「质量失败不能发布」的全部机关，测试对两个条件各做一次单条件摘除，谓词必须翻——少任何一个条件质量失败都能建 Release。
- 测试 helper 不算自己的期望：治理夹具的期望（诊断措辞、失败条数、工作流条件）全在测试文件里写死；`check-specs.sh` 只作为被测对象运行。短 hash（`31d7dde`）与版本号数字不成 40 位串，改写器按「整段十六进制恰 40 位」替换，不会误伤。
- README 的 rollback 句补「只换回旧二进制，不降级 Store」，与 `SCHEMA_NOTE` 同口径；不改任何快照或产品 JSON 字段。
- 替换窗口（§9 步 4/5 之间）注释与实现对齐：注释标清是第 4 步与第 5 步之间；两次 rename 失败都先清 tmp 再报错，失败窗口不留半成品。
- 失败窗口的 `.prev` 断言统一走 `assert_no_prev`：下载、摘要、解包与伪造名四个拒绝窗口都核「不建 `sheltie.prev`」，只有替换步动它（§9 步 4）。
- `acquire_lock` 把 `.lock`/根身份取不到（open 与记身份之间的小窗口）映射成 NotFound，走「根已删」整体重试，不漏成普通 I/O 失败。
- 并发用例 `purge_waiter_reacquires_on_fresh_root` 用 100ms 睡眠当同步点（主线程持锁、purge 删根、等待者接手新根）；独立 Reviewer 认为同步点可以更硬，按其意见留 M1 一并看。purge 腿用 `remove_tree_no_follow`（T04 边界），与 `uninstall --purge` 同口径，不走普通递归删除。
- `specs/contracts/storage.md` §9 步 1「解析发布身份」的措辞与固定 tag 实现（`resolve_tag` 一次固定）语义一致，但字面未对齐；contracts 不在本任务白名单（T17 的白名单含 `specs`），措辞对齐留 T17。
- 全量 nextest 有过两次负载偶发，**都不是 T15 用例**，属共享测试基建：一次 `scenario_spec_dev::replan_after_first_task_keeps_original_baseline_for_final_review` 撞上 `assert_cmd::cargo_bin` 取不到 `target/debug/sheltie`（另一 cargo 进程在同一 `CARGO_TARGET_DIR` 上重链接；那次原始输出存 [nextest-flake.txt](nextest-flake.txt)）；一次 `skill_delivery::check_skill_fails_when_delivery_reference_is_stale` 在 verify 第 2 步报「命令不在交付协议 §2」而不是预期的引用过期（那次原始输出被紧随的运行覆盖、未存档——执行日志摘要：`Summary 69/464 tests run: 68 passed, 1 failed, 0 skipped`）。两个失败用例各自隔离复跑 3/3 绿，最终门禁运行 464 全绿。这类偶发与并发预构建、100ms 同步点同属共享测试基建的加固项，留 M1 一并看。

## 门禁

全部在候选最终树实跑；原始输出存本目录。

| 命令 | 退出码 | 原始输出 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 0 | [fmt.txt](fmt.txt) |
| `cargo check --all-targets --all-features` | 0 | [check.txt](check.txt) |
| `cargo clippy --all-targets --all-features -- -D warnings` | 0 | [clippy.txt](clippy.txt) |
| `cargo nextest run --all-features --no-tests=pass` | 0；464 passed、0 skipped、1 leaky（归因见测试总表） | [nextest.txt](nextest.txt) |
| `cargo deny check` | 0 | [deny.txt](deny.txt) |
| `cargo +1.85.0 check --workspace --all-targets --all-features --locked` | 0 | [msrv.txt](msrv.txt) |
| `dist plan --output-format=json` | 0（stderr 空） | [dist-plan.json](dist-plan.json) |
| `scripts/check-docs.sh` | 0 | [docs.txt](docs.txt) |
| `scripts/check-specs.sh` | 0 | [specs.txt](specs.txt) |
| `scripts/check-core-vocab.sh` | 0 | [core-vocab.txt](core-vocab.txt) |
| `scripts/check-tests.sh` | 0 | [tests.txt](tests.txt) |
| `scripts/check-skill.sh` | 0 | [skill.txt](skill.txt) |
| `git diff --check` | 0 | [diff-check.txt](diff-check.txt) |
| `scripts/check-task.sh C002-T15 --staged` | 0 | [check-task.txt](check-task.txt) |

## 独立审查

独立 Reviewer（未参与实施的通用 agent，与实施会话隔离）按证据纪律、单条件反例、实现正确性、合同/边界、工作流治理、改动范围六面核查；治理夹具与 check-task 在其侧独立复跑。

**首轮「需修改」。必改四项及处置：**

| # | 问题 | 处置 |
| --- | --- | --- |
| 1 | 「新增 32 例」计数错：`install_modify_path_appends_export_line_to_shell_rc`（T20）是被改写而非新增 | 测试总表与 validation.md 改为「31 新增 + 1 改写（T23 夹具改编不新增）」 |
| 2 | dist plan 形状记述不符：README 称「只有本机平台条目带 `checksums`」，实跑 plan 里 `source.tar.gz` 也带；fixture 两处形状偏差（非本机 executable-zip 写 `null` 而非缺键；顶层 `assets` 带 `kind`、name 写成路径形态） | README 改为「已建产物（本机 tarball 与 `source.tar.gz`）带哈希，未建的 executable-zip 缺 `checksums` 键」；fixture 照真 plan 键集重写（缺键、无 `kind`、name 是 `sheltie`），tag 目录放同名毒字节作陷阱 |
| 3 | 处置记虚报「崩溃窗口测试在 update 前后比较 `store.db` 字节」，无此断言 | 真断言补进 `clean_home_install_then_update_two_step` 与 `update_pinned_version_then_rollback_restores_previous`（后者连同 rollback），处置记按实际用例改写 |
| 4 | `checked_version` 不拒 NUL（与 `checked_asset_name` 不对称） | 实现补 `v.contains('\0')`；`update_rejects_path_like_version_argument` 反例清单加 `"x\0y"` |

**可选项处置：** 锁记身份小窗口映射 NotFound 进整体重试、替换窗口注释步号与 rename 失败清 tmp、T23 补 `.prev` 字节相等断言、五窗口表 rollback 行改成 API 层 `NotFound` 口径、JSON/文本的 schema 提示边界写明仅文本模式、leaky 计时漂移如实记述——均改毕。并发用例 100ms 同步点按审查意见留 M1；`storage.md` §9 步 1 措辞对齐留 T17（contracts 不在 T15 白名单）。

**二轮「需修改」。** 唯一必改是证据换真：12:32 刷新的 `nextest.txt` 是一次失败运行（负载偶发之一，见处置记），与摘要「exit 0、464 passed、0 leaky」矛盾——原始输出与摘要对不上是证据纪律红线。实现与其余记述逐条复核闭合：计数、dist 形状、`store.db` 断言、NUL 拒绝、可选项处置均确认属实；六面结论里正反例/实现/合同边界/工作流治理/范围闭合，只有证据文字面未闭合。处置：失败运行原样留存（[nextest-flake.txt](nextest-flake.txt)）并在处置记记因；候选树串行重跑全绿，`nextest.txt` 换成那次原始输出，摘要与 leaky 归因照实写（464 passed、0 skipped、1 leaky，点名到用例）。留后两项维持（同步点 M1、`storage.md` 措辞 T17）。

**三轮「通过」。** Reviewer 独立复跑全量 nextest 464 passed、0 skipped（其本次 0 leaky，落在记述的漂移范围内），核对：`nextest-flake.txt` 与处置记逐字一致且未剪裁；`nextest.txt` 与摘要三处（测试总表、门禁表、validation.md）的退出码、passed 计数、leaky 归因对得上；两条负载偶发记因属实（含「未存档」的自认）；代码 mtime 早于绿色运行，「只改证据文字」属实。正反例、实现、合同边界、工作流治理、范围、证据文字六面闭合，无遗留必改；留档两项维持（并发同步点加固留 M1，`storage.md` §9 步 1 措辞留 T17）。审查结论只覆盖本任务候选与证据，不是 M1 产品修复 PASS，也不构成对外发布授权。

## 覆盖与边界

- 关闭：卡片四条的本地可证部分——self 生命周期入口与失败窗口、锁身份与 purge 等待者、并发串行、schema 2 提示、固定 tag 不混包、治理检查分流与缺历史诊断、发布工作流质量门禁接线、MSRV 实跑、`dist plan` 真实形状。
- 不含：真实发布挂资产与四平台资产取证（T17）；GitHub Actions 上实际跑通（本地只核 YAML 接线语义，工作流要在远端跑才算数）；真实宿主里的 self 安装/更新体验（T16）；`dist init` 重新生成后自定义步骤的回归（靠注释与 `allow-dirty`，无自动测试）。
