# C005 实施计划

状态：`draft`。本计划随提案一起准备，package 仍是 `proposed`，下表全部 `todo`。人采用第一阶段、目录移入 `active/` 之后，本计划才是实施进度的唯一权威。代码在 C002 通过验收（C002-T17）之后才开始。

本计划只把**第一阶段（人工接续）**拆成任务。第二阶段（辅助与自动接续）另行采用，§4 只列入口条件和候选分组，采用时再补任务卡。

C005 第一阶段可以排在 C004 之前，也可以排在它之后（README「采用条件」）。所以任务分两层：主线任务（T01–T14）不依赖 C004，按“承诺收窄”版本实施；依赖 C004 记录的字段与判定放在 §3 的增量组，C004 实现后由人决定插入。主线里不为这些字段另造发现账本或身份记录。

做法借鉴 [v0.1.0 plan](../../../releases/v0.1.0/plan.md)：**强模型先搭骨架、写全部测试，初级实现者按任务逐一填空，每完成一组里程碑由强模型审查一次。** 实现者第一次接触本仓库，只需要读本文 §0、自己那张任务卡、卡上列出的文件和测试。产品语义以 [spec.md](spec.md) 为准，状态转换以 [design.md](design.md) §5 的转换表为准，失败路径以 [validation.md](validation.md) §5 为准；采用后以同步过的根规格与 `contracts/` 为准。

模块路径与签名写的是**暂定位置**。C002（以及排在前面时的 C004）会改动 Store、请求格式和 `decide`，C005-T02 的骨架作者按届时的代码定死路径与签名；需要改路径时，在 T02 的提交里同时改本文任务卡和 [tasks.toml](tasks.toml)，并在 T02 卡的「记录」行写明原因。测试名是合同，T02 必须按任务卡的名字写出测试；确需改名的，同样在 T02 提交里改任务卡。

## 0. 执行方式

### 0.1 四层分工

| 层 | 谁 | 做什么 | 何时 |
| --- | --- | --- | --- |
| 决定 | 人 + 强模型 | 同步根规格、宪章、合同与路线图；写定 T01 卡列出的实施前决定 | C005-T01，一次 |
| 骨架 | 强模型 | 全部类型、签名、文档注释、`todo!("C005-Tnn")` 函数体、全部测试（禁用）、快照、工具修补 | C005-T02，一次 |
| 填空 | 初级开发者或初级模型 | 启用本任务测试、填函数体、跑门禁、提交 | C005-T03 起，每任务一次 |
| 审查 | 编译器、clippy、测试、`scripts/check-task.sh` | 逐任务自动 | 每次提交 |
| 审查 | 未参与该组实现的强模型 | 按 [engineering.md §5](../../../engineering.md) 检查表审一组 diff、跑突变测试、对照 validation.md §5 | C005-M1 到 M3（增量组另加 M4） |

两个宿主上的真实接续回归与发布由人完成（C005-T13、C005-T14）。

### 0.2 实现者规则

写给做填空任务的人或模型。每条都是硬规则，多数由 `scripts/check-task.sh` 机械核对。

1. **先按仓库入口开工，再聚焦任务。** 先读 `CONTEXT.md`、`specs/README.md`、`specs/changes/README.md`、本 active package 入口和 `specs/engineering.md`（AGENTS.md「开工入口」）；随后重点读本文 §0、自己的任务卡、卡上列出的文件与合同章节。遇到调用链超出卡片时继续追到真实使用者，不以“只读卡片”为由跳过。
2. **先看到红。** 运行 `scripts/task.sh C005-Tnn`（禁用的测试也会跑），确认本任务测试全红；然后删掉这些测试上的 `#[ignore = "C005-Tnn"]`。编译错误不算红。
3. **只填 `todo!("C005-Tnn")`。** 不改签名、类型、`pub` 可见性，不加依赖，不新建文件。同文件内新增私有辅助函数可以，但不得改变任何公开项的行为边界。文档注释就是函数要做的事。
4. **不改测试，不改快照。** 测试红了改实现。快照不一致改渲染代码，不运行 `cargo insta accept`。
5. **一次一个测试。** 让一个红测试变绿，再做下一个。
6. **绿了跑门禁。** `scripts/task.sh C005-Tnn` 全绿后，把 §1 表中本任务状态改为 `done`，运行 [engineering.md §2.3](../../../engineering.md) 的四条命令与 `scripts/check-task.sh C005-Tnn --staged`。
7. **一次提交。** 第一行用任务卡「提交」行；正文写为什么、怎么验证；末尾是 `Change: C005`、`Task: C005-Tnn`、`Agent: <名字>`。状态改为 `done` 放在同一提交。提交后再运行一次 `scripts/check-task.sh C005-Tnn`，并核对 `git show --stat`。
8. **不顺手改。** 别的任务的 `todo!`、测试、你觉得能优化的地方都不碰。
9. **卡住就停。** 出现下列任一情况时停手，不提交；把状态改为 `blocked`，在 [progress.md](progress.md) 写清原因：两个测试互相矛盾；不改签名或测试就做不到；需要新依赖；同一个测试改了五次还红；任务卡与合同说法不一；需要碰到后续任务的 `todo!`。
   **唯一例外：工具缺陷。** 如果 `scripts/` 或 `tasks.toml` 让一个按规则做的任务无法通过，可以修工具，但要单独提交，写 `Task: C005-T02`，并在提交说明中逐条写明原来错在哪、改成了什么。修复不得放松检查意图，不得改动测试、签名或合同。里程碑审查复核每一次工具改动。
