# 存储、事务与恢复

本合同定义 `~/.sheltie` 下的持久结构与写入规则。SQLite 是唯一状态权威（`INV-7`）；目录树里的一切都是投影或产物。

## 1. SQLite

文件 `store.db`。打开参数：WAL 模式，`synchronous = FULL`，`foreign_keys = ON`，`busy_timeout = 5000`。每次 CLI 调用开一个连接，结束即关，不做连接池。

### 1.1 版本与结构校验

`PRAGMA user_version` 存 `SCHEMA_VERSION`，当前值 `1`，由 `sheltie-runtime/src/store/schema.rs` 的常量唯一定义。

打开时：

1. 若库文件不存在且操作是写操作，建库并写入全部表。只读操作遇到不存在的库报 `NOT_FOUND`，不建库。
2. 若 `user_version ≠ 1`，报 `STORE_SCHEMA_MISMATCH`。
3. 对 `sqlite_master` 核对每张表的建表语句与期望完全一致（忽略空白）。不一致报 `STORE_SCHEMA_MISMATCH`。
4. 不自动迁移，不清空。测试里放一个手工造的旧结构库作负例。

后续改结构时只升 `SCHEMA_VERSION` 并同步建表语句与 fixture，旧库继续被拒绝。

### 1.2 表

```sql
CREATE TABLE workbooks (
  id          TEXT NOT NULL,
  version     TEXT NOT NULL,
  digest      TEXT NOT NULL,           -- sha256 hex
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
  work_id      TEXT,                   -- workbook add 为 NULL
  payload_hash TEXT NOT NULL,          -- canonical JSON 的 sha256
  reply_json   TEXT NOT NULL,          -- 原响应，重放直接返回
  at           TEXT NOT NULL
);

CREATE TABLE audit (
  seq          INTEGER PRIMARY KEY AUTOINCREMENT,
  work_id      TEXT NOT NULL,
  revision     INTEGER NOT NULL,       -- 提交后的 revision
  request_id   TEXT NOT NULL,
  principal    TEXT NOT NULL,
  command_json TEXT NOT NULL,          -- 去掉大字段后的 Command
  at           TEXT NOT NULL
);
```

一切以 `state_json` 为准。`status` 列由写入时从状态派生，读时不信它做业务判断，只用于 `work list` 排序过滤。

## 2. 一次写事务

```text
BEGIN IMMEDIATE
  SELECT reply_json, payload_hash FROM requests WHERE request_id = ?
    命中且 hash 相同 → ROLLBACK，返回原 reply（replayed = true）
    命中且 hash 不同 → ROLLBACK，REQUEST_CONFLICT
  SELECT revision, state_json FROM works WHERE work_id = ?        （start 时跳过；start 的序号已在 §7.1 的前置事务里分配）
    revision ≠ 调用前读到的 → ROLLBACK，REVISION_CONFLICT
  ── 在此之前 core::decide 已经在事务外算好 Decision ──
  UPDATE works SET revision = revision + 1, status, state_json, updated_at   （或 INSERT）
  INSERT INTO audit
  INSERT INTO requests
COMMIT
```

规则：

- 事务内不读文件、不算摘要、不调模型。文件观察全部在 `BEGIN` 之前完成，随 `Command` 一起传进 core。
- `decide` 在事务外调用。若事务因 `REVISION_CONFLICT` 回滚，整个调用重新开始（重读、重观察、重决定），最多 3 次。
- 写目录、写任务书、置只读、刷状态卡都在 `COMMIT` 之后执行。这些是效果，不是状态。

## 3. 崩溃语义

进程可能在任何时刻被杀。每个时刻的保证：

| 时刻 | 库 | 目录 | 重启后 |
| --- | --- | --- | --- |
| `COMMIT` 前 | 无变化 | 可能有半写的观察副产物（无） | 状态不变；协调者重放同 `request_id` 即可 |
| `COMMIT` 后、效果前 | 新状态 | 缺 `brief.md` 或状态卡 | `work status` 正确；`attempt begin` 重放返回原 reply 并补写 `brief.md`；状态卡在下次任何写操作后重新生成 |
| 效果中 | 新状态 | 部分文件 | 同上；效果幂等，重放补齐 |

