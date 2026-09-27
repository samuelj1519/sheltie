# C004 实施计划

状态：`draft`。本计划随提案一起准备，package 仍是 `proposed`，下表全部 `todo`。人采用、目录移入 `active/` 之后，本计划才是实施进度的唯一权威。代码在 C002 通过验收（C002-T17）之后才开始。

本计划借鉴 [v0.1.0 plan](../../../releases/v0.1.0/plan.md) 的做法：**强模型先搭骨架、写全部测试，初级实现者按任务逐一填空，每完成一组里程碑由强模型审查一次。** 实现者第一次接触本仓库，只需要读本文 §0、自己那张任务卡、卡上列出的文件和测试。产品语义以 [spec.md](spec.md) 为准，机制以 [design.md](design.md) 为准，失败路径以 [validation.md](validation.md) §6 为准；采用后以同步过的根规格与 `contracts/` 为准。

模块路径与签名写的是**暂定位置**。C002 会重排 Store、WorkLayout 与请求格式，C004-T02 的骨架作者按 C002 完成后的代码定死路径与签名；需要改路径时，在 T02 的提交里同时改本文任务卡和 [tasks.toml](tasks.toml)，并在 T02 卡的「记录」行写明原因。测试名是合同，T02 必须按任务卡的名字写出测试；确需改名的，同样在 T02 提交里改任务卡。

## 0. 执行方式

### 0.1 四层分工

| 层 | 谁 | 做什么 | 何时 |
| --- | --- | --- | --- |
| 决定 | 人 + 强模型 | 同步宪章、根规格、合同与 ADR；定下 design.md §9 中实施前必须回答的问题 | C004-T01，一次 |
| 骨架 | 强模型 | 全部类型、签名、文档注释、`todo!("C004-Tnn")` 函数体、全部测试（禁用）、夹具、样例、工具修补 | C004-T02，一次 |
| 填空 | 初级开发者或初级模型 | 启用本任务测试、填函数体、跑门禁、提交 | C004-T03 起，每任务一次 |
| 审查 | 编译器、clippy、测试、`scripts/check-task.sh` | 逐任务自动 | 每次提交 |
| 审查 | 未参与该组实现的强模型 | 按 [engineering.md §5](../../../engineering.md) 检查表审一组 diff、跑突变测试、对照 validation.md §6 | C004-M1 到 M4（VD-09 随版本实现时加 M5） |

真实宿主回归与发布由人完成（C004-T23、C004-T24）。

### 0.2 实现者规则

写给做填空任务的人或模型。每条都是硬规则，多数由 `scripts/check-task.sh` 机械核对。

1. **先按仓库入口开工，再聚焦任务。** 先读 `CONTEXT.md`、`specs/README.md`、`specs/changes/README.md`、本 active package 入口和 `specs/engineering.md`（AGENTS.md「开工入口」）；随后重点读本文 §0、自己的任务卡、卡上列出的文件与合同章节。遇到调用链超出卡片时继续追到真实使用者，不以“只读卡片”为由跳过。
2. **先看到红。** 运行 `scripts/task.sh C004-Tnn`（禁用的测试也会跑），确认本任务测试全红；然后删掉这些测试上的 `#[ignore = "C004-Tnn"]`。编译错误不算红。
3. **只填 `todo!("C004-Tnn")`。** 不改签名、类型、`pub` 可见性，不加依赖，不新建文件。同文件内新增私有辅助函数可以，但不得改变任何公开项的行为边界。文档注释就是函数要做的事。
4. **不改测试，不改快照。** 测试红了改实现。快照不一致改渲染代码，不运行 `cargo insta accept`。
5. **一次一个测试。** 让一个红测试变绿，再做下一个。
6. **绿了跑门禁。** `scripts/task.sh C004-Tnn` 全绿后，把 §1 表中本任务状态改为 `done`，运行 [engineering.md §2.3](../../../engineering.md) 的四条命令与 `scripts/check-task.sh C004-Tnn --staged`。
7. **一次提交。** 第一行用任务卡「提交」行；正文写为什么、怎么验证；末尾是 `Change: C004`、`Task: C004-Tnn`、`Agent: <名字>`。状态改为 `done` 放在同一提交。提交后再运行一次 `scripts/check-task.sh C004-Tnn`，并核对 `git show --stat`。
8. **不顺手改。** 别的任务的 `todo!`、测试、你觉得能优化的地方都不碰。
9. **卡住就停。** 出现下列任一情况时停手，不提交；把状态改为 `blocked`，在 [progress.md](progress.md) 写清原因：两个测试互相矛盾；不改签名或测试就做不到；需要新依赖；同一个测试改了五次还红；任务卡与合同说法不一；需要碰到后续任务的 `todo!`。
   **唯一例外：工具缺陷。** 如果 `scripts/` 或 `tasks.toml` 让一个按规则做的任务无法通过，可以修工具，但要单独提交，写 `Task: C004-T02`，并在提交说明中逐条写明原来错在哪、改成了什么。修复不得放松检查意图，不得改动测试、签名或合同。里程碑审查复核每一次工具改动。
10. **不猜。** 文档注释、测试和合同都没说的行为，不自己发明，按第 9 条停下。在注释里写下“这里由调用方保证”“解析不了当默认值”之类的约定，同样算发明。

实现者是强模型时，可以在一个会话里连做相邻任务，但仍是一任务一提交，每个任务各跑一遍 `task.sh` 与 `check-task.sh`。审查发现缺陷时，审查者只补测试（挂被审任务编号、禁用），实现者修代码。

### 0.3 测试与工具

- **命名与归属。** 测试名写“条件 → 行为”，不带任务编号。归属写在紧贴 `#[test]` 上方的一行注释里：

  ```rust
  // Task: C004-T06
  #[test]
  #[ignore = "C004-T06"]
  fn only_back_offered_when_any_required_condition_failed() { … }
  ```

  采用后 `scripts/check-tests.sh` 会解析本文每张卡的「**测试。**」行：行内每个反引号里的小写标识符都必须是真实存在、且归属本任务的测试。所以这一行只写测试名，说明放在全角括号（…）里。
- **基准 tag。** T02 提交后打 tag `c004-t02-skeleton`；审查者补测试后打 `c004-tNN-review`。实现者的测试零改动检查以最近一个 `c004-t*` tag 为基准。截至本文写作，`check-task.sh` 对 `Cnnn-Tnn` 默认取 README 的基线提交，这个默认基准会把整份骨架都算作“测试改动”。T02 必须先修好这一点（见 T02 卡）；如果另一个 package 已先修过，就直接复用。
- **命令。** `scripts/task.sh C004-Tnn` 只跑本任务测试；`scripts/check-tests.sh` 核对命名与任务卡；`scripts/check-task.sh C004-Tnn [base] [--staged]` 核对改动范围、占位清零、测试零改动、状态与 trailer；`scripts/mutants.sh <crate>` 供里程碑审查使用；`scripts/check-core-vocab.sh` 确保 core 与 runtime 不出现业务词汇（GF-01）。
- **测试分层。** 按 [engineering.md §3.2](../../../engineering.md) 放置：core 用模块内单元测试加 `insta` 快照；runtime 用临时 `SHELTIE_HOME` 的集成测试；验证器用临时 Git 仓库夹具；CLI 用 `assert_cmd` 端到端测试；崩溃用 fail-point 或杀子进程，放在 `crash.rs`。
- **独立 oracle。** 摘要、字节、路径集合与 Git 结果的期望值由测试直接写出，或用与被测代码无关的方式计算（例如测试里调用 `git` 命令、手写 SHA-256 向量）。测试 helper 不得复用被测函数来计算期望值。

