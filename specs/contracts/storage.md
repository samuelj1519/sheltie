# 存储、事务与恢复

本合同定义 `~/.sheltie` 下的持久结构与写入规则。SQLite 是唯一状态权威（`INV-7`）；目录树里的一切都是投影或产物。schema 版本 `2`；schema 1 的旧库被整体拒绝，不迁移、不清空（D-033）。

## 1. SQLite

文件 `store.db`。打开参数：WAL 模式，`synchronous = FULL`，`foreign_keys = ON`，`busy_timeout = 5000`。每次 CLI 调用开一个连接，结束即关，不做连接池。

### 1.1 版本与结构校验

`PRAGMA user_version` 存 `SCHEMA_VERSION`，当前值 `2`，由 `sheltie-runtime/src/store/schema.rs` 的常量唯一定义。

打开时：

1. 先以只读方式识别：库文件不存在时，只读操作报 `NOT_FOUND`，不建库；写操作在取得管理根写锁后才建库（§2）。
2. 库已存在时，先只读查询 `user_version` 与 `sqlite_master`：`user_version ≠ 2` 报 `STORE_SCHEMA_MISMATCH`（detail 说明应换新管理根或用旧二进制查旧记录）；逐表比对建表语句与期望一致（忽略空白）。**拒绝之前不得对库文件做任何写入**——不改 journal mode、不写 PRAGMA、不建表。
3. 结构校验通过后，读写连接才设置 WAL 与 `synchronous = FULL`。
4. 不自动迁移，不清空。测试放一个 schema 1 的库作负例，断言拒绝且文件字节不变。

建库的 DDL 与 `PRAGMA user_version = 2` 在**同一个事务**里执行，避免半结构库；首次并发建库由管理根写锁串行化。

### 1.2 表

```sql
CREATE TABLE workbooks (
  id          TEXT NOT NULL,
  version     TEXT NOT NULL,
  digest      TEXT NOT NULL,           -- workbook-digest/v2 的 64 位十六进制
  dir         TEXT NOT NULL,           -- 相对管理根
  added_at    TEXT NOT NULL,           -- RFC 3339 UTC
  PRIMARY KEY (id, version)
);

CREATE TABLE works (
  work_id     TEXT PRIMARY KEY,
  revision    INTEGER NOT NULL,
  status      TEXT NOT NULL,           -- active | blocked | succeeded | cancelled，冗余列，只为 list 查询
  state_json  TEXT NOT NULL,           -- 完整 WorkState，serde_json，deny_unknown_fields
  created_at  TEXT NOT NULL,
  updated_at  TEXT NOT NULL
);

CREATE TABLE work_sequence (
  day          TEXT PRIMARY KEY,      -- UTC 日期 YYYY-MM-DD
  last         INTEGER NOT NULL       -- 当日已分配的最大序号，1..999
);

CREATE TABLE requests (
  request_id   TEXT PRIMARY KEY,
  intent_hash  TEXT NOT NULL,          -- RequestIntent canonical JSON 的 sha256（§2.1）
  work_id      TEXT,                   -- 解析后的完整 WorkId；Workbook 级写操作为 NULL
  reply_json   TEXT NOT NULL,          -- ResponseSnapshot：提交时的完整响应（cli-result/v2）
  effects_json TEXT NOT NULL,          -- 效果意图与历史字节登记（§3.2）；删除完成另核 .deleted 标记
  published    INTEGER NOT NULL,       -- 0 初始效果未完成；1 已完成。历史 write_file 可显式核对/补齐
  at           TEXT NOT NULL
);

CREATE TABLE audit (
  seq          INTEGER PRIMARY KEY AUTOINCREMENT,
  work_id      TEXT NOT NULL,          -- Workbook 级操作为空串
  revision     INTEGER NOT NULL,       -- Work 提交后的 revision；Workbook 操作为 0
  request_id   TEXT NOT NULL,
  principal    TEXT NOT NULL,          -- 真实 OS 主体（§1.3）
  command_json TEXT NOT NULL,          -- 去掉大字段后的 Command
  at           TEXT NOT NULL
);
```

