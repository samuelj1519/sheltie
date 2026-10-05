# C003：MVP 文档归档

状态：`completed`
目标版本：`none`（文档归档）
兼容性：`none`
基线：`eb0918b2026190908e58c7c62cb95bb585abf194`
Owner：Codex
记录形式：`reference`
历史快照：`f38954d543ff01eb5a798be48f29060b80d5952c`

## 变化与理由

将已发布 MVP 与后续迭代分开，使用 Git tag 重建发布时规格，避免复制多份当前合同。历史任务不再作为开发入口。

## 验证与限制

只调整文档和历史任务检查路径，不改变产品行为。原任务正文与发布闭包可按固定快照读取。

本页保留设计与结果摘要；当前行为以根规格和合同为准。历史验证不能直接复用为当前候选 PASS。

## 参考

[v0.1.0](../../../reference/releases/v0.1.0/README.md)、[MVP 设计理由](../../../explanation/decisions/mvp.md)、[历史查阅](../../../how-to/maintain-docs.md#查阅历史原件)。完整任务、审查与运行原件按[历史查阅指南](../../../how-to/maintain-docs.md#查阅历史原件)从上述快照读取。
