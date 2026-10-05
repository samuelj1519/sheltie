# C011：Workbook 可视化作者工具

[English](README.md) | 简体中文

状态：`completed`
目标版本：`v0.3.0`
兼容性：引擎CLI/Store/Workbook格式保持；新增未发布外围网页工具
基线：`de6fde63094ebcd068e5df9710e2eedcb67a4377`
Owner：`Codex协调者；工作agent；独立Reviewer；实际用户`
记录形式：`reference`
历史快照：`9d98bf8f15944b7bda4fbd4096e7771b09eab728`

## 变化与理由

交付本地网页工具，支持创建／打开目录、节点画布、属性与输入输出编辑、真实 CLI 检查及完整新 ZIP 副本。完整原对象和未编辑文件字节是来源；画布位置仅存浏览器视图，不进入 Workbook 格式。工具不直接修改作者目录、正式 Home 或 Work 冻结副本。

## 验证与限制

72/72 工具测试与限定独立审查通过，实际用户接受并确认真正关闭、重开宿主；同 deliver#1.0 接续后五份最终成果字节核对通过。只验收所测可用性，个人分钟、费用和 ROI 未知；工具未发布。

本页保留设计与结果摘要；当前行为以根规格和合同为准。历史验证不能直接复用为当前候选 PASS。

## 参考

[工具使用说明](../../../../tools/workbook-editor/README.zh-CN.md)、[设计说明](../../../explanation/workbook-editor.zh-CN.md)、[当前限制](../../../reference/limitations.zh-CN.md)。完整任务、审查与运行原件按[历史查阅指南](../../../how-to/maintain-docs.zh-CN.md#查阅历史原件)从上述快照读取。