一切以 `state_json` 为准。`status` 列由写入时从状态派生，读时不信它做业务判断，只用于 `work list` 排序过滤。`requests.effects_json` 记录 I/O 完成情况，不参与业务选边，不构成第二套 Work 状态。

### 1.3 主体

审计与批准记录的主体是发起调用的真实 OS 进程身份：unix 上通过安全 Rust API 取 effective uid 对应的账户名；查不到或名称不是 UTF-8 时记 `uid:<数值>`；不读 `USER`/`USERNAME` 环境变量（D-036）。这是记账事实，不是真人认证（宪章 §5）。

## 2. 写操作与锁

### 2.1 RequestIntent 与意图指纹

runtime 在进入写路径前构造 `RequestIntent`，一个枚举覆盖全部写操作：`StartWork`（用户给的 Workbook selector、Flow、规范化名字、起始输入参数）、`BeginAttempt / SubmitAttempt / FailAttempt / ApproveGate / CancelWork`（解析后的完整 `WorkId` 加节点/Attempt/summary/reason 等用户参数）、`AddWorkbook`（源目录参数的词法规范化绝对路径）、`RemoveWorkbook`（完整 id 与 version）。

- 意图只含用户参数与解析后的目标，不含观察结果、时钟、模型自报事实。`StartWork` 的每个输入按键排序，记录字面值或 `@file` 的词法规范化绝对路径；`SubmitAttempt` 的 `--summary @file` 同样只记录源路径。文件内容是首次执行时的观察结果，不进意图。`AddWorkbook` 的源路径也只做不访问文件系统的词法规范化，不能靠 `canonicalize` 使历史重放依赖源目录仍存在。
- `intent_hash` 是意图 canonical JSON 的 sha256：枚举使用固定 tag，结构字段顺序固定，输入键按字节排序，整数只用整数编码，不使用浮点数。以该结构的 `serde_json::to_vec` 字节为准，T07 固定独立测试向量。观察到的文件摘要或 `@file` 内容变化不改变意图指纹；同路径同请求重放返回原响应，新的 request-id 才重新读取当前内容。更换字面值、`@file` 路径或其他用户参数会触发 `REQUEST_CONFLICT`。
- 新请求的 `Work` 前缀在只读预检阶段从 `works` 表解析为唯一完整 `WorkId`；已存在的 Work 行即使终态也能解析，不靠读 Workbook。已有 `request_id` 先读该行的 `work_id`：Work 命令的原始前缀必须匹配这条已提交目标，再用该完整 id 构造意图比对；不重新用当前 Work 列表解析历史前缀。后来创建的 Work 即使让此前缀不再唯一，也不能使历史重放失败或改绑目标。不匹配则 `REQUEST_CONFLICT`。
- `start` 未显式给版本时，第一次解析得到的实际版本进入提交响应；重放先查 `requests` 记录，不重新解释「最新版本」，也不读取当前 Workbook 或 `@file`。`attempt submit --summary @file` 与 `workbook add` 的重放也不要求原文件或源目录仍存在。
- `self` 命令组没有 RequestIntent，也不支持 `request_id`（协议 §1）。

### 2.2 管理根写锁

`<管理根>/.lock` 是整个管理根的排他文件锁（`fs4`）：

- 每个**写**操作（Work 与 Workbook 的写动词、`self` 的全部写动词）都使用这把锁；只在操作允许创建新管理根且预检通过时才创建根与 `.lock`。除这两个锁前置对象外，取得锁前不创建管理文件或目录。进程退出由 OS 释放。锁内依次：重核 schema → 查重放 → 恢复未完成效果（§3.2）→ 准备 → 事务 → 发布 → 标记完成。
- 只读操作不获取锁、不创建锁文件。
- 锁只针对本地协作进程，不声称约束同用户手工改文件；SQLite 的 revision CAS 保留为事务边界校验，不做自动业务重试框架。
- `self uninstall --purge` 持锁删除整个管理根。等待者在获得锁后必须复核管理根与锁对象身份：`.lock` 路径仍存在且与锁定的文件是同一对象（同 dev/inode）；发现根已删除或重建就释放并整体重试，不沿旧 inode 继续写。

