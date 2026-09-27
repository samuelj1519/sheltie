# C007 实施计划

状态：`draft`。本计划随提案一起准备，package 仍是 `proposed`，下表全部 `todo`。人采用第一阶段、目录移入 `active/` 之后，本计划才是实施进度的唯一权威。

本计划只把**第一阶段（生成、检查、确认，走现有安装路径）**拆成任务。第二阶段（临时启动入口、检查器产品化、`work start` 核对锁定文件）由第一阶段实验结果触发，另行采用，§4 只列入口条件和候选分组。

第一阶段**不改产品代码**（README「范围约束」「采用条件」）。本计划把这条写成机械约束：检查器是仓库里的独立程序 `sheltie-plan-check`，放在 `crates/sheltie-plan-check`（workspace 成员，`publish = false`，不进发布资产，不由 `self` 分发）；它不依赖任何 `sheltie-*` crate，只通过子进程调用 `sheltie` 公开命令；[tasks.toml](tasks.toml) 中每个代码任务的白名单都不含三个产品 crate 与 `skills/`，`scripts/check-task.sh` 会拒绝任何改动它们的提交。design.md 说的“外部脚本”，本计划理解为“引擎之外、不随 Sheltie 发布的独立程序”；用 Rust 写，是为了让检查器享有与产品代码同样的测试、门禁和骨架填空方法，第二阶段产品化（WG-10）时也不必重写。T01 在 ADR-E 里写明这一点。

C007 不以 C004、C005 采用为前提。主线（T01–T18）只用现有 Workbook 能表达的门槛、图、上限与 `requires`；“证据绑定”规则在主线中固定报告“不适用”。依赖 C004 验收合同、C005 执行绑定的规则放在 §3 的增量组。

做法借鉴 [v0.1.0 plan](../../../releases/v0.1.0/plan.md)：**强模型先搭骨架、写全部测试，初级实现者按任务逐一填空，每完成一组里程碑由强模型审查一次。** 实现者第一次接触本仓库，只需要读本文 §0、自己那张任务卡、卡上列出的文件和测试。产品语义以 [spec.md](spec.md) 为准，检查器算法以 [design.md](design.md) §3 为准，锁定文件与安装核对以 design.md §4、§5 为准，失败路径以 [validation.md](validation.md) §5 为准。

模块路径与签名写的是**暂定位置**，C007-T02 的骨架作者按届时的代码定死；需要改路径时，在 T02 的提交里同时改本文任务卡和 tasks.toml，并在 T02 卡的「记录」行写明原因。测试名是合同，T02 必须按任务卡的名字写出测试；确需改名的，同样在 T02 提交里改任务卡。

## 0. 执行方式

### 0.1 四层分工

| 层 | 谁 | 做什么 | 何时 |
| --- | --- | --- | --- |
| 决定 | 人 + 强模型 | 写定约束快照、约束映射、锁定文件的格式与位置，检查器命令，ADR-E；同步路线图 | C007-T01，一次 |
| 骨架 | 强模型 | 检查器 crate、全部类型与签名、文档注释、`todo!("C007-Tnn")` 函数体、调用 `sheltie` 的子进程封装（完整实现）、全部缺陷草案夹具、全部测试（禁用）、快照、工具修补 | C007-T02，一次 |
| 填空 | 初级开发者或初级模型 | 启用本任务测试、填函数体、跑门禁、提交 | C007-T03 起，每任务一次 |
| 审查 | 编译器、clippy、测试、`scripts/check-task.sh` | 逐任务自动 | 每次提交 |
| 审查 | 未参与该组实现的强模型 | 按 [engineering.md §5](../../../engineering.md) 检查表审一组 diff、跑突变测试、对照 validation.md §5 | C007-M1 到 M3（增量组另加 M4） |

authoring 实验与第一阶段收尾由人主持（C007-T17、C007-T18）。

### 0.2 实现者规则

写给做填空任务的人或模型。每条都是硬规则，多数由 `scripts/check-task.sh` 机械核对。

1. **只读三样东西。** 本文 §0、你的任务卡、卡上「文件」与「测试」列出的代码。任务卡点名某合同某节时，只读那一节。
2. **先看到红。** 运行 `scripts/task.sh C007-Tnn`（禁用的测试也会跑），确认本任务测试全红；然后删掉这些测试上的 `#[ignore = "C007-Tnn"]`。编译错误不算红。
3. **只填 `todo!("C007-Tnn")`。** 不改签名、类型、`pub` 可见性，不加依赖，不新建文件。同文件内新增私有辅助函数可以，但不得改变任何公开项的行为边界。文档注释就是函数要做的事。
4. **不改测试，不改快照。** 测试红了改实现。快照不一致改渲染代码，不运行 `cargo insta accept`。
5. **一次一个测试。** 让一个红测试变绿，再做下一个。
6. **绿了跑门禁。** `scripts/task.sh C007-Tnn` 全绿后，把 §1 表中本任务状态改为 `done`，运行 [engineering.md §2.3](../../../engineering.md) 的四条命令与 `scripts/check-task.sh C007-Tnn --staged`。
7. **一次提交。** 第一行用任务卡「提交」行；正文写为什么、怎么验证；末尾是 `Change: C007`、`Task: C007-Tnn`、`Agent: <名字>`。状态改为 `done` 放在同一提交。提交后再运行一次 `scripts/check-task.sh C007-Tnn`，并核对 `git show --stat`。
8. **不顺手改。** 别的任务的 `todo!`、测试、你觉得能优化的地方都不碰。
9. **卡住就停。** 出现下列任一情况时停手，不提交；把状态改为 `blocked`，在 [progress.md](progress.md) 写清原因：两个测试互相矛盾；不改签名或测试就做不到；需要新依赖；同一个测试改了五次还红；任务卡与合同说法不一；需要碰到后续任务的 `todo!`。
   **唯一例外：工具缺陷。** 如果 `scripts/` 或 `tasks.toml` 让一个按规则做的任务无法通过，可以修工具，但要单独提交，写 `Task: C007-T02`，并在提交说明中逐条写明原来错在哪、改成了什么。修复不得放松检查意图，不得改动测试、签名或合同。里程碑审查复核每一次工具改动。
10. **不猜。** 文档注释、测试和合同都没说的行为，不自己发明，按第 9 条停下。在注释里写下“这里由调用方保证”“解析不了当默认值”之类的约定，同样算发明。

实现者是强模型时，可以在一个会话里连做相邻任务，但仍是一任务一提交，每个任务各跑一遍 `task.sh` 与 `check-task.sh`。审查发现缺陷时，审查者只补测试（挂被审任务编号、禁用），实现者修代码。

### 0.3 测试与工具