### 0.4 复核与里程碑记录

- 每个填空任务在提交前由未参与该任务实现的复核者核对任务卡、合同、受影响调用链与本任务正反例；`check-task.sh`、门禁和测试负责机械检查。里程碑再做整组调用链、故障窗口与突变审查，不为每个小任务重复跑突变。
- 里程碑审查者没有参与本组实现。审查中发现缺陷时，审查者补禁用测试、打 `c004-tNN-review`，并把对应任务改回 `doing`；实现者修复后再提交一次，写被退回任务的 `Task:`，不写 `Task: C004-Mn`。审查者自己补测试的提交写 `Task: C004-Mn`。
- 缺陷涉及签名或数据结构、会让全仓夹具一起变红时，交回骨架作者修复并记录；未编写该修复的审查者再审修复 diff 与受影响调用链。边界清楚、能留成禁用测试的缺陷交给实现者。里程碑审查者不得审自己写的骨架或修复。
- 里程碑报告写入本 package 的 `milestones.md`（M1 创建此文件）。报告包含：检查表逐项结论、存活突变体的数量与处置、validation.md §6 的行与测试名对照、每一次工具改动的复核结论、退回清单，以及一节「流程教训」（每条写明证据与落点）。结论只用“通过 / 需修改 / 阻断”。validation.md 的执行状态表由里程碑审查者填写：写命令、原始输出路径、退出码和输入闭包，不只写 PASS。

### 0.5 成本

任务卡约 300–500 字；每个任务的骨架在 100–400 行，测试在 100–500 行；每个任务的输入上下文在 3k–10k token 之间。验证器任务要读 Git 夹具 helper，是上限一侧。

## A. 进入本计划前（不在任务表内）

以下事项在 package 仍为 `proposed` 时完成，结果记入 validation.md 与 README。任何一项未完成，人不采用，本计划不启动。

| 事项 | 谁 | 结果去向 |
| --- | --- | --- |
| C002 通过验收（至少 N04 真实 OS 身份） | C002 Owner | C002 release record |
| 阶段 1：3 任务摩擦探针；证据全部标为外部回填，不检验接收门控 | 人 + 强模型 | validation.md §3、§7 |
| 阶段 2：5–8 任务对照实验，按预写口径报告 | 人 + 强模型 | validation.md §4、§7 |
| ADR-B 审定（可先于实验） | 人 | README 采用条件 |
| ADR-A 审定；按阶段 1 两项计数决定 VD-09 是否随 v0.3 实现；决定是否修订 INV-1、INV-5、INV-6 | 人 | README 采用条件 |
| 决定 VD-11（保留样本）是否进入 v0.3 | 人 | README 采用条件 |

## 1. 任务表

状态只用 `todo | doing | blocked | done`。表格顺序就是执行顺序。VD-09、VD-11 的条件任务见 §3：由 C004-T01 按采用决定插入本表，未采用的不插入。

| ID | 状态 | 执行者 | 依赖 | 标题 | 结果 |
| --- | --- | --- | --- | --- | --- |
| C004-T01 | todo | 人 + 强模型 | 采用 | 上游同步与实施前决定 | 宪章、根规格、三份合同、ADR、CONTEXT 与本计划写定；README 基线改为 C002 完成后的提交 |
| C004-T02 | todo | 强模型 | T01 | 骨架、全部测试与工具 | 可编译骨架，全部测试存在且禁用；`scripts/task.sh C004-T03` 跑出红；tag `c004-t02-skeleton` |
| C004-T03 | todo | 初级 | T02 | core 验收合同解析 | 合同闭集字段解析；未知字段、外部回填来源、越界路径被拒 |
| C004-T04 | todo | 初级 | T03 | core 装入时的接收出口校验 | 候选生产节点默认全选、按排除清单扣除；删去接收边后入口与候选生产节点不能到达终点；`branch` 绕过被拒 |
| C004-T05 | todo | 初级 | T03 | core 证据有效性与条件状态 | 候选、验证输入、规则版本、来源四项核对；外部回填不满足条件 |
| C004-T06 | todo | 初级 | T05 | core 合同节点的合法出口 | VD-05 四行按优先级收窄 `next` |
| C004-T07 | todo | 初级 | T04、T06 | core 状态转换接线 | 无合法出口时拒绝提交；故障重试；`scope_violation` 阻塞 |
| C004-T08 | todo | 初级 | T07 | core 任务书、状态卡与 `next` 渲染 | 合同条件与证据指针出现在任务书、状态卡中 |
| C004-T09 | todo | 初级 | T05 | core 交付包渲染 | 用语与 Work 状态一一对应；未验证、回填、故障、局限逐项列出 |
| C004-M1 | todo | 强模型 | T03–T09 | 里程碑审查：core | diff、检查表、`scripts/mutants.sh sheltie-core`、GF-01 词汇与依赖树 |
| C004-T10 | todo | 初级 | M1 | runtime 证据表与合同冻结存储 | 证据只追加；全部字段往返；旧结构库被拒且不写 |
| C004-T11 | todo | 初级 | T10 | runtime 证据写入路径与执行身份 | 回填路径只能记外部回填；执行身份三栏落库 |
| C004-T12 | todo | 初级 | T11 | runtime 提交、失败与崩溃窗口 | 服务层按存储的证据推进；写入前被杀不留半份证据 |
| C004-T13 | todo | 初级 | T12 | cli 合同相关命令与输出 | `evidence add`、`work delivery`、`attempt fail` 故障类别、错误码 |
| C004-M2 | todo | 强模型 | T10–T13 | 里程碑审查：runtime 与 cli | diff、`scripts/mutants.sh sheltie-runtime`、崩溃窗口走查、写入路径审查 |
| C004-T14 | todo | 初级 | M2 | 验证器：固定输入与快照 | 完整提交标识；本地 clone 后核对检出；源仓库变化不影响 |
| C004-T15 | todo | 初级 | T14 | 验证器：额外文件与受保护路径 | 按实际字节记摘要；未授权覆盖为准备失败；越权路径列出 |
| C004-T16 | todo | 初级 | T14 | 验证器：执行检查 | 退出码分类；缺前提、超时分别记故障；超时终止进程组 |
| C004-T17 | todo | 初级 | T15、T16 | 验证器：记录与清理 | 先写证据再清理；清理失败记残留；来源固定为本地验证器 |
| C004-T18 | todo | 初级 | T17 | cli `verify` 命令组 | 真实 CLI 运行验证器写入证据；不接受外部结果 |
| C004-M3 | todo | 强模型 | T14–T18 | 里程碑审查：验证器 | 四个环节逐项核对 design.md §3；用户仓库 `.git` 不变；进程组清理 |
| C004-T19 | todo | 初级 | M3 | 场景：返工闭环 | 失败 → 返工 → 通过 → 接收出口 → 交付包，全程真实 CLI |
| C004-T20 | todo | 初级 | T19 | 场景：故障与停止 | 前提缺失、超时、只有未验证、越权、回填对照、门槛不开出口 |
| C004-T21 | todo | 初级 | T19 | 场景：装入校验与合同冻结 | 绕过路径与 `branch` 绕过装入失败；合同中途不可改；伪造 `USER` |
| C004-T22 | todo | 强模型 | T19–T21 | 样例 Workbook 与 skill | `examples/code-change` 与 skill 更新；机械检查通过 |
| C004-M4 | todo | 强模型 | T19–T22 | 里程碑审查：端到端 | validation.md §6 每行对到测试；spec.md §4 承诺表逐行正反例 |
| C004-T23 | todo | 人 | M4（及条件组的 M5） | 真实宿主回归 | 真实仓库、真实宿主走完一次含返工的 Work，逐命令记录 |
| C004-T24 | todo | 人 | T23 | 发布 | release record、CHANGELOG、package 转 `completed` |

