# Workbook 与 Flow 格式

本合同定义 Workbook 目录、`workbook.toml`、Flow 文件的精确形状与装入校验规则。版本串 `workbook/v1`、`flow/v1` 是初始值。所有对象**未知字段拒绝**。

## 1. 目录

一个 Workbook 是一个目录，可在任何仓库编写：

```text
my-workbook/
  workbook.toml
  flows/
    default.toml
  instructions/
    draft.md
    review.md
  resources/                 可选。给工作 agent 读的参考文件，用 resource.<path> 绑成输入
    review-checklist.md
```

`sheltie workbook add <dir>` 做四件事：解析 `workbook.toml`；解析每个 Flow 并编译成图；核对每个 `instruction.file`、`resource.<path>` 输入与 `requires` 引用都存在；把整个目录复制到 `~/.sheltie/workbooks/<id>/<version>/` 并置只读。任一步失败则整体拒绝，不落任何文件。校验不执行脚本、不调模型、不联网。

同一 `<id>/<version>` 已存在时拒绝（`WORKBOOK_EXISTS`）。要改内容就升版本。

## 2. `workbook.toml`

```toml
schema = "workbook/v1"
id = "article-review"          # 小写字母、数字、单个连字符；≤ 64 字节；全局唯一
version = "1.0.0"              # 语义化版本字符串；只做字面比较，不解析范围
name = "文章写作与审查"
description = "写一篇文章，由独立 agent 审查，不通过则打回修改。"

flows = ["flows/default.toml"] # 至少一个；路径相对 Workbook 根

[[requires]]                   # 可选。需要宿主已装的资源，只声明不打包
kind = "skill"                 # skill | agent | mcp
name = "company-api"
version = "^1"                 # 可选，语义化版本范围，只给人和安装工具看
digest = "sha256:…"            # 可选，给了就要求字节一致
source = "https://github.com/acme/skills/tree/main/company-api"   # 可选，告诉安装工具去哪拿
```

| 字段 | 类型 | 规则 |
| --- | --- | --- |
| `schema` | string | 必须等于 `workbook/v1` |
| `id` | string | 正则 `^[a-z0-9]+(-[a-z0-9]+)*$`，≤ 64 字节 |
| `version` | string | 非空，≤ 32 字节，只允许 `[0-9A-Za-z.+-]` |
| `name` | string | 非空，≤ 128 字节 |
| `description` | string | 可选，≤ 2 KiB |
| `flows` | array of string | 非空；每项是 Workbook 内相对路径，不得含 `..`、绝对路径或符号链接 |
| `requires` | array of table | 可选，≤ 32 项；`(kind, name)` 唯一 |

`requires[]` 字段：

| 字段 | 规则 |
| --- | --- |
| `kind` | `skill`、`agent`、`mcp` 之一 |
| `name` | 同 ID 字符规则。身份是 `kind + name`；两个 Workbook 声明同名同类即同一资源，宿主里已有的同名资源也算 |
| `version` | 可选，≤ 32 字节。MVP 只透传，不比较 |
| `digest` | 可选，`sha256:` 加 64 位小写十六进制。给了才要求字节一致 |
| `source` | 可选，≤ 512 字节的 URL |

**资源分两层。** 能让工作 agent「读文件」解决的，放进 `resources/` 用输入绑定，随版本冻结，不装、不撞名。只有需要宿主机制（skill 自动触发或工具授权、带模型与工具限制的命名 subagent、MCP 服务）才写进 `requires`。声明不等于打包；引擎只把声明列进任务书，不检查宿主、不安装（安装与就绪检查见 [路线图 GF-20](../roadmap.md)）。

## 3. Flow 文件