- **命名与归属。** 测试名写“条件 → 行为”，不带任务编号。归属写在紧贴 `#[test]` 上方的一行注释里：`// Task: C007-Tnn`，再加 `#[ignore = "C007-Tnn"]`。采用后 `scripts/check-tests.sh` 会解析本文每张卡的「**测试。**」行：行内每个反引号里的小写标识符都必须是真实存在、且归属本任务的测试。所以这一行只写测试名，说明放在全角括号（…）里。
- **基准 tag。** T02 提交后打 tag `c007-t02-skeleton`；审查者补测试后打 `c007-tNN-review`。实现者的测试零改动检查以最近一个 `c007-t*` tag 为基准。截至本文写作，`check-task.sh` 对 `Cnnn-Tnn` 默认取 README 的基线提交，这个默认基准会把整份骨架都算作“测试改动”。T02 必须先修好这一点；如果别的 package（C004、C005、C006）已先修过，就直接复用。
- **命令。** `scripts/task.sh C007-Tnn` 只跑本任务测试；`scripts/check-tests.sh` 核对命名与任务卡；`scripts/check-task.sh C007-Tnn [base] [--staged]` 核对改动范围、占位清零、测试零改动、状态与 trailer；`scripts/mutants.sh sheltie-plan-check` 供里程碑审查使用。
- **产品代码零改动。** 检查器放在 `crates/sheltie-plan-check`（放在 `crates/` 下是为了让现有脚本不改扫描范围就能覆盖它）。[tasks.toml](tasks.toml) 中 T03 起每个任务的白名单都只含这个目录；T02 的白名单也不含 `crates/sheltie-core`、`crates/sheltie-runtime`、`crates/sheltie-cli` 与 `skills/`。任何任务需要改这些地方，都按 §0.2 第 9 条停下：那说明第一阶段的前提不成立，要回到人的决定。
- **检查器是黑盒调用者。** 检查器不依赖任何 `sheltie-*` crate。图结构（入口、节点、`gate`、四种边、`requires`）只取自引擎在临时管理根里给出的 `sheltie --json workbook show` 输出；Workbook 摘要只取自引擎的 `workbook add` 返回值。`workbook show` 不输出的 `max_visits`、`max_retries` 由检查器按 [contracts/workbook.md](../../../contracts/workbook.md) 的字段表从 flow 文件读取，缺省值取合同默认值，节点集合必须与 `show` 输出一致，不一致即输入错误。填空任务不得自己写 Workbook 装入校验、不得自己算 Workbook 目录摘要：这两件事只有引擎能说了算。
- **临时管理根。** 每次装入都在系统临时目录下新建一个 `SHELTIE_HOME`，用完即删，失败时也删。检查器从不读写调用者环境里的 `SHELTIE_HOME` 或 `~/.sheltie`；唯一的例外是 `verify-install`，它以只读命令 `workbook show` 读取正式管理根。创建、调用、清理临时管理根的子进程封装 `engine.rs` 由骨架**完整实现**。
- **测试中的 `sheltie`。** 检查器的测试调用真实的 `sheltie` 二进制。定位它的 helper 由骨架完整实现：取与测试二进制同一 `target/<profile>/` 下的 `sheltie`，不存在时 panic 并提示先构建 `sheltie-cli`；`scripts/task.sh` 在运行本 package 的任务前先构建它。测试不 mock 引擎。
- **正式管理根哨兵。** 所有调用引擎的测试都把环境里的 `SHELTIE_HOME` 指向一个预先装好一个 Workbook 的“正式管理根”，断言测试前后它的 `store.db` 字节与目录清单不变。helper 由骨架完整实现。
- **不调用模型。** 检查器与全部测试都不调模型、不联网（validation.md §5）。规划者起草草案不在本计划的自动测试里，只在 T17 的人工实验里出现。
- **测试分层。** 纯规则（约束、映射、图、各条规则、判定、报告）用模块内单元测试，报告用 `insta` 快照；调用引擎的部分（装入、摘要、锁定文件、安装核对）用 crate 内集成测试加临时目录；命令行与场景用 `assert_cmd` 驱动 `sheltie-plan-check` 和 `sheltie` 两个真实二进制。
- **独立 oracle。** 图规则的期望路径由测试直接写出；另有暴力枚举简单路径的对照测试（小图上随机生成），oracle 在测试文件里由骨架写好，不复用被测函数。摘要的期望值由测试用 `sha2` 对写死的规范文本重新计算；Workbook 摘要的期望值由测试直接调用引擎取得。
- **夹具。** `crates/sheltie-plan-check/tests/fixtures/` 下的草稿、约束快照与映射全部由骨架写好，含 validation.md §3 要求的缺陷草案：图路径类三份（绕过审查的 `branch` 边、返工后不经测试直达终点、测试之后未排除的门槛节点直达终点），输入类两份（审查映射到空壳节点、真正改候选的节点被排除出 \(P\)）。填空任务不增删夹具。

### 0.4 复核与里程碑记录

- 逐任务不安排模型复核；由 `check-task.sh`、门禁和测试完成机械审查。
- 里程碑审查者没有参与本组实现。审查中发现缺陷时，审查者补禁用测试、打 `c007-tNN-review`，并把对应任务改回 `doing`；实现者修复后再提交一次，写被退回任务的 `Task:`，不写 `Task: C007-Mn`。审查者自己补测试的提交写 `Task: C007-Mn`。
- 缺陷涉及签名、数据结构、`engine.rs` 或夹具、会让大量测试一起变红时，由审查者（骨架作者）直接修改，并在报告中记录。
- 里程碑报告写入本 package 的 `milestones.md`（M1 创建此文件；`review.md` 是提案讨论记录，不混写）。报告包含：检查表逐项结论、存活突变体的数量与处置、validation.md §5 的行与测试名对照、每一次工具改动的复核结论、退回清单，以及一节「流程教训」（每条写明证据与落点）。结论只用“通过 / 需修改 / 阻断”。validation.md 的执行状态表由里程碑审查者填写：写命令、原始输出路径、退出码和输入闭包，不只写 PASS。

### 0.5 成本

任务卡约 300–500 字；每个任务的骨架在 80–250 行，测试在 100–350 行；每个任务的输入上下文在 3k–7k token 之间。检查器整体不大（估计 2–3k 行 Rust，其中一半是测试与夹具），但它是 C007 唯一的可信部件：规划者、执行者都可能出错，检查器错了就没人兜底。所以骨架、oracle 和夹具都由强模型写，初级实现者只做图遍历、解析和组装这类有测试钉住的工作。

## A. 进入本计划前（不在任务表内）

| 事项 | 谁 | 结果去向 |
| --- | --- | --- |
| 人采用 C007 第一阶段（README「采用条件」） | 人 | README 状态改为 `active`，目录移入 `active/` |
| C002 已完成，或已明确不影响 `workbook add` 的装入校验与摘要编码（`workbook-digest/v2`） | C002 Owner | C002 release record；T01 核对 |
| C004 阶段 1 的任务样本已登记，可以共用（validation.md §3） | C004 Owner | C004 validation.md；T17 前置 |
| 人审定 ADR-E（建议 E2：引擎之外的独立检查器） | 人 | README 采用条件；T01 移入 `specs/decisions/` |

C007 第一阶段不以 C004、C005 采用为前提，也不需要等待 C006。

## 1. 任务表

状态只用 `todo | doing | blocked | done`。表格顺序就是执行顺序。§3 的增量组在 C004、C005 实现后由人决定插入本表，排在 T17 之前。