## 2. 任务卡

每张卡给：结果（做完能观察到什么）、文件（只能改这些）、测试（要变绿的测试名）、实现要点（填空提示，不是设计）、停止条件、提交（信息第一行）。T01、T02、M 卡和人工任务卡另有格式。

### C004-T01 上游同步与实施前决定

**执行者。** 人审定，强模型起草。

**结果。** 按 spec.md §6 的上游条款表修订 `specs/constitution.md`（INV-1、INV-3、INV-5、INV-6、§5、§7）、`specs/spec.md`（§1、§2、GF-02、GF-06、GF-10、GF-11、GF-12、GF-14、§8）、`specs/roadmap.md` 和 `specs/contracts/{protocol,storage,workbook}.md`。ADR-A、ADR-B 移入 `specs/decisions/D-*.md` 并编号。`CONTEXT.md` 加入验收合同、证据、证据来源、接收出口、候选生产节点、验证器故障等词汇。README 的 `基线：` 改为 C002 完成后的提交。

**必须写定的决定。** 下列问题不回答，T02 就无法写出签名，所以必须在本任务中写进合同或 D 记录。括号里是本计划的建议：

1. 谁调用验证器（design.md §9 第 1 问；建议由协调者在验证节点的 Attempt 内运行 `sheltie verify run`，引擎不在 `attempt begin` 时自动触发）。
2. 合同的机器部分放在哪里（第 2 问；建议：条件、命令、受保护清单、额外文件放在 Workbook 冻结资源中，并由节点字段引用；仓库路径与基线提交作为 `work start` 输入，冻结进 Work）。
3. 交付包的入口（第 3 问；建议新增只读命令 `sheltie work delivery`，只做投影）。
4. 验证器的形态（第 5 问；建议新建 Cargo 包 `crates/sheltie-verify`，由 `sheltie-cli` 的 `verify` 命令组在进程内调用。这样“本地验证器取得”只能来自这条代码路径，公开的 `evidence add` 永远记“外部回填”，满足 design.md §4.4）。
5. 快照复制方式（第 9 问；首版只维护本地 clone 一种）。
6. 候选生产节点的表示与输出（第 11 问；与 C007 共用一种表示）。建议：默认全选，终点节点之外的每个节点都算候选生产节点，门槛节点也算：门槛节点先由 agent 执行、再由人批准（GF-12），它同样可能改动候选；若默认不算，接收出口之后一个设了 `gate = true` 的节点就能改了候选再走到终点，装入校验却悄悄通过。终点节点只能不算，否则“候选生产节点不能到达终点”恒不成立。合同用 `not_candidate_producers` 列出显式排除的节点，清单只能写 Flow 中存在的非终点节点，否则装入失败；接收出口之后不改候选的节点（例如只做发布批准的门槛节点）要写进这份清单。不用节点上的 `candidate = true` 逐个标注：多算生产节点只会让装入校验更严，漏标则只能以一条看得见的排除出现。`sheltie workbook show --json` 为每个声明了合同的 Flow 输出接收节点、每个合同节点声明的条件 id、排除清单和生效的候选生产节点集合；C007 检查器只通过这份输出取得它们（C007 plan.md §3.1）。
7. 同一条件、同一候选有多份有效证据时取哪一份；交付包的大小上限及超出时的截断规则；Store 是否升结构版本，旧库如何处理（沿用 C002 的“显式拒绝、不写旧库”原则）。
8. 新增错误码、`BlockedReason::ScopeViolation` 的线上名、`attempt fail` 故障类别的闭集线上名（`snapshot | prerequisite | timeout | interrupted | unknown`）。
9. VD-09、VD-11 是否进入本版。进入的，把 §3 对应的条件任务插入 §1 表与 tasks.toml；不进入的，在合同中写明装入时拒绝“构成必要条件”的审查作用（VD-09 推迟路径），并在 §3 标注“未采用”。

第 4 问（缓存）、第 6 问（处置代理）、第 10 问（越权后送回）按 spec.md §7 留作后续，不在本版实现。

**文件。** `specs/`、`CONTEXT.md`、本 package 目录。

**验证。** `scripts/check-docs.sh`、`scripts/check-specs.sh` 绿。人逐条确认上面 9 项都已写定且有出处。核对 spec.md §4 承诺表的每一行都能落到某份合同的某一节。

**停止条件。** 任何一项无法写定，就停在本任务，不进入 T02。人不修订 INV-5、INV-6 时，VD-09 只能走推迟路径。

**提交。** `docs(specs): 同步 C004 验收合同、证据来源与接收出口的上游条款`

### C004-T02 骨架、全部测试与工具

**执行者。** 强模型。这是唯一做代码设计的任务，之后的任务都是填空。

**结果。** 全仓可编译；新增类型、签名和文档注释齐全，函数体是 `todo!("C004-Tnn")`；本文每张卡列出的测试全部写好，挂 `// Task:` 注释并禁用；快照预写；夹具与样例就位；`tasks.toml` 按最终路径写定；CI 全绿。`scripts/task.sh C004-T03` 退出非零，并列出 T03 的测试名。

**文件（暂定，按 C002 完成后的代码定死）。**

- core：新模块 `crates/sheltie-core/src/contract/{mod,parse,load_check,evidence,exits}.rs`，放合同类型、装入校验、证据有效性和合法出口；修改 `work/{state,command,decide,next,render}.rs` 与 `error.rs`，加入 `BlockedReason::ScopeViolation`、故障类别、新错误码、`Effect`/`Reply` 的新字段；新增 `work/delivery.rs` 放交付包渲染。
- runtime：`store/{schema,commit,read}.rs` 加证据表与合同冻结存储；`service.rs` 加两条证据写入路径与执行身份；`crash.rs` 与 `failpoint.rs` 加新窗口。
- 新 Cargo 包 `crates/sheltie-verify`：`src/{lib,pin,snapshot,inputs,scope,exec,record,error}.rs`。它可以依赖 `sheltie-core` 的公开类型和 `sheltie-runtime` 的证据写入接口，不被 core 或 runtime 依赖。
- cli：`cli.rs` 的命令树（`verify run`、`evidence add`、`work delivery`、`attempt fail --fault`），`commands/{verify,evidence,work,attempt}.rs`、`error_map.rs`、`output.rs`。
- 测试：各卡的全部测试；验证器与 CLI 场景共用的 Git 仓库夹具 helper（在临时目录中 `git init`、写文件、提交、移动分支，返回完整提交标识），**完整实现，不留填空**。
- `examples/code-change/`：一份声明合同的代码变更 Workbook，结构为 `implement ⇄ verify → deliver`，验收命令是仓库内的一条 shell 命令。
- 工具：`scripts/check-task.sh` 对 `Cnnn-Tnn` 的默认基准改为最近一个匹配 `<cnnn 小写>-t[0-9]*` 的 tag，没有这样的 tag 时再退回 README 基线；`scripts/check-core-vocab.sh` 如有需要，把 `sheltie-verify` 排除在 core 词汇检查之外（它本来就允许出现 Git 词汇）。

**骨架的写法。** 类型逐字段对照 T01 写定的合同。每个公开项的文档注释，第一行写对应的合同章节，第二行写要返回的错误。复杂函数拆成私有函数，各自 `todo!` 并各自带注释，长度以初级实现者一次能填完为准。`decide` 对合同节点的额外判断单独成私有函数，不散落在已有分支里。夹具、固定时钟、`for_test` 构造器在骨架里写完。进程组终止如果需要平台调用，在骨架中选定无 `unsafe` 的做法（例如带 `process_group` 的 `std::process::Command`，再加一个现成依赖或调用系统 `kill`），并把依赖加进 `Cargo.toml` 与 `deny.toml` 的审查范围；实现者不能加依赖。

