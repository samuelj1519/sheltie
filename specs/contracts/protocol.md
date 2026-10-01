# 公开操作、状态卡与错误

本合同定义协调者与人能对引擎做的全部操作。MVP 只有 CLI 一个接口；MCP 接口是同一组操作的薄封装（见 [路线图](../roadmap.md)）。响应格式版本串 `cli-result/v2`：Work 与 Workbook 写操作的响应字段全部来自提交时快照，CLI 不在提交后回读状态拼数据。

## 1. 全局约定

```text
sheltie [--json] [--home <dir>] <group> <verb> [args]
```

| 旗标 | 含义 |
| --- | --- |
| `--json` | stdout输出一行JSON（§5响应封装）。维护告警独立写stderr，不改业务JSON。不带时输出人读文本 |
| `--home <dir>` | 覆盖管理根。默认取 `SHELTIE_HOME`，再默认 `~/.sheltie` |
| `--request-id <uuid>` | 仅 Work 与 Workbook 写操作可选。不给时引擎生成并在响应里返回。协调者若要安全重试，先记下 id 再调用。只读操作和整个 `self` 命令组都不支持：给出即 `INVALID_REQUEST`、退出码 2，查询不虚构 request-id |

**主体。** 每次调用的操作者身份取发起进程的真实 OS 账户（unix 的 effective uid 对应账户名；查不到或名称不是 UTF-8 时记 `uid:<数值>`），记进审计与批准记录；不采信 `USER`/`USERNAME` 环境变量。同一 OS 账户环境下不提供独立真人认证（宪章 §5）。

**只读操作**（`list`、`show`、`status`、`stats`、`verify`、`self version`）不改业务状态、不建Home或引擎`.lock`、不刷状态卡。WAL查询可按D-039维护已有Store的共享内存控制文件，或在WAL缺失时创建零字节WAL控制载体。`work start`确定性拒绝不建Home（GF-30）。`workbook add`源结构/类型/限额粗检失败不建Home；粗检通过后内容校验只针对锁内私有副本，失败不得登记业务行、request、audit或最终Workbook，可保留空schema 2控制Store、锁和自有未提交pending。

## 2. 操作一览

| 命令 | 写 | 作用 |
| --- | --- | --- |
| `self install` | 是 | 把当前运行的二进制装到 `bin/sheltie`，建管理根。不写 shell 配置 |
| `self update [--version <v>]` | 是 | 下载新版本，校验，原子替换 |
| `self rollback` | 是 | 换回上一版本 |
| `self uninstall [--purge]` | 是 | 默认删`bin/`；确认purge删用户数据/binary，保留空根和同一个`.lock` |
| `self version` | 否 | 版本、平台、管理根、`SCHEMA_VERSION` |
| `workbook add <dir>` | 是 | 校验并复制 Workbook 到管理根 |
| `workbook list` | 否 | 列出已装 Workbook 的 id、版本、名称，标出每个 id 的最高版本；待发布版本带 `pending_publish` |
| `workbook show <id>[@<version>]` | 否 | 打印 manifest、宿主资源声明与每个 Flow 的节点、边、有序起始输入键 |
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

`<work>` 接受完整 `work_id`（如 `2026-09-24-001-article`）或唯一前缀（如 `2026-09-24-001`）。新请求的前缀有多个匹配时报 `INVALID_REQUEST` 并列出候选；已有 `request_id` 的重放先与 `requests.work_id` 中的原目标核对，后来的同前缀 Work 不改变历史请求。`--input k=v` 的 `v` 以 `@` 开头时读文件内容。`--version` 省略时取该 id 已装的最高版本（字面排序）。

## 3. 各操作细则

### `self` 组

`self install`。把 `std::env::current_exe()` 复制到 `bin/sheltie`，先创建管理根目录（含父目录）再建 `store.db`。已存在且字节相同则直接返回 `data.already_installed = true`（与[存储合同 §9](storage.md) 同一口径，见 [v0.1.0 decision log](../releases/v0.1.0/decisions.md) D-31）。只打印一行「把 `~/.sheltie/bin` 加进 PATH」的提示文本，不写任何 shell 配置文件（引擎只写管理根，`INV-3`）。

