# 测试处置

基线为 README 中的完整提交，`scripts/check-tests.sh` 识别 951 项；这是静态归属计数，不是本轮执行结果。实现时在下表记录确切后继，不以同名、同数字或旧 Task 编号取消义务。

| 原测试或支持 | 拟处置 | 必须保留 |
| --- | --- | --- |
| attempt_begin_returns_brief_path_that_exists | 并入 two_step_via_cli_reaches_succeeded | 实际 brief、标题、Attempt 身份、返回 next 的完整推进 |
| workbook_verify_exits_1_after_tamper / verify_detects_the_edit | 两种 selector 共享夹具 | 全库/精确版本、exit 1、错误 code 与 detail |
| brief_shows_entered_from_line_or_entry | 入口断言并入同夹具的 brief 测试 | entry 与 main/back 来源独立验证 |
| update_replaces_binary_and_keeps_prev | 由 clean_home_install_then_update_two_step 接替 | 更新版本、当前和 prev 字节、Store 未改 |
| insert_workbook_on_readonly_store_is_not_workbook_exists | 去掉第二测试专用插入实现；必要 readonly 检查走 commit | 非 constraint SQLite 错误与原事务回滚 |
| CLI 目录复制和现场快照 | 共享测试支持 | 原 bytes/mode/dev/inode/nlink/类型检查，不用生产 helper 算期望 |
| runtime SQL/树快照和初始化同步 | 共享测试支持 | 五表原记录、文件对象、私有 session 与真实调用入口 |
| exporter kill 辅助流程 | 共用启动/kill/保全/rerun | manifest 缺失/部分字节、rename 前后窗口与不覆盖原件 |
| spec-dev cold_replan/cold_lineage 负例 | 保留代表性 CLI 闭环，其余缩小夹具 | 各字段/基线/历史证据拒绝，不证明实际 agent 遵从 |

恢复、重放、严格解码、各文件读取入口、大小边界、真实进程 kill 与提交前后窗口保持。没有后继的当前义务不得删除。最终测试数量、确切迁移和运行资格在 validation 记录。