**验证。**

1. 四条门禁、`cargo deny check`、`scripts/check-docs.sh`、`scripts/check-core-vocab.sh`、`scripts/check-tests.sh` 全绿。
2. `cargo nextest run --all-features` 显示新增测试全部 ignored、零失败。
3. **正例试跑：** 临时填满 T03，运行 `scripts/check-task.sh C004-T03` 必须通过，然后撤掉填充。
4. **依赖顺序检查：** 对每个填空任务，只启用本任务的测试时，触发的 panic 都来自本任务的 `todo!("C004-Tnn")`；碰到其他任务的 `todo!`，就调整顺序或夹具。
5. **反例：** 改一行测试后，`check-task.sh` 必须失败。

**记录。** 相对本文暂定路径的每一处改动写在这里：原路径 → 新路径 → 原因。

**提交。** `chore(workspace): 搭起 C004 骨架、全部禁用测试与任务工具`。提交后打 tag `c004-t02-skeleton`。

### C004-T03 core 验收合同解析

**结果。** 合同的闭集字段可以解析成强类型：条件 id、说明、证据要求、允许来源、受保护路径 glob、额外文件、声明的环境输入、审查作用、分离模式。未知字段、重复 id、外部回填出现在允许来源里、路径越出工作区，都会被拒绝。

**文件。** `crates/sheltie-core/src/contract/{mod,parse}.rs`。

**测试。** `acceptance_contract_parses_minimal_machine_condition`、`acceptance_contract_rejects_unknown_field`、`acceptance_contract_rejects_external_backfill_in_allowed_sources`、`acceptance_contract_rejects_duplicate_condition_id`、`acceptance_contract_accepts_condition_count_at_limit_and_rejects_limit_plus_one`、`acceptance_contract_rejects_protected_glob_escaping_root`、`acceptance_contract_rejects_extra_file_target_outside_workspace`、`acceptance_contract_marks_condition_without_check_as_human_judgement`、`acceptance_contract_parses_low_assurance_separation_mode`

**实现要点。** 用 serde `deny_unknown_fields`，枚举用 `snake_case`，线上名以 T01 写定的 `contracts/workbook.md` 为准。条件 id 复用已有 kebab id 的校验方式。上限数字取合同常量，不在函数里写字面量。core 不知道候选是不是 commit，也不校验提交标识的格式，那是验证器的事。

**停止条件。** 合同没写的字段含义；需要读文件。

**提交。** `feat(core): 解析验收合同的闭集字段`

### C004-T04 core 装入时的接收出口校验

**结果。** 声明了合同的 Flow 在编译时多做一项检查：从图中删去接收出口这条边后，入口和每个候选生产节点都不能到达任一终点，否则返回错误并写出一条绕过路径。候选生产节点集合按 T01 写定的规则算出：终点节点之外的全部节点（门槛节点也计入），减去合同的 `not_candidate_producers`；排除清单写了不存在的节点或终点节点时拒绝，排除后集合为空时拒绝。算出集合的函数是公开的，T13 的 `workbook show` 直接调用它，不在 cli 重算。VD-09 推迟时，审查作用为“构成必要条件”的合同被拒绝。

**文件。** `crates/sheltie-core/src/contract/load_check.rs`、`crates/sheltie-core/src/flow/compile.rs`（只填其中标 `C004-T04` 的调用点）。

**测试。** `contract_flow_loads_when_terminal_reachable_only_through_acceptance_exit`、`contract_flow_rejects_producer_path_bypassing_acceptance_exit`、`contract_flow_rejects_branch_edge_from_acceptance_node_to_terminal`、`contract_flow_rejects_entry_reaching_terminal_without_acceptance_exit`、`contract_flow_accepts_back_edge_returning_to_producer`、`contract_flow_accepts_acceptance_node_not_excluded_from_producers`、`candidate_producers_default_to_every_non_terminal_node_including_gates`、`candidate_producers_drop_nodes_listed_in_exclusions`、`contract_flow_rejects_gate_node_after_acceptance_exit_reaching_terminal`、`contract_flow_accepts_excluded_gate_node_after_acceptance_exit`、`contract_flow_rejects_exclusion_naming_unknown_or_terminal_node`、`contract_flow_rejects_exclusions_leaving_no_candidate_producer`、`contract_flow_rejects_necessary_review_role_while_review_claims_deferred`、`contract_flow_accepts_advisory_review_role_while_review_claims_deferred`、`flow_without_contract_skips_acceptance_exit_check`、`contract_flow_error_names_bypass_path`

**实现要点。** 判断的对象是边，不是节点（design.md §2.1）：只删接收节点的那一条 `main` 边，节点本身和它的其他边都保留。可达性用 BFS，边的四种类型都要算。错误里的路径从起点写到终点，节点 id 按到达顺序排列，写法以测试断言为准。

**停止条件。** 需要判断节点“做了什么”；测试期望的路径与 BFS 的任意一条最短路径不一致，而文档注释又没有规定选哪一条。

**提交。** `feat(core): 装入时校验终点只能经接收出口到达`

### C004-T05 core 证据有效性与条件状态

**结果。** 给定冻结合同、当前候选和证据集，为每个条件算出 `satisfied | failed | not_verified | human_judgement` 之一。只有候选标识、验证输入摘要、规则版本三者都匹配，且来源在该条件的允许集合里的证据，才计入判断。外部回填的 `satisfied` 不满足必需条件。

**文件。** `crates/sheltie-core/src/contract/evidence.rs`。

**测试。** `evidence_satisfies_condition_when_candidate_input_rule_and_source_match`、`evidence_for_other_candidate_is_stale`、`evidence_with_changed_rule_version_is_stale`、`evidence_with_same_candidate_but_different_input_digest_is_stale`、`external_backfill_satisfied_does_not_satisfy_required_condition`、`not_verified_evidence_leaves_condition_unverified`、`missing_evidence_leaves_condition_unverified`、`human_judgement_condition_never_counts_as_verified`、`condition_states_follow_contract_order`、`multiple_valid_evidence_resolved_by_contract_rule`（规则由 T01 写定）

**实现要点。** 纯函数，不读时钟。“仍然有效”只按上面三项身份判断，不看时间。结果枚举与来源枚举都用穷尽 `match`，不写 `_ =>`。

**停止条件。** 需要解析证据里的自然语言或日志。

**提交。** `feat(core): 按候选、输入、规则与来源判定证据有效性`

### C004-T06 core 合同节点的合法出口

**结果。** 按 spec.md VD-05 的优先级收窄合同节点的出口。越权时没有出边；任一必需条件 `failed` 时只有 `back`；没有 `failed`，但有 `not_verified` 或缺证据时，没有可提交的出口，`next` 只给 `attempt fail` 与 `work cancel`；全部满足且来源合规时，才开放接收出口。非合同节点的 `next` 不变。

**文件。** `crates/sheltie-core/src/contract/exits.rs`、`crates/sheltie-core/src/work/next.rs`（只填标 `C004-T06` 的分支）。

**测试。** `acceptance_exit_offered_when_all_required_conditions_satisfied`、`only_back_offered_when_any_required_condition_failed`、`failed_and_not_verified_together_offer_only_back`、`not_verified_only_offers_attempt_fail_and_cancel`、`missing_evidence_offers_attempt_fail_and_cancel`、`scope_violation_takes_priority_over_satisfied_evidence`、`human_judgement_only_contract_offers_acceptance_exit`、`non_contract_node_next_is_unchanged`

