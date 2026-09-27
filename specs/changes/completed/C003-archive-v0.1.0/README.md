# C003：统一归档 v0.1.0 MVP 文档

状态：`completed`
目标版本：`none`（文档归档）
兼容性：`none`
基线：`eb0918b2026190908e58c7c62cb95bb585abf194`
Owner：Codex

## 要解决的问题

C001 已把 MVP plan、decision log 与 T25/T26 runbook 标为关闭历史，但它们和根 `tasks.toml` 仍散落在一级目录。当前规范入口与版本历史没有完成物理分层。

## 成功判据

- v0.1.0 的 release record、plan、decision log、runbook 与 tasks 位于同一目录。
- 根 `specs/` 只保留当前权威文档与治理入口。
- 所有 Markdown 链接、测试归属和 legacy `check-task` 继续工作。
- C002 保持 proposed/`not_run`，产品代码和 MVP 状态不变。

## 不做什么

- 不改产品代码、公开合同、MVP 完成结论或 C002 采用状态。
- 不重写 MVP 历史内容，只调整路径和必要的归档说明。

## 文档入口

- [plan.md](plan.md)：实施任务与完成判据。
- [progress.md](progress.md)：当前交接。
- [validation.md](validation.md)：候选与验证证据。
