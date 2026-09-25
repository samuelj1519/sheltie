# 工程规范

给在本仓库写代码、写文档、做 Review 的人和 agent。目标读者具备 Rust 基础但第一次接触项目。本文只讲怎么干活；产品语义见 [规格](spec.md)，机制见 [架构](architecture.md) 与 [contracts/](contracts/)。

## 1. 工作方式：spec 先于代码

1. 动手前找到依据。产品行为 → `spec.md`；类型与模块 → `architecture.md`；字段、命令、表结构 → 对应合同；做哪一步 → `plan.md`。
2. 依据缺失或冲突时，**先改文档再写代码**。产品问题改 `spec.md`，机制问题改合同，顺序问题改 `plan.md`。在提交里同时带上文档改动。
3. 文档描述目标，不描述进度。代码进度只看 `plan.md` 每个任务的状态与 git 历史。不得把计划中的能力写成「已支持」。
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
tasks.toml            机器可读的任务白名单（T01 生成）
.config/nextest.toml  测试运行配置（crash 测试串行）
dist-workspace.toml   cargo-dist 发布配置（T20 生成）
AGENTS.md             agent 入口；CLAUDE.md 只含 @AGENTS.md
```

依赖方向只能向下：`cli → runtime → core`。`sheltie-core` 的 `Cargo.toml` 不得出现 `rusqlite`、`tokio`、`rand`、任何文件系统或时钟库。

### 2.2 代码风格

- 每个 crate 根 `#![forbid(unsafe_code)]`；lib crate 另加 `#![deny(clippy::unwrap_used, clippy::expect_used)]`。测试代码可以 `unwrap`。
- 每个 crate 一个 `Error` 枚举，用 `thiserror`。错误携带足够定位的字段（路径、字段名、规则名），CLI 层映射为 [协议](contracts/protocol.md) §7 的错误码。不用 `anyhow` 穿透 crate 边界；`cli` 内部可以用。
- ID、路径、摘要、有界文本都用 newtype，构造函数校验，字段私有。模块边界不传裸 `String`。
- 所有 `serde` 结构 `#[serde(deny_unknown_fields)]`。枚举用 `rename_all = "snake_case"`。
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

MVP 期间测试由 [plan.md](plan.md) T01 一次写好并禁用，实现者不写测试、不改测试。实现者的循环是：

1. 解开本任务的测试（删 `#[ignore = "Tnn"]`），跑 `scripts/task.sh Tnn`，看到全红。编译不过不算红；先让它编译。
2. 挑一个红的，填对应的 `todo!()`，让它绿。不改签名，不改断言，不改快照。
3. 下一个红的。
4. 全绿后跑四条门禁与 `scripts/check-task.sh Tnn`，提交。

写新测试的人（T01 的骨架作者、里程碑审查者）遵守：测试名写「条件 → 行为」，例如 `begin_rejects_node_not_in_next`，不带任务编号，归属写在上方的 `// Task: Tnn` 注释里（[plan.md §0.5](plan.md)）；每条能力至少一对：一个合法例，一个只改一个条件的拒绝例；期望值来自合同、手写字节或独立计算，不调用被测代码生成同一个答案。另外四条（M1 教训）：每个大小或个数上限有一对测试，恰好上限接受、多一个拒绝；快照与断言里出现的每个数值字段至少有一条非零、非默认值的断言（夹具时钟固定时，时间差单独造数据测）；合同里每句「不得」「必须」都有一条拒绝例；骨架函数的文档注释要用到的每个值都必须能从参数得到，做不到就改签名，不留给实现者在函数里重算。

### 3.2 分层

| 层 | 在哪 | 测什么 | 工具 |
| --- | --- | --- | --- |
| core 单元 | `crates/sheltie-core/src/**/tests.rs` 或 `tests/` | 解析、编译、`decide`、`legal_next`、状态卡渲染。输入手写，时间与 ID 固定 | 普通 `#[test]`；状态卡用 `insta` 快照 |
| runtime 集成 | `crates/sheltie-runtime/tests/` | 事务原子、重放、`REVISION_CONFLICT`、封存、路径约束、旧库拒绝 | `tempfile` 建独立 `SHELTIE_HOME` |
| CLI 端到端 | `crates/sheltie-cli/tests/` | 用 `examples/` 三份 Workbook 走完整场景；`--json` 输出可解析；退出码 | `assert_cmd` + `tempfile` |
| 崩溃 | `crates/sheltie-runtime/tests/crash.rs` | 在 `COMMIT` 前后注入失败（feature `fail-points` 或子进程 kill），重启后状态一致 | 子进程 + 临时目录 |