**实现要点。** 先判越权，再判 `failed`，再判未验证或缺证据，最后判全部满足，顺序不能换。收窄只能从已有的合法边里去掉边，不能新增边，也不能替协调者选边（INV-1）。

**停止条件。** 测试要求的出口在 Flow 图中不存在。

**提交。** `feat(core): 合同节点按证据收窄合法出口`

### C004-T07 core 状态转换接线

**结果。** `decide` 接入合同规则：合同节点没有可提交的出口时，`SubmitAttempt` 被拒绝，且不产生 `blocked(no_legal_edge)`；`FailAttempt` 记故障类别，并消耗该节点的 `max_retries`；重试耗尽时 `blocked(retries_exhausted)`；越权时 `blocked(scope_violation)`，此后只能 `work cancel`；门槛批准不会打开接收出口；经接收出口到达终点后 Work `succeeded`。

**文件。** `crates/sheltie-core/src/work/decide.rs`（只填标 `C004-T07` 的私有函数）。

**测试。** `submit_on_contract_node_rejected_without_legal_exit`、`submit_with_failed_evidence_then_back_consumes_producer_visit`、`fail_attempt_on_contract_node_records_fault_category_and_consumes_retry`、`fault_retries_exhausted_blocks_with_retries_exhausted`、`scope_violation_submit_blocks_work_with_scope_violation`、`scope_violation_blocked_work_only_offers_cancel`、`gate_approval_on_contract_node_does_not_open_acceptance_exit`、`acceptance_exit_to_terminal_succeeds_work`、`evidence_bound_to_previous_candidate_does_not_open_exit_after_rework`

**实现要点。** 出口集合只来自 T06 的函数，这里不重复实现判断。计数沿用已有的 `max_visits`、`max_retries` 字段，不新增计数器。故障类别只记录并被引用，不据此分支。

**停止条件。** 需要新增计数器或状态才能通过测试。

**提交。** `feat(core): 合同节点的提交、故障重试与越权停止`

### C004-T08 core 任务书、状态卡与 `next` 渲染

**结果。** 合同节点的任务书列出条件 id 和对应的证据指针。经 `back` 回到实现节点时，任务书指向失败的证据。状态卡显示每个条件的状态和证据来源；越权时写出命中的路径。`next` 的 JSON 在只有未验证时只给 `attempt fail` 与 `work cancel`。

**文件。** `crates/sheltie-core/src/work/render.rs`（只填标 `C004-T08` 的函数）。

**测试。** `brief_for_contract_node_lists_condition_ids_and_evidence_pointers`、`brief_for_producer_after_back_points_to_failed_evidence`、`status_card_shows_condition_states_and_sources`、`status_card_for_scope_violation_names_hit_paths`、`next_json_for_not_verified_lists_fail_and_cancel_only`

**实现要点。** 按快照逐字节输出，格式以 T01 写定的 `contracts/protocol.md` 为准。只输出指针与闭集字段，不复制日志正文。

**停止条件。** 快照与协议不一致。改哪一边由里程碑审查者决定，实现者不改快照。

**提交。** `feat(core): 任务书与状态卡加入合同条件和证据指针`

### C004-T09 core 交付包渲染

**结果。** 交付包是一份有界的只读投影。Work `succeeded` 时写“候选已经过接收出口”；未结束、`blocked` 或 `cancelled` 时写“候选未接收”，并附停止原因；两种说法不会同时出现。它还逐条列出：需人判断的条件；外部回填的证据；验证器故障，与失败的检查分开列；“审查不属于接收条件”（VD-09 推迟时）；执行身份三栏；冻结范围之外的已知局限。

**文件。** `crates/sheltie-core/src/work/delivery.rs`。

**测试。** `delivery_for_succeeded_work_says_candidate_passed_acceptance_exit`、`delivery_for_blocked_work_says_candidate_not_accepted_with_reason`、`delivery_for_cancelled_work_says_candidate_not_accepted`、`delivery_lists_human_judgement_conditions_as_unverified`、`delivery_without_machine_conditions_says_no_machine_verified_conditions`、`delivery_marks_external_backfill_evidence`、`delivery_lists_verifier_faults_separately_from_failed_checks`、`delivery_states_review_is_not_acceptance_condition_when_deferred`、`delivery_shows_declared_observed_enforced_identity_columns`、`delivery_lists_frozen_scope_known_limitations`、`delivery_respects_size_limit_at_boundary`

**实现要点。** 用语只从 Work 状态推出，不另设“接收”状态（VD-07）。“全部已验证”只在没有需人判断的条件、没有回填、所有机器条件都满足时出现。大小上限与截断规则以 T01 写定的合同为准，测试覆盖恰好等于上限和上限加一两种情况。

**停止条件。** 需要读 Store 或文件（这里是纯渲染）。

**提交。** `feat(core): 渲染与 Work 状态一一对应的交付包`

### C004-M1 里程碑审查：core

**执行者。** 强模型，未参与 T03–T09。

**做什么。**

1. 读 `git diff c004-t02-skeleton..HEAD -- crates/sheltie-core`，按 engineering.md §5 检查表逐项打钩。
2. 运行 `scripts/mutants.sh sheltie-core`，逐个处置存活的突变体：测试漏了就补测试，是死代码就删。
3. 核对 `cargo tree -p sheltie-core` 中没有 I/O crate；运行 `scripts/check-core-vocab.sh`，确认 core 里没有 Git、commit、Cargo 等词。
4. grep 确认 T03–T09 的 `todo!`、`#[allow(unused_variables)]`、`#[ignore = "C004-T0x"]` 都已清零。
5. 复核 `git log c004-t02-skeleton..HEAD -- scripts/ specs/changes/*/C004*/tasks.toml` 中的每一次工具改动。
6. 对照 spec.md VD-05、VD-06、VD-07 与 design.md §2.1、§2.2，逐行找到正例和只改一个条件的反例。

**产出。** 创建 `milestones.md`，写入 M1 报告（格式见 §0.4）。

### C004-T10 runtime 证据表与合同冻结存储

**结果。** Store 新增证据记录和冻结合同的存储，证据只追加。所有字段都能往返：条件 id、候选标识、验证输入摘要与额外文件清单、规则版本、执行方式、结果、故障类别、起止时间、来源。遇到旧结构版本的库时拒绝打开，并且不写入。

**文件。** `crates/sheltie-runtime/src/store/{schema,commit,read}.rs`（只填标 `C004-T10` 的函数）。

**测试。** `evidence_insert_is_append_only`、`evidence_rows_round_trip_all_fields`、`evidence_source_column_rejects_unknown_value`、`evidence_read_returns_rows_in_sequence_order`、`frozen_contract_round_trips_byte_equal`、`store_rejects_pre_evidence_schema_without_writing`

**实现要点。** 建表语句是骨架给出的常量，不改。证据写入与 Attempt 状态变化必须在同一个事务中提交，沿用已有的 `commit()`。来源列存闭集线上名，读出后用穷尽 `match` 还原成枚举。

**停止条件。** 需要改表结构。

**提交。** `feat(runtime): 证据只追加存储与合同冻结`

### C004-T11 runtime 证据写入路径与执行身份

**结果。** 服务层有两条证据写入路径：外部回填路径不接收来源参数，永远记“外部回填”；验证器路径只在合同节点有运行中的 Attempt 时可用，只对 `sheltie-verify` 可见。`work start` 冻结合同和基线输入，冻结后不可改。实现与审查尝试的执行身份三栏落库：声明取协调者报告，观测为空，强制记实际实施的分离要求。

