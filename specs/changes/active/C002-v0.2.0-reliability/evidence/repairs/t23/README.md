# C002-T23 self 文件链与 purge

候选基线 `fd21a63d9d714cf92befb4c2f0d4c7ad3cc11355`。平台 macOS aarch64；Linux 运行按用户授权豁免，状态为 `not_run`，不推导跨平台结论。Rust 命令使用 `RUSTC_WRAPPER=` 与独立 target `/private/tmp/sheltie-c002-target`。

## 改动与依据

- `install`、`update`、`rollback`、`uninstall` 的管理根写入均经受管文件 API；update 使用摘要已核字节解包和新建候选，保留同一 `SafeFile` 到候选 chmod、rename/exchange 和落位后字节复核。
- rename/exchange 已发生而身份、内容或目录同步无法确认时返回 `RecoveryRequired`。self 临时目录清理按错误变体决定；恢复状态不确定时保留目标、prev 与临时目录，不通过错误文案推断对象状态。
- 子进程 stdout/stderr 都有硬限额；超限或流错误会停止子进程、等待退出并 join 输入输出线程。tar 成员参数使用 `--` 结束选项解析。
- purge 按对象身份保留原 `.lock`，先预检再按树、pending/tmp/bin、SQLite sidecar、主库顺序删除，最终复核残留对象与根/锁身份。晚到的大小写不敏感名称 `store.db-shm` 被识别为 SQLite sidecar；其他新对象会报告准确 partial 状态。
- CLI 的 purge 确认文本与成功响应一致：清理管理数据和 binary，保留根目录及原锁文件。相对 `SHELTIE_RELEASE_BASE` 按命令当前目录词法解析。

## 反例与调用链验证

`task.stdout.txt` 对应 `scripts/task.sh C002-T23`：12 passed，516 skipped（这是任务筛选，不是全套跳过）。其中有实际 CLI 的相对 release base 与 purge 响应；runtime 测试覆盖冻结 Work/Workbook purge、外部哨兵、旧 binary/prev 保持、归档链接、late SHM、锁在最终复核前被 unlink，以及摘要校验后同 inode 候选字节改变。后者断言 update 拒绝且原 binary、`.prev` 字节未变。

影响面 runtime selfmgmt/crash/fs-boundary 运行 `bc1fc18d-48cb-42c7-a2cb-077138cd1a02`：61 passed、0 skipped、2 leaky。leaky 是既有 fs_boundary 的 `managed_rename_rejects_symlink_source_without_moving_or_touching_target` 与 `ensure_dirs_rejects_symlink_and_file_placeholder_below_root`。CLI self 命令运行 `c61255cc-b220-450a-b3f6-394a288fbd00`：8 passed、0 skipped。

全仓 nextest 运行 `29a636e5-cd43-4218-9063-df2603c9ac68`：528 passed、0 skipped，71.275 秒；MSRV `cargo +1.85.0 check --all-targets --all-features` 通过。格式、全 targets/features 编译、Clippy、依赖、docs/specs/test-owner 和相关命令的完整输出保存在本目录。`cargo deny check` 默认 advisory DB 因用户级只读锁失败；使用 `/private/tmp/sheltie-cargo-home3` 中本机缓存的 `cargo deny --offline check` 通过。失败原文保存在 `cargo-deny.stdout.txt`；通过运行的完整输出为 `cargo-deny-isolated.stdout.raw.b64`，用 `base64 -D < cargo-deny-isolated.stdout.raw.b64` 无损还原。

## 独立审查与剩余边界

Spec Reviewer 与 Standards Reviewer 对最终工作树给出 PASS。审查结论和逐轮修复往返保存在本会话记录；reviewer 未修改代码。Linux 原生验证仍为 `not_run`。本任务没有执行 T31 锁等待者交错实验，也不代表 C002-M1 或 C002 整体完成。