10. **不猜。** 文档注释、测试和合同都没说的行为，不自己发明，按第 9 条停下。在注释里写下“这里由调用方保证”“解析不了当默认值”之类的约定，同样算发明。

实现者是强模型时，可以在一个会话里连做相邻任务，但仍是一任务一提交，每个任务各跑一遍 `task.sh` 与 `check-task.sh`。审查发现缺陷时，审查者只补测试（挂被审任务编号、禁用），实现者修代码。

### 0.3 测试与工具

- **命名与归属。** 测试名写“条件 → 行为”，不带任务编号。归属写在紧贴 `#[test]` 上方的一行注释里：`// Task: C005-Tnn`，再加 `#[ignore = "C005-Tnn"]`。采用后 `scripts/check-tests.sh` 会解析本文每张卡的「**测试。**」行：行内每个反引号里的小写标识符都必须是真实存在、且归属本任务的测试。所以这一行只写测试名，说明放在全角括号（…）里。
- **基准 tag。** T02 提交后打 tag `c005-t02-skeleton`；审查者补测试后打 `c005-tNN-review`。实现者的测试零改动检查以最近一个 `c005-t*` tag 为基准。截至本文写作，`check-task.sh` 对 `Cnnn-Tnn` 默认取 README 的基线提交，这个默认基准会把整份骨架都算作“测试改动”。T02 必须先修好这一点；如果别的 package（C004、C006、C007）已先修过，就直接复用。
- **命令。** `scripts/task.sh C005-Tnn` 只跑本任务测试；`scripts/check-tests.sh` 核对命名与任务卡；`scripts/check-task.sh C005-Tnn [base] [--staged]` 核对改动范围、占位清零、测试零改动、状态与 trailer；`scripts/mutants.sh <crate>` 供里程碑审查使用；`scripts/check-core-vocab.sh` 确保 core 与 runtime 不出现业务词汇（GF-01：不得出现 Claude、Codex、订阅等）；`scripts/check-skill.sh` 核对 skill。
- **时间。** core 不读时钟。等待开始时间、恢复时间由 runtime 从注入的时钟取得后传给 core；测试用固定时钟，总等待期限的边界用“恰好等于上限”和“上限加一秒”两种情况覆盖。
- **测试分层。** 按 [engineering.md §3.2](../../../engineering.md) 放置：core 用模块内单元测试加 `insta` 快照；runtime 用临时 `SHELTIE_HOME` 的集成测试；CLI 用 `assert_cmd` 端到端测试；崩溃用 fail-point 加子进程，放在 `crash.rs`；并发用两个真实子进程争同一把写锁。
- **独立 oracle。** 计数、状态和 JSON 字段的期望值由测试直接写出。测试 helper 不得复用被测函数来计算期望值。

### 0.4 复核与里程碑记录

- 每个填空任务在提交前由未参与该任务实现的复核者核对任务卡、合同、受影响调用链与本任务正反例；`check-task.sh`、门禁和测试负责机械检查。里程碑再做整组调用链、故障窗口与突变审查，不为每个小任务重复跑突变。
- 里程碑审查者没有参与本组实现。审查中发现缺陷时，审查者补禁用测试、打 `c005-tNN-review`，并把对应任务改回 `doing`；实现者修复后再提交一次，写被退回任务的 `Task:`，不写 `Task: C005-Mn`。审查者自己补测试的提交写 `Task: C005-Mn`。
- 缺陷涉及签名或数据结构、会让全仓夹具一起变红时，交回骨架作者修复并记录；未编写该修复的审查者再审修复 diff 与受影响调用链。里程碑审查者不得审自己写的骨架或修复。
- 里程碑报告写入本 package 的 `milestones.md`（M1 创建此文件；`review.md` 是提案讨论记录，不混写）。报告包含：检查表逐项结论、存活突变体的数量与处置、validation.md §5 的行与测试名对照、每一次工具改动的复核结论、退回清单，以及一节「流程教训」（每条写明证据与落点）。结论只用“通过 / 需修改 / 阻断”。validation.md 的执行状态表由里程碑审查者填写：写命令、原始输出路径、退出码和输入闭包，不只写 PASS。

### 0.5 成本

任务卡约 300–500 字；每个任务的骨架在 100–300 行，测试在 100–400 行；每个任务的输入上下文在 3k–8k token 之间。C005 第一阶段比 C004 小：核心是一张状态转换表，大部分任务是这张表的一部分行。

## A. 进入本计划前（不在任务表内）

| 事项 | 谁 | 结果去向 |
| --- | --- | --- |
| C002 通过验收（至少恢复闭合、摘要编码、N04 身份） | C002 Owner | C002 release record |
| 第一阶段人工接续实验：用现有状态卡和输出文件模拟交接包，测缺什么 | 人 + 强模型 | validation.md、README |
| 人决定 GF-14 的修改方案（可恢复等待） | 人 | README 采用条件 |
| 与 C004 的共用接口（候选快照、执行引用、证据记录）定稿；决定 C005 第一阶段排在 C004 之前还是之后 | 人 | README 采用条件；决定 §3 增量组何时插入 |

## 1. 任务表

状态只用 `todo | doing | blocked | done`。表格顺序就是执行顺序。§3 的增量组在 C004 实现后由 T01（或届时的补充计划提交）插入本表，排在 T13 之前；C005 排在 C004 之后实施时，T01 直接插入。

