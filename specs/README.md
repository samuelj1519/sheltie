# 文档地图

## 当前状态

Released：[`v0.2.0`](releases/v0.2.0/README.md)，发布目标为 `aarch64-apple-darwin`
开发目标：`v0.3.0`
Active change：无

C002与C004当前采用恢复义务已按限定范围独审完成；C005当前技术与需求负前检也已完成，C006当前实际副本与APFS载体验收已完成，C007当前agent观察/文档交付限定完成，C008当前条件复核完成。
Completed：[`C011 Workbook 可视化作者工具`](changes/completed/C011-workbook-visual-editor/README.md)（72/72 工具测试、独立审查、真人接受与同 Attempt 宿主重开通过；未发布），[`C010 局部精简与方法交付`](changes/completed/C010-local-simplification/README.md)（948测试/5doc/独立审查通过；方法0.2.2），[`C009 实现与测试精简`](changes/completed/C009-project-simplification/README.md)（行为保持、948测试/5doc/独立审查通过），[`C007 当前真实agent观察与三文档交付`](changes/completed/C007-pre-run-workbook-generation/README.md)（4质量过/2交付阻断，严格公平与15%条件未满足，原code/human未执行）、[`C006 当前真实agent副本与跨APFS验收`](changes/completed/C006-result-delivery/README.md)（原真人/费用/物理外置盘/其他OS/旧LEAK因果留原）、[`C005 当前原生/MSRV技术与负前检`](changes/completed/C005-executor-continuity/README.md)（原真实撤销、真人/费用及旧LEAK因果未执行/未知留原）、[`C004 当前自动agent交付与冷接续验收`](changes/completed/C004-verifiable-delegation/README.md)（原真人/净收益/费用/旧取证偏差留原）、[`C002 当前恢复验收`](changes/completed/C002-v0.2.0-reliability/README.md)（当前macOS aarch64/APFS可构造输入scoped PASS，物理/旧native未执行限制留原）、[`C008 条件前检与缺项交接`](changes/completed/C008-dependency-readiness/README.md)（probe未采用/真实价值not_run）。C002 的已发布历史和本轮补验见 completed package [plan.md](changes/completed/C002-v0.2.0-reliability/plan.md)
Proposed：无。本轮自动恢复工作已归档；原真实条件未满足的义务保留，见 [C001–C008 总交接](changes/completed/C008-dependency-readiness/evidence/resume-20261004/completion-report.md)。

此前 C004–C008 的限定实施已归档。2026-10-03 用户恢复全部跳过义务；一次只激活一个 package，缺真实输入时继续可独立执行的工作，未执行项保持 `not_run`。原生 Git 检查不作为通用结果/接续的前置条件。产品目标与采用顺序见[路线图](roadmap.md)，实施方式见[按完整行为制定和实施方案](guides/proposal-implementation.md)。

首屏开发目标指定当前源码候选的基础版本，尚未发布。数值产品 active 目标必须与它一致；非产品实验或暂无 active 时，该权威仍有效。已发布版本继续按 release record、tag 和验收证据核对，不由开发目标推断发布完成。

C005 已采用的 schema 4、`cli-result/v4`、number/superseded、`work-result/v1` 和 `workbook-digest/v2` 保持。C006的raw与完整副本实现已限定验收，实际范围见[当前 plan](changes/completed/C006-result-delivery/plan.md)，旧 schema 原件保留。C002 的验收范围、授权例外与已知限制见 release record 和 completed package。没有 active change 时，不从 proposed package 自行选择方案实施。

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
