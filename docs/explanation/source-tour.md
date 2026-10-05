# 源码导读

本文供具备 Rust 基础、第一次阅读 Sheltie 的开发者使用。先读[领域词汇](../../CONTEXT.md)，再沿一条公开命令追踪行为。精确接口由[合同](../../specs/contracts)定义，完整结构由[架构](../../specs/architecture.md)定义。

## 三层引擎与两个外围工具

| 位置 | 职责 | 推荐入口 |
| --- | --- | --- |
| sheltie-cli | 参数解析、公开命令、错误码、文本／JSON 输出 | [main.rs](../../crates/sheltie-cli/src/main.rs)、[commands/](../../crates/sheltie-cli/src/commands)、[output.rs](../../crates/sheltie-cli/src/output.rs) |
| sheltie-runtime | 观察文件与 OS 事实、Store 事务、执行效果和恢复 | [service.rs](../../crates/sheltie-runtime/src/service.rs)、[load.rs](../../crates/sheltie-runtime/src/load.rs)、[recovery.rs](../../crates/sheltie-runtime/src/recovery.rs) |
| sheltie-core | 校验定义、状态决策、合法下一步和纯渲染 | [flow/compile.rs](../../crates/sheltie-core/src/flow/compile.rs)、[work/decide.rs](../../crates/sheltie-core/src/work/decide.rs)、[work/next.rs](../../crates/sheltie-core/src/work/next.rs) |
| sheltie-export | 通过 CLI 核最终成果，在明确授权父目录发布新副本 | [crate](../../crates/sheltie-export)、[导出指南](../how-to/export-results.md) |
| workbook-editor | 编辑 Workbook 草稿，以可信 CLI 检查并导出 ZIP | [工具 README](../../tools/workbook-editor/README.md)、[设计](workbook-editor.md) |

引擎依赖为 cli → runtime → core。core 接收已观察事实，不做 I/O；runtime 执行既定决定，不判断报告内容。业务方法在 [examples](../../examples) 与 [workbooks](../../workbooks) 中，不写进引擎分支。

## 沿一次写命令阅读

以 attempt submit 为例：

1. [commands/attempt.rs](../../crates/sheltie-cli/src/commands/attempt.rs) 将参数交给 WorkService。请求 ID 与原始意图用于区分新写入和历史重放。
2. [service.rs](../../crates/sheltie-runtime/src/service.rs) 的 submit／run_command 装入可信 Work 与图，观察声明输出，形成 core 所需事实。文件存在只是观察起点，还要核身份、归属、大小和字节。
3. [decide.rs](../../crates/sheltie-core/src/work/decide.rs) 的 decide／decide_submit 检查 Attempt 资格、状态和输出合同，返回新状态、回复与文件效果。输出报告的“通过／失败”不参与引擎判断。
4. [store/commit.rs](../../crates/sheltie-runtime/src/store/commit.rs) 登记状态、请求响应与待执行效果，使用 revision 防止并发覆盖。
5. [effects.rs](../../crates/sheltie-runtime/src/effects.rs) 执行文件效果；[recovery.rs](../../crates/sheltie-runtime/src/recovery.rs) 处理提交后的未完成效果。数据库提交成功与文件发布完成可能分属两个时点，错误响应必须保留这一差别。
6. CLI 映射准确错误或返回冻结回复与 next。历史重放回复属于当时快照，后续操作先查询新的 status。

排查故障时从公开命令与错误开始，定位提交前／后，再读相应恢复窗口测试。不要只改最后一个报错 helper，也不要直接修改 Store 或受管元数据。

## 沿一次查询阅读

[store/read.rs](../../crates/sheltie-runtime/src/store/read.rs) 的 read_work_bundle 在单一只读事务取得 Work、关联请求与效果。load 核完整载荷、状态和文件身份；status_read、result、stats 从同一上下文投影状态和 next，避免不同 revision 拼接。

[core 的 result_view](../../crates/sheltie-core/src/work/result.rs) 只按成功终点声明选择成果。需要真实字节时，再沿 [runtime/result.rs](../../crates/sheltie-runtime/src/result.rs) 和 CLI 的 result-artifact 路径看同快照、同句柄核验。普通结果查询不重新证明所有文件内容。

只读不推进业务状态、不补文件效果；SQLite 控制文件例外见 [D-039](decisions/D-039-sqlite-read-control-files.md)。

## 按问题寻找测试

| 要理解或修改的行为 | 独立期望与真实消费者 |
| --- | --- |
| 图与状态规则 | core 的 flow／work 模块测试与渲染快照 |
| 请求身份和原回复 | [CLI replay](../../crates/sheltie-cli/tests/replay.rs)、[runtime schema2_replay](../../crates/sheltie-runtime/tests/schema2_replay.rs) |
| 发布、效果与崩溃恢复 | [effect_contracts](../../crates/sheltie-runtime/tests/effect_contracts.rs)、[reliability_crash](../../crates/sheltie-cli/tests/reliability_crash.rs) |
| 文件身份与路径边界 | [fs_boundary](../../crates/sheltie-runtime/tests/fs_boundary.rs)、[snapshot_qualification](../../crates/sheltie-cli/tests/snapshot_qualification.rs) |
| 最终成果、原字节与撤销 | [work_result](../../crates/sheltie-cli/tests/work_result.rs)、[result_artifact](../../crates/sheltie-cli/tests/result_artifact.rs)、[attempt_replace](../../crates/sheltie-cli/tests/attempt_replace.rs) |
| 真实方法使用 | [scenario_code_change](../../crates/sheltie-cli/tests/scenario_code_change.rs)、[scenario_spec_dev](../../crates/sheltie-cli/tests/scenario_spec_dev.rs) |
| 外围工具与发布治理 | [export tests](../../crates/sheltie-export/tests)、[editor tests](../../tools/workbook-editor/test)、[release_governance](../../crates/sheltie-cli/tests/release_governance.rs) |

测试名描述行为，Task 注释供工具选组。历史任务号不表示功能仍在实施。修改前把合同义务、真实入口、正反期望与恢复时点连起来；精简测试时保留不同的身份、字节、观察时点和后继调用证据。验证范围见[工程规范](../../specs/engineering.md)与[预算指南](../how-to/validate-change.md)。