| ID | 状态 | 执行者 | 依赖 | 标题 | 结果 |
| --- | --- | --- | --- | --- | --- |
| C005-T01 | todo | 人 + 强模型 | 采用 | 上游同步与实施前决定 | GF-14、GF-13、GF-11 等与协议、存储合同写定；命令名、闭集线上名、上限来源、交接包入口写定 |
| C005-T02 | todo | 强模型 | T01 | 骨架、全部测试与工具 | 可编译骨架，全部测试存在且禁用；`scripts/task.sh C005-T03` 跑出红；tag `c005-t02-skeleton` |
| C005-T03 | todo | 初级 | T02 | core 取代运行中的 Attempt | `superseded` 终态；不耗业务重试；切换计数与耗尽 |
| C005-T04 | todo | 初级 | T03 | core 资源等待与恢复 | `resource_wait` 进出；资源重试与总等待期限；耗尽进入 `continuity_exhausted` |
| C005-T05 | todo | 初级 | T04 | core `next` 形状与分开计数 | `running`、`resource_wait`、`continuity_exhausted` 三种 `next`；四种计数分开报告 |
| C005-T06 | todo | 初级 | T05 | core 状态卡按来源显示 | 模型自报显示“暂停，未核实”；计数与耗尽项可见 |
| C005-T07 | todo | 初级 | T05 | core 交接包投影 | 有界、只读、可重生成；C004 字段不出现；副作用风险写明 |
| C005-M1 | todo | 强模型 | T03–T07 | 里程碑审查：core | 转换表逐行正反例；`scripts/mutants.sh sheltie-core`；GF-01 词汇 |
| C005-T08 | todo | 初级 | M1 | runtime 存储 | `superseded`、等待记录、迟到提交记录、四种计数落库；旧库被拒 |
| C005-T09 | todo | 初级 | T08 | runtime 服务、并发与崩溃窗口 | 取代与等待各一个事务；并发至多一个生效；崩溃无中间状态 |
| C005-T10 | todo | 初级 | T09 | cli 命令与输出 | `attempt supersede`、`work wait`、`work resume`、交接包命令；错误码 |
| C005-M2 | todo | 强模型 | T08–T10 | 里程碑审查：runtime 与 cli | `scripts/mutants.sh sheltie-runtime`；事务边界与并发走查 |
| C005-T11 | todo | 初级 | M2 | 场景：换宿主、等待与耗尽 | 真实 CLI 走完转换表的每条路径 |
| C005-T12 | todo | 强模型 | T11 | 两份宿主 skill | Claude Code 与 Codex 的接手说明；`scripts/check-skill.sh` 通过 |
| C005-M3 | todo | 强模型 | T11–T12 | 里程碑审查：端到端 | validation.md §5 第一阶段各行对到测试；spec.md §4 第一阶段承诺逐行正反例 |
| C005-T13 | todo | 人 | M3（及增量组的 M4） | 两宿主真实接续回归 | Claude Code 做到一半，换 Codex 接手完成同一个 Work；逐命令记录 |
| C005-T14 | todo | 人 | T13 | 发布 | release record、CHANGELOG、package 转 `completed`（第一阶段） |

## 2. 任务卡

每张卡给：结果、文件（只能改这些）、测试（要变绿的测试名）、实现要点（填空提示，不是设计）、停止条件、提交（信息第一行）。

### C005-T01 上游同步与实施前决定

**执行者。** 人审定，强模型起草。

**结果。** 按 spec.md §6 中标为“一”的各行修订 `specs/spec.md`（GF-05、GF-06、GF-10、GF-11、GF-13、GF-14、GF-15、GF-18、GF-23）、`specs/architecture.md`（Attempt 编号与重试判定）、`specs/constitution.md`（INV-6 的来源分项、§7）、`specs/roadmap.md`、`specs/contracts/{protocol,storage}.md`；`CONTEXT.md` 加入取代、资源等待、接续耗尽、切换次数、资源重试、总等待期限、交接包、原因来源等词汇。C004 已采用时，GF-14 的停止原因措辞与 C004 合并（`scope_violation`、`retries_exhausted`、`continuity_exhausted` 只能取消，`resource_wait` 可恢复）。README 的 `基线：` 改为实施起点提交。

**必须写定的决定。** 括号里是本计划的建议：

1. 三个新命令的最终名字与参数（建议沿用 `attempt supersede --reason --source`、`work wait --reason resource_unavailable --source`、`work resume --source [--switch]`）。
2. 中断原因闭集与来源闭集的线上名（建议原因：`resource_unavailable | candidate_rejected | environment_fault | policy_denied | capability_gap | unknown`；来源：`provider | host | user | model_self_report`）。
3. 原因为 `policy_denied` 时的引擎行为。spec.md EX-03 要求“停止，交给人；不能借换宿主绕过”（建议：`attempt supersede` 拒绝该原因并提示只能 `work cancel` 或找人；`work wait` 本来就只接受 `resource_unavailable`）。
4. 三种接续上限的声明位置与默认值（建议：Flow 顶层一个可选表，缺省时取合同常量；上限随 Workbook 冻结，执行中不变）。
5. “尚未领取”时是否允许 `work wait`（design.md §5 转换表写“`running` 或尚未领取”，建议允许，此时不产生 `superseded`）。
6. 拒收的迟到提交留一条记录（design.md §4）。这是现有“拒绝不写入”原则的例外，需要在 storage 合同中写明：记录写在哪张表、是否与拒绝应答同一事务、重复迟到提交是否各记一条。
7. `work resume` 超过总等待期限时的判定时刻，以及计时用哪个时钟（建议：恢复时用 runtime 注入的时钟计算本次等待时长，累计值超过上限则进入 `continuity_exhausted`）。
8. 交接包的入口与大小上限（建议新增只读命令 `sheltie work handoff`，只做投影；超出上限时只给指针）。
9. Store 是否升结构版本、旧库怎么处理（沿用 C002 的“显式拒绝、不写旧库”）。
10. 两份宿主 skill 的位置（建议：`skills/sheltie/SKILL.md` 增加接手一节，另建 `skills/sheltie-codex/SKILL.md`；T02 把 `scripts/check-skill.sh` 扩到 `skills/*/SKILL.md`）。
11. §3 增量组是否现在插入（取决于 C004 是否已实现）。
12. Attempt id 后缀与 `max_retries` 解耦：后缀是 Occurrence 内每次 `begin` 的唯一顺序号，core 字段与 CONTEXT 词条不再叫 `retry`；业务失败数从 `failed` Attempt 派生。写定 `next` 与 `attempt fail` 的上限判定、Store 往返和旧结构库拒绝方式，并同步 `architecture.md`。`max_retries = 0` 时仍可取代后重新领取，但随后第一次失败必须阻塞。

