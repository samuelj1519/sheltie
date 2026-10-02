# 文档地图

## 当前状态

Released：[`v0.2.0`](releases/v0.2.0/README.md)，发布目标为 `aarch64-apple-darwin`
Active change：无
Completed：[`C002 v0.2.0 可靠性修复`](changes/completed/C002-v0.2.0-reliability/README.md)，任务历史见其 [plan.md](changes/completed/C002-v0.2.0-reliability/plan.md)
Proposed：[C004 可核对的任务闭环](changes/proposed/C004-verifiable-delegation/README.md)、[C005 显式执行者替换](changes/proposed/C005-executor-continuity/README.md)、[C006 按需导出明确结果](changes/proposed/C006-result-delivery/README.md)、[C007 固定模板编写实验](changes/proposed/C007-pre-run-workbook-generation/README.md)、[C008 单宿主外部预检实验](changes/proposed/C008-dependency-readiness/README.md)。任务与实验尚未执行，不构成必须全部实施的版本清单。

下一阶段先验证完整任务的人工投入与质量，C004 提供核心闭环，C005/C006 按实际需要实施，C007/C008 只采用实验范围。共同实施方式见[分阶段搭建与实现](guides/proposal-implementation.md)。

当前版本为 v0.2.0。上游规格与合同描述当前格式（schema 2、`workbook-digest/v2`、新 Work 目录布局、`cli-result/v2`）；C002 的验收范围、授权例外与已知限制见 release record 和 completed package。没有 active change 时，不从 proposed package 自行选择方案实施。

## 权威文档

第一次接触项目时按任务需要渐进读取，不要求每次通读全部文件。

| 文件 | 回答什么 | 何时读 |
| --- | --- | --- |
| [constitution.md](constitution.md) | 为什么存在；哪些规则永远不破 | 改产品边界或架构前 |
| [../CONTEXT.md](../CONTEXT.md) | 一个概念叫什么 | 写代码或文档前 |
| [spec.md](spec.md) | 当前开发线的产品目标与验收 | 改行为前 |
| [architecture.md](architecture.md) | crate、类型、状态机、目录 | 改 module/interface 前 |
| [contracts/](contracts/) | Workbook、CLI、Store 的精确 reference | 改字段、命令、错误或持久格式前 |
| [engineering.md](engineering.md) | 怎么写代码、测试、验证、提交与 review | 开始实现前 |
| [roadmap.md](roadmap.md) | 未立项方向与立项条件 | 评估未来工作时 |
| [decisions/](decisions/README.md) | 当前决定及其理由 | 需要理解取舍时 |

冲突时按“宪章 → 规格 → 架构 → 合同”裁决。active change 在实施前先更新这些上游权威；package plan 只规定执行顺序，不能反向改变产品含义。

## 迭代与历史

| 入口 | 用途 |
| --- | --- |
| [changes/README.md](changes/README.md) | proposed、active、completed、rejected change 及当前进度入口 |
| [releases/README.md](releases/README.md) | tag、候选、验收闭包与已知限制 |
| [guides/README.md](guides/README.md) | 仍可执行的 how-to 与 runbook |
| [research/README.md](research/README.md) | 一手来源笔记；不具有项目权威性 |

MVP 历史统一保存在 [v0.1.0 release archive](releases/v0.1.0/README.md)，不作为后续版本入口。

## 文档职责

- 根规格与合同描述当前目标，不描述任务进度。
- active package 的 `plan.md` 是当前实施进度的唯一权威。
- `progress.md` 只保存跨会话交接，不复制任务状态。
- ADR 解释“为什么”；architecture/contracts 定义“现在是什么”。
- review 与 validation 保存候选和证据，不替代完成状态。
- `CHANGELOG.md` 只记录已采用、用户可感知的版本变化。

## 检查

修改文档后运行：

```bash
scripts/check-docs.sh
scripts/check-specs.sh
```
