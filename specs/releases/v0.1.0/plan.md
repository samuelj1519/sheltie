# MVP 实现计划

状态：`closed`。T01–T26 与 M1–M3 已完成，本文保留 MVP 的任务、复核与验证历史，不再追加后续版本任务。当前迭代入口见 [change 索引](../../changes/README.md)，发布闭包见 [v0.1.0 release record](README.md)。

从 `Hello, world!` 到能跑完 [规格 §7](../../spec.md) 全部验收场景的 `sheltie` 二进制。

计划按「强模型搭骨架、初级实现者填空、机器当审查」的方式执行。T01 由一个强模型一次性写出全部 crate 骨架、全部测试（先禁用）与检查脚本；T02 起每个任务是「启用一组测试，填满几个 `todo!()`，让它们变绿」，由初级开发者或初级模型完成。实现者不做设计决定、不写测试、不改签名。编译器、测试与脚本承担逐任务审查；模型审查只在三个里程碑各做一次。

进度只看状态列：`todo | doing | done | blocked`。改状态的提交和改代码的提交可以是同一个。

## 0. 执行方式

### 0.1 三层分工

| 层 | 谁 | 做什么 | 什么时候 |
| --- | --- | --- | --- |
| 骨架 | 强模型 | 全部类型、签名、文档注释、`todo!()` 函数体、全部测试与快照、脚本、样例 Workbook | T01，一次 |
| 填空 | 初级开发者或初级模型 | 按任务卡启用测试、填函数体、跑门禁、提交 | T02 到 T23，每任务一次 |
| 审查 | 编译器、clippy、测试、`scripts/check-task.sh` | 每任务自动 | 每次提交 |
| 审查 | 强模型 | 按 [engineering.md §5](../../engineering.md) 检查表审里程碑之间的 diff，跑 `cargo mutants` 看存活的突变体 | M1、M2、M3，共三次 |

### 0.2 实现者的十条规则

写给做 T02 到 T23 的人或模型。每条都是硬规则，违反任何一条 `scripts/check-task.sh` 会失败。

1. **只读三样东西。** 本任务的任务卡（下面 §2 对应一节）、任务卡「文件」列出的源码文件、任务卡「测试」对应的测试代码。不读整个仓库。合同只在任务卡指名「按某合同某节」时去读那一节。
2. **先启用测试。** 删掉本任务测试上的 `#[ignore = "Tnn"]`，运行 `scripts/task.sh Tnn`，看到它们全红。编译错误不算红，先让它编译。
3. **只填 `todo!()`。** 把「文件」里函数体的 `todo!()` 换成实现。不改函数签名，不改类型定义，不加 `pub`，不加依赖，不新建文件。文档注释就是这个函数要做的事。同文件内新增私有辅助函数是实现细节，允许；它们不得改变任何公开项的行为边界。
4. **不改测试，不改快照。** 测试是合同。测试红了改实现，不改断言。快照不一致改渲染代码，不 `cargo insta accept`。
5. **一次一个测试。** 挑一个红的，让它绿，再挑下一个。不要一次改很多再跑。
6. **绿了跑门禁。** `scripts/task.sh Tnn` 全绿后，把状态列改为 `done`，跑 [engineering.md §2.3](../../engineering.md) 四条命令与 `scripts/check-task.sh Tnn`。
7. **一次提交。** 提交信息按 [engineering.md §4](../../engineering.md) 的格式，第一行用任务卡「提交」那一行，末尾 `Task: Tnn` 与 `Agent: <名字>`。改状态列为 `done` 放进同一提交。提交后再跑 `scripts/check-task.sh Tnn`，这时补查提交信息。
8. **不顺手改。** 看到别的 `todo!()`、别的任务的测试、觉得能优化的地方，都不碰。
9. **卡住就停。** 下面任何一种情况，停手、不提交、把状态列改成 `blocked`、在提交信息或聊天里写清原因：两个测试互相矛盾；不改签名或测试就做不到；需要新依赖；同一个测试改了五次还红；任务卡和合同说的不一样。
   **唯一例外：工具本身有缺陷。** `scripts/` 下的脚本或 `tasks.toml` 让一个按规则做的任务无法通过时，可以修工具，但要单独一个提交、`Task: T01`、提交说明逐条写原来错在哪与改成什么；工具修复不得放松任何检查的意图，不动测试、签名与合同。M1 到 M3 复核每一次工具改动。
10. **不猜。** 文档注释、测试、合同都没说的行为，不自己发明。按第 9 条卡住。在注释里写下「这部分由 runtime 补」「解析不了当 0」「出错就用默认内容」一类约定，同样是发明。

### 0.2.1 实现者是强模型时

十条规则不变，多两条便利：

- 可以在一个会话里连做多个相邻任务（例如 T03 到 T05），省重复加载上下文；仍然一任务一提交，每个任务各跑一遍 `task.sh` 与 `check-task.sh`。
- 复核发现的缺陷由实现者自己修。复核者只补测试（加 `#[ignore = "Tnn"]`，任务状态改回 `doing`），实现者启用测试、修代码、再提交一次 `fix(<scope>)`，`Task: Tnn`。测试仍归骨架与复核者，实现者仍不写测试。

### 0.3 每个任务的成本

任务卡约 300 字，「文件」里的骨架每个任务 100 到 400 行，测试 100 到 500 行。实现者一个任务的输入上下文在 3k 到 8k token 之间，不含编译器输出。全部 22 个填空任务合计输入约 15 万 token；每个任务失败重试一次的预算翻倍。三次里程碑审查各读一段 diff，每次约 2 到 5 万 token。

### 0.4 工具

| 命令 | 作用 |
| --- | --- |
| `scripts/task.sh Tnn` | 只跑本任务的测试，禁用的也跑（`--run-ignored all`），所以不删标记也能看到红。归属看测试上方的 `// Task: Tnn` 注释，不看名字。零个测试匹配视为失败 |
| `scripts/check-tests.sh` | 测试名不带任务前缀；每个 `#[test]` 紧贴上方有且只有一行 `// Task: Tnn`，`#[ignore = "Tnn"]` 与它一致；测试名全仓唯一；本文每张任务卡「测试」行列的名字都存在且归属该任务。pre-commit 与 CI 跑 |
| `scripts/check-task.sh Tnn [base] [--staged]` | 核对：基准以来提交说明含 `Task: Tnn` 的提交的改动与未提交改动的并集，都在 `tasks.toml` 该任务的 `files` 与 `test_files` 里（`plan.md`、`tasks.toml` 始终允许）；`files` 里没有 `todo!("Tnn")` 与不带标签的 `todo!()`（条目可为文件或目录，目录只取其中的 `.rs`，文档会引用 `todo!()` 字样），`#[allow(unused_variables)]` 只准留在还有 `todo!()` 的函数上；没有残留 `#[ignore = "Tnn"]`；`test_files` 相对基准（默认最近一个 `tNN-*` tag：`t01-skeleton` 或复核者打的 `tNN-review`）的测试代码零改动，只允许删 `#[ignore` 行，与 `files` 重叠的混合源文件比对 `#[cfg(test)]` 起的测试模块，快照零改动（`allow_test_changes = true` 的任务除外）；`plan.md` 该任务状态为 `done`；提交信息含 `Task: Tnn` 与 `Agent:` 两行（工作树干净时才查，`--staged` 跳过） |
| `tasks.toml` | 机器可读的任务表，与本文 §2 同源，T01 生成，之后改本文必改它。写法：<br>`[T05]`<br>`files = ["crates/sheltie-core/src/flow/compile.rs"]`<br>`test_files = ["crates/sheltie-core/src/flow/compile.rs"]`（允许删禁用标记的文件）<br>`allow_test_changes = false` |
| `scripts/mutants.sh <crate>` | 里程碑用。包一层 `cargo mutants`：目标目录固定在仓库内（全局 `target-dir` 会让并行副本共用产物，结果作废），用 nextest，排除 `testkit.rs`。给源码注入突变，看测试能否杀死。存活的突变体就是没被测到的逻辑 |

### 0.5 测试命名

