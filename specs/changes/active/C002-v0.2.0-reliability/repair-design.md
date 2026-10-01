# M1 问题修复方案

依据：[2026-09-28 独立审查](review-m1-2026-09-28.md)，被审候选 `e1a8126a432981df40023628dd06feec7884d458`。本文件给出 R01–R19 的解决方案，并记录 T31 新发现的 R20；任务步骤见 [repair-plan.md](repair-plan.md)，验收样例见 [repair-validation.md](repair-validation.md)。实际进度只看 [plan.md](plan.md)。

这里描述待实施目标，不表示已支持。既有产品与机制仍以上游规格、架构和合同为准。T18 先固定本文点明的合同调整与 API 选型，再开始代码任务；初级工程师不得自行在矛盾之间选择。其他方案均落实已采用的 C002 行为，不另建兼容格式、业务状态或宿主适配框架。

## 1. 先理解责任链

一次写命令应按以下顺序运行：

```text
CLI 只解析参数
 → runtime 只读识别 Store、查历史请求与目标
 → 新请求才读 @file / Workbook 并检查确定性条件
 → 获得本管理根写锁
 → 锁内重核 schema、请求、目标
 → 同一恢复流程完成旧效果
 → 观察文件 → core::decide → 一个 SQLite 事务
 → 完成本请求效果 → 标 published → 清理本请求元数据
 → CLI 渲染快照或准确的恢复错误
```

core 继续只处理类型、图和状态转换；runtime 承担文件对象、身份、持久化与恢复；CLI 不读文件、不打开数据库、不拼历史业务事实。状态卡是当前投影，brief/stats 是历史字节，二者不能使用同一重建口径。

## 2. 文件操作：限定路径，再打开真实对象

### 2.1 采用的接口形状

在 runtime 的 `fsx` 下集中放置受管文件操作。拆文件仅在一个文件难以导航时进行，可采用 `fsx/path.rs`、`fsx/managed.rs`、`fsx/tree.rs`；不建立通用文件系统 trait。以下名称、所有权边界、可观察结果和参数约束是方案要求；示例签名只说明职责，不是T18交付接口。

| 名称 | 保存什么 | 谁可以构造 |
| --- | --- | --- |
| `ManagedRelPath` | 非空、无绝对前缀、`.`、`..`、空段、NUL 的根内相对路径 | 校验构造或 Raw DTO 校验，字段私有 |
| `ManagedFs` | 已确认身份的管理根目录句柄及用于响应的规范路径 | Home 打开；写操作还须持 HomeLock |
| `ManagedDir` | 从可信目录逐段打开的子目录句柄 | ManagedFs 内部；不接受任意 AbsPath |
| `SafeFile` | 已打开普通文件的句柄、dev/inode、实际字节观察 | 受管目录句柄或明确的外部只读入口 |

`ManagedFs` 对真实 caller 提供 `open_regular / ensure_dir / write_new / write_atomic / rename_new / remove_owned_tree`。目录遍历、chmod、sync 和临时文件名字隐藏在这个模块里；caller 不再自行拼路径后调用 `std::fs`。用于输出/任务书的绝对路径只能由 Home、WorkId、WorkLayout 派生，不能作为文件操作的授权依据。

供T19实现者参考的签名草图如下。写方法借用HomeLock并检查锁根与ManagedFs根是同一对象；只读方法不取锁。外部只读源使用不同的只读类型，不能调用seal。内部字段私有，若某方法没有真实caller就不公开；不是为mock增trait。T19按真实caller确定私有签名，并同时迁移调用方和反例。

