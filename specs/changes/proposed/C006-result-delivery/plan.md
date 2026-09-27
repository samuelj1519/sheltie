# C006 实施计划

状态：`draft`。本计划随提案一起准备，package 仍是 `proposed`，下表全部 `todo`。人采用本 package、目录移入 `active/` 之后，本计划才是实施进度的唯一权威。代码在 C002 通过验收（C002-T17，至少恢复闭合与 T04 根内文件操作已落地）之后才开始。

C006 不以 C004、C005、C008 采用为前提（README「采用条件」）。主线任务（T01–T21）按“未采用 C004”的口径实施：交付物称「流程完成后的产物副本」，证据摘要只说明流程完成。依赖 C004 证据记录、C005 交接包的部分放在 §3、§4 的增量组，对应 change 实现后由人决定插入。主线里不为这些字段另造证据表或交接包。

做法借鉴 [v0.1.0 plan](../../../releases/v0.1.0/plan.md)：**强模型先搭骨架、写全部测试，初级实现者按任务逐一填空，每完成一组里程碑由强模型审查一次。** 实现者第一次接触本仓库，只需要读本文 §0、自己那张任务卡、卡上列出的文件和测试。产品语义以 [spec.md](spec.md) 为准，交付协议与目标根写入合同以 [design.md](design.md) §3 为准，失败路径以 [validation.md](validation.md) §3 为准；采用后以同步过的根规格、`architecture.md` 与 `contracts/` 为准。

本 package 与前几份计划有一处本质不同：它要在用户授权的目录里写文件。符号链接、检查后替换、硬链接、崩溃残留这些问题，初级实现者很难自己想全。所以本计划把**平台系统调用的薄封装**也划给骨架（强模型写完整，不留填空），初级实现者只填“怎样组合这些封装”；每一条竞态都由骨架提供确定性的注入点，测试不靠 sleep 碰运气。

模块路径、crate 名与签名写的是**暂定位置**。C002 会改动 Store、摘要编码和根内文件操作，C006-T02 的骨架作者按届时的代码定死路径与签名；需要改路径时，在 T02 的提交里同时改本文任务卡和 [tasks.toml](tasks.toml)，并在 T02 卡的「记录」行写明原因。测试名是合同，T02 必须按任务卡的名字写出测试；确需改名的，同样在 T02 提交里改任务卡。

## 0. 执行方式

### 0.1 四层分工

| 层 | 谁 | 做什么 | 何时 |
| --- | --- | --- | --- |
| 决定 | 人 + 强模型 | 同步根规格、宪章解释、architecture、AGENTS.md、CONTEXT.md 与合同；写定 T01 卡列出的实施前决定 | C006-T01，一次 |
| 骨架 | 强模型 | 两个新 crate、全部类型与签名、文档注释、`todo!("C006-Tnn")` 函数体、平台系统调用封装（完整实现）、竞态注入点、全部测试（禁用）、快照、工具修补 | C006-T02，一次 |
| 填空 | 初级开发者或初级模型 | 启用本任务测试、填函数体、跑门禁、提交 | C006-T03 起，每任务一次 |
| 审查 | 编译器、clippy、测试、`scripts/check-task.sh`、CI 的 Linux 与 macOS 两个平台 | 逐任务自动 | 每次提交 |
| 审查 | 未参与该组实现的强模型 | 按 [engineering.md §5](../../../engineering.md) 检查表审一组 diff、跑突变测试、对照 validation.md §3 | C006-M1 到 M4（增量组另加 M5） |

真实交付回归与发布由人完成（C006-T20、C006-T21）。

### 0.2 实现者规则

写给做填空任务的人或模型。每条都是硬规则，多数由 `scripts/check-task.sh` 机械核对。

1. **先按仓库入口开工，再聚焦任务。** 先读 `CONTEXT.md`、`specs/README.md`、`specs/changes/README.md`、本 active package 入口和 `specs/engineering.md`（AGENTS.md「开工入口」）；随后重点读本文 §0、自己的任务卡、卡上列出的文件与合同章节。遇到调用链超出卡片时继续追到真实使用者，不以“只读卡片”为由跳过。
2. **先看到红。** 运行 `scripts/task.sh C006-Tnn`（禁用的测试也会跑），确认本任务测试全红；然后删掉这些测试上的 `#[ignore = "C006-Tnn"]`。编译错误不算红。
3. **填空任务只填 `todo!("C006-Tnn")`。** 不改签名、类型、`pub` 可见性，不加依赖，不新建文件。同文件内新增私有辅助函数可以，但不得改变任何公开项的行为边界。文档注释就是函数要做的事。T17 是单独的跨文件自管理改造，由强模型按 T01 已审定的合同完成，不套用填空限制；改动仍受白名单、测试和独立复核约束。
4. **不改测试，不改快照。** 测试红了改实现。快照不一致改渲染代码，不运行 `cargo insta accept`。
5. **一次一个测试。** 让一个红测试变绿，再做下一个。
6. **绿了跑门禁。** `scripts/task.sh C006-Tnn` 全绿后，把 §1 表中本任务状态改为 `done`，运行 [engineering.md §2.3](../../../engineering.md) 的四条命令与 `scripts/check-task.sh C006-Tnn --staged`。
7. **一次提交。** 第一行用任务卡「提交」行；正文写为什么、怎么验证；末尾是 `Change: C006`、`Task: C006-Tnn`、`Agent: <名字>`。状态改为 `done` 放在同一提交。提交后再运行一次 `scripts/check-task.sh C006-Tnn`，并核对 `git show --stat`。
8. **不顺手改。** 别的任务的 `todo!`、测试、你觉得能优化的地方都不碰。
9. **卡住就停。** 出现下列任一情况时停手，不提交；把状态改为 `blocked`，在 [progress.md](progress.md) 写清原因：两个测试互相矛盾；不改签名或测试就做不到；需要新依赖；同一个测试改了五次还红；任务卡与合同说法不一；需要碰到后续任务的 `todo!`。
   **唯一例外：工具缺陷。** 如果 `scripts/` 或 `tasks.toml` 让一个按规则做的任务无法通过，可以修工具，但要单独提交，写 `Task: C006-T02`，并在提交说明中逐条写明原来错在哪、改成了什么。修复不得放松检查意图，不得改动测试、签名或合同。里程碑审查复核每一次工具改动。
10. **不猜。** 文档注释、测试和合同都没说的行为，不自己发明，按第 9 条停下。在注释里写下“这里由调用方保证”“解析不了当默认值”之类的约定，同样算发明。

实现者是强模型时，可以在一个会话里连做相邻任务，但仍是一任务一提交，每个任务各跑一遍 `task.sh` 与 `check-task.sh`。审查发现缺陷时，审查者只补测试（挂被审任务编号、禁用），实现者修代码。


### 0.3 测试与工具

