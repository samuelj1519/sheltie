# Change 索引

当前 release：[`v0.2.0`](../releases/v0.2.0/README.md)
Active change：[C005 原子撤销与重新领取](active/C005-executor-continuity/README.md)

只执行 active package 的计划。proposed package 尚未采用，不得自行实施。

## Proposed

| Change | 目标 | 状态 | 入口 |
| --- | --- | --- | --- |
| C006 | 核完全部字节后发布一份完整新副本 | proposed，按真实复制需求采用 | [README](proposed/C006-result-delivery/README.md) |
| C007 | 当前版本上的完整任务、方法复用与续接试用 | proposed，实验 not_run | [README](proposed/C007-pre-run-workbook-generation/README.md) |
| C008 | 真实必需资源造成摩擦后的单宿主只读预检 | proposed，条件未满足不采用 | [README](proposed/C008-dependency-readiness/README.md) |

优先采用 C007 的共同试用，完整观察用户结果、质量和总成本；根据证据选择 C004 的最小增量。明确的同等近期需求也可直接支持采用，不要求先发生事故。C005/C006 依赖实际通用合同但不相互依赖；C008 只由真实宿主资源问题触发。本次用户已明确授权 C004–C008 按顺序实施；一次只执行一个 active package，实际证据与 `not_run` 分开记录。任务以完整行为和实际风险划分，流程见[共同指南](../guides/proposal-implementation.md)。

## Active

| Change | 目标 | 入口 |
| --- | --- | --- |
| C005 | 原子撤销旧资格并重新领取；2026-10-03 由用户采用全部范围 | [README](active/C005-executor-continuity/README.md)、[plan](active/C005-executor-continuity/plan.md) |

## Completed

| Change | 结果 | 入口 |
| --- | --- | --- |
| C004 | 明确成果与可靠接续实现闭包通过；实际试用和环境缺项按授权延期 | [README](completed/C004-verifiable-delegation/README.md) |
| C002 | v0.2.0 可靠性修复，按授权范围完成并发布 macOS aarch64 版本 | [README](completed/C002-v0.2.0-reliability/README.md) |
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
