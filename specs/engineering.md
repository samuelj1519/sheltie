# 工程规范

给在本仓库写代码、写文档、做 Review 的人和 agent。目标读者具备 Rust 基础但第一次接触项目。本文只讲怎么干活；产品语义见 [规格](spec.md)，机制见 [架构](architecture.md) 与 [contracts/](contracts/)。

## 1. 工作方式：spec 先于代码

1. 动手前找到依据。产品行为 → `spec.md`；类型与模块 → `architecture.md`；字段、命令、表结构 → 对应合同；当前任务与进度 → active change 的 `plan.md`。入口见 [change 索引](changes/README.md)。
2. 依据缺失或冲突时，**先改文档再写代码**。产品问题改 `spec.md`，机制问题改架构或合同，跨任务的重要选择新增 ADR，任务顺序改 active change plan。在提交里同时带上文档改动。合同与任务卡点名第三方库时，先核对其公开 API 能支撑合同的每一步再写入（M3 教训 D-30：`axoupdater` 写进合同后核对不可行）。
3. 文档描述目标，不描述进度。代码进度只看 active change plan 的任务状态与 git 历史。没有 active change 时不得从 proposed package 自行开工。不得把计划中的能力写成「已支持」。
4. 一个事实只在一处定义，别处链接。`scripts/check-docs.sh` 检查断链与禁用词。

## 2. Rust 约定

### 2.1 Workspace

```text
Cargo.toml            虚拟 workspace；[workspace.dependencies] 统一版本
crates/sheltie-core
crates/sheltie-runtime
crates/sheltie-cli    二进制名 sheltie
examples/             三份样例 Workbook（同时是测试 fixture）
workbooks/            可分发的业务 Workbook（spec-dev）
skills/sheltie/       SKILL.md
scripts/              check-docs.sh、check-core-vocab.sh、check-tests.sh、check-skill.sh、task.sh、check-task.sh、mutants.sh
specs/releases/v0.1.0/tasks.toml  MVP legacy 任务白名单
specs/changes/        后续迭代 package；active change 自带 plan、progress、validation 与 tasks.toml
.config/nextest.toml  测试运行配置（crash 测试串行）
dist 配置            根 Cargo.toml 的 [workspace.metadata.dist]（T25 起；0.32.0 不认 T20 手写的 dist-workspace.toml）
AGENTS.md             agent 入口；CLAUDE.md 只含 @AGENTS.md
```

依赖方向只能向下：`cli → runtime → core`。`sheltie-core` 的 `Cargo.toml` 不得出现 `rusqlite`、`tokio`、`rand`、任何文件系统或时钟库。

### 2.2 代码风格

- 每个 crate 禁止 `unsafe`；根 `Cargo.toml` 的 `[workspace.lints.rust] unsafe_code = "forbid"` 是实际门禁。lib crate 另禁用 `clippy::unwrap_used` 与 `clippy::expect_used`；测试代码可以 `unwrap`。OS 主体使用 D-036 选定的安全 Rust API，不直接调用 `libc`。
- 每个 crate 一个 `Error` 枚举，用 `thiserror`。错误携带足够定位的字段（路径、字段名、规则名），CLI 层映射为 [协议](contracts/protocol.md) §7 的错误码。不用 `anyhow` 穿透 crate 边界；`cli` 内部可以用。
- ID、路径、摘要、有界文本都用 newtype，构造函数校验，字段私有。模块边界不传裸 `String`。
- 完整合同载荷的 `serde` 解码必须使用 `#[serde(deny_unknown_fields)]`，拒绝未知字段。枚举用 `rename_all = "snake_case"`。私有、只读的身份投影可以仅解码所需字段；投影通过不代表完整载荷合格，其余字段仍须在任何业务 I/O 前由完整严格解码拒绝。投影只服务已有调用义务，不作为新的载荷入口。
- 状态机用 `enum` 表达，`match` 穷尽，不写 `_ =>` 兜底。
- 函数默认私有；`pub` 只给真实有外部调用者的项。不为 mock 加 trait。
- 注释只写代码说不出的「为什么」。不写阶段叙述，不写显而易见的「这里做 X」。
- 业务词汇（Review、Verdict、Pass、Fail、Spec、Plan、Git）不得出现在 `core` 与 `runtime` 的类型名、变体名与分支里。`scripts/check-core-vocab.sh` 在 CI 里 grep。

### 2.3 命令

在仓库根运行。每次提交前四条都要过：

```bash
cargo fmt --all -- --check
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run --all-features --no-tests=pass
```

另有 `cargo deny check`（依赖许可与漏洞）与 `scripts/check-docs.sh`。CI 跑同一组，本地 pre-commit 也跑同一组。

