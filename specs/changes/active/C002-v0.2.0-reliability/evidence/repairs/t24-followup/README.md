# C002-T24 后续修复：并发创建 Store 的只读预检竞态

平台：macOS aarch64。Linux 按用户授权保持 `not_run`。

## 发现与修复

T25 全仓运行 `8a458924-bebc-418c-9100-8480ee62743c` 曾发现 `parallel_adds_do_not_delete_each_others_staging` 失败：首个写者已持 `.lock` 并创建空 `store.db`，尚未提交 schema；第二个写者的只读 Store 预检读到 `user_version = 0`，误判成旧 schema。

`Store::open_for_home` 现在遇到 `StoreSchemaMismatch` 时，只有在当前管理根内已有安全的 `.lock` 文件才等待它，随后重新只读识别。等待入口通过现存根目录句柄，以 no-follow 打开既有 `.lock`，核对普通文件类型、单链接、dev/inode；获锁后还会核对根和锁的身份。检查期间不创建根或锁。锁不存在时直接返回原 schema 错误；锁在观察后被删除或替换时停止，不创建替代锁。RW Store 仍由 `WriteSession` 在同一把锁下打开。

该共同预检被 Workbook add 和 self install 的真实调用使用，因此两个方向的 add/install 初始化交错都覆盖了。

## 验证

- [`task.stdout.txt`](task.stdout.txt)：`scripts/task.sh C002-T24`，29 passed、541 task-filtered，run `8af2bde6-0b87-4134-87a4-9dccd1288459`。
- [`clippy.stdout.txt`](clippy.stdout.txt)：全 targets/features Clippy `-D warnings` 通过。
- 确定性交错 `session::tests::concurrent_store_initializer_waits_for_schema_before_preflight_rejection`：首个 Session 停在空库创建后；第二个 Session 通过已有锁等待，schema 初始化完成后重新识别成功。
- 真实调用 `real_install_waits_for_a_concurrent_workbook_add_store_initializer` 与 `real_workbook_add_waits_for_a_concurrent_install_store_initializer`：两个调用方向均等待同一初始化窗口并成功。
- `home::lock_retry_tests::existing_lock_removed_after_open_is_never_recreated`：在锁句柄打开后删除 `.lock`，放开同步点后操作拒绝；管理根和旧文件保留，`.lock` 不重建。
- 原有 `schema1_store_rejected_without_touching_file` 核对 `.lock` 不存在时，add/install 拒绝且 schema 1 的 main/WAL 字节不变。
- `scripts/check-docs.sh`、`scripts/check-specs.sh`、`scripts/check-tests.sh` 均通过；`git diff --check` 通过。

该 follow-up 只关闭并发初始化竞态；完整候选回归仍须由当前 C002 任务复跑。结论限于 macOS，Linux `not_run`。