- **命名与归属。** 测试名写“条件 → 行为”，不带任务编号。归属写在紧贴 `#[test]` 上方的一行注释里：`// Task: C006-Tnn`，再加 `#[ignore = "C006-Tnn"]`。采用后 `scripts/check-tests.sh` 会解析本文每张卡的「**测试。**」行：行内每个反引号里的小写标识符都必须是真实存在、且归属本任务的测试。所以这一行只写测试名，说明放在全角括号（…）里。
- **基准 tag。** T02 提交后打 tag `c006-t02-skeleton`；审查者补测试后打 `c006-tNN-review`。实现者的测试零改动检查以最近一个 `c006-t*` tag 为基准。截至本文写作，`check-task.sh` 对 `Cnnn-Tnn` 默认取 README 的基线提交，这个默认基准会把整份骨架都算作“测试改动”。T02 必须先修好这一点；如果别的 package（C004、C005、C007）已先修过，就直接复用。
- **命令。** `scripts/task.sh C006-Tnn` 只跑本任务测试；`scripts/check-tests.sh` 核对命名与任务卡；`scripts/check-task.sh C006-Tnn [base] [--staged]` 核对改动范围、占位清零、测试零改动、状态与 trailer；`scripts/mutants.sh <crate>` 供里程碑审查使用；`scripts/check-core-vocab.sh` 确保 core 与 runtime 不出现业务词汇（GF-01）；`scripts/check-skill.sh` 核对 skill。
- **文件系统。** 填空任务不写 `unsafe`，不直接调用 `libc`，不用 `std::fs` 的路径字符串接口打开目标根里的东西；只用骨架在 `sys` 模块里给出的封装（目录句柄、`openat` 式相对打开、独占创建、「不替换」改名、设备号与 inode）。需要的封装不存在，按 §0.2 第 9 条停下。
- **竞态注入。** “检查之后、改名之前”这类窗口由骨架提供的具名注入点触发（测试打开注入点，注入点里执行替换目录、创建同名文件或改写旧文件）。测试不用 sleep，不用多线程赌时序。注入点只在测试构建中存在，发布构建里没有。
- **根外哨兵。** 所有“拒绝写到根外”的测试都在目标根外放哨兵文件，断言测试前后哨兵的字节、权限与修改时间不变。helper 由骨架完整实现。
- **两个平台。** CI 在 Linux 与 macOS 上各跑一遍全部测试。与平台有关的测试不用 `#[cfg]` 静默跳过：平台没有等价的原子“不替换”操作时，测试断言“拒绝提供导出”。
- **只读 `$HOME`。** runtime 与 cli 的交付测试在只读的假 `$HOME` 下运行（`SHELTIE_HOME` 指向其下可写的临时目录），证明 `sheltie` 除管理根外不写任何地方。helper 由骨架完整实现。
- **测试分层。** 按 [engineering.md §3.2](../../../engineering.md) 放置：core 用模块内单元测试加 `insta` 快照；两个新 crate 用 crate 内集成测试；runtime 用临时 `SHELTIE_HOME` 的集成测试；CLI 与端到端用 `assert_cmd` 驱动两个真实二进制；崩溃用 fail-point 加子进程。
- **独立 oracle。** 摘要、状态和 JSON 字段的期望值由测试直接写出，或由测试自己用标准库重新计算。测试 helper 不得复用被测函数来计算期望值。

### 0.4 复核与里程碑记录

- 每个填空任务在提交前由未参与该任务实现的复核者核对任务卡、合同、受影响调用链与本任务正反例；`check-task.sh`、门禁和测试负责机械检查。里程碑再做整组调用链、故障窗口与突变审查，不为每个小任务重复跑突变。
- 里程碑审查者没有参与本组实现。审查中发现缺陷时，审查者补禁用测试、打 `c006-tNN-review`，并把对应任务改回 `doing`；实现者修复后再提交一次，写被退回任务的 `Task:`，不写 `Task: C006-Mn`。审查者自己补测试的提交写 `Task: C006-Mn`。
- 缺陷涉及签名、数据结构或 `sys` 封装、会让全仓夹具一起变红时，交回骨架作者修复并记录；未编写该修复的审查者再审修复 diff 与受影响调用链。里程碑审查者不得审自己写的骨架或修复。
- 里程碑报告写入本 package 的 `milestones.md`（M1 创建此文件；`review.md` 是提案讨论记录，不混写）。报告包含：检查表逐项结论、存活突变体的数量与处置、validation.md §3 的行与测试名对照、每一次工具改动的复核结论、退回清单，以及一节「流程教训」（每条写明证据与落点）。结论只用“通过 / 需修改 / 阻断”。validation.md 的执行状态表由里程碑审查者填写：写命令、原始输出路径、退出码、平台和输入闭包，不只写 PASS。

### 0.5 成本

任务卡约 300–500 字；每个任务的骨架在 100–300 行，测试在 100–400 行；每个任务的输入上下文在 3k–8k token 之间。T02 比前几份计划的骨架重：两个新 crate、平台封装和竞态注入点都在里面，预计是全部代码量的三分之一。这笔钱花在强模型上是值得的，因为目标根写入合同一旦写错，后果是改坏用户的文件。

## A. 进入本计划前（不在任务表内）

| 事项 | 谁 | 结果去向 |
| --- | --- | --- |
| C002 通过验收（恢复闭合、摘要编码、T04 根内文件操作） | C002 Owner | C002 release record |
| validation.md §2 探针：外部脚本实现交付协议，三种目标位置各一次，先测基线 | 人 + 强模型 | validation.md、README |
| 人决定 GF-14 的终态例外，以及管理根之外写入边界的上游修订方案 | 人 | README 采用条件 |
| 人审定 ADR-D（建议 E2b：独立 crate 与独立二进制） | 人 | README 采用条件；T01 移入 `specs/decisions/` |

## 1. 任务表

状态只用 `todo | doing | blocked | done`。表格顺序就是执行顺序。§3、§4 的增量组在 C004、C005 实现后由人决定插入本表，排在 T20 之前。

