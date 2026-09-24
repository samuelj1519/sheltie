# Sheltie 架构

本文把 [产品规格](spec.md) 翻译成 crate、模块、数据类型与不变式的落点。产品语义冲突时以规格为准；机制细节以 [contracts/](contracts/) 三份合同为准。

## 1. 总览

```text
  协调者 agent（Claude Code / Codex / 人）
        │  只通过 CLI（后续加 MCP）
        ▼
┌──────────────────────── sheltie（二进制）────────────────────────┐
│ sheltie-cli      解析参数 → 调 runtime → 渲染文本或 JSON           │
│ sheltie-runtime  读状态 → 调 core 决定 → 一个事务写回 → 刷状态卡    │
│ sheltie-core     纯函数：解析、编译、状态机、合法下一步、状态卡渲染  │
└──────────────────────────────────────────────────────────────────┘
        │
        ▼
  ~/.sheltie/   store.db（唯一状态权威）  workbooks/  works/
```

三个 crate，依赖单向向下：`sheltie-cli → sheltie-runtime → sheltie-core`。

| crate | 职责 | 禁止 |
| --- | --- | --- |
| `sheltie-core` | 类型、TOML 解析、图编译、`decide` 状态机、`legal_next`、状态卡渲染 | 任何 I/O。不出现 `std::fs`、`std::time::SystemTime::now`、随机数、SQLite。时间与 ID 作为参数传入 |
| `sheltie-runtime` | 管理根目录、Workbook 仓库、SQLite 存储、产物封存、把观察到的文件事实交给 core、执行 core 返回的效果 | 业务决定。不重算 core 已决定的事；不解释自然语言 |
| `sheltie-cli` | `clap` 命令树、参数到 `Command` 的转换、文本与 `--json` 输出、退出码 | 直接碰数据库或文件 |

**为什么 core 是纯的。** 状态机是产品的全部规则所在。纯函数让每条规则都能用手写输入与期望输出做单元测试，不需要临时目录或数据库。runtime 的测试则只关心「事务是否原子、文件是否封住」。

## 2. 核心数据类型

以下是 `sheltie-core` 的公开类型骨架。字段完整定义见各合同；此处只画形状。

```rust
// 强类型 ID，禁止裸 String 在模块边界流动
pub struct WorkbookId(String);   pub struct FlowId(String);
pub struct NodeId(String);       pub struct WorkId(String);   // "2026-09-24-001-<name>"

// 定义层（装入 Workbook 后不再变）
pub struct WorkbookDef { id, version, name, flows: Vec<FlowDef>, requires: Vec<HostRequire>, digest: Sha256 }
pub struct HostRequire { kind: RequireKind, name: String, version: Option<String>, digest: Option<Sha256>, source: Option<String> }
pub enum   RequireKind { Skill, Agent, Mcp }
pub struct FlowDef     { id, entry: NodeId, nodes: Vec<NodeDef>, edges: Vec<EdgeDef> }
pub struct NodeDef     { id, title, executor: Executor, tier: Tier, instruction: Instruction,
                         inputs: Vec<InputDecl>, outputs: Vec<OutputDecl>,
                         requires: Vec<(RequireKind, String)>,
                         gate: bool, max_visits: u32, max_retries: u32 }
pub enum   Executor    { Agent, Human }
pub enum   Tier        { Strong, Standard }   // 只是给协调者的标签，引擎不据此做任何事
pub enum   Instruction { File(RelPath), Text(String) }
pub struct InputDecl   { name, from: InputSource, required: bool }   // required=false 只允许 Node 来源
pub enum   InputSource { Start { key }, Resource { path: RelPath }, Node { node: NodeId, output: String } }
pub struct OutputDecl  { name, path: RelPath, required: bool, max_bytes: u64 }
pub struct EdgeDef     { from: NodeId, to: NodeId, kind: EdgeKind }
pub enum   EdgeKind    { Main, Back, Branch, ReReview }

// 编译后的图（校验通过才存在）
pub struct Graph { nodes: BTreeMap<NodeId, NodeDef>, out_edges: BTreeMap<NodeId, Vec<EdgeDef>>, entry: NodeId }

// 运行层
pub struct WorkState {
    work_id, name, workbook: WorkbookRef, flow: FlowId,
    inputs: BTreeMap<String, ArtifactRef>,           // 起始输入已物化成文件
    status: WorkStatus,                               // Active | Blocked(reason) | Succeeded | Cancelled
    current: Option<Occurrence>,                      // 当前所在节点与第几次到达
    visits: BTreeMap<NodeId, u32>,
    attempts: Vec<Attempt>,
    approvals: Vec<Approval>,                         // gate 批准记录
    revision: u64, created_at, updated_at,
}
pub struct Occurrence { node: NodeId, n: u32 }
pub struct Attempt {
    id: AttemptId,                                    // "<node>#<n>.<retry>"
    occurrence: Occurrence, retry: u32,
    status: AttemptStatus,                            // Running | Succeeded | Failed
    entered_from: Option<(Occurrence, EdgeKind)>,     // 从哪个 Occurrence 经哪种边到达；入口为 None。任务书「来自」行的来源
    inputs: BTreeMap<String, Option<ArtifactRef>>,    // 开工时冻结；None 只出现在 required=false 且上游尚无产出
    outputs: BTreeMap<String, ArtifactRef>,           // 提交时封存
    summary: Option<BoundedText>,                     // ≤ 4 KiB
    started_at, ended_at: Option<_>,
}
pub struct ArtifactRef { path: AbsPath, sha256: Sha256, bytes: u64 }
pub struct Approval    { node: NodeId, occurrence: u32, by: Principal, at: Timestamp }
```