期望值来自合同、手写字节或独立计算，不调用被测代码生成同一个答案。fake 只替换外部边界（时钟、ID、文件观察），不直接写「成功」进状态。

### 3.3 什么不算验证

- 「编译过了」。
- 只断言内部计数或 `is_ok()`。
- 测试和实现调用同一个 helper 算期望。
- 删掉失败的用例或把断言改成实现现在的输出。
- 重跑一次变绿的偶发失败。先找原因，不重跑到绿。

## 4. 提交

- 一个 `plan.md` 任务 = 一个提交。提交时 §2.3 四条命令全绿。
- 不留编译不过的中间态；不为「先让它编译」加空实现、假成功或长期兼容层。
- 一次接口变化涉及的全部调用方、fixture、文档在同一提交里改完。
- 信息格式（本仓库与 `spec-dev` Workbook 共用）：

  ```text
  <type>(<scope>): <中文摘要，一行，不超过 50 字，不带句号>

  <正文：改了什么、为什么、怎么验证。每段一个意思。不复述 diff。>

  Task: T05
  Work: 2026-09-24-001-xxx
  Agent: Claude
  ```

  `type` 取 `feat | fix | refactor | test | docs | chore | perf | revert`，`scope` 是 crate 名或目录名（`core`、`runtime`、`cli`、`specs`、`workbook`）。类型与范围用英文，`git-cliff` 按它分组生成变更日志；摘要与正文用中文。末尾三行是 git trailer：`Task` 对应 `plan.md` 的任务或 `spec-dev` 的 `Tnn`，`Work` 只在 Sheltie Work 里运行时填 `work_id`，`Agent` 必填，写实际提交者（模型名或人名）。不适用的 trailer 省略，不填占位符。`Co-Authored-By` 等其他 trailer 与它们放在同一段，中间不空行，否则 git 不把 `Task`、`Agent` 认作 trailer。

  ```text
  feat(core): 把 Flow 编译成校验过的图

  按 contracts/workbook.md §4 实现规则 1 到 9。可达性用 BFS。每条规则一个拒绝测试，
  另有 proptest 证明任意 2 到 8 节点的图编译不会 panic。

  Task: T05
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
| 突变 | `cargo mutants` 的幸存突变每个都有处置：补测试或删死代码？ |
| 证据 | 每条结论附可重跑的命令与完整输出，不是摘要？对被审者的反驳逐条核实过？ |

结论只有三种：通过、需修改（列出每条与依据）、阻断（缺的输入是什么）。

MVP 期间逐任务的审查由编译器、测试与 `scripts/check-task.sh` 承担；模型审查只在 [plan.md](plan.md) 的 M1、M2、M3 三个里程碑做，范围是两个里程碑之间的 diff。

## 6. 文档写法

- 简体中文，短句，一段一个意思。术语按 [CONTEXT.md](../CONTEXT.md)，同一个东西只用一个名字。
- 命令、路径、字段、错误码放代码环境，写真实符号，不写近义描述。
- 表格用于并列映射，编号列表用于顺序步骤。
- 「必须」「不得」表示强制与禁止，「可以」表示可选。不用「大概」「尽量」。
- 每份文档只做一种事：规格讲目标，合同讲字段，计划讲步骤，本文讲方法。不混。
- 改文档跑 `scripts/check-docs.sh`。

## 7. 遇到缺口

| 发现 | 去哪 |
| --- | --- |
| 产品该不该做某事、边界在哪 | 改 `spec.md`，在 `decisions.md` 追加一条 `D-nn` |
| 字段、命令、表结构、状态转换没定义 | 改对应合同 |
| 任务顺序不对、依赖缺失 | 改 `plan.md` |
| 已定义的东西实现错了 | 直接修，补拒绝例 |
| 局部写法选择 | 自己定，不用问 |

不要为了让当前任务编译通过而临时发明第二套状态、兼容入口或「先放这里以后再改」。