| ID | 状态 | 执行者 | 依赖 | 标题 | 结果 |
| --- | --- | --- | --- | --- | --- |
| C006-T01 | todo | 人 + 强模型 | 采用 | 上游同步与实施前决定 | GF-14、GF-27、architecture、AGENTS.md、CONTEXT.md、storage、protocol 写定；ADR-D 编号；命令、默认位置、平台清单与共享 crate 写定 |
| C006-T02 | todo | 强模型 | T01 | 骨架、全部测试与工具 | 两个新 crate 与各层骨架可编译；平台封装与注入点完整；全部测试存在且禁用；tag `c006-t02-skeleton` |
| C006-T03 | todo | 初级 | T02 | core 交付条件与交付请求 | 只在 Work `succeeded` 时生成；导出清单路径闭集校验；请求绑定结果根版本 |
| C006-T04 | todo | 初级 | T03 | core 终态例外与送达状态 | 终态只追加交付记录；五种送达状态由记录派生；请求不可变 |
| C006-T05 | todo | 初级 | T04 | core 回执核对 | 引擎读回摘要、期望摘要、组件陈述三方比对；重复回执拒绝 |
| C006-T06 | todo | 初级 | T04 | core 送达观测与状态卡用语 | 四种观测只追加；原件缺失为 `STORE_CORRUPT`；未采用 C004 的用语 |
| C006-T07 | todo | 初级 | T04 | core 来源清单与证据摘要投影 | 可重生成；只用根绑定加相对路径；只给指针 |
| C006-M1 | todo | 强模型 | T03–T07 | 里程碑审查：core | 送达状态与回执核对逐条正反例；`scripts/mutants.sh sheltie-core`；GF-01 词汇 |
| C006-T08 | todo | 初级 | M1 | 根文件操作：重叠、根身份与逐段打开 | 与管理根重叠时拒绝；根身份不符拒绝；逐段不跟随链接 |
| C006-T09 | todo | 初级 | T08 | 根文件操作：临时文件与原子发布 | 用「不替换」改名；目标在检查后出现也不能覆盖 |
| C006-T10 | todo | 初级 | T08 | 根文件操作：不跟随链接的读回摘要 | 缺失、不可访问、根身份不符分开报告；摘要编码与 core 一致 |
| C006-M2 | todo | 强模型 | T08–T10 | 里程碑审查：根文件操作 | Linux 与 macOS 两个平台逐条验证；哨兵；突变测试；`sys` 封装复核 |
| C006-T11 | todo | 初级 | M2 | runtime 交付记录存储 | 根绑定、请求、回执、观测、组件报告落库；请求行不可更新；旧库被拒 |
| C006-T12 | todo | 初级 | T11 | runtime 服务、读回与崩溃窗口 | 授权、请求、回执读回、观测各一个事务；结果根只读；只读 `$HOME` 下通过 |
| C006-T13 | todo | 初级 | T12 | cli 交付命令与输出 | `work start` 授权参数；`delivery` 命令组；错误码 |
| C006-M3 | todo | 强模型 | T11–T13 | 里程碑审查：runtime 与 cli | `scripts/mutants.sh sheltie-runtime`；事务边界、只读访问与 `confine()` 走查 |
| C006-T14 | todo | 初级 | M3 | 导出组件：读请求与核对源文件 | 只经 `sheltie` 公开命令读请求；只读打开源文件并核对摘要 |
| C006-T15 | todo | 初级 | T14 | 导出组件：写入与恢复 | 按 DL-06 恢复表逐文件处理；来源清单最后写；清单外文件不动 |
| C006-T16 | todo | 初级 | T15 | 导出组件：读回与回执提交 | 逐个读回后经 `sheltie` 提交回执或报告冲突、失败；从不写管理根 |
| C006-T17 | todo | 强模型 | T16 | `self` 分发第二个二进制 | 旧布局安全迁移；两个二进制同版本安装、更新与回滚；以单个版本指针切换 |
| C006-T18 | todo | 初级 | T17 | 场景：交付、恢复与拒绝 | 两个真实二进制走完 validation.md §3 各行 |
| C006-T19 | todo | 强模型 | T18 | skill 交付一节 | 协调者何时调用导出组件、怎样读送达状态；`scripts/check-skill.sh` 通过 |
| C006-M4 | todo | 强模型 | T14–T19 | 里程碑审查：导出组件与端到端 | validation.md §3 逐行对到测试；spec.md §3 承诺逐行正反例；两个平台 |
| C006-T20 | todo | 人 | M4（及插入的增量组里程碑） | 真实交付回归 | 三种目标位置各一次真实 Work；与探针基线对照 |
| C006-T21 | todo | 人 | T20 | 发布 | release record、CHANGELOG、两个二进制的发布资产、package 转 `completed` |

## 2. 任务卡

每张卡给：结果、文件（只能改这些）、测试（要变绿的测试名）、实现要点（填空提示，不是设计）、停止条件、提交（信息第一行）。

### C006-T01 上游同步与实施前决定

**执行者。** 人审定，强模型起草。

**结果。** 按 spec.md §5 修订 `specs/spec.md`（GF-14 终态例外、GF-27）、`specs/architecture.md`（§1 crate 清单、§6 INV-3 实现行、`confine()` 条款、「不拆独立安装器」一节）、`AGENTS.md`（项目一句话、硬边界）、`CONTEXT.md`（结果根、导出组件、交付请求、送达回执、送达观测）、`specs/contracts/{storage,protocol}.md`、`specs/roadmap.md`；ADR-D 移入 `specs/decisions/` 并编号。C004 或 C005 已采用时，GF-14 的措辞与它们合并（终态只追加交付记录；`resource_wait` 可恢复；`scope_violation`、`retries_exhausted`、`continuity_exhausted` 只能取消）。README 的 `基线：` 改为实施起点提交。

**必须写定的决定。** 括号里是本计划的建议：

1. 命令与参数（建议：`work start --result-root <位置>` 授权；`delivery retarget <work> <位置>` 形成新绑定版本；`delivery request <work>` 生成请求；`delivery cancel <request>` 取消未送达的请求；`delivery show <request> --json` 只读，供导出组件读取；`delivery receipt <request>` 从标准输入读组件陈述；`delivery report <request> --outcome conflict|failed` 记组件报告；`delivery check <work>` 只读核对目标并追加观测；`delivery claim <work>` 记协调者自报；`delivery status <work>` 只读）。首版没有覆盖参数；已有不同字节的目标文件一律冲突。
2. 交付请求由显式命令生成，还是在 Work 成功的事务里自动生成（建议显式命令：不改动现有成功事务；副本丢失后重新导出、取消后重新请求走同一路径）。
3. 默认交付位置（design.md §5 第 1 问；建议管理根之外的用户级目录下按 Work 标识分子目录，不默认放进项目目录，避开 DL-10 的 `.gitignore` 问题）；授权时若不给 `--result-root` 是否允许创建 Work（建议允许，取默认位置并在状态卡写明）。
4. 证据摘要是否包含日志片段（第 2 问；建议只给指针）；来源清单是否作为 `export/v1`（第 3 问；建议先用 package 内的 `provenance/v0`，不占用 `export/v1`）。同时写定两份元数据的有界字节格式、每请求独立的相对路径和大小上限：请求创建时生成完整字节并存入不可变请求，`delivery show` 经公开协议交给导出组件；运行时不得按当前时钟重生成。
5. 导出二进制随 `self` 与 `sheltie` 一起分发与回滚：两者同一版本，缺一个就拒绝安装。必须先写定从现行 `bin/sheltie` 布局迁到版本目录与单个原子指针的过程、稳定入口如何解析该指针、切换前后崩溃恢复和旧版本回滚；导出组件调用 `sheltie` 时必须选同一版本目录中的同伴，不能在更新窗口重新从 `PATH` 找到另一版本。不可用逐个文件改名冒充“两者一步切换”。
6. 受限文件操作的归属（第 5 问；建议新建无业务词汇的 crate `sheltie-rootfs`，runtime 读回回执与导出组件写入都用它，保证“以与写入相同的方式读回”；C002 T04 的管理根实现是否迁入另行决定，本 package 不迁）。
7. 首批支持的平台（第 6 问；建议 Linux 与 macOS；任何平台没有等价的原子“不替换”操作时不提供导出；Windows 不提供导出命令）。
8. 导出组件对 `sheltie-core` 的依赖（建议只依赖 core 的摘要编码与导出清单路径校验这两个纯函数模块，不依赖 `sheltie-runtime`；用测试核对依赖图）。
9. 组件报告的冲突与失败怎样记（建议作为“导出组件陈述”单独保存，写入方式标「导出组件」；送达状态由它派生为送达冲突或送达失败，但不当作系统核对过的事实）。
10. 修改结果根（retarget）在终态 Work 上是否允许（建议允许，属于只追加的交付记录）。
11. Store 是否升结构版本、旧库怎么处理（沿用 C002 的“显式拒绝、不写旧库”）。
12. `delivery check` 的观测是否需要协调者触发，还是 `work status` 时顺带只读核对（建议只由 `delivery check` 触发，`work status` 不访问结果根）。

**文件。** `specs/`、`AGENTS.md`、`CONTEXT.md`、本 package 目录。

**验证。** `scripts/check-docs.sh`、`scripts/check-specs.sh` 绿。人逐条确认上面 12 项都已写定且有出处；design.md §3.1 的五步、§3.2 的五个故障窗口在 protocol 或 storage 合同里都能找到对应的命令、字段与错误码。

**停止条件。** 任一项无法写定；或宪章 INV-3 的解释需要改条文（spec.md §5 写的是条文不变）。

**提交。** `docs(specs): 同步 C006 结果根、交付请求与终态例外的上游条款`

### C006-T02 骨架、全部测试与工具