**类型纪律。** 非法状态不可表示：`Attempt.outputs` 在 `Running` 时为空由构造函数保证；`WorkStatus::Succeeded` 只能由 `decide` 产生。ID 用 newtype，路径用 `RelPath`（Workbook 内相对路径，禁止 `..`）与 `AbsPath` 两种类型区分。

## 3. 状态机

唯一入口是一个纯函数：

```rust
pub fn decide(state: &WorkState, graph: &Graph, cmd: &Command, ctx: &Context)
    -> Result<Decision, DomainError>;

pub struct Context  { now: Timestamp, principal: Principal }
pub struct Decision { state: WorkState, effects: Vec<Effect>, reply: Reply }
```

`Command` 是协调者能做的全部写操作的闭集：

| Command | 前提 | 结果 |
| --- | --- | --- |
| `Start { workbook, flow, name, inputs }` | 无 | 新 `WorkState`，`current = entry#1`，`status = Active` |
| `BeginAttempt { node }` | `node` 出现在当前 `next` 里 | 若 `node ≠ current.node`：按边进入，`visits[node] += 1`，`current = node#n`。新建 `Attempt(Running)`，输入按当前字节冻结 |
| `SubmitAttempt { attempt, summary, observed: Vec<ObservedFile> }` | 该 Attempt `Running` | 校验输出合同；`Attempt → Succeeded`；然后：有 `gate` → `Blocked(Gate)`；无出边 → `Succeeded`；有出边但目标都到 `max_visits` → `Blocked(NoLegalEdge)`；否则 `Active` |
| `FailAttempt { attempt, reason }` | 该 Attempt `Running` | `Attempt → Failed`；`retry == max_retries` 则 `Blocked(RetriesExhausted)` |
| `ApproveGate { node }` | `status = Blocked(Gate)` 且 `node = current.node` | 记录 `Approval`；再按 `SubmitAttempt` 除门槛外的规则定状态 |
| `Cancel` | 非终态 | `status = Cancelled` |

`observed` 是 runtime 在调用前对输出目录做的只读观察（路径、大小、sha256）。core 拿观察对照合同，不自己读文件。这就是 `INV-6` 的落点：摘要由 runtime 算，不由模型报。

### 3.1 合法下一步

```rust
pub fn legal_next(state: &WorkState, graph: &Graph) -> Vec<NextOp>;
```