## 3. 测试：先红后绿

### 3.1 循环

MVP 期间测试由 [legacy plan](releases/v0.1.0/plan.md) T01 一次写好并禁用，实现者不写测试、不改测试。该方法保留为历史，不自动适用于后续 change。 后续方案采用[分阶段搭建与实现](guides/proposal-implementation.md)时，由强模型先定总体接口与关键 oracle，再按阶段搭骨架和测试，简单模型实现，独立强模型审查里程碑。后续任务按 active package plan 指定测试 Owner；interface 或行为修复必须由同一任务补真实 caller 回归。

MVP 填空任务的循环是：

1. 启用本任务的测试（删 `#[ignore = "Tnn"]`），跑 `scripts/task.sh Tnn`，看到全红。编译不过不算红；先让它编译。
2. 挑一个红的，填对应的 `todo!()`，让它绿。不改签名，不改断言，不改快照。
3. 下一个红的。
4. 全绿后跑四条门禁与 `scripts/check-task.sh Tnn`，提交。

写新测试的人（T01 的骨架作者、里程碑审查者）遵守：测试名写「条件 → 行为」，例如 `begin_rejects_node_not_in_next`，不带任务编号，归属写在上方的 `// Task: Tnn` 注释里（[legacy plan §0.5](releases/v0.1.0/plan.md)）；每条能力至少一对：一个合法例，一个只改一个条件的拒绝例；期望值来自合同、手写字节或独立计算，不调用被测代码生成同一个答案。另外五条（M1 教训）：每个大小或个数上限有一对测试，恰好上限接受、多一个拒绝；快照与断言里出现的每个数值字段至少有一条非零、非默认值的断言（夹具时钟固定时，时间差单独造数据测）；合同里每句「不得」「必须」都有一条拒绝例；骨架函数的文档注释要用到的每个值都必须能从参数得到，做不到就改签名，不留给实现者在函数里重算；查表与换算类函数（日期、进位、修正项）的用例要覆盖每个修正项生效的区段，找不到判定输入时穷举可达定义域找第一个判定点（`from_unix_secs` 的世纪修正项在 1970 到 2100 年的用例下全部摸不到，判定点是 1970-03-01）。第六条（M2 教训）：重放与恢复类测试要断言重建出的内容与提交时逐字节一致（或摘要相等），只断言「文件存在」不够——`stats.json` 的重建口径与提交口径不一致就是这样漏掉的。

### 3.2 分层

| 层 | 在哪 | 测什么 | 工具 |
| --- | --- | --- | --- |
| core 单元 | `crates/sheltie-core/src/**/tests.rs` 或 `tests/` | 解析、编译、`decide`、`legal_next`、状态卡渲染。输入手写，时间与 ID 固定 | 普通 `#[test]`；状态卡用 `insta` 快照 |
| runtime 集成 | `crates/sheltie-runtime/tests/` | 事务原子、重放、`REVISION_CONFLICT`、封存、路径约束、旧库拒绝 | `tempfile` 建独立 `SHELTIE_HOME` |
| CLI 端到端 | `crates/sheltie-cli/tests/` | 用 `examples/` 三份 Workbook 走完整场景；`--json` 输出可解析；退出码 | `assert_cmd` + `tempfile` |
| 崩溃 | `crates/sheltie-runtime/tests/crash.rs` | 在 `COMMIT` 前后注入失败（feature `fail-points` 或子进程 kill），重启后状态一致 | 子进程 + 临时目录 |

测试要构建产物（如带特性的二进制）时问 cargo 要路径（`cargo build --message-format=json` 里的 `executable`），不要按相对路径猜 target 目录——全局 `~/.cargo/config.toml` 可能把它指到别处（M3 教训）。

期望值来自合同、手写字节或独立计算，不调用被测代码生成同一个答案。fake 只替换外部边界（时钟、ID、文件观察），不直接写「成功」进状态。

### 3.3 什么不算验证

- 「编译过了」。
- 只断言内部计数或 `is_ok()`。
- 测试和实现调用同一个 helper 算期望。
- 删掉失败的用例或把断言改成实现现在的输出。
- 重跑一次变绿的偶发失败。先找原因，不重跑到绿。

## 4. 提交