```toml
schema = "flow/v1"
id = "default"                 # Workbook 内唯一
entry = "draft"                # 入口节点

[[nodes]]
id = "draft"
title = "写初稿"
executor = "agent"
instruction = { file = "instructions/draft.md" }
inputs  = [{ name = "topic", from = "start.topic" }]
outputs = [{ name = "article", path = "article.md", max_bytes = 262144 }]

[[nodes]]
id = "review"
title = "审查初稿"
executor = "agent"
instruction = { file = "instructions/review.md" }
inputs  = [
  { name = "article",   from = "draft.article" },
  { name = "checklist", from = "resource.resources/review-checklist.md" },
]
outputs = [{ name = "verdict", path = "review.md", max_bytes = 65536 }]
max_visits = 3

[[nodes]]
id = "publish"
title = "定稿"
executor = "human"
instruction = { text = "阅读审查通过的文章，确认可以发布。把最终版复制到 final.md。" }
inputs  = [{ name = "article", from = "draft.article" }, { name = "review", from = "review.verdict" }]
outputs = [{ name = "final", path = "final.md" }]

[[edges]]
from = "draft"
to = "review"
kind = "main"

[[edges]]
from = "review"
to = "publish"
kind = "main"

[[edges]]
from = "review"
to = "draft"
kind = "back"
```

### 3.1 Flow 字段

| 字段 | 类型 | 规则 |
| --- | --- | --- |
| `schema` | string | 必须等于 `flow/v1` |
| `id` | string | 同 `workbook.id` 的字符规则 |
| `entry` | string | 必须是某个节点的 `id` |
| `nodes` | array of table | 1 到 64 个；`id` 唯一 |
| `edges` | array of table | 0 到 256 条；`(from, to)` 唯一 |

### 3.2 Node 字段

| 字段 | 类型 | 默认 | 规则 |
| --- | --- | --- | --- |
| `id` | string | 必填 | 同 ID 字符规则 |
| `title` | string | 必填 | ≤ 128 字节，给人看 |
| `executor` | `"agent"` 或 `"human"` | 必填 | 只表示谁干活。不推断门槛 |
| `tier` | `"strong"` 或 `"standard"` | `"standard"` | 给协调者选模型的标签：`strong` 需要设计判断，`standard` 是填空、跑命令、对照清单。引擎只透传到 `next` 与任务书，不据此做任何事。`executor = human` 时不得出现 |
| `instruction` | `{ file = ... }` 或 `{ text = ... }` | 必填 | 恰一个。`file` 是 Workbook 内相对路径，文件 ≤ 64 KiB，UTF-8；`text` 非空、≤ 8 KiB |
| `inputs` | array | `[]` | 每项 `{ name, from, required? }`；`name` 在节点内唯一；`required` 默认 `true` |
| `outputs` | array | `[]` | 每项 `{ name, path, required?, max_bytes? }`；`name` 与 `path` 在节点内唯一 |
| `requires` | array of string | `[]` | 每项 `"<kind>:<name>"`，必须对应 `workbook.toml` 的一条 `requires`。引擎把它们列进任务书 |
| `gate` | bool | `false` | `true` 表示 Attempt 成功后要真人批准才能离开本节点 |
| `max_visits` | integer | `1` | 1 到 32。本节点被到达的次数上限，含回环 |
| `max_retries` | integer | `1` | 0 到 8。同一次到达内失败重试上限 |

`inputs[].from` 三种写法：

| 写法 | 含义 | 校验 |
| --- | --- | --- |
| `"start.<key>"` | 起始输入里的键 | `key` 是 ID 字符规则；运行时缺该键则 `work start` 拒绝 |
| `"resource.<path>"` | Workbook 内的一个文件，随版本冻结 | `path` 是 Workbook 内相对路径，文件存在、是普通文件、≤ 32 MiB。运行时指向本 Work 的冻结副本 |
| `"<node>.<output>"` | 某节点最近一次成功 Attempt 的某个输出 | `node` 存在且不是自己，且 `node` 不能叫 `start` 或 `resource`；`output` 是该节点声明的输出名；见 §4 第 5 条 |

`inputs[].required = false` 只对 `<node>.<output>` 来源有意义：上游还没有成功的 Attempt 时，这个输入不绑定，任务书里标「尚无」，`attempt begin` 不报 `INPUT_UNAVAILABLE`。这是回环的标准写法：被打回的节点用可选输入接收审核意见，第一次到达时没有意见也能开工。`start.<key>` 与 `resource.<path>` 来源上 `required = false` 编译拒绝，它们永远存在。