**文件。** `specs/`、`CONTEXT.md`、本 package 目录。

**验证。** `scripts/check-docs.sh`、`scripts/check-specs.sh` 绿。人逐条确认上面 12 项都已写定且有出处；design.md §5 转换表的每一行都能在 protocol 合同里找到对应的命令、前提、结果和错误码。

**停止条件。** 任一项无法写定。

**提交。** `docs(specs): 同步 C005 取代、资源等待与分开计数的上游条款`

### C005-T02 骨架、全部测试与工具

**执行者。** 强模型。这是唯一做代码设计的任务。

**结果。** 全仓可编译；先按 T01 的合同完整改好现行 `BeginAttempt`、`FailAttempt` 的编号与失败计数语义，让已有路径和测试保持可用；这部分不能留 `todo!` 破坏现行入口。新命令的类型、签名和文档注释齐全，未实现的新路径函数体是 `todo!("C005-Tnn")`；本文每张卡列出的新测试全部写好并禁用，既有路径的回归测试启用；快照预写；CI 全绿。`scripts/task.sh C005-T03` 退出非零，并列出 T03 的测试名。

**文件（暂定，按实施起点代码定死）。**

- core：`work/state.rs` 加 `AttemptState::Superseded`、`BlockedReason::{ResourceWait, ContinuityExhausted}`、中断原因与来源枚举、接续计数与上限；Attempt id 后缀改为唯一顺序号，业务失败数从 `failed` 记录派生，不另存；`work/command.rs` 加 `SupersedeAttempt`、`WaitForResource`、`Resume` 三个命令；`work/decide.rs` 为三条命令各留一个私有 `todo!` 函数并替换原先 `retry == max_retries` 的判断；`work/next.rs`、`work/render.rs` 留对应分支；新增 `work/handoff.rs` 放交接包投影；`error.rs` 加新错误码。
- runtime：`store/{schema,commit,read}.rs` 加等待记录、迟到提交记录与计数；`service.rs` 加三条命令的服务入口和时钟注入；`failpoint.rs` 加 `work wait` 事务前后的 fail-point。
- cli：`cli.rs` 命令树；`commands/{attempt,work}.rs`、`output.rs`、`error_map.rs`。
- 测试：各卡的全部测试；并发测试用的“两进程争锁” helper 与固定时钟 helper，**完整实现，不留填空**。
- 工具：`scripts/check-task.sh` 的 tag 基准（如尚未修）；`scripts/check-skill.sh` 扩到全部 `skills/*/SKILL.md`（按 T01 第 10 项）。

**骨架的写法。** 类型逐字段对照 T01 写定的合同。每个公开项的文档注释，第一行写对应的合同章节，第二行写要返回的错误。`decide` 的三条新命令各拆成一个私有函数，前提核对和结果构造分开，长度以初级实现者一次能填完为准。计数只在 core 的状态里增减，runtime 不自己算。

**验证。**

1. 四条门禁、`cargo deny check`、`scripts/check-docs.sh`、`scripts/check-core-vocab.sh`、`scripts/check-tests.sh`、`scripts/check-skill.sh` 全绿。
2. `cargo nextest run --all-features` 显示新增测试全部 ignored、零失败。
3. **现行路径回归：** `max_retries = 0` 首次失败阻塞、`max_retries = 1` 首次失败可重试而第二次失败阻塞；T02 的编号重构不能改变这些行为。临时填满 T03，运行 `scripts/check-task.sh C005-T03` 必须通过，然后撤掉填充。
4. **依赖顺序检查：** 对每个填空任务，只启用本任务的测试时，触发的 panic 都来自本任务的 `todo!("C005-Tnn")`。
5. **反例：** 改一行测试后，`check-task.sh` 必须失败。

**记录。** 相对本文暂定路径的每一处改动写在这里：原路径 → 新路径 → 原因。

**提交。** `chore(workspace): 搭起 C005 骨架、全部禁用测试与任务工具`。提交后打 tag `c005-t02-skeleton`。

### C005-T03 core 取代运行中的 Attempt

**结果。** 最新 Attempt 为 `running` 时，`SupersedeAttempt` 把它转为 `superseded`，记录原因与来源，切换计数加一，同一 Occurrence 开放 `attempt begin`；`max_retries` 不变。本次取代将超过切换上限时，旧 Attempt 仍转为 `superseded`，Work 进入 `blocked(continuity_exhausted)` 并记录耗尽项。旧 Attempt 迟到的 `submit`、`fail` 按 `ATTEMPT_NOT_RUNNING` 拒绝。