测试名写「条件 → 行为」，例如 `rejects_self_loop_edge`，不带任务编号、工单号这类排期信息。归属写在紧贴 `#[test]` 上方的注释里：

```rust
// Task: T05
#[test]
#[ignore = "T05"]
fn rejects_self_loop_edge() { … }
```

测试属于哪个任务以 §2 任务卡为准，注释是它在代码里的机器可读副本，`scripts/check-tests.sh` 核对两者一致。复核者补的测试挂被复核任务的编号，不必回写任务卡。快照用显式名字（`insta::assert_snapshot!("brief_for_review_node", …)`），测试改名不牵动快照文件。

### 0.6 复核记录

每个任务完成后由强模型复核一次，结论写进任务卡的「复核」行：日期、通过与否、待修项编号。待修项用 `Bn` 编号，修复提交说明里引用它。复核者补的测试列在同一行；补测试的提交打 tag `tNN-review`，`check-task.sh` 以它为新的测试基准，实现者的修复提交才不会被判为「改了测试」。

里程碑审查退回的修复也一样：提交写被退回任务的编号（`Task: T10`），不写 `Task: M1`。`check-task.sh` 按 `Task:` 找白名单，M1 没有白名单，写成 M1 就等于没核。复核者自己的补测试、改合同提交才写 `Task: Mn`。

退回的缺陷若动了签名或数据结构、全仓夹具一起红（夹具都依赖它），由复核者（骨架作者）直接改完并记进决策记录；边界清楚、能留成禁用测试的才交实现者。分不开的拆两半会让夹具在中间态无法编译。

## 1. 任务表

| ID | 状态 | 执行者 | 标题 | 结果 |
| --- | --- | --- | --- | --- |
| T01 | done | 强模型 | 骨架、全部测试与脚本 | 三个 crate 可编译，全部测试存在且禁用，`scripts/task.sh T02` 能跑出红 |
| T02 | done | 初级 | core 基础类型 | ID、路径、有界文本、摘要 newtype 与错误枚举 |
| T03 | done | 初级 | core 解析 `workbook.toml` | 合法 manifest 解析；未知字段与错 schema 拒绝 |
| T04 | done | 初级 | core 解析 Flow | 节点、边、输入来源、输出声明解析 |
| T05 | done | 初级 | core 编译图 | 合同 §4 九条规则各有拒绝例 |
| T06 | done | 初级 | core `Start` | 新 Work 状态与首个 `next` |
| T07 | done | 初级 | core `BeginAttempt` | 选边、访问计数、输入冻结、任务书效果 |
| T08 | done | 初级 | core `SubmitAttempt` 与 `FailAttempt` | 输出合同、摘要上限、门槛阻断、重试耗尽 |
| T09 | done | 初级 | core `ApproveGate` 与 `Cancel` | 门槛放行、取消、终态拒写 |
| T10 | done | 初级 | core 渲染 | 任务书、状态卡、`next` 命令行与预写快照一致 |
| M1 | done | 强模型 | 里程碑审查：core | diff T01..T10；`scripts/mutants.sh sheltie-core` 存活的突变体逐条处置 |
| T11 | done | 初级 | 样例 Workbook 编译测试 | 三份样例与 `spec-dev` 全部编译通过 |
| T12 | done | 初级 | runtime 管理根与文件观察 | `SHELTIE_HOME`、`confine()`、`ObservedFile` |
| T13 | done | 初级 | runtime SQLite 存储 | 建库、结构校验、`commit()` 去重与 CAS、序号分配 |
| T14 | done | 初级 | runtime Workbook 仓库 | `add / list / load` 含 staging 与只读 |
| T15 | done | 初级 | runtime Workbook `remove` 与 `verify` | 引用检查、摘要核对 |
| T16 | done | 初级 | runtime Work 服务 | 观察 → 决定 → 提交 → 效果；`start` 冻结 Workbook 副本；库级端到端 |
| M2 | done | 强模型 | 里程碑审查：runtime | diff M1..T16；`scripts/mutants.sh sheltie-runtime`；崩溃窗口人工走查 |
| T17 | done | 初级 | cli `workbook` 组 | `add / list / show / remove / verify`，`--json`，退出码 |
| T18 | done | 初级 | cli `work` 组 | `start / list / status / stats / cancel` |
| T19 | done | 初级 | cli `attempt` 与 `gate` 组 | 两步样例从 CLI 走完 |
| T20 | done | 初级 | cli `self` 组与发布链 | `install / update / rollback / uninstall / version`；`cargo-dist` 配置 |
| T21 | done | 初级 | 场景：审查回环 | `back` 边、二次到达、`max_visits` 耗尽、`human` 执行者、`resource` 输入 |
| T22 | done | 初级 | 场景：门槛、产物与 Workbook 生命周期 | 门槛阻断、`ARTIFACT_MODIFIED`、`OUTPUT_MISSING`、`WORKBOOK_IN_USE` |
| T23 | done | 初级 | 场景：重放与崩溃 | 同 id 重放、载荷冲突、`COMMIT` 前后被杀、`self update` 中途被杀 |
| M3 | done | 强模型 | 里程碑审查：端到端 | diff M2..T23；对照 [规格 §7](../../spec.md) 十三个场景逐条找到测试 |
| T24 | done | 强模型 | sheltie skill | `SKILL.md` 与机械检查 |
| T25 | done | 人 | 收口 | 用 `install.sh` 从零装起走通快速开始、`cargo deny`、`v0.1.0` |
| T26 | done | 人 | 真实宿主实测 | 在 Claude Code 里用 skill 走完一个样例，记录结果 |

## 2. 任务

每张任务卡给：结果（做完能观察到什么）、文件（只能改这些）、测试（要变绿的测试名）、实现要点（填空时的提示，不是设计）、提交（信息第一行）。

### T01 骨架、全部测试与脚本

**执行者。** 强模型。这是唯一一个做设计的任务，之后的任务都是填空。

**结果。** 仓库有完整可编译的骨架：类型、签名、文档注释齐全，函数体全是 `todo!()`；全部测试写好并 `#[ignore]`；快照预写；三份样例 Workbook 就位；脚本与 `tasks.toml` 就位；CI 全绿；`scripts/task.sh T02` 跑出非零个红测试。

**文件。**

- 根 `Cargo.toml` 改为虚拟 workspace，`[workspace.dependencies]` 集中版本：`clap`（derive）、`serde`、`serde_json`、`toml`、`thiserror`、`rusqlite`（bundled）、`uuid`（v7）、`sha2`、`camino`、`axoupdater`、`insta`、`assert_cmd`、`tempfile`、`proptest`。删根 `src/main.rs`。
- `crates/sheltie-core/src/`：`lib.rs`、`ids.rs`、`path.rs`、`text.rs`、`digest.rs`、`error.rs`、`workbook/{mod,manifest}.rs`、`flow/{mod,def,parse,compile,graph}.rs`、`work/{mod,state,command,decide,next,render}.rs`。每个公开类型与函数带文档注释，注释第一行写它对应的合同章节，第二行写要返回的错误。函数体 `todo!()`。
- `crates/sheltie-runtime/src/`：`lib.rs`、`home.rs`、`observe.rs`、`error.rs`、`store/{mod,schema,commit,read}.rs`、`workbook_repo.rs`、`service.rs`、`selfmgmt.rs`、`failpoint.rs`。`schema.rs` 的建表语句是完整常量，不是 `todo!()`。
- `crates/sheltie-cli/src/`：`main.rs`、`cli.rs`（完整 `clap` 命令树，这是协议的机器形式，由骨架定死）、`output.rs`、`error_map.rs`、`commands/{workbook,work,attempt,gate,self_cmd}.rs`。
- 测试：T02 到 T23 每张任务卡列出的全部测试，按 [engineering.md §3.2](../../engineering.md) 的分层放置，按 §0.5 命名并挂 `// Task: Tnn`，全部 `#[ignore = "Tnn"]`。`insta` 快照文件按 [协议 §4、§6](../../contracts/protocol.md) 手写。fixture：`tests/fixtures/manifest/*.toml`、`tests/fixtures/flow/*.toml`、`tests/fixtures/old-shape.db`、`tests/fixtures/release/`。
- `examples/two-step/`、`examples/article-review/`、`examples/gated-release/` 按 [Workbook 合同 §6](../../contracts/workbook.md)。`two-step` 是「列提纲 → 按提纲写摘要」；`article-review` 的审查节点用 `resources/review-checklist.md` 作输入；`gated-release` 是「生成发布说明（gate）→ 归档」。三份都不带 `requires`。
- `tasks.toml`、`scripts/task.sh`、`scripts/check-task.sh`、`scripts/check-core-vocab.sh`（对 `crates/sheltie-core/src` 与 `crates/sheltie-runtime/src` grep `Verdict|Pass\b|Fail\b|Review|PackageId|PackageCatalog`，另对 core 查 `std::fs`）。
- `.github/workflows/build.yml` 加三个脚本；`.pre-commit-config.yaml` 同步；`.config/nextest.toml` 把 `crash` 测试设为串行。
- 每个 crate 根 `#![forbid(unsafe_code)]`，lib 加 `#![deny(clippy::unwrap_used, clippy::expect_used)]`。

