# Workbook 参考

[English](../../en/reference/workbook.md) | 简体中文

Workbook 是一份方法的完整目录。此页提供目录、声明与样例的查询摘要；字段的类型、限额和装入拒绝顺序由[Workbook 合同](../../../specs/contracts/workbook.md)定义。编写步骤见[编写 Workbook](../how-to/write-workbook.md)。

## 目录与格式

```text
my-workbook/
  workbook.toml
  flows/default.toml
  instructions/draft.md
  resources/checklist.md
```

| 位置或标识 | 用途 |
| --- | --- |
| `workbook.toml` | `workbook/v1` manifest：ID、版本、名称、Flow 文件列表和可选宿主资源声明 |
| manifest 的 `flows` | 明确列出全部 Flow 文件；不自动扫描目录 |
| Flow 文件 | `flow/v1`：`id`、`entry`、Node 和显式 Edge |
| `instructions/` | Node 的说明文件；也可使用 `instruction.text` 内嵌正文 |
| `resources/` | 参考文件，使用 `resource.<path>` 绑定；随方法版本冻结 |
| `workbook-digest/v2` | 完整目录的摘要算法，规则见[存储合同 §5.1](../../../specs/contracts/storage.md#51-workbook-digestv2) |

所有定义对象拒绝未知字段。图坐标属于作者工具的视图，不写入 manifest 或 Flow。安装以完整目录为输入，运行以本 Work 的冻结副本为依据；作者目录的后续修改不改变已有运行。

## Node、Edge 与默认值

| 声明 | 查询要点 |
| --- | --- |
| `instruction` | `file` 与 `text` 恰选一个；文件相对 Workbook 根 |
| `executor` | 必填 `agent` 或 `human`；不自动设置门槛 |
| `tier` | agent 默认 `standard`，也可为 `strong`；human 不得声明 |
| `gate` | 默认 `false`；为 `true` 时成功提交后停在批准门槛 |
| `max_visits` | 默认 `1`，控制节点到达次数；包括沿回边再次到达 |
| `max_retries` | 默认 `1`，控制同一 Occurrence 的执行失败重试 |
| `inputs`、`outputs` | 默认空数组；`required` 默认 `true` |
| `outputs[].max_bytes` | 默认 1 MiB，最大 32 MiB；输出必须是安全普通文件 |
| `result` | input/output 默认 `false`；只有终点的必需项可选择为最终成果 |
| `edges[].kind` | `main`、`back`、`branch`、`re_review`；引擎采用相同的合法性判断，标签供协调者判断用途 |

Edge 的 `(from,to)` 不重复，不允许自环。所有 Node 必须从 entry 可达，至少有一个没有出边的终点；允许有环，以到达额度限制运行。连线与输入来源分别声明，连线不会自动传递文件。

输出 `path` 相对本 Attempt 的 `outputs/`，不能逃逸或与其他输出重名、互为祖先、构成 ASCII 大小写别名。输出路径使用合同规定的 ASCII 字符；不要将可显示的中文名称规则套到文件路径上。完整规则见[Node 字段](../../../specs/contracts/workbook.md#32-node-fields)和[编译校验](../../../specs/contracts/workbook.md#4-compilation-validation)。

## 输入来源

| `from` | 绑定内容 | 缺失规则 |
| --- | --- | --- |
| `start.<key>` | start 时物化的文本或 `@file` 内容 | 必须提供，不能设 `required=false` |
| `resource.<path>` | 本 Work 冻结方法中的参考文件 | 必须存在，不能设 `required=false` |
| `engine.stats` | 本次开工生成并冻结的统计 JSON | 引擎提供，不能设 `required=false` |
| `<node>.<output>` | 该节点最近一次成功 Attempt 的声明输出 | 必需时无来源就拒绝；可选时可为 `null` |

上游引用必须符合图上的可达关系，不能引用自己。可选上游输出不能作为必需输入。可选反馈没有更新时仍可能绑定以前的成功输出；从任务书的来源判断当前材料，不能把“已绑定”当作“刚生成”。

终点选择的输入固定为该终点开工时的引用，选择的输出固定为该终点提交时的引用。成功但没有 `result=true` 声明时，最终集合为空；准确来源见[数据模型](data-model.md#最终成果)。

## 宿主资源声明

manifest 的 `requires` 以 `kind + name` 标识 `skill`、`agent` 或 `mcp`，Node 用 `kind:name` 引用。版本、摘要与 source 是声明资料，引擎在响应和任务书中透传，不探测宿主、不求值兼容范围、不安装资源。协调者确认实际可用性后派活。

能够通过读文件解决的资料放在 `resources/`；宿主机制才使用 `requires`。普通参考资源、宿主安装与实际权限不是同一件事，边界见[限制](limitations.md)。

## 仓库内的方法

| 方法 | 起始输入 | 学习或使用目的 |
| --- | --- | --- |
| [two-step](../../../examples/two-step) | `topic` | 提纲 → 摘要；无门槛、回边或最终成果声明 |
| [article-review](../../../examples/article-review) | `topic` | 可选审查反馈、参考资源、返工与 human 执行者 |
| [gated-release](../../../examples/gated-release) | `version` | 提交后门槛阻断与批准；不实际发布软件 |
| [code-change](../../../examples/code-change/README.md) | `task`、`project` | 仓库修改、独立审查、返工与 change/review/delivery 成果 |
| [spec-dev](../../../workbooks/spec-dev/README.md) | `request`、`project` | 规格、规划、逐任务实现和验证、重规划、交付及反思 |

运行前用 `workbook show <id>@<version>` 核实际 Flow、起始键与声明。默认英文 spec-dev 方法版本为 `0.2.3`；独立中文方法 `spec-dev-zh-cn` 位于 `workbooks/spec-dev-zh-CN`，版本为 `0.2.2`。样例的英文默认目录与显式 `*-zh-CN` 中文方法分别使用独立 ID。方法版本不等于引擎版本。对应操作见[code-change](../how-to/run-code-change.md)和[spec-dev](../how-to/run-spec-dev.md)。
