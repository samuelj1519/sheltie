# D-041：创建顺序号与行政替换

[English](../../../en/explanation/decisions/D-041-attempt-number-and-replacement.md) | 简体中文

状态：`accepted`
日期：2026-10-03
关联 change：[C005](../../history/changes/C005-executor-continuity/README.md)

当前适用范围：schema 4／cli-result/v4 与行政替换。它延续 [D-033](D-033-store-schema-2.md)、[D-040](D-040-result-resume-format.md) 的单一格式与冻结读取原则，替代其中的 schema 2／3 和 cli-result/v2／v3 版本选择；不废除它们仍有效的设计理由。

## 背景

撤销正式提交资格需要在一个事务中结束旧 Attempt 并开始新 Attempt，不能拆为 fail 和 begin。业务失败额度描述实际执行失败，不能被行政撤销消耗。

## 选择

Attempt 后缀采用创建顺序号 `number`，只维护这一套字段。`failed` 与 `superseded` 计数从事实序列派生；每个 Occurrence 固定最多替换一次。

Store 升为 schema 4，公开响应快照采用 `cli-result/v4`。`replacement_reason` 必须存在，值可以为 null；serde 不将缺字段默认为 None。旧 schema 1/2/3 原件保留并整体拒绝，不迁移，不同时维护 retry 与 number。Workbook、成果与目录摘要格式保持。

新 Attempt 继承旧冻结的非统计输入引用与进入来源。runtime 使用同一文件句柄核路径、sha256 与 bytes；新 stats 按含新 Attempt 的提交后状态生成，精确历史字节随请求登记。

历史 fail 响应核原 Failed Attempt 及截至它的失败前缀，不用当前总失败数或 number 代替。Replace 审计保存物化后的完整有界 reason，并与旧 Attempt 的 `replacement_reason` 逐字核对；意图仍保存字面参数或文件源路径。此核验不新增状态库，其他命令的审计摘要政策保持。

## 后果与确认

替换只撤销正式接口资格，不停止进程、不隔离宿主、不认证接手者。操作者确认旧执行者和共享工作区处置，引擎不把这种外部确认升级为自己的已核事实。

真实消费者核合法与拒绝例、一次额度、`max_retries=0/1`、迟到提交、同意图重放、并发单胜及 COMMIT 前后的原字节恢复。独立 Reviewer 不编写被审修复。当前操作见[接续指南](../../how-to/resume-work.md)，格式见[存储合同](../../../../specs/contracts/storage.md)。