- active change plan 的一个任务 = 一个提交。提交时 §2.3 四条命令和 package plan 的附加门禁全绿。MVP legacy task 保持原有映射。
- 不留编译不过的中间态；不为「先让它编译」加空实现、假成功或长期兼容层。
- 一次接口变化涉及的全部调用方、fixture、文档在同一提交里改完。
- 信息格式（本仓库与 `spec-dev` Workbook 共用）：

  ```text
  <type>(<scope>): <中文摘要，一行，不超过 50 字，不带句号>

  <正文：改了什么、为什么、怎么验证。每段一个意思。不复述 diff。>

  Change: C002
  Task: C002-T05
  Work: 2026-09-24-001-xxx
  Agent: Claude
  ```

  `type` 取 `feat | fix | refactor | test | docs | chore | perf | revert`，`scope` 是 crate 名或目录名（`core`、`runtime`、`cli`、`specs`、`workbook`）。类型与范围用英文，`git-cliff` 按它分组生成变更日志；摘要与正文用中文。末尾是 git trailer：新迭代的 `Change` 与 `Task` 对应 active package；`Work` 只在 Sheltie Work 里运行时填 `work_id`；`Agent` 必填，写实际提交者。MVP T01–T26 保留只有 `Task` 的 legacy 格式。不适用的 trailer 省略，不填占位符。`Co-Authored-By` 等其他 trailer 与它们放在同一段，中间不空行。

  ```text
  feat(core): 把 Flow 编译成校验过的图

  按 contracts/workbook.md §4 实现规则 1 到 9。可达性用 BFS。每条规则一个拒绝测试，
  另有 proptest 证明任意 2 到 8 节点的图编译不会 panic。

  Change: C002
  Task: C002-T05
  Agent: Claude
  ```

- 提交后核对 `git show --stat`，范围只包含本任务的文件。

## 5. Review 检查表

审查者不参与被审代码的编写。按下表逐项打钩，不适用项写原因。

| 项 | 问什么 |
| --- | --- |
| 依据 | 每个行为能指回 `spec.md`、合同或 `plan.md` 的哪一条？有没有文档没写却实现了的行为？ |
| 不变式 | `INV-1` 到 `INV-7` 有没有被碰？`core` 有没有 I/O？`runtime` 有没有做业务判断？ |
| 正反例 | 每条新规则都有合法例与拒绝例？拒绝例只改了一个条件？合同里每句「不得」「必须」都找得到拒绝例？ |
| 真实链 | 端到端测试走的是 CLI 入口与真实临时目录，不是私有函数？ |
| 崩溃 | 新写路径在 `COMMIT` 前后被杀，重启后状态一致？效果幂等？ |
| 边界 | 外部输入的路径都经 `confine()`？大小上限都有「恰好上限接受、多一个拒绝」一对测试？未知字段被拒？ |
| 文档 | 合同、样例、状态卡快照与实现一致？`check-docs.sh` 过？ |
| 提交 | 一个任务一个提交？信息格式对？无关文件没混进来？ |
| 突变 | `cargo mutants` 里存活的突变体每个都有处置：补测试或删死代码？ |
| 证据 | 每条结论附可重跑的命令与完整输出，不是摘要？对被审者的反驳逐条核实过？ |

结论只有三种：通过、需修改（列出每条与依据）、阻断（缺的输入是什么）。

全仓措辞清扫（改名、去翻译腔）不得动 `#[cfg(test)]` 模块：测试是合同，`check-task.sh` 按「测试零改动」核对。非动不可时，由复核者逐处核实改动只是文字，然后重打 `tNN-review` 基准 tag（M1 第三轮 N1）。

MVP 的逐任务与 M1–M3 审查规则保存在 [legacy plan](releases/v0.1.0/plan.md)。后续 change 的逐任务与里程碑审查范围由 package plan 定义；最终 review、候选 hash 和输入闭包写入 package `review.md` 与 `validation.md`。

## 6. 文档写法

- 简体中文，短句，一段一个意思。术语按 [CONTEXT.md](../CONTEXT.md)，同一个东西只用一个名字。
- 命令、路径、字段、错误码放代码环境，写真实符号，不写近义描述。
- 表格用于并列映射，编号列表用于顺序步骤。
- 「必须」「不得」表示强制与禁止，「可以」表示可选。不用「大概」「尽量」。
- 每份文档只做一种事：规格讲目标，合同讲字段，change plan 讲步骤，progress 讲当前交接，validation 讲证据，ADR 讲理由，本文讲方法。不混。
- 改文档跑 `scripts/check-docs.sh`。

## 7. 遇到缺口

| 发现 | 去哪 |
| --- | --- |
| 产品该不该做某事、边界在哪 | proposed change 写问题与候选；采用后改 `spec.md`，重要选择新增 `decisions/D-nnn-*.md` |
| 字段、命令、表结构、状态转换没定义 | 改对应合同 |
| 任务顺序不对、依赖缺失 | 改 active change 的 `plan.md` |
| 已定义的东西实现错了 | 直接修，补拒绝例 |
| 局部写法选择 | 自己定，不用问 |

不要为了让当前任务编译通过而临时发明第二套状态、兼容入口或「先放这里以后再改」。