| ID | 状态 | 执行者 | 依赖 | 标题 | 结果 |
| --- | --- | --- | --- | --- | --- |
| C007-T01 | todo | 人 + 强模型 | 采用 | 实施前决定与 ADR-E | 约束快照、映射、锁定文件的格式与位置写定；检查器命令与退出码写定；ADR-E 编号；README 的“外部脚本”说法改为本计划的形态 |
| C007-T02 | todo | 强模型 | T01 | 骨架、全部测试、夹具与工具 | `sheltie-plan-check` 可编译；`engine.rs` 与测试 helper 完整；全部测试存在且禁用；缺陷草案夹具齐全；tag `c007-t02-skeleton` |
| C007-T03 | todo | 初级 | T02 | 约束快照：解析与合成 | `constraints/v0` 闭集校验；项目默认与本次差异合成，每项保留来源；放宽与加严逐条标出 |
| C007-T04 | todo | 初级 | T03 | 约束映射、图模型与候选生产集合 | 映射解析；从 `workbook show` 输出与 flow 文件建图；\(P\) 默认全选、显式排除 |
| C007-T05 | todo | 初级 | T04 | 规则：映射完整与路径覆盖 | 未映射条件、未知节点逐条列出；删去 \(N_c\) 后入口不能到达终点，否则给出见证路径 |
| C007-T06 | todo | 初级 | T05 | 规则：候选之后与门槛存在 | \(N_c \cap P = \varnothing\)；删去 \(N_c\) 后 \(P\) 不能到达终点；门槛条件的节点都是 `gate = true` |
| C007-T07 | todo | 初级 | T04 | 规则：上限有界与不越权 | 每节点上限；尝试次数上界计算并报告；`requires` 名单 |
| C007-T08 | todo | 初级 | T05–T07 | 未决清单与规则判定 | 规则判定与未决清单两个独立字段；证据绑定标“不适用”；装入失败后不再检查其他规则 |
| C007-M1 | todo | 强模型 | T03–T08 | 里程碑审查：规则 | 规则逐条正反例；暴力枚举 oracle 复核；`scripts/mutants.sh sheltie-plan-check`；validation.md §5 规则类各行对到测试 |
| C007-T09 | todo | 初级 | M1 | 可装入检查与取图 | 每次新建临时管理根，用完即删；调用者的管理根不被触碰；装入时排除已有锁定文件 |
| C007-T10 | todo | 初级 | T09 | 摘要与锁定文件格式 | `plan_digest` 取引擎对去掉锁定文件的目录给出的摘要；约束与映射摘要按规范文本计算；`plan-lock/v0` 读写 |
| C007-T11 | todo | 初级 | T10 | 写锁定文件与确认口令 | 只在规则判定通过时写；已有锁定文件时拒绝；确认口令必须对应同一组输入 |
| C007-T12 | todo | 初级 | T11 | 安装前核对、安装后比对与默认要求确认记录 | 重算三项摘要；最终目录装入得到 \(D\)；正式安装摘要与 \(D\) 比较；默认要求文件摘要变化时要求重新确认 |
| C007-M2 | todo | 强模型 | T09–T12 | 里程碑审查：引擎交互与锁定文件 | 哨兵与临时目录清理逐条验证；摘要 oracle 复核；突变测试；`engine.rs` 复核 |
| C007-T13 | todo | 初级 | M2 | 确认材料与报告用语 | 文本与 JSON 报告；排除清单逐条展示；注明映射真实性需用户确认；不出现“整份约束已满足” |
| C007-T14 | todo | 初级 | T13 | 命令行与退出码 | `check`、`lock`、`preinstall`、`verify-install` 四个子命令；退出码闭集；`--json` 字段稳定 |
| C007-T15 | todo | 初级 | T14 | 场景：validation.md §5 全部 23 行 | 两个真实二进制逐行走完；每行一个测试 |
| C007-T16 | todo | 强模型 | T15 | authoring skill | 规划者按它起草草案与映射；示例草案都能通过检查器；命令都存在 |
| C007-M3 | todo | 强模型 | T13–T16 | 里程碑审查：端到端 | validation.md §5 逐行对到测试；spec.md §4 承诺逐行正反例；三个产品 crate 与 `skills/` 零改动；发布资产不含检查器 |
| C007-T17 | todo | 人 + 强模型 | M3（及插入的增量组里程碑） | 第一阶段 authoring 实验 | validation.md §3、§4 按预先写定的口径执行并报告 |
| C007-T18 | todo | 人 | T17 | 第一阶段收尾与第二阶段决定 | 实验结论入 README 与 validation.md；决定第二阶段是否采用；package 转 `completed` 或保持 `active` |

## 2. 任务卡

每张卡给：结果、文件（只能改这些）、测试（要变绿的测试名）、实现要点（填空提示，不是设计）、停止条件、提交（信息第一行）。下文 `PC` 指 `crates/sheltie-plan-check`；[tasks.toml](tasks.toml) 里写的是完整路径。

### C007-T01 实施前决定与 ADR-E

**执行者。** 人审定，强模型起草。

**结果。** ADR-E 移入 `specs/decisions/` 并编号，写明第一阶段检查器的形态：仓库内不发布的独立程序 `sheltie-plan-check`（workspace 成员，`publish = false`，不进发布资产，不由 `self` 分发），只通过 `sheltie` 公开命令与引擎交互，不依赖任何 `sheltie-*` crate。本 package 的 README、design.md 中“外部脚本”的说法改为这一形态，并写明它仍满足“第一阶段不改产品代码”：`sheltie` 二进制、三个产品 crate 与 `skills/` 零改动。`specs/roadmap.md` 登记 C007 第一阶段为进行中的实验。第一阶段不改 `specs/spec.md`、`constitution.md` 与 `contracts/`（spec.md §6 的上游修订属于第二阶段）。README 的 `基线：` 改为实施起点提交。

**必须写定的决定。** 括号里是本计划的建议，写进 design.md 对应章节：

1. 约束快照格式 `constraints/v0`（design.md §2；建议：`[[require]]` 的 `kind` 闭集 `machine_check | review | gate`，回答第 4 问；每项带 `source` 闭集 `project_default | run_delta | execution_policy`；新增 `[budget]` 表，出现任何键都进未决清单并标“需运行时控制”；新增 `[[unstructured]]`，每项 `id` 与 `needs = "runtime" | "human"`，用来写“不改变鉴权语义”一类要求；未知顶层键是输入错误，不静默忽略）。
2. 快照怎样合成（建议：检查器读项目默认文件与本次差异文件，合成生效快照；差异文件可以新增条目、覆盖上限、用 `[[drop]]` 删去条目；覆盖后变宽的上限与删去的条目标“放宽”，其余标“加严”或“新增”，全部进确认材料；`[execution]` 只记录）。
3. 约束映射格式（建议 `mapping/v0`：`flow` 写 Flow id；`[map]` 表写条件 id → 节点列表；`not_candidate_producers` 写 \(P\) 的排除清单，放在顶层，不与条件 id 共用命名空间；`[[suggest]]` 写规划者的差异说明，只展示、不参与判定）。
4. 草稿有多个 Flow 时怎么办（建议：第一阶段只接受恰好一个 Flow 的草稿，否则输入错误；生成场景一次只为一个任务起草一个 Flow）。
5. 锁定文件路径与格式（design.md §4，第 2 问；建议 `resources/plan.lock.toml`、`plan-lock/v0`，字段同 design.md §4，`[mapping]` 表照抄映射的 `[map]` 与 `not_candidate_producers`；第一阶段不要求 `contracts/workbook.md` 保留这个名字，第二阶段再定）。
6. 三个摘要怎样算（建议：`plan_digest` 与 `draft_load_digest` 都取引擎 `workbook add` 对去掉锁定文件的目录返回的 `digest`，所以锁定时两者相等，分开保留是因为前者在安装前重算、后者记录检查时装入了什么；`constraints_digest` 与 `mapping_digest` 是生效快照与映射的规范 JSON（键排序、条目按 id 排序、UTF-8、无多余空白）的 SHA-256）。
7. 检查器的命令与退出码（建议：`check`、`lock --confirm <口令>`、`preinstall`、`verify-install --expected <D> <id>@<version>`；退出码 `0` 通过、`1` 规则不通过或核对不一致、`2` 输入错误、`3` 需要重新确认项目默认要求、`4` 引擎调用失败；`--json` 输出字段写进 design.md；`--sheltie <路径>` 指定引擎，缺省从 `PATH` 找）。
8. 确认口令（建议：`check` 输出一个口令，等于约束摘要、映射摘要与计划摘要拼接后的摘要前 12 位；`lock` 重新计算，不同就拒绝写。这样“看到的”与“锁定的”必须是同一组输入）。
9. 项目默认要求的确认记录（design.md §2 输入闭包，第 1 问第一阶段部分；建议：记录文件放在用户配置目录下，不在仓库、不在 `~/.sheltie`；按默认要求文件的绝对路径记最近一次确认的摘要；不一致或没有记录时 `check` 返回退出码 `3`，`lock --reconfirm-defaults` 更新记录）。
10. authoring skill 放哪（第 5 问；建议放在 `crates/sheltie-plan-check/skill/SKILL.md`，随检查器走，不进 `skills/`，不随 Sheltie 分发；第二阶段产品化时再决定分发）。
11. 用户确认记录（第 6 问；第一阶段只在 T17 的实验记录里写，不设格式，不进锁定文件）。