按 `status` 与当前 Occurrence 的最新 Attempt 计算：

| 情形 | `next` |
| --- | --- |
| 终态 | 空 |
| `Blocked(Gate)` | `gate approve <node>`、`work cancel` |
| `Blocked(RetriesExhausted)`、`Blocked(NoLegalEdge)` | `work cancel` |
| 当前 Occurrence 无 Attempt，或最新 Attempt `Failed` 且 `retry < max_retries` | `attempt begin <current>`、`work cancel` |
| 最新 Attempt `Running` | `attempt submit`、`attempt fail`、`work cancel` |
| 最新 Attempt `Succeeded`（门槛已过或无门槛） | 对每条出边 `current → to`：若 `visits[to] < max_visits[to]` 则 `attempt begin <to>`（附 `edge.kind`）；`work cancel` |

三条硬规则在这里落地：没有边就没有 `attempt begin`（规则 1）；`BeginAttempt` 冻结输入时对照上游 `ArtifactRef.sha256`，不符即 `ARTIFACT_MODIFIED`（规则 2）；`Blocked(Gate)` 时 `next` 不含任何出边（规则 3）。

### 3.2 效果

core 不做 I/O，但会告诉 runtime 做什么：

| Effect | runtime 的动作 |
| --- | --- |
| `WriteBrief { path, content }` | 把任务书写到 Attempt 目录 |
| `SealOutputs { refs }` | 把输出文件置为只读（尽力而为；权威是记录的 sha256） |
| `RefreshStatusCard` | 用 `render_status_card` 重写 `status-card.md` |

效果在事务提交后执行。效果失败不回滚状态，只记日志；状态卡可随时从状态重生。

## 4. 一次写操作的路径

```text
CLI 解析参数
 → runtime.load(work_id)            读 WorkState + 编译好的 Graph（缓存不可变定义）
 → runtime.observe(cmd)             只读观察：输出文件、输入文件当前摘要
 → core::decide(state, graph, cmd, ctx)
 → store.commit(work_id, expected_revision, new_state, audit_row, request_id)
        单个 SQLite 事务：revision CAS + request_id 去重 + 状态 + 审计
 → runtime.apply_effects(effects)
 → CLI 渲染 reply + next
```

同一个 `request_id` 第二次到达时，`store.commit` 在事务里查到原记录：载荷相同返回原 reply，不同报 `REQUEST_CONFLICT`。core 不会被调用第二次。

`work start` 多一步：在 `observe` 之前先用一个独立的小事务分配当日序号并拼出 `work_id`（[存储合同 §7](contracts/storage.md)），之后才能建目录、写起始输入、观察、决定、提交。序号一旦分配不回收，`start` 后续失败会留下一个空号，这是接受的代价。

## 5. 目录布局

管理根默认 `~/.sheltie`，环境变量 `SHELTIE_HOME` 覆盖。只有 runtime 能写这棵树。

```text
~/.sheltie/
  bin/sheltie                       当前二进制；bin/sheltie.prev 供回滚
  tmp/                              下载与 staging，随时可删
  store.db                          SQLite，SCHEMA_VERSION = 1
  workbooks/<id>/<version>/         workbook add 复制进来的目录，只读
    workbook.toml
    flows/<flow-id>.toml
    instructions/*.md
    resources/*                     可选，用 resource.<path> 绑成节点输入
  works/<work-id>/
    workbook/                       start 时复制的冻结副本；本 Work 之后只读这里
    status-card.md                  投影，可重生成
    inputs/<key>                    起始输入物化成文件
    attempts/<node>/<n>/<retry>/    每次 Attempt 一个目录
      brief.md                      任务书（引擎写）
      <declared output paths>       工作 agent 写；提交后封存
```

产物不复制。输出文件在 Attempt 目录里原地封存，权威是 `ArtifactRef.sha256`；下游绑定时重算摘要核对。

Work 持有 Workbook 的冻结副本，`runtime.load(work_id)` 从 `works/<id>/workbook/` 编译图，不读 `workbooks/`。于是 `workbook remove` 与升级都不牵连运行中的 Work（[存储合同 §5.1](contracts/storage.md)）。