`self update [--version <v>]`。发布身份先固定：未给版本用 latest，给了版本则锁定 tag `v<v>`，清单与资产都取自同一 tag（[存储合同 §9](storage.md)）。此后按存储合同的顺序：下载到 `tmp/`、核对 sha256、`sheltie` 挪到 `sheltie.prev`、`rename` 新文件到位。没有对应平台或版本的发布报 `UPDATE_UNAVAILABLE`；摘要不符报 `UPDATE_CHECKSUM_MISMATCH` 并删下载文件。成功返回 `{ from, to }`。已是最新返回 `data.up_to_date = true`。

`self rollback`。`sheltie.prev` 不存在报 `NOT_FOUND`。只保留一级。

`self uninstall`默认只删`bin/`并打印保留的Store/workbooks/works。`--purge`经显式确认后删所有用户数据/binary，保留空管理根及原`.lock`；执行前列出删除范围，文本模式输入`yes`，JSON模式给`--yes`。失败报告部分清理，重复purge可继续。

`self version`。只读，不需要管理根存在。

### `workbook add <dir>`

先按源目录参数构造意图并查 `request_id`，命中时不读取当前源目录；新请求才按 [Workbook 合同](workbook.md) §1 校验并复制。登记身份与摘要来自复制后的最终副本。成功返回 `{ id, version, digest, flows: [...], requires: [...] }`。`digest` 是 `workbook-digest/v2`（[存储合同 §5.1](storage.md)）。支持 `--request-id` 重放语义（GF-15）。

已提交但尚未发布的 add 在 `workbook list/show/verify` 中仍可按 [存储合同 §3.3](storage.md) 的同一 pending 原件只读访问；list 的对应项、show 的 `data`、verify 的对应结果均带 `pending_publish: true`，文本标「待发布」。verify 对原件摘要相符时 `status = ok`，不把暂缺最终目录误报为 `missing`。待发布的 `workbook remove` 则由下一次写操作先恢复后继续；只读遇其已删行不重新展示旧版本。

### `workbook remove <id>@<version>`

1. 版本必须给全，不接受「最高版本」默认，防误删。
2. 引用检查、删行、审计与请求登记在同一个事务（[存储合同 §5.2](storage.md)）：先校验全部 `works` 行的 `state_json` 与冗余列，再从已校验状态查非终态引用；有引用时报 `WORKBOOK_IN_USE`，`detail.works` 列出它们。损坏行报 `STORE_CORRUPT`，不得按冗余 `status` 预筛后跳过。终态 Work 不阻断，它们各有冻结副本。
3. 提交后把已核归属的目录移入本操作 pending 再删除；失败由效果恢复处理，不静默。成功 `data` 为 `{ id, version, replayed }`；支持 `--request-id` 重放语义。

### `workbook show <id>[@<version>]`

打印 manifest、宿主资源声明与每个 Flow 的节点、边，外加**有序起始输入键** `start_inputs`：该 Flow 全部 `start.<key>` 引用按节点声明顺序首次出现的列表。协调者第一次调用就能拿全开一个 Work 需要的键，不用失败 start 探测。文本与 JSON 都带；JSON 里 `flows[].start_inputs` 是字符串数组。

### `workbook verify [<id>@<version>]`

省略参数时核对全部已装版本。对每个版本重算目录摘要与 `workbooks.digest` 对比，输出一张表：`ok | tampered | missing`。任一非 `ok` 则 `ok = false`，错误码 `WORKBOOK_TAMPERED`，`detail.results` 是整张表。

### `work start`

以下步骤先做**无业务副作用预检**（GF-30）。命中已提交的 `request_id` 时，在读取当前 Workbook 或 `@file` 前按存储合同 §2.1 比对意图；相同则完成必要的既有效果恢复并返回原响应，不重新解释最新版本。未命中才继续确定性校验。校验失败时不分配当日序号、不创建最终目录、不登记请求，也不改变已有主数据库/WAL/业务文件；只读SQLite依D-039可能维护已有Store的`store.db-shm`或创建缺失的零字节`store.db-wal`，且不因此创建管理根、`.lock`或数据库。