因此所有效果必须幂等：写任务书用「写临时文件再 rename」，置只读可重复执行，状态卡整份重写。

## 4. 产物封存

输出文件由工作 agent 写在 `attempts/<node>/<n>/<retry>/` 下。`attempt submit` 时 runtime：

1. 对每个声明输出 `lstat`：必须是普通文件（不接受符号链接、目录）；路径经 `confine()` 确认仍在 Attempt 目录内。
2. 读文件算 sha256 与字节数，组成 `ObservedFile`。
3. core 校验通过并提交后，`chmod a-w`。

**以 `ArtifactRef.sha256` 为准，不是只读位。** 下游 `attempt begin` 绑定输入时重算摘要核对；不符报 `ARTIFACT_MODIFIED`。这就是「输入按字节冻结」的实现。

起始输入在 `work start` 时写成 `inputs/<key>` 文件并同样记 `ArtifactRef`。

## 5. Workbook 仓库

`workbook add` 的复制顺序：

1. 校验通过后，先复制到 `workbooks/.staging/<uuid>/`。
2. 全部文件写完并 `fsync`，计算目录摘要。
3. 事务内 `INSERT INTO workbooks`；冲突报 `WORKBOOK_EXISTS` 并删 staging。
4. `COMMIT` 后 `rename` 到 `workbooks/<id>/<version>/`，整棵置只读。

重启后若发现 `.staging/` 里有残留目录，下次 `workbook add` 顺手清掉；库里有记录但目录不在时 `workbook verify` 报 `missing`，其他读操作报 `STORE_CORRUPT`。

复制时拒绝符号链接、硬链接、非普通文件、含 `..` 的路径、单文件超 32 MiB、总量超 256 MiB。

`workbook remove` 的顺序：事务内先查 `works` 表有无 `status IN ('active','blocked')` 且 `state_json` 的 `workbook` 等于该版本的行，有则 `ROLLBACK` 报 `WORKBOOK_IN_USE`；无则删 `workbooks` 行并 `COMMIT`；提交后把目录重命名到 `tmp/` 再删除，失败只记日志。

### 5.1 Work 的冻结副本

`work start` 在分配 `work_id` 之后、写起始输入之前，把 `workbooks/<id>/<version>/` 整棵复制到 `works/<work_id>/workbook/` 并置只读。`WorkState.workbook.digest` 记的是这份副本的摘要，与仓库里的相同。之后所有对该 Work 的操作（编译图、读说明书、绑 `resource.<path>` 输入）只读副本。这样有三个结果：

- `workbook remove` 与运行中的 Work 无关，只需拦非终态引用作为安全网。
- 有人改了 `workbooks/` 下的文件，已开始的 Work 不受影响；`workbook verify` 能发现。
- 终态 Work 在 Workbook 被删后仍能完整 `work status`。

副本目录缺失或摘要不符时，对该 Work 的一切操作报 `STORE_CORRUPT`，不回退到仓库副本。

## 6. 路径约束

runtime 只写 `SHELTIE_HOME` 之下。所有由外部输入拼出的路径（Workbook 内相对路径、输出 `path`、`--input k=@file` 之外的一切）都先经 `confine(root, rel) -> Result<AbsPath>`：拒绝绝对路径、`..`、空段；对已存在的路径做 `canonicalize` 后核对前缀。`--input k=@file` 是用户显式指向自己文件的读操作，不受此限制。

## 7. 时间与 ID