**文件。** `specs/`、`CONTEXT.md`（如需新增“约束快照”“约束映射”“锁定文件”三个词）。

**验证。** `scripts/check-docs.sh`、`scripts/check-specs.sh` 绿。人逐条确认上面 11 项都已写定且有出处；validation.md §5 的 23 行，每行都能在写定的格式和命令里找到对应的输入与输出。

**停止条件。** 任一项需要改引擎（例如发现 `workbook show --json` 缺少取图必需的字段）；这说明第一阶段“不改产品代码”不成立，交回人决定。

**提交。** `docs(specs): 写定 C007 约束快照、映射、锁定文件与检查器命令`

### C007-T02 骨架、全部测试、夹具与工具

**执行者。** 强模型。这是唯一做代码设计的任务。

**结果。** 全仓可编译；新 crate 的类型、签名和文档注释齐全，函数体是 `todo!("C007-Tnn")`；`engine.rs`、命令行定义与测试 helper 完整实现；本文每张卡列出的测试全部写好，挂 `// Task:` 注释并禁用；报告快照预写；夹具齐全。`scripts/task.sh C007-T03` 退出非零，并列出 T03 的测试名。

**文件（暂定，按实施起点代码定死）。** 新 crate `crates/sheltie-plan-check`（二进制 `sheltie-plan-check`，`publish = false`，`[package.metadata.dist] dist = false`）：

- `src/main.rs`、`src/cli.rs`（clap 命令树，**完整实现**）、`src/commands.rs`（四个子命令的组装，T14 填）；
- `src/constraints.rs`（T03）、`src/mapping.rs` 与 `src/graph.rs`（T04；`graph.rs` 里的 `reachable_avoiding` 归 T05）；
- `src/rules/{coverage,candidate,bounds}.rs`（T05、T06、T07）、`src/verdict.rs`（T08）；
- `src/engine.rs`（新建与清理临时管理根、调用 `sheltie --json workbook add|show`、解析返回的信封，**完整实现**）、`src/load.rs`（T09）；
- `src/digest.rs`、`src/lock.rs`（T10、T11）、`src/install.rs` 与 `src/defaults.rs`（T12）；
- `src/report.rs`（T13）；
- `skill/SKILL.md`（T16 写，骨架只放标题）；
- 测试：模块内单元测试与快照；`tests/{load,digest,lock,install,cli,scenario_check,skill}.rs`；`tests/fixtures/`；`tests/support/`（定位 `sheltie`、正式管理根哨兵、临时草稿复制、暴力路径 oracle、随机小图生成，**完整实现，不留填空**）。
- 工具：`scripts/check-task.sh` 的 tag 基准（如尚未修）；`scripts/task.sh` 在运行 `C007-*` 任务前构建 `sheltie`；根 `Cargo.toml` 加 workspace 成员；`deny.toml` 按需更新；CI 覆盖新 crate。

**骨架的写法。** 类型逐字段对照 T01 写定的格式。每个公开项的文档注释，第一行写对应的出处（WG-nn、design.md §n 或 validation.md §5 的行），第二行写返回的错误或报告项。每条规则是一个函数，输入是图、生效快照与映射，输出是本规则的结果（通过，或不通过加上涉及的条件、节点与见证路径），不读文件、不调引擎。报告与判定用的类型里，规则判定与未决清单是两个字段，类型上就不能合并。

**验证。**

1. 四条门禁、`cargo deny check`、`scripts/check-docs.sh`、`scripts/check-core-vocab.sh`、`scripts/check-tests.sh`、`scripts/check-skill.sh` 全绿。
2. `cargo nextest run --all-features` 显示新增测试全部 ignored、零失败。
3. **正例试跑：** 临时填满 T03，运行 `scripts/check-task.sh C007-T03` 必须通过，然后撤掉填充。
4. **依赖顺序检查：** 对每个填空任务，只启用本任务的测试时，触发的 panic 都来自本任务的 `todo!("C007-Tnn")`。
5. **反例：** 改一行测试后，`check-task.sh` 必须失败；改一行 `crates/sheltie-core` 后，`check-task.sh C007-T03` 必须失败。
6. **夹具试跑：** 用一个临时的参考实现跑一遍全部夹具，确认每份草稿在它该通过或不通过的规则上表现正确，然后撤掉参考实现。夹具的预期结果写在夹具目录的 `EXPECTED.md` 里。
7. **依赖图：** `cargo tree -p sheltie-plan-check` 不含任何 `sheltie-*` crate；`git diff <基线> -- crates/sheltie-core crates/sheltie-runtime crates/sheltie-cli skills` 为空。

**记录。** 相对本文暂定路径的每一处改动写在这里：原路径 → 新路径 → 原因。

**提交。** `chore(workspace): 搭起 C007 检查器骨架、全部禁用测试、夹具与任务工具`。提交后打 tag `c007-t02-skeleton`。

### C007-T03 约束快照：解析与合成

**结果。** 解析 `constraints/v0`：schema 不符、`kind` 或 `source` 不在闭集、未知顶层键、重复条件 id 都是输入错误。项目默认与本次差异合成生效快照，每项保留来源；差异造成的放宽（删去条目、上限变宽）与加严逐条标出。`[execution]` 原样保留，标“只记录”。生效快照能输出规范文本（条目按 id 排序）。

**文件。** `PC/src/constraints.rs`。

**测试。** `constraints_parse_minimal_snapshot`、`constraints_reject_unknown_schema`、`constraints_reject_kind_outside_closed_set`、`constraints_reject_unknown_top_level_key`、`constraints_reject_duplicate_require_id`、`constraints_compose_keeps_source_per_item`、`constraints_delta_relaxation_is_marked`、`constraints_delta_tightening_is_marked`、`constraints_execution_section_recorded_not_judged`、`constraints_canonical_form_sorts_requires_by_id`

**实现要点。** 用 `serde` 的 `deny_unknown_fields` 挡未知键，闭集用枚举。合成是纯函数：先放项目默认，再按差异文件的顺序应用新增、覆盖与删去。上限“变宽”指数值变大。

**停止条件。** 需要判断一条差异是否“合理”；检查器只标出放宽，不评价。

**提交。** `feat(plan-check): 解析并合成约束快照`

### C007-T04 约束映射、图模型与候选生产集合