**执行者。** 强模型。这里写定交付协议的主要接口与测试；T17 的自管理布局改造由另一张强模型任务卡负责。

**结果。** 全仓可编译；新增类型、签名和文档注释齐全，函数体是 `todo!("C006-Tnn")`；平台封装、竞态注入点与测试 helper 完整实现；本文每张卡列出的测试全部写好，挂 `// Task:` 注释并禁用；快照预写；CI 在 Linux 与 macOS 上全绿。`scripts/task.sh C006-T03` 退出非零，并列出 T03 的测试名。

**文件（暂定，按实施起点代码定死）。**

- core：新增 `delivery/` 模块：`request.rs`（交付条件与导出清单）、`state.rs`（送达状态闭集与派生、交付命令、终态例外）、`receipt.rs`（回执核对）、`observe.rs`（观测闭集）、`render.rs`（状态卡交付一节）、`provenance.rs`（来源清单与证据摘要）；`error.rs` 加新错误码。
- 新 crate `crates/sheltie-rootfs`：`sys.rs`（Linux 与 macOS 的系统调用薄封装，**完整实现**）、`overlap.rs`、`walk.rs`、`write.rs`、`read.rs`、`inject.rs`（测试构建的竞态注入点，**完整实现**）。
- 新 crate `crates/sheltie-deliver`（二进制 `sheltie-deliver`）：`main.rs`、`cli.rs`（命令行，**完整实现**）、`sheltie.rs`（调用 `sheltie` 公开命令的子进程封装，**完整实现**）、`request.rs`、`source.rs`、`export.rs`、`recover.rs`、`receipt.rs`。
- runtime：`store/{schema,commit,read}.rs` 加交付表；`service.rs` 加授权、请求、回执、观测的服务入口；`failpoint.rs` 加回执事务前后的 fail-point。
- cli：`cli.rs` 命令树；`commands/{work,delivery}.rs`、`output.rs`、`error_map.rs`。
- 测试与 helper：各卡的全部测试；根外哨兵、只读假 `$HOME`、在测试里定位两个二进制、崩溃子进程的 helper，**完整实现，不留填空**。
- 工具：`scripts/check-task.sh` 的 tag 基准（如尚未修）；`Cargo.toml` 的 workspace 成员与 dist 配置；`deny.toml` 按新依赖更新；CI 两个平台都跑新 crate。

**骨架的写法。** 类型逐字段对照 T01 写定的合同。每个公开项的文档注释，第一行写对应的合同章节（DL-nn 或 design.md §3.x 的步号），第二行写要返回的错误。`sys` 封装每个函数只做一次系统调用加错误翻译，不含判断；判断全部留在上层函数里给实现者填。导出组件的每一步（读请求、核对源、逐文件处理、读回、提交）各是一个函数，恢复表的三行各是一个分支。

**验证。**

1. 四条门禁、`cargo deny check`、`scripts/check-docs.sh`、`scripts/check-core-vocab.sh`、`scripts/check-tests.sh`、`scripts/check-skill.sh` 全绿；CI 两个平台全绿。
2. `cargo nextest run --all-features` 显示新增测试全部 ignored、零失败。
3. **正例试跑：** 临时填满 T03，运行 `scripts/check-task.sh C006-T03` 必须通过，然后撤掉填充。
4. **依赖顺序检查：** 对每个填空任务，只启用本任务的测试时，触发的 panic 都来自本任务的 `todo!("C006-Tnn")`。
5. **反例：** 改一行测试后，`check-task.sh` 必须失败。
6. **注入点试跑：** 每个注入点至少被一个测试打开，并在骨架阶段用一个临时实现证明它确实能让“检查后替换”发生在检查与改名之间。
7. **依赖图：** `cargo tree -p sheltie-deliver` 不含 `sheltie-runtime`；`sheltie-rootfs` 不依赖任何 sheltie crate。

**记录。** 相对本文暂定路径的每一处改动写在这里：原路径 → 新路径 → 原因。

**提交。** `chore(workspace): 搭起 C006 骨架、平台封装、全部禁用测试与任务工具`。提交后打 tag `c006-t02-skeleton`。

### C006-T03 core 交付条件与交付请求

**结果。** 只在 Work `succeeded` 时允许生成交付请求；Work 活动（包括终点 Attempt 已成功但门槛未批准）、`blocked`、`cancelled` 都拒绝。导出清单每项的目标相对路径拒绝绝对路径、`..`、空段、平台保留名和重复目标。core 接收已生成的有界元数据字节，校验摘要与路径后组装不可变请求；元数据的实际生成由 T07 提供，T12 才把两者接到请求命令。请求记录每项产物或元数据的期望摘要、当前有效的结果根绑定版本和请求标识；证据摘要与来源清单使用由请求标识确定的独立路径，来源清单不包含自身摘要；首版没有覆盖策略字段。

**文件。** `crates/sheltie-core/src/delivery/request.rs`。

**测试。** `request_allowed_when_work_succeeded`、`request_rejected_when_work_active`、`request_rejected_when_terminal_attempt_succeeded_but_gate_unapproved`、`request_rejected_when_work_blocked_or_cancelled`、`manifest_rejects_absolute_path`、`manifest_rejects_dotdot_segment`、`manifest_rejects_empty_segment`、`manifest_rejects_platform_reserved_name`、`manifest_rejects_duplicate_target_path`、`request_binds_current_result_root_version`、`request_records_expected_digest_per_entry`、`request_metadata_paths_unique_per_request`、`request_has_no_overwrite_option`

**实现要点。** 交付条件只看 Work 状态，不看 Attempt 或门槛的细节（GF-14 已经保证 `succeeded` 蕴含无未批准门槛）。路径校验逐段做，不用字符串包含判断。保留名清单是骨架给的常量。

**停止条件。** 需要读文件系统判断路径是否存在（core 不做 I/O）。

**提交。** `feat(core): 交付条件与不可变的交付请求`

### C006-T04 core 终态例外与送达状态

**结果。** 交付命令（请求、取消请求、回执、组件报告、观测、自报、修改结果根）只作用于交付记录：在终态 Work 上可以追加，执行结果不变；其他写操作在终态 Work 上仍被拒绝。请求一经生成不可修改，只能取消后重新请求。送达状态从记录派生：未请求、已请求未送达、已送达、送达冲突、送达失败。修改结果根形成新绑定版本，旧版本保留。

**文件。** `crates/sheltie-core/src/delivery/state.rs`。

**测试。** `delivery_records_append_on_succeeded_work`、`delivery_append_leaves_execution_result_unchanged`、`other_writes_on_terminal_work_still_rejected`、`delivery_state_unrequested_without_request`、`delivery_state_requested_until_receipt`、`delivery_state_delivered_after_accepted_receipt`、`delivery_state_conflict_from_component_report`、`delivery_state_failed_from_component_report`、`request_is_immutable_once_created`、`cancelled_request_allows_new_request`、`delivered_request_cannot_be_cancelled`、`retarget_creates_new_binding_version_and_keeps_old`

**实现要点。** 送达状态不落库，每次从记录派生，穷尽 `match`。“其他写操作仍被拒绝”这条测试走现有的 `decide`，不改它；交付命令走本模块自己的决定函数。

**停止条件。** 需要改现有 `decide` 对终态的拒绝逻辑。

**提交。** `feat(core): 终态 Work 只追加交付记录，送达状态由记录派生`

### C006-T05 core 回执核对

