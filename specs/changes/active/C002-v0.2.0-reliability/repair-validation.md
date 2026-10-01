# 修复验收样例与证据

供 [repair-plan.md](repair-plan.md) 的 T18–T31 使用。这里规定验收清单；各项当前结果只看 [plan.md](plan.md) 与 [validation.md](validation.md) 的固定候选记录，未执行的样例仍为 `not_run`。历史失败原文在 [M1 evidence](evidence/m1-2026-09-28/README.md)；原始探针证明旧候选的问题，不是新候选的通过证据。

## 1. 每条样例怎样写成测试

1. 用 tempfile 创建独立管理根和根外哨兵目录。通过真实 `workbook add/work start/attempt begin` 建立合法状态；只有模拟持久损坏或精确 crash 窗口时才直接修改临时 SQLite。
2. 记录合法例的 argv、exit、完整 JSON、Store行数/revision/published、文件字节与权限。
3. 从同样的合法条件另建夹具，只改变表中指定的一个条件。模拟窗口所需的共同准备同时应用到合法例与反例，不把它算成业务条件。
4. 用合同固定值、独立手写路径、Python hashlib/Git查询，或变更前捕获的原始字节写 expected。不要调用生产 helper 生成同一个答案。
5. 核对停止路线：拒绝发生在 COMMIT 前还是后；有没有新请求行；原件是否保留；下一次同请求怎样恢复。文件存在、`is_ok()`、测试数量都不是完整 oracle。

建议测试名已经给出，实施时须先用 `rg -n 'fn <name>' crates` 确认全仓唯一。名称描述行为；归属使用紧贴 `#[test]` 的 `// Task: C002-Tnn`，不加 tNN 前缀。对照表中的“拟新增”不是声称源码已经有该测试。已有 distinct oracle/真实入口/故障窗口的测试保留；重复断言可以合并，但不能删除失败反例来变绿。

## 2. 小型 oracle 工具

### Store

每次操作前后用测试连接或 Python sqlite3 读取同一临时库：

```sql
SELECT work_id, revision, status, state_json FROM works ORDER BY work_id;
SELECT request_id, work_id, published, reply_json, effects_json
  FROM requests ORDER BY request_id;
SELECT seq, request_id, revision, command_json FROM audit ORDER BY seq;
SELECT day, last FROM work_sequence ORDER BY day;
SELECT id, version, digest, dir FROM workbooks ORDER BY id, version;
```

确定性preflight拒绝：业务行、requests/audit/sequence与基线相同，最终目录无新增，不创建新根。序号已分配、Work COMMIT前的准备失败：works/requests/audit/final不新增，但sequence允许留下一个空号；必须不回收、不复用。COMMIT后失败：请求与新revision已持久，published=0，原件保留，检查准确EFFECT_PENDING。新B被阻断：B在requests/audit中不存在，A原响应不变。不要为满足错误oracle回收合法空号。

### 文件

保存sentinel bytes、Unix mode、dev/inode，反例后再比较。历史文件推进后删缺再重放，比较原bytes；状态卡捕获最新bytes防旧请求覆盖。只读访问比较业务路径/bytes/mode和引擎.lock是否新建；T18采用的SQLite控制文件例外单独列before/after，不伪称文件系统零写。主库/WAL、业务状态与用户原件仍按合同核不可改。

### JSON

自己的已提交恢复错误必须断言顶层 `committed/request_id/original`；Work 还有 revision，Workbook 没有。旧 A 阻断新 B 必须断言顶层 committed=false/request_id=B，detail.pending_request_id=A 与完整 pending_original；顶层 original/revision 不得出现。cause 是 ErrorCode 的机器可读串，message/path 另存。不要只断言 ErrorCode 或用包含字符串判断提交归属。

## 3. 回归样例清单