1. 解析 `--input` 的键和字面值或 `@file` 词法路径，以用户提供的name原值及省略状态构造 `RequestIntent`；此时不规范化 `WorkName`、不读 `@file` 内容、不装入 Workbook。名字参数是意图的一部分，省略与显式flow名不同，见存储合同 §2.1。
2. 只读识别已有 Store 并查 `request_id`：意图相同按原快照重放，不同报 `REQUEST_CONFLICT`。已有请求的恢复失败按 §5 返回 `EFFECT_PENDING`。新请求才继续。
3. 仅新请求规范化WorkName：名字省略时取`flow` id；去首尾空白，连续空白替换为一个`-`，转小写。规范化后必须只含小写字母、数字、汉字与单个`-`（不以`-`开头或结尾，无连续`-`）且≤48字节，否则`INVALID_REQUEST`。「汉字」是下列码点区间的闭集，与实现逐区间一致：`3400–4DBF`（扩展 A）、`4E00–9FFF`（基本区）、`F900–FAFF`（兼容）、`20000–2A6DF`（B）、`2A700–2B73F`（C）、`2B740–2B81F`（D）、`2B820–2CEAF`（E）、`2CEB0–2EBEF`（F）、`2EBF0–2EE5F`（I）、`2F800–2FA1F`（兼容补充）、`30000–3134F`（G）、`31350–323AF`（H）。部首、康熙部首、`〇`等`Han`脚本的其他码点不接受。
4. 读取全部 `@file` 内容（失败退出码 2）；按 `--workbook` 找到已装版本并编译图。已提交未发布的 add 可按存储合同 §3.3 从受保护 pending 原件读取，锁内恢复后重核。确实缺 Workbook 或 Flow 才报 `NOT_FOUND`。
5. 核对起始输入：Flow 里所有 `start.<key>` 引用的键都必须给出；多给的键拒绝。键集合与顺序来自 core 的 `start_requirements`，与 `workbook show` 的 `start_inputs` 同源。

预检通过后进入[存储合同 §2.3](storage.md) 的写路径：

6. 分配 `work_id = <UTC 日期 YYYY-MM-DD>-<当日序号 001..999>-<名字>`。序号按 UTC 日期在 SQLite 事务内递增（[存储合同 §7](storage.md)），同一天第 1000 个 Work 报 `INVALID_REQUEST`。序号之后的失败会留下空号，这是接受的代价；序号之前的失败（上面五步）不烧号。
7. 在本操作自己的 `pending/<内部 id>/payload/` 里建 Work 目录：复制 Workbook 到 `workbook/` 并对副本重新核验摘要（[存储合同 §5.4](storage.md)），置只读；建 `start-inputs/`，把每个输入值写成文件 `start-inputs/<key>`，记 `ArtifactRef`。这是本 Work 的冻结定义，之后每次操作都从这里加载，不再读 `workbooks/`。
8. `current = entry#1`，`status = active`；COMMIT 后把 `payload/` rename 到 `works/<work_id>/` 并刷新状态卡。
9. 返回 `{ work_id, name, workbook: { id, version, digest }, flow, work_dir, requires: [...] }` 与 `next`，全部来自提交时快照。`requires` 是 Workbook 声明的全部宿主资源，按 manifest 声明顺序，每项原样是那条声明 `{ kind, name, version, digest, source }`（没写的字段为 `null`；`digest` 与其他回复一样是裸 64 位十六进制，不带 `sha256:` 前缀），供协调者在开工前自行确认；MVP 的引擎不检查宿主。

例：`sheltie work start --workbook article-review --flow default --name "文章 初稿"` 得到 `2026-09-24-003-文章-初稿`。

### `attempt begin <work> --node <node>`