**文件。** `crates/sheltie-core/src/work/decide.rs`（只填标 `C005-T03` 的私有函数）。

**测试。** `supersede_running_attempt_marks_superseded_and_opens_begin`、`supersede_does_not_consume_max_retries`、`supersede_at_zero_business_retries_opens_new_attempt_with_unique_id`、`first_failure_after_supersede_at_zero_retries_exhausts`、`supersede_increments_switch_count`、`switch_count_at_limit_allowed_and_limit_plus_one_exhausts`、`supersede_exhausting_switches_still_marks_attempt_superseded`、`supersede_rejected_when_latest_attempt_not_running`、`supersede_with_policy_denied_reason_rejected`（按 T01 第 3 项）、`late_submit_after_supersede_rejected_attempt_not_running`、`late_fail_after_supersede_rejected_attempt_not_running`

**实现要点。** 先核对前提（Work 活动、最新 Attempt `running`、原因允许），再算切换次数，最后决定是否耗尽，顺序不能换。耗尽时旧 Attempt 也要转 `superseded`，不能留在 `running`。新 Attempt 的唯一编号由已有记录顺序得出；业务失败数只数 `failed`，不另存“代次”。

**停止条件。** 测试要求改动 `max_retries` 或 `max_visits`；需要新增状态。

**提交。** `feat(core): 取代运行中的 Attempt，只计切换次数`

### C005-T04 core 资源等待与恢复

**结果。** Work 活动时，`WaitForResource` 若有 `running` Attempt 就把它转为 `superseded`，Work 进入 `blocked(resource_wait)`，记录来源与开始时间，资源重试加一；将超过资源重试上限时改为 `continuity_exhausted`。只接受 `resource_unavailable`。`Resume` 只在 `resource_wait` 可用：Work 回到活动，同一 Occurrence 开放 `attempt begin`，本次等待时长计入累计；累计已超过总等待期限时进入 `continuity_exhausted`；声明换执行者时再计一次切换。`resource_wait` 中 `work cancel` 取消 Work；`continuity_exhausted` 只允许取消。等待与恢复都不改变 `max_retries`、`max_visits`。

**文件。** `crates/sheltie-core/src/work/decide.rs`（只填标 `C005-T04` 的私有函数）。

**测试。** `wait_with_running_attempt_supersedes_and_enters_resource_wait`、`wait_without_running_attempt_enters_resource_wait`、`wait_increments_resource_retry_count`、`resource_retry_at_limit_allowed_and_limit_plus_one_exhausts`、`wait_rejects_reason_other_than_resource_unavailable`、`resume_returns_active_with_begin_in_same_occurrence`、`resume_adds_wait_duration_to_total`、`total_wait_at_limit_resumes_and_limit_plus_one_second_exhausts`、`resume_with_executor_switch_counts_switch`、`resume_rejected_outside_resource_wait`、`resource_wait_cancel_cancels_work`、`continuity_exhausted_rejects_everything_but_cancel`、`wait_and_resume_leave_business_counters_unchanged`、`late_submit_during_resource_wait_rejected_attempt_not_running`

**实现要点。** 时间作为参数传入，不读时钟。“到达重置时间”不是恢复条件，core 里没有这个概念。每个分支都对照 design.md §5 转换表的一行；表里没有的组合按“操作不合法”拒绝。

**停止条件。** 转换表与测试不一致；需要读时钟。

**提交。** `feat(core): 可恢复的资源等待与接续上限`

### C005-T05 core `next` 形状与分开计数

**结果。** `running` 时，`next` 在 `submit`、`fail`、`cancel` 之外加入 `supersede` 与 `wait`；`resource_wait` 时只给 `resume` 与 `cancel`；`continuity_exhausted` 只给 `cancel`；`retries_exhausted` 不变。状态视图中分开报告业务失败数、资源重试、切换次数与累计等待，各带上限；业务失败数和执行代次从 Occurrence 内 Attempt 记录派生。

**文件。** `crates/sheltie-core/src/work/next.rs`（只填标 `C005-T05` 的分支）、`crates/sheltie-core/src/work/state.rs`（只填标 `C005-T05` 的派生函数）。

**测试。** `next_for_running_attempt_adds_supersede_and_wait`、`next_for_resource_wait_offers_resume_and_cancel_only`、`next_for_continuity_exhausted_offers_cancel_only`、`next_for_retries_exhausted_is_unchanged`、`counters_reported_separately_with_limits`、`attempt_generation_derived_from_occurrence_order`

**实现要点。** `next` 只列合法操作，不排序偏好、不推荐（INV-1）。代次不落库，每次从记录算出。

**停止条件。** 需要把代次存进状态。

**提交。** `feat(core): 接续相关的 next 形状与分开计数`

### C005-T06 core 状态卡按来源显示

**结果。** 资源等待按来源显示：模型自报写“暂停，未核实”，人工报告写“用户报告不可用”，提供方通道写“提供方报告耗尽”；宿主通道按 T01 写定的措辞显示。状态卡列出四种计数与上限；`continuity_exhausted` 写明耗尽的是哪一项。

**文件。** `crates/sheltie-core/src/work/render.rs`（只填标 `C005-T06` 的函数）。

**测试。** `status_card_resource_wait_from_model_self_report_says_paused_unverified`、`status_card_resource_wait_from_user_report_says_user_reported`、`status_card_resource_wait_from_provider_says_provider_reported`、`status_card_shows_four_counters_with_limits`、`status_card_names_exhausted_item_for_continuity_exhausted`

**实现要点。** 措辞只从来源枚举映射，穷尽 `match`；任何来源都不得输出“额度耗尽”这四个字，只有提供方来源可以写“提供方报告耗尽”。快照逐字节比对。