**结果。** 回执核对是纯函数：输入是请求、已有回执、引擎读回的每个文件结果（摘要，或缺失、符号链接、根身份不符、不可访问）、组件陈述。三者一致才接受：请求存在且没有回执、绑定版本一致、引擎摘要等于期望摘要、组件陈述等于引擎摘要。任一不符拒绝，并说明是哪一项、哪个文件。接受时，回执保存引擎摘要与读取时间，组件陈述另存。协调者自报只记为声明。

**文件。** `crates/sheltie-core/src/delivery/receipt.rs`。

**测试。** `receipt_accepted_when_engine_and_stated_digests_match_request`、`receipt_rejected_when_request_unknown`、`second_receipt_for_same_request_rejected`、`receipt_rejected_on_binding_version_mismatch`、`receipt_rejected_when_engine_digest_differs_from_request`、`receipt_rejected_when_stated_digest_differs_from_engine`、`receipt_rejected_when_target_file_missing`、`receipt_rejected_when_segment_is_symlink`、`receipt_rejected_when_root_identity_differs`、`accepted_receipt_keeps_engine_digest_and_statement_separate`、`self_report_recorded_as_claim_not_delivery`

**实现要点。** 核对顺序按 DL-05 的三步，先请求与绑定，再逐文件。组件陈述永远不替代引擎读回：引擎读回失败时，无论组件说什么都拒绝。

**停止条件。** 测试要求在核对中读文件。

**提交。** `feat(core): 以引擎读回为准核对送达回执`

### C006-T06 core 送达观测与状态卡用语

**结果。** 观测闭集：目标副本缺失、目标暂时不可访问、交付副本被修改；管理根的原件缺失映射为 `STORE_CORRUPT`，不作为观测。观测只追加，不改回执和送达状态。副本缺失时允许新的交付请求。状态卡的交付一节分开列出送达状态与观测；未采用 C004 时称「流程完成后的产物副本」，任何情况下都不出现「候选已经过接收出口」或质量结论；自报显示为自报。

**文件。** `crates/sheltie-core/src/delivery/observe.rs`、`crates/sheltie-core/src/delivery/render.rs`。

**测试。** `observation_missing_copy_when_target_absent_and_original_intact`、`observation_inaccessible_not_treated_as_deleted`、`original_missing_maps_to_store_corrupt`、`observation_modified_copy_compares_against_receipt_digest`、`observation_appends_without_changing_receipt_or_state`、`missing_copy_allows_new_request`、`status_card_says_product_copy_after_flow_completed`、`status_card_never_claims_acceptance_without_c004`、`status_card_shows_self_report_as_claim`、`status_card_lists_delivery_state_and_observations_separately`

**实现要点。** 措辞只从枚举映射，穷尽 `match`。快照逐字节比对。“被修改”与“原样”只按回执摘要区分，不猜用户做了什么。

**停止条件。** 快照与协议不一致。

**提交。** `feat(core): 送达观测与状态卡交付一节`

### C006-T07 core 来源清单与证据摘要投影

**结果。** 来源清单列出 Work 标识、Workbook 版本与摘要、每个交付产物及证据摘要的摘要（不含清单自身）、结果根绑定版本、请求标识与请求创建时间；位置只写根绑定加相对路径，不出现绝对路径。证据摘要在未采用 C004 时只说明“流程完成”，不给质量结论；只给指针，不含原始日志。同一请求两次渲染字节相等，新请求使用不同元数据路径。

**文件。** `crates/sheltie-core/src/delivery/provenance.rs`。

**测试。** `provenance_lists_work_workbook_digests_binding_and_request`、`provenance_uses_binding_and_relative_paths_not_absolute`、`provenance_regenerates_byte_equal_from_same_request`、`provenance_omits_own_digest`、`new_request_uses_new_metadata_paths`、`evidence_summary_without_c004_states_flow_completed_only`、`evidence_summary_gives_pointers_not_raw_logs`

**实现要点。** 纯函数：输入是请求创建时的冻结状态、原始证据记录、预分配的请求标识和请求创建时间，不读本次运行时钟。格式按 T01 第 4 项写定的版本号；先生成证据摘要，再生成只列产物及证据摘要的来源清单，避免清单引用自己的摘要。T12 在请求写入 Store 前调用本函数。

**停止条件。** 需要读文件内容。

**提交。** `feat(core): 渲染可重生成的来源清单与证据摘要`

### C006-M1 里程碑审查：core

**执行者。** 强模型，未参与 T03–T07。

**做什么。**

1. 读 `git diff c006-t02-skeleton..HEAD -- crates/sheltie-core`，按 engineering.md §5 检查表逐项打钩。
2. DL-03、DL-05、DL-07、DL-08 逐条对到一个正例测试和一个只改一个条件的反例测试；缺的补禁用测试并退回。重点：组件陈述正确但引擎读回不符时一定拒绝。
3. 运行 `scripts/mutants.sh sheltie-core`，逐个处置存活的突变体。
4. 运行 `scripts/check-core-vocab.sh`；grep 确认 core 中没有宿主名、Git 词汇。
5. grep 确认 T03–T07 的 `todo!`、`#[allow(unused_variables)]`、`#[ignore = "C006-T0x"]` 清零；复核本组的每一次工具改动。

**产出。** 创建 `milestones.md`，写入 M1 报告（格式见 §0.4）。

### C006-T08 根文件操作：重叠、根身份与逐段打开

**结果。** 目标根与管理根相等、包含或位于其内时拒绝（规范化路径与设备号、inode 双重比较，别名也能识别）。打开目标根后核对设备号与 inode 与绑定一致，不一致拒绝。从根目录句柄出发逐段不跟随链接地打开父目录，缺失的目录在父句柄内创建；任一段是符号链接或不是目录就拒绝。检查之后把某段换成符号链接，写入也不会到根外。

**文件。** `crates/sheltie-rootfs/src/overlap.rs`、`crates/sheltie-rootfs/src/walk.rs`。

**测试。** `overlap_rejects_target_equal_to_management_root`、`overlap_rejects_target_containing_management_root`、`overlap_rejects_target_inside_management_root`、`overlap_detects_alias_by_device_and_inode`、`open_root_rejects_identity_mismatch`、`open_parent_creates_missing_dirs_inside_root`、`open_parent_rejects_symlink_segment_and_sentinel_unchanged`、`open_parent_rejects_non_directory_segment`、`segment_swapped_to_symlink_after_check_does_not_escape_root`

**实现要点。** 每一步都拿上一步的目录句柄往下走，不拼接完整路径再打开。句柄用完即关，不缓存。

**停止条件。** `sys` 里没有需要的封装；某个平台的测试只能用 `#[cfg]` 跳过才能通过。

**提交。** `feat(rootfs): 根重叠、根身份与逐段不跟随链接的打开`

### C006-T09 根文件操作：临时文件与原子发布

**结果。** 在最终父目录句柄内以独占方式创建带请求标识的临时名，写入后 `sync_all`，以原子「不替换」改名发布；目标已存在或在检查后出现时改名失败，报冲突，对方文件字节不变，临时名清掉。已有且摘要相同的文件在上层恢复流程中跳过，不调用发布操作。目标文件的其他硬链接不被写穿。清理只删本请求标识的临时名。平台没有等价操作时不提供导出。

**文件。** `crates/sheltie-rootfs/src/write.rs`。

**测试。** `temp_name_carries_request_id_and_is_created_exclusively`、`publish_writes_when_target_absent`、`publish_conflicts_when_target_exists`、`publish_conflicts_when_target_appears_after_check_and_other_bytes_unchanged`、`publish_does_not_write_through_existing_hardlink`、`cleanup_removes_only_this_request_temp_names`、`platform_without_atomic_no_replace_refuses_export`

