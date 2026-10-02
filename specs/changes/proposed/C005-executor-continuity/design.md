# C005 设计

状态：`proposed`；基线见 README。路径表中的「拟新增」文件尚不存在，不是已实现能力。

## 1. 最小增量与模块职责

| 层 | 入口与职责 | 不承担的职责 |
| --- | --- | --- |
| core | `Command::ReplaceAttempt`、`decide`、`legal_next`；校验状态、派生计数、产生新状态/brief 效果 | 时钟、宿主观察、进程停止、文件 I/O |
| runtime | `WorkService::replace`、`RequestIntent::ReplaceAttempt`；取得主体/时间、核输入、持锁提交、快照与恢复 | 原因判断、模型选择、审查结论 |
| CLI | `AttemptCmd::Replace`；严格解析参数，输出持久响应中的旧/新 Attempt 和任务书 | 回读当前状态来改写历史成功响应 |
| Workbook 与协调者 | 决定是否替换、处理未封存草稿、安排独立审查 | 把模型自报写成系统核实事实 |

不增加 crate、独立状态库、资源等待、执行者绑定或宿主适配器。C004 的交接与结果视图继续从 Store 和冻结引用投影。

## 2. 现有入口与新增位置

| 真实路径 / 符号 | 实施内容 |
| --- | --- |
| `crates/sheltie-core/src/ids.rs` / `AttemptId` | 后缀字段 `retry` 改为 `number`，显示形状不变，严格解析与序号检查；同步 `work/layout.rs::attempt_dir` 的真实引用，保持目录标签与字节不变 |
| `crates/sheltie-core/src/flow/{def,parse,compile,graph}.rs` / `NodeDef` | 替换上限声明、默认值、范围、图节点投影 |
| `crates/sheltie-core/src/work/{state,command,decide,next,layout,render}.rs` | 新终态与命令；替换决策；failed/superseded 派生计数；统计、任务书与状态卡 |
| `crates/sheltie-core/src/error.rs` | 上限拒绝的明确错误 |
| `crates/sheltie-runtime/src/{request,service,snapshot,load}.rs` | 新意图、输入观察、持久响应严格校验、完整 WorkState 读取 |
| `crates/sheltie-runtime/src/store/{schema,read,commit}.rs` | 采用起点后的格式版本与一致读写；不另建替换计数表 |
| `crates/sheltie-runtime/src/effects.rs`、`failpoint.rs` | 核对真实效果入口后复用 prepare_attempt / write_file 恢复；只为新增窗口补故障点 |
| `crates/sheltie-cli/src/{cli,error_map,output}.rs`、`commands/attempt.rs` | 参数、错误码、原响应输出 |
| 拟新增 `crates/sheltie-runtime/tests/attempt_replace.rs` | 事务、重放、恢复与快照损坏测试 |
| 拟新增 `crates/sheltie-cli/tests/attempt_replace.rs` | 真实 CLI 调用链与约束保持 |

C004 新增文件以其完成候选为准。C005-T01 必须核对对应符号并更新任务白名单；不能把拟新增路径当成现有入口。采用起点发现模块已经拆分时，先由架构 Owner 修改 plan 与 tasks.toml。

## 3. 状态与输入算法

1. 读取当前 Work 与冻结图，要求状态为 active、目标等于当前 Occurrence 的最新 running Attempt。
2. 对同一 Occurrence 分别统计 failed 和 superseded；检查节点替换上限。
3. 对旧 frozen inputs 中非空引用做真实文件观察并核摘要。可选输入的 None 继续 None，不重新挑上游产物。不读取旧未封存输出作为正式输入。
4. 用检查加法分配下一个 number。复制 entered_from、冻结输入和当前 Node 的说明书依据。
5. 在一个纯 Decision 中把旧 Attempt 结束为 superseded，并追加新 running Attempt。旧 Attempt 的附属观察若为 open，在同一个 Decision 中关闭为 interrupted；completed 记录保持不变，新 Attempt 的观察为空。替换原因使用有界 Summary，记录为调用者报告；审计的 principal 只来自 runtime 系统身份。
6. 若节点绑定 engine.stats，从此 Decision 的完整状态生成新引用与精确字节，回填新 Attempt 输入。产生新任务书和刷新状态卡的效果。