**骨架的写法。** 类型按 [架构 §2](../../architecture.md)，逐字段核对合同。`Error` 枚举一次写全 [协议 §7](../../contracts/protocol.md) 的 23 个码，每个变体带定位字段。`Command`、`Effect`、`NextOp`、`WorkStatus`、`BlockedReason` 一次写全。`decide` 拆成每个 `Command` 一个私有函数，签名与文档注释写好，体为 `todo!()`。一个函数只做一件事，长度以初级实现者一次能填完为准；复杂的拆成多个私有函数，各自 `todo!()`，各自有注释。测试里用到的构造 helper（`fixture_graph()`、`ObservedFile::for_test()`、固定时钟）在骨架里写完实现，不留给填空。

**验证。** 四条门禁绿；`cargo nextest run --all-features` 显示全部测试 ignored、零失败；`scripts/task.sh T02` 退出非零且列出 T02 的测试名；`scripts/check-docs.sh`、`check-core-vocab.sh` 绿；`cargo deny check` 绿。另做一次正例试跑：临时填满一个最小任务，跑 `check-task.sh` 必须通过，再把填充撤掉。只测负例会漏掉「按规则做也过不了」的脚本缺陷。再对每个填空任务核对依赖顺序：只启用本任务的测试时，panic 的都是本任务的 `todo!("Tnn")`；碰到后面任务的 `todo!` 说明顺序或夹具有缺陷，调顺序或改夹具，不留给实现者跨任务填（M1 教训，T07 实测）。

**教训（T02 复核）。** T01 只验证了 `check-task.sh` 的负例。实现者发现三个缺陷（测试与源码同文件时检查 4 必然失败；检查 2 与规则 8 在共享文件上冲突；检查 1 把历史提交都算进本次），自行修复并单独提交。规则 9 的例外与「正例试跑」由此加入。

**提交。** `chore(workspace): 搭起三 crate 骨架、全部禁用测试与任务工具`。提交后打 tag `t01-skeleton`，`check-task.sh` 以它为基准比对测试文件。

### T02 core 基础类型

**结果。** `sheltie_core::ids::{WorkbookId, FlowId, NodeId, WorkId, WorkName, AttemptId}`、`sheltie_core::path::RelPath`、`sheltie_core::text::BoundedText<N>`、`sheltie_core::digest::Sha256Hex` 构造即校验。

**文件。** `crates/sheltie-core/src/{ids,path,text,digest}.rs`。

**测试。** `workbook_id_accepts_kebab_case`、`workbook_id_rejects_uppercase_and_double_dash`、`rel_path_rejects_dotdot_and_absolute`、`bounded_text_rejects_over_limit_bytes`（按字节不按字符）、`sha256_hex_requires_64_lowercase_hex`、`attempt_id_formats_as_node_hash_n_dot_retry`、`work_id_builds_from_day_seq_and_name`（`2026-09-24`、`3`、`文章-初稿` 得到 `2026-09-24-003-文章-初稿`）、`work_name_normalizes_whitespace_and_case`、`work_name_rejects_over_48_bytes_and_bad_chars`、`work_id_rejects_seq_zero_or_over_999`。

**实现要点。** ID 正则与长度按 [Workbook 合同 §2](../../contracts/workbook.md)。`RelPath` 拒绝 `..`、绝对路径、空段。`WorkName::normalize` 与 `WorkId::new` 按 [协议](../../contracts/protocol.md) `work start` 第 3、4 步。字节长度用 `str::len`，不用 `chars().count()`。序号与到达次数的数字部分必须全是 ASCII 数字，`str::parse` 接受的前导 `+` 要另行拒绝。

**提交。** `feat(core): 强类型 ID、相对路径、有界文本与摘要`

**复核（2026-09-24，两轮）。** 通过。第一轮列出 B1 到 B4；实现者复核了这份复核，指出两处事实不实（`text.rs` 换了一行 `use`；新增了两个私有函数）与三处漏检（`check-task.sh` 第 124 行全角冒号让 `set -u` 中止，负例其实在崩溃；`manifest.rs:114` 有 T01 残留的 `#[allow]` 会让 T03 必然过不了检查 2；检查 1 的两条路径互斥）。全部核实成立，见 [decisions.md D-25](decisions.md)。

归属与状态：

- 复核者（骨架）已改：`manifest.rs:114` 残留删除；`kebab_id!` 的错误 `field` 改为 `workbook_id / flow_id / node_id`（B3 根因）；协议第 3 步改为列出十二个码点区间（B2）；`tasks.toml` 加 `[T01]` 白名单，工具提交不再免检；规则 3 明确允许私有辅助函数。补测试五条并禁用：`work_id_parse_rejects_plus_sign_in_seq`、`attempt_id_parse_rejects_plus_sign`、`parse_rejects_leading_zero_in_numeric_segments`、`work_name_accepts_han_extension_f_h_i_and_compat_supplement`、`attempt_id_parse_error_field_is_attempt_id`。
- 实现者待修（`Task: T02`）：B1 扩到 `AttemptId::parse` 的到达次数与重试段，数字段必须是规范十进制（无前导 `+`、无前导零，`0` 本身除外）；B2 `is_han` 补扩展 F、H、I 与兼容补充区，与协议列表逐区间一致；B3 `AttemptId::parse` 的 `field` 归位。修复提交 `fix(core): 数字段拒绝前导加号与前导零，Han 区段补齐，错误字段归位`。
- 实现者待修（`Task: T01`，工具）：`check-task.sh` 第 124 行 `$residue：` 改 `${residue}：`；检查 1 改为「基准以来提交说明含 `Task: Tnn` 的提交的改动 ∪ 未提交改动」，两条路径不再互斥。

### T03 core 解析 `workbook.toml`

**结果。** `parse_manifest(&str) -> Result<Manifest, Error>`。

**文件。** `crates/sheltie-core/src/workbook/manifest.rs`。

