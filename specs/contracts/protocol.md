# 公开操作、状态卡与错误

本合同定义协调者与人能对引擎做的全部操作。MVP 只有 CLI 一个接口；MCP 接口是同一组操作的薄封装（见 [路线图](../roadmap.md)）。响应格式版本串 `cli-result/v1`。

## 1. 全局约定

```text
sheltie [--json] [--home <dir>] <group> <verb> [args]
```

| 旗标 | 含义 |
| --- | --- |
| `--json` | 输出一行 JSON（§5 的响应封装）。不带时输出给人读的文本 |
| `--home <dir>` | 覆盖管理根。默认取 `SHELTIE_HOME`，再默认 `~/.sheltie` |
| `--request-id <uuid>` | 写操作可选。不给时引擎生成并在响应里返回。协调者若要安全重试，先记下 id 再调用 |

**主体。** 每次调用的操作者身份取当前操作系统用户名，记进审计与批准记录。MVP 是单用户本地工具，不做认证。

**只读操作**（`list`、`show`、`status`、`stats`、`verify`、`self version`）不改任何状态，不建目录，不刷状态卡。

## 2. 操作一览

| 命令 | 写 | 作用 |
| --- | --- | --- |
| `self install [--modify-path]` | 是 | 把当前运行的二进制装到 `bin/sheltie`，建管理根 |
| `self update [--version <v>]` | 是 | 下载新版本，校验，原子替换 |
| `self rollback` | 是 | 换回上一版本 |
| `self uninstall [--purge]` | 是 | 删 `bin/`；`--purge` 才删整个管理根 |
| `self version` | 否 | 版本、平台、管理根、`SCHEMA_VERSION` |
| `workbook add <dir>` | 是 | 校验并复制 Workbook 到管理根 |
| `workbook list` | 否 | 列出已装 Workbook 的 id、版本、名称，标出每个 id 的最高版本 |
| `workbook show <id>[@<version>]` | 否 | 打印 manifest、宿主资源声明与每个 Flow 的节点、边 |
| `workbook remove <id>@<version>` | 是 | 删除一个已装版本；有非终态 Work 引用时拒绝 |
| `workbook verify [<id>@<version>]` | 否 | 重算目录摘要与库中记录对比 |
| `work start --workbook <id>[@<version>] --flow <flow> [--name <n>] [--input k=v]...` | 是 | 创建 Work，冻结一份 Workbook 副本 |
| `work list` | 否 | 列出 Work 的 id、名称、状态、当前节点、更新时间 |
| `work status <work>` | 否 | 打印状态卡（文本或 JSON） |
| `work stats <work>` | 否 | 打印事实视图：每个节点到达、尝试、失败几次，平均耗时，从哪进来 |
| `work cancel <work>` | 是 | 取消 |
| `attempt begin <work> --node <node>` | 是 | 进入节点并开始一次尝试；返回任务书 |
| `attempt submit <work> --attempt <id> --summary <text\|@file>` | 是 | 提交尝试；引擎校验并封存输出 |
| `attempt fail <work> --attempt <id> --reason <text>` | 是 | 标记尝试失败 |
| `gate approve <work> --node <node>` | 是 | 真人批准门槛 |

`<work>` 接受完整 `work_id`（如 `2026-09-24-001-article`）或唯一前缀（如 `2026-09-24-001`）。多个匹配报 `INVALID_REQUEST` 并列出候选。`--input k=v` 的 `v` 以 `@` 开头时读文件内容。`--version` 省略时取该 id 已装的最高版本（字面排序）。

## 3. 各操作细则

### `self` 组

`self install`。把 `std::env::current_exe()` 复制到 `bin/sheltie`，建 `store.db`。已存在且字节相同则直接返回 `data.already_installed = true`（与[存储合同 §9](storage.md) 同一口径，见 [v0.1.0 decision log](../releases/v0.1.0/decisions.md) D-31）。默认只打印一行「把 `~/.sheltie/bin` 加进 PATH」的提示；`--modify-path` 才往 shell rc 文件追加一行，追加前打印将写入的文件与内容。