**结果。** 解析 `mapping/v0`，规划者的差异说明只保留为展示项。从引擎 `workbook show` 的 JSON 建图：入口、节点、`gate`、四种边保持声明顺序，终点是没有出边的节点。从 flow 文件读每个节点的 `max_visits`、`max_retries`，缺省取合同默认值；节点集合与 `show` 输出不一致是输入错误。草稿不止一个 Flow 时是输入错误。候选生产集合 \(P\) 默认是除终点外的全部节点，门槛节点也计入，再减去排除清单。

**文件。** `PC/src/mapping.rs`、`PC/src/graph.rs`。

**测试。** `mapping_parse_minimal`、`mapping_reject_unknown_schema`、`mapping_suggestions_kept_as_display_only`、`graph_from_show_json_keeps_entry_edges_and_gates`、`graph_terminals_are_nodes_without_out_edges`、`graph_limits_default_to_contract_values`、`graph_limits_node_set_must_match_show_output`、`graph_rejects_draft_with_more_than_one_flow`、`producers_default_to_all_but_terminal_including_gates`、`producers_exclusion_list_removes_named_nodes`

**实现要点。** 图用骨架给的邻接表类型，节点与边的顺序照输入，不排序（见证路径要可复现）。读 flow 文件只取节点 id 与两个上限，其他字段不看，也不做任何装入校验：那是引擎的事。

**停止条件。** 发现 `workbook show` 的 JSON 缺少建图需要的字段；或需要自己校验 Workbook 合法性。

**提交。** `feat(plan-check): 约束映射、图模型与候选生产集合`

### C007-T05 规则：映射完整与路径覆盖

**结果。** 映射完整：每个必需条件至少映射到一个节点，列出全部未映射条件；映射引用的节点、排除清单里的节点都必须存在；映射里出现快照没有的条件 id 也不通过。路径覆盖：对每个必需条件 \(c\)，从图中删去 \(N_c\) 后入口不能到达任一终点；否则不通过，并给出一条见证路径。

**文件。** `PC/src/rules/coverage.rs`、`PC/src/graph.rs`（只填 `reachable_avoiding`）。

**测试。** `mapping_completeness_lists_unmapped_required_conditions`、`mapping_completeness_rejects_unknown_node`、`mapping_completeness_rejects_unknown_condition_id`、`mapping_completeness_rejects_unknown_excluded_node`、`path_coverage_passes_when_every_path_hits_mapped_node`、`path_coverage_fails_on_bypassing_branch_with_witness_path`、`path_coverage_counts_all_four_edge_kinds`、`path_coverage_agrees_with_bruteforce_on_small_graphs`

**实现要点。** `reachable_avoiding(图, 起点集合, 删去集合)` 用广度优先，按边的声明顺序展开，找到第一个终点就回溯出路径；起点本身在删去集合里时视为不可达。不要只判断“图里有这个节点”。

**停止条件。** 暴力对照测试与见证路径测试要求的路径不同（说明“第一条”的定义不清）。

**提交。** `feat(plan-check): 映射完整与路径覆盖规则`

### C007-T06 规则：候选之后与门槛存在

**结果。** 对 `after_candidate = true` 的必需条件：\(N_c\) 与 \(P\) 有交集时不通过，列出交集节点并提示“列入排除清单或拆出单独的检查节点”；否则从图中删去 \(N_c\) 后，\(P\) 中任何节点都不能到达终点，不然给出见证路径。`after_candidate = false` 的条件跳过本规则，报告中注明。`kind = "gate"` 的条件：\(N_c\) 中每个节点都必须 `gate = true`，列出不满足的节点。

**文件。** `PC/src/rules/candidate.rs`。

**测试。** `after_candidate_fails_when_mapped_node_is_producer`、`after_candidate_passes_when_mapped_check_node_excluded`、`after_candidate_fails_when_back_edge_rework_reaches_terminal_unchecked`、`after_candidate_fails_for_unexcluded_node_after_check`、`after_candidate_fails_for_unexcluded_gate_node_after_check`、`after_candidate_skipped_when_condition_not_after_candidate`、`after_candidate_agrees_with_bruteforce_on_small_graphs`、`gate_rule_fails_when_mapped_node_not_gate`、`gate_rule_passes_when_all_mapped_nodes_gate`、`gate_rule_applies_only_to_gate_kind`

**实现要点。** 复用 T05 的 `reachable_avoiding`，起点集合是 \(P\)。门槛规则只看节点的 `gate` 字段；它同时要满足的路径覆盖与候选之后由 T05 和本卡的另一半负责，这里不重复判断。

**停止条件。** 需要知道节点内部先改后测还是先测后改（INV-2，检查器看不到）。

**提交。** `feat(plan-check): 候选之后与门槛存在规则`

### C007-T07 规则：上限有界与不越权

**结果。** 每个节点的 `max_visits`、`max_retries` 分别不超过快照的 `max_visits_per_node`、`max_retries_per_node`，列出超限节点与数值。尝试次数上界 \(\sum_n \text{max\_visits}(n) \times (1 + \text{max\_retries}(n))\) 对全部节点计算，无论通过与否都写进结果；超过 `max_attempts_total` 时不通过。第一阶段的不越权：Workbook `requires` 的每一项 `(kind, name)` 都在 `[requires_allow]` 中，列出不在名单内的项。

**文件。** `PC/src/rules/bounds.rs`。

**测试。** `limits_fail_when_node_max_visits_over_limit`、`limits_fail_when_node_max_retries_over_limit`、`limits_report_computed_attempt_bound`、`limits_fail_when_attempt_bound_over_total`、`limits_attempt_bound_counts_every_node`、`requires_rule_fails_when_require_not_in_allowlist`、`requires_rule_passes_with_empty_requires`、`requires_rule_matches_kind_and_name_together`

**实现要点。** 求和用 `u64` 并用 `checked_*`，溢出按超限处理。门槛与终点节点同样计入上界。`kind` 与 `name` 必须同时匹配，只有名字相同不算。

**停止条件。** 快照缺少某个上限字段时不知道算通过还是不通过（T01 应已写定，没写就停）。

**提交。** `feat(plan-check): 上限有界与 requires 名单规则`

### C007-T08 未决清单与规则判定

**结果。** 未决清单：`[budget]` 的每个键一条，标“需运行时控制”；`[[unstructured]]` 每项一条，按 `needs` 标“需运行时控制”或“需人判断”。规则判定：适用的规则全部通过才是 `pass`；不通过时汇总每条规则的条件、节点与见证路径。证据绑定规则在主线中固定为“不适用（C004 未采用）”，不计入通过也不计入不通过。装入失败时判定为不通过，只报告装入错误，不运行其他规则。规划者的差异说明不影响判定。规则判定与未决清单在结果类型里是两个独立字段。

**文件。** `PC/src/verdict.rs`。

**测试。** `unresolved_lists_budget_entries_as_runtime`、`unresolved_lists_unstructured_items_with_declared_need`、`unresolved_is_independent_of_rule_verdict`、`evidence_binding_not_applicable_without_c004`、`verdict_fails_if_any_applicable_rule_fails`、`verdict_lists_rule_nodes_and_paths_on_failure`、`verdict_ignores_planner_suggestions`、`verdict_stops_after_load_failure`

**实现要点。** 规则顺序固定：映射完整、路径覆盖、候选之后、门槛存在、上限有界、不越权、证据绑定。映射不完整时后面的图规则仍然运行（给出全部问题），但装入失败时全部不运行。

**停止条件。** 需要根据未决条目的内容决定判定结果（未决清单永不影响规则判定）。

**提交。** `feat(plan-check): 规则判定与未决清单`

### C007-M1 里程碑审查：规则

