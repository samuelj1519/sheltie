# C002-T24 请求解析、历史目标与锁内建库

候选基线 `7e9178e`；平台 macOS aarch64。Linux 原生运行按用户授权保持 `not_run`，结论不外推到跨平台。Rust 命令使用 `RUSTC_WRAPPER=` 和独立 target `/private/tmp/sheltie-c002-target`；MSRV 使用 `/private/tmp/sheltie-c002-msrv-target`。

## 改动与边界

- `WorkService::new(Home)` 与 `WorkbookRepo::new(Home)` 只保存延迟 Store 句柄，不访问文件系统；CLI 不再自行打开读写 Store。
- `WriteSession` 持有 `HomeLock` 与 RW Store。普通 Work 写入和 Workbook remove 只打开现存库；Workbook add、self install 才能初始化，并先做无锁只读 schema/来源预检，再取锁及锁内复核。
- 低层 Store、OpenMode 与 `Store::open` 不再是外部产品 API。SQLite 每次连接均先安全检查数据库/侧文件、复核 schema，再设 RW PRAGMA；RW Session 保留 Arc<HomeLock>，每次连接前后复核根、锁和 Store inode。只读连接只用 `READ_ONLY | NOFOLLOW` 并设置 `NO_CKPT_ON_CLOSE`。
- `store.db-wal`、`store.db-shm`、`store.db-journal` 的符号链接、硬链接和特殊文件在 SQLite 打开前拒绝；SQLite 竞争关闭时已 unlink 的零链接控制文件作为不可达瞬态处理。
- CLI 对 `@file` 只做绝对路径词法化。runtime 在请求查重后用逐段 no-follow 外部只读句柄读取；父/叶链接、硬链接、特殊对象、缺失、非 UTF-8 和超过 32 MiB 均返回 `INVALID_REQUEST`/退出码 2，并带路径与原因。summary/reason 超过其 4096 字节业务上限仍沿原错误码。
- RequestIntent 字段和序列化未改。请求命中时从 `requests.work_id` 先取完整历史 WorkId，再按大小写敏感字面前缀核验；所有 Work 写入口共用此规则。Start 只在未命中后规范化名称或读取输入。

## 验证

`task.stdout.txt` 的 `scripts/task.sh C002-T24` 为 25 passed、527 task-filtered。覆盖路径解析不探测文件、锁内建库、invalid add/start 不留 Store/锁、恰好 32 MiB / 多一字节、@file 父链接、start/submit/fail 的删源重放、请求已用后的名称冲突、当前前缀变歧义、大小写错误、schema 1 含有效 WAL 的锁前拒绝、主库及 WAL 字节保持、孤儿SQLite sidecar拒绝新建Store、SQLite main/sidefile 链接预检、RW Session 被锁外换根/换锁后停止、锁竞争中移除Store后旧Work不重建，以及创建Store时被换入完整旧SQLite文件仍不修改其 user_version/表/记录。

全仓 Nextest `4bc43a1b-65eb-4de4-b3da-bba47518259b`：552 passed、0 skipped、0 leaky，74.955 秒。`cargo fmt --all -- --check`、全 targets/features `cargo check`、Clippy、Rust 1.85.0 全 targets/features check、test-owner、docs、specs 和离线 `cargo deny` 均通过。`cargo deny` 用 `/private/tmp/sheltie-cargo-home3` 中的本机缓存；完整输出以 `cargo-deny-isolated.stdout.raw.b64` 保存，可用 `base64 -D` 还原，保留既有 winnow/未命中 license allowance 警告。

Spec 与 Standards 两轴独立 Review 均为 PASS。中间全仓运行发现旧的无效-add测试仍期待新库存在，已改为分别断言“已有库行数不变”和“新 Home 不建 Store/锁”；另一次场景测试并发子进程空输出未能稳定复现，独立场景 binary 与最终全仓运行均通过。中间失败日志保存在 `workspace-nextest-*` 文件中；`workspace-nextest-scenario-diagnostic.stdout.raw.b64` 是为保留空行尾空格而编码的原始输出，可用 `base64 -D` 还原。

T24 不关闭后续 T25 恢复协议、T31 全链确定性交错或 C002-M1；旧 Work 等待锁后不重建库的本任务反例使用真实锁竞争点和数据库移除模拟，purge 全链仍由 T31 验证。Linux `not_run`。

完整候选回归另发现并修复了新Store初始化期间另一写者的只读预检竞态；T24 follow-up 的实现、真实add/install同步测试和门禁见[并发Store初始化修复](../t24-followup/README.md)。
