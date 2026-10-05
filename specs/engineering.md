# 工程规范

给在本仓库写代码、写文档、做 Review 的人和 agent。目标读者具备 Rust 基础但第一次接触项目。本文只讲怎么干活；产品语义见 [规格](spec.md)，机制见 [架构](architecture.md) 与 [contracts/](contracts)。

## 1. 工作方式：spec 先于代码

1. 动手前找到依据。产品行为 → `spec.md`；类型与模块 → `architecture.md`；字段、命令、表结构 → 对应合同；当前任务与进度 → active change 的 `plan.md`。入口见 [change 索引](changes/README.md)。
2. 依据缺失或冲突时，**先改文档再写代码**。产品问题改 `spec.md`，机制问题改架构或合同，跨任务的重要选择新增 ADR，任务顺序改 active change plan。在提交里同时带上文档改动。合同与任务卡点名第三方库时，先核对其公开 API 能支撑合同的每一步再写入（M3 教训 D-30：`axoupdater` 写进合同后核对不可行）。
3. 文档描述目标，不描述进度。代码进度只看 active change plan 的任务状态与 git 历史。没有 active change 时不得从 proposed package 自行开工。不得把计划中的能力写成「已支持」。
4. 一个事实只在一处定义，别处链接。`scripts/check-docs.sh` 检查断链与禁用词。

未发布Cargo候选的基础版本以[specs入口](README.md)首屏唯一`开发目标`字段为准；数值产品active目标必须一致。已完成产品、非产品实验或暂无active都不把候选变成release。已发布版本继续核release/tag/历史和CHANGELOG，见[D-043](../docs/explanation/decisions/D-043-development-target-authority.md)。

## 2. Rust 约定

### 2.1 Workspace

目录与源码入口见[实现定位](../docs/reference/implementation.md)。依赖统一在 `[workspace.dependencies]` 管理；发布配置在根 Cargo.toml，变更须核实际分发消费者。

引擎依赖方向只能向下：`cli → runtime → core`；exporter只可依赖纯core与公共安全库，不依赖runtime。`sheltie-core` 的 `Cargo.toml` 不得出现 `rusqlite`、`tokio`、`rand`、任何文件系统或时钟库。

### 2.2 代码风格

- 每个 crate 禁止 `unsafe`；根 `Cargo.toml` 的 `[workspace.lints.rust] unsafe_code = "forbid"` 是实际门禁。lib crate 另禁用 `clippy::unwrap_used` 与 `clippy::expect_used`；测试代码可以 `unwrap`。OS 主体使用 D-036 选定的安全 Rust API，不直接调用 `libc`。
- 每个 crate 一个 `Error` 枚举，用 `thiserror`。错误携带足够定位的字段（路径、字段名、规则名），CLI 层映射为 [协议](contracts/protocol.md) §7 的错误码。不用 `anyhow` 穿透 crate 边界；`cli` 内部可以用。
- ID、路径、摘要、有界文本都用 newtype，构造函数校验，字段私有。模块边界不传裸 `String`。
- 完整合同载荷的 `serde` 解码必须使用 `#[serde(deny_unknown_fields)]`，拒绝未知字段。枚举用 `rename_all = "snake_case"`。私有、只读的身份投影可以仅解码所需字段；投影通过不代表完整载荷合格，完整 Command 与成功快照（含 data）仍须在读取冻结业务文件前严格解码。效果载荷按存储合同 §3.2 在任何效果动作前严格解码并校验整组闭包；核实原响应资格可以只读冻结定义，效果错误不得丢掉已核合法 original。投影只服务已有调用义务，不作为新的载荷入口。
- 状态机用 `enum` 表达，`match` 穷尽，不写 `_ =>` 兜底。
- 函数默认私有；`pub` 只给真实有外部调用者的项。不为 mock 加 trait。
- 注释只写代码说不出的「为什么」。不写阶段叙述，不写显而易见的「这里做 X」。
- 业务词汇（Review、Verdict、Pass、Fail、Spec、Plan、Git）不得出现在 `core` 与 `runtime` 的类型名、变体名与分支里。`scripts/check-core-vocab.sh` 在 CI 里 grep。

### 2.3 命令

在仓库根运行。按实际影响选择每次提交的必需检查；源码、fixture、说明文件、生成输入和构建配置都可能是消费者的输入，不能只按文件后缀判断。

| 变化 | 每次提交的必需检查 | 完整工程门禁 |
| --- | --- | --- |
| 纯文案、方案或实验记录 | docs/specs、TOML 与事实/引用复核；涉及测试声明时 check-tests | 不要求无关 Rust 重跑 |
| Workbook、skill、fixture 或生成输入 | 上述检查与实际解析、CLI 场景或生成消费者 | 影响边界无法证明时扩展 |
| Rust 实现 | fmt/check/clippy、任务测试和受影响真实消费者；新增依赖另跑 deny | 跨 crate 公共接口、协议、状态/持久格式改变时运行 |
| 稳定产品里程碑或发布候选 | 完整采用闭包与独立审查 | 必须运行全部四条；deny 及文档/规格/skill/测试治理按实际闭包运行 |