**执行者。** 未参与 T03–T08 的强模型。

**做什么。** 按 [engineering.md §5](../../../engineering.md) 检查表审 T03–T08 的 diff；运行 `scripts/mutants.sh sheltie-plan-check`，存活的突变体逐个处置（补测试或写明等价）；复核暴力枚举 oracle 与随机图生成覆盖了四种边、回环与多终点；把 validation.md §5 中只涉及规则的各行（映射、路径、候选之后、门槛、上限、未决、`requires`、证据绑定不适用）对到测试名。另外亲手构造至少三张夹具之外的图，核对见证路径。

**产出。** 创建 `milestones.md`，写 M1 报告。

**提交。** `docs(c007): M1 规则里程碑审查`（补测试时另起提交，写 `Task: C007-M1`）

### C007-T09 可装入检查与取图

**结果。** 可装入检查：把草稿复制到临时目录（去掉已有的锁定文件），新建一个临时 `SHELTIE_HOME`，运行 `workbook add`；成功就用同一临时管理根运行 `workbook show` 取图，并记下返回的 `digest`；失败就返回引擎的错误码与信息。无论成败，临时目录与临时管理根都删除。调用者环境里的 `SHELTIE_HOME` 与 `~/.sheltie` 从不被读写。检查通过后，在另一个管理根正式安装同一草稿不会遇到 `WORKBOOK_EXISTS`。

**文件。** `PC/src/load.rs`。

**测试。** `load_check_uses_fresh_temp_home_each_time`、`load_check_reports_engine_error_code_on_invalid_draft`、`load_check_never_touches_ambient_sheltie_home`、`load_check_removes_temp_home_after_success`、`load_check_removes_temp_home_after_failure`、`load_check_excludes_existing_lock_file`、`load_check_extracts_graph_from_workbook_show`、`formal_install_after_load_check_does_not_hit_workbook_exists`

**实现要点。** 只用 `engine.rs` 提供的函数启动 `sheltie`，它已经负责设置环境变量、解析信封和清理；本卡负责按顺序组合，并保证清理在每条返回路径上都执行（用骨架给的守卫类型）。

**停止条件。** 需要改 `engine.rs`，或需要在调用者的管理根里做任何事。

**提交。** `feat(plan-check): 在临时管理根中检查可装入并取图`

### C007-T10 摘要与锁定文件格式

**结果。** `plan_digest`：引擎对去掉锁定文件的草稿目录返回的 `digest`（经 T09 的临时装入取得）。`constraints_digest`、`mapping_digest`：生效快照与映射规范 JSON 的 SHA-256。`plan-lock/v0` 能读写并往返一致；规则判定与未决清单分成两个字段；未知 schema 拒绝。

**文件。** `PC/src/digest.rs`、`PC/src/lock.rs`。

**测试。** `plan_digest_equals_engine_digest_of_lockless_copy`、`plan_digest_ignores_lock_file`、`constraints_digest_matches_hand_computed_sha256`、`mapping_digest_matches_hand_computed_sha256`、`lock_file_roundtrips_plan_lock_v0`、`lock_file_keeps_rule_verdict_and_unresolved_apart`、`lock_file_rejects_unknown_schema`

**实现要点。** 规范 JSON 由 T03 与 T04 给出的规范形式序列化，不在这里重新排序。Workbook 目录摘要不自己算，调用 T09。

**停止条件。** 手算的期望摘要与实现不一致，且找不到规范文本的差异（说明 T01 的规范定义有歧义）。

**提交。** `feat(plan-check): 计划、约束与映射摘要及锁定文件格式`

### C007-T11 写锁定文件与确认口令

**结果。** `lock` 重新运行全部检查，重新计算确认口令；口令与 `--confirm` 给的不同就拒绝写。只有规则判定为 `pass` 时才写 `resources/plan.lock.toml`（`resources/` 不存在时创建）；草稿里已有锁定文件时拒绝，提示先删除再确认。写入的内容含检查器版本、四个摘要、规则判定、未决清单和映射。草稿里的其他文件不变。

**文件。** `PC/src/lock.rs`。

**测试。** `lock_written_only_when_verdict_passes`、`lock_refuses_when_lock_file_exists`、`lock_rejects_confirm_token_from_different_inputs`、`lock_records_checker_version_and_digests`、`lock_written_under_resources_directory`、`lock_leaves_other_draft_files_unchanged`

**实现要点。** 先写同目录临时文件再改名，避免留下半个锁定文件。检查器版本取 `CARGO_PKG_VERSION`。

**停止条件。** 需要在规则判定不通过时也写点什么（不写，报告即可）。

**提交。** `feat(plan-check): 确认口令与锁定文件写入`

### C007-T12 安装前核对、安装后比对与默认要求确认记录

**结果。** `preinstall`：锁定文件的规则判定必须是 `pass`；重算 `plan_digest`、`constraints_digest`、`mapping_digest`，逐项与锁定文件比较，不一致就逐项报告；都一致时，把含锁定文件的最终目录装入一个新的临时管理根，返回摘要 \(D\)。`verify-install`：以只读命令读取正式管理根里 `<id>@<version>` 的 `digest`，与 `--expected` 的 \(D\) 比较；正式管理根的 `store.db` 字节不变。默认要求确认记录：默认要求文件的摘要与记录不一致或没有记录时，返回“需要重新确认”；`--reconfirm-defaults` 更新记录。

**文件。** `PC/src/install.rs`、`PC/src/defaults.rs`。

**测试。** `preinstall_reports_draft_change_after_lock`、`preinstall_reports_constraints_change`、`preinstall_reports_mapping_change`、`preinstall_returns_final_load_digest`、`preinstall_refuses_lock_with_failing_verdict`、`verify_install_reports_mismatch_when_installed_digest_differs`、`verify_install_passes_when_digests_match`、`verify_install_reads_formal_home_read_only`、`defaults_digest_mismatch_requires_reconfirm`、`defaults_reconfirm_updates_record`

**实现要点。** `verify-install` 是检查器唯一读正式管理根的地方，只用 `engine.rs` 的只读调用。确认记录的写法同 T11：临时文件加改名。

**停止条件。** 需要写正式管理根；或需要把 \(D\) 存在草稿目录里（存了就会改变草稿摘要）。

**提交。** `feat(plan-check): 安装前核对、安装后比对与默认要求确认记录`

### C007-M2 里程碑审查：引擎交互与锁定文件

**执行者。** 未参与 T09–T12 的强模型。

**做什么。** 按检查表审 T09–T12 的 diff；复核 `engine.rs` 与每一条清理路径（手动在临时装入中途杀掉检查器，确认临时目录的处置与报告一致，并把结果写进报告）；复核所有摘要测试的 oracle 独立于被测函数；运行突变测试；把 validation.md §5 中涉及装入、摘要、安装与默认要求的各行对到测试名。

**产出。** `milestones.md` 追加 M2 报告。

**提交。** `docs(c007): M2 引擎交互与锁定文件里程碑审查`

### C007-T13 确认材料与报告用语

**结果。** 报告有文本与 JSON 两种形式，内容相同：计划摘要（Flow、节点数、边数）、生效快照（每项带来源，放宽与加严标出）、约束映射、\(P\) 的排除清单逐条列出、规划者的差异说明（标为“建议，不影响判定”）、逐条规则结果（不适用的规则标“不适用”并写原因）、规则判定、未决清单、确认口令。报告固定写一句“映射是否真实、被排除的节点是否真的不改候选，检查器无法判断，需要你确认”。规则判定通过、未决清单非空时写“所有可检查规则通过，另有未决条目”，任何情况下都不出现“整份约束已满足”或同义说法。

