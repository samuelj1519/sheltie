# C003 实施计划

| ID | 状态 | Owner | 结果 |
| --- | --- | --- | --- |
| C003-T01 | done | Codex | 归档五份 MVP 文件，更新引用和检查脚本，完成独立 review |

## C003-T01 统一归档 MVP 文档

**文件。** `specs/releases/v0.1.0/`、引用 MVP 历史的当前文档、`scripts/{check-docs,check-specs,check-task,check-tests}.sh`、本 package。

**正例。** 新会话从 release record 可找到完整 MVP 计划、决定、runbook 与任务白名单。

**反例。** 根 `specs/plan.md`、`specs/decisions.md`、`specs/t25-t26-runbook.md` 或根 `tasks.toml` 仍存在时，治理检查失败。

**停止条件。** 任一历史链接无法无损重定向，或测试归属/legacy task 检查失效时停止并修复，不保留一级目录 stub。

**验证。** `check-specs`、`check-docs`、`check-tests`、legacy `check-task` 的隔离 smoke、格式与独立 review。

**提交。** `docs(specs): 统一归档 v0.1.0 MVP 文档`
