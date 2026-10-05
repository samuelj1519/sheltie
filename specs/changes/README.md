# 变更索引

Active change：无

此目录用于采用前的提案、进行中的实施与完成后的参考摘要。当前行为见[规格](../spec.md)，发布事实见[发布记录](../releases/README.md)。

## Proposed 与 Active

目前无未采用提案、无 active change。人采用后才执行 package plan；一次只允许一个 active change。模板见 [templates/](templates/)，准备和实施方法见[实施指南](../guides/proposal-implementation.md)。

## Completed

下列页面保留变化、设计理由、验证范围与阅读入口。completed 表示各自采用范围已关闭，不表示所有实验目标、平台或发布均通过。跨变更的支持与未知事项集中在[当前限制](../limitations.md)。

| Change | 长期参考 |
| --- | --- |
| C001 | [文档治理](completed/C001-specs-governance/README.md) |
| C002 | [可靠性修复](completed/C002-v0.2.0-reliability/README.md) |
| C003 | [MVP 文档归档](completed/C003-archive-v0.1.0/README.md) |
| C004 | [明确成果与可靠接续](completed/C004-verifiable-delegation/README.md) |
| C005 | [Attempt 资格撤销](completed/C005-executor-continuity/README.md) |
| C006 | [可编辑成果副本](completed/C006-result-delivery/README.md) |
| C007 | [完整任务体验实验](completed/C007-pre-run-workbook-generation/README.md) |
| C008 | [宿主依赖前检](completed/C008-dependency-readiness/README.md) |
| C009 | [实现与测试精简](completed/C009-project-simplification/README.md) |
| C010 | [局部精简与方法交付](completed/C010-local-simplification/README.md) |
| C011 | [Workbook 可视化作者工具](completed/C011-workbook-visual-editor/README.md) |

## Rejected

无。

## 生命周期与完成后收敛

```text
proposed ──人采用──▶ active ──实现、验证、独立 review──▶ completed ──纳入──▶ release
    └──人否决──▶ rejected
```

目录位置表示状态。active 的 plan 是当前进度唯一权威；完成时先保存完整资格记录，再按[文档维护指南](../guides/documentation.md)收敛为参考摘要。任务流水和运行原件从固定 Git 快照读取，摘要不能改写原结果。