`self update`。按 [存储合同 §9](storage.md) 的顺序：下载到 `tmp/`、核对 sha256、`sheltie` 挪到 `sheltie.prev`、`rename` 新文件到位。没有对应平台的发布报 `UPDATE_UNAVAILABLE`；摘要不符报 `UPDATE_CHECKSUM_MISMATCH` 并删下载文件。成功返回 `{ from, to }`。已是最新返回 `data.up_to_date = true`。

`self rollback`。`sheltie.prev` 不存在报 `NOT_FOUND`。只保留一级。

`self uninstall`。默认只删 `bin/`，`store.db`、`workbooks/`、`works/` 全部保留并打印它们的路径。`--purge` 删整个管理根，执行前打印将删除的路径并要求输入 `yes`（`--json` 模式下必须同时给 `--yes`）。

`self version`。只读，不需要管理根存在。

### `workbook add <dir>`

按 [Workbook 合同](workbook.md) §1 校验并复制。成功返回 `{ id, version, digest, flows: [...], requires: [...] }`。`digest` 是目录内全部文件按相对路径排序后拼接 `路径\0内容` 的 sha256。

### `workbook remove <id>@<version>`

1. 版本必须给全，不接受「最高版本」默认，防误删。
2. 查 `works` 表：任何 `status ∈ {active, blocked}` 且 `workbook` 等于该 id 与版本的 Work 存在，报 `WORKBOOK_IN_USE`，`detail.works` 列出它们。终态 Work 不阻断，它们各有冻结副本。
3. 事务内删行，提交后删目录。目录删除失败只记日志，下次 `workbook add` 顺手清。

### `workbook verify [<id>@<version>]`

省略参数时核对全部已装版本。对每个版本重算目录摘要与 `workbooks.digest` 对比，输出一张表：`ok | tampered | missing`。任一非 `ok` 则 `ok = false`，错误码 `WORKBOOK_TAMPERED`，`detail.results` 是整张表。

### `work start`

1. 找到 Workbook 与 Flow，编译图。
2. 核对起始输入：Flow 里所有 `start.<key>` 引用的键都必须给出；多给的键拒绝。
3. 规范化名字：`--name` 省略时取 `flow` id；去首尾空白，连续空白替换为一个 `-`，转小写。规范化后必须只含小写字母、数字、汉字与单个 `-`（不以 `-` 开头或结尾，无连续 `-`）且 ≤ 48 字节，否则 `INVALID_REQUEST`。「汉字」是下列码点区间的闭集，与实现逐区间一致：`3400–4DBF`（扩展 A）、`4E00–9FFF`（基本区）、`F900–FAFF`（兼容）、`20000–2A6DF`（B）、`2A700–2B73F`（C）、`2B740–2B81F`（D）、`2B820–2CEAF`（E）、`2CEB0–2EBEF`（F）、`2EBF0–2EE5F`（I）、`2F800–2FA1F`（兼容补充）、`30000–3134F`（G）、`31350–323AF`（H）。部首、康熙部首、`〇` 等 `Han` 脚本的其他码点不接受。
4. 分配 `work_id = <UTC 日期 YYYY-MM-DD>-<当日序号 001..999>-<名字>`。序号按 UTC 日期在 SQLite 事务内递增（[存储合同 §7](storage.md)），同一天第 1000 个 Work 报 `INVALID_REQUEST`。
5. 建目录 `works/<work_id>/`。把整个 Workbook 目录复制到 `works/<work_id>/workbook/` 并置只读，这是本 Work 的冻结定义，之后每次操作都从这里加载，不再读 `workbooks/`。
6. 建 `inputs/`，把每个输入值写成文件 `inputs/<key>`，记 `ArtifactRef`。
7. `current = entry#1`，`status = active`。
8. 返回 `{ work_id, name, workbook: { id, version, digest }, flow, work_dir, requires: [...] }` 与 `next`。`requires` 是 Workbook 声明的全部宿主资源，按 manifest 声明顺序，每项原样是那条声明 `{ kind, name, version, digest, source }`（没写的字段为 `null`；`digest` 与其他回复一样是裸 64 位十六进制，不带 `sha256:` 前缀），供协调者在开工前自行确认；MVP 的引擎不检查宿主。

例：`sheltie work start --workbook article-review --flow default --name "文章 初稿"` 得到 `2026-09-24-003-文章-初稿`。

### `attempt begin <work> --node <node>`