### 2.3 一次写事务

```text
只读预检（无锁）：解析用户参数与目标身份、识别 schema、查 request_id 可重放项；未命中才读取 @file/Workbook 并做确定性校验
取得管理根写锁（合法写操作才创建管理根与 .lock）
锁内重核：schema、request_id、受并发影响的前置事实；先恢复未完成效果
BEGIN IMMEDIATE
  SELECT intent_hash, reply_json, effects_json, published FROM requests WHERE request_id = ?
    命中且 hash 相同 → ROLLBACK，返回原 reply（恢复未完成效果后，replayed = true）
    命中且 hash 不同 → ROLLBACK，REQUEST_CONFLICT
  SELECT revision FROM works WHERE work_id = ?        （start 与 Workbook 写操作跳过）
    revision ≠ 调用前读到的 → ROLLBACK，REVISION_CONFLICT
  ── Work 新请求的 core::decide 已在事务外算好 Decision；重放不调用 core ──
  UPDATE/INSERT works 或 INSERT/DELETE workbooks（按操作选择）
  INSERT audit
  INSERT requests（intent_hash、reply_json、effects_json、published = 0）
COMMIT
发布效果（§3.2）；全部完成后 UPDATE requests SET published = 1
释放锁
```

规则：

- 事务内不读文件、不算摘要、不调模型。文件观察全部在 `BEGIN` 之前完成，随 `Command` 一起传进 core。
- `decide` 在事务外调用。锁内单写者的常态下 `REVISION_CONFLICT` 不会出现；仍保留 CAS 校验，命中即整个调用重新开始（重读、重观察、重决定），最多 3 次。
- 私有 `pending/` 准备在 `COMMIT` 前完成；最终目录发布、任务书写入、产物封存和状态卡刷新在 `COMMIT` 之后执行。这些是效果，不是 Work 状态。

## 3. 崩溃语义与恢复

### 3.1 窗口表

进程可能在任何时刻被杀。每个窗口的保证：

| 窗口 | 库 | 目录 | 下一次 |
| --- | --- | --- | --- |
| 只读预检失败 | 无变化（不存在的新根不建库） | 无变化 | 修参数或补输入后重试，可用同 request-id |
| 序号分配后、staging 完成 | 无 Work 行；可能留空号与本操作的 `pending/<内部 id>/` | 只有本操作的私有暂存；最终目录不存在 | 持写锁时核对暂存标记与「无 Store 引用」后清理；空号不回收 |
| COMMIT 前 | 无变化 | 同上 | 同 request-id 重试或换新请求，均从预检重走 |
| COMMIT 后、发布前 | 已提交；`requests.published = 0` | `pending/<内部 id>/payload/` 是唯一原件，**永不按年龄清理** | 下一次写操作在锁内先按 `effects_json` 发布，再处理新命令；同请求重放返回原响应 |
| 发布中（rename 后、标记前） | `published = 0` | 最终对象在位 | 恢复核对最终对象归属与摘要：同对象视为已完成；不同对象报错，不覆盖 |
| 发布失败（磁盘满、权限） | 已提交 | 部分 | 响应报 `EFFECT_PENDING`，携带 `committed = true`、request-id 与原响应；Work 写操作另带 revision，不回滚状态 |

### 3.2 效果登记

`requests.effects_json` 是效果对象数组，按下表六种 `kind` 登记：