**实现要点。** 顺序：独占创建 → 写入 → `sync_all` → 原子「不替换」改名 → 同步目录。不存在“先检查目标是否存在再普通改名”的分支，也没有交换后补偿回退。

**停止条件。** 需要普通改名才能让测试通过。

**提交。** `feat(rootfs): 独占临时文件与原子不替换发布`

### C006-T10 根文件操作：不跟随链接的读回摘要

**结果。** 按与写入相同的逐段方式只读打开目标文件，读取字节并计算摘要；任一段是符号链接时拒绝。缺失、根不可访问、根身份不符三种情况分开报告，不混为一种。摘要编码与 core 一致。

**文件。** `crates/sheltie-rootfs/src/read.rs`。

**测试。** `read_digest_follows_no_symlink_in_any_segment`、`read_digest_reports_missing_file`、`read_digest_reports_root_identity_mismatch`、`read_digest_reports_inaccessible_root_distinct_from_missing`、`read_digest_matches_core_digest_encoding`

**实现要点。** 复用 T08 的逐段打开，只是全程只读、不创建目录。“不可访问”与“缺失”按系统调用的错误类别区分，映射表是骨架给的。

**停止条件。** 需要以写方式打开文件。

**提交。** `feat(rootfs): 不跟随链接的只读读回摘要`

### C006-M2 里程碑审查：根文件操作

**执行者。** 强模型，未参与 T08–T10。

**做什么。**

1. 读 `git diff <M1 结束提交>..HEAD -- crates/sheltie-rootfs`，逐项核对检查表；重点审 `sys` 封装的每个系统调用的标志位与错误翻译（包括骨架作者自己写的部分）。
2. 确认 CI 在 Linux 与 macOS 上都跑了本组全部测试，没有一条靠 `#[cfg]` 跳过；把两个平台的运行记录写进报告。
3. 运行 `scripts/mutants.sh sheltie-rootfs`，重点看 `O_NOFOLLOW`、`O_EXCL` 与「不替换」标志被删掉后是否有测试变红。
4. 每条拒绝测试都核对哨兵断言确实存在且覆盖字节、权限与修改时间。
5. 对照 design.md §3.3 六步，逐步写出实现位置与测试名；复核本组的每一次工具改动。

**产出。** 在 `milestones.md` 中追加 M2 报告。

### C006-T11 runtime 交付记录存储

**结果。** Store 能存取根绑定（用途、位置、身份、版本、授权来源）、交付请求（导出清单、期望摘要、绑定版本、请求标识）、回执（引擎摘要、读取时间、组件陈述）、送达观测、组件报告与自报。所有字段往返一致。请求行写入后不能更新。遇到旧结构版本的库时拒绝打开，并且不写入。

**文件。** `crates/sheltie-runtime/src/store/{schema,commit,read}.rs`（只填标 `C006-T11` 的函数）。

**测试。** `root_binding_round_trips_purpose_identity_version_source`、`delivery_request_round_trips_manifest_and_policy`、`delivery_request_row_cannot_be_updated`、`receipt_round_trips_engine_digests_and_statement`、`observation_round_trips_and_appends`、`component_report_and_claim_round_trip`、`delivery_columns_reject_unknown_closed_set_value`、`store_rejects_pre_delivery_schema_without_writing`

**实现要点。** 建表语句是骨架给出的常量，不改。闭集列存线上名，读出后用穷尽 `match` 还原。绝对路径只出现在根绑定表。

**停止条件。** 需要改表结构。

**提交。** `feat(runtime): 存储根绑定、交付请求、回执与送达观测`

### C006-T12 runtime 服务、读回与崩溃窗口

**结果。** `work start` 时授权结果根，记录绑定版本 1，与管理根重叠时拒绝。请求、回执、观测、修改结果根各在一个写锁事务内完成。创建请求时预分配请求标识与时间，调用 T07 生成元数据字节，再交给 T03 校验并作为同一不可变请求提交；失败时不留下半份请求。回执服务在写事务之前按绑定只读打开结果根，用 `sheltie-rootfs` 读回每个清单文件，把结果交给 core 核对，再在一个事务里写入。结果根从不以写方式打开。`delivery check` 只读核对并追加观测；管理根原件缺失报 `STORE_CORRUPT`。进程在回执事务提交前后被杀，重启后没有中间状态，回执至多一份。全部测试在只读假 `$HOME` 下通过。

**文件。** `crates/sheltie-runtime/src/service.rs`、`crates/sheltie-runtime/src/failpoint.rs`（只填标 `C006-T12` 的函数）。

**测试。** `service_work_start_authorizes_result_root_with_binding_v1`、`service_authorization_rejects_overlap_with_management_root`、`service_request_on_succeeded_work_commits_once`、`service_request_freezes_generated_metadata_bytes_and_paths`、`service_receipt_reads_back_target_and_commits`、`service_receipt_rejects_when_target_changed_before_read_back`、`service_receipt_never_opens_result_root_for_write`、`service_check_appends_missing_copy_and_inaccessible`、`service_check_reports_store_corrupt_when_original_missing`、`service_retarget_appends_new_binding_version`、`crash_before_receipt_commit_leaves_request_undelivered`、`crash_after_receipt_commit_keeps_single_receipt`、`delivery_services_write_nothing_outside_sheltie_home`

**实现要点。** 判断全部来自 core，runtime 只做读回、存取与事务（INV-2）。读回放在写锁之外，读回结果和读取时间一起交给 core。崩溃测试用骨架给出的 fail-point 名，不自己加 sleep。

**停止条件。** 需要以写方式打开结果根；需要第二把锁或第二套状态。

**提交。** `feat(runtime): 结果根授权、回执读回与交付事务`

### C006-T13 cli 交付命令与输出

**结果。** `work start` 接受结果根参数、不接受覆盖参数；`delivery` 命令组按 T01 第 1 项可用，`--json` 输出与协议一致；只读命令（`show`、`status`）不改变任何状态；缺参数、状态不合法、回执不符时给出协议规定的错误码与退出码。

**文件。** `crates/sheltie-cli/src/commands/{work,delivery}.rs`、`crates/sheltie-cli/src/{output,error_map}.rs`（只填标 `C006-T13` 的函数）。

**测试。** `cli_work_start_accepts_result_root_without_overwrite_option`、`cli_work_start_rejects_result_root_overlapping_home`、`cli_delivery_request_json_lists_manifest_and_binding`、`cli_delivery_request_rejected_before_success`、`cli_delivery_show_is_read_only`、`cli_delivery_receipt_accepts_matching_statement`、`cli_delivery_receipt_duplicate_rejected`、`cli_delivery_report_records_conflict_and_failure`、`cli_delivery_status_separates_state_and_observations`、`cli_delivery_claim_recorded_as_self_report`、`cli_error_codes_for_delivery_rejections_match_protocol`

**实现要点。** 命令树在骨架中定死，这里只填处理函数。输出不自己加字段。

**停止条件。** 协议与命令树不一致。

**提交。** `feat(cli): 结果根授权参数与 delivery 命令组`

### C006-M3 里程碑审查：runtime 与 cli

**执行者。** 强模型，未参与 T11–T13。

**做什么。**

1. 读 `git diff <M2 结束提交>..HEAD -- crates/sheltie-runtime crates/sheltie-cli`，逐项核对检查表。
2. 运行 `scripts/mutants.sh sheltie-runtime`。
3. 人工走查四个事务的边界、回执读回与写锁的先后，以及 fail-point 位置；确认 runtime 访问结果根的唯一路径是只读读回，与 T01 修订后的 `confine()` 条款一致。
4. 在只读假 `$HOME` 下重跑 runtime 与 cli 全部测试，记录结果；核对 INV-3 实现行。
5. 核对旧库拒绝；grep 确认 runtime 中没有宿主名或 Git 词汇。