**文件。** `crates/sheltie-runtime/src/service.rs`（只填标 `C004-T11` 的函数）。

**测试。** `backfill_write_path_always_records_external_backfill`、`verifier_write_requires_running_attempt_on_contract_node`、`evidence_write_rejected_for_terminal_work`、`work_start_freezes_contract_and_baseline_inputs`、`frozen_contract_cannot_be_changed_after_start`、`attempt_identity_records_declared_and_leaves_observed_empty`

**实现要点。** 两条路径的可见性由骨架定死，不改 `pub`。服务层照常走“观察 → 决定 → 提交 → 效果”，由 core 做判断，runtime 只做存取（INV-2）。

**停止条件。** 需要让回填路径接收来源参数。

**提交。** `feat(runtime): 区分验证器与外部回填的证据写入路径`

### C004-T12 runtime 提交、失败与崩溃窗口

**结果。** 服务层提交合同节点时，读取已存储的证据并交给 core 判断。`attempt fail` 记故障类别，并消耗重试次数。越权时记录命中路径并阻塞。验证器在证据写入前被杀时，Attempt 仍为 `running`，没有半份证据；写入之后被杀时，证据只有一份。相同请求重放是幂等的。

**文件。** `crates/sheltie-runtime/src/service.rs`、`crates/sheltie-runtime/src/failpoint.rs`（只填标 `C004-T12` 的函数）。

**测试。** `service_submit_on_contract_node_uses_stored_evidence`、`service_fail_with_fault_category_consumes_verify_retry`、`service_records_scope_violation_paths_and_blocks`、`crash_before_evidence_commit_leaves_attempt_running_without_evidence`、`crash_after_evidence_commit_keeps_evidence_once`、`replayed_evidence_request_is_idempotent`

**实现要点。** 崩溃测试用子进程加 `--features failpoint`，fail-point 名字由骨架给出。幂等沿用 C002 的请求意图机制，不另建去重表。

**停止条件。** 崩溃后的 Store 需要人工修复才能读。

**提交。** `feat(runtime): 合同节点提交、故障重试与证据写入的崩溃窗口`

### C004-T13 cli 合同相关命令与输出

**结果。** `sheltie evidence add` 只能写外部回填，并且没有来源选项。`sheltie work delivery` 输出交付包，不改变任何状态。`attempt fail --fault <类别>` 记录故障类别。`workbook show --json` 为声明了合同的 Flow 增加接收节点、合同节点的条件 id、排除清单与生效的候选生产节点集合，集合取自 T04 的公开函数；没有合同的 Flow 输出不变。合同相关的拒绝有协议规定的错误码与退出码；`--json` 输出与协议一致。

**文件。** `crates/sheltie-cli/src/commands/{evidence,work,attempt,workbook}.rs`、`crates/sheltie-cli/src/{output,error_map}.rs`（只填标 `C004-T13` 的函数）。

**测试。** `cli_evidence_add_records_external_backfill`、`cli_evidence_add_has_no_source_option`、`cli_attempt_submit_rejected_on_contract_node_without_exit_json`、`cli_attempt_fail_records_fault_category`、`cli_next_json_for_scope_violation_offers_cancel_only`、`cli_work_delivery_prints_projection_without_changing_state`、`cli_workbook_show_json_lists_contract_conditions_and_candidate_producers`、`cli_workbook_show_json_unchanged_for_flow_without_contract`、`cli_error_codes_for_contract_rejections_match_protocol`

**实现要点。** 命令树在骨架的 `cli.rs` 中定死，这里只填处理函数。输出格式以 T01 写定的协议为准，不自己加字段。

**停止条件。** 协议与命令树不一致。

**提交。** `feat(cli): 证据回填、交付包与故障类别命令`

### C004-M2 里程碑审查：runtime 与 cli

**执行者。** 强模型，未参与 T10–T13。

**做什么。**

1. 读 `git diff <M1 结束提交>..HEAD -- crates/sheltie-runtime crates/sheltie-cli`，逐项核对检查表。
2. 运行 `scripts/mutants.sh sheltie-runtime`。
3. 人工走查证据写入的崩溃窗口：事务边界、fail-point 位置、恢复后 `next` 的形状。
4. 专项审查写入路径：沿调用链确认除 `sheltie-verify` 外，没有任何代码路径能写出来源为“本地验证器取得”的证据，包括测试 helper 是否只在 `cfg(test)` 下可用。
5. 核对旧库拒绝，以及根外哨兵文件不变。
6. 核对 INV-3：引擎自己的写入只落在 `~/.sheltie`。

**产出。** 在 `milestones.md` 中追加 M2 报告。

### C004-T14 验证器：固定输入与快照

**结果。** 验证器先把候选解析为完整提交标识；提交不存在时报告快照准备失败。之后在 `~/.sheltie/tmp` 下执行 `git clone --local --no-hardlinks --no-checkout`，再 detached checkout 到这个提交，并核对检出结果。源仓库在固定之后移动分支，不影响验证的对象。整个过程不在用户仓库的 `.git` 中新增条目。

**文件。** `crates/sheltie-verify/src/{pin,snapshot}.rs`。

**测试。** `pin_candidate_resolves_full_commit_id`、`pin_candidate_rejects_missing_commit`、`snapshot_clone_checks_out_pinned_commit_detached`、`snapshot_uses_pinned_commit_after_source_branch_moves`、`snapshot_missing_commit_reports_snapshot_fault_not_head`、`snapshot_does_not_add_entries_to_source_git_dir`、`snapshot_workspace_lives_under_sheltie_tmp`

**实现要点。** 调用系统 `git`，参数列表逐项传入，不拼 shell 字符串。固定后只使用完整标识，不再解析分支名或 `HEAD`（design.md §3.2 a）。测试的期望提交标识由夹具 helper 用 `git rev-parse` 独立取得。

**停止条件。** 需要写用户仓库；需要 `git worktree`。

**提交。** `feat(verify): 固定候选提交并在临时目录建立快照`

### C004-T15 验证器：额外文件与受保护路径

**结果。** 合同列出的额外文件被复制进快照，摘要按实际复制的字节计算。合同未允许时覆盖提交中的同名文件，记为快照准备失败；已允许的覆盖写进验证输入，但目标路径命中受保护 glob 时仍在复制前拒绝并记为越权。合同未列出的未跟踪文件不复制。相同提交、额外文件字节不同，得到不同的验证输入摘要。比较基线与候选之间改动的路径，并检查全部额外文件目标，命中受保护 glob 时列出命中的路径。

**文件。** `crates/sheltie-verify/src/{inputs,scope}.rs`。

**测试。** `extra_file_copied_and_digest_taken_from_delivered_bytes`、`extra_file_overwrite_without_permission_is_snapshot_fault`、`extra_file_overwrite_with_permission_recorded_in_input`、`extra_file_overwrite_of_protected_path_is_scope_violation_even_when_permitted`、`extra_file_new_target_matching_protected_glob_is_scope_violation`、`untracked_file_not_listed_is_not_copied`、`same_commit_different_extra_bytes_gives_different_input_digest`、`protected_glob_hit_between_baseline_and_candidate_reports_paths`、`change_outside_protected_list_is_not_scope_violation`

**实现要点。** 先按合同的 glob 语义检查额外文件目标与受保护清单；无交集才复制，摘要读快照中的实际字节。提交改动路径用 `git diff --name-only <基线>..<候选>` 取得，它不能代替额外文件目标检查。测试里的 SHA-256 期望值是手写常量。

**停止条件。** 合同没有定义 glob 语义（例如 `**` 是否跨目录）。

**提交。** `feat(verify): 额外文件按交付字节入账，受保护路径越权检测`