`outputs[]`：

| 字段 | 默认 | 规则 |
| --- | --- | --- |
| `name` | 必填 | ID 字符规则 |
| `path` | 必填 | Attempt 目录内相对路径；不得含 `..`；不得与 `brief.md` 相同 |
| `required` | `true` | `false` 时缺文件不算错，下游不得把它当必需输入 |
| `max_bytes` | `1048576` | 1 到 33554432（32 MiB） |

### 3.3 Edge 字段

| 字段 | 规则 |
| --- | --- |
| `from`、`to` | 都必须是节点 `id`；`from ≠ to` |
| `kind` | `main`、`back`、`branch`、`re_review` 之一 |

`kind` 只是给协调者看的标签，引擎对四种边的合法性判断完全相同。约定含义：`main` 主干推进；`back` 打回上游；`branch` 进入修复旁支；`re_review` 从旁支回到审查节点。

## 4. 编译校验

装入时按顺序检查，任一失败则报 `FLOW_INVALID` 并附字段路径与原因：

1. 所有 ID 合规、唯一；节点 id 不得是保留字 `start`、`resource`；`entry` 存在。
2. 每条边两端存在、不自环、`(from, to)` 不重复。
3. 从 `entry` 出发每个节点可达。不可达节点是错误，不是警告。
4. 至少存在一个没有出边的节点（终点）。
5. `inputs[].from` 引用的节点与输出存在，且从被引用节点出发能沿边走到本节点。这是静态能做的全部检查；运行时若上游还没有成功的 Attempt，`attempt begin` 报 `INPUT_UNAVAILABLE`。
6. `gate = true` 的节点 `instruction` 不得为空文本。
7. 文件引用存在、大小合规；`instruction.file` 另须 UTF-8，`resource.<path>` 不限编码。
8. 节点 `requires[]` 的每项都能在 `workbook.toml` 的 `requires` 里找到；同一节点内不重复。
9. `executor = human` 的节点不得声明 `tier`。

**为什么允许有环。** `back` 与 `re_review` 边形成环，这是修复回环的本意。`max_visits` 给每个节点一个硬上限，保证任何路径有限。编译期不做环检测，运行期靠计数。

## 5. 说明书写法

`instruction` 是给工作 agent 读的自然语言，用法和一份 skill 一样：写方向、标准、产出要求，留发挥空间。三条建议：

- 写清「读什么、产出什么、结论写在哪」。例：「读 `article`，按下面三条标准审查，把结论写进 `review.md`，第一行只写『通过』或『不通过』。」
- 不写运行时才知道的路径。引擎会在任务书里把每个输入的绝对路径和每个输出的目标路径附在说明之后。
- 不写「然后走哪条边」。选边是协调者读了输出文档之后的事。
- 节点有多个可选输入时，用任务书的「来自」行告诉工作 agent 读哪一份。可选输入一旦绑定过就会一直带着上一次的内容，「来自」是区分新旧的机械依据。
- 让每份输出文档的第一行成为结论。协调者按第一行选边，下游按第一行决定读不读。

## 6. 三份样例 Workbook

仓库 `examples/` 下维护三份样例，它们同时是端到端测试的 fixture：

| 目录 | 证明什么 |
| --- | --- |
| `examples/two-step/` | 两个 `agent` 节点，一条 `main` 边，无审查、无门槛。证明业务无关与最小闭环 |
| `examples/article-review/` | 即本文 §3 的图。证明 `back` 回环、`max_visits`、`human` 执行者、`resource.<path>` 输入 |
| `examples/gated-release/` | 一个 `agent` 节点 `gate = true`，后接一个终点节点。证明门槛阻断与 `gate approve` |

另有一份完整的业务 Workbook `workbooks/spec-dev/`（规格驱动的软件开发，十个节点、二十三条边、可选输入、`tier` 标签、两个人审节点、验证与审查两个独立回环、卡住时升级给人的旁支）。它不是测试 fixture，但 T11 的编译测试同样覆盖它，保证合同改动不会让它失效。

样例文件的字节由测试固定。实现与样例不一致时，先改合同与样例再改实现，不能改期望迎合实现。
