# Change 索引

当前 release：[`v0.1.0`](../releases/v0.1.0.md)
Active change：无

没有 active change 时，不得从 proposed 中自行选择方案实施。提案采用、任务开始和发布由人决定。

## Proposed

| Change | 目标 | 状态 | 入口 |
| --- | --- | --- | --- |
| C002 | v0.2.0 可靠性修复 | proposed，产品修复 `not_run` | [README](proposed/C002-v0.2.0-reliability/README.md) |

## Active

无。

## Completed

| Change | 结果 | 入口 |
| --- | --- | --- |
| C001 | 建立 specs、change、decision 与 release 文档治理 | [README](completed/C001-specs-governance/README.md) |

## Rejected

无。

## 生命周期

```text
proposed ──人采用──▶ active ──实现、验证、独立 review──▶ completed ──纳入──▶ release
    └──人否决──▶ rejected
```

目录位置就是状态。默认只允许一个 active change。模板见 [templates/](templates/)。