| ID / 任务 / finding | 合法例 | 唯一改变的条件与反例 | 必须观察到的结果 / 拟新增回归 |
| --- | --- | --- | --- |
| V01 / T19 / R01 | 根内嵌套相对路径 | 改成空、绝对、含`.`、`..`、空段或NUL，逐项单独运行 | 构造拒绝；无文件动作；`managed_path_rejects_dot_and_absolute_segments` |
| V02 / T19 / R01 | 已打开合法父目录 | 在打开父目录同步点后将原路径换成根外软链 | 不操作软链指向的外部对象；原句柄使用策略按设计检查，不假称可阻止同账户搬整个目录；`managed_write_does_not_follow_replaced_parent` |
| V03 / T20 / R01 | 合法 Work state/效果 | 仅改work_dir；另例仅改真实登记write_file.path为../外部；另例仅改输出ref为另一Work | STORE_CORRUPT 或已提交 EFFECT_PENDING，哨兵 bytes/mode 不变；`corrupt_work_directory_cannot_redirect_cancel`、`corrupt_effect_path_is_rejected_before_any_effect` |
| V04 / T19/T24 / R01,R19 | 普通 `.lock/store.db`、父进程持同一根锁 | 逐项把锁/库换成叶软链、硬链；另例只让新CLI等待父进程锁 | 链接拒绝且外部对象不变；锁释放前库/业务目录不存在；`locked_initialization_waits_before_creating_store` |
| V05 / T21 / R14 | 普通installed根与普通文件 | 仅换root/父目录/叶为软链，分别运行；在stat/open同步点换同名另一inode | digest/show/verify不接受替换对象；外部树不读为可信副本；`workbook_digest_rejects_symlink_root` |
| V06 / T21 / R14 | 单文件32MiB、目录256MiB与合法UTF-8名字 | 分别多1字节、读取中增长、硬链、FIFO/特殊文件、非UTF-8名字 | 恰好上限接受，多1准确拒绝；FIFO在受控超时内拒绝而非挂住；bounded实际计数，不整树缓存；保留独立digest向量与旧编码碰撞对 |
| V07 / T22 / R02 | 普通输出观察→COMMIT→保留句柄seal | 一例改原路径为外部软链；另两例在COMMIT后seal前只改同inode bytes或只增加硬链 | 路径例只封原对象、外部mode不变并报绑定错误；bytes/nlink例须重新fstat/hash拒绝、published0、不封错bytes，不能用cached meta；`seal_uses_observed_handle_after_path_replacement`、`normal_seal_rechecks_changed_bytes_and_link_count` |
| V08 / T22 / R02 | 已提交待封存ref仍同bytes | 恢复前仅改bytes、nlink、叶对象类型，分别运行 | committed=true、original/revision正确，published0；不重造输出、不chmod外部对象；`recovery_refuses_changed_output_before_sealing` |
| V09 / T23 / R03 | clean home install/合法rollback | 分别仅把tmp或bin父换根外软链 | 拒绝，根外bytes/mode/路径集不变；合法fixture仍install/update/rollback；`self_install_rejects_symlink_tmp_parent`、`self_rollback_rejects_symlink_bin_parent` |
| V10 / T23 / R04 | 有安装Workbook、冻结Work、pending、binary的管理根 | 在目录删除前同步点注入一个权限/删除错误 | 根和同inode锁始终保留；成功时全部数据树/库/bin消失，失败准确报部分删除且Store保留到最后；`purge_frozen_tree_preserves_lock_and_removes_data` |
| V11 / T23/T31 / R04,R17 | purge与合法install/add、旧Work写命令共享锁，两个场景分开 | 让purge停在清理中，收到waiter“已尝试取锁”事件后继续 | 等待期间不写；结束后合法install/add沿同锁创建空Store，旧Work目标则准确NOT_FOUND且不重建旧Work；不靠sleep；`purge_waiter_starts_only_after_locked_cleanup_finishes` |
| V12 / T24 / R05 | 用@file成功start/submit/fail | 只删源文件；另例只改内容，同argv/request-id | 原响应业务字段、revision、next不变，仅replayed=true；不重新读取；`replay_with_deleted_text_source_returns_saved_response` |
| V13 / T24 / R05 | 新请求的可读@file | 只改为不可读、非UTF-8、超限，分别运行 | 协议退出码、准确错误；不建根、不烧号、不登记成功；修复唯一条件后同request-id可成功 |
| V14 / T24 / R06 | 当时唯一前缀的已提交begin/gate/cancel | 只新增同前缀Work；另例只换成不匹配原Work的前缀 | 原前缀重放成功且目标不变，错误前缀REQUEST_CONFLICT；`historical_prefix_replay_uses_recorded_work_id` |
| V15 / T25 / R07 | 真实begin成功，卡正常 | 只把卡目标换成目录；另例自己pending原件损坏 | requests已提交，顶层committed=true/rid/revision/original完整，cause正确，published0；`post_commit_card_failure_returns_committed_response` |
| V16 / T25 / R07 | 新B无阻断；自己的历史请求正常恢复 | 只让旧A的效果无法完成 | B未提交；B/A字段位置和值正确；自己的A重放不能误报false；`old_effect_blocks_new_request_with_distinct_identities` |
| V17 / T25 / R08 | completed Workbook add/remove重放 | 同一效果变成合法未发布窗口 | 同请求先恢复，完成才成功；失败带已提交原响应；源消失/新生命周期不重新安装或删；`workbook_replay_finishes_unpublished_effects` |
| V18 / T25 / R10 | start COMMIT后尚未发布 | 仅将下一写动词从Work改为Workbook add/remove | 两者都完成同一Work card/effects，原card bytes符合最新Store，mark之前卡存在；`workbook_write_recovers_work_status_card` |
| V19 / T26 / R12 | start COMMIT后唯一pending原件正确 | 分别只改start-input字节、owner format/rid/op/id、final owner、digest_root | 停止、保留原件、不推进B；合法例从pending或已rename的final校验全闭包；`publication_checks_each_start_input_reference` |
| V20 / T26 / R18 | publish的rename与所有sync成功 | 分别在文件sync、源父sync、目标父sync注入错误 | 不标published；自己的错误true/旧A阻B false；按精确对象重试；`publication_sync_failure_keeps_effect_unfinished` |
| V21 / T27 / R11 | 不同internal-id的两个合法remove | 删除后标记前kill；另例给第二请求放第一请求标记；另例坏format/id | 对应标记只证明自己的删除；无证明结果不明；新B不提交；`deletion_without_completion_proof_stops_recovery` |
| V22 / T27 / R11 | payload是登记原目录 | 仅换payload目录内容/对象；另例仅换final成不同摘要的新对象 | 停止、不删不同对象；不能写marker假成功；`delete_recovery_refuses_different_owned_tree` |
| V23 / T27 / R11 | remove成功published1 | 仅重新add相同id/version，之后重放旧remove/add | 新对象bytes与row不变；旧请求只返历史快照；保留已有生命周期回归 |
| V24 / T28 / R13 | owner合法、未提交、无任何Store引用 | 分别改为被published0引用、published1非空payload、无owner目录、Store效果解不开 | 只清合法孤儿；已提交原件与异常树保留；异常引用不明停止；`pending_cleanup_preserves_referenced_originals` |
| V25 / T28 / R13 | 合法完成请求的空container/owner/marker | 仅让cleanup失败；另例只有不完整owner且无目录 | 业务成功/published1保持，stderr警告明确；不重做业务；残片保留且其他合法请求可执行；`cleanup_failure_does_not_repeat_completed_effects` |
| V26 / T28 / R09 | 合法待发布Workbook/Work与合法侧车；另有published1且已清元数据的final | 仅让final暂缺；另例只在读期间rename | list/show/verify/status及start preflight从同一原件读，pending_publish准确、verify为ok；完成对象清理后仍可读/重放；有限重读，无业务写/引擎.lock，SQLite控制变化另记；`pending_workbook_is_readable_without_recovery` |
| V27 / T28 / R09 | pending副本完整 | 仅删owner或唯一原件；另例旁边另装同id其他版本 | STORE_CORRUPT/暂时IO按合同，不能回退其他版本、不能写或恢复 |
| V28 / T29 / R15 | stats/next来自同一装入state | 装入后、渲染前同步让另一写者改变Store | 本响应stats/next仍同revision事实；随后新查询才能反映新状态；`stats_and_next_share_one_loaded_state` |
| V29 / T30 / R16 | plan第一次，旧输入null | 仅让plan-review沿back再次plan | reviewed-plan/tasks是被审输入的原字节镜像；第二brief包含其冻结路径，合法不同节点来源；`replan_brief_binds_previous_plan_and_tasks` |
| V30 / T30 / R16 | task1提交并经独立verify记累计表后重规划 | 仅改整体基线；另例丢verify累计表中一条完成记录/原始证据 | fresh worker只读brief输入+真实Git得到Task/commit/原基线，非法材料停止；最终范围仍含task1；不从冻结tasks或测试闭包猜完成事实 |
| V31 / T31 / R17 | 先启动全部参与者、同步事件控制窗口 | 独立审查旧lazy spawn→join及sleep | 两个线程确实同时到达指定窗口，最后才join；否定控制摘掉锁/同步应暴露违规，不靠重复运行碰运气 |
| V32 / T31 / R17 | 每个能力有生产入口与独立oracle | mutant移除/改变单一保护条件 | 被正确反例捕获；存活逐一说明当前义务、consumer、oracle与保留/删除理由；未运行不能PASS |
| V33 / T31 / R20 | 合法Gate提交count=1，批准后NoLegalEdge count=2；纯core合法submit/fail/approve | 仅把当前持久count改MAX或0；纯core仅改同一字段 | status/stats/list/写命令均准确STORE_CORRUPT而非panic；works/revision/requests/audit与业务文件不变，无新rid；纯core返回既有取值错误且原state不变 |

