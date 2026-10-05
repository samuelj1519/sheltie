# 状态与成果参考

术语定义见 [CONTEXT](../../CONTEXT.md)。本页按运行对象、状态查询与成果组织事实；精确持久结构见[存储合同](../../specs/contracts/storage.md)，响应字段见[协议合同](../../specs/contracts/protocol.md)。

## 对象关系

| 对象 | 关系或身份 | 实现入口 |
| --- | --- | --- |
| `WorkbookDef`、`FlowDef`、`NodeDef` | 已校验的方法定义；编译为 `Graph` | [workbook](../../crates/sheltie-core/src/workbook)、[flow](../../crates/sheltie-core/src/flow) |
| `WorkState` | 冻结一个方法版本的一次运行；带 revision、当前到达和历史执行事实 | [work/state.rs](../../crates/sheltie-core/src/work/state.rs) |
| `Occurrence` | Node 在本 Work 内第 n 次到达；形如 `review#2` | work/state.rs |
| `Attempt` | 一次到达内的执行记录；形如 `review#2.0` | work/state.rs |
| `Command`、`Decision`、`Reply`、`Effect` | 已观察命令、纯决策、回复及待执行文件效果 | [work/command.rs](../../crates/sheltie-core/src/work/command.rs)、[decide.rs](../../crates/sheltie-core/src/work/decide.rs) |
| `ArtifactRef` | 已冻结文件的绝对路径、sha256 和 bytes | work/state.rs |
| `WorkReadBundle`、`StatusReadView` | 单一读取快照及其状态投影 | [store/read.rs](../../crates/sheltie-runtime/src/store/read.rs)、[runtime/result.rs](../../crates/sheltie-runtime/src/result.rs) |
| `ResultView` | 成功终点明确选择的冻结引用集合 | [work/result.rs](../../crates/sheltie-core/src/work/result.rs) |

Occurrence 从 1 开始，Attempt 的 `number` 从 0 连续递增。执行失败后重试仍在同一次到达内；沿边重新到达 Node 才新增 Occurrence。行政替换增加 number，但不增加真实失败数。实际 ID 以响应为准。

## Work 状态与合法操作

JSON 状态为对象，例如 `{"kind":"active"}`、`{"kind":"blocked","reason":"gate"}`；文本状态卡显示 `active`、`blocked(gate)`。

| 状态 | 含义 | 合法操作范围 |
| --- | --- | --- |
| `active` | 可以继续当前运行 | 无 running Attempt 时按 `next` 领取节点；running 时可提交、真实 fail、额度内 replace；可取消 |
| `blocked(gate)` | 成功提交后等待门槛批准 | 当前 Node 的 `gate approve` 或取消 |
| `blocked(retries_exhausted)` | 同一次到达的真实执行失败次数耗尽 | 只剩取消；不能提高冻结额度继续 |
| `blocked(no_legal_edge)` | 所有出边目标的到达额度耗尽 | 只剩取消；不能补边或跳节点 |
| `succeeded` | 无出边终点成功，所需门槛已批准 | 终态；`next=[]`，仍可查询 |
| `cancelled` | 已取消运行 | 终态；`next=[]`，仍可查询；不停止宿主进程 |

表格是查询摘要，调用资格以最新 `work status` 的 `next` 为准。`next` 中的 `executor` 和 `tier` 用于派活；标签不执行模型调用或宿主安装。

## Attempt 状态

| 状态 | 含义与文件边界 |
| --- | --- |
| `running` | 执行中；输出位置是草稿，文件可能还不存在 |
| `succeeded` | 输出合同已通过并登记封存；内容质量由协调者判断 |
| `failed` | 实际执行失败；计入 `max_retries` |
| `superseded` | 旧正式提交资格已撤销；保留旧草稿事实，不计业务失败 |

每个 Occurrence 最多替换一次。新 Attempt 继承冻结的非统计输入，不接管旧草稿；统计按新提交状态生成。取消 Work 不等于把 running Attempt 标为 failed，也不授予继续提交的资格。

## `status`、`resume` 与统计

`work status` 的 `data` 含 `work_id`、`status`、`current`、`done`、`resume`、`revision`、`effects_pending` 和 `pending_publish` 等字段，顶层 `next` 来自同一快照。完整状态卡见[协议 §6](../../specs/contracts/protocol.md#6-状态卡-status-cardmd)。

`resume=null` 表示没有当前 Attempt；非空包含当前 Attempt 身份、任务书、冻结输入与 `draft_outputs`。`resume.inputs` 按名称保存 ArtifactRef 或 null，读取文件时取其 `path`；`draft_outputs` 按名称给草稿路径。它们不同于 begin 响应中直接给绝对路径的 `data.inputs/outputs`。非空不保证 Attempt running。草稿位置只在 running 时列出，也不保证文件存在。接续前同时核 Work 状态、Attempt 身份与当前操作资格。

`effects_pending` 表示相关已提交请求有未完成文件效果；`pending_publish` 表示 Work 发布尚待完成。只读查询不恢复它们。磁盘 `status-card.md` 是写操作生成的投影，不能替代新的状态查询。

`work stats` 按 Node 列出 `visits`、`attempts`、`failed`、`superseded`、`avg_seconds` 与边来源计数，并给出 Work 汇总。耗时来自已结束 Attempt 的记录，包含人工与等待；不是模型 usage 或费用。完整字段见[统计合同](../../specs/contracts/protocol.md#work-stats-work)。

## 最终成果

`work result` 的 `data.format=work-result/v1`；状态、revision、效果事实与选集来自同一读取快照。

| 结果 | 解释 |
| --- | --- |
| `final=false`、`artifacts=[]` | 尚未成为合格成功终点，或文件效果未完成 |
| `final=true`、`artifacts=[]` | 成功终点没有声明成果选择 |
| `final=true`、非空 `artifacts` | 列出该终点明确选择的冻结输入或封存输出 |

每项含 `key`、`path`、`sha256`、`bytes`、`source`。`source.attempt` 是进行绑定或封存的成功终点 Attempt；`source.kind` 为 `input` 或 `output`，`source.name` 为该槽的逻辑名。输入文件的生产者可以是更早的 Attempt，不能把选择方误当原生产者。

引用查询不重新认证全部源 bytes，也不验证报告中的代码候选或检查结果。真正读取或复制使用同一次查询的 revision 与 literal key，核最终退出码、bytes 与 sha256，见[导出参考](export.md)。