1. `node` 必须出现在当前 `next` 里，否则 `ILLEGAL_NEXT`（响应里附上当前 `next`）。
2. 若 `node ≠ current.node`：按边进入，`visits[node] += 1`，`current = node#n`，记下来自哪个 Occurrence 与边类型。重试时沿用上一次的来源。
3. 绑定输入：对每个 `inputs[]`，找到来源文件，重算 sha256 与已记录值核对。不符报 `ARTIFACT_MODIFIED`。来源为 `resource.<path>` 的输入读 Work 的冻结副本，它没有单独记录的摘要，由副本整体摘要覆盖：副本缺失或摘要不符报 `STORE_CORRUPT`（[存储合同 §5.1](storage.md)），不报 `ARTIFACT_MODIFIED`。上游还没成功产出时，`required = true` 报 `INPUT_UNAVAILABLE`，`required = false` 则不绑定，任务书标「尚无」。
4. 建 Attempt 目录 `attempts/<node>/<n>/<retry>/`，写 `brief.md`（§4）。
5. 返回 `{ attempt_id, node, occurrence, retry, brief_path, output_dir, inputs: {name: path}, outputs: {name: path}, requires: [...] }`。

`inputs` 与 `outputs` 里的路径都是绝对路径；来源为 `resource.<path>` 的输入指向 `works/<work_id>/workbook/<path>`。`requires` 是本节点引用的宿主资源，按节点里的书写顺序，每项是 manifest 里对应的那条声明，格式同 `work start`。协调者把 `brief_path` 交给工作 agent 即可。

### `attempt submit <work> --attempt <id> --summary <text>`

1. Attempt 必须 `running`，否则 `ATTEMPT_NOT_RUNNING`。
2. `summary` ≤ 4096 字节，否则 `SUMMARY_TOO_LONG`。
3. 对每个声明输出：文件在 `output_dir/<path>` 存在且是普通文件；`required = true` 缺失报 `OUTPUT_MISSING`；大小超 `max_bytes` 报 `OUTPUT_TOO_LARGE`。任一失败则 Attempt 仍 `running`，不改任何状态。
4. 记录每个输出的 `ArtifactRef`，文件置只读。
5. Attempt → `succeeded`。然后按顺序判断：节点 `gate = true` → Work `blocked(gate)`；节点无出边 → Work `succeeded`；有出边但每条的目标都已达 `max_visits` → Work `blocked(no_legal_edge)`；否则保持 `active`。
6. 返回 `{ attempt_id, outputs: {name: ArtifactRef}, work_status }` 与 `next`。

### `attempt fail <work> --attempt <id> --reason <text>`

Attempt → `failed`，记 `reason`（≤ 4096 字节）。`max_retries = k` 表示同一 Occurrence 最多 `k + 1` 次尝试；若这次失败的是第 `k + 1` 次（`retry == max_retries`），Work → `blocked(retries_exhausted)`。返回 `next`。

### `gate approve <work> --node <node>`

Work 必须是 `blocked(gate)` 且 `node = current.node`，否则 `ILLEGAL_NEXT`。记录 `{ node, occurrence, by, at }`。然后按 `attempt submit` 第 5 步除门槛之外的规则决定 Work 状态：无出边 → `succeeded`；无合法边 → `blocked(no_legal_edge)`；否则 `active`。返回 `next`。

### `work stats <work>`

只读。对 `WorkState` 做计数，不含任何判断：

```text
# Stats 2026-09-24-001-t

status: active   total: 3120s   blocked: 1   approvals: 0

| node | visits | attempts | failed | avg | entered_via |
| --- | --- | --- | --- | --- | --- |
| draft | 2/3 | 3 | 1 | 640s | entry×1, review×1 |
| review | 1/3 | 1 | 0 | 150s | draft×1 |
| publish | 0/1 | 0 | 0 | 0s |  |
```

行按图声明顺序。`total` 是 `created_at` 到 `updated_at`。`blocked` 是 Work 被挡住的累计次数：每个节点每个 Occurrence 只看最后一次 Attempt，`gate` 节点提交成功（含已批准）算一次，用尽重试的失败算一次，Work 当前停在 `blocked(no_legal_edge)` 再算一次。`approvals` 是 `gate approve` 的次数。`avg` 是该节点已结束 Attempt 的平均耗时。`entered_via` 按首次出现顺序列出 `来源节点×次数`，入口写 `entry`。`--json` 输出同样字段（`nodes[]` 各项 `node / visits / max_visits / attempts / failed / avg_seconds / entered_via`）。

