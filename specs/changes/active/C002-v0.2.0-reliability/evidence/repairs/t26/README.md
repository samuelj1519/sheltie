# C002-T26：发布归属与必要同步

候选平台：macOS arm64。Linux原生运行按用户授权保留`not_run`，本任务不作Linux或跨平台PASS结论。输入闭包为本任务提交、`Cargo.lock`、全特性与本机Rust稳定版；另以Rust 1.85.0运行locked check。

## 实现

- Work start发布把`WorkState.inputs`中的每个已提交ArtifactRef保留在`CheckedEffects`中。发布前与落位后均按精确路径、SHA-256和字节数核对`start-inputs/`原件；长度正确但内容错误、单独篡改引用长度、final-only损坏均拒绝。
- Workbook add恢复把sidecar的`format/internal_id/request_id/op`与Store请求逐项核对。发布前与落位后通过受管目录装入manifest，并同时核manifest id/version与摘要。Work和Workbook的请求、审计、响应及效果owner闭包由现有 checked-effects校验绑定。
- 发布四格状态改用管理根目录句柄与`statat(..., SYMLINK_NOFOLLOW)`观察。仅真正的`NOENT`视为缺失；权限、类型、符号链接与其他I/O错误停止恢复。
- `ManagedTree`在受管句柄上贯穿整树文件/目录sync、`NOREPLACE` rename和落位后身份复核。发布后重新核完整对象、Work输入或Workbook manifest/digest、只读权限与最终四格状态。final-only恢复重做原件树、pending源父、final目标父、final目录和只读权限sync；同步失败不得推进`published`。
- 真实I/O错误保留`IO` cause；对象缺失、身份、manifest、摘要或输入字节不符映射为`STORE_CORRUPT`。命名同步故障注入按管理根隔离。

## 反例与真实入口

`task.stdout.txt`运行`scripts/task.sh C002-T26`，Nextest run `8bcc2825-029a-400b-91a1-94069b101a90`：8 passed、577 task-filtered。用例包含：Work输入同长度异字节与引用长度分别篡改；sidecar和PublishDir单字段篡改；final-only输入篡改；Workbook pending/final manifest、摘要和sidecar单字段损坏；不可读文件保持`IO`；sync后payload根目录被替换；真实Workbook add、Work start、status及show入口。

V20使用九个命名点分别注入文件、嵌套目录、payload根目录、rename源父、rename目标父、final根目录、只读文件、只读嵌套目录和只读根目录sync失败。同一失败点重复注入时仍须返回`EFFECT_PENDING`并保持`published=0`；rename后的重试保留原inode，解除故障后重试成功。另以同一start request-id在rename后源父sync失败，连续两次验证`committed=true`、本请求id、revision、完整original与快照原字节；故障解除后原请求重放成功，不新建请求、不重复制原件。

同步后目录替换通过带request-id的真实恢复交错点注入：持有并完成sync的原目录移出pending后，在原路径放入不同inode目录；恢复在NOREPLACE rename前拒绝替代目录，`published=0`，并保留原对象与替代对象。

## 门禁与审查

- `task.stdout.txt`：8 passed；`workspace-nextest.stdout.txt`：run `871f460c-ab00-4531-87c2-179b50b98d4e`，585 passed、0 skipped、1 slow，命令exit 0。
- `fmt.stdout.txt`、`cargo-check.stdout.txt`、`clippy.stdout.txt`通过；`msrv-check.stdout.txt`以Rust 1.85.0运行`cargo +1.85.0 check --locked --all-targets --all-features`。
- `deny.stdout.txt`运行`cargo deny --offline check`并exit 0；使用复制到临时Cargo home的本机advisory DB，未联网。`deny.stdout.raw.gz`是逐字节原始输出；文本副本只去掉cargo-deny表格空行的尾随空格以通过`git diff --check`。最初默认Cargo advisory目录因只读锁路径失败，历史失败原文留在`deny-initial-permission.stdout.txt`和`deny-temp-lock.stdout.txt`。
- `check-docs.stdout.txt`、`check-specs.stdout.txt`、`check-core-vocab.stdout.txt`、`check-tests.stdout.txt`、`check-skill.stdout.txt`和`check-task.stdout.txt`均通过。
- 独立Spec与Standards Reviewer均给出最终PASS；复核者未参与实现。

## 边界

T26只关闭发布归属与必要sync。Linux保持`not_run`；T27删除完成证明、T28 pending清理/只读发现与V25维护诊断、T29一致快照、T30 spec-dev交接、T31故障窗口/突变验证及C002-M1仍待各自门槛。T16真实宿主回归与T17发布仍由真人操作者完成。
