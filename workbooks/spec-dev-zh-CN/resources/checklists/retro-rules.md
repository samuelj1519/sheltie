# 反思规则

简体中文 | [English](../../../spec-dev/resources/checklists/retro-rules.md)

## 六个类别（闭集）

| 类别 | 典型证据 | 通常改哪 |
| --- | --- | --- |
| 说明书歧义 | 同一节点多次被打回且报告指向同一句话；`change.md` 备注问「这是什么意思」 | `instructions/<node>.md` |
| 清单缺项 | 审查报告的发现不在任何清单里 | `resources/checklists/*.md` |
| 任务拆分 | 单个任务 `fix` 两轮以上；或任务平均耗时远低于其他任务 | `resources/checklists/task-rules.md`、`plan` 说明书 |
| 档位误判 | `standard` 节点反复卡住或反复被打回；`strong` 节点一次通过且耗时短 | `flows/default.toml` 的 `tier` |
| 工具缺陷 | `change.md` 备注写明脚本或单任务命令有问题；有 `Task: scaffold` 的修复提交 | `resources/checklists/scaffold-rules.md` |
| 门槛位置 | 人介入时说「这一步不用问我」或「这里该问我」 | `flows/default.toml` 的 `gate`、`executor` |

## 两条硬要求

1. **每条建议有证据。** 证据是一个能打开的位置：`<node>#<n>.<retry>/<文件>` 与行号，或 `stats` 里的一个数字。没有就不是建议。
2. **每条建议有落点。** 落点是这份 Workbook 里的一个文件与一段。改不到 Workbook 的事（例如项目代码本身的问题）不属于反思，写进「不建议改的」。

## 不做的

- 不评价实现者写的代码好坏，那是 `review` 的事。
- 不改 Workbook。你写的是建议，人读了决定，改进下一版。
- 不写感想。「这次很顺利」不算，「`fix` 零次到达，`implement` 六次全部一次通过」才算。