**停止条件。** 快照与协议不一致。

**提交。** `feat(core): 状态卡按来源显示资源等待`

### C005-T07 core 交接包投影

**结果。** 交接包是一份有界的只读投影：Work、当前节点与 Occurrence、冻结的 Workbook 与合同版本、门槛批准等已确认的决定、最近一次输出的指针、当前节点任务书指针、中断原因与来源、“旧执行者是否确认停止、是否共用工作区”这一交接条件。C004 的候选快照、验证结果和未处置发现在主线中**不出现**，也不从输出文档里抽取发现清单。超出大小上限时只给指针。不含对话内容。

**文件。** `crates/sheltie-core/src/work/handoff.rs`。

**测试。** `handoff_lists_work_node_occurrence_and_frozen_versions`、`handoff_lists_gate_approvals_as_confirmed_decisions`、`handoff_points_to_latest_output_without_extracting_findings`、`handoff_lists_interruption_reason_and_source`、`handoff_states_old_executor_stop_unconfirmed_and_separate_workspace`、`handoff_omits_c004_fields_in_mainline`、`handoff_respects_size_limit_at_boundary`、`handoff_regenerates_byte_equal_from_same_state`

**实现要点。** 纯函数：输入是状态和记录，输出是字节。同一状态两次渲染必须字节相等（GF-15）。大小上限取合同常量，测试覆盖恰好等于上限和上限加一两种情况。

**停止条件。** 需要读文件内容（这里只给指针）。

**提交。** `feat(core): 渲染可重生成的交接包`

### C005-M1 里程碑审查：core

**执行者。** 强模型，未参与 T03–T07。

**做什么。**

1. 读 `git diff c005-t02-skeleton..HEAD -- crates/sheltie-core`，按 engineering.md §5 检查表逐项打钩。
2. 把 design.md §5 转换表逐行对到一个正例测试和一个只改一个条件的反例测试；缺的补禁用测试并退回。
3. 运行 `scripts/mutants.sh sheltie-core`，逐个处置存活的突变体，重点看计数加一与上限比较（`<` 与 `<=`）。
4. 运行 `scripts/check-core-vocab.sh`；grep 确认 core 中没有 Claude、Codex、订阅、额度窗口等词。
5. grep 确认 T03–T07 的 `todo!`、`#[allow(unused_variables)]`、`#[ignore = "C005-T0x"]` 清零；复核本组的每一次工具改动。

**产出。** 创建 `milestones.md`，写入 M1 报告（格式见 §0.4）。

### C005-T08 runtime 存储

**结果。** Store 能存取 `superseded` 状态、中断原因与来源、等待记录（来源、开始时间、结束时间）、拒收的迟到提交记录，以及四种计数。所有字段往返一致。遇到旧结构版本的库时拒绝打开，并且不写入。

**文件。** `crates/sheltie-runtime/src/store/{schema,commit,read}.rs`（只填标 `C005-T08` 的函数）。

**测试。** `superseded_attempt_round_trips_reason_and_source`、`wait_record_round_trips_source_and_times`、`rejected_late_submit_record_round_trips`、`continuity_counters_round_trip`、`reason_and_source_columns_reject_unknown_value`、`store_rejects_pre_continuity_schema_without_writing`

**实现要点。** 建表语句是骨架给出的常量，不改。闭集列存线上名，读出后用穷尽 `match` 还原。

**停止条件。** 需要改表结构。

**提交。** `feat(runtime): 存储取代、等待记录与接续计数`

### C005-T09 runtime 服务、并发与崩溃窗口

**结果。** 服务层的三条命令各在一个写锁事务内完成“观察 → 决定 → 提交”。`work wait` 在同一事务内完成“取代 Attempt + 进入等待”。恢复时用注入的时钟计算等待时长。迟到提交被拒绝并按 T01 第 6 项留下记录。被取代 Attempt 目录中的未提交文件不并入新 Attempt。两个进程同时对同一 Attempt 做 `submit` 与 `supersede` 时至多一个被接受，重启后从 Store 重建同一结果。进程在 `work wait` 事务提交前后被杀，重启后没有中间状态。

**文件。** `crates/sheltie-runtime/src/service.rs`、`crates/sheltie-runtime/src/failpoint.rs`（只填标 `C005-T09` 的函数）。

**测试。** `service_supersede_commits_in_one_transaction`、`service_wait_supersedes_and_blocks_in_one_transaction`、`service_resume_uses_injected_clock_for_wait_duration`、`service_late_submit_rejected_and_recorded`、`superseded_attempt_output_not_merged_into_new_attempt`、`concurrent_submit_and_supersede_accept_at_most_one`、`restart_after_race_rebuilds_same_result`、`crash_before_wait_commit_leaves_attempt_running_and_work_active`、`crash_after_wait_commit_leaves_superseded_and_resource_wait`

**实现要点。** 判断全部来自 core，runtime 只做存取与事务（INV-2）。并发与崩溃测试用骨架给出的 helper 和 fail-point 名，不自己加 sleep。

**停止条件。** 需要第二把锁或第二套状态。

**提交。** `feat(runtime): 取代与资源等待的事务、并发与崩溃窗口`

### C005-T10 cli 命令与输出

**结果。** `attempt supersede`、`work wait`、`work resume` 与交接包命令可用，`--json` 输出与协议一致；缺参数、原因不合法、状态不合法时给出协议规定的错误码与退出码。交接包命令不改变任何状态。