1. `node` 必须出现在当前 `next` 里，否则 `ILLEGAL_NEXT`（响应里附上当前 `next`）。
2. 若 `node ≠ current.node`：按边进入，`visits[node] += 1`，`current = node#n`，记下来自哪个 Occurrence 与边类型。重试时沿用上一次的来源。
3. 绑定输入：对每个 `inputs[]`，找到来源文件，重算 sha256 与已记录值核对。不符报 `ARTIFACT_MODIFIED`。来源为 `resource.<path>` 的输入读 Work 的冻结副本，它没有单独记录的摘要，由副本整体摘要覆盖：副本缺失或摘要不符报 `STORE_CORRUPT`（[存储合同 §5.4](storage.md)），不报 `ARTIFACT_MODIFIED`。上游还没成功产出时，`required = true` 报 `INPUT_UNAVAILABLE`，`required = false` 则不绑定，任务书标「尚无」。
4. 在提交前确定 `brief.md`（§4）与 `engine/stats.json` 的精确字节和目标路径；COMMIT 后按 `prepare_attempt` 效果建立 Attempt 目录 `attempts/<node>/occurrence-<NNN>/attempt-<NNN>/`（标签零补齐三位，`AttemptId` 仍是 `node#n.retry`）、`engine/`、`outputs/` 与声明输出的父目录，再按 `write_file` 写入历史文件。效果失败按 §5 返回 `EFFECT_PENDING`。
5. 返回 `{ attempt_id, node, occurrence, retry, brief_path, output_dir, inputs: {name: path}, outputs: {name: path}, requires: [...] }`；`output_dir` 是 Attempt 目录下的 `outputs/`，`outputs` 各项是 `output_dir/<declared-path>`。

`inputs` 与 `outputs` 里的路径都是绝对路径；来源为 `resource.<path>` 的输入指向 `works/<work_id>/workbook/<path>`。第 3 步未绑定的可选输入仍在 `inputs` 里占一行，值是 `null`，任务书对它标「尚无」。`requires` 是本节点引用的宿主资源，按节点里的书写顺序，每项是 manifest 里对应的那条声明，格式同 `work start`。协调者把 `brief_path` 交给工作 agent 即可。

### `attempt submit <work> --attempt <id> --summary <text|@file>`

1. Attempt 必须 `running`，否则 `ATTEMPT_NOT_RUNNING`。
2. `summary` ≤ 4096 字节，否则 `SUMMARY_TOO_LONG`。
3. 对每个声明输出：文件在 `output_dir/<path>` 存在、是普通文件且非符号链接、硬链接计数为 1（[存储合同 §4](storage.md)）；`required = true` 缺失报 `OUTPUT_MISSING`；大小超 `max_bytes` 报 `OUTPUT_TOO_LARGE`。任一失败则 Attempt 仍 `running`，不改任何状态。
4. 在提交时记录每个输出的 `ArtifactRef`；COMMIT 后按效果登记封存同一文件对象，失败按 §5 返回 `EFFECT_PENDING`。
5. Attempt → `succeeded`。然后按顺序判断：节点 `gate = true` → Work `blocked(gate)`；节点无出边 → Work `succeeded`；有出边但每条的目标都已达 `max_visits` → Work `blocked(no_legal_edge)`；否则保持 `active`。
6. 返回 `{ attempt_id, outputs: {name: ArtifactRef}, work_status }` 与 `next`。

### `attempt fail <work> --attempt <id> --reason <text>`

Attempt → `failed`，记 `reason`（≤ 4096 字节）。`max_retries = k` 表示同一 Occurrence 最多 `k + 1` 次尝试；若这次失败的是第 `k + 1` 次（`retry == max_retries`），Work → `blocked(retries_exhausted)`。返回 `{ attempt_id, work_status }` 与 `next`。

### `gate approve <work> --node <node>`

Work 必须是 `blocked(gate)` 且 `node = current.node`，否则 `ILLEGAL_NEXT`。记录 `{ node, occurrence, by, at }`（`by` 是发起调用的 OS 账户）；用户授权后由 agent 代执行时，`by` 记的是 agent 进程的账户，如实呈现，不表述为「已验证独立真人」（宪章 §5）。然后按 `attempt submit` 第 5 步除门槛之外的规则决定 Work 状态：无出边 → `succeeded`；无合法边 → `blocked(no_legal_edge)`；否则 `active`。返回 `{ node, occurrence, by, at, work_status }` 与 `next`。

### `work stats <work>`

只读。对 `WorkState` 做计数，不含任何判断：

```text
# Stats 2026-09-24-001-t

status: active   total: 3120s   blocked: 1   approvals: 0

| node | visits | attempts | failed | avg | entered_via |
| --- | --- | --- | --- | --- | --- |
| draft | 2/3 | 3 | 1 | 640s | entry×1, review(back)×1 |
| review | 1/3 | 1 | 0 | 150s | draft(main)×1 |
| publish | 0/1 | 0 | 0 | 0s |  |
```