| kind | 字段 | 恢复动作 |
| --- | --- | --- |
| `publish_dir` | `pending`（`pending/<id>/payload/`）、`final`、`owner`（`work:<work_id>` 或 `workbook:<id>@<version>`）、`digest` | `final` 不存在且 `pending` 在 → 核原件并 fsync 后 rename，置最终目录只读（Workbook）；仅 `final` 在 → 核归属与摘要后视为完成；两者都在或内容不符 → 停止，不覆盖、不删原件 |
| `prepare_attempt` | `work_id`、`attempt_id`、`dirs`（按父先于子排序的目录路径数组） | 从已提交 WorkState 核 Attempt 身份，安全建立 Attempt、`engine/`、`outputs/` 及声明输出所需的父目录；已有目录逐个核类型与归属，拒绝链接或文件占位 |
| `write_file` | `path`、`sha256`、`content`（精确字节） | 缺失则写；存在且摘要相同不写；不同是完整性错误，不掩盖修改 |
| `seal_outputs` | `refs`（原产物的完整 `ArtifactRef` 数组） | 对原引用核对后置只读；不存在或改变时停止，不重造、不越界 chmod |
| `delete_dir` | `pending`、`final`、`owner`、删除前的 `digest` | 核原目录身份与摘要后移入本操作 pending 再删；已移入的只删除同一对象。完成删除后写持久完成标记（§3.3）；两处都缺而无标记时不猜为成功 |
| `refresh_status_card` | `work_id` | 读取该 Work 最新的 `state_json` 生成当前状态卡；不保存历史卡字节，不用旧请求快照覆盖新 revision |

`publish_dir.digest` 对 Workbook 是登记的 `workbook-digest/v2`；对 Work 是冻结副本的同一摘要，另核 `WorkState` 中每个起始输入的 `ArtifactRef` 与 `work_id`。`delete_dir.digest` 是被删除 Workbook 在删除前的登记摘要。效果路径必须从管理根派生并核对目标身份，不直接信任 JSON 中的裸绝对路径。

`effects_json` 是 UTF-8 JSON 数组；每个对象有上表的 `kind` 和该行字段，拒绝未知字段。所有路径是相对管理根的 UTF-8 路径，`sha256`/`digest` 是裸 64 位小写十六进制。`write_file.content` 是 UTF-8 字符串；历史 brief 与 stats 均为 UTF-8，其 JSON 解码后的 UTF-8 字节就是要恢复的精确字节。`seal_outputs` 的原产物引用使用协议中的完整 `ArtifactRef`。恢复读取时先校验对象结构、路径与摘要，再执行文件动作；格式不符报 `STORE_CORRUPT`，不猜默认值。

恢复顺序：按 `audit.seq` 递增处理未完成请求（每个已提交的 Work/Workbook 请求同事务写一条 audit）；同请求先 `publish_dir`、`prepare_attempt`，再处理历史文件、封存或删除，最后处理 `refresh_status_card`。全部效果成功后置 `published = 1`，再清理只剩标记的暂存容器；清理是可重试的元数据维护，不改变已完成的业务效果。已 `published = 1` 的请求重放不重做 `publish_dir`、`prepare_attempt`、`seal_outputs`、`delete_dir` 或旧状态卡；但若该请求登记了 `write_file`，显式重放时仍逐项核历史文件：缺失且父目录仍可信时按提交字节恢复，字节不同或父目录缺失时停止并返回 `EFFECT_PENDING`。状态卡始终从**最新**状态生成（§6）。当前请求自身的效果失败返回 `committed = true` 与本请求原响应；旧请求 A 的效果阻断新请求 B 时，B 未进入新事务，返回 `committed = false`、B 的 request-id 与 `pending_request_id = A`，不把 A 的原响应写成 B 的结果。文件归属或摘要不符在 `detail.cause = STORE_CORRUPT` 中说明。

### 3.3 pending 与清理