```rust
impl ManagedFs {
    fn open_existing(home: &Home) -> Result<Self>;
    fn open_regular(&self, path: &ManagedRelPath) -> Result<SafeFile>;
    fn ensure_dir(&self, lock: &HomeLock, path: &ManagedRelPath) -> Result<ManagedDir>;
    fn write_new(&self, lock: &HomeLock, path: &ManagedRelPath, bytes: &[u8]) -> Result<()>;
    fn write_atomic(&self, lock: &HomeLock, path: &ManagedRelPath, bytes: &[u8]) -> Result<()>;
    fn rename_new(&self, lock: &HomeLock, from: &ManagedRelPath, to: &ManagedRelPath) -> Result<()>;
    fn remove_owned_tree(&self, lock: &HomeLock, path: &ManagedRelPath) -> Result<()>;
}
impl SafeFile {
    fn read_bounded(&self, max: u64) -> Result<Vec<u8>>;
    fn sha256_bounded(&self, max: u64) -> Result<(Sha256Hex, u64)>;
    fn seal(&self, lock: &HomeLock, expected: &ArtifactRef) -> Result<()>;
}
```

这里只锁定职责、参数和错误面；T18用macOS真实API探针确认所需操作可行，T19确定最终私有签名并迁移全部受影响caller；不能将草图作为todo骨架先提交。HomeLock创建新根/.lock是唯一锁前例外，不能调用上述需要已有锁的ensure_dir形成递归。

外部 Workbook 源目录与显式 `@file` 单独走只读入口。`@file` 的意图仍只保存词法绝对路径，不因 canonicalize 或读取而破坏重放；首次读取时检查叶对象类型与 nlink。这个入口没有写入、删除或 chmod 方法。操作系统合法的 `/tmp`、`/var` 别名在 Home 入口按既有规则确定一次规范根，不能把受管子目录的软链当相同例外。

### 2.2 具体系统 API

选用当前依赖闭包已有版本 `rustix 1.1.4` 的安全 `fs` API，作为 runtime 的直接依赖；项目源码仍 `unsafe_code = forbid`。不要让初级工程师写 libc/FFI 或手工持裸 fd。

- 用目录 `OwnedFd`、`openat` 和 `OFlags::DIRECTORY | NOFOLLOW | CLOEXEC` 逐段打开；传给 openat 的每次只有已经校验过的单个段，不能把带多级路径的字符串直接交给一次 openat。
- 先以同父目录句柄fstatat且不跟随链接，拒已知特殊文件；打开叶文件用NOFOLLOW、NONBLOCK、CLOEXEC，随后fstat再核普通文件、nlink=1及前后dev/inode。NONBLOCK避免stat/open之间被换成FIFO时在类型检查前永久阻塞；不能只信DirEntry或打开前stat。
- 用 `Dir::read_from` 枚举目录句柄，再从同一目录句柄打开条目；名字非 UTF-8、宿主元数据、链接和特殊对象准确拒绝。
- 独占创建临时文件；发布到不存在的新目标使用 `renameat_with(..., RenameFlags::NOREPLACE)`，两个端点都是已打开父目录及合法叶名。目标已存在时按caller合同核对：已登记Workbook/Work不可覆盖；`self install` 对同字节二进制幂等，对合法普通单链接分叉二进制原子替换，保留既有行为。各caller都拒绝链接、特殊对象和多链接，不沿路径跟随。
- 状态卡是允许替换的投影，使用目录句柄内的独占临时文件、sync、普通 renameat；历史文件只允许补缺或接受相同字节。
- 文件权限用 `File::set_permissions` 或安全 fchmod 作用于同一个句柄；目录权限与 fsync 也作用于目录句柄。

