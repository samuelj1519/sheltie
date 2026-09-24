# 文档地图

Sheltie 的全部规范都在这个目录。按下面的顺序读，第一次接触项目大约需要一小时。

| 顺序 | 文件 | 回答什么 | 多久 |
| --- | --- | --- | --- |
| 1 | [constitution.md](constitution.md) | 为什么存在；哪些规则永远不破 | 5 分钟 |
| 2 | [../CONTEXT.md](../CONTEXT.md) | 词怎么用 | 5 分钟 |
| 3 | [spec.md](spec.md) | 产品要做成什么样；验收场景 | 15 分钟 |
| 4 | [architecture.md](architecture.md) | crate、类型、状态机、目录 | 15 分钟 |
| 5 | [contracts/workbook.md](contracts/workbook.md) | Workbook 与 Flow 的每个字段 | 10 分钟 |
| 6 | [contracts/protocol.md](contracts/protocol.md) | 每条命令、状态卡、错误码 | 10 分钟 |
| 7 | [contracts/storage.md](contracts/storage.md) | SQLite、事务、崩溃语义 | 10 分钟 |
| 8 | [engineering.md](engineering.md) | 怎么写代码、测试、提交、Review | 10 分钟 |
| 9 | [plan.md](plan.md) | 下一步做哪个任务 | 按需 |
| 10 | [roadmap.md](roadmap.md) | MVP 之后做什么、为什么现在不做 | 按需 |
| 11 | [decisions.md](decisions.md) | 每个有争议的设计决定：背景、选择、否决的方案 | 按需 |

## 权威顺序

冲突时按这个顺序裁决：宪章 → 规格 → 架构 → 合同 → 计划。下游改动不能反向修改上游的意思；要改上游先改上游。

## 文档说的是目标

所有「必须」都是目标合同。代码做到哪一步只看 [plan.md](plan.md) 的状态列。不得把计划中的能力写成已支持。

## 改文档之后

```bash
scripts/check-docs.sh
```

断链或遗留词命中即失败。