输入核对失败时，不能先把旧状态改为 superseded；新的 AttemptId、任何输出目录和审计均不能发表。旧任务书仍是历史文件。新 brief 对旧草稿仅显示未封存指针，由接手者决定如何使用。

## 4. 原子提交与并发

`WorkService::replace` 使用 `WorkService::run_command` 写链，与 begin 同样经过目标解析、管理根写锁、request 查重、待完成效果恢复、输入观察、core 决策和 revision CAS。WorkState、audit、request 快照在同一 SQLite 事务提交。旧 Attempt 如有 C004 的 open 原生执行记录，在同一事务置为 interrupted；completed 记录保持原字节与归属，新 Attempt 不复制完成资格。旧 runner 的完成写入必须按 execution_id 和仍为 open 的状态做 CAS，不能复活被替换的记录。新 Attempt 以后可以按原合同启动自己的执行。新 Attempt 目录与 brief 由已登记效果在 COMMIT 后发布。

- submit 先提交：旧状态已 succeeded，replace 拒绝，不创建新尝试。
- replace 先提交：旧状态已 superseded，旧 submit / fail 拒绝。
- 两个不同请求同时替换同一旧 Attempt：第一个成功，第二个不再符合 running 条件。
- 上限拒绝：不撤销旧资格，不增 revision，不插入成功请求，旧进程可以完成。

写锁约束合作式 CLI。共享 Git 工作区或同权限直接写 Store 的动作不在此并发保证内。

## 5. 幂等、快照与恢复

`RequestIntent::ReplaceAttempt` 包含完整 WorkId、旧 AttemptId 和 reason 参数的字面值或词法绝对 @file 路径。它不包含时钟、OS 主体、观察摘要或 @file 内容。已提交重放先查原意图，不重新读取 reason 文件或冻结输入。

响应快照保存旧 AttemptId、新 AttemptId/number、提交时节点与 Occurrence、任务书与输出目录、输入输出路径、requires、revision 和 next。snapshot 与 service 的业务绑定校验同时更新：

- reply 与 data 的新旧身份必须一致；新旧属于同一 node/Occurrence、顺序号相邻；
- 对照原 audit 命令和已提交 WorkState 核替换理由、旧终态、新尝试及其冻结输入身份；
- 历史新 Attempt 后来可以已成功或再次被替换，不能拿当前 running 状态否定原快照；
- 请求的 next 按提交时状态合法，不用当前次数与状态重新推导历史 next；
- 效果只核本请求的新 Attempt 与 write_file 字节，不凭当前 brief 重建旧字节。

COMMIT 前故障没有替换业务事实。COMMIT 后故障保留请求与精确效果；返回 EFFECT_PENDING 或在下次写操作恢复。同 request-id 返回原新 Attempt，不能再 replace 一次。重放后刷新状态卡使用最新 revision，不把历史 next 写成当前状态卡。

## 6. 统计与格式

attempts 仍表示实际创建过的 Attempt 总数；succeeded、failed、superseded 分开派生。业务 retry 统计从 failed 事实计算，不用 number 的最大值。测试必须包含「替换 2 次后第 1 次失败」和非默认上限，防止默认 0 数据掩盖混淆。

WorkState、新旧响应数据与 strict serde 同一提交闭合。采用时为整个持久格式选择新的 schema 版本，为修改的响应字段选择明确协议版本；不得在同一个已公布版本里偷偷把 retry 改成 number。旧版本库拒绝而保留。C004 已更改版本时，以 C004 完成候选作为输入确定下一版本；不在 proposal 内提前分配冲突编号。

## 7. 首次实施前必须确认

- C004 的交接视图能够显式区分正式 ArtifactRef 与旧未封存草稿。
- 实际模块和效果入口与路径表一致；当前 effects 模块如已拆分，T01 同步路径。
- max_replacements 默认关闭是否适合首个 Workbook；启用值由作者在冻结方法里选择，不给 runtime 动态开关。
- 真实使用确实需要撤销提交资格；若只缺交接材料，先修 C004，C005 不启动。
