# Change 索引

当前 release：[`v0.2.0`](../releases/v0.2.0/README.md)
Active change：无

C002与C004当前恢复采用范围已完成，C005当前技术及负前检已完成，C006当前实际副本与APFS验收完成，后续依次恢复C007–C008。

只执行 active package 的计划。proposed package 尚未采用，不得自行实施。

## Proposed

无未采用候选。

优先采用 C007 的共同试用，完整观察用户结果、质量和总成本；根据证据选择 C004 的最小增量。明确的同等近期需求也可直接支持采用，不要求先发生事故。C005/C006 依赖实际通用合同但不相互依赖；C008 只由真实宿主资源问题触发。本次用户已明确授权 C004–C008 按顺序实施；一次只执行一个 active package，实际证据与 `not_run` 分开记录。任务以完整行为和实际风险划分，流程见[共同指南](../guides/proposal-implementation.md)。


## Active

无。

## Completed

| Change | 结果 | 入口 |
| --- | --- | --- |
| C006 | 真实agent副本/两组同质量/编辑与newcopy/跨APFS导出卸载和完整限定链通过；原真人/费用/物理盘/旧unknown留原 | [README](completed/C006-result-delivery/README.md) |
| C005 | 当前macARM真实1.85新951/5doc及EX01–08同源工程和需求负前检通过；原真实撤销/真人/费用/旧LEAK因果留原 | [README](completed/C005-executor-continuity/README.md) |
| C004 | 当前agent实际质量/同Attempt冷接续/三refs与新消费者逐查询和实际使用通过；旧真人/成本/取证偏差留原 | [README](completed/C004-verifiable-delegation/README.md) |
| C002 | 当前采用恢复义务完整Spec/工程scoped PASS；macOS aarch64/APFS可构造输入，215处分与旧未执行限制分列 | [README](completed/C002-v0.2.0-reliability/README.md) |
| C008 | 真实条件前检/规则/延期交接限定完成；probe未采用，原机制/观察/value not_run | [README](completed/C008-dependency-readiness/README.md) |
| C007 | 技术资产/同源方法与缺项交接通过；原正式准入、六trial与真实价值not_run | [README](completed/C007-pre-run-workbook-generation/README.md) |
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