## 3.1 前置任务与最终闭环

同一个V可能有底层、产品caller、协议封装三个子义务。只完成其中一个不能标整个V或R通过。当前候选已经复现的后继失败继续保留FAIL；尚未运行的新回归为not_run。以下分界同时约束task done与最终M1，修复者不能自行缩短oracle。

| V | 当前任务可验收的子义务 | 最终caller/协议Owner |
| --- | --- | --- |
| V04 | T19：锁/库候选对象的链接与类型拒绝 | T24：真实CLI等待期间不建库，按操作限定初始化权限 |
| V07 | T19：SafeFile对句柄的权限操作 | T22：submit观察→COMMIT→原句柄seal及路径绑定完整性 |
| V08 | T22：原ref/Store/published/结构化提交归属与外部对象不动 | T25：准确完整CLI original/revision/cause/位置 |
| V19/V20 | T26：完整发布闭包与sync失败停止 | T28：pending读取/标记的只读入口；不能要求T26先通过后继读功能 |
| V11 | T23：含冻结树purge的同锁与结果 | T31：等待者到指定窗口的实际同步事件，分别核初始化/旧Work拒绝 |

## 3.2 R01–R20 的最终关闭Owner

| finding | 修复任务 | 验收入口 | 最后关闭前必须具备 |
| --- | --- | --- | --- |
| R01 | T19、T20、T24 | V01–V04 | 路径/持久身份/Store入口全链拒绝，根外哨兵不动 |
| R02 | T19、T22、T25 | V07/V08 | 同观察对象seal与恢复ref、COMMIT后错误封装 |
| R03 | T19、T23 | V09 | install/rollback全部受管caller拒父链接 |
| R04 | T18、T23、T31 | V10/V11 | purge合同采用、冻结树清理、同锁生命周期/等待者 |
| R05 | T24 | V12/V13 | start/submit/fail查重前无源读取，参数/业务错误分类 |
| R06 | T24 | V14 | 全部Work写动词按历史完整目标重放 |
| R07 | T25 | V15/V16/V08 | 自己/旧A阻B/卡/mark/历史核验的完整字段 |
| R08 | T25 | V17/V23 | 未完成重放先恢复，已完成历史不碰新生命周期 |
| R09 | T28 | V26/V27 | 所有只读入口及start preflight，清理后final仍可读 |
| R10 | T25 | V18 | 每类写入口完成Work card后才mark |
| R11 | T27、T28 | V21–V23/V25 | 同对象/本请求marker/结果不明/完成后不重做 |
| R12 | T20、T26 | V19 | 侧车/请求/final/冻结副本/每个起始输入 |
| R13 | T28 | V24/V25 | 全请求引用索引、合法孤儿、异常保留与维护告警 |
| R14 | T19、T21 | V05/V06 | 全树caller与独立摘要向量，软链/FIFO/限额 |
| R15 | T29 | V28 | 同次装入的stats/next及写者实际交错 |
| R16 | T30 | V29/V30 | 合法镜像输入、真实完成来源、fresh worker与Git范围 |
| R17 | T31及独立M1 | V31/V32、§4 | 真实同步点、macOS窗口、Linux豁免/not_run、存活体处置 |
| R18 | T19、T26、T27 | V20及§4 | 必需sync传播与标完成次序；不把kill当断电 |
| R19 | T24 | V04/V11 | 除根/.lock外锁前不初始化；purge后的旧Work不CREATE |
| R20 | T31及独立M1 | V33 | 非法持久blocked_count在真实只读与写入口均拒绝；合法计数推进、纯core溢出与原状态不变 |

