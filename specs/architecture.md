# Sheltie 架构

本文把产品规格落到 crate、模块、数据类型与不变式上。产品语义冲突时以规格为准；机制细节以 [contracts/](contracts/) 三份合同为准。

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
| `sheltie-core` | 类型、TOML 解析、图编译、`decide` 状态机、`legal_next`、状态卡渲染 | 任何 I/O。不出现 `std::fs`、`std::time::SystemTime::now`、随机数、SQLite。时间与 ID 从参数传进来 |
| `sheltie-runtime` | 管理根目录、受管文件句柄、Workbook 仓库、SQLite 存储、产物封存、把观察到的文件事实交给 core、执行 core 返回的效果 | 业务决定。不重算 core 已决定的事；不解释自然语言 |
| `sheltie-cli` | `clap` 命令树、参数到 `Command` 的转换、文本与 `--json` 输出、退出码 | 直接碰数据库或文件 |

**为什么 core 是纯的。** 状态机是产品的全部规则所在。纯函数让每条规则都能用手写输入与期望输出做单元测试，不需要临时目录或数据库。runtime 的测试则只关心「事务是否原子、文件是否封住」。

## 2. 核心数据类型

以下是 `sheltie-core` 的公开类型骨架。字段完整定义见各合同；此处只列轮廓。

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
pub enum   InputSource { Start { key }, Resource { path: RelPath }, EngineStats, Node { node: NodeId, output: String } }
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
    blocked_count: u32,                               // 累计受阻事实，由状态转换递增（GF-29）
    created_at, updated_at,
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

**类型纪律。** ID 用 newtype，路径用 `RelPath`（Workbook 内相对路径，禁止 `..`）与 `AbsPath` 两种类型区分。已校验定义只经解析、编译构造，字段只读。`WorkState` 是可序列化的事实数据，公开字段不保证全部跨字段组合合法；生产转换由 `decide` 生成，Store 装入时经 `validate_persisted` 核关键状态组合，再结合冻结 Graph 校 gate 与路径归属。`Running` 的输出为空等规则由转换与装入校验共同保证，不声称全部非法状态在类型层不可表示。

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

core 另外提供两个纯函数作为 start 的共享事实源（GF-30）：`start_requirements(graph)` 返回图里全部 `start.<key>` 键（按节点声明顺序首次出现），`validate_start_inputs(graph, keys)` 校验给出的键集合恰好相等。decide、runtime preflight 与 `workbook show` 用同一份结果，不各自重算。

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
| `WriteFile { path, content }` | 写引擎生成的输入文件（`engine.stats` 的 `stats.json`）。内容与摘要在 core 里已定 |
| `SealOutputs { refs }` | 把输出文件置为只读（尽力而为；以记录的 sha256 为准） |
| `RefreshStatusCard` | 从最新状态重写 `status-card.md`；失败保留请求效果未完成标记 |

效果在事务提交后执行。当前请求自己的效果失败不回滚状态，返回 `EFFECT_PENDING`（`committed = true`，带原响应）；旧效果阻断新请求时，新请求尚未提交，返回 `EFFECT_PENDING`（`committed = false`，带阻断请求 id），按协议 §5 处理。下一次写操作先完成未发布效果。状态卡始终从最新状态重新生成。

runtime 另有**目录效果**，不是 core Effect：`publish_dir`（把 `pending/<内部 id>/payload/` 原子改名为最终目录）、`prepare_attempt`（在已提交 Attempt 下安全建 `engine/`、`outputs/` 与输出父目录）、`delete_dir`（把已核归属的目录移入本操作自己的 pending 再删除）。这些效果随请求登记进 `requests.effects_json`，带完成标记；崩溃后由下一次写操作在同一管理根写锁内恢复（[存储合同 §3.2](contracts/storage.md)）。文件级效果的精确字节同样登记，保证历史任务书与 `engine/stats.json` 按提交时字节恢复，而不是从最新状态重算。

### 3.3 接续与成果投影

core 的 `status_view`/`StatusCardJson` 提供当前 Occurrence 最新 Attempt 的 resume：任务书路径、已冻结 inputs、仅 running 的声明草稿位置。文本卡与 JSON 使用相同组装来源；卡片不保存实时 I/O 就绪事实。

core 的 `result_view(state, graph, revision, effects_pending)` 从合法成功终点的具体 Attempt.inputs/outputs 选择 result 项。有效无选择为空，成功状态与终点/必需引用矛盾拒绝；revision 由 runtime 参数给出，不从 WorkState 推断。ResultView 不缓存到 Store，不构成第二事实源。