`pending/` 只放引擎持锁创建的私有暂存：未提交的准备区、已提交未发布的 Work/Workbook 原件、待删除目录。每次操作生成内部 UUID v7，先独占创建并 fsync 侧车文件 `pending/<内部 id>.owner`，再 fsync `pending/` 使侧车目录项持久，最后独占创建 `pending/<内部 id>/payload/`；因此正常持久性保证下不会留下没有归属侧车的 payload。`.owner` 是单行 UTF-8 JSON 加换行，字段顺序固定为 `format`、`internal_id`、`request_id`、`op`；`format = "pending/v1"`，`internal_id` 与目录名相同，`request_id` 是本次请求 id，`op` 只取 `start_work | add_workbook | remove_workbook`。侧车不进入最终 Work 或 Workbook。提交后的 `effects_json` 记录 `payload/` 的路径和业务对象归属。恢复和清理前均核对根内路径、侧车、目录类型与 Store 引用，不跟随链接。发布 rename 前 fsync 原件与必要父目录，rename 后 fsync 源/目标父目录，再标记效果完成；平台文件 API 在 T04/T07 验证。

`delete_dir` 完成删除后，独占创建 `pending/<内部 id>.deleted`，写入并 fsync `{"format":"delete-complete/v1","internal_id":"<内部 id>"}` 加换行。若请求仍 `published = 0`，最终目录与 pending payload 都不存在时，只有合法完成标记才能证明删除完成；标记缺失或损坏时报 `EFFECT_PENDING`，说明结果不明并保留原响应，不能把外部删除误判为本操作成功。COMMIT 后唯一原件被外部删除时同样停止，不重造。清理规则：

- 持写锁时，只删除侧车合法且不被任何 `requests.effects_json` 引用的完整 `pending/<内部 id>/` 与对应侧车；这是 COMMIT 前失败的残留。只有侧车而无目录时可核格式后清理；侧车写入中崩溃形成的无目录残片保留并报告，但不阻断其他已归属请求。无侧车的目录、标记不符、路径异常或引用不明时停止并报告，不猜所有权。
- 已提交且 `published = 0` 的原件在效果完成前不得删除。`published = 1` 后，本请求只剩的空容器、`.owner` 与 `.deleted` 可在持锁时核 Store 记录后清理；清理失败不重做已完成效果，也不按年龄删。

`tmp/` 与 pending 无关：下载、解包等一次性暂存，任何进程可随时清理；写操作顺手清理其中修改时间超过 24 小时的条目，清理不跟随符号链接。过期、时间戳、目录名都不构成删除 pending 的依据。

只读查询遇到未发布 Work 或 Workbook：不执行恢复。Work 从 `works.state_json` 读状态，冻结副本按本 Work 的 `publish_dir` 效果指向的 `pending/<内部 id>/payload/workbook/` 读取；Workbook 的 list/show/verify 按本版本已提交 add 的 `publish_dir` 效果指向的 `pending/<内部 id>/payload/` 读取。响应标 `pending_publish: true`；verify 对 pending 原件校验登记摘要，不能把合法待发布版本报作 missing 或 tampered。`work start` 的无锁预检也可从该已提交 pending Workbook 编译 Flow；取得写锁后先恢复 add 效果，再从最终版本重核身份、摘要及受并发影响的前置事实，不能因最终路径暂缺而提前报 `NOT_FOUND`。这些读取只在 Store 存有归属效果且侧车合法时允许；只读与 rename 交错时按同一 effect 的归属在 pending/final 两处有限重读。超出界限报暂时 I/O 错误，不写文件、不回退到任意已装版本。

## 4. 产物封存

输出文件由工作 agent 写在 `attempts/<node>/occurrence-*/attempt-*/outputs/` 下。`attempt submit` 时 runtime：

1. 以不跟随符号链接的方式打开每个声明输出，在同一句柄上 `fstat`：必须是普通文件、`nlink = 1`；路径经 `confine()` 确认仍在 Attempt 目录内。
2. 先查大小上限，再从同一句柄流式读取算 sha256 与字节数（读到的字节数与 `fstat` 不符按实际观察为准）。
3. core 校验通过并提交后，在同一句柄上置只读。

