# C002 设计

状态：`proposed`，产品实现 `not_run`

本文只定义候选 interface、顺序和兼容策略。人采用 C002 并同步根规格、架构、合同与 ADR 后，设计才成为实施依据。

## 设计决定

### StartRequirements 是起始输入的唯一事实来源

在 core 中提供一个纯 interface：

```text
start_requirements(graph) -> 有序 key 集合
validate_start_inputs(graph, provided_keys) -> Result
```

`decide_start`、runtime preflight 与 `workbook show` 都调用同一实现。不要在 CLI、runtime 和 core 各写一遍收集 `start.<key>` 的循环。

`WorkService::start` 的新顺序：

```text
构造 RequestIntent
→ 请求重放预检
→ 加载并核对已装 Workbook
→ 找 Flow
→ 规范化 WorkName
→ validate_start_inputs
→ 分配序号
→ 在 tmp/starts/<request-id>/ 物化 Workbook 与 start inputs
→ core decide
→ 单事务写 Work、audit、StoredOutcome
→ rename staging 到 works/<work-id>
→ 写 status-card
```

前六步以前失败必须“无变化”。分配序号之后发生 I/O 故障或进程崩溃，可以留下不回收的空号；中间文件只能在 `tmp/`，不能出现在 `works/`。COMMIT 后、rename 前崩溃时，同 request-id 重放完成 rename。

### WorkLayout 集中管理全部路径

新增 core module `work::layout`。它是 Work 路径的唯一 interface，负责：

- 冻结 Workbook 路径。
- 起始输入路径。
- Occurrence 与 Attempt 目录。
- `brief.md`、引擎文件、worker output 目录。
- `status-card.md`。

新 Work 使用 `readable_v2`：

```text
works/<work-id>/
  status-card.md
  workbook/
  start-inputs/
    <key>
  attempts/
    <node>/
      occurrence-001/
        attempt-000/
          brief.md
          engine/
            stats.json
          outputs/
            <声明路径>
```

两层目录保留，因为 `Occurrence` 表示 Node 再次到达，`Attempt` 表示同一次到达内的执行与重试。数字补零只用于目录排序；CLI AttemptId 保持 `node#occurrence.retry`。

`WorkState` 新增 layout version。旧 state 缺字段时默认 `legacy_v1`，所以已有 Work 不重命名、不迁移；新 Work 固定使用 `readable_v2`。core、runtime、渲染、Workbook 说明和测试不得再手工拼目录字面量。

兼容只覆盖升级前仍完整的 Work。已经因 `.DS_Store` 或人工修改发生摘要不符的冻结副本不能自动修复；修复工具不知道哪组字节才是原件，必须继续报错并交给人处理。

### 冻结目录的权限与摘要各管一层

`set_tree_readonly` 必须把传入根目录设为 `0555`，子目录 `0555`，文件 `0444`。rename 一个目录只要求父目录可写；删除前使用既有 `make_tree_writable` 恢复权限。

Workbook source 含 `.DS_Store`、`Thumbs.db` 等已知宿主元数据时，`workbook add` 明确拒绝并指出路径，不静默忽略。安装目录与冻结副本根只读，防止 Finder 后续写入；目录摘要继续检测显式篡改。

已装 Workbook 只能通过 `WorkbookRepo::load_installed` 读取。它核对数据库行、目录摘要、manifest id/version 和 Flow。`load_dir` 只用于 add 的源目录并降为 crate-private。

### 请求生命周期保存响应和可恢复效果

runtime 新增内部 module `request`：

```text
RequestIntent      用户提交的操作、目标与参数；不含观察结果
ResponseSnapshot   提交时返回给 CLI 的完整事实；不含 replayed
DurableEffect      可逐字节恢复的引擎效果
StoredOutcome      stored-outcome/v1 = snapshot + effects
```

请求命中必须发生在加载 Work、读取 Workbook、观察输出之前。目标 Work 是 intent 的一部分；submit intent 使用 `work_id + attempt_id + summary`，不使用重新观察到的输出摘要。

brief、engine/stats、status-card 的 exact bytes 在 COMMIT 前物化并写入 `StoredOutcome`。重放只能补做保存的效果，不能用当前 WorkState 重算历史文件。worker 输出正文不进 SQLite，WorkState 仍是唯一可推进状态。

沿用现有 `requests.reply_json` 物理列，新记录写版本化封装，不升 `SCHEMA_VERSION`。旧请求只有在能证明相同且效果文件完整时才重放；不能证明时返回明确错误，不猜。

`request_id` 只覆盖写 `store.db` 的业务命令。`self` 命令收到 request-id 时参数错误；它们继续按各自文件合同实现幂等。

### Workbook 摘要保持 v1 兼容

`v0.1.0` 已持久化当前两阶段目录摘要。`v0.2.0` 把它正式命名为 `workbook-digest/v1`，补独立测试向量，不改变已有字节。未来改算法时必须增加算法版本与迁移，不能复用同一字段静默换含义。

### T26 是已完成的历史运行，后续版本另做回归

保留 `31d7dde` 作为 MVP 与 T26 完成记录，不回滚其状态。修复后的 rc 新增独立宿主回归任务，补充：

- 人自己执行 human 节点任务书。
- 取得 `/cost` 前后值或等效宿主 token 读数。
- 保存逐条命令与 response 的证据位置。

历史记录中互斥的开场 prompt 事实按原始 transcript 修正；取不到 transcript 时明确写证据缺失。
