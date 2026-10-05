# C005：Attempt 资格撤销

状态：`completed`
目标版本：`v0.3.0`（开发目标，尚未发布）
兼容性：`不保留格式兼容；旧 Store 保留并拒绝自动迁移`
基线：`23932afc1577a0b20a6b1cf7ad23ab8ac6186571`
Owner：`Codex /root`
记录形式：`reference`
历史快照：`f38954d543ff01eb5a798be48f29060b80d5952c`

## 变化与理由

以 attempt replace 在一次写操作中将旧 running Attempt 标为 superseded，并开始新 Attempt。number 是创建顺序号，业务失败只统计 failed；每个 Occurrence 固定最多一次替换。非统计输入继承旧冻结绑定，统计输入按含新 Attempt 的状态重新生成。

schema 4 与 cli-result/v4 整组切换；旧 Store 保留并拒绝自动迁移。

## 验证与限制

已完成原生／MSRV 技术验证与真实需求负前检。没有真实资格撤销事件时继续普通 resume，不制造事故。引擎不停止旧进程、不认证接手者、不隔离宿主，也不撤销已发生的外部副作用；真实撤销价值、真人成本与原 LEAK 因果仍无充分证据。

本页保留设计与结果摘要；当前行为以根规格和合同为准。历史验证不能直接复用为当前候选 PASS。

## 参考

[接续与撤销](../../../how-to/resume-work.md)、[D-041](../../../explanation/decisions/D-041-attempt-number-and-replacement.md)、[当前限制](../../../reference/limitations.md)。完整任务、审查与运行原件按[历史查阅指南](../../../how-to/maintain-docs.md#查阅历史原件)从上述快照读取。
