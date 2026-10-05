# C002：可靠性修复

状态：`completed`
目标版本：`不进入产品 release`
兼容性：`breaking`（schema 2 拒绝旧库；旧管理根和数据原样保留）
基线：`a664e75a3ab3041d09cd0a1ab4d69336f2dcd055`
Owner：Claude（原实施）、Codex（后续修复与收敛）；采用人：用户（2026-09-27）
记录形式：`reference`
历史快照：`f38954d543ff01eb5a798be48f29060b80d5952c`

## 变化与理由

修复请求身份与历史响应重放、受管文件身份、Workbook 摘要框架、事务发布与恢复、删除证明、统计快照以及协调者接续。共同原则是先核完整身份和输入闭包，再执行文件效果；历史响应保留提交时事实，当前查询使用同一读取快照。

格式从 schema 1 切换到 schema 2，明确拒绝旧库，不增加双解释或自动迁移。后续源码格式以当前存储合同为准。

## 验证与限制

v0.2.0 已按 macOS aarch64 范围发布；发布实物、checksum 与授权例外见发布记录。后续恢复验收只覆盖 macOS aarch64／APFS 可构造输入。215 项处置包含动态、限定静态与结构检查，不等于 215 个历史原生变异全部执行。原 SK01/SK02、历史 LEAK 与旧取证限制保留其原结果，不能用新运行回填。

本页保留设计与结果摘要；当前行为以根规格和合同为准。历史验证不能直接复用为当前候选 PASS。

## 参考

[v0.2.0](../../../reference/releases/v0.2.0/README.md)、[存储合同](../../../../specs/contracts/storage.md)、[文件句柄](../../../explanation/decisions/D-037-managed-file-handles.md)、[SQLite 只读边界](../../../explanation/decisions/D-039-sqlite-read-control-files.md)。完整任务、审查与运行原件按[历史查阅指南](../../../how-to/maintain-docs.md#查阅历史原件)从上述快照读取。