**文件。** `crates/sheltie-cli/src/commands/{attempt,work}.rs`、`crates/sheltie-cli/src/{output,error_map}.rs`（只填标 `C005-T10` 的函数）。

**测试。** `cli_attempt_supersede_json_reports_superseded_and_next_begin`、`cli_attempt_supersede_requires_reason_and_source`、`cli_work_wait_json_reports_resource_wait_and_source`、`cli_work_wait_rejects_other_reasons`、`cli_work_resume_json_reports_active_and_next_begin`、`cli_work_handoff_prints_projection_without_changing_state`、`cli_late_submit_after_supersede_matches_protocol_error`、`cli_error_codes_for_continuity_rejections_match_protocol`

**实现要点。** 命令树在骨架中定死，这里只填处理函数。输出不自己加字段。

**停止条件。** 协议与命令树不一致。

**提交。** `feat(cli): attempt supersede、work wait、work resume 与交接包命令`

### C005-M2 里程碑审查：runtime 与 cli

**执行者。** 强模型，未参与 T08–T10。

**做什么。**

1. 读 `git diff <M1 结束提交>..HEAD -- crates/sheltie-runtime crates/sheltie-cli`，逐项核对检查表。
2. 运行 `scripts/mutants.sh sheltie-runtime`。
3. 人工走查三个事务的边界和 fail-point 位置，以及迟到提交记录是否与拒绝应答处在 T01 写定的事务关系中。
4. 并发测试在本机连跑 50 次，记录结果；有一次不稳定就退回 T09。
5. 核对旧库拒绝和根外哨兵文件不变；核对 INV-3。

**产出。** 在 `milestones.md` 中追加 M2 报告。

### C005-T11 场景：换宿主、等待与耗尽

**结果。** 全程用真实 CLI：取代后新执行者完成同一个 Work；等待、恢复后在同一 Occurrence 领取，业务计数不变；反复自报不可用导致接续耗尽后只能取消；总等待期限在恢复时耗尽；等待中旧执行者迟到提交被拒；两个真实 CLI 进程并发 `submit` 与 `supersede`；单 agent 全程只用等待与恢复也能跑完。

**文件。** 无生产文件（场景测试只验证已有实现）。

**测试。** `scenario_switch_host_supersede_then_new_attempt_succeeds`、`scenario_wait_resume_same_occurrence_keeps_business_counters`、`scenario_repeated_self_reported_unavailable_exhausts_continuity`、`scenario_total_wait_limit_exhausts_on_resume`、`scenario_late_submit_during_resource_wait_rejected`、`scenario_concurrent_submit_and_supersede_real_cli`、`scenario_single_agent_wait_and_resume_completes_work`、`scenario_policy_denied_cannot_continue_by_switching_host`

**实现要点。** 本任务只删 `#[ignore`，让已写好的场景变绿。任一场景红，说明前面某个任务有缺陷：记下测试名和失败输出，把状态改为 `blocked`，交给审查者，不改生产代码。

**停止条件。** 任一场景红。

**提交。** `test(cli): 启用换宿主、资源等待与接续耗尽场景`

### C005-T12 两份宿主 skill

**执行者。** 强模型。skill 是给协调者读的说明书，措辞直接影响行为。

**结果。** Claude Code 与 Codex 各有一份接手说明：怎样读交接包、何时用 `attempt supersede`、何时用 `work wait`、怎样在恢复后领取新 Attempt、为什么不能降低合同标准、为什么“政策不允许”时要停下找人，以及无法确认旧执行者停止时要换工作区。skill 不保存状态、不做推进判断（GF-18）。`scripts/check-skill.sh` 通过。

**文件。** `skills/`、`crates/sheltie-cli/tests/skill.rs`。

**测试。** `every_command_in_codex_skill_exists`、`handoff_section_commands_exist_in_protocol`（T02 按 T01 第 10 项写定位置）

**提交。** `feat(skill): Claude Code 与 Codex 的接手说明`

### C005-M3 里程碑审查：端到端

**执行者。** 强模型，未参与 T11–T12。

**做什么。**

1. 把 validation.md §5 中不标“第二阶段”、不依赖 C004 的每一行对到一个通过的测试名，写成对照表；依赖 C004 的行标“增量组”，第二阶段的行标“不适用：第二阶段”。
2. spec.md §4 中标“第一阶段实现”的承诺逐行给出正例、反例和信任前提。
3. grep CLI 输出、skill 与交付物，确认没有出现 spec.md §5“不能说”清单里的措辞，尤其是“自动不断档”“额度耗尽”。
4. 全量运行 `cargo nextest run --all-features`，记录总数与耗时；在 validation.md 执行状态表中填写失败路径测试一行。

**产出。** 在 `milestones.md` 中追加 M3 报告。

### C005-T13 两宿主真实接续回归

**执行者。** 人（真实宿主操作者）。实施者准备绑定 M3 候选的二进制、独立管理根、两份 skill 和逐命令记录模板。

**结果。** Claude Code 执行一个 Work 到一半，由人模拟额度耗尽后停下；打开 Codex 调用 skill，读交接包，`attempt supersede` 后领取新 Attempt 完成同一个 Work。再跑一次只有一个宿主的等待与恢复。记录重新解释量、约束丢失、重做量、逐命令响应、宿主 usage（拿不到就记缺失，不记 0）与耗时，并与采用前的人工接续实验对照。

**文件。** 本 package 的 `validation.md`、`progress.md` 与原始记录目录。

**停止条件。** 新宿主需要大量重新解释才能继续：按 engineering.md §7 路由到交接包字段或 skill，不进入发布。

**提交。** `docs(specs): 记录 C005 两宿主真实接续回归`