**文件。** `PC/src/report.rs`。

**测试。** `report_text_snapshot_pass_with_unresolved`、`report_text_snapshot_fail_with_paths`、`report_json_keeps_rule_verdict_and_unresolved_separate`、`report_never_says_all_constraints_satisfied`、`report_lists_every_excluded_producer`、`report_notes_mapping_truth_needs_user_confirmation`、`report_shows_planner_suggestions_as_suggestions`、`report_marks_not_applicable_rules`、`report_shows_source_per_constraint`

**实现要点。** 固定用语是骨架给的常量，照用，不改写。快照不一致改渲染代码，不运行 `cargo insta accept`。

**停止条件。** 需要新增一种固定用语。

**提交。** `feat(plan-check): 确认材料与报告用语`

### C007-T14 命令行与退出码

**结果。** `check`、`lock`、`preinstall`、`verify-install` 按 T01 写定的参数工作，把前面各任务的函数组装起来；退出码按闭集返回；`--json` 输出字段与 design.md 一致；`--sheltie` 指定引擎，缺省从 `PATH` 查找，找不到时退出码 `4` 并说明。

**文件。** `PC/src/commands.rs`。

**测试。** `cli_check_exit_code_zero_on_pass`、`cli_check_exit_code_one_on_fail`、`cli_check_exit_code_two_on_input_error`、`cli_check_exit_code_three_when_defaults_need_reconfirm`、`cli_check_json_output_has_stable_fields`、`cli_lock_reports_confirm_token_mismatch`、`cli_preinstall_prints_final_digest`、`cli_verify_install_exit_code_on_mismatch`、`cli_reports_missing_sheltie_binary`

**实现要点。** 每个子命令只做“读参数 → 调一个已有函数 → 渲染 → 映射退出码”，不在这里写判断。

**停止条件。** 需要改 `cli.rs` 的参数定义。

**提交。** `feat(plan-check): 四个子命令与退出码`

### C007-T15 场景：validation.md §5 全部 23 行

**结果。** 每行一个测试，按表格顺序排列，用两个真实二进制和骨架给的夹具走完“检查 → 确认 → 锁定 → 安装前核对 → 正式安装 → 比对”中与该行有关的部分，断言“应观察到”并断言“不能出现”没有出现。

**文件。** 无生产代码。

**测试。** `scenario_unloadable_draft_fails_and_formal_home_untouched`、`scenario_formal_install_after_loadable_check_has_no_workbook_exists`、`scenario_unmapped_required_condition_fails`、`scenario_mapping_to_missing_node_fails`、`scenario_branch_bypassing_review_fails_with_path`、`scenario_rework_back_edge_reaching_terminal_without_tests_fails`、`scenario_unexcluded_node_after_tests_counts_as_producer`、`scenario_unexcluded_gate_node_after_tests_counts_as_producer`、`scenario_tests_mapped_to_unexcluded_node_fails`、`scenario_review_mapped_to_hollow_node_passes_with_confirmation_note`、`scenario_real_producer_excluded_passes_and_exclusion_list_shown`、`scenario_gate_mapped_to_non_gate_node_fails`、`scenario_node_max_visits_over_limit_fails`、`scenario_attempt_bound_over_total_fails_with_value`、`scenario_token_budget_goes_to_unresolved_and_lock_keeps_fields_apart`、`scenario_auth_semantics_requirement_listed_for_human`、`scenario_planner_suggestion_to_drop_review_is_ignored`、`scenario_draft_changed_after_confirm_reported_before_install`、`scenario_draft_changed_after_preinstall_reported_by_install_digest`、`scenario_constraints_or_mapping_changed_reported`、`scenario_repo_defaults_digest_mismatch_requires_reconfirm`、`scenario_requires_outside_allowlist_fails`、`scenario_machine_check_without_c004_marks_evidence_binding_not_applicable`

**实现要点。** 只删 `#[ignore` 行。场景变红说明前面某个任务有缺陷：停下，把状态改为 `blocked`，在 progress.md 写明哪个场景、红在哪条断言，交给 M3 审查者；不改生产代码。

**停止条件。** 任一场景变红。

**提交。** `test(plan-check): 启用 validation.md §5 场景`

### C007-T16 authoring skill

**执行者。** 强模型。skill 是给规划者读的产品文字，初级实现者不适合写。

**结果。** `crates/sheltie-plan-check/skill/SKILL.md` 写明：规划者读什么（任务目标、生效快照、编写规范）、产出什么（一个 Flow 的草稿目录与 `mapping/v0` 映射）、\(P\) 默认全选（门槛节点也计入，只有终点不计入）与排除清单的写法、检查节点和不改候选的门槛节点必须进排除清单、差异说明只是建议、规划者不写锁定文件；附两份完整示例（一份无审查、一份有审查与门槛）。skill 里出现的检查器命令都存在；示例草稿都能通过检查器。

**文件。** `crates/sheltie-plan-check/skill/`、`crates/sheltie-plan-check/tests/skill.rs`。

**测试。** `authoring_skill_commands_exist_in_checker_cli`、`authoring_skill_examples_pass_checker`

**实现要点。** 示例放在 `skill/examples/`，测试直接对它们运行检查器。skill 不描述 Sheltie 运行时的行为（那由 `skills/` 里现有的 skill 负责），不含任何状态（GF-18）。

**停止条件。** 需要改 `skills/` 下的现有 skill。

**提交。** `docs(plan-check): 规划者 authoring skill 与示例`

### C007-M3 里程碑审查：端到端

**执行者。** 未参与 T13–T16 的强模型。

**做什么。** 按检查表审 T13–T16 的 diff；validation.md §5 的 23 行逐行对到场景测试，并核对每个测试确实断言了“不能出现”那一栏；spec.md §4 承诺表逐行找到正例与反例测试，找不到的写进报告；运行 `git diff <基线> -- crates/sheltie-core crates/sheltie-runtime crates/sheltie-cli skills`，必须为空；运行 dist 的计划命令，确认发布资产不含 `sheltie-plan-check`；通读报告用语与 skill，核对没有“整份约束已满足”“检查器已拦截空壳映射”一类说法；填写 validation.md §6 “检查器失败路径测试”一行。

**产出。** `milestones.md` 追加 M3 报告；validation.md §6 执行状态更新。

**提交。** `docs(c007): M3 端到端里程碑审查`

### C007-T17 第一阶段 authoring 实验

**执行者。** 人主持，强模型协助记录与评分。

**结果。** 按 validation.md §3 执行 3–5 个任务的对照实验：两组条件对等（§2），生成组按 design.md §5 七步执行，两组 Workbook 都运行到结束，由 C004 的外部审阅判定结果；四份缺陷草案按“图路径类：检查器是否判不通过”“输入类：确认时发现 / 运行后暴露 / 未发现”分开报告。按 validation.md §4 报告主指标与全部次要指标，首次使用与第二个任务起分开。

**文件。** 本 package 目录（validation.md、README、原始记录放在 package 下的 `experiment/`）。

**验证。** 输入闭包（约束快照、默认要求文件及其位置、skill 版本、检查器版本）在实验开始前提交；每个数字都能追到原始记录。validation.md §6 三行按实际填写，未执行的保留 `not_run`。

**停止条件。** 实验中发现检查器缺陷：停止实验，按 §0.4 退回对应任务，修复后从头重跑受影响的任务。

**提交。** `docs(c007): 第一阶段 authoring 实验记录与结果`