## 4. 真实 crash 矩阵

T31 把下面窗口做成子进程测试；生产 feature 名使用现有 `sheltie-runtime/failpoint`。故障点命名不得复用一个点冒充多个不同窗口。测试进程收到明确事件才终止目标进程；exit70或kill的模式与原始输出分开记。点只在 failpoint feature 中生效，正常生产不开。

| 操作 | 窗口 | 下一次动作 | 正确结果 |
| --- | --- | --- | --- |
| Work start / Workbook add | owner已sync、payload创建前 | 新合法写请求 | 合法无引用残留可清；不得有final或成功请求 |
| Work start / Workbook add | 私有副本完成、COMMIT前 | 同request-id重试 | 无业务提交；不发布旧孤儿；重走preflight；Work已分配序号允许空号且不回收/复用 |
| Work start / Workbook add | COMMIT后、publish前 | 同请求重放；另用另一类写动词 | 同原件发布、原快照、全部效果及最新状态卡完成 |
| Work start / Workbook add | rename后、父目录sync/mark前 | 同请求重放 | 核final完整闭包，补必要sync/权限，不再复制、不覆盖 |
| attempt begin | COMMIT后、prepare前/历史write之间 | 新写请求 | 建目录、按登记bytes恢复brief/stats，卡最新 |
| attempt submit | COMMIT后、seal前 | 新写请求 | 核原引用再seal；产物改变则准确停止，不重造 |
| Workbook remove | COMMIT后、移入前 | 同请求重放/新B | 核原对象后移入；不同对象停；B归属准确 |
| Workbook remove | 移入后、删除前 | 重放 | 只删已核同一payload |
| Workbook remove | 部分删除后 | 重放 | payload已不匹配完整登记摘要时停止并保留可用证据，不猜外部替换或自动重新add |
| Workbook remove | 最后删除后、marker前 | 新B/同请求 | 结果不明并停止；B为false，自身重放为true；不得伪造marker |
| Workbook remove | marker已sync、mark前 | 重放 | 合法本请求marker证明完成，mark后cleanup；不能删新生命周期 |
| 任一提交请求 | published1、cleanup前 | 新写请求 | 清元数据，不重做已完成业务效果 |
| self update | binary移到prev后、替换前 | 用prev执行rollback | 原binary字节恢复，Store逐字节不变 |
| self purge | 数据树删除中、删除Store前 | 等待者install/add、旧Work写命令分别执行；同purge重试 | 同锁串行；失败报部分清理；根/锁保持；合法初始化创建空Store，旧Work命令拒绝且不复活旧Work |

