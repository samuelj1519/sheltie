# C005 设计

状态：`completed`。源码基线见 [README](README.md)；路径与职责已经实现，当前进度和完整候选只看 plan/validation。

## 1. 模块职责与真实入口

| 位置 / 符号 | 责任 |
| --- | --- |
| `crates/sheltie-core/src/ids.rs::AttemptId` | `retry` 改为 `number`；字符串形状不变，检查顺序号运算 |
| `crates/sheltie-core/src/work/state.rs::AttemptStatus`、`WorkState::validate_persisted` | Superseded、理由与终止字段一致性；同 Occurrence 至多一次替换；连续顺序号 |
| `crates/sheltie-core/src/work/command.rs::{Command,Reply}` | ReplaceAttempt 及持久响应所需数据 |
| `crates/sheltie-core/src/work/decide.rs::{decide,reply_status_matches}` | 纯替换决策、按 failed 计数；历史 fail 的前缀校验 |
| `crates/sheltie-core/src/work/next.rs::legal_next` | 固定一次替换资格和真实失败重试资格 |
| `crates/sheltie-core/src/work/{layout,render}.rs` | 顺序号引用、任务书、紧凑状态与事实统计 |
| `crates/sheltie-runtime/src/service.rs::{WorkService::run_command,validate_command_owner}` | 输入观察、同事务提交、历史请求归属校验 |
| `crates/sheltie-runtime/src/{request,snapshot,load}.rs` | 意图、严格响应解码、状态及路径不变式 |
| `crates/sheltie-runtime/src/{effects,recovery,failpoint}.rs` | 复用 prepare_attempt/write_file 的精确字节恢复与必要故障点 |
| `crates/sheltie-runtime/src/store/{schema,read,commit}.rs` | 明确的新格式及原子状态、audit、request 写入 |
| `crates/sheltie-cli/src/cli.rs::AttemptCmd`、`commands/attempt.rs::run` | 参数和真实 WorkService 调用；按原快照输出 |

只新增 `Command::ReplaceAttempt`、`Reply::AttemptReplaced`、`WorkService::replace` 与所需状态字段。不增加 crate、身份注册、后台进程、宿主适配器或另一份事实来源。core 不做 I/O；runtime 不判断理由合理性。

## 2. 一个完整原子操作

1. 沿用真实写链，先识别 request-id 与意图；已提交请求走原快照重放。
2. 新请求持管理根写锁，装入严格校验的 Work、冻结图与当前最新 Attempt，检查 active/running 和固定一次额度。
3. 观察旧冻结输入的实际文件，同句柄核大小与摘要。可选 None 保持 None，不重新选择较新的上游产物；旧草稿不参与绑定。
4. 用检查加法分配 `number+1`。纯 Decision 将旧 Attempt 结束为 Superseded，追加新 Running，并继承 `entered_from` 与非统计输入。
5. 用 Decision 的完整提交后状态生成新 `engine.stats`，固定引用与精确字节，生成任务书和刷新状态卡效果。
6. 在一次 SQLite 事务中提交 state、audit、request snapshot、effects，并做 revision CAS。COMMIT 后按既有效果链准备新目录、发布任务书和统计文件。

Superseded 的状态字段约束为：有 ended_at 和 replacement_reason，没有 summary、fail_reason、封存 outputs。其他状态不得带 replacement_reason。理由使用既有有界文本类型；Store 读取必须验证字段组合，不把缺失当默认值。

拒绝不能先撤销旧 Attempt，不能发表新任务书或增加审计成功事实。输入观测、身份、时钟属于 runtime 边界；不从 CLI 接收引擎核实事实。

## 3. 顺序号与失败数

创建 Attempt 时只按同一 Occurrence 最新顺序号分配。业务失败数来自 `AttemptStatus::Failed`；替换数来自 Superseded，不能另存可漂移的计数。统计分别展示 attempts、failed、superseded；现有耗时统计沿既有口径纳入已结束的尝试。

所有依赖 `attempt.retry` 的真实调用方须同时闭合：ID 序列化、布局、begin 数据、render、next、failure 决策、snapshot、service 与 fixtures。不保留 retry/number 双字段，不用后缀推导失败数。

`reply_status_matches` 当前被 `service.rs::validate_command_owner` 用来校验持久历史响应。它必须获得原 Attempt 所在 Occurrence 的事实序列，并以截至该 Attempt 的 failed 前缀判断当时 Work status。不得使用后续总失败数或当前 Work status。历史归属仍要核目标确为 Failed，不能只通过前缀计数就接受伪造 snapshot。

关键真实 caller 回归：`max_retries=1`，begin→replace→首次 fail（number=1，仍 active）→begin（number=2）→第二次 fail（blocked）→同 request-id 重放首次 fail，仍返回原 active 响应。该路径同时暴露把顺序号当失败数和把当前失败数用于历史响应的错误。

## 4. 并发、重放与恢复

- submit 先提交：replace 因目标非 running 拒绝；replace 先提交：旧 submit/fail 拒绝。
- 两个不同 replace 请求竞争同一旧 Attempt：至多一个成功。
- 固定一次额度耗尽：整个请求拒绝，当前 running Attempt 仍可 submit/fail；不重置额度，不增加 blocked 原因。
- COMMIT 前中断：没有替换业务事实。COMMIT 后中断：保留原 request 与精确 effects；恢复同一新 Attempt、brief 与 stats 字节。

`RequestIntent::ReplaceAttempt` 保存完整 WorkId、旧 AttemptId、理由字面参数或词法绝对文件路径，不包含当前时钟、主体、摘要或文件内容。重放不重新读取理由文件或输入文件。

原替换响应的旧/新身份、相邻顺序号、冻结输入、requires、路径和 revision 必须与原审计及对应历史 Attempt 相符。新 Attempt 后来可以 succeeded 或 failed，不能因此否定原 replace/begin 快照。历史 next 只与提交时事实对应；当前状态卡从最新 revision 刷新，不写入历史 next。

## 5. 最小 Rust 实施

状态机使用穷尽 enum；ID、理由、摘要与路径使用已有校验类型；新增公开项只服务真实 CLI→runtime→core caller。沿用每 crate Error 枚举和错误码映射，新增 `REPLACEMENTS_EXHAUSTED`。完整 serde 载荷拒绝未知字段。

## 6. 阶段接口与实现责任

T01 由复杂模型完整闭合 number 的全部消费者、Superseded 严格载荷、失败前缀、冻结输入观察、历史快照归属和精确 effects/recovery。手写合法持久夹具走真实读取 caller 验证历史校验原语；它只证明读取合同，不计实际替换已发生。M1 核完整原语与阶段测试后，T02 用真实 replace→fail→begin→fail 链复核同一义务。

T01 固定 ReplaceAttempt/AttemptReplaced、WorkService::replace 的输入输出和恢复数据。本轮复杂作者在 T01 完成必要耦合的纯状态变换及 runtime 库替换原语；留给 T02 的是公开 CLI 与正常 next 接线；正常 CLI 与 next 在完整行为完成前不提供 replace。阶段骨架须编译并保持普通行为，不能返回假成功。复杂作者创建全部阶段 tests、fixtures、冻结命令和真实使用手册，原语用例归 T01，新行为用例归 T02 并先 ignore。

T02 不改状态合同、serde、snapshot、事务、输入/文件政策或测试期望；缺口交复杂作者新增明确修复任务，独立复核后更新测试基准。M2 核正式入口、原子并发、历史重放、恢复与真实接续。未变化的 M1 原语证据可按相同闭包引用，不重复全套验证。
