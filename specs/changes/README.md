# Change 索引

当前 release：[`v0.1.0`](../releases/v0.1.0/README.md)
Active change：无

只执行 active package 的计划。proposed package 尚未采用，不得自行实施。

## Proposed

| Change | 目标 | 状态 | 入口 |
| --- | --- | --- | --- |
| C002 | v0.2.0 可靠性修复 | proposed，产品修复 `not_run` | [README](proposed/C002-v0.2.0-reliability/README.md) |
| C004 | 面向代码库变更的可验收委派（候选 v0.3.0） | proposed，实验 `not_run` | [README](proposed/C004-verifiable-delegation/README.md) |
| C005 | 执行者替换与任务接续（第一阶段候选随 v0.3.0） | proposed，实验 `not_run` | [README](proposed/C005-executor-continuity/README.md) |

## Active

| Change | 目标 | 当前任务 | 入口 |
| --- | --- | --- | --- |
| 无 | — | — | — |

## Completed

| Change | 结果 | 入口 |
| --- | --- | --- |
| C003 | 统一归档 v0.1.0 MVP 文档与历史任务检查 | [README](completed/C003-archive-v0.1.0/README.md) |
| C001 | 建立 specs、change、decision 与 release 文档治理 | [README](completed/C001-specs-governance/README.md) |

## Rejected

无。

## 生命周期

```text
proposed ──人采用──▶ active ──实现、验证、独立 review──▶ completed ──纳入──▶ release
    └──人否决──▶ rejected
```

目录位置就是状态。默认只允许一个 active change。模板见 [templates/](templates/)。
