# 测试处置

基线为 README 中的完整提交，`scripts/check-tests.sh` 识别 951 项；这是静态归属计数，不是本轮执行结果。实现时在下表记录确切后继，不以同名、同数字或旧 Task 编号取消义务。

| 原测试或支持 | 拟处置 | 必须保留 |
| --- | --- | --- |
| attempt_begin_returns_brief_path_that_exists | 并入 two_step_via_cli_reaches_succeeded | 实际 brief、标题、Attempt 身份、返回 next 的完整推进 |
| workbook_verify_exits_1_after_tamper / verify_detects_the_edit | verify_detects_the_edit 并入 workbook_verify_exits_1_after_tamper；两种 selector 共享夹具 | 全库/精确版本、exit 1、错误 code 与 detail |
| brief_shows_entered_from_line_or_entry | 并入 brief_for_node_without_requires_omits_section | entry 与 main/back 来源独立验证 |
| update_replaces_binary_and_keeps_prev | 由 clean_home_install_then_update_two_step 接替 | 更新版本、当前和 prev 字节、Store 未改 |
| insert_workbook_on_readonly_store_is_not_workbook_exists | 接替为 commit_workbook_on_readonly_store_preserves_sqlite_error_and_rows；删除第二测试专用插入实现 | 非 constraint SQLite 错误与原事务回滚 |
| CLI 目录复制和现场快照 | 共享测试支持 | 原 bytes/mode/dev/inode/nlink/类型检查，不用生产 helper 算期望 |
| runtime SQL/树快照和初始化同步 | 共享测试支持 | 五表原记录、文件对象、私有 session 与真实调用入口 |
| exporter kill 辅助流程 | 共用启动/kill/保全/rerun | manifest 缺失/部分字节、rename 前后窗口与不覆盖原件 |
| spec-dev cold_replan/cold_lineage 负例 | 保留代表性 CLI 闭环，其余缩小夹具 | 各字段/基线/历史证据拒绝，不证明实际 agent 遵从 |

恢复、重放、严格解码、各文件读取入口、大小边界、真实进程 kill 与提交前后窗口保持。没有后继的当前义务不得删除。最终测试数量、确切迁移和运行资格在 validation 记录。

## 名称与调用入口纠偏

| 原名称 | 当前后继 | 不变 oracle |
| --- | --- | --- |
| handwritten_schema2_control_accepts_both_open_modes | handwritten_current_schema_control_accepts_both_open_modes | 手写 schema 4、双打开模式及序号 7 |
| open_creates_schema_with_user_version_1 | open_creates_schema_with_current_user_version | 当前 SCHEMA_VERSION 与五张表 |
| observe_file_rejects_symlink_and_directory | external_file_rejects_symlink_directory_and_missing | 实际 ExternalReadFile 拒绝链接、目录、缺失 |
| observe_file_sha256_matches_known_vector | external_file_sha256_matches_known_vector | 手写 SHA256 向量与 6 bytes |
| resource_index_marks_non_utf8 | 同名迁到 fsx::tree_tests | inspect_tree_v2 实际扫描，两文件 UTF-8 正反值 |

readonly 后继还比对五表全行不变；原资源 alias 用实际 digest/文件读取链验证。SQL、树现场工具仅供测试，保存原字段，不调用生产摘要或建表脚本生成期望。初始化仍分私有 WriteSession 和真实 install/add 两层；rollback 的 latest/pinned 与二次 NOT_FOUND 未取消。Exporter 十个 kill 窗口名称和断言保持。

spec-dev 九组检查器负例改用实际文件/Git 小夹具，各自先验证完整合法材料，再改变一个条件并核拒绝原因；完整 CLI 冷接续、continue、审批更正、三任务与整体 fix 正例保留。负例不再逐变体重复整条 CLI，因而不证明每种损坏与全过程的组合覆盖，也不证明真实 agent 遵从方法。

独审发现初始化共享夹具的退出顺序需闭合，已改 scoped threads 并新增 concurrent_initialization_fixture_joins_the_waiting_writer_after_failure，覆盖 Err/panic 两变体。全量实际948/948、0 skip；四项退休、一项新增，变化不是通过删失败用例取得。
