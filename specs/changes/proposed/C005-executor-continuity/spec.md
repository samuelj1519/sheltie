# C005 产品方案

状态：`proposed`；实现与实验均为 `not_run`。本文件定义候选行为，采用后先同步上游合同。

## 1. 何时继续，何时替换

| 场景 | 操作 | 结果 |
| --- | --- | --- |
| 原会话恢复、协调者重开、人工等待后继续 | 读取 `work handoff`，继续原 Attempt | 不改状态，不消耗任何次数 |
| 新执行者接手，旧执行者已确认停止且不需撤销提交资格 | 读取交接视图，继续原 Attempt | 原任务书和提交资格保持有效 |
| 需要撤销旧 Attempt 的提交资格 | `attempt replace` | 旧尝试终止，新尝试在同一 Occurrence 开始 |
| 任务执行失败 | `attempt fail` | 按 `max_retries` 处理 |
| 候选需返工 | Workbook 的显式回边 | 按 `max_visits` 处理 |

操作者判断是否需要替换。引擎不从额度错误、自然语言回复或宿主名推断。替换理由是调用者报告；引擎确立的事实只有「这个请求替换了这个 Attempt」。

## 2. 候选命令与返回

```text
sheltie attempt replace <work> --attempt <旧 AttemptId> --reason <文本或 @file>
```

全局 `--request-id` 规则与其他写命令相同。时间由 runtime 的系统时钟提供，主体由 runtime 的真实 OS 进程身份取得；均不接受调用者注入。

`EX-01` **原子替换。** 只接受活动 Work 当前 Occurrence 的最新 `running` Attempt。一次事务内将它记为 `superseded`，登记替换理由与结束时间，创建同一 Occurrence 的新 `running` Attempt。响应含 `replaced_attempt`、新 `attempt`、`number`、`brief_path`、`output_dir`、输入输出路径、`requires` 和提交时的 `next`。不得分成「先撤销、再 begin」两次写操作。C004 的原生执行记录若仍为 open，同一事务将其关闭为 interrupted；completed 记录原样保留并归属旧 Attempt，新 Attempt 不继承旧记录的完成资格。旧 runner 的迟到完成通过 execution_id/状态 CAS 拒绝。

`EX-02` **保持工作依据。** 新 Attempt 继承旧 Attempt 的 `entered_from` 和冻结输入引用，开始前核对这些输入的摘要。`engine.stats` 是引擎投影，以包含新 Attempt 的提交后状态生成；它的输入引用和精确字节随此次请求保存。新任务书使用本 Work 冻结说明书，并链接 C004 的交接视图。旧未提交输出没有 ArtifactRef，只提供草稿路径和未封存标记，不自动提升为新 Attempt 的正式输入或输出。

`EX-03` **撤销正式提交资格。** 旧 Attempt 保留原输入、时间和历史请求，其状态为 `superseded`。旧 `submit`、`fail` 返回 `ATTEMPT_NOT_RUNNING`。旧 Attempt 的成功 `begin` 请求仍可历史重放，返回历史快照；返回的 `next` 不代表当前合法操作，继续前查询当前状态。

## 3. 三种计数

`EX-04` **顺序号。** Attempt 的显示形状为 `node#occurrence.number`。后缀从 0 开始，每次创建新 Attempt 加 1，包括失败后的重试与替换。字段称为 `number`，不再用 `retry` 表示这个后缀。加法采用检查运算；溢出拒绝请求且不改变状态。

`EX-05` **业务失败。** 在同一 Occurrence 中直接统计 `failed` Attempt 数量。`max_retries = k` 允许第一个尝试失败后再有 k 次业务重试；第 `k + 1` 次真实失败按已有合同进入 `blocked(retries_exhausted)`。`superseded` 不计入失败；顺序号大于 `max_retries` 也不会独立触发阻塞。上游返工次数仍由 `max_visits` 定义。