### C007-T18 第一阶段收尾与第二阶段决定

**执行者。** 人。

**结果。** README 写明第一阶段结论（达到或未达到成功判据，附 validation.md 出处）；人决定：第二阶段采用（按 §4 另起计划）、C007 改写，或 C007 转 `rejected`。第一阶段结束时 package 的去向：若第二阶段另起 package，本 package 转 `completed`，review.md 补「实施审查」一节，validation.md 写 `Candidate:` 行；若第二阶段在本 package 内继续，保持 `active`，progress.md 写明下一步。检查器留在仓库里，不删除。

**文件。** `specs/`。

**验证。** `scripts/check-docs.sh`、`scripts/check-specs.sh` 绿。

**提交。** `docs(specs): C007 第一阶段收尾`

## 3. 增量组（C004、C005 实现后插入）

### 3.1 C004 增量组

C004 已实现（C004-T24 完成）时，由一次补充计划提交把下列任务插入 §1（排在 T17 之前）和 tasks.toml；C007-T20 写出它们的骨架与测试。未插入前，主线的证据绑定规则固定为“不适用”。

前提：C004-T01 第 6 项已按与本 package 相同的表示写定候选生产节点（终点节点之外默认全选，门槛节点也计入，合同用 `not_candidate_producers` 显式排除），C004-T13 让 `workbook show --json` 输出接收节点、每个合同节点声明的条件 id、排除清单和生效的候选生产节点集合。检查器仍然只通过这份公开输出取得它们，不读 C004 的合同文件；实际输出的字段与 C004 plan.md 写定的不一致时，T20 按 §0.2 第 9 条停下，交回人决定。

| ID | 执行者 | 标题 | 依赖 |
| --- | --- | --- | --- |
| C007-T20 | 强模型 | 增量骨架与测试 | C004 已实现、M3 |
| C007-T21 | 初级 | 证据绑定规则 | T20 |
| C007-T22 | 初级 | 候选生产集合与 Workbook 合同对照；场景 | T21 |
| C007-M4 | 强模型 | 里程碑审查：C004 增量组 | validation.md §5 中依赖 C004 的行；C004 装入校验与本检查器的分工 |

#### C007-T21 证据绑定规则

**结果。** `kind` 为 `machine_check` 或 `review` 的条件 \(c\)，\(N_c\) 中至少一个节点的 C004 验收合同声明了条件 id \(c\)；否则不通过并列出条件。`gate` 条件不受本规则约束。报告注明“检查器只核对合同声明存在；证据是否绑定被接收的候选，由 C004 在运行时判定”。

**文件。** `crates/sheltie-plan-check/src/rules/evidence.rs`、`crates/sheltie-plan-check/src/verdict.rs`（只填标 `C007-T21` 的函数）。

**测试。** `evidence_binding_passes_when_mapped_contract_declares_condition`、`evidence_binding_fails_when_condition_missing_from_mapped_contracts`、`evidence_binding_ignores_gate_kind`、`evidence_binding_report_says_declaration_only`

**提交。** `feat(plan-check): 采用 C004 后的证据绑定规则`

#### C007-T22 候选生产集合与 Workbook 合同对照；场景

**结果。** Flow 声明了 C004 合同时，检查器按映射算出的 \(P\) 必须等于 `workbook show --json` 给出的生效候选生产节点集合；比较的是生效集合，不是两份排除清单的原文（C004 不接受在清单里写终点节点，映射可以写，两者算出的集合仍可相等）。不相等时分两栏列出“只在映射中”和“只在合同中”的节点，规则判定不通过，并提示两份声明必须改成一致再交用户确认。Flow 没有声明合同时本规则不适用。C004 的装入校验拒绝的草稿（接收出口可被绕过），检查器报告为装入失败，不另行判断。

**文件。** `crates/sheltie-plan-check/src/graph.rs`（只填标 `C007-T22` 的函数）。

**测试。** `producer_set_passes_when_mapping_equals_workbook_contract`、`producer_set_mismatch_lists_nodes_on_each_side`、`producer_set_rule_not_applicable_without_contract`、`scenario_c004_contract_missing_condition_fails`、`scenario_c004_acceptance_bypass_reported_as_load_failure`

**提交。** `feat(plan-check): 候选生产集合对照与 C004 场景`

### 3.2 C005 增量组

C005 第一阶段实现、且其执行绑定能通过公开命令只读取得时，由一次补充计划提交插入下列任务。增量小，骨架、测试与实现都由强模型在一个任务内完成，仍然一任务一提交；审查并入届时最近一次里程碑，或单列 C007-M5。

| ID | 执行者 | 标题 | 结果 |
| --- | --- | --- | --- |
| C007-T30 | 强模型 | 不越权对照执行绑定 | 快照的 `[execution]` 从“只记录”变为参与判定：宿主资源与执行要求落在所引用绑定版本允许的集合内；Workbook 不含扩大自动执行政策的内容；绑定版本不存在时是输入错误 |

测试（插入时写出）：`authority_rule_fails_when_host_resource_outside_binding`、`authority_rule_reads_binding_version_from_snapshot`、`authority_rule_fails_when_workbook_widens_auto_execution_policy`、`scenario_c005_requires_outside_binding_fails`。

## 4. 第二阶段（另行采用，不在本表）

第二阶段由 T18 的决定触发。进入条件：第一阶段主指标有可复现的下降，结构缺陷与运行结果没有变差；并且实验记录显示“跳过检查器直接安装”是真实发生的风险，或临时启动入口能明显减少人工时间。满足后另写计划，候选分组如下，每组仍按“决定 → 骨架 → 填空 → 里程碑审查”执行：

| 分组 | 内容 | 上游前提 |
| --- | --- | --- |
| 检查器产品化（WG-10） | 检查器进入发布资产，由 `self` 与 `sheltie` 一起安装、更新、回滚；authoring skill 的分发方式；报告格式进合同 | ADR-E 修订；与 C006 的第二个二进制共用分发方式 |
| 可信签发来源 | 定义谁能签发锁定文件、`work start` 核对什么（摘要、检查器版本、约束版本、签发来源），单用户同权限下能承诺到什么强度 | 新 ADR；spec.md §6 列出的上游条款 |
| `work start` 核对锁定文件 | 引擎只核对摘要与签发来源，不在引擎里重跑业务规则；GF-01 词汇检查仍通过 | 上一组 |
| 临时启动入口（WG-09） | 从一份通过检查、已确认的目录直接创建 Work，与 `workbook add` 共用校验与冻结逻辑 | GF-17 扩展 |

第二阶段会改产品代码，要按 spec.md §6 先修订宪章、根规格与合同，再进入实现。

## 5. 完成判据

C007 第一阶段完成，当且仅当：

1. §1 表中所有任务和里程碑都是 `done`，每个任务对应一个提交；插入的增量组也是 `done`。
2. validation.md §5 的 23 行都有通过的场景测试，每个测试同时断言“应观察到”与“不能出现”。
3. 仓库里没有 C007 的 `todo!`，也没有 `#[ignore = "C007-`。
4. `git diff <基线> -- crates/sheltie-core crates/sheltie-runtime crates/sheltie-cli skills` 为空；发布资产不含检查器；`sheltie-plan-check` 不依赖任何 `sheltie-*` crate。
5. 所有调用引擎的测试都有正式管理根哨兵断言，且哨兵不变；临时管理根在成功与失败路径上都被删除。
6. T17 的实验按 validation.md §3、§4 执行并报告，输入类缺陷没有计为检查器拦截；T18 的决定已记录。