`engine.stats` 输入绑定的就是这份 JSON：`attempt begin` 时引擎把它写到 `attempts/<node>/<n>/<retry>/stats.json`，记 sha256，任务书输入表里像其他文件一样列出。口径含本次 Attempt 本身（当前节点的 `visits` 与 `attempts` 都已计入，`total_seconds` 算到本次提交时刻）：只有用提交后的完整状态计算，崩溃后的重放才能从库里的状态逐字节重建同一份文件（[v0.1.0 decision log](../releases/v0.1.0/decisions.md) D-29）。

### `work cancel <work>`

非终态即可。Work → `cancelled`。正在 `running` 的 Attempt 保持原样，不伪造结束。

## 4. 任务书 `brief.md`

引擎生成，写在 Attempt 目录。正文是说明书原文，前后各加一段引擎生成的固定格式：

```markdown
# 任务书：<node.title>

Work: <work_id>（<name>）
节点: <node>#<n>，第 <retry> 次尝试
来自: <上游节点>#<m>（<edge.kind> 边）        入口节点写「入口」
执行者: agent（standard）                      human 节点只写 human

## 输入

| 名称 | 路径 | sha256 |
| --- | --- | --- |
| topic | /Users/me/.sheltie/works/<id>/inputs/topic | 3f2a… |
| checklist | /Users/me/.sheltie/works/<id>/workbook/resources/review-checklist.md | 8b40… |
| decision | 尚无（上游 plan-review 还没有产出） | |

## 需要的宿主资源

| 类型 | 名称 | 版本 | 说明 |
| --- | --- | --- | --- |
| skill | company-api | ^1 | 请确认你的宿主已装此 skill；未装请停下并告知用户 |

（节点没有声明时省略本节。「此 skill」随类型变成「此 agent」「此 mcp」；版本没写时填 `-`。）

## 说明

<instruction 原文，逐字；末尾的换行去掉，由模板统一换行>

## 输出要求

| 名称 | 写到 | 必需 | 上限 |
| --- | --- | --- | --- |
| article | /Users/me/.sheltie/works/<id>/attempts/draft/1/0/article.md | 是 | 256 KiB |

完成后不要自己修改输入文件。回复协调者时用几句话说明结论，并列出你写了哪些输出文件。
```

执行者为 `human` 时，最后一段换成提交命令，人写完输出后直接运行：

```markdown
写完输出文件后，在终端运行：

    sheltie attempt submit <work_id> --attempt <attempt_id> --summary "<一句话结论>"
```

协调者可以在交给工作 agent 前在任务书后追加上下文，或改写措辞，但不得改变说明书的原意（方向、标准、产出要求）。改写后的版本由协调者自己保存，引擎不收。

## 5. 响应封装（`--json`）

成功：

```json
{
  "ok": true,
  "request_id": "0192…",
  "revision": 7,
  "data": { },
  "next": [
    { "op": "attempt begin", "args": { "work": "…", "node": "review" }, "edge": "main", "executor": "agent", "tier": "strong" },
    { "op": "attempt begin", "args": { "work": "…", "node": "draft" },  "edge": "back", "executor": "agent", "tier": "standard" },
    { "op": "work cancel",   "args": { "work": "…" } }
  ]
}
```

失败：

```json
{
  "ok": false,
  "error": { "code": "OUTPUT_MISSING", "message": "必需输出 article 不存在：/…/article.md", "detail": { "output": "article" } },
  "next": [ ]
}
```

`next` 把当前合法下一步列成命令行，每项能直接执行。只读操作也带 `next`。`edge` 只在 `attempt begin` 进入另一节点时出现；`executor` 与 `tier` 只在 `attempt begin` 项上出现，让协调者在派活前就知道该找谁、用什么模型。

退出码：成功 `0`；`ok = false` 时 `1`；参数解析错误 `2`。

## 6. 状态卡 `status-card.md`

每次写操作提交后重写到 `works/<work_id>/status-card.md`，`work status` 打印同样内容。有界，无历史正文，只有指针。