### C005-T14 发布

**执行者。** 人（发布操作者）；需要用户授权。

**结果。** M3（及增量组的 M4）、T13 通过后发布；写 release record 与 CHANGELOG；第一阶段 package 转为 `completed`，在 `review.md` 新增「实施审查」一节并写最终结论行，满足 `scripts/check-specs.sh`。第二阶段若日后采用，另开 change package。

**提交。** `chore(release): 发布 C005 第一阶段`（版本号以采用时的 README 为准）

## 3. C004 增量组（C004 实现后插入）

C004 已实现（C004-T24 完成）时，T01 或一次补充计划提交把下列任务插入 §1（排在 T13 之前）和 tasks.toml；T02 或届时的补充骨架任务（编号 C005-T20）写出它们的骨架与测试。未插入前，主线不出现这些字段。

| ID | 执行者 | 标题 | 依赖 |
| --- | --- | --- | --- |
| C005-T20 | 强模型 | 增量骨架与测试 | C004 已实现、M3 |
| C005-T21 | 初级 | 交接包加入候选快照、验证结果与未处置发现 | T20 |
| C005-T22 | 初级 | 验证器故障与资源等待分开计数 | T20 |
| C005-T23 | 初级 | 独立审查资格不随换人放宽（仅 C004 VD-09 严格分离已实现时） | T20 |
| C005-T24 | 初级 | 场景：审查者接替与没有合格备选 | T21–T23 |
| C005-M4 | 强模型 | 里程碑审查：增量组 | validation.md §5 中依赖 C004 的各行 |

#### C005-T21 交接包加入 C004 记录

**文件。** `crates/sheltie-core/src/work/handoff.rs`（只填标 `C005-T21` 的函数）。

**测试。** `handoff_lists_candidate_snapshot_from_c004_record`、`handoff_lists_verification_results_with_sources`、`handoff_lists_unresolved_findings_from_c004_records`、`handoff_marks_external_backfill_evidence`

**提交。** `feat(core): 交接包加入候选快照、证据与未处置发现`

#### C005-T22 验证器故障与资源等待分开计数

**结果。** C004 验证节点上的验证器故障经 `attempt fail` 消耗验证节点的 `max_retries`，不计资源重试；对验证节点的 `work wait` 不改变证据。

**文件。** `crates/sheltie-core/src/work/decide.rs`（只填标 `C005-T22` 的函数）。

**测试。** `verifier_timeout_fail_consumes_verify_retries_not_resource_retries`、`wait_on_verify_node_leaves_evidence_unchanged`

**提交。** `feat(core): 验证器故障与资源等待分开计数`

#### C005-T23 独立审查资格

**结果。** 合同要求独立审查时，只剩作者可用的换人被拒绝，`next` 只给等待或取消；资格从 C004 VD-10 的执行身份记录推导，只有声明没有观测时按合同选定的分离模式处理。C004 未实现 VD-09 严格分离时，本任务不插入，EX-07 继续只是 skill 中的流程约定。

**文件。** `crates/sheltie-core/src/work/decide.rs`、`crates/sheltie-core/src/work/next.rs`（只填标 `C005-T23` 的函数）。

**测试。** `supersede_to_author_rejected_when_independent_review_required`、`only_author_available_offers_wait_or_cancel`、`declared_only_identity_follows_contract_separation_mode`

**提交。** `feat(core): 换人不放宽独立审查资格`

#### C005-T24 场景：审查者接替与没有合格备选

**文件。** 无生产文件。

**测试。** `scenario_reviewer_replacement_keeps_findings`、`scenario_no_qualified_reviewer_waits_instead_of_self_review`

**提交。** `test(cli): 启用审查者接替与无合格备选场景`

## 4. 第二阶段（未拆任务）

第二阶段按 README「采用条件」另行采用：第一阶段交接质量稳定；实测存在明显的无人响应等待；ADR-C 审定并修订 spec §6 非目标；确认 C008 是否已采用。采用时按本计划的同一方法补写任务卡，候选分组如下，只作占位：

| 组 | 内容 | 主要风险 |
| --- | --- | --- |
| 执行绑定 | EX-08 三层配置、冻结的候选集合与新授权版本 | 新授权不改写旧 Attempt |
| 执行记录与额度观测 | EX-09、EX-10：请求配置与实际配置分开；缺失记未知 | 不把缺失当零或充足 |
| 候选过滤 | EX-11：机械过滤加固定优先级；C008 未采用时写“依赖未检查” | 额度未知只作一次获准探测 |
| 驱动层 | EX-12、EX-14：内核之外的前台进程；只执行政策允许的操作；领取时重新确认 | 不生成人工批准；不把唯一 `next` 当授权 |
| 宿主失败类别映射 | design.md §8：按后端版本记录映射，映射不了记未知 | 不为单个宿主写专用逻辑进内核 |

## 5. 完成判据

C005 第一阶段完成，当且仅当：

1. §1 表中所有任务和里程碑都是 `done`，每个任务对应一个提交；插入的增量组也是 `done`。
2. design.md §5 转换表每一行都有正例和反例测试；validation.md §5 第一阶段各行都有通过的测试，或写明属于增量组、第二阶段。
3. 仓库里没有 C005 的 `todo!`，也没有 `#[ignore = "C005-`。
4. `scripts/check-core-vocab.sh` 通过，core 中没有宿主、模型、订阅词汇。
5. 业务重试与三种接续计数在所有路径上互不影响；模型自报的等待从不显示为已确认。
6. T13 有人工记录；对外用语与 spec.md §5 一致。