行按图声明顺序。`total` 是 `created_at` 到 `updated_at`。`approvals` 是 `gate approve` 的次数。`avg` 是该节点已结束 Attempt 的平均耗时。`entered_via` 按首次出现顺序列出 `来源节点(边类型)×次数`，入口写 `entry`。`--json` 输出同样字段（`nodes[]` 各项 `node / visits / max_visits / attempts / failed / avg_seconds / entered_via`；`entered_via` 是 `{"from": string|null, "edge": string|null, "count": n}` 数组）。

`entered_via` 保留边的类型：JSON 里是 `{ from, edge, count }` 数组（`from` 为 `null` 表示入口，`edge` 是 `main | back | branch | re_review`）；文本里写 `draft(back)×1`、`entry×1`。`blocked` 是累计受阻事实（GF-29）：gate 节点提交成功 +1，重试耗尽 +1，`no_legal_edge` 发生 +1；取消后不减少，由状态转换在发生时记录。

`engine.stats` 输入绑定的就是这份 JSON：`attempt begin` 时引擎把它写到 `attempts/<node>/occurrence-*/attempt-*/engine/stats.json`，记 sha256，任务书输入表里像其他文件一样列出。口径含本次 Attempt 本身（当前节点的 `visits` 与 `attempts` 都已计入，`total_seconds` 算到本次提交时刻）：内容在提交前定稿，精确字节登记进 `requests.effects_json`，崩溃后的重放按登记字节恢复同一份文件，不从最新状态重算（D-29 的口径不变，恢复方式按 v2）。

### `work cancel <work>`

非终态即可。Work → `cancelled`。正在 `running` 的 Attempt 保持原样，不伪造结束。返回 `{ work_id, work_status: "cancelled" }` 与空 `next`。

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
| topic | /Users/me/.sheltie/works/<id>/start-inputs/topic | 3f2a… |
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
| article | /Users/me/.sheltie/works/<id>/attempts/draft/occurrence-001/attempt-000/outputs/article.md | 是 | 256 KiB |

完成后不要自己修改输入文件。回复协调者时用几句话说明结论，并列出你写了哪些输出文件。
```

执行者为 `human` 时，最后一段换成提交命令，人写完输出后直接运行：

```markdown
写完输出文件后，在终端运行：

    sheltie attempt submit <work_id> --attempt <attempt_id> --summary "<一句话结论>"