已核对 rustix 1.1.4 源码：声明 MSRV 1.63，`renameat_with` 对 apple/linux 提供，NOREPLACE 的平台映射按源码核查；macOS arm64探针已实测API。用户豁免Linux运行，因此Linux文件API结果保留`not_run`，不宣称实测通过。其公开接口见 [openat](https://docs.rs/rustix/1.1.4/rustix/fs/fn.openat.html)、[renameat_with](https://docs.rs/rustix/1.1.4/rustix/fs/fn.renameat_with.html)、[Dir](https://docs.rs/rustix/1.1.4/rustix/fs/struct.Dir.html)。T19在macOS跑真实caller。

同账户可以手工改文件，不提供独立身份隔离。句柄解决的是路径被替换后错误操作另一个对象的问题；不声称能禁止同账户把整个已打开目录搬走。Store不引入自写SQLite VFS：它位于持锁、已核根身份的目录，连接用rusqlite SQLITE_OPEN_NOFOLLOW，每次连接前后核根/锁身份，数据库及wal/shm已存在对象均拒链接与多链接。bundled SQLite的unix路径解析也能识别路径段软链，但该flag不是目录fd锚定，不提供相同强度的硬链/并发替换保证；Home先确定规范根，常规受管祖先仍要校验。见 [rusqlite OpenFlags](https://docs.rs/rusqlite/0.37.0/rusqlite/struct.OpenFlags.html)。

### 2.3 摘要、复制与封存

复用一个受限目录枚举器。第一遍收集名字、对象身份和长度，不收集正文；按相对路径字节序排序并计算目录总量。第二遍逐个安全打开，核它仍是同一对象，在该句柄上 stat、有限读取、喂入摘要。目录摘要编码保持已采用的 `workbook-digest/v2`，不能改为“各文件 hash 的 hash”。

复制时用已打开源句柄读取、独占目标句柄写入并 sync。最终副本的 manifest/Flow/编译/摘要来自副本，不能沿用源目录预检的身份。资源正文流式读取，仅缓存编译实际需要的 UTF-8 文本；不长期缓存整个 Workbook，也不同时持有整棵树全部文件句柄。每次实际读取同时计单文件和总量，增长不能绕过上限。

正常submit将观察句柄保留到COMMIT后：同一句柄先算bytes/sha256送core；封存前再核该对象类型/nlink和当前bytes/sha256仍等于已提交ArtifactRef，随后同句柄chmod/sync，再核原路径仍绑定该对象。这个再核是覆盖观察→提交窗口，不能以“减少扫描”删除。路径被换时不chmod新路径对象，已观察对象仍是封存对象，但请求保持效果未完成并准确报告。恢复不能保存fd，重启后从登记引用安全重开并验证再seal；已完成submit重放不seal。COMMIT前拒绝不提交，COMMIT后错误不回滚成功状态。

## 3. 持久数据必须先通过归属校验

### 3.1 Work 路径

`Store::decode_row` 继续做纯结构、身份、revision/status 校验。runtime load 再核：`state.work_dir == Home::work_dir(state.work_id)`；起始输入路径等于 WorkLayout 为对应 key 派生的路径；各 Attempt 输出路径属于该 Work、该 Attempt 的 `outputs/` 声明路径。运行中的输入可以引用同 Work 的起始输入、冻结资源、上游输出或本 Attempt 的 engine.stats；不得引用另一个 Work 或任意绝对路径。

这项归属检查必须覆盖 status/stats/list 后续的文件使用、begin/submit/cancel、状态卡、封存和恢复。失败返回带 row 与字段定位的 STORE_CORRUPT，不从 basename 猜正确路径、不自动修写坏数据。

Workbook行也要校验既有id/version/digest/added_at的合法构造，row.dir必须等于Home按id/version派生的目录；不能把损坏行里的dir直接作为文件授权。已装副本manifest身份、内容摘要和当前登记行必须一致。纯校验复用core已有规则，不在runtime复制一套版本/ID正则。

### 3.2 请求与效果

现有持久 JSON 的字段和 schema 2 不变。增加 Raw DTO→已校验内存对象的转换，不能把另一套“已校验状态”存进数据库。`decode_effects` 只解析还不够；执行前必须加载本请求、对应 audit、快照与已校验 Work 行，校验整组效果后才能执行第一个动作。

Store 的请求读取需同时返回 request_id、work_id、intent_hash、reply_json、effects_json、published、audit.seq/revision/command_json；audit 归属缺失或不唯一时报 STORE_CORRUPT。Work 效果归属由 request.work_id、快照中的 Attempt 和当前保留的 Attempt 历史核对；Workbook 操作种类从已登记 audit 的闭集字段读取，id/version 来自同请求快照。不得通过自然语言判断操作种类。

| 效果 | 必须核对的闭包 |
| --- | --- |
| publish_dir | 未完成时核pending `<id>/payload`、合法owner侧车、本请求Store引用、owner业务身份、唯一final；Work再核冻结摘要及每个start-input ArtifactRef。完成请求的纯快照重放不执行此动作，也不要求已清侧车 |
| prepare_attempt | 已提交 Work 中真实存在的 Attempt；dirs 恰为 WorkLayout 的目录及声明输出父目录，不能额外创建目录 |
| write_file | 只能是该 begin 的 brief 或 engine.stats；路径属于同一 Attempt，content 的 sha256 与登记值相同 |
| seal_outputs | 只能是该 submit 保留的输出引用，不能 chmod 其他路径或改写引用 |
| delete_dir | 本请求是 remove、侧车 op 一致、目标 Workbook 身份/登记摘要与私有 pending 正确 |
| refresh_status_card | work_id 与本请求绑定 Work 一致，目标从 Home/WorkId 派生，正文从最新 Store 生成 |

摘要字段使用 Sha256Hex，路径字段使用 ManagedRelPath，owner/op 用闭集枚举，不能用空串、`serde(default)` 或 `unwrap_or(root)` 猜缺失事实。`digest_root` 在当前实现中已写入每次 publish：Work 固定 `workbook`，Workbook 固定空根；T18 把这一现有字段补入合同，不引入第三种解释。无归属的目录、旧错误 `payload.deleted` 不自动归入某个请求。

## 4. 请求入口与 Store 初始化

WorkService/WorkbookRepo公开构造只接受Home，构造不做I/O。迁移全部CLI/runtime tests/common fixtures，不保留接受任意RW Store的兼容入口。只读命令按操作打开RO Store；写命令预检后创建WriteSession，拥有HomeLock和锁内RW Store直到效果结束/错误返回。初始化权限只有两个固定入口：install/add允许创建空Store；start/begin/submit/fail/approve/cancel/remove必须打开已存在Store，purge等待结束后库已缺就NOT_FOUND，不创建新库或复活旧Work。Session不接受caller自由传一个create=true绕规则。

Store的RW创建/连接入口收进runtime内部。只读识别不用可能CREATE数据库的默认Connection::open，也不因WAL/权限问题降级为读写连接。新根add/install确定性校验成功才允许创建根/.lock，库和业务目录只能在锁后创建；self update/rollback不打开Store。

SQLite只读连接可能建立/维护WAL的store.db-shm，READ_ONLY不等于文件系统零写。用户已授权T18采用的合同是：只读/预检/拒旧不得写主数据库、WAL记录、schema、业务文件或引擎.lock，不得创建新管理根；允许SQLite在已存在根内维护该数据库的常规共享内存控制文件，单独记录其变化。schema1的main/WAL及用户记录仍逐字节保留，不迁移、不checkpoint、不清空；不声称整个旧根逐字节不变。不能使用immutable=1或自写VFS。

T18必须在真实bundled rusqlite上验证主库+WAL的ro读取、缺shm、活动写者、main/WAL/shm软硬链接、schema1拒绝前后main/WAL字节及关闭连接后的变化。只读识别连接在首次查询前用安全set_db_config(SQLITE_DBCONFIG_NO_CKPT_ON_CLOSE,true)禁止关闭时checkpoint。不能只按main文件长度拒绝：macOS实测当短main对应有效且含完整已提交快照的WAL时，SQLite可从WAL读出数据库；若WAL header损坏但main有效，bundled SQLite忽略WAL并返回main中的旧快照。运行时遵循SQLite可查询视图，不自写WAL解析来猜测忽略原因；任何open/query/close路径仍须保持main/WAL字节不变。链接/特殊对象调用SQLite前拒绝；缺shm维护与缺失WAL的零字节创建只属于上述控制文件。immutable=1忽略锁与变化检测，不是默认替代；参考 [SQLite连接语义](https://www.sqlite.org/c3ref/open.html) 与 [WAL只读数据库](https://www.sqlite.org/wal.html#read_only_databases)。控制文件例外不放宽引擎创建库/目录/锁的顺序，API无法守住main/WAL原字节时T18停止。

历史 Work 前缀先查 request_id 的 work_id，要求该完整 id 以用户前缀开头，再构造意图；不匹配则 REQUEST_CONFLICT。无请求记录才解析当前唯一前缀。所有 Work 写动词都走此入口，不能只修 attempt begin。

CLI对@file只解析路径，新请求才读取，submit/fail移到查重后构造Command。保留已固定RequestIntent的JSON字段/顺序和独立向量，不在时序修复时悄悄改已有intent_hash。名字选择器保留用户参数及省略状态，WorkName规范化是新请求的纯校验/创建结果，写入WorkState与成功快照；T18勘误上游“规范化名字进意图”的笼统表述为这个明确分工。不同名字参数可产生同样WorkName，但同request-id仍按参数冲突；不增加旧/新编码fallback。原始文件bytes不进意图。

T18明确新请求@file参数失败的形状：不可读/非UTF-8/非普通文件/符号链接/硬链接（`nlink > 1`）/超过文件源读取上限，使用专用runtime参数源错误，CLI返回INVALID_REQUEST与exit2，detail带path/原因。文件源上限及数值以storage §5.3为唯一依据（32 MiB）。其他受管IO仍exit1，不能一概映射为参数错误。已读内容超过summary/reason的4096字节是core的SUMMARY_TOO_LONG/exit1；这两个上限不可混淆。错误分类与读取只在新请求发生。该专用错误属于解析后的外部参数来源，不表示引擎推断业务内容。

## 5. 一个恢复模块完成全部效果

新增 `runtime/recovery.rs` 统一恢复循环。它只依赖自己定义的 `RecoveryAccess` 能力接口、Home、Store 与效果执行器，不持有 WorkService/WorkbookRepo 的具体类型；两个服务各自提供持久行与效果校验，Work 服务也提供最新 Work 状态读取。状态卡刷新循环放在 `runtime/load.rs`，通过读取回调从最新 Store 状态编译并渲染卡片，不在 recovery 或 repo 中重写恢复流程。

恢复模块只保留两个对 caller 的动作：`recover_before(current_request)` 与 `finish_request(request_id)`。前者按 audit.seq 处理未完成请求，后者处理本请求的效果或已完成 begin 的历史文件核对。它们共同使用同一个校验、效果执行、当前状态卡刷新、mark 与清理实现。删除 `WorkService::recover/finish_request/replay` 与 `WorkbookRepo::recover_workbook_effects` 中的重复 I/O；快照解码仍可按 Work/Workbook 的回复类型区分。

先区分当前意图命中的历史请求，才决定恢复错误归属：

| 场景 | committed | 顶层字段 | detail |
| --- | --- | --- | --- |
| 本请求 COMMIT 后失败，或显式重放自己未完成的请求 | true | request_id、本请求完整 original；Work 另有 revision | cause 的协议错误码、对象/路径、恢复提示 |
| 旧 A 阻断尚未提交的新 B | false | request_id=B；不带 original/revision | pending_request_id=A、A 的完整 pending_original、cause |
| 非提交阶段拒绝 | 不附提交字段 | 原错误封装 | 精确定位 |

完整成功响应由 runtime 快照与统一协议转换生成，CLI 不回读当前状态补字段。错误内部保留结构化 cause 与快照，而非把错误先变 String 再猜码。CLI `output.rs/error_map.rs` 只把这些字段放到 protocol §5 规定的位置。自己的 `original` 与旧 A 的 `pending_original` 都是完整成功封装，不是内部 Reply 或缺字段的 snapshot。

任何效果、状态卡刷新、mark 的错误都在该模块按真实提交状态包装。历史文件字节不同/父目录缺失同样是已提交恢复错误。全部效果成功且必要 sync 完成才置 published=1；cleanup 失败是已完成后的维护错误，不得把业务标回未完成或重新执行已经完成的删除/封存。T18 固定维护告警规则：runtime 将告警作为调用结果的独立诊断交给 CLI，CLI 写 stderr，带 request-id、对象与原因；业务成功仍 exit0，stdout成功JSON与历史快照字段不变。不能把告警塞进业务data造成历史响应漂移；这一报告规则先在protocol明确。

## 6. 发布、删除与清理

### 6.1 发布状态表

| pending payload | final | 行为 |
| --- | --- | --- |
| 有 | 无 | 校验完整闭包、sync 原件与必要目录、NOREPLACE rename、sync 源/目标父目录、完成只读权限与 sync |
| 无 | 有 | 核 final 是登记对象，包括 Work 的起始输入；完成未完权限/sync；同对象才视为已发布 |
| 有 | 有 | 停止，不覆盖、不删除任一对象 |
| 无 | 无 | 已提交错误，说明唯一原件缺失，不从 Workbook 仓库或用户源目录重造 |

所有必需 sync 返回 Result。目录 sync 不支持或失败都停止标完成；平台差异只能用平台实测后在 T18 固定的实现处理，不能吞错误写 PASS。文件 API/故障模拟证明程序走了正确顺序，不等于证明真实断电持久性；报告分开记录。

### 6.2 删除状态表

remove 准备阶段创建并 sync owner 和私有 container，**不预建空 payload**；payload 只在原目录被移入时出现。T18 修正 storage §3.3/§5.2 中把 remove 与 add/start 一律预建 payload 的表述。internal_id 从合法 PendingOwner/规范路径一致性取得，不从 `payload` 的 basename 取得。

| final | payload | 合法 `<id>.deleted` | 行为 |
| --- | --- | --- | --- |
| 有 | 无 | 无 | 核原对象身份/摘要、移入私有 payload、sync 两个父目录、删除已核同一对象 |
| 无 | 有 | 无 | 再核 payload 的归属/摘要，再继续删除；不能只看目录在 |
| 无 | 无 | 有 | 核标记 format/internal_id/文件类型/nlink及本请求绑定后，视为删除已完成 |
| 无 | 无 | 无/损坏 | 结果不明，停止；不能新建标记把“无法证明”改成成功 |
| 有 | 有 | 任意 | 冲突，停止，保留两者 |
| 有且摘要不同 | 无 | 无 | 不同对象，停止；不能写完成标记冒充删除完成 |

真实执行完成删除后 sync payload 父目录，再独占写 `<id>.deleted`、sync 文件和 pending 父目录；随后才 mark。“删除后、标记前”进程被杀必须保留结果不明，这个 failpoint 的正确结果是停止，不是自动恢复 PASS。已完成 remove 重放不再进入 delete，后续相同 id/version 的新生命周期保持原样。

### 6.3 pending 清理与只读发现

新增 `runtime/pending.rs`，仅集中 owner/marker、Store引用索引、对象发现与清理，不新增业务状态。扫描全部请求的效果而非只查 unpublished：完成请求也可能有待清理元数据。先把全部路径/归属/引用校验完再删除，不能边读遇损坏记录边猜未引用。

同request-id的COMMIT前崩溃重试，必须在新请求登记前清掉已核合法、未被任何Store引用的本rid原件。若必需清理失败，尚未提交的新请求停止，保留可用证据；不能先登记新row再把旧原件当异常。同rid以外的普通维护仍由CLI独立stderr诊断，不改变已成功业务结果。

- 合法 owner + 无 Store 引用：删除本操作的未提交私有树与 owner；不能动旁边另一操作的树。
- published=0 且被 Store 引用：保留原件，只交 recovery；年龄不参与判断。
- published=1：只清本请求空 container/owner/deleted；非空 payload 或不同对象保留并报告，不能重做业务效果。
- 无目录 owner 残片：保留并警告；不阻断其他已归属的合法请求。无 owner 目录、路径异常或 Store 引用不明必须停止，不能当普通垃圾。

Work/Workbook只读发现使用同一locate_committed_object：从当前业务行及其所属publish请求找final或pending。未完成发布/读取pending必须核合法侧车；published=1且元数据已清的当前final由该业务行、对应成功请求/effect和内容身份验证，不要求已清owner。历史已完成add/start/remove的纯快照重放不定位当前final，避免误碰重加同版本的新生命周期；已完成begin只核自己的历史write_file。整批Checked效果校验同样区分持久结构归属与本阶段才需要的文件证明。

只读与rename交错时最多一次final→pending→final重试，仍失败返回暂时IO；不得恢复、造业务目录、取引擎.lock或回退任意版本。副本打开后复核身份。pending_publish来自当前对象所属publish请求是否未完成，不只看目录在哪；rename已成功但mark前也为true。已完成begin的其他未完效果不冒充start发布。SQLite控制文件例外仅按§4已采用合同处理，不视为业务恢复。

Work 的图编译与资源/说明书读取使用本次实际定位到的同一冻结目录句柄；ArtifactRef 保留已登记 final 绝对路径，只读阶段不得改写成 pending 路径。只有写操作完成发布后才能 begin/观察输入。

## 7. self：管理路径与 purge 的锁生命周期

install/update/rollback/uninstall 全部使用 T19 文件模块，临时名字只在受管 tmp 中创建，bin 与 tmp 祖先都核对。正常 update 按固定 tag 验包，校验失败旧 binary/prev 不变；rollback 保留已采用的“目标缺失时用 prev 恢复”。权限、rename、删除与 sync 都不能绕过该模块。

已获用户授权的 purge 合同：**删除全部用户数据和 binary，保留空的管理根与原 `.lock`**。这样purge与下一次写入共享同一个锁对象，没有“先删锁、后删根”产生的新锁空窗；同时仍只写管理根。CLI文本明确保留这两个控制对象，成功时store/workbooks/works/pending/tmp/bin全部不存在。下一次install/add在持同一锁后创建新的空Store；没有第二套Work状态。

该合同已由用户明确采用，并同步到spec/storage/protocol/README与D-035/D-038。T23只实现这一种purge模式；初级工程师不维护第二种行为，也不自行添加根外锁文件或兄弟目录暂存。

删除前先只读遍历并验证对象类型与所需权限，打开将使用的目录句柄；只对已核归属对象放开必要目录写权限。按数据树→pending/tmp/bin→Store侧文件与数据库顺序删除；`.lock`从不删除。失败时报部分清理与定位，Store在其他树清理完成前保留，不声称全部文件未变；再次purge可继续。purge结束后合法install/add可沿同锁初始化空Store，等待的旧Work写命令必须NOT_FOUND，不能因Session默认CREATE建库或恢复已删除Work。

## 8. 一次事实装入与 fresh worker 交接

`load.rs` 的只读 Work 装入返回 state/revision/graph/定位事实。runtime 同一次装入构建 stats 与 next，CLI 不再另调 status。测试用明确同步事件在“装入完成/渲染之前”让写者改变 Store；响应中的统计与 next 仍来自原一份 state，不要求它等于随后发起的最新查询。

spec-dev不修改core“来源节点不能是自己”的规则。plan-review保留decision.md/16384字节，新增必需reviewed-plan→reviewed-plan.md、reviewed-tasks→reviewed-tasks.md，两个上限均65536字节；审核人按任务书把本轮plan/tasks输入原字节复制到这些输出再提交decision。批准/打回都保存被审副本。plan增加optional previous_plan←plan-review.reviewed-plan、previous_tasks←plan-review.reviewed-tasks；不同节点显式back满足现行规则。首次null，之后绑定上次被审副本，不依赖聊天或缓存。

冻结旧tasks只含定义，不含后来完成事实。implement.change/fix.change记录“继承验证报告”的实际绑定路径并原样携带其累计表；verify在report写本轮检查的change/fix路径和继承来源，先与源表逐行比对、保留所有已有行，独立过Git/范围/门禁后才追加本任务，失败不追加。每行含Task、任务基线/候选commit、原审批spec/plan摘要、冻结verify报告和原始门禁路径。report写自己输出路径即可，不算自引用sha；表累计到最多20任务仍受32768字节报告上限，不复制全文历史。这是协调者/worker核查的业务文档，引擎不解释结论、不写为WorkState字段。

plan还绑定optional previous_verification←verify.report、previous_change←implement.change、previous_fix_change←fix.change。fresh worker从旧计划拿原始整体基线，从累计verify表找历史任务，沿报告声明的检查/继承路径比对累计前缀；有遗漏、改写或不相干来源就停止。若latest change/fix是尚未验证的下一任务，它携带前一次完整验证表即可，不能要求它等于另一个任务的已追加结果。逐行核项目commit/祖先关系、原始证据、批准版本与Task；change/fix的“完成”字样不自动变已验证。条件改动须列重验任务，旧PASS不套新spec；缺材料停止，只有首次无旧计划才取HEAD。

## 9. 验证与交付

每个行为任务都包含合法例、单条件反例、独立 oracle、真实 CLI/文件/SQLite 状态和失败停止路线。修复通过只覆盖本任务能力；最终 R01–R20 与原 O/N 矩阵由独立 M1 Reviewer 合并。并发用同步点制造确定性交错；同一候选的 crash binary 只构建一次，从 Cargo JSON 获取路径，测试期间不再重链接共享 binary。

最后运行core/runtime突变，先列候选再分批执行，记录每个存活体对应的当前能力和处置；不按存活数强行加镜像测试。完整Work/Workbook kill窗口、macOS文件API、本地发布形状和MSRV仍是M1必需证据。Linux运行按用户指示豁免，保留`not_run`与平台风险，不作跨平台PASS。真实宿主、usage与四平台发布资产分别留T16/T17，不能替其填PASS。

## 12. T31 增量发现 R20：持久受阻次数触发溢出

2026-09-30 独立 CLI 探针从合法 Gate 状态只把 `blocked_count` 的 1 改成 `u32::MAX`。status/stats 接受该值，gate approve 在 core 增量时 panic，stdout 没有协议响应；2cfe8027 固定二进制复验一致。原始证据见 [计数探针](evidence/repairs/t31/blocked-count-independent-probe.json)。这不是新产品功能或任意篡改检测承诺，而是 GF-29 与可信装入的内部事实约束缺口。

修复使用已有状态中的必要界：每次受阻至多归属一次 Attempt 结束或一次 Gate Approval；所以累计不超过 `attempts.len() + approvals.len()`。每条 Approval 已经历一次 Gate 受阻；当前 Blocked 状态还证明一次新的受阻，因此累计至少为 `approvals.len() + 当前是否Blocked`。仅校验必要界，不重算精确次数，不改原事实、不读 audit、不添字段。runtime 的现有 decode/commit caller 将不一致映射为 STORE_CORRUPT；core 三处增量使用同一 checked helper，避免独立 API 输入触发 panic 或 release wrap。

实施归 C002-T31：先补真实 CLI 的合法 count=1→2 与单字段 MAX/0 反例，核 status/stats/list/approve、原 works/revision/requests/audit 和业务文件；再补纯 core submit/fail/approve 的溢出正反例，最后核合法 Gate 重访和取消仍保持累计。保留红输出，固定新源码/测试/配置候选，再跑默认/优化完整基线及新完整变异清单；旧 2cfe 部分结果不作为新候选通过。

## 13. T31 实现审查后的闭环

2026-09-30 用户授权修复[实现审查](review-implementation-2026-09-30.md)的七项问题并完成T31。Store首次初始化按storage §1.1在自有tmp中完成后再发布，既有schema拒绝规则保持；崩溃与并发正反例走真实add/install。持久状态结合冻结Graph核门槛Occurrence与批准记录，缺失事实报STORE_CORRUPT并保留原数据，不通过重放或推断补造批准。历史文件及PrepareAttempt目录存在只证明对象在位，未发布效果恢复仍须完成同对象的文件/目录链sync，连续失败不得mark。

协议映射复用core的单一NextOp编码；快照数据形状在解码边界严格校验，业务归属仍用可信状态核对；同一写锁内复用已核请求和装入结果，普通查询按目标定位引用，清理保留全Store引用检查。add源仅做结构、对象类型和限额预检，最终副本才做parse/compile/digest。以上不新增持久视图、兼容格式或业务状态。原T31增量与失败证据保留；新源码形成新mutation输入闭包，所有旧结果标历史或superseded，不能混为新候选PASS。
