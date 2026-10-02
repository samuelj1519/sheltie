# C005 产品方案

状态：`proposed`。本文件定义候选行为，采用后先更新上游权威。实施、测试与真实使用为 `not_run`。

## 1. 使用场景

| 场景 | 操作 | 引擎变化 |
| --- | --- | --- |
| 重开会话、等待后继续、已确认旧执行者停止后换人 | 读取当前 `work status` 与任务书，继续原 Attempt | 无 |
| 必须撤销旧 Attempt 的正式提交资格 | `attempt replace` | 原子结束旧尝试并开始新尝试 |
| 执行确实失败 | `attempt fail` | 统计真实失败，按 `max_retries` 处理 |
| 产物需要返工 | Workbook 显式回边 | 按 `max_visits` 到达下一 Occurrence |

操作者判断是否撤销资格。替换理由属于调用者报告；引擎只确立替换请求和状态转换事实。不同执行者可以使用同一 Attempt，因此 Attempt 不表示已认证的执行者身份。

## 2. 命令与响应

```text
sheltie attempt replace <work> --attempt <旧 AttemptId> --reason <文本或 @file>
```

全局 `--request-id` 沿用写命令的幂等规则。时间和操作者身份来自 runtime 系统边界，不接受 CLI 注入。理由最多 4096 字节，超出报 `SUMMARY_TOO_LONG`；文件参数的意图与重放按公开写命令合同处理。

成功的持久响应包含 `replaced_attempt`、新 `attempt`、`node`、`occurrence`、`number`、`brief_path`、`output_dir`、`inputs`、`outputs`、`requires`、`revision` 和提交时 `next`。路径与声明来源和 `attempt begin` 一致。CLI 不在提交后回读当前状态补造历史响应。

## 3. 状态与计数合同

| ID | 规则 |
| --- | --- |
| EX-01 | 只接受 active Work 当前 Occurrence 的最新 `running` Attempt。在一次事务中将旧 Attempt 结束为 `superseded`，记录理由与结束时间，并追加一个新 `running` Attempt。不能拆成两次写操作。 |
| EX-02 | 同一 Occurrence 最多存在一个 `superseded` Attempt。已有替换事实时，新替换返回 `REPLACEMENTS_EXHAUSTED`，不改变业务状态；当前 Attempt 保持原资格。每个新 Occurrence 有自己的固定一次额度。 |
| EX-03 | 新 Attempt 继承旧 `entered_from` 和冻结输入引用，包括原本未绑定的可选输入。非 `engine.stats` 输入在提交前核对实际摘要和大小。`engine.stats` 从含新 Attempt 的提交后状态生成，精确字节随请求保存。 |
| EX-04 | 旧未提交输出仍是未封存草稿，不成为新 Attempt 的正式输入或输出。旧任务书和原请求保留。新任务书使用 Work 的冻结说明书。 |
| EX-05 | Attempt 显示为 `node#occurrence.number`。后缀从 0 起，每次创建新 Attempt 加 1；它是顺序号，不是失败数。加法溢出拒绝且不改变业务状态。 |
| EX-06 | 业务失败直接统计同一 Occurrence 的 `failed` Attempt。`max_retries=k` 允许 k 次业务重试；第 k+1 次真实失败进入 `blocked(retries_exhausted)`。`superseded` 不计失败；顺序号不能决定重试资格或历史 fail 状态。 |
| EX-07 | 非终态 Work 中旧 superseded Attempt 的新 `submit`、`fail` 返回 `ATTEMPT_NOT_RUNNING`；Work 已终态时按前置顺序返回 `WORK_TERMINAL`。旧成功请求仍按历史快照重放，其 `next` 只代表当时状态；继续操作前查询当前 `work status`。 |
| EX-08 | 替换不跳过 Node、修改输入、增加失败、自动批准门槛或改变成果标准。等待不产生新状态。 |

一个固定替换机会用于行政撤销，业务失败仍按已有 Workbook 重试规则处理。固定额度不需要 Workbook 配置；替换事实由 Attempt 历史统计。

## 4. 合法操作与失败

| 条件 | 结果 | 业务状态 |
| --- | --- | --- |
| active、当前最新 running、没有替换事实、输入完整 | 成功返回新任务书 | 旧 superseded、新 running、一次 revision |
| 已有同 request-id、同参数的成功请求 | 原成功响应重放 | 无新增 Attempt 或 revision |
| 同 request-id 改目标或理由参数 | `REQUEST_CONFLICT` | 无变化 |
| 目标不存在 | `NOT_FOUND` | 无变化 |
| 目标存在但非 running | `ATTEMPT_NOT_RUNNING` | 无变化 |
| running 目标非当前最新 | `ILLEGAL_NEXT` | 无变化 |
| Work 已 succeeded / cancelled | `WORK_TERMINAL` | 无变化 |
| Work blocked | `ILLEGAL_NEXT` | 无变化 |
| 同 Occurrence 已经替换过 | `REPLACEMENTS_EXHAUSTED` | 当前 Attempt 不变 |
| 冻结输入缺失、修改或路径不安全 | 现有输入或完整性错误 | 无变化 |
| COMMIT 后任务书效果失败 | `EFFECT_PENDING`、`committed=true` | 替换已提交；同 request-id 恢复 |

新请求先核 Work 非终态，再核目标身份、active/当前最新资格、替换额度、理由和输入；幂等重放先于这些新请求校验。

`next` 仅对满足 EX-01、EX-02 的状态列出 `attempt replace` 与目标 AttemptId。替换额度耗尽不新增 Work 阻塞状态；原有 `submit`、`fail`、`cancel` 仍遵守各自条件。

## 5. 保证范围

引擎保证正式接口拒绝已撤销资格的 Attempt。它不停止旧进程、不阻止同权限程序修改宿主文件、不撤销已经发生的副作用，也不提供多人并发开发隔离。接手前，操作者检查旧进程与共享工作区；这项人工报告不升级为引擎核实事实。

## 6. 采用后同步的上游

| 文件 | 内容 |
| --- | --- |
| `CONTEXT.md`、`specs/spec.md` | superseded、顺序号、显式撤销和固定一次规则 |
| `specs/architecture.md` | 状态转换、统计、原子替换与持久状态不变式 |
| `specs/contracts/protocol.md` | 命令、响应、next、错误、历史响应范围 |
| `specs/contracts/storage.md` | 新状态与理由字段、意图、快照、效果、格式版本 |
| `skills/sheltie/SKILL.md` | 继续与替换的选择、共享工作区责任、历史 next |

本方案不改变 Workbook 的配置字段。采用时为持久格式及变更的响应格式明确选择新版本，完整严格解码；旧 Store 保留并整体拒绝，不自动迁移、不清空，不建立兼容读写路径。