**产出。** 在 `milestones.md` 中追加 M3 报告。

### C006-T14 导出组件：读请求与核对源文件

**结果。** 导出组件只经同版本的 `sheltie delivery show <request> --json` 读请求；`sheltie` 报请求不存在、已取消或已送达时不做任何写入。对产物条目，在管理根里只读打开冻结源文件并逐个核对摘要；对证据摘要和来源清单条目，使用请求给出的有界完整字节并核对期望摘要，不把它们伪装成源文件。源文件缺失或任一摘要不符记为送达失败，不写目标。

**文件。** `crates/sheltie-deliver/src/request.rs`、`crates/sheltie-deliver/src/source.rs`。

**测试。** `reads_request_through_sheltie_json_command`、`deliver_uses_sibling_sheltie_version_during_update`、`stops_without_writing_when_request_not_open`、`opens_source_files_read_only`、`source_digest_mismatch_is_delivery_failure`、`source_missing_reported_without_writing_target`、`generated_metadata_bytes_must_match_request_digest`

**实现要点。** 调用 `sheltie` 用骨架给出的子进程封装，不自己拼命令行、不直接读 Store。源文件的摘要用 core 的编码。

**停止条件。** 需要直接打开 `store.db`。

**提交。** `feat(deliver): 经公开命令读取交付请求并核对源文件`

### C006-T15 导出组件：写入与恢复

**结果。** 每次导出前重新检查目标根与管理根不重叠、根身份一致。对交付产物与证据摘要按 DL-06 处理：存在且摘要一致的不重写；不存在的清理本请求临时名后以原子“不替换”方式补写；存在但摘要不一致的记为送达冲突并停下，用户可另选结果根。上述文件一致后处理本请求独立路径下的来源清单：缺失则新增、相同则跳过、不同则冲突。恢复判断不以来源清单为依据。清单之外的文件不动。平台不支持时不提供导出。

**文件。** `crates/sheltie-deliver/src/export.rs`、`crates/sheltie-deliver/src/recover.rs`。

**测试。** `fresh_export_writes_all_entries_then_provenance_last`、`recover_skips_matching_files_without_rewrite`、`recover_writes_only_missing_files_after_cleaning_own_temp`、`recover_conflicts_on_modified_file_even_if_provenance_present`、`recover_does_not_treat_missing_provenance_as_unwritten`、`repeat_request_skips_identical_provenance_bytes`、`new_request_does_not_overwrite_old_provenance`、`modified_provenance_for_same_request_conflicts`、`files_outside_manifest_untouched`、`existing_different_file_conflicts_without_overwrite`、`retarget_after_conflict_keeps_original_target_bytes`、`overlap_and_root_identity_rechecked_before_every_export`、`unsupported_platform_offers_no_export`

**实现要点。** 第一次导出和恢复走同一段代码：没有“首次”与“恢复”两套逻辑，只有按实际字节分三种情况。文件写入全部经 `sheltie-rootfs`。

**停止条件。** 需要依据来源清单判断文件是否已写。

**提交。** `feat(deliver): 以交付请求为基准的写入与恢复`

### C006-T16 导出组件：读回与回执提交

**结果。** 全部文件处理完后，逐个读回目标文件的实际字节并计算摘要，经 `sheltie delivery receipt` 提交；遇到冲突或失败时经 `sheltie delivery report` 报告。已有回执时再次运行不重复提交、不重写。导出组件不直接写管理根；`sheltie` 命令可在 Store 中追加预期的回执或报告，管理根中的冻结原件保持不变。

**文件。** `crates/sheltie-deliver/src/receipt.rs`。

**测试。** `reads_back_each_target_before_submitting_receipt`、`submits_receipt_through_sheltie_command`、`reports_conflict_and_failure_through_sheltie_command`、`rerun_after_receipt_does_not_resubmit_or_rewrite`、`export_run_preserves_frozen_sources_and_adds_only_expected_store_record`、`deliver_crate_does_not_depend_on_sheltie_runtime`

**实现要点。** 组件陈述的摘要只是陈述，`sheltie` 会自己读回；这里不要试图“跳过读回以提速”。测试逐字节核对冻结原件，并经公开命令核对 Store 只新增与本请求对应的记录；不能要求整个管理根字节不变。

**停止条件。** 需要直接写 Store 或管理根里的任何文件。

**提交。** `feat(deliver): 读回目标并经 sheltie 提交回执`

### C006-T17 `self` 分发第二个二进制

**结果。** `self install`、`self update`、`self rollback` 把 `sheltie` 与 `sheltie-deliver` 当作同一版本处理：两个都在才安装；更新时版本不一致就拒绝；回滚同时恢复两个。先把两个已核对的二进制放进同一不可变版本目录，再原子切换一个当前版本指针；稳定入口都经该指针解析，不逐个替换两个二进制。旧的单文件布局迁移、指针切换前后崩溃与回滚按 T01 写定的合同处理。仍只写 `~/.sheltie`（GF-27、INV-3）。

**文件。** `crates/sheltie-runtime/src/selfmgmt.rs`、`crates/sheltie-cli/src/`、根 `Cargo.toml` 与 `crates/sheltie-deliver/Cargo.toml`，只改自管理入口、版本布局与发布资产所需部分；最终路径在 T02 按 T01 合同定死并同步 tasks.toml。

**测试。** `self_install_places_both_binaries_same_version`、`self_install_rejects_release_missing_deliver_binary`、`self_update_rejects_mismatched_deliver_version`、`self_migrates_legacy_binary_without_deleting_original_before_switch`、`self_switch_pointer_before_crash_keeps_old_pair`、`self_switch_pointer_after_crash_keeps_new_pair`、`self_rollback_after_migration_restores_old_binary`、`self_rollback_restores_both_binaries`

**实现要点。** 两个二进制先放在同一版本目录并逐个核对，只有一个指针参与可见版本切换。现有 `bin/sheltie` 的逐文件改名写法不能直接复用；迁移与回滚必须保留原二进制，不能在切换前删掉旧版本。静态检查真实发布包内的两个二进制及其版本，动态测试分别在迁移、指针切换前后注入中断。

**停止条件。** 需要写 `~/.sheltie` 之外的位置。

**提交。** `feat(runtime): self 同版本分发导出二进制`

### C006-T18 场景：交付、恢复与拒绝

**结果。** 全程用 `sheltie` 与 `sheltie-deliver` 两个真实二进制，覆盖 validation.md §3 中不依赖 C004、C005 的各行：写完文件、回执前被杀后恢复不重写；部分写入且来源清单缺失时只补缺失；来源清单在但文件被改时冲突；回执后被改只追加观测；组件谎报摘要被拒；符号链接段与目标根被替换被拒且哨兵不变；目标根与管理根重叠被拒；门槛未批准不生成请求；终态只接受交付记录；副本删除后重新导出；修改结果根后旧记录按旧绑定解析；导出组件不直接写管理根，`sheltie` 只追加本请求的交付记录。

**文件。** 无生产文件（场景测试只验证已有实现）。

