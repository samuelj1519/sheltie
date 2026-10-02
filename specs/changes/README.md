# Change 索引

当前 release：[`v0.2.0`](../releases/v0.2.0/README.md)
Active change：无

只执行 active package 的计划。proposed package 尚未采用，不得自行实施。

## Proposed

| Change | 目标 | 状态 | 入口 |
| --- | --- | --- | --- |
| C004 | 可核对的任务闭环：实际工具检查、接续材料、明确结果 | proposed，探针与实现 not_run | [README](proposed/C004-verifiable-delegation/README.md) |
| C005 | 显式替换旧 Attempt；普通恢复继续原 Attempt | proposed，按替换需求采用 | [README](proposed/C005-executor-continuity/README.md) |
| C006 | 消费 C004 结果的独立按需导出 | proposed，按外部副本需求采用 | [README](proposed/C006-result-delivery/README.md) |
| C007 | 固定模板与任务输入的编写体验实验 | proposed，不改产品代码 | [README](proposed/C007-pre-run-workbook-generation/README.md) |
| C008 | 有真实必需资源时做单宿主外部预检 | proposed，不改内核准入 | [README](proposed/C008-dependency-readiness/README.md) |

先以 C007 或 C004 的轻量实验取得真实摩擦与质量证据；产品实施以 C004 为核心，C005/C006 由各自的需求触发。C008 没有真实必需宿主资源时不采用。一次只执行一个 active package，不为了凑齐编号实现全部方案。各阶段的强模型骨架、简单模型实现与独立强模型里程碑按[共同指南](../guides/proposal-implementation.md)执行。

## Active

无。

## Completed

| Change | 结果 | 入口 |
| --- | --- | --- |
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