**以 `ArtifactRef.sha256` 为准，不是只读位。** 下游 `attempt begin` 绑定输入时重算摘要核对；不符报 `ARTIFACT_MODIFIED`。这就是「输入按字节冻结」的实现。

起始输入在 `work start` 时写成 `start-inputs/<key>` 文件并同样记 `ArtifactRef`。

## 5. Workbook 仓库与目录摘要

### 5.1 workbook-digest/v2

目录摘要是下列精确字节流的一次 SHA256（十六进制）：

```text
ASCII "sheltie-workbook-digest/v2\0"
BE64(file_count)
对规范 UTF-8 相对路径按字节序排序的每个普通文件：
  BE64(path_byte_length) || path_bytes || BE64(content_byte_length) || content_bytes
```

- BE64 是 8 字节大端无符号整数；数量与长度都入流，文件边界无歧义（O07）。
- 路径按字节排序，不做 Unicode/大小写转换；路径不含 `.`、`..`、NUL 或空段。
- 空目录不参与摘要。拒绝符号链接、硬链接（`nlink > 1`）与特殊文件。
- 读取前先核对单文件与总量上限（§5.3），之后流式读取并准确计数，文件在读取间增长不能绕过限额。
- 实现不得以「每文件各自 sha256 再拼接」等双阶段变体冒充本格式；期望值必须由手工拼字节的独立测试给出。

### 5.2 add 与 remove 的事务

`workbook add <dir>`：

1. 只读预检：源目录可读、结构粗检（§5.3 限额、拒绝链接）。
2. 取写锁；按 §3.3 先持久化本操作的 `pending/<内部 id>.owner` 侧车及 `pending/` 目录项，再独占创建 `pending/<内部 id>/payload/`，受限复制源目录到 payload，逐文件 fsync。
3. **只对最终副本** parse manifest、parse+compile 每个 Flow、算 `workbook-digest/v2`；登记的 id/version/digest 全部来自副本。复制期间源目录变化造成不完整或非法副本时拒绝；副本完整合法时登记它的实际字节，不以预检时的源元数据冒充最终身份。
4. 一个事务：requests 查重（重放）→ `INSERT workbooks` + audit + requests（`effects_json = publish_dir`，`published = 0`）。主键冲突报 `WORKBOOK_EXISTS`。
5. COMMIT 后 rename `pending/<内部 id>/payload/ → workbooks/<id>/<version>/`，整棵含根置只读（目录 0555、文件 0444）；其他效果完成后先标 `published = 1`，再清理空容器与侧车。

每次 add 只操作自己的 pending 目录，不清理别人的；恢复未完成的旧 add 先于新 add。

`workbook remove <id>@<version>`：

1. 版本必须给全，不接受「最高版本」默认，防误删。
2. 一个事务：requests 查重 → 遍历全部 `works` 行并用 T14 的持久状态验证入口核对 `work_id`、revision、`status` 与 `state_json` 的一致性；任何损坏行都报 `STORE_CORRUPT`，不得靠冗余 `status` 列预筛而跳过 → 从已校验状态找引用该版本且非终态的 Work，有则 `ROLLBACK` 报 `WORKBOOK_IN_USE`（`detail.works` 列出）→ 删 `workbooks` 行 + audit + requests（`effects_json = delete_dir`，记录删除前 digest 与对象归属）。
3. COMMIT 后核对目录身份与摘要，再移入本操作 `pending/<内部 id>/payload/` 删除并写 `pending/<内部 id>.deleted` 完成标记；其他效果完成后标 `published = 1`。请求未标完成时，最终目录与 pending payload 都不存在却没有合法 `.deleted`，必须报 `EFFECT_PENDING` 并保留原响应；内容或归属不符时同样停止，不删除。
4. 完成的删除不会因历史请求重放再次执行；同 id/version 的新生命周期不被旧 remove 的重放删除、不被旧 add 的重放覆盖（请求按 intent_hash 区分；同 id 同 version 的新 add 是新请求）。