runtime 的 `Store::read_work_bundle` 在单一只读事务取得 WorkRow、完整关联 requests/audit/效果与 Start 定位；已有 bundle 的可信加载不跨连接回查。公开 StatusReadView 将状态投影与 revision/effects_pending/pending_publish 合成一个读取 DTO；WorkService::result 返回同一上下文的 ResultView 与 next。真实文件字节不由普通结果查询重新认证。精确字段见 protocol/storage。

## 4. 一次写操作的流程

写操作分三段：只读预检、持锁执行、提交后发布（[存储合同 §2](contracts/storage.md)）。`work start`的确定性输入拒绝在创建Home/锁和业务目录之前完成；`workbook add`只在锁前做源结构、类型和限额粗检，内容解析、编译与摘要在锁内私有副本进行。副本内容失败可保留控制Store/锁及自有暂存，业务行、请求、审计与最终Workbook不得出现。

```text
CLI 解析参数（只解析 @file 路径，不读内容）
 → 只读预检：识别已有 schema、解析目标身份、构造 RequestIntent，先按 request_id
   查可重放请求；未命中的Work写操作才读 @file、装入 Workbook/Flow 并完成确定性校验；
   Workbook add此处仅做源结构/类型/限额粗检。
   @file 读取失败退出码 2。不建目录、不建锁、不写 PRAGMA。
 → 取得管理根写锁（合法写操作才创建管理根与 .lock；只读命令永远不碰）
 → 锁内：
     重核 schema、request_id 与受并发影响的前置事实
     恢复未完成的发布效果（先于一切新命令）
     runtime.load(work_id) / 解析 Workbook
     runtime.observe(cmd)             只读观察：输出文件、输入文件当前摘要
     core::decide(state, graph, cmd, ctx)
     store.commit(...)                单个 SQLite 事务：request 去重 + revision CAS + 状态 + 审计 + 效果登记
     发布效果（rename pending/payload → 最终目录、写 brief/stats、封存、刷新状态卡）并标记完成
 → CLI 渲染 ResponseSnapshot + next
```

同一个 `request_id` 在预检或事务内命中已提交记录：意图相同先完成未发布效果，再返回原 `ResponseSnapshot`（`replayed = true`）；不同报 `REQUEST_CONFLICT`。这两种情形都不再调用 core，也不读当前 Workbook、源目录、`@file` 或输出文件来重新决定业务。CLI 不再在提交后回读 Store 拼响应——响应字段全部来自提交时快照。

`work start` 的当日序号在锁内、目录物化之前用独立小事务分配（[存储合同 §7](contracts/storage.md)）。序号一旦分配不回收，`start` 在序号之后的失败会留下一个空号，这是接受的代价；确定性拒绝（缺输入、非法名字、缺 Workbook/Flow）发生在分配之前，不烧号（GF-30）。

## 5. 目录布局

管理根默认 `~/.sheltie`，环境变量 `SHELTIE_HOME` 覆盖。入口通过 `Home::resolve` 将最深已存在祖先解析为真实路径，再拼接不存在的尾部；只有 `NotFound` 可向父目录退让，权限及其他 I/O 错误必须传播。无法表示为 UTF-8 的配置、当前目录或真实祖先路径明确拒绝，不替换字节或改选其他根。只有 runtime 能写这棵树。所有 managed 路径（Store、锁、workbooks、works、pending、tmp、bin 及恢复与删除目标）都从根派生并经 `confine` 检查。

```text
~/.sheltie/
  store.db                          SQLite，SCHEMA_VERSION = 3（schema 1/2 明确拒绝）
  .lock                             管理根写锁（写操作创建；只读命令不碰）
  bin/sheltie                       当前二进制；bin/sheltie.prev 供回滚
  tmp/                              下载与解包等一次性暂存，可按年龄清理
  pending/<内部 id>.owner           先创建的引擎归属侧车；无 Store 引用时用于安全清理
  pending/<内部 id>.deleted         Workbook 删除完成的持久标记
  pending/<内部 id>/payload/        发布暂存：未提交准备区、已提交未发布原件、待删除目录。
                                     永不按年龄清理；只按归属与 Store 引用恢复或清理
  workbooks/<id>/<version>/         workbook add 复制进来的目录，只读（含根）
    workbook.toml
    flows/<flow-id>.toml
    instructions/*.md
    resources/*                     可选，用 resource.<path> 绑成节点输入
  works/<work-id>/
    status-card.md                  投影，从最新状态生成
    workbook/                       start 时复制的冻结副本；本 Work 之后只读这里
    start-inputs/<key>              起始输入物化成文件
    attempts/<node>/occurrence-001/attempt-000/
      brief.md                      任务书（引擎写）
      engine/stats.json             engine.stats 输入（引擎写）
      outputs/<declared-path>       工作 agent 写；提交后封存
```