```markdown
# Work <work_id>（<name>）

workbook: article-review@1.0.0   flow: default   status: active
current: review#2
done: draft#1, review#1, draft#2
pending: publish
visits: draft 2/3, review 2/3, publish 0/1

## 最近一次尝试

review#1.0 succeeded
summary: 不通过。第二段论据不足，见 review.md 第 12 行。
outputs:
  verdict → /…/attempts/review/1/0/review.md (sha256 9c1e…, 2.1 KiB)

## 合法下一步

- sheltie attempt begin <work_id> --node review
- sheltie work cancel <work_id>
```

`--json` 时输出同样字段的结构化形式。`done` 列出所有成功的 Occurrence；`pending` 列出从未到达的节点；`blocked` 存在时另起一行说明原因（`gate: review#2 需要 gate approve`、`retries_exhausted: draft#2` 或 `no_legal_edge: review#3 的全部出边目标已达 max_visits`）。

## 7. 错误码闭集

每个错误都属于且只属于下表一项。新增错误必须改本表并加一条测试。

| 代码 | 含义 | 已发生什么 | 下一步 |
| --- | --- | --- | --- |
| `INVALID_REQUEST` | 参数格式或取值不对 | 无变化 | 修参数 |
| `NOT_FOUND` | Workbook、Work、节点或 Attempt 不存在 | 无变化 | 核对 id |
| `WORKBOOK_INVALID` | manifest 或文件引用不合规，`detail.path` 指出位置 | 未复制任何文件 | 修 Workbook |
| `FLOW_INVALID` | 图编译失败，`detail.path` 与 `detail.rule` 指出哪条规则 | 未复制任何文件 | 修 Flow |
| `WORKBOOK_EXISTS` | 同 id 同版本已装 | 无变化 | 升版本 |
| `WORKBOOK_IN_USE` | 有非终态 Work 引用该版本，`detail.works` | 无变化 | 先完成或取消这些 Work |
| `WORKBOOK_TAMPERED` | 已装目录摘要与记录不符或目录缺失，`detail.results` | 无变化 | 重新 `workbook add`；运行中的 Work 不受影响 |
| `UPDATE_UNAVAILABLE` | 没有当前平台的发布，或网络不可达，`detail.reason` | 无变化 | 稍后重试或手工安装 |
| `UPDATE_CHECKSUM_MISMATCH` | 下载文件摘要与发布清单不符 | 下载文件已删 | 重试；仍失败则报告 |
| `INPUT_MISSING` | `work start` 缺起始输入键 | 无变化 | 补 `--input` |
| `WORK_TERMINAL` | Work 已是终态 | 无变化 | 无 |
| `ILLEGAL_NEXT` | 操作不在当前 `next` 里，`detail.next` 给出合法集合 | 无变化 | 从 `next` 里选 |
| `INPUT_UNAVAILABLE` | 上游节点还没有成功产出 | 未进入节点 | 先完成上游 |
| `ARTIFACT_MODIFIED` | 输入文件当前摘要与记录不符，`detail.path` | 未进入节点 | 人工核查该文件 |
| `ATTEMPT_NOT_RUNNING` | 对非 `running` 的 Attempt 提交或标失败 | 无变化 | 看状态卡 |
| `SUMMARY_TOO_LONG` | 摘要超 4096 字节 | 无变化 | 缩短，细节放文件 |
| `OUTPUT_MISSING` | 必需输出文件不存在，`detail.output` | 无变化，Attempt 仍 `running` | 让工作 agent 补文件 |
| `OUTPUT_TOO_LARGE` | 输出超 `max_bytes` | 同上 | 缩小 |
| `REQUEST_CONFLICT` | 同 `request_id` 不同载荷 | 无变化 | 换新 id |
| `REVISION_CONFLICT` | 并发写入，`expected_revision` 不符 | 无变化 | 重读状态卡再试 |
| `STORE_SCHEMA_MISMATCH` | 数据库结构与 `SCHEMA_VERSION = 1` 不符 | 未打开 | 换 `--home` 或删库重建 |
| `STORE_CORRUPT` | 数据库内容解不出合法状态 | 未修改 | 人工处理 |
| `IO` | 文件系统错误，`detail.path` 与系统错误文本 | 视具体操作，响应说明 | 检查权限与磁盘 |

**重放不是错误。** 同 `request_id` 同载荷返回原响应，`ok = true`，`data.replayed = true`。