复制时拒绝符号链接、硬链接、非普通文件、含 `..` 的路径、单文件超 32 MiB、总量超 256 MiB；清单里的宿主元数据文件（如 `.DS_Store`）按 §5.3 处理。

### 5.3 源目录与宿主元数据

`manifest.version` 必须能作为单个安全目录段：在既有字符规则上再拒绝 `.`、`..` 与保留名 `.staging`。源目录只收普通文件；Finder 常见的 `.DS_Store` 等宿主元数据文件**准确拒绝**并点名文件，不静默忽略字节，让用户先清理再装。

### 5.4 Work 的冻结副本

`work start` 在分配 `work_id` 之后、写起始输入之前，持写锁按 §3.3 先持久化本操作的 `pending/<内部 id>.owner` 侧车及 `pending/` 目录项，再独占创建 `pending/<内部 id>/payload/`，把 `workbooks/<id>/<version>/` 整棵复制到 `payload/workbook/`。复制完成后重新计算副本摘要与登记值核对，不符报 `STORE_CORRUPT`；核对通过才置只读，并把起始输入写入 `payload/start-inputs/`。成功 COMMIT 后，`publish_dir` 才把 `payload/` rename 到 `works/<work_id>/`。`WorkState.workbook.digest` 记这份副本的摘要。之后所有对该 Work 的操作（编译图、读说明书、绑 `resource.<path>` 输入）只读冻结副本；尚未发布时按 §3.3 读同一 pending 原件。这样有三个结果：

- `workbook remove` 与运行中的 Work 无关，只需拦非终态引用作为安全网。
- 有人改了 `workbooks/` 下的文件，已开始的 Work 不受影响；`workbook verify` 能发现。
- 终态 Work 在 Workbook 被删后仍能完整 `work status`。

副本目录缺失或摘要不符时，对该 Work 的一切操作报 `STORE_CORRUPT`，不回退到仓库副本（未发布 Work 例外：按 §3.3 从 pending 原件读取）。

## 6. 历史文件与当前投影

`brief.md` 与 `engine/stats.json` 是历史事实：内容在提交前确定，精确字节存进 `requests.effects_json`，未完成效果恢复和已完成请求的显式重放都按 §3.2 的 `write_file` 核对或补齐，不从最新状态重算；已完成 `submit` 的重放不重做输出封存。`status-card.md` 是当前状态的投影：`refresh_status_card` 每次从最新 `WorkState` 生成，失败保留 `published = 0` 供下次写操作恢复；任何请求重放都不回写旧版本。

## 7. 时间与 ID

时间统一为秒精度的 RFC 3339 UTC 字符串，固定形如 `2026-09-24T03:00:00Z`（`YYYY-MM-DDTHH:MM:SSZ`，大写 `T` 与 `Z`，无小数秒、无时区偏移），由 runtime 取 `SystemTime::now()` 后放进 `Context` 传给 core。默认 `request_id` 是 UUID v7，在 runtime 生成。core 不碰时钟与随机数，测试里传固定值。core 的 `Timestamp` 构造与读取都校验这个格式，不合法就报错：格式固定，所以字符串字典序就是时间序。由 Unix 秒数换算（`from_unix_secs`）时超出 9999 年的输入饱和到 `9999-12-31T23:59:59Z`，引擎产生的时间永远是这个格式。

### 7.1 `work_id` 的序号分配

`work_id = <day>-<seq>-<name>`，`seq` 为三位十进制补零。分配发生在写锁内、目录物化之前，用一个独立的短事务：

```text
BEGIN IMMEDIATE
  INSERT INTO work_sequence (day, last) VALUES (?, 1)
    ON CONFLICT(day) DO UPDATE SET last = last + 1
  SELECT last FROM work_sequence WHERE day = ?
    last > 999 → ROLLBACK，INVALID_REQUEST
COMMIT
```

规则：