路径由 core 的单一 `WorkLayout` 函数生成：`AttemptId = node#n.retry` 保持原含义，目录标签 `occurrence-001` / `attempt-000` 只是零补齐的浏览形式。引擎文件（`brief.md`、`engine/stats.json`）与 worker 输出（`outputs/` 之下）分目录，输出声明路径不再与引擎文件比较。

产物不复制。输出文件在 Attempt 的 `outputs/` 下原地封存，以 `ArtifactRef.sha256` 为准；下游绑定时重算摘要核对。

Work 持有 Workbook 的冻结副本，`runtime.load(work_id)` 从 `works/<id>/workbook/` 编译图，不读 `workbooks/`。于是 `workbook remove` 与升级都不牵连运行中的 Work（[存储合同 §5.4](contracts/storage.md)）。

## 6. 不变式的机械落点

| 不变式 | 守在哪 |
| --- | --- |
| `INV-1` `INV-2` `INV-5` | `sheltie-core` 里禁用词 lint：`Verdict`、`Pass`、`Fail`、`Review`、`approve_by_content` 等不得用作类型或变体名。`scripts/check-core-vocab.sh` 在 CI 跑 |
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
| 摘要 | `sha2` | 产物冻结与 `workbook-digest/v2` |
| 受管文件调用 | `rustix` 1.1.4 安全文件系统调用（仅 runtime；`unsafe_code = forbid`） | 目录句柄锚定、拒绝路径跟随软链、核文件对象身份；具体契约见 storage §6 与 D-037 |
| 管理根写锁 | `fs4`（std 文件的排他锁） | 文件生命周期串行化；进程退出由 OS 释放（D-035） |
| OS 主体 | `uzers`（effective uid + 账户查询，仅 unix） | 安全 Rust API 取得进程身份，不采信 `USER`/`USERNAME`（D-036） |
| 错误 | `thiserror` | 每个 crate 一个错误枚举；CLI 映射为错误码与退出码 |
| 路径 | `camino` | UTF-8 路径，避免 `OsStr` 泛滥 |
| 测试 | `insta`（状态卡快照）、`assert_cmd` + `tempfile`（CLI 端到端）、`proptest`（图编译） | 测行为，不测私有 helper |

MSRV 1.85，edition 2024，由根 `Cargo.toml` 与 `rust-toolchain.toml` 固定。

## 8. 刻意不做的设计

- **不做事件溯源。** Work 很小，整份 `WorkState` 存成一列 JSON，配 `revision` 做乐观并发。审计表另存每次 Command。读写都简单，恢复只需读一行。
- **不做异步。** 单用户本地 CLI，一次调用一个事务，`tokio` 只会加复杂度。
- **不做插件系统。** 执行方式只有 `agent | human` 两种字面量；宿主差异由 skill 文本与 CLI 参数吸收。
- **不做通用表达式路由。** 协调者选边，引擎不算条件。需要机器路由时再按路线图立项。
- **不做自适应。** 引擎不按统计调 `max_visits`、换 `tier`、跳节点。它只把统计整理成 `work stats` 给 Workbook 的反思节点读，改进由人冻结新版本。同一版本每次运行行为相同，审计才成立。
- **不拆独立安装器。** `self` 组管二进制，`workbook` 组管方法，两者都只写 `~/.sheltie`，不碰宿主，`INV-3` 不受影响。`INV-3` 真正要防的是把 skill、subagent 装进 Claude Code 之类的宿主，那部分代码将来放独立 crate 与二进制（[路线图 GF-20](roadmap.md)），`self` 与 `workbook` 留在 `sheltie` 里不动。
- **宿主资源只声明不打包。** 能让工作 agent「读文件」解决的都放 `resources/` 绑成输入。真要宿主机制的写进 `requires`，引擎列进任务书，不检查、不安装。

## 9. 术语

见根目录 [CONTEXT.md](../CONTEXT.md)。
