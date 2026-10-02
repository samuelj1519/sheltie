# C005 接续执行手册

本手册区分机制验证与真实撤销需求。当前没有提供真实任务、需撤销的旧执行者或宿主会话记录；真实使用暂为 `not_run`。不得把故障注入、夹具或继续原 Attempt 当成真实撤销价值。

## 1. 机制验证

测试名和归属见 [plan](../plan.md) 的 T01、T02“测试”清单；过滤器由 [commands.sh](../verification/commands.sh) 从同一清单的源码 Task 标记生成。T01 原语必须 green；T02 的完整用户入口和 `next` 预期在 T01 显式运行得到 red。普通回归跳过这些未来用例，不能报告其已通过。

使用隔离 target、清空 wrapper；保留候选完整 SHA、dirty diff、工具版本、原输出与 nextest run ID。单次原语或 feature 预算 5 分钟，普通回归及完整门禁分别 12 分钟；超时保留输出并交作者诊断，不算通过。并发、拒绝、历史重放和恢复均核状态、revision、请求、审计与原文件。

```bash
bash specs/changes/completed/C005-executor-continuity/verification/commands.sh primitives
bash specs/changes/completed/C005-executor-continuity/verification/commands.sh future-red
bash specs/changes/completed/C005-executor-continuity/verification/commands.sh feature
bash specs/changes/completed/C005-executor-continuity/verification/commands.sh regression
```

本机 nextest 0.9.140 低于配置要求 0.9.145，原命令退出 92，未执行测试。授权延期该工具版本门禁后，可显式设置 `SHELTIE_NEXTEST_VERSION_OVERRIDE=1` 执行补充验证；记录实际版本和原命令 `not_run`。这不等于已运行 0.9.145。

## 2. 真实接续的前提

操作者提供一个正在执行且确需撤销旧正式提交资格的真实 Work、原目标、冻结输入、成果标准和 gate；说明普通继续为什么不足。记录旧进程停止或宿主隔离的实际证据。引擎不停止进程，也不核实这项人工报告。

所有命令使用同一个明确的绝对管理根和完整 WorkId。开始前固定候选、Workbook 摘要、模型、环境、权限、实际执行者和质量检查命令；不在实验中更换标准。先记录当前卡和原任务书，普通重开沿 `resume` 继续原 Attempt。只有已成立的撤销需求才执行替换。

```bash
sheltie --home "$trial_root" --json work status "$work_id"
sheltie --home "$trial_root" --json --request-id "$replace_request" attempt replace "$work_id" --attempt "$old_attempt" --reason "$reason"
sheltie --home "$trial_root" --json work status "$work_id"
```

变量由真实任务记录填入，不使用示例值冒充真实样本。从响应读取新任务书、冻结输入和草稿路径，逐项对照原标准；完成原目标后按 Workbook 的合法 submit、review、gate 路径操作。旧迟到提交拒绝应有真实接口证据；不能故意让旧进程继续写共享目录来制造事故。

## 3. 记录、停止与交接

记录选择理由、操作者报告与宿主事实的边界、原与新 Attempt、请求 ID、revision、实际检查输出，以及解释、重新验证、重做和总投入。使用未知字段值 `null`，不把 Attempt 耗时当成人工时间或成本。

输入、说明、成果标准或 gate 丢失即停止，保留原件，交责任作者修复；并发和恢复异常不由手册执行者调整测试或合同。没有真实对象时只交接冻结步骤与缺项，不执行替代样本。T03 为说明及实际使用任务，没有归属的 Rust 场景，不以零测试运行验收。

## 4. 阶段二接线入口

先读 M1 的完整骨架 SHA；不得用随手的 HEAD 代替冻结基准。源码允许修改点只有 `core/work/next.rs`、CLI `cli.rs` 和 `commands/attempt.rs`。core 的纯 Decision 与 runtime 库替换、观察、事务和恢复已在 T01 完成。测试文件只删本任务 ignore；其他生产、测试期望、fixtures 和 snapshots 保持冻结。

`WorkService::replace(&WorkId, &AttemptId, &InputValue, Option<String>) -> Result<Response>` 接收已解析的旧身份及理由源。理由使用既有 `parse_text_arg`，身份使用 `AttemptId::parse`，Work 使用既有 resolve。`Response.reply` 必须为 `AttemptReplaced`，其新旧 Attempt、任务书和输出目录用于文本，`ok_response` 输出已持久的 data、revision、request_id、next。不可回读 Store 补造历史数据。

`legal_next` 的 Running 分支在 fail 后、cancel 前，仅在当前 Occurrence 没有 Superseded 时列 `NextOp::ReplaceAttempt { attempt: latest.id.clone() }`。参数 JSON 及文本占位符渲染已完成，不修改合同。按 plan 的 11 项完整 caller 用例执行 feature；全部通过后删除它们的 ignore，再运行同一完整门禁。

T04 补齐同一完整状态卡恢复场景的未来 replace 行，独立审查后给出新的完整测试基准。T02 采用该修订 SHA，而非旧 M1 SHA；其他基础原语不变。公开运行草稿与失败原文单独保存并逐字节恢复。