完整工程门禁为：

```bash
cargo fmt --all -- --check
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run --all-features --no-tests=pass
```

另有 `cargo deny check`（依赖许可与漏洞）、`scripts/check-docs.sh`、`scripts/check-specs.sh`、`scripts/check-tests.sh`、`scripts/check-skill.sh` 和 `scripts/check-core-vocab.sh`。CI 根据自己的候选和环境运行完整要求；本地 pre-commit 有文件触发过滤，不能代替任务卡对间接消费者的判断。手工验证记录保存在 package validation 中，不假设钩子已经支持结果复用。

## 3. 测试：先红后绿

### 3.1 循环

后续任务按[完整行为实施指南](../docs/how-to/implement-change.md)准备接口、独立期望与真实消费者。复杂模型或同等经验作者完成高风险原语和必要测试，实施者在冻结接口上完成有界主体；由未参与准备或实现的 Reviewer 核对应行为。

缺陷先复现，新行为先核预期失败，再实现和验证；编译失败、环境拒绝与零测试不算行为红。

测试要求：

- 名称描述行为，不带任务编号；Task 注释紧贴测试属性，任务归属由工具核对。
- 每项能力有合法例与只改一个条件的拒绝例；每条强制或禁止规则均有拒绝 oracle。
- 期望来自合同、手写字节或独立计算，不调用被测代码生成同一答案。
- 每个大小／数量上限有 exact 与 +1；每个数值字段至少一次非零、非默认断言，时间差单独构造。
- 接口文档要求的值必须从参数取得；不能留给实现者在函数内猜测或重算。
- 查表、日期、进位与修正项覆盖其生效区段；必要时在可达定义域寻找判定输入。
- 重放与恢复断言原内容逐字节一致或摘要相等，不只断言文件存在。

### 3.2 分层

| 层 | 在哪 | 测什么 | 工具 |
| --- | --- | --- | --- |
| core 单元 | `crates/sheltie-core/src/**/tests.rs` 或 `tests/` | 解析、编译、`decide`、`legal_next`、状态卡渲染。输入手写，时间与 ID 固定 | 普通 `#[test]`；状态卡用 `insta` 快照 |
| runtime 集成 | `crates/sheltie-runtime/tests/` | 事务原子、重放、`REVISION_CONFLICT`、封存、路径约束、旧库拒绝 | `tempfile` 建独立 `SHELTIE_HOME` |
| CLI 端到端 | `crates/sheltie-cli/tests/` | 用 `examples/` 三份 Workbook 走完整场景；`--json` 输出可解析；退出码 | `assert_cmd` + `tempfile` |
| 崩溃 | `crates/sheltie-runtime/tests/crash.rs` | 在 `COMMIT` 前后注入失败（feature `fail-points` 或子进程 kill），重启后状态一致 | 子进程 + 临时目录 |

测试构建产物时从 `cargo build --message-format=json` 的 `executable` 取得真实路径，不猜 target 目录。

fake 只替换外部边界（时钟、ID、文件观察），不直接写「成功」进状态。

### 3.3 什么不算验证

- 「编译过了」。
- 只断言内部计数或 `is_ok()`。
- 测试和实现调用同一个 helper 算期望。
- 删掉失败的用例或把断言改成实现现在的输出。
- 重跑一次变绿的偶发失败。先找原因，不重跑到绿。

## 4. 提交

- active change plan 的一个任务 = 一个提交。提交时 §2.3 适用门禁和 package plan 的附加门禁全绿。MVP legacy task 保持原有映射。
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

- 提交后核对 `git show --stat`，范围只包含本任务的文件。

## 5. Review 检查表

审查者不参与被审代码的编写。按下表逐项核对，不适用项写原因。

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

措辞清扫不得混改冻结测试。确需改动时，由独立复核者核对应合同和测试基准，按任务计划重新冻结；MVP 的 tNN-review 方法只用于历史任务。

逐任务与里程碑审查范围由 package plan 定义；最终 review、候选 hash 和输入闭包写入 package `review.md` 与 `validation.md`；完成后按[文档维护指南](../docs/how-to/maintain-docs.md)收敛，原完成资格仍可核验。

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
| 产品该不该做某事、边界在哪 | proposed change 写问题与候选；采用后改 `spec.md`，重要选择新增 `docs/explanation/decisions/D-nnn-*.md` |
| 字段、命令、表结构、状态转换没定义 | 改对应合同 |
| 任务顺序不对、依赖缺失 | 改 active change 的 `plan.md` |
| 已定义的东西实现错了 | 直接修，补拒绝例 |
| 局部写法选择 | 自己定，不用问 |

不要为了让当前任务编译通过而临时发明第二套状态、兼容入口或「先放这里以后再改」。