### C004-T16 验证器：执行检查

**结果。** 在快照目录中按参数列表运行冻结的命令，环境变量只包含合同声明的部分。退出码 0 记 `satisfied`，非零记 `failed`。程序不存在或无法启动时记 `not_verified`，故障类别为 `prerequisite`，并附缺失项。超时时终止整个进程组，记 `not_verified`，故障类别为 `timeout`。

**文件。** `crates/sheltie-verify/src/exec.rs`。

**测试。** `command_exit_zero_gives_satisfied`、`command_nonzero_exit_gives_failed`、`missing_program_gives_not_verified_prerequisite`、`timeout_kills_process_group_and_gives_not_verified_timeout`、`command_runs_with_declared_env_only`、`command_runs_in_snapshot_workspace_not_source_repo`

**实现要点。** 进程组的创建与终止用骨架选定的做法。超时测试让子命令再拉起一个后台子进程，然后断言它被终止；不能只断言父进程退出。macOS 上清理不干净时如实返回残留，不报成已清理。

**停止条件。** 需要 `unsafe`；需要新依赖。

**提交。** `feat(verify): 在快照中运行冻结命令并区分失败与故障`

### C004-T17 验证器：记录与清理

**结果。** 验证器先经由 runtime 的验证器路径写入证据，再清理临时目录。证据来源固定为“本地验证器取得”，并记录完整的验证输入。清理失败时，已写入的证据不变，另外记录残留路径。验证器进程被杀时不留半份证据。

**文件。** `crates/sheltie-verify/src/{record,lib}.rs`。

**测试。** `verifier_writes_evidence_before_cleanup`、`cleanup_failure_keeps_evidence_and_records_residue`、`verifier_evidence_source_is_local_verifier`、`verifier_run_records_full_verification_input`、`killed_verifier_leaves_no_partial_evidence`

**实现要点。** 顺序是固定输入 → 执行 → 写证据 → 清理，不能调换。清理失败测试用骨架提供的钩子，让某个路径不可删除。

**停止条件。** 需要从 runtime 以外的路径写 Store。

**提交。** `feat(verify): 先写证据再清理，记录清理残留`

### C004-T18 cli `verify` 命令组

**结果。** `sheltie verify run` 对当前合同节点上运行中的 Attempt 运行验证器，写入证据，并输出每个条件的结果。命令不接受任何外部结果输入。非合同节点被拒绝。

**文件。** `crates/sheltie-cli/src/commands/verify.rs`。

**测试。** `cli_verify_run_writes_local_verifier_evidence`、`cli_verify_run_rejects_attempt_not_on_contract_node`、`cli_verify_run_json_lists_condition_results`、`cli_verify_run_has_no_result_input_option`

**实现要点。** 只做参数到 `sheltie-verify` 调用的转换和输出。测试用夹具 helper 建临时 Git 仓库，二进制按 engineering.md §3.2 从 `cargo build --message-format=json` 的 `executable` 取得。

**停止条件。** 需要改命令树。

**提交。** `feat(cli): verify run 命令组`

### C004-M3 里程碑审查：验证器

**执行者。** 强模型，未参与 T14–T18。

**做什么。**

1. 读 `git diff <M2 结束提交>..HEAD -- crates/sheltie-verify crates/sheltie-cli`。
2. 对照 design.md §3.1 四个环节与 §3.2 a–g 逐条核对。
3. 运行 `scripts/mutants.sh sheltie-verify`。
4. 手工补跑三件事：源仓库在 pin 与 clone 之间出现新提交；快照目录被外部删除；超时子进程拉起守护进程。
5. 核对验证前后用户仓库 `.git` 的条目清单（用独立的 `find` 输出比对）。
6. 核对 `sheltie-verify` 没有被 core 或 runtime 依赖（`cargo tree -i`）。

**产出。** 在 `milestones.md` 中追加 M3 报告。

### C004-T19 场景：返工闭环

**结果。** 全程用真实 CLI 和真实临时 Git 仓库：第一次候选的验收命令失败，于是只有 `back`；实现节点从任务书读到失败证据的指针，提交新候选；验证通过后开放接收出口；Work `succeeded`；交付包写“候选已经过接收出口”，并附证据指针。返工次数用尽时 Work 阻塞。

**文件。** 无生产文件（场景测试只验证已有实现）。若场景暴露缺陷，停下来交给里程碑审查者。

**测试。** `scenario_rework_loop_accepts_fixed_candidate`、`scenario_rework_consumes_implement_visits_until_blocked`、`scenario_new_candidate_invalidates_old_evidence`、`scenario_delivery_after_success_lists_evidence_pointers`

**实现要点。** 本任务只删 `#[ignore`，让已写好的场景变绿。场景变红说明前面某个任务有缺陷：记下测试名和失败输出，把状态改为 `blocked`，交给审查者，不在本任务里改生产代码。

**停止条件。** 任一场景红。

**提交。** `test(cli): 启用返工闭环场景`

### C004-T20 场景：故障与停止

**结果。** 下列场景全部通过：缺前提时在验证节点重试；超时耗尽重试后 `blocked(retries_exhausted)`；只有未验证时提交被拒；`failed` 与 `not_verified` 同时出现时只有 `back`；越权后只能取消；同一候选的一对证据中，验证器证据开放出口，外部回填证据不开放；门槛批准不开放出口；验证前后用户仓库的 `.git` 不变。

**文件。** 无生产文件。

**测试。** `scenario_prerequisite_fault_retries_in_verify_node`、`scenario_timeout_fault_exhausts_retries_and_blocks`、`scenario_submit_rejected_with_only_not_verified`、`scenario_failed_plus_not_verified_offers_only_back`、`scenario_scope_violation_blocks_and_allows_only_cancel`、`scenario_backfill_satisfied_does_not_open_exit_but_verifier_does`、`scenario_gate_approval_does_not_open_exit_with_failed_condition`、`scenario_user_repo_git_dir_unchanged_after_verification`

**实现要点。** 同 T19。

**停止条件。** 任一场景红。

**提交。** `test(cli): 启用验证器故障与停止路径场景`

### C004-T21 场景：装入校验与合同冻结

**结果。** 存在绕过路径的 Workbook 装入失败，并指出路径；接收节点另有 `branch` 边直通终点时装入失败；VD-09 推迟时，审查作用为“构成必要条件”的合同装入失败；审查节点设了 `gate` 时，交付包写“审查不属于接收条件”；执行中改合同被拒；伪造 `USER` 不改变批准人。

**文件。** 无生产文件。

**测试。** `scenario_workbook_with_bypass_path_fails_to_load`、`scenario_workbook_with_branch_bypass_fails_to_load`、`scenario_necessary_review_role_fails_to_load_while_deferred`、`scenario_review_gate_delivery_says_review_not_acceptance_condition`、`scenario_contract_cannot_change_during_work`、`scenario_forged_user_env_does_not_change_approver`

**实现要点。** 同 T19。VD-09 随版本实现时，第三、四个测试由 T01 和 T02 改写为 VD-09 路径的对应场景。

**停止条件。** 任一场景红。

**提交。** `test(cli): 启用装入校验与合同冻结场景`

### C004-T22 样例 Workbook 与 skill

**执行者。** 强模型。skill 是给协调者读的说明书，措辞直接影响行为。

**结果。** `examples/code-change/` 能装入，并通过接收出口校验。`skills/sheltie/SKILL.md` 增加三件事：在验证节点运行 `sheltie verify run`；读失败证据，决定怎样返工；遇到 `blocked(scope_violation)` 或故障重试耗尽时停下找人，不自行回填证据。`scripts/check-skill.sh` 通过。