每个窗口都要检查Store与文件字节，不能只看路径存在。macOS运行文件模块的目标用例；Linux运行按用户指示豁免并留`not_run`，不据macOS证据声称跨平台行为。完整窗口矩阵在macOS验证本机平台相关动作，同一候选/lockfile/feature原始run可追溯。

## 5. 证据目录与关闭格式

每任务写 `evidence/repairs/tNN/README.md` 及原始 stdout/stderr/JSON；不要覆盖 `evidence/m1-2026-09-28/` 的历史失败。最终 M1 写独立新目录。README 每行用下表，不预填退出码：

| 字段 | 内容 |
| --- | --- |
| 候选 | task、基准HEAD、待提交文件摘要、提交后hash、Cargo.lock sha256、features、rustc、target triple、独立home、run_id |
| 样例 | V编号、合法例、唯一变化条件、真实argv、独立expected、actual、exit、原始输出链接 |
| 文件/Store oracle | 哨兵before/after、原件bytes、SQL快照、request/published/revision |
| 结果 | PASS/FAIL/not_run；模拟窗口/exit70/实际kill/API编译/目标平台实跑分别说明 |
| 审查 | 未参与实现的Reviewer、读取的diff/caller/合同、问题答复、复核结论 |
| 交接 | 新接口及已迁移caller、下一任务输入、剩余失败Owner，不把依赖任务覆盖写成独立执行 |

R01–R20 每行最终包含：修复task/commit、V样例、真实入口、独立oracle、原始run、Reviewer、结论及剩余风险。原O/N矩阵同期更新，只关闭义务已满足的行。完整kill或突变仍缺时，M1保持未通过。Linux运行已由用户明确豁免，保留`not_run`与平台风险，不阻挡M1但不得作跨平台通过结论。

R20由T31负责V33；独立探针与修复结果必须同样记录固定候选、真实入口、单字段反例、原始run和Reviewer，未完成仍WIP。
