# C002-T21 受限树、摘要与复制

基线：`22942ee`（C002-T20）。最终提交 hash 记录在本任务提交与 package validation。平台：macOS arm64，Rust `1.98.1`，target `aarch64-apple-darwin`；MSRV `1.85.0` 检查通过。`Cargo.lock` SHA-256：`d9e4b72b4810337fa0c260223f2104907347a05da51f3ab146445ce04d295e4f`。所有测试用各自临时目录；管理根不指向真实用户 home。

Linux 原生运行由用户明确豁免，记为 `not_run`。下列结论只覆盖 macOS，不代表 Linux 或跨平台 PASS。

## 实现与实际调用链

| 调用方 | 受限树入口 | 结果 |
| --- | --- | --- |
| `workbook add` 源目录 | `ExternalReadTree::open`；受限扫描、首轮元数据限额、句柄复制；装入最终 pending 副本后重新解析/编译/摘要 | 登记的是已复制候选的实际字节 |
| installed Workbook `load` / `verify` | `ExternalReadTree::open_managed(Home, path)` | Home 目录句柄逐段 no-follow，父目录替换为软链会拒绝 |
| Work start 冻结复制 | 外部只读树 + `copy_tree_confined`；目标经 `ManagedFs` 独占写入 | 复制后从受管冻结树重新加载并校验 |
| Work load / begin | 同一受管树一次读取 manifest、Flow、说明书；一次 inspect 生成摘要、ResourceIndex 和逐文件 hash | `WorkService::load` 同时得到已校验图、资源观察与说明书文本；`begin` 使用该快照，不重开冻结资源/说明书 |
| 发布恢复校验 | `digest_managed_dir_v2` | final/pending 都按 Home 句柄受限扫描 |

`ExternalTreeFileHandle` 只实现 `Read`，不向调用者暴露权限修改能力。目录和文件条目都保存身份；子目录与文件按父句柄 no-follow 打开并比较设备号、inode 和声明长度。两类树扫描都拒绝软链、硬链、FIFO/其他特殊文件及非 UTF-8 名称。macOS `/var`、`/tmp` 仅在读链接确认为系统目标时映射；其他路径先做词法规范化，不通过 `canonicalize(parent)` 吸收任意软链。

摘要继续使用 schema 2 的既有 v2 字节帧、路径字节排序和单次 SHA-256。全树单文件/总字节限额在打开正文前一次核完；读时仍核实际字节数。一次 inspect 同时产生整个目录摘要、每文件 SHA 和 UTF-8/长度事实；只有被编译消费的 manifest、Flow、instruction 文本保留在 `LoadedWorkbook`，资源正文不做整树缓存。

## 关键反例与独立 oracle

- 独立 Python 手工字节向量固定为 `a0f849d6ac09cc3a1fdcf37a5f494c6c3dd881aba1ac5f54ce1a67bc8944bd01`。测试先捕获 manifest 字节，再仅改变同 inode、同长度正文；另一个资源仅替换成同长度非法 UTF-8。摘要必须对应实际编译消费的捕获字节，ResourceIndex 必须对应该次摘要所读资源字节。
- Workbook 摘要正例、O07 碰撞对、字节序、单文件 32 MiB 与总量 256 MiB 的原独立向量均保持不变。
- 单文件恰好上限与多一字节、总量恰好上限与多一字节继续回归。T21 总量反例把排序最前的大文件设为无读权限，后续文件使总量超限；必须先报总量错误，且无 Workbook 行与 pending 目录，证明总量预检先于正文读取和发布准备。
- 文件树内软链、根软链、父目录软链、硬链、特殊文件与非 UTF-8 名称各由拒绝例覆盖；真实 CLI `workbook verify` 拒绝 installed 根软链，真实 CLI `work start` 拒绝 installed id 父软链，外部哨兵原字节保持不变。
- UTF-8 分块状态机覆盖 64 KiB 边界上的合法三字节字符、错误 continuation byte 与 EOF 残尾。
- 真实 CLI `workbook add .` 从 example Workbook 目录执行成功，避免安全路径规范化误拒绝合法点段参数。

## 门禁与运行

所有命令退出码保存在同名 `.exit`，stdout/stderr 分开保存在此目录。Rust 命令使用 `RUSTC_WRAPPER=` 与 `CARGO_TARGET_DIR=/private/tmp/sheltie-c002-t21-target`，避免写入不可用的全局 target。

`cargo deny` 的原始 stderr 含表格空格，为通过 `git diff --check` 以无损 Base64 保存到 `deny.stderr.raw.b64`；用 `base64 -D < deny.stderr.raw.b64` 还原。

| Gate | 命令 | 结果 |
| --- | --- | --- |
| fmt | `cargo fmt --all -- --check` | PASS |
| check | `cargo check --offline --all-targets --all-features` | PASS |
| Clippy | `cargo clippy --offline --all-targets --all-features -- -D warnings` | PASS |
| Nextest | `cargo nextest run --offline --workspace --all-features --no-tests=pass --no-fail-fast` | PASS；run `8cc95446-f6f1-4bfd-91be-8e8653c31956`；502 passed、0 skipped、1 leaky |
| T21 定向 | `scripts/task.sh C002-T21` | PASS；run `d7f8c42d-a6ea-4c97-a86f-9987d2af3a65`；10 passed、492 skipped |
| 暂存区白名单 | `scripts/check-task.sh C002-T21 --staged` | PASS；`check-task: C002-T21 OK` |
| 影响面 | runtime `workbook_digest/workbook_identity/workbook_repo/service` 与 CLI `scenario_article_review/scenario_workbook_lifecycle` | PASS；80 runtime passed、0 skipped；18 CLI passed、1 leaky、0 skipped |
| MSRV | `cargo +1.85.0 check --offline --workspace --all-targets --all-features --locked` | PASS |
| deny | `cargo deny --offline check` | PASS；使用 `/private/tmp/sheltie-cargo-home3` 的本机 advisory DB；既有未命中 license allowance 警告详见原始 stderr |
| 文档与治理 | `scripts/check-docs.sh`、`scripts/check-specs.sh`、`scripts/check-core-vocab.sh`、`scripts/check-tests.sh`、`scripts/check-skill.sh`、`typos`、`git diff --check` | PASS |

完整 Nextest 的单个 leaky 用例为既有 `sheltie-cli::attempt::attempt_begin_returns_brief_path_that_exists`；该测试没有失败。影响面 CLI 的单个 leaky 用例为既有场景测试，原始日志列出其名称。原始运行输出：[nextest](nextest.stderr.txt)、[T21定向](task.stderr.txt)、[影响面runtime](runtime-focused.stderr.txt)、[影响面CLI](cli-focused.stderr.txt)。Linux 为 `not_run`。

## 独立 Review

- Spec：`/root/spec_review` PASS。核对 managed Home-FD 与外部只读入口、六类真实调用方、Work load 到 begin 的图/摘要/资源/说明书快照闭包、摘要向量、链接拒绝、限额顺序与路径点段回归。
- Rust Standards：`/root/standards_review` PASS。核对 Read-only 文件句柄、单次观察结果、二分查找、MSRV、UTF-8 分块校验、调用方迁移与限额内存边界。
- 两位 Reviewer 均未修改或运行代码。结论仅关闭 T21 自身；T22–T31 与最终 M1 尚未关闭。