```

任务书里的输出路径都在 Attempt 目录的 `outputs/` 之下；`engine/stats.json` 在 `engine/` 之下，不占用输出命名空间。

协调者可以在交给工作 agent 前在任务书后追加上下文，或改写措辞，但不得改变说明书的原意（方向、标准、产出要求）。改写后的版本由协调者自己保存，引擎不收。

## 5. 响应封装（`--json`，`cli-result/v2`）

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

Work 与 Workbook 写操作的 `data.replayed` 首次为 `false`，重放为 `true`；其他业务字段来自提交时的 `ResponseSnapshot`（[存储合同 §1.2](storage.md)），历史 `next` 原样保留——它是历史响应的一部分，续接要查当前状态。`request_id` 在这些写操作成功或返回 `EFFECT_PENDING` 时出现；`revision` 只在已提交的 Work 写操作中出现。只读与 `self` 响应省略不适用字段，`self` 没有请求快照或 `replayed`；所有响应都有 `next` 数组，无合法下一步时为空。

失败：

```json
{
  "ok": false,
  "error": { "code": "OUTPUT_MISSING", "message": "必需输出 article 不存在：/…/article.md", "detail": { "output": "article" } },
  "next": [ ]
}
```

本次 Work/Workbook 写操作已提交，但自己的效果失败时，失败封装额外携带：

```json
{
  "ok": false,
  "error": { "code": "EFFECT_PENDING", "message": "…", "detail": { …恢复动作… } },
  "next": [ ],
  "committed": true,
  "revision": 7,
  "request_id": "0192…",
  "original": { "ok": true, "…": "提交时的完整成功响应" }
}
```

调用者据此知道**本次请求**已提交，用同一 `request_id` 重试触发恢复，不能当成未提交而换新请求。示例是 Work 写操作；Workbook 写操作的 `EFFECT_PENDING` 省略 `revision`，仍带 `request_id`、`committed` 与 `original`。

`original` 和 `pending_original` 只提供已核实的提交快照。字段类型合法、Reply与data相互一致，还不足以证明业务绑定；须核请求、audit、目标及冻结定义中的对应事实。元数据或绑定损坏时省略无法核实的原响应及revision，保留已确认的提交身份和准确cause，不构造成功回复、不修写历史记录。已核合法快照遇到效果载荷、路径、sync或完成标记错误时，仍提供原响应。若Start或add的发布登记本身使快照身份无法独立核实，也省略原响应，不改用当前已装Workbook或猜测载荷。

若新请求 B 取得写锁后被旧请求 A 的未完成效果阻断，B 尚未提交。此时仍用 `EFFECT_PENDING`，但返回 `committed = false`、`request_id = B`，且 `error.detail.pending_request_id = A`、`error.detail.pending_original` 是 A 的提交时响应；不在顶层放 B 的 `original` 或 revision。调用者修复 A 的效果后，用 B 原来的 request-id 重试 B。不能把 A 的快照当作 B 的成功结果：

```json
{
  "ok": false,
  "error": { "code": "EFFECT_PENDING", "message": "旧请求的文件效果未完成", "detail": { "pending_request_id": "A", "pending_original": { "ok": true }, "cause": "IO" } },
  "next": [],
  "committed": false,
  "request_id": "B"
}
```

`next` 把当前合法下一步列成命令行，每项能直接执行。`next` 项的形状全协议只有一种（与状态卡 `data.next` 完全相同）：`op`、`args`、以及仅在 `attempt begin` 项上的 `edge`、`executor`、`tier`。只读操作也带 `next`。`executor` 与 `tier` 让协调者在派活前就知道该找谁、用什么模型。

新请求读取用户提供的`@file`时，缺失、不可读、非UTF-8、非普通文件、符号链接、硬链接（`nlink > 1`）或超出[存储合同§5.3](storage.md)的单文件读取上限（32 MiB）均报`INVALID_REQUEST`、退出码2，detail给path/reason。只有新请求读取文件；同路径重放不重新打开。文件源上限依据storage §5.3，摘要/原因文本本身超过4096字节仍报`SUMMARY_TOO_LONG`、退出码1，不与文件源上限合并。

退出码：成功 `0`；`ok = false` 时 `1`；参数解析错误 `2`。

## 6. 状态卡 `status-card.md`

每次影响该 Work 的写操作提交后，`refresh_status_card` 从最新状态重写 `works/<work_id>/status-card.md`；失败按 §5 返回 `EFFECT_PENDING`。`work status` 直接从已校验的 Store 状态生成同样内容，未发布时按存储合同 §3.3 读取受保护的 pending 原件。有界，无历史正文，只有指针。

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
  verdict → /…/attempts/review/occurrence-001/attempt-000/outputs/review.md (sha256 9c1e…, 2.1 KiB)

## 合法下一步

- sheltie attempt begin <work_id> --node review
- sheltie work cancel <work_id>
```

`--json` 时输出同样字段的结构化形式，字段与文本一一对应：`work_id / name / workbook / flow / status / current / done / pending / visits / blocked / last_attempt / next`。`last_attempt` 含 `attempt / status / summary / reason / outputs`：失败时 `reason` 是失败原因文本；`outputs` 是 `{name: {path, sha256, bytes}}` 的完整产物引用（与文本卡同样的路径、摘要与大小）。`blocked` 是与文本行相同的说明串（如 `gate: review#2 需要 gate approve`），无则 `null`。`next` 与 §5 响应封装的 `next` 项完全同形。Work 尚未发布完成时另带 `pending_publish: true`（[存储合同 §3.3](storage.md)）。

`done` 列出所有成功的 Occurrence；`pending` 列出从未到达的节点；`blocked` 存在时另起一行说明原因（`gate: review#2 需要 gate approve`、`retries_exhausted: draft#2` 或 `no_legal_edge: review#3 的全部出边目标已达 max_visits`）。

## 7. 错误码闭集

每个错误都属于且只属于下表一项。新增错误必须改本表并加一条测试。