`EX-06` **替换上限。** Node 候选字段 `max_replacements` 为整数，范围 `0..=32`，默认 `0`，随 Workbook 冻结。0 表示关闭替换；32 是首版有界合同的候选上限，不代表观测数据。已替换次数直接统计同一 Occurrence 的 `superseded` Attempt，不另存计数。若已替换数等于上限，再次替换返回 `REPLACEMENTS_EXHAUSTED`，整个请求无状态改变，旧 Attempt 继续 `running`。它的收集/提交资格继续遵守 C004 的原生观察规则：无记录可收集，open 时不能 submit，完整报告发布后可 submit；fail/cancel仍可用，不进入新阻塞状态。上限不能通过重新开始进程、换 request-id 或切模型重置。

## 4. 合法操作与失败

| 前置条件 | `replace` 的结果 | Work / Attempt 的变化 |
| --- | --- | --- |
| 当前最新尝试 running，替换数小于上限，输入核对成功 | 成功，返回新任务书 | 同一 revision 同时旧 superseded、新 running |
| 同一 request-id、相同参数已成功 | 重放原响应 | 无新增 Attempt、审计或 revision |
| 同一 request-id、目标或理由参数不同 | `REQUEST_CONFLICT` | 无变化 |
| 旧尝试已 succeeded / failed / superseded | `ATTEMPT_NOT_RUNNING` | 无变化 |
| 给出的 running 尝试不属于当前 Occurrence | 非法操作错误 | 无变化 |
| Work 已 blocked / succeeded / cancelled | 现有状态对应的非法操作或终态错误 | 无变化 |
| 替换数达到上限 | `REPLACEMENTS_EXHAUSTED` | 旧尝试仍 running |
| 冻结输入缺失、变化或路径不安全 | 现有输入/完整性错误 | 无变化 |
| COMMIT 后任务书发布失败 | `EFFECT_PENDING`，`committed = true` | 替换已提交；用同 request-id 恢复 |

`next` 仅在活动 Work 的当前 running Attempt 且上限未耗尽时列出 `attempt replace`，并携带该 AttemptId。其他操作仍遵守 C004 的观察规则：无记录仍给CollectOutput和candidate待填参数，open时不重新开放submit，完整报告发布后才给submit；replace只追加自身操作，不重算或覆盖C004这组资格。查询仍提供交接与结果视图；它们是只读入口，不进入推进操作集合。

## 5. 保证与责任

- 引擎保证正常接口不会接受已被替换 Attempt 的新提交。它不停止旧进程、不阻止同权限程序直接修改文件或 Store，也不撤销已发生的宿主副作用。
- 旧执行者是否停止、是否共享工作区，由操作者检查。无法确认停止时，先停止旧进程或准备隔离工作区，再让新执行者工作；Sheltie 不代办这些宿主动作。
- 输入与标准不因换人放宽。审查者独立性由 Workbook 和协调者安排，不新增身份资格系统或审查发现账本。
- 人工暂停不需要写专用状态。Work 在没有命令时保持原状态；不会因等待太久被引擎转换为只能取消。

## 6. 采用时同步的上游

| 文件 | 必须同步的内容 |
| --- | --- |
| `CONTEXT.md`、`specs/spec.md` | Attempt 的 superseded 状态、顺序号定义、显式替换与上限；不新增资源等待承诺 |
| `specs/architecture.md` | 状态转换、原子替换、合法操作、统计和交接投影 |
| `specs/contracts/workbook.md` | Node `max_replacements` 的类型、范围、默认值与冻结规则 |
| `specs/contracts/protocol.md` | replace 参数、成功载荷、next 项、错误和历史响应口径 |
| `specs/contracts/storage.md` | 新状态字段、请求意图、快照绑定、效果恢复与格式版本 |
| `skills/sheltie/SKILL.md` | 继续与替换的选择、共享工作区限制、历史 next 使用规则 |

格式版本由采用起点一次确定。旧 Store 整体只读拒绝，不自动迁移、不清空；拒绝前不能改业务数据。已有数据保留供原版本读取。新 Workbook 字段省略即关闭替换，普通流程不增加动作。