**文件。** `examples/code-change/`、`skills/sheltie/`、`crates/sheltie-core/tests/examples.rs`、`crates/sheltie-cli/tests/skill.rs`。

**测试。** `code_change_example_loads_and_passes_acceptance_exit_check`

**提交。** `feat(skill): 代码变更样例与验收闭环说明`

### C004-M4 里程碑审查：端到端

**执行者。** 强模型，未参与 T19–T22。

**做什么。**

1. 把 validation.md §6 的每一行对到一个通过的测试名，写成对照表。VD-09 推迟时，相关行标“不适用：VD-09 推迟”，并写出替代路径的测试名。
2. spec.md §4 承诺表逐行给出正例、反例和信任前提。
3. 核对 §5 保证范围声明：CLI 输出、skill 与交付包中没有出现“不能说”清单里的措辞（用 grep 列出结果）。
4. 全量运行 `cargo nextest run --all-features`，记录总数与耗时。
5. 在 validation.md 执行状态表中填写失败路径测试一行：命令、原始输出路径、退出码、输入闭包。

**产出。** 在 `milestones.md` 中追加 M4 报告。

### C004-T23 真实宿主回归

**执行者。** 人（真实宿主操作者）。实施者准备绑定 M4 候选的二进制、独立管理根、skill 交付物和逐命令记录模板。

**结果。** 在一个真实的个人仓库中，用 Claude Code 让协调者跑完 `code-change`：至少一次验收失败后返工，最后经接收出口结束，交付包可读。另外再跑一次缺前提的情况，确认协调者停下找人、没有回填证据。记录逐命令响应、人工操作、宿主 usage（拿不到就记缺失，不记 0）与耗时。

**文件。** 本 package 的 `validation.md`、`progress.md` 与原始记录目录。

**停止条件。** 协调者绕过 `next`，或回填证据冒充验证结果：按 engineering.md §7 路由到 skill 或合同，不用“模型没遵守”关闭问题。

**提交。** `docs(specs): 记录 C004 真实宿主回归`

### C004-T24 发布

**执行者。** 人（发布操作者）；需要用户授权。

**结果。** M4、T23（以及条件组的 M5）全部通过后，按 C002-T17 的方式发布：写 release record、CHANGELOG 与四平台资产核对；package 转为 `completed`；在 `review.md` 中新增「实施审查」一节，写最终结论行，满足 `scripts/check-specs.sh` 对 completed package 的要求。

**提交。** `chore(release): 发布 v0.3.0`（版本号以采用时的 README 为准）

## 3. 条件任务

T01 按采用决定把下列任务插入 §1 表（排在 M4 之后、T23 之前）和 tasks.toml。未采用的保留在本节，并标注“未采用”。编号从 30、40 开始，避免重排主线。

### VD-09 审查声明与阻断项（采用时插入）

| ID | 执行者 | 标题 | 测试 |
| --- | --- | --- | --- |
| C004-T30 | 初级 | core 审查记录与接收条件 | 见下 |
| C004-T31 | 初级 | runtime 审查执行、发现、处置只追加存储 | 见下 |
| C004-T32 | 初级 | cli 审查声明与处置命令 | 见下 |
| C004-T33 | 初级 | 场景：接替与处置 | 见下 |
| C004-M5 | 强模型 | 里程碑审查：审查声明 | validation.md §6 的 VD-09 各行、INV-5 修订后的边界 |

T02 同时写出这些任务的骨架与测试。VD-09 采用时，T21 中推迟路径的两个场景由 T02 改写。

#### C004-T30 core 审查记录与接收条件

**结果。** design.md §4.2 的接收条件成为纯函数：当前候选需要一次已完成、来源合规的审查执行，并且每条阻断发现都已处置。中断的审查不算完成。后来给出“通过”的审查不能清除更早的阻断发现。新候选会延续旧的阻断发现。严格分离模式下只有声明、没有观测时，条件不满足。带升级标记时 Work 停下找人。

**文件。** `crates/sheltie-core/src/contract/review.rs`、`crates/sheltie-core/src/contract/exits.rs`（只填标 `C004-T30` 的分支）。

**测试。** `acceptance_requires_completed_review_on_current_candidate`、`interrupted_review_leaves_review_condition_unmet`、`unresolved_blocking_finding_closes_acceptance_exit`、`disposed_blocking_finding_reopens_acceptance_exit`、`later_passing_review_does_not_clear_earlier_finding`、`new_candidate_carries_old_blocking_findings`、`strict_separation_without_observation_leaves_condition_unmet`、`escalation_flag_blocks_work_for_human`

**提交。** `feat(core): 审查声明与阻断发现参与接收条件`

#### C004-T31 runtime 审查记录存储

**结果。** 审查执行、发现和处置三类记录只追加。处置不删除原发现。v0.3 中处置人只能是当前 OS 用户。

**文件。** `crates/sheltie-runtime/src/store/{commit,read}.rs`、`crates/sheltie-runtime/src/service.rs`（只填标 `C004-T31` 的函数）。

**测试。** `review_findings_are_append_only`、`disposition_does_not_delete_finding`、`disposition_records_os_user_principal`

**提交。** `feat(runtime): 审查执行、发现与处置只追加存储`

#### C004-T32 cli 审查声明与处置命令

**结果。** 审查节点可以提交结构化声明；格式错误的声明被拒绝。用户可以处置发现。执行中不能修改审查作用。

**文件。** `crates/sheltie-cli/src/commands/`（骨架写定的审查命令文件）。

**测试。** `cli_review_declare_records_structured_claim`、`cli_review_declare_rejects_malformed_claim`、`cli_finding_dispose_records_disposition`、`cli_review_role_cannot_change_during_work`

**提交。** `feat(cli): 审查声明与发现处置命令`

#### C004-T33 场景：接替与处置

**文件。** 无生产文件。

**测试。** `scenario_review_replacement_keeps_findings`、`scenario_not_applicable_disposition_keeps_original_report`、`scenario_comment_only_candidate_keeps_findings`

**提交。** `test(cli): 启用审查接替与处置场景`

### VD-11 保留验收样本（采用时插入）

| ID | 执行者 | 标题 |
| --- | --- | --- |
| C004-T40 | 初级 | 验证器带入保留样本并标注暴露 |

#### C004-T40 验证器带入保留样本并标注暴露

**结果。** 保留样本由验证器带入快照，不出现在任务书中；失败反馈完整返回样本内容时，该样本标为已暴露，交付包如实显示。

**文件。** `crates/sheltie-verify/src/inputs.rs`、`crates/sheltie-core/src/work/{render,delivery}.rs`（只填标 `C004-T40` 的函数）。

**测试。** `holdout_samples_copied_into_snapshot_not_brief`、`holdout_sample_returned_in_feedback_marked_exposed`

**提交。** `feat(verify): 保留样本带入快照与暴露标注`

## 4. 完成判据

C004 完成，当且仅当：

1. §1 表中所有任务和里程碑都是 `done`，每个任务对应一个提交；采用的条件任务也在表中，并且是 `done`。
2. validation.md §6 每一行都有通过的测试，或写明“不适用”及替代测试；spec.md §4 承诺表逐行有正例、反例和信任前提。
3. 仓库里没有 C004 的 `todo!`，也没有 `#[ignore = "C004-`。
4. `scripts/check-core-vocab.sh` 通过：core 与 runtime 不含 Git、Cargo、CI 等词；`cargo tree -p sheltie-core` 不含 I/O crate。
5. 验证器前后用户仓库的 `.git` 不变；引擎与验证器自己的写入只在 `~/.sheltie`（INV-3）。
6. T23 有人工记录；对外用语与 spec.md §5 一致。