## 6. 不变式的机械落点

| 不变式 | 守在哪 |
| --- | --- |
| `INV-1` `INV-2` `INV-5` | `sheltie-core` 里禁用词 lint：`Verdict`、`Pass`、`Fail`、`Review`、`approve_by_content` 等不得作为类型或变体名。`scripts/check-core-vocab.sh` 在 CI 跑 |
| `INV-3` | `sheltie-runtime` 只写 `SHELTIE_HOME` 之下；路径进入 runtime 前经 `confine()` 检查。集成测试用只读的假 `$HOME` 证明引擎从不写它 |
| `INV-4` | 同一 lint 禁 `PackageId`、`PackageCatalog` |
| `INV-6` | `SubmitAttempt` 的摘要字段类型是 `ObservedFile`，构造函数标 `#[doc(hidden)]`，只有 runtime 的 `observe_file` 调用；CLI 参数里没有任何摘要字段 |
| `INV-7` | 只有 `store.commit` 能写 `works` 表；状态卡渲染函数只读 `WorkState` |
| 纯 core | `sheltie-core/Cargo.toml` 不依赖 `rusqlite`、`tokio`、`rand`、`chrono` 的时钟特性；`#![forbid(unsafe_code)]`；clippy 禁 `std::fs` 路径 |

## 7. 技术选型

| 需求 | 选择 | 理由 |
| --- | --- | --- |
| CLI | `clap` derive | 子命令树、自动帮助、`--json` 全局旗标 |
| 序列化 | `serde` + `toml` + `serde_json` | Workbook 用 TOML，状态与 `--json` 用 JSON。`#[serde(deny_unknown_fields)]` 实现「未知字段拒绝」 |
| 存储 | `rusqlite`（`bundled`） | 单文件、事务、零运维。WAL 模式 |
| ID | `work_id` 为 `<UTC 日期>-<当日序号>-<名字>`；`request_id` 用 `uuid` v7 | `work_id` 对人可读、目录名即 id；序号在 SQLite 事务内分配（见 [存储合同 §7](contracts/storage.md)） |
| 摘要 | `sha2` | 产物冻结 |
| 错误 | `thiserror` | 每个 crate 一个错误枚举；CLI 映射为错误码与退出码 |
| 路径 | `camino` | UTF-8 路径，避免 `OsStr` 泛滥 |
| 测试 | `insta`（状态卡快照）、`assert_cmd` + `tempfile`（CLI 端到端）、`proptest`（图编译） | 测行为，不测私有 helper |

MSRV 1.85，edition 2024，由根 `Cargo.toml` 与 `rust-toolchain.toml` 固定。

## 8. 刻意不做的设计

- **不做事件溯源。** Work 很小，整份 `WorkState` 作为 JSON 存一列，配 `revision` 做乐观并发。审计表另存每次 Command。读写都简单，恢复只需读一行。
- **不做异步。** 单用户本地 CLI，一次调用一个事务，`tokio` 只会加复杂度。
- **不做插件系统。** 执行方式只有 `agent | human` 两种字面量；宿主差异由 skill 文本与 CLI 参数吸收。
- **不做通用表达式路由。** 协调者选边，引擎不算条件。需要机器路由时再按路线图立项。
- **不拆独立安装器。** `self` 组管二进制，`workbook` 组管方法，两者都只写 `~/.sheltie`，不碰宿主，`INV-3` 不受影响。`INV-3` 真正要防的是把 skill、subagent 装进 Claude Code 之类的宿主，那部分代码将来放独立 crate 与二进制（[路线图 GF-20](roadmap.md)），`self` 与 `workbook` 留在 `sheltie` 里不动。
- **宿主资源只声明不打包。** 能让工作 agent「读文件」解决的都放 `resources/` 绑成输入。真要宿主机制的写进 `requires`，引擎列进任务书，不检查、不安装。

## 9. 术语

见根目录 [CONTEXT.md](../CONTEXT.md)。