- 序号跨 Workbook、Flow 与名字共用一条计数，只按 UTC 日期分组。
- 分配后不回收。序号之后的步骤失败或崩溃，这个号就空着；`works` 表里没有对应行即可。序号**之前**的确定性拒绝（缺输入、非法名字、缺 Workbook/Flow）不进入本步（GF-30）。
- `day` 取 `Context.now` 的 UTC 日期，与 `created_at` 同源，避免跨午夜时日期与序号不一致。
- 名字部分只是可读后缀，身份由整个字符串承担；同名不同序号是不同 Work。
- 同 `request_id` 的重放查 `requests` 表命中即直接返回，不分配新号。

## 8. 清理

MVP 不提供 `clear`。删除一个 Work 的方法是手工删目录再删行，文档明说。带归属核对的 `work clear` 见 [路线图](../roadmap.md)。

`tmp/` 里的一切都可以在任何时刻删（§3.3）；`pending/` 永不按年龄清理。

## 9. 二进制自身

目录：

```text
~/.sheltie/
  bin/
    sheltie          当前版本
    sheltie.prev     上一版本，供回滚
  tmp/               下载与解包
  .lock              管理根写锁（§2.2）
```

`self install`：创建管理根目录（含父目录），把当前可执行文件复制到 `tmp/`，`fsync`，`rename` 到 `bin/sheltie`。已存在且字节相同则不动。不写任何 shell 配置；PATH 提示只是输出文本。`self` 的写动词同样持管理根写锁。

`self update` 的顺序：

1. 解析发布身份。未给 `--version` 用 latest；给了则固定为 tag `v<version>`，清单与资产都从同一 tag 取，不混用两次解析的结果。
2. 读该 tag 的发布清单，找到当前平台（Rust target triple）的包与 sha256。没有报 `UPDATE_UNAVAILABLE`。
3. 下载到 `tmp/<uuid>/`，算发布包文件的摘要，不符报 `UPDATE_CHECKSUM_MISMATCH` 并删 `tmp/<uuid>/`；通过后若是压缩包（`.tar.gz` / `.tar.xz`）则解包，取包内唯一名为 `sheltie` 的普通文件（cargo-dist 的布局是 `<产物名去掉扩展>/sheltie`），瘦格式的资产就是二进制本身，不解包。
4. `rename bin/sheltie → bin/sheltie.prev`（覆盖旧的 `.prev`）。
5. `rename tmp/<uuid>/sheltie → bin/sheltie`。
6. 删 `tmp/<uuid>/`。

第 4 步与第 5 步之间崩溃，`bin/sheltie` 不存在但 `.prev` 在；用户用 `~/.sheltie/bin/sheltie.prev self rollback` 恢复，`self rollback` 对此情形要能处理（`bin/sheltie` 缺失时直接把 `.prev` 挪回）。Unix 允许替换正在运行的可执行文件，当前进程继续用旧映像跑完。

`self update` 不改 `store.db`。新版本带更高的 `SCHEMA_VERSION` 时，下次任何操作按 §1.1 报 `STORE_SCHEMA_MISMATCH`；v0.2.0 的说明写明：旧二进制也拒绝 schema 2，rollback 回旧二进制要配旧管理根，新管理根不被旧二进制误写。

发布链用 `cargo-dist`：从 git tag 生成 GitHub Release、各平台压缩包、sha256 清单、`install.sh`。`self update` 读发布清单 `dist-manifest.json`，认两种写法：瘦格式 `{ version, assets: [{ platform, name, sha256 }] }`（本地发布目录与测试用，`SHELTIE_RELEASE_BASE` 指向本地目录时不联网），以及 cargo-dist 发布的完整清单（在 `selfmgmt` 里适配成同一形状）。本地发布目录按 `latest/dist-manifest.json` 与 `v<version>/dist-manifest.json` 镜像 tag 布局，与远端走同一解析路径。网络下载用系统 `curl`。不引 `axoupdater`（D-30）。二进制只包含 `sheltie` 一个可执行文件。
