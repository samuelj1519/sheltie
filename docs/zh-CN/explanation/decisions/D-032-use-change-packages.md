# D-032 使用 change package 管理版本迭代

[English](../../../en/explanation/decisions/D-032-use-change-packages.md) | 简体中文

状态：`accepted`
日期：2026-09-27
关联 change：C001

部分取代：D-22 的“AGENTS.md 写当前阶段”和后续版本只用 `Task` trailer；D-22 的 MVP 历史保持不变。

## 背景

MVP 完成后，`plan.md` 与 `decisions.md` 同时包含当前入口、任务历史、复核流水、设计理由和真实运行记录。`AGENTS.md` 还保存了已经过期的下一任务。继续追加会增加默认上下文、事实重复和状态误读。

## 选择

根 `specs/` 保留稳定权威文档；每次具体迭代使用独立 change package，并按 `proposed / active / completed / rejected` 管理；完成后的参考记录和发布历史按 [D-045](D-045-specs-and-diataxis.md) 归入 docs。`AGENTS.md` 只保存长期规则和入口路由。

详细职责、生命周期、模板与迁移规则见 [C001 设计](../../history/changes/C001-specs-governance/README.md)。

## 否决方案

- **继续扩展根 `plan.md`。** 它会让已关闭 MVP 方法与后续版本任务共享状态表。
- **按版本复制全部 spec 与 contracts。** 多份当前规则会漂移；发布快照可由 Git tag 重建。
- **只依靠 issue 或聊天管理。** 新会话和本地 agent 无法稳定发现外部上下文。
- **把当前状态写进 `AGENTS.md`。** 始终加载的入口会随任务推进快速过期。

## 后果

- 未采用提案不会被误当成授权实施。
- 当前进度只存在于 active package plan。
- 决定与发布记录保留长期解释；完成资格和原始运行记录由固定 Git 快照追溯。
- 完成后可按[文档维护指南](../../how-to/maintain-docs.md)提炼参考摘要，不将执行流水永久留在默认阅读集合中。
- 需要维护 change 索引和结构检查脚本。

## 如何确认仍成立

新会话只读 `AGENTS.md`、`specs/README.md` 和 change 索引，应能准确回答当前 release、active change、proposed change 与下一任务。`scripts/check-specs.sh` 机械检查目录状态、索引、ADR 与 release 闭包。
