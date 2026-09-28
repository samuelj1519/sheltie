# C002-T18 API 探针（macOS 已执行，Linux 按用户指示豁免）

## 运行闭包

运行ID、完整命令、环境、Rust版本摘要、源码/lock/binary SHA-256、外层Python runner 15秒超时、耗时和exit见[macOS运行清单](run-manifest-darwin-arm64.txt)、[stdout](darwin-arm64.stdout)与[stderr](darwin-arm64.stderr)。宿主为`Darwin arm64`；探针源码`src/main.rs`、Cargo.toml、Cargo.lock、运行二进制、编译器与平台输出都保留在本目录。bundled SQLite实际版本为3.50.2。探针 crate 自包含，可直接在Linux runner上用相同manifest与lockfile重跑。

首次编译失败原文保留在`run-first-failed.stderr`：`rustix::fs::mkfifoat`在Apple配置不可用。FIFO仅是探针夹具，改由宿主`mkfifo`创建；runtime不创建FIFO。修订后的源码及后续PASS日志均在本目录，不覆盖首次失败记录。

## macOS 已执行项

- SQLite真实bundled WAL只读查询、活动写者未提交数据不可见、缺`-shm`时建立共享内存控制文件、查询前设置`SQLITE_DBCONFIG_NO_CKPT_ON_CLOSE`；连接关闭后main/WAL字节保持不变。
- schema 1只读识别后拒绝，main字节保持不变且不生成WAL。
- 有效WAL含完整已提交快照时，即使main仅64字节，SQLite仍能读出一行；只读查询后main/WAL字节不变。这是探针发现并写回合同的语义：不得仅按main长度提前拒绝。
- 配对夹具先在main建好表和一行，再向WAL提交第二行并损坏WAL header。SQLite忽略损坏的WAL，从main返回一行；只读打开/查询/关闭没有改变main/WAL字节。不能把“WAL损坏”概括成“SQLite一定报错”，也不应自写WAL解析器猜SQLite忽略状态。
- main、WAL、SHM名称上的软链接通过`statat(SYMLINK_NOFOLLOW)`识别为link并在SQLite调用前的句柄预检处拒绝；三种sidecar的硬链接均由`nlink=2`识别。主库叶链接另由SQLite `NOFOLLOW` flag拒绝。
- rustix 1.1.4目录句柄遍历，祖先与叶软链接拒绝；模拟stat后换入软链接并验证open拒绝；已打开句柄在rename/路径替换后仍指向原对象；外部哨兵字节与权限不变。
- FIFO通过带15秒外层timeout的运行测试：NONBLOCK打开及时返回；独立`open_regular`探针先识别其`FileType::Fifo`并返回错误。句柄测试也覆盖普通文件、硬链接计数、只读0444句柄上的`fchmod`、NOREPLACE不覆盖、成功文件/目录`fsync`及socket上的`fsync`错误返回。

这些是隔离API探针，不是Sheltie真实CLI/runtime caller回归；后者仍属于T19及后续任务。

## 平台豁免与剩余范围

- Linux原生运行：`not_run`，用户于2026-09-28明确要求跳过Linux相关部分。此前确认本机没有Linux daemon的原始信息保留在[运行时探查输出](linux-runtime-availability.log)，用于说明本地环境，不再构成任务阻塞。没有把cross-compile或静态检查记作Linux文件API证据。
- Linux上的SQLite、目录句柄、FIFO、NOREPLACE和fsync全部`not_run`。
- macOS以外的Linux真实runtime caller、Home/Store初始化顺序和产品路径尚未验证，留给T19以后。

按用户授权，T18在macOS证据、合同与独立review通过后可结束；Linux结果仍为`not_run`并作为平台限制随所有后续任务保留。最终不得声称Linux或跨平台已验证。

## T18 仓库门禁与独立复核

完整仓库门禁运行ID和命令见[workspace-gates-darwin-arm64.txt](workspace-gates-darwin-arm64.txt)，每条命令的原始stdout/stderr按名称保留。`git diff --check`、docs/specs/test映射检查、fmt、check、Clippy、nextest全部exit 0；nextest为464 passed、0 skipped。T18合同delta经独立Spec与Standards复核通过；复核未覆盖后续生产caller或M1。

仓库四条门禁在macOS stable执行；T18小探针本体以MSRV Rust 1.85.0 `--offline --locked` 编译并运行。该工具链未安装rustfmt/Clippy组件，探针格式与Clippy使用stable Rust 1.98.1检查并留存日志。Linux豁免仅改变平台验证要求，不豁免上述任一门禁。

提交钩子首次运行发现仓库钩子会规范化machine evidence字节，并把审查表里的真实commit短哈希自动改写成拼写建议。为保留原始证据且防止哈希损坏，本任务让所有修改型钩子和typos跳过active/completed change下的machine evidence扩展名（含BOM、EOF、行尾、空白、拼写自动修复），并只排除C002 M1 review与两份hash-bearing轴报告在active/completed中的精确路径；其他人工Markdown仍受文档和拼写检查。修改和范围记录在T18白名单内。变更前已被钩子改写的工作树恢复为索引原始版本。`pre-commit validate-config`、任务白名单和文档门禁通过；完整commit钩子在修订后重跑。