**测试。** `scenario_killed_after_write_before_receipt_recovers_without_rewrite`、`scenario_partial_write_without_provenance_completes_missing_only`、`scenario_modified_target_with_provenance_present_conflicts`、`scenario_target_modified_after_receipt_adds_observation`、`scenario_component_misreports_digest_receipt_rejected`、`scenario_symlink_segment_rejected_sentinel_unchanged`、`scenario_replaced_root_rejected`、`scenario_target_overlapping_management_root_rejected`、`scenario_unapproved_gate_produces_no_request`、`scenario_terminal_work_accepts_only_delivery_records`、`scenario_deleted_copy_reexported_with_new_request`、`scenario_retarget_old_records_resolve_old_binding`、`scenario_component_does_not_write_management_root_directly`

**实现要点。** 本任务只删 `#[ignore`，让已写好的场景变绿。任一场景红，说明前面某个任务有缺陷：记下测试名和失败输出，把状态改为 `blocked`，交给审查者，不改生产代码。

**停止条件。** 任一场景红。

**提交。** `test(deliver): 启用交付、恢复与拒绝场景`

### C006-T19 skill 交付一节

**执行者。** 强模型。skill 是给协调者读的说明书，措辞直接影响行为。

**结果。** `skills/sheltie/SKILL.md` 增加交付一节：Work `succeeded` 后怎样生成请求并调用 `sheltie-deliver`；怎样读送达状态与观测；送达冲突时停下交给人，不重跑覆盖；为什么不能用“我已经复制好了”代替回执；未采用 C004 时怎样称呼交付物。skill 不保存送达状态，不判断是否需要重试（GF-18）。`scripts/check-skill.sh` 通过。

**文件。** `skills/`、`crates/sheltie-cli/tests/skill.rs`。

**测试。** `delivery_section_commands_exist_in_protocol_or_deliver_cli`

**提交。** `feat(skill): 交付与送达状态说明`

### C006-M4 里程碑审查：导出组件与端到端

**执行者。** 强模型，未参与 T14–T19。

**做什么。**

1. 读 `git diff <M3 结束提交>..HEAD`，逐项核对检查表；运行 `scripts/mutants.sh sheltie-deliver`。
2. 把 validation.md §3 的每一行对到一个通过的测试名，写成对照表；依赖 C004、C005 的行标“增量组”；两个平台各记一次运行。
3. spec.md §3 的每条承诺逐行给出正例、反例和信任前提。
4. grep CLI 输出、skill 与来源清单，确认没有 spec.md §4“不能说”清单里的措辞，未采用 C004 时没有“候选已经过接收出口”。
5. 全量运行 `cargo nextest run --all-features`，记录总数与耗时；在 validation.md 执行状态表中填写失败路径测试一行。

**产出。** 在 `milestones.md` 中追加 M4 报告。

### C006-T20 真实交付回归

**执行者。** 人（真实宿主操作者）。实施者准备绑定 M4 候选的两个二进制、独立管理根、skill 和逐步记录模板。

**结果。** 用自备任务样本（与探针相同口径）跑真实 Work，三种目标位置（默认、项目目录内、自定义）各至少一次；其中一次在导出途中杀掉进程再恢复，一次在送达后编辑副本再 `delivery check`。记录找到成果的时间、复制与核对的人工分钟、送达错误，与探针基线对照（validation.md §2 指标）。

**文件。** 本 package 的 `validation.md`、`progress.md` 与原始记录目录。

**停止条件。** 用户仍需要自己去管理根找文件或手工核对：按 engineering.md §7 路由到命令、默认位置或 skill，不进入发布。

**提交。** `docs(specs): 记录 C006 真实交付回归`

### C006-T21 发布

**执行者。** 人（发布操作者）；需要用户授权。

**结果。** M4（及插入的增量组里程碑）、T20 通过后发布；发布资产包含两个二进制；写 release record 与 CHANGELOG；package 转为 `completed`，在 `review.md` 新增「实施审查」一节并写最终结论行，满足 `scripts/check-specs.sh`。

**提交。** `chore(release): 发布 C006`（版本号以采用时的 README 为准）

## 3. C004 增量组（C004 实现后插入）

C004 已实现（C004-T24 完成）时，由一次补充计划提交把下列任务插入 §1（排在 T20 之前）和 tasks.toml；C006-T30 写出它们的骨架与测试。未插入前，主线不出现 C004 的字段和用语。

| ID | 执行者 | 标题 | 依赖 |
| --- | --- | --- | --- |
| C006-T30 | 强模型 | 增量骨架与测试 | C004 已实现、M4 |
| C006-T31 | 初级 | 证据摘要来自 C004 证据记录 | T30 |
| C006-T32 | 初级 | 采用 C004 时的交付物用语与来源清单的候选标识 | T30 |
| C006-T33 | 初级 | 场景：接收出口之后交付 | T31–T32 |
| C006-M5 | 强模型 | 里程碑审查：C004 增量组 | validation.md §3 中依赖 C004 的各行 |

#### C006-T31 证据摘要来自 C004 证据记录

**结果。** 证据摘要列出每个条件的结果、写入方式（本地验证器取得或外部回填）和原件指针；外部回填标明，不显示为已验证；不含原始日志。

**文件。** `crates/sheltie-core/src/delivery/provenance.rs`（只填标 `C006-T31` 的函数）。

**测试。** `evidence_summary_lists_condition_results_with_write_mode`、`evidence_summary_marks_external_backfill_as_not_verified`

**提交。** `feat(core): 证据摘要引用 C004 证据记录`

#### C006-T32 交付物用语与候选标识

**结果。** 采用 C004 且 Work `succeeded` 时，状态卡与来源清单称“候选已经过接收出口”，来源清单写入候选标识；“经过接收出口”不附带质量结论。

**文件。** `crates/sheltie-core/src/delivery/render.rs`、`crates/sheltie-core/src/delivery/provenance.rs`（只填标 `C006-T32` 的函数）。

**测试。** `status_card_with_c004_says_candidate_passed_acceptance_exit`、`provenance_with_c004_lists_candidate_identity`

**提交。** `feat(core): 采用 C004 时的交付用语与候选标识`

#### C006-T33 场景：接收出口之后交付

**文件。** 无生产文件。

**测试。** `scenario_delivery_after_acceptance_exit_lists_evidence_summary`

**提交。** `test(deliver): 启用接收出口之后的交付场景`

## 4. C005 增量（C005 第一阶段实现后插入）

交接材料（C005 EX-01）只在请求时导出。C005-T07 的交接包投影落地后，由一次补充计划提交插入下列任务。增量小，骨架、测试与实现都由强模型在一个任务内完成，仍然一任务一提交，里程碑并入届时最近一次审查。

| ID | 执行者 | 标题 | 结果 |
| --- | --- | --- | --- |
| C006-T40 | 强模型 | 按请求导出交接材料 | `delivery request --include-handoff` 把 C005 交接包作为导出清单的一项；不请求时不导出；交接包仍是只读投影 |

测试（插入时写出）：`handoff_exported_only_when_requested`、`handoff_entry_digest_matches_projection`。

## 5. 完成判据

C006 完成，当且仅当：

1. §1 表中所有任务和里程碑都是 `done`，每个任务对应一个提交；插入的增量组也是 `done`。
2. validation.md §3 的每一行都有在 Linux 与 macOS 上通过的测试，或写明属于增量组。
3. 仓库里没有 C006 的 `todo!`，也没有 `#[ignore = "C006-`。
4. `scripts/check-core-vocab.sh` 通过；`sheltie-deliver` 不依赖 `sheltie-runtime`；`sheltie` 的测试在只读假 `$HOME` 下通过，导出组件的测试证明它不写管理根。
5. 所有拒绝写入的测试都有根外哨兵断言，且哨兵不变。
6. T20 有人工记录；对外用语与 spec.md §4 一致。