| 代码 | 含义 | 已发生什么 | 下一步 |
| --- | --- | --- | --- |
| `INVALID_REQUEST` | 参数格式或取值不对 | 无变化 | 修参数 |
| `NOT_FOUND` | Workbook、Work、节点或 Attempt 不存在 | 无变化 | 核对 id |
| `WORKBOOK_INVALID` | manifest 或文件引用不合规，`detail.path` 指出位置 | 无 Workbook 行或最终目录；可能留本请求私有 pending | 修 Workbook；下次持锁写操作核归属后清理 pending |
| `FLOW_INVALID` | 图编译失败，`detail.path` 与 `detail.rule` 指出哪条规则 | 无 Workbook 行或最终目录；可能留本请求私有 pending | 修 Flow；下次持锁写操作核归属后清理 pending |
| `WORKBOOK_EXISTS` | 同 id 同版本已装 | 无变化 | 升版本 |
| `WORKBOOK_IN_USE` | 有非终态 Work 引用该版本，`detail.works` | 无变化 | 先完成或取消这些 Work |
| `WORKBOOK_TAMPERED` | 已装目录摘要与记录不符或目录缺失，`detail.results` | 无变化 | 人工核查并恢复已登记版本的原字节；同版本 `add` 不会覆盖，已有 Work 使用各自冻结副本 |
| `UPDATE_UNAVAILABLE` | 没有当前平台的发布，或网络不可达，`detail.reason` | 无变化 | 稍后重试或手工安装 |
| `UPDATE_CHECKSUM_MISMATCH` | 下载文件摘要与发布清单不符 | 下载文件已删 | 重试；仍失败则报告 |
| `INPUT_MISSING` | `work start` 缺起始输入键 | 无变化 | 补 `--input` |
| `WORK_TERMINAL` | Work 已是终态 | 无变化 | 无 |
| `ILLEGAL_NEXT` | 操作不在当前 `next` 里，`detail.next` 给出合法集合 | 无变化 | 从 `next` 里选 |
| `INPUT_UNAVAILABLE` | 上游节点还没有成功产出，`detail.input` 与 `detail.node` 指出哪条输入 | 未进入节点 | 先完成上游 |
| `ARTIFACT_MODIFIED` | 输入文件当前摘要与记录不符，`detail.path` | 未进入节点 | 人工核查该文件 |
| `ATTEMPT_NOT_RUNNING` | 对非 `running` 的 Attempt 提交或标失败 | 无变化 | 看状态卡 |
| `SUMMARY_TOO_LONG` | 摘要超 4096 字节 | 无变化 | 缩短，细节放文件 |
| `OUTPUT_MISSING` | 必需输出文件不存在，`detail.output` | 无变化，Attempt 仍 `running` | 让工作 agent 补文件 |
| `OUTPUT_TOO_LARGE` | 输出超 `max_bytes` | 同上 | 缩小 |
| `REQUEST_CONFLICT` | 同 `request_id` 不同目标或载荷 | 无变化 | 换新 id |
| `REVISION_CONFLICT` | 并发写入，`expected_revision` 不符 | 无变化 | 重读状态卡再试 |
| `EFFECT_PENDING` | 效果未完成：当前请求已提交时 `committed = true` 并带自己的原响应；旧效果阻断新请求时 `committed = false`、`pending_request_id` 指向旧请求 | 看 `committed`：当前请求已提交或尚未提交；旧效果仍待恢复 | 先恢复 `pending_request_id`（若有），再用本次 `request_id` 重试 |
| `STORE_SCHEMA_MISMATCH` | 数据库结构与 `SCHEMA_VERSION = 2` 不符（含 schema 1 旧库） | 未打开，拒绝前无任何写入 | 换新管理根；旧记录用旧二进制配旧管理根查 |
| `STORE_CORRUPT` | 数据库内容、持久身份或未提交阶段的受管文件不符合同 | 未提交新业务状态 | 人工核查；已提交效果中的完整性错误由 `EFFECT_PENDING.detail.cause` 指明 |
| `IO` | 文件系统错误，`detail.path` 与系统错误文本 | 视具体操作，响应说明 | 检查权限与磁盘 |

**重放不是错误。** 同 `request_id` 同意图（目标与用户参数相同）返回原响应，`ok = true`，`data.replayed = true`；观察到的文件变化不影响意图指纹。