时间统一为秒精度的 RFC 3339 UTC 字符串，固定形如 `2026-09-24T03:00:00Z`（`YYYY-MM-DDTHH:MM:SSZ`，大写 `T` 与 `Z`，无小数秒、无时区偏移），由 runtime 取 `SystemTime::now()` 后放进 `Context` 传给 core。默认 `request_id` 是 UUID v7，在 runtime 生成。core 不碰时钟与随机数，测试里传固定值。core 的 `Timestamp` 构造与读取都校验这个格式，不合法就报错：格式固定，所以字符串字典序就是时间序。由 Unix 秒数换算（`from_unix_secs`）时超出 9999 年的输入饱和到 `9999-12-31T23:59:59Z`，引擎产生的时间永远是这个格式。

### 7.1 `work_id` 的序号分配

`work_id = <day>-<seq>-<name>`，`seq` 为三位十进制补零。分配在一个独立的短事务里：

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
- 分配后不回收。`start` 的后续步骤失败或崩溃，这个号就空着；`works` 表里没有对应行即可。
- `day` 取 `Context.now` 的 UTC 日期，与 `created_at` 同源，避免跨午夜时日期与序号不一致。
- 名字部分只是可读后缀，身份由整个字符串承担；同名不同序号是不同 Work。
- 同 `request_id` 重放在分配序号之前先查 `requests` 表，命中则直接返回原响应，不再分配新号。

## 8. 清理

MVP 不提供 `clear`。删除一个 Work 的方法是手工删目录再删行，文档明说。带归属核对的 `work clear` 见 [路线图](../roadmap.md)。

`tmp/` 里的一切都可以在任何时刻删。每次写操作开始时清掉 `tmp/` 下修改时间超过 24 小时的条目。

## 9. 二进制自身

目录：

```text
~/.sheltie/
  bin/
    sheltie          当前版本
    sheltie.prev     上一版本，供回滚
  tmp/               下载与 staging
```

`self install`：把当前可执行文件复制到 `tmp/`，`fsync`，`rename` 到 `bin/sheltie`。已存在且字节相同则不动。

`self update` 的顺序：

1. 查发布清单，找到当前平台（Rust target triple，如 `aarch64-apple-darwin`）的包与 sha256。没有报 `UPDATE_UNAVAILABLE`。
2. 下载到 `tmp/<uuid>/`，算发布包文件的摘要，不符报 `UPDATE_CHECKSUM_MISMATCH` 并删 `tmp/<uuid>/`；通过后若是压缩包（`.tar.gz` / `.tar.xz`）则解包，取包内唯一名为 `sheltie` 的普通文件（cargo-dist 的布局是 `<产物名去掉扩展>/sheltie`），瘦格式的资产就是二进制本身，不解包。
3. `rename bin/sheltie → bin/sheltie.prev`（覆盖旧的 `.prev`）。
4. `rename tmp/<uuid>/sheltie → bin/sheltie`。
5. 删 `tmp/<uuid>/`。

第 3 步与第 4 步之间崩溃，`bin/sheltie` 不存在但 `.prev` 在；用户用 `~/.sheltie/bin/sheltie.prev self rollback` 恢复，`self rollback` 对此情形要能处理（`bin/sheltie` 缺失时直接把 `.prev` 挪回）。Unix 允许替换正在运行的可执行文件，当前进程继续用旧映像跑完。

`self update` 不改 `store.db`。新版本若带更高的 `SCHEMA_VERSION`，下次任何操作按 §1.1 报 `STORE_SCHEMA_MISMATCH`，提示 `self rollback`。MVP 只有 schema 1。

发布链用 `cargo-dist`：从 git tag 生成 GitHub Release、各平台压缩包、sha256 清单、`install.sh`。`self update` 读发布清单 `dist-manifest.json`，认两种写法：瘦格式 `{ version, assets: [{ platform, name, sha256 }] }`（本地发布目录与测试用，`SHELTIE_RELEASE_BASE` 指向本地目录时不联网），以及 cargo-dist 发布的完整清单（在 `selfmgmt` 里适配成同一形状）。网络下载用系统 `curl`。不引 `axoupdater`：其公开 API 只能执行安装脚本，不暴露清单与 sha256，与本节五步冲突（[v0.1.0 decision log](../releases/v0.1.0/decisions.md) D-30）。二进制只包含 `sheltie` 一个可执行文件。