**测试。** `parses_minimal_manifest`、（原列 rejects_unknown_field：C002-T40 加强并迁移归属；当前保留 `rejects_unknown_field`；见 [T40验证](../../changes/active/C002-v0.2.0-reliability/validation.md#c002-t40-测试精简验证)）、`rejects_wrong_schema_string`、`rejects_empty_flows`、`rejects_flow_path_with_dotdot`、`rejects_description_over_2kib`、`parses_requires_with_optional_fields`、`rejects_duplicate_require_kind_name`、`rejects_unknown_require_kind`、`rejects_require_digest_without_sha256_prefix`。

**实现要点。** DTO 已在骨架里带 `#[serde(deny_unknown_fields)]`，填的是 DTO 到 `Manifest` 的逐字段转换。错误的 `field` 用 `flows[1]` 这种路径写法。

**提交。** `feat(core): 解析 workbook.toml`

### T04 core 解析 Flow

**结果。** `parse_flow(&str) -> Result<FlowDef, Error>`，含三种 `InputSource`、`required`、节点 `requires`。

**文件。** `crates/sheltie-core/src/flow/parse.rs`。

**测试。** `parses_three_node_flow`（用 [合同 §3](../../contracts/workbook.md) 的样例字节）、`instruction_requires_exactly_one_of_file_or_text`、`input_from_parses_start_resource_and_node_forms`、`input_from_parses_engine_stats_only`、`input_from_rejects_three_segments_for_start_and_node`、`input_from_resource_keeps_slashes_in_path`、`node_requires_parses_kind_colon_name`、`node_requires_rejects_bad_kind`、`input_required_defaults_true_and_parses_false`、`tier_defaults_standard_and_parses_strong`、`defaults_gate_false_visits_1_retries_1`、`rejects_max_visits_zero_or_over_32`（原列 rejects_output_path_brief_md：C002-T03 起输出与引擎文件分目录，该拒绝取消，由 C002-T03 的 accepts_output_paths_named_brief_or_stats 覆盖，见 [C002历史验证索引](../../changes/active/C002-v0.2.0-reliability/validation.md#历史证据恢复)）、`rejects_unknown_edge_kind`。

**实现要点。** `InputSource::from_str` 先按第一个 `.` 切，前缀是 `start` 或 `resource` 走对应分支，否则是 `node.output` 且只允许一个 `.`。`resource.` 后面的路径可以含 `/`。默认值按合同 §3.2 表。

**提交。** `feat(core): 解析 flow/v1 的节点、边与输入来源`

### T05 core 编译图

**结果。** `compile(def, manifest, res) -> Result<Graph, Error>`，实现 [合同 §4](../../contracts/workbook.md) 规则 1 到 9。

**文件。** `crates/sheltie-core/src/flow/compile.rs`。

**测试。** `rejects_entry_not_a_node`、`rejects_self_loop_edge`、`rejects_duplicate_from_to`、`rejects_unreachable_node`、`rejects_graph_without_terminal_node`、`rejects_input_from_unknown_output`、`rejects_input_from_node_that_cannot_reach_consumer`、`rejects_gate_node_with_empty_text`、`rejects_missing_instruction_file`、`rejects_non_utf8_instruction`、`rejects_node_id_start_or_resource`、`rejects_node_id_engine`、`rejects_missing_resource_input_file`、`accepts_binary_resource_input`、`rejects_node_require_not_declared_in_manifest`、`rejects_duplicate_node_require`、（原列 rejects_optional_input_on_start_or_resource_source：C002-T40 加强并迁移归属；当前保留 `rejects_optional_input_on_start_or_resource_source`；见 [T40验证](../../changes/active/C002-v0.2.0-reliability/validation.md#c002-t40-测试精简验证)）、`rejects_optional_engine_stats_input`、`rejects_tier_on_human_node`、`compiles_article_review_example`、`proptest_compile_never_panics`。

**实现要点。** 骨架里已把九条规则拆成九个私有函数 `check_rule_1` 到 `check_rule_9`，各自 `todo!()`，按顺序调用。可达性用 BFS，骨架里有 `reachable_from(&out_edges, start) -> BTreeSet<NodeId>` 的签名，先填它。错误的 `rule` 字段填 `"1"` 到 `"9"`。

**提交。** `feat(core): 把 Flow 编译成校验过的图`

**复核（M1，2026-09-25，三轮后通过）。** 需修改。B1：合同 §3.2 说可选输出「下游不得把它当必需输入」，§4 规则 5 与骨架注释都漏了这一句，编译放行后 `next` 会给出一个必然 `INPUT_UNAVAILABLE` 的 `begin`，只能取消。合同与 `check_rule_5` 注释已补；复核者补测试 `rejects_required_input_on_optional_output`（禁用，待启用）与 `accepts_optional_input_on_optional_output`。缺口在上游文档，不算实现者违规。修完提交 `fix(core): 规则 5 拒绝把可选输出当必需输入`，`Task: T05`。第三轮复审确认修复（5f4b6d6），另补交叉声明测试 `begin_reply_requires_ignore_crossed_kind_name_pairs`（挂 T07）。见 [decisions.md 里程碑记录](decisions.md)。

### T06 core `Start`

**结果。** `decide` 对 `Command::Start` 返回新 `WorkState`；`legal_next` 对新状态返回 `attempt begin <entry>` 与 `work cancel`。

**文件。** `crates/sheltie-core/src/work/{decide,next}.rs` 中的 `decide_start`、`legal_next`。

**测试。** `start_sets_current_to_entry_occurrence_1`、`start_rejects_missing_start_input_key`、`start_rejects_extra_start_input_key`、`start_next_is_begin_entry_and_cancel`、`start_records_workbook_ref_and_frozen_inputs`。

**实现要点。** `legal_next` 骨架是一个对 `status` 与最新 Attempt 的 `match`，本任务只填「终态」与「当前 Occurrence 无 Attempt」两个分支，其余分支保留 `todo!()` 给 T07、T08。缺键与多键都是 `INPUT_MISSING`，`detail` 说明是缺还是多。

**提交。** `feat(core): Work 状态、Start 命令与首个合法下一步`

### T07 core `BeginAttempt`

**结果。** `Command::BeginAttempt` 按 [协议](../../contracts/protocol.md) `attempt begin` 细则；`Effect::WriteBrief`。T07 测试的夹具会 `submit` / `fail`，`decide` 入口先过 `guard_not_terminal`，`IllegalNext.next` 与任务书、`engine.stats` 又分别要 `to_command_line`、`render_brief`、`render_stats_json`。因此本任务一并填 `decide_submit`、`check_outputs`、`status_after_success`、`decide_fail`、`output_paths_for`、`guard_not_terminal` 与上述三个渲染函数（原属 T08–T10）；不这样，T07 测试在夹具阶段就 `todo!()` 或 `WORK_TERMINAL` 直接崩。

**文件。** `crates/sheltie-core/src/work/{decide,next,render}.rs` 中的 `decide_begin`、`bind_inputs`、`input_paths_for`、`decide_submit`、`check_outputs`、`status_after_success`、`decide_fail`、`output_paths_for`、`guard_not_terminal`、`legal_next` 的「最新 Attempt `Succeeded`」与「`Failed` 且可重试」分支、`NextOp::to_command_line`、`render_brief`、`render_stats_json`。

**测试。** `begin_on_entry_creates_running_attempt_with_frozen_inputs`、`begin_rejects_node_not_in_next`（`ILLEGAL_NEXT` 且 `detail.next` 等于 `legal_next`）、`begin_via_edge_increments_visits_and_occurrence`、`begin_filters_edges_whose_target_hit_max_visits`、`begin_rejects_modified_upstream_artifact`、`begin_rejects_upstream_without_succeeded_attempt`、`begin_leaves_optional_input_unbound_when_upstream_has_no_attempt`、`begin_binds_optional_input_when_upstream_succeeded_later`、`begin_after_failed_attempt_increments_retry_not_occurrence`、`begin_binds_resource_input_under_frozen_workbook_dir`、`begin_reply_lists_node_requires`、`begin_records_entered_from_occurrence_and_edge_kind`、`retry_keeps_entered_from_of_first_attempt`、`begin_binds_engine_stats_and_emits_write_file`、`begin_emits_write_brief_effect`。

**实现要点。** 先填 `legal_next` 的两个分支再填 `decide_begin`，因为后者第一步就是「`node` 在 `legal_next` 里吗」。`bind_inputs` 对每个 `InputDecl` 分三种来源处理；`Node` 来源找该节点最新 `Succeeded` 的 Attempt，找不到时看 `required`。摘要比较用 `ObservedFile.sha256 == ArtifactRef.sha256`。

**提交。** `feat(core): BeginAttempt 选边、计数与输入冻结`

### T08 core `SubmitAttempt` 与 `FailAttempt`

**结果。** `legal_next` 的「`Running`」与「`Blocked`」分支。提交与失败的 `decide_*`、输出合同与 `status_after_success` 已在 T07 填（顺序原因见 T07），本任务启用测试核对它们，再补 `legal_next` 剩余分支。

**文件。** `crates/sheltie-core/src/work/next.rs` 中的 `legal_next` 剩余分支。

**测试。** `submit_marks_attempt_succeeded_and_records_outputs`、`submit_rejects_when_attempt_not_running`、（原列 submit_rejects_summary_over_4096_bytes：C002-T40 加强并迁移归属；当前保留 `submit_rejects_summary_over_4096_bytes`；见 [T40验证](../../changes/active/C002-v0.2.0-reliability/validation.md#c002-t40-测试精简验证)）、`submit_rejects_missing_required_output`、`submit_accepts_missing_optional_output`、（原列 submit_rejects_output_over_max_bytes：C002-T40 加强并迁移归属；当前保留 `submit_rejects_output_over_max_bytes`；见 [T40验证](../../changes/active/C002-v0.2.0-reliability/validation.md#c002-t40-测试精简验证)）、`submit_on_gate_node_blocks_work`、`submit_on_terminal_node_succeeds_work`、`submit_on_terminal_gate_node_blocks_not_succeeds`、`submit_when_every_out_edge_target_hit_max_visits_blocks_no_legal_edge`、`fail_marks_attempt_failed_and_allows_retry`、`fail_at_max_retries_blocks_work`、`next_after_success_lists_out_edges_with_kind`、`next_when_blocked_gate_has_only_approve_and_cancel`。

**实现要点。** `check_outputs` 校验失败返回 `Err`，此时不动状态。`status_after_success` 是一个按顺序的判断：`gate` → 无出边 → 无合法边 → `Active`，[协议](../../contracts/protocol.md) `attempt submit` 第 5 步写了顺序，照抄。`Effect::SealOutputs` 与 `Effect::RefreshStatusCard` 在成功路径末尾追加。

**提交。** `feat(core): SubmitAttempt 与 FailAttempt 及输出合同校验`

### T09 core `ApproveGate` 与 `Cancel`

**文件。** `crates/sheltie-core/src/work/decide.rs` 中的 `decide_approve`、`decide_cancel`。（`guard_not_terminal` 已在 T07 填。）

**测试。** `approve_unblocks_and_records_principal_and_time`、`approve_rejects_when_not_blocked_on_gate`、`approve_rejects_wrong_node`、`approve_on_terminal_node_succeeds_work`、`cancel_from_active_and_blocked`、`terminal_work_rejects_every_command_with_work_terminal`。

**实现要点。** `decide_approve` 复用 T07 的 `status_after_success`，只是跳过 `gate` 那一步。

**提交。** `feat(core): 门槛批准、取消与终态守卫`

### T10 core 渲染

**结果。** `render_status_card`、`render_stats`、`status_card_json` 与骨架预写的快照逐字节一致。`render_brief`、`render_stats_json`、`NextOp::to_command_line` 已在 T07 填，本任务对着快照校正格式。

**文件。** `crates/sheltie-core/src/work/render.rs` 中的 `render_status_card`、`render_stats`、`status_card_json`。

**测试。** 快照：`brief_for_review_node`、`brief_for_node_with_requires`、`brief_for_node_without_requires_omits_section`、`brief_marks_unbound_optional_input_as_absent`、`brief_for_human_executor_ends_with_submit_command`、`brief_shows_entered_from_line_or_entry`、`status_card_active_mid_flow`、`status_card_blocked_on_gate`、`status_card_succeeded`、`stats_table_mid_flow`。断言：`next_op_renders_begin_with_node_flag`、`next_op_begin_carries_executor_and_tier`、`status_card_lists_done_occurrences_in_order`、`stats_json_counts_visits_failures_and_entered_via`。

**实现要点。** 格式按 [协议 §4、§6](../../contracts/protocol.md)。快照就是标准答案，红了看 `insta` 的 diff 逐行对。不 `cargo insta accept`。

**提交。** `feat(core): 渲染任务书、状态卡与 next 投影`

**复核（M1 第二轮，2026-09-25）。** 退回 `doing`，两项，一个提交 `fix(core): …`，`Task: T10`：

1. 任务书宿主资源表的说明写「此 <kind>」（协议 §4）。启用 `brief_for_node_with_requires`、`brief_require_row_takes_version_from_manifest_and_names_kind`。`render_brief` 上方的格式注释同步改。
2. 实现 `Timestamp::unix_secs`（`state.rs`），启用 `timestamp_unix_secs_matches_independent_calendar_math`。然后 `secs_between` 改用它，删掉 `rfc3339_secs` 与「解析不了当 0」的兜底，`StatsJson` 注释里的「解析不了的当 0」同步删。`Timestamp` 构造已校验格式，这里没有失败路径。

第三轮复审（2026-09-25）确认修复（bc14d36），另补交叉声明测试 `brief_require_version_ignores_crossed_kind_name_pairs` 与远年时间用例。见 [decisions.md 里程碑记录](decisions.md) M1 各轮处置与 D-28。
### M1 里程碑审查：core

**执行者。** 强模型，未参与 T02 到 T10。

**做什么。** 读 `git diff T01..T10 -- crates/sheltie-core`，按 [engineering.md §5](../../engineering.md) 检查表逐项打钩。跑 `scripts/mutants.sh sheltie-core`，对每个存活的突变体判断：是测试漏了（补测试，算 M1 的提交），还是死代码（删）。核对 `cargo tree -p sheltie-core` 无 I/O crate。全仓 grep 确认零 `todo!()`、零 `#[allow(unused_variables)]`，`#[ignore` 只剩后续任务的标签（本里程碑覆盖的任务标签为零），有就退回对应任务。复核 `git log t01-skeleton..HEAD -- scripts/ tasks.toml` 里每一次工具改动：意图没被放松、提交说明写清了原因。

**产出。** 一份 `M1` 报告放进 `decisions.md` 末节之前的「里程碑记录」，列检查表结果、存活的突变体数与处置、修复提交。有阻断项时对应任务状态改回 `doing`，由实现者修，M1 再复核。另写一节「流程教训」，格式同 `spec-dev` 的 `lessons.md` 建议表：每条有证据（哪个任务的哪次提交或复核）、有落点（`plan.md`、`engineering.md`、`scripts/` 的哪一段）；采纳的直接改，否决的写原因。M2、M3 同。

### T11 样例 Workbook 编译测试

**文件。** 允许改 `examples/**`（样例本身若有错就改样例）。不改测试。

**测试。** （原列 all_examples_compile：C002-T40 退休或合并；当前保留 `no_example_declares_requires`；见 [T40验证](../../changes/active/C002-v0.2.0-reliability/validation.md#c002-t40-测试精简验证)）、`two_step_has_one_main_edge_and_no_gate`、`article_review_has_back_edge_human_publish_and_resource_input`、`gated_release_first_node_is_gate`、`no_example_declares_requires`、（原列 spec_dev_compiles_with_eleven_nodes_twenty_four_edges：C002-T12 给 spec-dev 补了 escalate→verify 回程边，边数 24→25，该测试改名 spec_dev_compiles_with_eleven_nodes_twenty_five_edges 并由 C002-T12 接管，见 [C002历史验证索引](../../changes/active/C002-v0.2.0-reliability/validation.md#历史证据恢复)）、`spec_dev_retro_reads_engine_stats`、（原列 spec_dev_optional_inputs_all_point_to_reachable_upstream：C002-T40 退休或合并；当前保留 `rejects_optional_input_on_start_or_resource_source`、`rejects_optional_engine_stats_input`、`spec_dev_replanning_uses_required_review_copies_and_reachable_optional_history`、`spec_dev_escalation_inputs_cover_return_edge_to_verify`、`spec_dev_binds_decision_into_scaffold_implement_verify`；见 [T40验证](../../changes/active/C002-v0.2.0-reliability/validation.md#c002-t40-测试精简验证)）、`spec_dev_only_retro_is_gated_and_human_nodes_are_plan_review_and_escalate`、`spec_dev_strong_tier_nodes_are_spec_plan_scaffold_review`、（原列 mutated_two_step_manifest_with_extra_field_is_rejected：C002-T40 退休或合并；当前保留 `rejects_unknown_field`；见 [T40验证](../../changes/active/C002-v0.2.0-reliability/validation.md#c002-t40-测试精简验证)）。

**实现要点。** 正常情况下启用测试就全绿。红了先看是样例不合合同（改样例）还是 T03 到 T05 有 bug（按第 9 条卡住，报回对应任务）。

**提交。** `test(core): 样例 Workbook 按合同编译通过`

### T12 runtime 管理根与文件观察

**文件。** `crates/sheltie-runtime/src/{home,observe}.rs`。

**测试。** `home_prefers_cli_then_env_then_default`、`confine_rejects_dotdot_absolute_and_empty_segment`、`confine_rejects_symlink_escaping_root`、`observe_file_rejects_symlink_and_directory`、`observe_file_sha256_matches_known_vector`（`hello\n` 的已知摘要）、`resource_index_marks_non_utf8`。

**实现要点。** `confine` 按 [存储合同 §6](../../contracts/storage.md)：先按写法拒绝不合规的，再对存在的路径 `canonicalize` 后比前缀。`observe_file` 用 `symlink_metadata` 而不是 `metadata`，否则测不出软链。

**提交。** `feat(runtime): 管理根解析、路径约束与文件观察`

### T13 runtime SQLite 存储

**文件。** `crates/sheltie-runtime/src/store/{mod,commit,read}.rs` 与 `crates/sheltie-runtime/src/failpoint.rs`。`schema.rs` 的建表常量已由骨架写死，本任务只填 `open` 里的结构校验与 `allocate_seq`。`commit()` 入口就调用 `failpoint::maybe_exit("before_commit")`，而 `--all-features` 启用 `failpoint` 特性后该函数是 `todo!("T23")`，本任务的提交类测试必经此处；按 T07 的先例，`maybe_exit` 一并在本任务填（原属 T23，实现即 T23 任务卡的「实现要点」那四行），T23 只剩启用自己的测试。

**测试。** `open_creates_schema_with_user_version_1`、`open_readonly_on_missing_db_is_not_found`、`open_rejects_wrong_user_version`、（原列 open_rejects_same_version_different_table_shape：C002-T40 加强并迁移归属；当前保留 `open_rejects_same_version_different_table_shape`；见 [T40验证](../../changes/active/C002-v0.2.0-reliability/validation.md#c002-t40-测试精简验证)）、`commit_inserts_state_audit_and_request_atomically`、`commit_replays_same_request_id_and_payload`、`commit_rejects_same_request_id_different_payload`、`commit_rejects_stale_revision`、`status_column_mirrors_state_json`、`allocate_seq_starts_at_1_per_day_and_increments`、（原列 allocate_seq_is_not_reused_after_failed_start：C002-T40 退休或合并；当前保留 `allocate_seq_starts_at_1_per_day_and_increments`；见 [T40验证](../../changes/active/C002-v0.2.0-reliability/validation.md#c002-t40-测试精简验证)）、`allocate_seq_rejects_1000th_of_day`、`allocate_seq_under_two_threads_yields_distinct_numbers`。

**实现要点。** `commit` 的事务顺序按 [存储合同 §2](../../contracts/storage.md) 逐行写，先 `BEGIN IMMEDIATE`。结构校验把 `sqlite_master.sql` 与常量都做「去掉全部空白」后比较。`allocate_seq` 按 [存储合同 §7.1](../../contracts/storage.md) 的 SQL 原样写。

**提交。** `feat(runtime): SQLite 存储：结构校验、去重、revision CAS 与序号`

### T14 runtime Workbook 仓库

**文件。** `crates/sheltie-runtime/src/workbook_repo.rs` 中的 `add`、`list`、`load`、`digest_dir`、`copy_confined`。

**测试。** `add_two_step_example_copies_and_marks_readonly`、`add_rejects_duplicate_id_version_with_workbook_exists`、`add_rejects_symlink_inside_workbook`、`add_rejects_file_over_32mib`（sparse 文件）、`add_failure_leaves_no_staging_and_no_row`、`load_recompiles_graph_from_installed_copy`、`list_orders_by_id_then_version`。

**实现要点。** 顺序按 [存储合同 §5](../../contracts/storage.md)：staging → fsync → 摘要 → 事务插行 → rename → 只读。`digest_dir` 按 [协议](../../contracts/protocol.md) `workbook add` 一节的定义：路径排序后拼 `路径\0内容`。

**提交。** `feat(runtime): Workbook 仓库与 staging 原子入库`

### T15 runtime Workbook `remove` 与 `verify`

**文件。** `crates/sheltie-runtime/src/workbook_repo.rs` 中的 `remove`、`verify`、`works_referencing`，`store/read.rs` 的 `delete_workbook`；另按依赖拉入 `service.rs` 的 `start`、`cancel`、`run_command`、`load` 与效果执行（原属 T16）——本任务的两条引用检查测试必经 `svc.start` 与 `svc.cancel`，不先有它们测试只能撞 `todo!("T16")`。

**复核记录（2026-09-25，实现前）。** 夹具修复一次：`verify_reports_missing_when_directory_gone` 把整棵目录 chmod 0644 后 `remove_dir_all` 在 POSIX 上必失败（目录失 x 位进不了子目录），任何实现都不可能通过；改为目录 0755、文件 0644，提交 fe7f5fe，tag `t15-review` 作新基准。T16 只剩 `begin/submit/fail/approve/status/stats/list/resolve_work` 与其测试。

**测试。** `remove_deletes_row_and_directory`、`remove_requires_explicit_version`、（原列 remove_rejects_when_active_work_references_version：C002-T40 退休或合并；当前保留 `remove_rejects_active_reference_and_rolls_back_row`；见 [T40验证](../../changes/active/C002-v0.2.0-reliability/validation.md#c002-t40-测试精简验证)）、`remove_allows_when_only_terminal_works_reference_version`、（原列 remove_moves_dir_to_tmp_before_delete：C002-T40 退休或合并；当前保留 `remove_deletes_row_and_directory`；见 [T40验证](../../changes/active/C002-v0.2.0-reliability/validation.md#c002-t40-测试精简验证)）、`verify_reports_ok_for_untouched_install`、（原列 verify_reports_tampered_after_byte_change：C002-T40 退休或合并；当前保留 `load_rejects_tampered_registered_digest`；见 [T40验证](../../changes/active/C002-v0.2.0-reliability/validation.md#c002-t40-测试精简验证)）、`verify_reports_missing_when_directory_gone`、`verify_all_when_filter_omitted`。

**实现要点。** `works_referencing` 先用 `status` 列过滤非终态，再解 `state_json` 核对 `workbook`，不只信冗余列。

**提交。** `feat(runtime): Workbook remove 引用检查与 verify`

### T16 runtime Work 服务

**结果。** `WorkService` 提供 `start / begin / submit / fail / approve / cancel / status / stats / list`；`start` 复制冻结副本；`REVISION_CONFLICT` 最多重试 3 次；效果幂等，含 `WriteFile`。

**文件。** `crates/sheltie-runtime/src/service.rs`（`start`、`cancel`、`run_command`、`load` 与效果执行已在 T15 一并填，原因见 T15 任务卡；本任务填 `begin / submit / fail / approve / status / stats / list / resolve_work`）。

**测试。** `two_step_runs_to_succeeded`、`start_allocates_work_id_with_today_and_seq_001`、`start_replay_returns_same_work_id_without_new_seq`、`start_copies_workbook_into_work_dir_readonly`、`begin_loads_graph_from_frozen_copy_not_repository`、（原列 status_works_after_workbook_removed：C002-T40 退休或合并；当前保留 `work_readable_after_workbook_removed`；见 [T40验证](../../changes/active/C002-v0.2.0-reliability/validation.md#c002-t40-测试精简验证)）、`begin_writes_brief_md_with_absolute_input_paths`、`begin_binds_resource_input_to_frozen_copy_path`、`status_card_regenerated_after_each_commit`、`concurrent_writers_one_gets_revision_conflict`、`begin_on_tampered_frozen_copy_is_store_corrupt`、`missing_frozen_copy_is_store_corrupt_for_begin_and_status`、`tampered_resource_input_is_store_corrupt_not_artifact_modified`。

**实现要点。** 每个写操作都是同一个私有函数 `run_command` 的调用，骨架已写好它的签名与「加载 → 观察 → 决定 → 提交 → 效果」五个步骤的注释，先填它，再填八个薄的公开方法。`start` 多一步前置：先查 `requests` 表重放，再 `allocate_seq`，再复制冻结副本。`REVISION_CONFLICT` 重试循环最多 3 次。效果按 [存储合同 §3](../../contracts/storage.md) 幂等。

**提交。** `feat(runtime): Work 服务与每 Work 冻结副本`

**M1 遗留。** [decisions.md 里程碑记录](decisions.md) M1 的 O1、O3、O4 已在 core 修完；剩 O2，测试已补（上面最后三条）。加载 Work 时先核对冻结副本：目录在、摘要等于 `WorkState.workbook.digest`，否则 `STORE_CORRUPT`，不回退到仓库。这一步放在「加载」里，`status` 与所有写操作都经过它；`resource.<path>` 输入因此不需要单独记摘要（[存储合同 §5.1](../../contracts/storage.md)、[协议](../../contracts/protocol.md) attempt begin 第 3 步）。

### M2 里程碑审查：runtime

**执行者。** 强模型。

**做什么。** 同 M1，范围 `crates/sheltie-runtime`。另加一项人工走查：对 [存储合同 §3](../../contracts/storage.md) 的每个崩溃时刻，在 `service.rs` 里指出对应代码行，确认 `COMMIT` 之后才有文件写入。`MUTANTS_TIMEOUT=300 scripts/mutants.sh sheltie-runtime`。

### T17 cli `workbook` 组

**文件。** `crates/sheltie-cli/src/{output,error_map}.rs`、`commands/workbook.rs`。`cli.rs` 的命令树由骨架定死，不改。

**测试。** `workbook_add_prints_id_version_digest`、`workbook_add_json_has_ok_true_and_data`、`workbook_add_invalid_dir_exits_1_with_workbook_invalid`、`workbook_list_after_add_shows_one_row_marked_latest`、`workbook_show_lists_nodes_edges_and_requires`、`workbook_remove_without_version_exits_2`、`workbook_remove_then_list_is_empty`、`workbook_verify_exits_1_after_tamper`、`unknown_subcommand_exits_2`。

**实现要点。** `error_map` 是 runtime `Error` 到 [协议 §7](../../contracts/protocol.md) 错误码与退出码的一张 `match`，一次填全。`output` 有两个函数：文本与 JSON 响应封装，格式按 [协议 §5](../../contracts/protocol.md)。

**提交。** `feat(cli): workbook add/list/show/remove/verify`

### T18 cli `work` 组

**文件。** `crates/sheltie-cli/src/commands/work.rs`、`crates/sheltie-cli/src/cli.rs`，另按依赖拉入 `commands/attempt.rs` 的 `begin` 分支与 `cli.rs` 的 `read_text_arg`（原属 T19）——本任务的 `work_cancel_then_any_write_is_work_terminal` 要走 `attempt begin` 拿到 `WORK_TERMINAL` 封装，不先有它测试只能撞 `todo!("T19")` 的 panic（T07、T15 的先例）；`parse_input_arg` 的 `@file` 逻辑与 `read_text_arg` 同一段代码。T19 只剩 `submit`、`fail` 与 `gate`。

**测试。** `work_start_creates_work_and_prints_next`、（原列 work_start_missing_input_exits_1_with_input_missing：C002-T40 退休或合并；当前保留 `start_deterministic_rejections_leave_home_unchanged_and_do_not_burn_seq`；见 [T40验证](../../changes/active/C002-v0.2.0-reliability/validation.md#c002-t40-测试精简验证)）、（原列 work_start_accepts_at_file_input：C002-T40 退休或合并；当前保留 `start_request_replay_does_not_require_the_original_input_file`；见 [T40验证](../../changes/active/C002-v0.2.0-reliability/validation.md#c002-t40-测试精简验证)）、`work_list_shows_status_and_current`、`work_status_prints_status_card`、`work_stats_prints_table_and_json`、`work_status_json_matches_schema`、`work_cancel_then_any_write_is_work_terminal`、`work_id_prefix_resolves_when_unique`、`work_id_prefix_ambiguous_lists_candidates`、（原列 work_start_default_name_is_flow_id：C002-T40 退休或合并；当前保留 `work_start_creates_work_and_prints_next`；见 [T40验证](../../changes/active/C002-v0.2.0-reliability/validation.md#c002-t40-测试精简验证)）、`work_start_with_chinese_name_creates_matching_directory`。

**实现要点。** `--input k=v` 解析在骨架的 `parse_input_arg` 里，`@` 开头读文件。前缀解析 `resolve_work` 查 `list_works` 做前缀匹配。

**提交。** `feat(cli): work start/list/status/cancel`

### T19 cli `attempt` 与 `gate` 组

**文件。** `crates/sheltie-cli/src/commands/{attempt,gate}.rs`。`attempt` 的 `begin` 分支与 `cli.rs` 的 `read_text_arg` 已在 T18 一并填（原因见 T18 任务卡）；本任务填 `submit`、`fail` 与 `gate`。

**测试。** `two_step_via_cli_reaches_succeeded`（每步只从 `--json` 的 `next` 里取命令拼出来跑）、`attempt_begin_returns_brief_path_that_exists`、（原列 attempt_submit_summary_from_at_file：C002-T40 退休或合并；当前保留 `submit_request_replay_does_not_reread_a_deleted_summary_file`；见 [T40验证](../../changes/active/C002-v0.2.0-reliability/validation.md#c002-t40-测试精简验证)）、`attempt_fail_then_begin_retries_same_occurrence`。

**提交。** `feat(cli): attempt begin/submit/fail 与 gate approve`

### T20 cli `self` 组与发布链

**文件。** `crates/sheltie-runtime/src/selfmgmt.rs`、`crates/sheltie-cli/src/commands/self_cmd.rs`，另加 `specs/contracts/storage.md` 与 `specs/decisions.md` 的对应改动（见下）。`dist-workspace.toml` 与 `.github/workflows/release.yml` 由 `cargo dist init` 与 `cargo dist generate` 生成，本任务允许新建这两个文件（手写，T25 打 tag 后用 `cargo dist generate` 校对）。

**测试。** `install_copies_current_exe_and_is_idempotent`、`install_prints_path_hint_and_does_not_touch_rc_by_default`、`update_replaces_binary_and_keeps_prev`、（原列 update_rejects_checksum_mismatch_and_leaves_binary_intact：C002-T40 退休或合并；当前保留 `update_failed_digest_leaves_old_binary_and_cleans_tmp`；见 [T40验证](../../changes/active/C002-v0.2.0-reliability/validation.md#c002-t40-测试精简验证)）、`update_reports_unavailable_when_no_asset_for_platform`、`rollback_swaps_prev_back`、`rollback_recovers_when_current_missing`、`uninstall_keeps_store_and_works`、`uninstall_purge_requires_yes`、`self_version_works_without_home`。

**实现要点。** 原「加 `axoupdater` 依赖」经核对不可行：其 0.10.2 公开 API 不暴露清单与 sha256，唯一完整入口会执行安装脚本，与五步冲突。改为直接读 `dist-manifest.json`（瘦格式为本地与测试的合同；cargo-dist 完整清单在 `selfmgmt` 里适配），网络下载用系统 `curl`，见 [decisions.md D-30](decisions.md) 与存储合同 §9 的同步改写。测试通过 `SHELTIE_RELEASE_BASE` 指向测试自己生成的本地发布目录，不联网。替换顺序按 [存储合同 §9](../../contracts/storage.md) 五步，`update_between_renames` 故障点挪到第 3、4 步之间。目标平台 `aarch64-apple-darwin`、`x86_64-apple-darwin`、`x86_64-unknown-linux-gnu`、`aarch64-unknown-linux-gnu`。

**提交。** `feat(cli): self 命令组与 cargo-dist 发布链`

### T21 场景：审查回环

**文件。** 允许改 `crates/**/src/**`（场景暴露的 bug 在哪就修哪），不改测试。

**测试。** `review_back_edge_creates_second_draft_occurrence`、`next_after_review_offers_both_main_and_back_with_kinds`、`max_visits_exhaustion_blocks_with_no_legal_edge`、`human_executor_node_is_begun_and_submitted_like_agent`、（原列 downstream_binds_latest_succeeded_occurrence_output：C002-T40 退休或合并；当前保留 `review_back_edge_creates_second_draft_occurrence`；见 [T40验证](../../changes/active/C002-v0.2.0-reliability/validation.md#c002-t40-测试精简验证)）、`review_brief_lists_checklist_resource_with_frozen_path`。

**实现要点。** 这是第一次把 T06 到 T10 的状态机放到真实目录里跑回环。红了先看错误码，回到对应 `decide_*` 函数。

**提交。** `test(cli): 审查回环场景：回边、访问上限与参考输入`

### T22 场景：门槛、产物与 Workbook 生命周期

**文件。** 同 T21。

**测试。** `gate_node_success_blocks_work_and_next_has_only_approve_and_cancel`、`begin_next_node_before_approve_is_illegal_next`、`approve_records_os_user_and_unblocks`、`approve_on_terminal_gate_node_succeeds_work`、`modifying_upstream_output_makes_downstream_begin_fail_with_artifact_modified`、`submit_without_required_output_is_output_missing_and_attempt_stays_running`、`submit_oversize_output_is_output_too_large`、（原列 submit_with_symlink_output_is_rejected：C002-T40 加强并迁移归属；当前保留 `submit_with_symlink_output_is_rejected`；见 [T40验证](../../changes/active/C002-v0.2.0-reliability/validation.md#c002-t40-测试精简验证)）、`remove_in_use_workbook_is_rejected_with_work_list`、`remove_after_work_succeeds_then_status_still_renders`、`editing_repository_copy_does_not_change_running_work_brief`、`verify_detects_the_edit`、`add_second_version_marks_it_latest_and_start_defaults_to_it`。

**提交。** `test(cli): 门槛、产物完整性与 Workbook 生命周期场景`

### T23 场景：重放与崩溃

**文件。** `crates/sheltie-runtime/src/failpoint.rs`（`maybe_exit(name)` 已在 T13 提前填，原因见 T13 任务卡）；三处 `failpoint::maybe_exit("…")` 调用不动：`commit.rs` 入口的 `before_commit`、`service.rs` `commit_one` 里 COMMIT 之后的 `after_commit_before_effects`、`selfmgmt.rs` 的 `update_between_renames`。

**复核记录（2026-09-25，M2，实现前）。** 骨架把 `after_commit_before_effects` 留在 `run_command` 入口，进程在提交前就退出，`kill_after_commit_…` 测试任何实现都不可能通过；M2 已把它挪到 `commit_one` 的 COMMIT 之后、效果之前（见 [decisions.md](decisions.md) M2 记录 B1）。

**测试。** `kill_before_commit_leaves_state_unchanged_and_replay_succeeds`、`kill_after_commit_leaves_state_advanced_and_replay_returns_original_reply_and_rewrites_brief`、（原列 status_card_missing_is_regenerated_on_next_write：C002-T40 加强并迁移归属；当前保留 `status_card_missing_is_regenerated_on_next_write`；见 [T40验证](../../changes/active/C002-v0.2.0-reliability/validation.md#c002-t40-测试精简验证)）、`kill_between_update_renames_leaves_prev_and_rollback_recovers`、`same_request_id_same_payload_returns_replayed_true`、`same_request_id_different_payload_is_request_conflict`。

**实现要点。** `maybe_exit` 只在 `cfg(feature = "failpoint")` 下读环境变量 `SHELTIE_FAILPOINT`，相等则 `std::process::exit(70)`；没有该特性时是空函数。测试用子进程跑 `--features failpoint`。

**提交。** `test(runtime): 崩溃窗口、请求重放与升级恢复`

### M3 里程碑审查：端到端

**执行者。** 强模型。

**做什么。** 对 [规格 §7](../../spec.md) 十三个场景逐行找到对应测试名并确认通过。读 `git diff M2..T23` 按检查表打钩。跑一次 `cargo nextest run --all-features` 全量，记录测试总数与耗时。

### T24 sheltie skill

**执行者。** 强模型。skill 是给 agent 读的说明书，措辞质量直接影响协调者行为。

**结果。** `skills/sheltie/SKILL.md`（frontmatter `name: sheltie`，`description` 写清何时用）教协调者：列 Workbook、开 Work、读 `next`、领任务书、核对任务书里「需要的宿主资源」是否已装、派工作 agent、提交、选边、遇到 `blocked` 找人。`scripts/check-skill.sh`：skill 里出现的每条 `sheltie ...` 命令必须在 [协议 §2](../../contracts/protocol.md) 表里；不得出现 `store.db`、`state_json`、`sqlite`、`status-card.md` 的写入指令。

**测试。** 脚本本身就是测试；另在 `crates/sheltie-cli/tests/skill.rs` 里读 `SKILL.md`，对每条命令用 `sheltie <group> <verb> --help` 确认存在。

**实现要点。** 按 Anthropic Agent Skills 格式：frontmatter 只放 `name` 与 `description`；正文短，写成步骤；细节链接到 `specs/contracts/protocol.md`。说明书里明确写：结论在输出文档里，你（协调者）读文档选边；引擎给的 `next` 之外的事不要做。

**提交。** `feat(skill): sheltie SKILL.md 与命令白名单检查`

### T25 收口

**执行者。** 人。

**结果。** `README.md` 的「快速开始」从 `curl … install.sh | sh` 开始，由一个从未接触项目的人（或一个新开的 agent 会话）在干净的 `SHELTIE_HOME` 下只按文字操作走通 `two-step`；`cargo deny check` 绿；`CHANGELOG.md` 由 `git-cliff` 生成；打 tag `v0.1.0`，确认 release 工作流产出四个平台的包与 `install.sh`。

**文件。** `README.md`、`CHANGELOG.md`、`cliff.toml`、`Cargo.toml`、`Cargo.lock`、`dist-workspace.toml`、`.github/workflows/`、`crates/sheltie-cli/tests/{version,self_cmd}.rs`、`crates/sheltie-runtime/{src/selfmgmt.rs,tests/selfmgmt.rs}`（后两者仅当真实清单不适配时）、`specs/`。执行手册见 [t25-t26-runbook.md](runbook.md)。

**验证。** 记录实测者、日期、卡住的地方；卡住即改文档再测，直到一次走通。`sheltie self update` 从 `v0.1.0-rc` 升到 `v0.1.0` 走一次真实网络。

**提交。** `docs(specs): 从 install.sh 实测快速开始并发布 v0.1.0`

### T26 真实宿主实测

**执行者。** 人。

**结果。** 在 Claude Code 里把 `skills/sheltie` 装到用户 skill 目录（手工 `cp`，MVP 不做安装工具），输入 `/sheltie`，让协调者用 `article-review` 样例走完一次含打回的流程。记录：协调者是否只用了 `next` 里的命令；有没有试图绕过；任务书是否够用；token 用量的宿主观测值。

**文件。** `specs/`。执行手册见 [t25-t26-runbook.md](runbook.md)。

**结果去向。** 写进 `specs/decisions.md` 末节「首次真实运行」。发现的问题按 [engineering.md §7](../../engineering.md) 路由。这一项不产生代码提交，产生一次文档提交。

## 3. 完成判据

MVP 完成当且仅当：

1. T01 到 T25 与 M1 到 M3 状态全部 `done`，每个任务对应一个提交。
2. [规格 §7](../../spec.md) 十三个场景每个至少对应一个通过的自动化测试，`新人上手` 与 T26 有人工记录。
3. 仓库里没有 `todo!()`，没有 `#[ignore`。
4. `scripts/check-docs.sh`、`scripts/check-core-vocab.sh`、`scripts/check-skill.sh` 在 CI 里跑且绿。
5. `sheltie-core` 的依赖树不含任何 I/O crate（`cargo tree -p sheltie-core` 人工核对一次并把结果贴进 T25 的提交信息）。
