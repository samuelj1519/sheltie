# 初次接触项目的工程师：M1 修复执行手册

目标读者：会使用 Rust、Cargo、Git，尚未接触 Sheltie 的初级工程师。目标：关闭 [M1 审查](review-m1-2026-09-28.md) 的 R01–R19，再完成 C002-M1。方案见 [repair-design.md](repair-design.md)，具体 oracle、窗口和证据模板见 [repair-validation.md](repair-validation.md)。任务状态只在 [plan.md](plan.md) 的主表维护；本手册不另建状态表。

2026-09-28 用户已明确授权按本计划修复全部问题，并明确豁免本修复线的Linux验证。macOS arm64/Rust 1.85.0探针已通过，原始证据见[evidence/t18-api-probe-2026-09-28](evidence/t18-api-probe-2026-09-28/README.md)。Linux继续记录为`not_run`，所有修复结论明确限定为macOS验证；不把单平台结果说成跨平台PASS。

## 1. 第一天从这里开始

### 1.1 必要角色与环境

package Owner 指派实施者、架构/合同 Reviewer 和未参与代码修改的独立 Reviewer。初级实施者负责按卡实现和提供证据；Reviewer 负责确认产品选择、接口和实际 caller，不能把这些责任丢给实施者自己猜。用户已明确豁免本修复线Linux运行：Linux相关项保留`not_run`，所有结论限定为macOS；不以cross-compile代替Linux证据，也不宣称跨平台PASS。

在仓库根先执行只读检查：

```bash
pwd
git status --short
git rev-parse HEAD
rustup toolchain list
cargo nextest --version
cargo deny --version
cargo mutants --version
dist --version
```

预期根目录为 `/Users/shushu/orca/workspaces/sheltie/codex`，也可使用另一个经 Owner 指定的 Sheltie checkout。被审源码基准是 `e1a8126a432981df40023628dd06feec7884d458`；不同 HEAD 不是自动失败，但必须先记录差异并由 Reviewer 确认哪些原反例仍适用。当前未提交的 M1 审查报告/evidence 是要保留的材料，不运行 git clean/reset 来清掉。

必须具备 Rust stable 的 fmt/Clippy、Rust 1.85.0、cargo-nextest、cargo-deny、cargo-mutants、dist 0.32.0 和 Python3。缺工具记录具体命令和版本，由 Owner 完成环境准备；不对真实宿主或真实 `.sheltie` 自动安装/格式实验。环境准备不是产品修复 PASS。

依次读取：

1. 根 `CONTEXT.md`：牢记 Workbook/Work/Attempt，Package 只指 Cargo 包。
2. `specs/README.md`、`specs/changes/README.md`：确认仍以 C002 为 active。
3. `specs/constitution.md`、`specs/architecture.md`：理解纯 core、runtime I/O、CLI 解析渲染与单一 SQLite 权威。
4. `specs/engineering.md`：了解测试归属、提交门禁、Reviewer 与白名单。
5. 本手册及 `repair-design.md`：先读到本次任务，再按任务卡读精确合同和代码入口。

### 1.2 用真实链理解代码

先理解五个词：文件路径是名字，文件句柄指向已打开对象，rename后句柄仍指原对象；fsync让所选文件/目录的修改完成必要同步，失败不能吞；COMMIT只提交SQLite内的状态；published表示该请求的文件效果完成，不是Work成功；HomeLock是协作进程串行写管理根的锁，不是独立真人认证。遇到这些机制无法判断时交T18的架构Reviewer，不用unsafe或“先检查再按路径重开”凑实现。

```bash
rg -n 'fn (dispatch|start|begin|submit|fail|cancel|add|remove)' crates/sheltie-cli/src
rg -n 'fn (run_command|recover|finish_request|replay|load)' crates/sheltie-runtime/src
rg -n 'fn (decide|legal_next)|struct WorkLayout' crates/sheltie-core/src
rg -n 'fn (commit|inspect_request|unpublished_requests|mark_published)' crates/sheltie-runtime/src/store
```

沿一条 begin 看 caller→观察→core::decide→Store::commit→effects→CLI；不要逐文件通读整个仓库。`service.rs` 与 `workbook_repo.rs` 是当前流程入口，`effects.rs` 是错误集中处，`fsx.rs` 是底层文件入口。行号可能变化，后续以本卡给出的函数名搜索。

## 2. 修复分工和执行顺序

T18–T31 是追加任务；T16/T17 的号码和含义不变。推荐按下表顺序执行，便于同一位新人逐步接手；依赖允许并行时也只有 Owner 可以安排，不能多个实施者同时改共享模块。

| 任务 | 交付内容 | 依赖 | finding | 责任范围 |
| --- | --- | --- | --- | --- |
| T18 | 固定合同调整、接口选型与小探针 | 用户授权修复并豁免Linux验证；macOS原生探针、门禁与独立review | 所有任务的语义前提 | 合同/架构 Owner 与 Reviewer |
| T19 | 受管路径、目录句柄、文件原子操作 | T18 | R01/R02/R03/R14/R18 的底层能力 | fsx/Home |
| T20 | Work与效果的可信装入/归属 | T19 | R01 | Store/load/effects |
| T21 | 受限目录枚举、摘要与复制caller | T19 | R14 | Workbook/digest/observe |
| T22 | 同句柄观察与封存 | T19、T20 | R02 | submit/observe/seal |
| T23 | self全部文件caller与purge | T19、T21、T18合同采用 | R03/R04 | selfmgmt/CLI self |
| T24 | 请求解析、历史目标、锁内Store创建 | T20、T21 | R05/R06/R19 | Work/Workbook请求、CLI、Store |
| T25 | 统一恢复与正确提交错误 | T20、T21、T22、T24 | R07/R08/R10 | recovery/load/CLI结果 |
| T26 | 发布完整闭包及必要sync | T19–T21、T25 | R12/R18 | publish/pending |
| T27 | 删除对象/完成证明 | T25、T26 | R11 | remove/delete/pending |
| T28 | pending清理与只读对象发现 | T25–T27 | R09/R13 | pending/load/Workbook views |
| T29 | stats与next同次装入 | T25、T28 | R15 | Work事实视图/CLI stats |
| T30 | spec-dev旧计划/任务输入 | T18 | R16 | Workbook/真实CLI-Git回归 |
| T31 | 确定性交错、完整窗口、突变处置 | T19–T30 | R17及全链防退化 | 验证/CI/M1 Reviewer |

T19 交付真实文件操作并迁移现有 helper 的调用方式；T20–T23 各关闭自身的产品 caller。T19 不能先声称四个后续问题全部关闭。T25 建统一恢复主链；T26–T28 增强该同一模块，不新增第二个恢复器。

## 3. 每个任务都执行这一套循环

1. Owner 在主 plan 将本任务设 doing、写明实际实施者/Reviewer；确认依赖已 done。记录当前 HEAD、dirty 文件和本任务白名单。本文列“文件”是导航；可修改范围以 tasks.toml 为准，超界先改白名单并说明原因。
2. 先找到对应 V 样例与旧失败，补一个能编译的真实反例并看到语义失败；编译错误不算红。不要盲目复制旧探针的绝对 binary/temp 路径，它们只指旧运行。
3. 每次只完成卡中的一个职责；接口改变与全部真实 caller/fixture 在本任务一起迁移。不能提交空实现、假成功或接受旧入口的兼容代理。
4. 开发阶段只跑本任务归属与受影响 caller；代码稳定后按下面公共门禁跑一次。失败先查原因，不重复跑到绿。若 review 后改了受影响输入，再按影响链补验。
5. 将命令、原始输出、exit、file/Store oracle、候选文件摘要交 Reviewer。Reviewer 看合同、真实 caller 和失败停止路线，不能只读绿摘要。
6. 局部门槛与独立 review 通过后，主 plan 状态改 done，显式 stage 本任务文件，运行 check-task，再一个任务一个提交。未经成功退出码与提交后核对，不声称已提交。T16/T17另受宿主/发布边界控制。

### 3.1 命令和目标目录

选独立目标目录，示例中的变量不是 HOME：

```bash
export CARGO_TARGET_DIR=/private/tmp/sheltie-c002-repair-target
export RUSTC_WRAPPER=
scripts/task.sh C002-T19
```

Task 号换成当前任务。T18 只有文档与外置小探针，不使用 `task.sh`；没有归属测试时 task.sh 非零是正确诊断，不靠 `--no-tests=pass` 放行。代码任务新测试用本任务注释；移动/取消旧测试先做能力→入口→oracle→Owner→保留/替换矩阵，不为了 task.sh 命中而随意改归属。

每次代码提交前依工程规范运行：

```bash
cargo fmt --all -- --check
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run --all-features --no-tests=pass
scripts/check-docs.sh
scripts/check-specs.sh
scripts/check-core-vocab.sh
scripts/check-tests.sh
scripts/check-skill.sh
git diff --check
```

改依赖另跑 `cargo deny check`、`cargo +1.85.0 check --workspace --all-targets --all-features --locked`。T31/M1还跑完整MSRV、deny与dist plan。工具缺失或 sandbox/cache错误记录环境原因与原文，不把它当产品反例，也不偷用以前候选PASS。

Reviewer 放行后：

```bash
git diff --cached --name-status
scripts/check-task.sh C002-T19 --staged
git commit
git show --stat --oneline HEAD
git log -1 --format=%B
git status --short
```

`git add` 必须按本任务实际文件清单执行，不使用 `git add .`。提交信息示例：`fix(runtime): 限定受管目录句柄操作`，正文解释原因与验证，trailers 为 `Change: C002`、`Task: C002-T19`、`Agent: <实际实施者>`。没有运行过的结果保留 not_run；提交/push不替代门槛。

## 4. 逐任务卡

### C002-T18 固定机制合同与API可行性

Owner：产品合同由用户授权采用；实施者Codex负责整理上游合同和小探针；独立Reviewer确认文件与恢复语义。Linux运行由用户明确豁免并保留为风险/`not_run`；macOS原生API证据、合同diff与Reviewer结论为本任务实际门槛。完成现象：后续每个文件操作/恢复错误都有一套固定合同，无需新人选方案，记录跨平台验证范围限制。

**入口与文件。** 本方案 §2–§7；storage §2.2/§3.2/§3.3/§5.2/§9，protocol §5，architecture §3.2/§4/§5；D-035。拟新增 `D-037-managed-file-handles.md`、`D-038-purge-lock-lifecycle.md`。任务白名单已允许这些目标。

1. 逐条对照根合同，整理“现有要求/采用的调整/采用依据”表。已按用户授权固定purge保留根/.lock、remove仅预建container、cleanup stderr、SQLite只读控制文件例外；补digest_root形状、名字选择器与规范化产物名的分工以保持现有指纹编码；确认T30镜像/累计验证交接且不改core自来源规则。
2. 更新spec/storage/protocol/README与D-035，D-038解释为何同一个锁对象保留；不要重新询问已授权的合同调整，也不要新增根外锁。
3. 更新效果归属与错误合同：自己/旧A阻B的字段、内部snapshot→完整原响应、maintenance stderr不改业务JSON。默认字段不得掩盖损坏；不增加Store schema/持久业务状态。
4. 在 `/private/tmp/sheltie-c002-api-probe-<唯一id>` 建独立Cargo crate，精确rustix1.1.4+fs及rusqlite0.37.0+bundled、unsafe forbid。验证祖先/叶NOFOLLOW、stat/open替换、NONBLOCK FIFO打开后识别并由安全wrapper拒绝、枚举、只读句柄chmod、成功及失败sync、NOREPLACE/目录sync；再验证bundled SQLite RO/WAL/缺shm/活动写者、main/WAL/shm软硬链接、schema1拒绝、短main+有效WAL可读时字节保留。RO在首次查询前set_db_config NO_CKPT_ON_CLOSE=true，关闭后核字节；配对测试main有效且有旧行、WAL含后续提交再损坏WAL header，记录SQLite是否回退main及main/WAL是否不变，不要求SQLite检测/拒绝它忽略的WAL。用外部timeout包住FIFO探针，避免错误实现永久阻塞。记录实际sqlite_version，不以Python另一版本替代bundled gate。源码、lockfile、可执行文件、命令/环境/源码摘要和日志入evidence。
5. 在macOS跑探针，用1.85.0 `--locked`验证选型。Linux由用户豁免，明确记录所有Linux子项为`not_run`和平台风险，不把cross-compile或静态检查当作Linux证据。
6. 架构Reviewer确认最小接口、SQLite NOFOLLOW+受信根限制、无同账户认证承诺，以及完整caller迁移任务归属。文档门禁、macOS原生探针和独立review通过后可结束T18，同时在交接中保留Linux验证豁免与风险。

**验收与停止。** API合法文件成功、叶/祖先软链/FIFO拒绝、目标存在NOREPLACE不覆盖、sync成功和失败都可观察；SQLite控制文件例外和main/WAL不变都有原始证据。短main有效WAL按SQLite提交视图读取，不能预先仅按长度拒绝；WAL损坏时以bundled SQLite实际视图为准，不能宣称SQLite会拒绝它忽略的WAL。任一目标平台/API/MSRV门槛未通过，T18保持未完成。不能用immutable=1绕并发读取，不能偷偷放宽旧数据保留。纯文档低风险部分用链接/字段/差异核对；小探针不代表真实Sheltie caller通过。

**交接。** 给T19的输入为最终ADR、函数职责表、macOS run_id、Linux豁免/not_run记录和已采用合同diff；给T23的是确定purge行为，给T25/T27的是错误/删除状态表。本文材料不等于这些采用与运行已经完成。

### C002-T19 受管文件原语

Owner：runtime fsx/Home；依赖T18。完成现象：真实helper不再靠路径字符串重开对象，根内操作只接受已校验路径与打开目录。

**定位。** `fsx.rs` 的 SafeFile/ensure_dirs_under/write_exclusive_atomic/write_new_file/remove_tree_no_follow/fsync_dir；`home.rs` 的 resolve/rel/acquire_lock_once；runtime Cargo.toml与根workspace.dependencies。按方案 §2。

1. 引入ManagedRelPath校验与安全目录句柄；写至少V01反例，禁止从AbsPath或String不经检查造受管目标。去掉Home::rel失败退root与ParentDir跳过路径。
2. 实现逐段openat、NOFOLLOW、fstat类型/nlink/身份检查；目录枚举复用打开句柄。明确外部只读文件入口与受管入口，@file没有写权限方法。
3. 实现独占临时创建、write+file sync、原子替换投影、NOREPLACE发布新目标及父目录sync。必要sync返回Result，传播到全部现有caller；不留忽略Result的包装。
4. 让HomeLock从同一根句柄打开/校验普通单链接`.lock`，保留既有purge身份复核；代码不把根外system祖先的合法别名当子目录软链例外。
5. 迁移既有fsx/Home helper及所有签名受影响caller，在这一个任务里编译、更新fixture和真实目录测试；后续任务再收掉各产品caller自行调用std::fs的遗漏。若typos hook误将rustix API常量`WRONLY`改成无效拼写，只将该精确API词条加入根`_typos.toml`并扩展本任务白名单，不排除整个源码文件。
6. 开发跑fs_boundary/home_observe和本任务归属用例，稳定后执行公共门禁、依赖deny/MSRV、独立review与check-task。
7. 移除历史T15的`sleep(100ms)` + purge删根 waiter 用例：它断言的旧生命周期已被T18采纳的D-038取代，且没有同步点。T15原始evidence保留；正常同锁purge等待者由T23/T31按V11确定性重写；锁外根或`.lock`被替换后的等待者身份复核与重试由T31独立验证，二者不能合并成一个测试，也不能把旧测试改成Linux豁免理由。

**验收。** 本任务执行V01/V02、V04的锁/文件类型子条件、V07的SafeFile句柄权限子条件；用真实底层入口和已迁移helper确认嵌套文件、stat/open替换、链接、FIFO、占位与exact/+1。V04“真实CLI锁前不建库”留T24，V07“submit同句柄封存”留T22，原FAIL保留，不把整条V先标PASS。停止：哨兵bytes/mode变化、sync被吞、加unsafe/路径fallback。交接：接口与已迁移caller清单，剩余产品caller分别归T20–T24；不标对应R全链关闭。

### C002-T20 持久路径与效果的可信装入

Owner：Store/load/effects归属；依赖T19。完成现象：损坏state/effects在任何文件动作前被准确拒绝。

**定位。** `store/read.rs::decode_row/inspect_request`，`WorkState::validate_persisted`，`service.rs::load/pending_workbook/core_effects_to_ops`，`effects.rs::decode_effects/RefJson`。新增 `load.rs` 与必要Raw/Checked内存类型，按方案 §3。

1. 保留纯core的状态组合校验，在runtime补Home/WorkId/WorkLayout精确路径归属；key/node/output使用已校验定义，不能仅用starts_with。
2. 读取请求时一并装入audit/快照/对应Work，检查audit唯一且归属一致；缺失、重复、损坏均携带请求或row定位。恢复枚举必须先从requests取全量未完成行，再查audit；不能用INNER JOIN把缺audit的请求过滤掉。
3. 将Raw效果批次完整校验为内部Checked对象：每个命令有固定效果种类、数量与顺序；paths、digest、contenthash、Attempt和目录/输出归属都与同一请求绑定。`Reply`、`NextOp`、persisted response与Workbook snapshot严格拒未知/缺字段；原schema 2 audit字节保持不变（add.source仍是intent hash，remove仍是id@version target）。整批校验完成前不得执行第一个效果，缺必需效果也不得标published。
4. 把Work的冻结图装入提到`load.rs`；Work根、start inputs、每次Attempt输入/输出、状态卡和WorkBook路径均与WorkId、WorkLayout及冻结定义精确比较。WorkBook行id/version/digest/added_at/dir与最终manifest identity一致。迁移Service与Workbook真实caller，Raw效果不再有直达execute入口。
5. pending owner侧车由`deny_unknown_fields` DTO解析，stage也用serde序列化；request-id含引号/换行仍能往返。每个EngineStats引用的hash/bytes与同次begin登记的唯一WriteFile相同。补合法真实请求及单字段path/owner/hash/bytes/audit/reply错例；第二效果坏路径要断言第一个历史文件仍缺、哨兵bytes/mode不变。
6. 跑store/schema2_replay/workbook_repo/fs_boundary与本任务用例；记录每条非法字段定位、Store/哨兵原值。Review确认没有未经Checked校验的effects执行入口。持久字段或schema不得重编码、迁移或加默认猜测。

**停止与交接。** 异常值被自动纠正、绝对路径流进file caller、RawDTO仍可直接execute都停止。交T25已校验请求/效果批次接口，交T26发布/删除所需闭包；持久JSON字段不重新编码或迁移。

### C002-T21 受限树、摘要与复制

Owner：Workbook/digest/observe；依赖T19。完成现象：installed/source/frozen目录均走同一安全枚举器，摘要向量不变。

**定位。** workbook_digest::collect_regular_files/walk_regular_files/stream_file_into；observe::build_resource_index/walk_dir；fsx::copy_tree_confined/set_tree_readonly_confined；WorkbookRepo::load_dir/copy_confined/digest_dir。

1. 列出add源、installed load/verify、start冻结复制、Work load、publish verify六个caller，逐一标它们用哪个目录句柄和只读/受管权限。
2. 替换read_dir/stat/按路径File::open组合，第一遍取名字/身份/长度，第二遍开同一对象有限读取；拒根、父、叶链接及非UTF-8名字。
3. digest保持prefix/BE64/filecount/path/contents单SHA256，独立向量原样保留。资源索引只缓存实际需要的文本，资源正文流式核限额。
4. 复制使用打开源/独占目标句柄，记录实际字节和sync结果；最终副本再parse/compile/digest确认身份。全部readonly权限包含目录根，chmod从句柄执行。
5. 补V05/V06；合法副本与只有installed根软链的反例走真实show/verify/start，不只测digest helper。
6. 跑workbook_digest/workbook_identity/workbook_repo与受影响CLI；复核边界和原oracle，不把摘要格式“优化”成新算法。

**停止与交接。** digest合法向量变化、整树正文缓存、copy目标可跟随链接、失败后以预检源身份登记都停止。交T26安全遍历/同步/副本核验能力及真实caller清单。

### C002-T22 同句柄输出观察与封存

Owner：submit/observe/SealOutputs；依赖T19、T20。完成现象：正常submit封住原观察对象，恢复只封验证过的原引用。

**定位。** service::submit/observe_output/run_command，fsx::SafeFile，effects::SealOutputs，core::ObservedFile与Effect::SealOutputs。core不保存或读取fd。

1. 把runtime观察结果分为送core的文件事实和本调用持有的SafeFile集合；按输出名对应，不把fd序列化入effects或WorkState。
2. 正常COMMIT后传同一集合与已提交ArtifactRef给seal；先核原句柄类型/nlink及当前sha/bytes仍相符，再chmod/sync同对象，禁止按路径重开或chmod。删除错误macOS注释。
3. seal后核原路径仍绑定观察对象；被替换时外部对象不动、原观察对象已封存，但本请求按已提交效果错误停止标完成，保留published0。原对象内容已变则先停止，不能封错bytes。
4. 崩溃恢复从Checked ArtifactRef重新安全打开并核类型、nlink、bytes、sha后封存；已完成submit重放不seal。关闭fd的时点不能早于本调用封存。
5. V07用同步点分别换路径、在COMMIT后seal前改同inode bytes或加硬链，确认seal重做fstat/hash而非缓存；V08分别改重启恢复产物bytes/nlink/type。先验原ref/requests/revision/published/结构化归属与原/外部mode；完整CLI original/cause/revision封装由T25验，保留后继FAIL，不缩成假PASS。
6. 跑fs_boundary/schema2_replay/CLI scenario_artifacts，核COMMIT前/后Store与错误归属。

**停止与交接。** 重新按路径chmod、外部mode变化、封存失败回滚成功状态、恢复重造输出都停止。交T25可使用的正常封存上下文与恢复动作；最终错误封装由T25统一，但本任务必须保留结构化错误与提交信息。

### C002-T23 self 文件链与 purge

Owner：selfmgmt/CLI self；依赖T19、T21、T18采用purge合同。完成现象：全部self动作根内、冻结树可清，purge始终保留同一根/.lock。

**定位。** selfmgmt::install/update/rollback/uninstall及make_executable/find_named_file/download helpers；commands/self_cmd.rs，runtime/CLI selfmgmt tests，README与storage §9。

1. 逐个替换create_dir_all/rename/remove/set_permissions的受管caller；install的tmp和rollback的bin/tmp不能遗漏。外部current_exe只读，解包候选必须普通文件，包内链接不能引出根。
2. install/update/rollback仍同HomeLock；update/rollback不打开Store。download/checksum/extract失败保持旧binary/prev，固定tag不混包，tmp安全清理。
3. purge先确认授权flag，再持原锁预检将删对象和权限；安全放开已核目录的必要权限，按树→pending/tmp/bin→数据库顺序删，根/.lock排除。
4. 成功输出明确保留根/.lock，kept/路径载荷按T18合同；失败保留准确部分清理信息，不能假称完全未动或自动重建Store。已有用户schema数据不因install/update被清空。
5. 补V09–V11及五个update失败窗口；测试使用真实含冻结Workbook/Work的home，比较Store字节、哨兵与lock inode。
6. 独立review实际每条self caller，再跑selfmgmt/self_cmd/crash、本地release fixture与公共门禁。

**停止与交接。** 根外对象变化、purge删锁、失败先删Store、新binary校验失败替换旧binary都停止。T31要再用真实同步点验证purge等待者；本任务不能用睡眠代替交错证明。

### C002-T24 纯参数、历史目标和锁内建库

Owner：请求入口/CLI/Store；依赖T20、T21。完成现象：同请求不依赖当前文件/前缀唯一性，新库只在取得锁后出现。

**定位。** cli::parse_input_arg/parse_text_arg/check_at_file；commands::Ctx::store与commands/work::service/resolve；WorkService::new/start/submit/fail；WorkbookRepo::new/add/remove；Store::open/connect。

1. 将Service/Repo构造改为Home-only、无I/O，迁移全部生产CLI、public tests、common fixtures；删除Ctx里写命令先开RW的流程，不保留旧构造兼容入口。
2. 新增内部WriteSession拥有HomeLock与锁内RW Store；先只读预检，合法新请求才创建根/.lock。固定install/add可初始化，其他Work写动词和remove只打开存在库。所有RW/PRAGMA在锁内，旧Work等待purge结束后NOT_FOUND且不能建新库。
3. 历史request的完整WorkId先于当前前缀解析；所有begin/submit/fail/approve/cancel走同一resolver。原前缀不匹配该Work即RequestConflict。
4. CLI只解析@path，不stat/open；submit/fail读取放查重后build。新请求@file拒绝叶/祖先链接、硬链接、不可读、非UTF8、非普通文件及超过storage §5.3所定32 MiB单文件上限，报专用INVALID_REQUEST/exit2与path/reason；普通managed IO仍exit1，已读摘要超过4096字节仍SUMMARY_TOO_LONG/exit1。保持RequestIntent固定字段/序列化和既有独立向量；名字参数含省略状态仍进意图，规范化产物名进state/snapshot，文件bytes不进指纹，不能自动改旧hash。
5. 只读Store用正确READ_ONLY/NOFOLLOW flags，无CREATE数据库与RW fallback；叶/侧文件及根/锁身份按T18限定，SQLite共享内存控制文件变化单独报告。老schema拒绝前不写PRAGMA/main/WAL/业务文件，不清空旧根；不存在根不因只读而创建。
6. V04、V12–V14：父进程先持锁，观察child等待时库不存在；源删/内容变重放、同前缀新Work、wrong前缀；SQL核无额外request/revision/sequence。

**停止与交接。** 锁前库出现、重放触发任何file读、历史目标改绑、旧库bytes变化或readonly自动RW均停止。交T25统一的当前请求身份和Session，self文件专用锁不能被强行塞进会开Store的Session。

### C002-T25 统一恢复和提交错误协议

Owner：recovery/load/结果封装；依赖T20、T21、T22、T24。完成现象：任一Work/Workbook写入口都完成同一效果集合，错误字段忠实描述本请求。

**定位。** service::recover/finish_request/replay/refresh_cards_of，WorkbookRepo::recover_workbook_effects与add/remove早返，effects::execute，error.rs::EffectPending，CLI error_map/output；新增recovery.rs。

1. 把恢复循环移到不依赖Service/Repo的recovery模块，使用load.rs刷新卡；两个入口recover_before(current_request)/finish_request共享Checked批次执行。
2. 查意图相同时先确认“当前请求已提交”的身份，不能把自己的pending失败包成新请求被阻断。按audit.seq恢复，所有旧效果完成后新B才能进入事务。
3. 执行包含prepare、write、seal/delete、最新card的完整序列，再mark；删除Workbook replay的无锁成功早返，保留完成请求不再读源的语义。
4. 用结构化cause和完整原snapshot产生准确顶层committed/request_id/revision/original；旧A阻B的detail.pending_*位置正确。单个效果失败、卡失败、mark失败、历史核验失败均走同一包装。
5. T25只统一效果失败的业务错误封装。当前候选尚无`published`元数据清理器；清理执行、诊断传递、CLI stderr/exit 0、成功JSON与历史快照不漂移，以及清理失败不重做效果，全部明确交T28实现并由V25验收。T25不能把这项标为已通过；T28完成前该维护诊断义务保持未关闭。
6. V15–V18：同请求pending、旧A阻B、卡目录占位、missing历史父、Workbook恢复Work；检查完整JSON与SQL，不仅看code。迁移所有旧回复fixture和skill里恢复用法所需字段引用。

**停止与交接。** 自己committed=false、B请求id为空、原响应混当前状态、漏card却mark成功、异常被先stringify丢code都停止。交T26–T28唯一恢复入口与错误上下文；未来修复不得重新在Service/Repo加恢复循环。

### C002-T26 发布归属与sync

Owner：publish/pending；依赖T19–T21、T25。完成现象：唯一原件的完整身份/输入bytes验证通过且必要sync成功才标完成。

**定位。** effects::publish_dir/verify_owned_digest，service::stage_pending/start，WorkbookRepo::add，新增pending owner解析；方案 §3/§6.1。

1. 先serde写、sync合法owner与pending父，再创建payload；rid为opaque数据，不能用字符串插值生成JSON或当路径段。
2. publish前核owner格式/id/rid/op、request/audit/snapshot归属、final与digest_root；Work逐个核start-input引用，Workbook核最终副本manifest身份与digest。
3. 按四格状态表处理pending/final；“仅final在”也核全闭包，完成权限和sync。两者都在或都缺停止保留原件，禁止覆盖/从源目录重造。
4. 逐个sync实际文件/必要目录、NOREPLACE rename、源与目标父sync、readonly权限sync；错误传播到T25，再mark。用命名fault hook确认顺序，不能依赖mock自己记录成功。
5. V19分别只改一个输入/owner字段；V20分别注入每个sync错误；合法rename后恢复必须不重新复制，card用最新state。
6. 跑真实start/add与下一类写请求恢复，核请求行/final内容/原响应/published；已完成final的status/show作为真实caller正例。pending专用只读发现/标记是T28义务，保留其原FAIL，T26不要求后继能力提前通过。

**停止与交接。** owner只判非空、只核Workbook不核start-input、sync失败仍published1、不同final被覆盖都停止。交T27规范pending路径/owner与fsync helper，交T28只读对象发现可用的闭包。

### C002-T27 删除同对象与完成证明

Owner：remove/delete/pending；依赖T25、T26。完成现象：只有本请求的合法标记能证明删完，结果不明准确停止。

**定位。** WorkbookRepo::remove的stage_pending与CommitInput；effects::delete_dir/write_deleted_marker；方案 §6.2与storage §5.2。

1. remove只准备owner/container，不预建payload；从合法owner与规范路径得到internal_id，禁止file_name(payload)作为id。
2. 保持引用扫描/删row/requests/audit同事务；COMMIT前核登记目标，目录不存在或登记摘要缺失不能用空串跳过校验。
3. final移入前核同对象与完整摘要；安全移入、sync父目录。恢复payload也重新核，部分删除后摘要变化则停止，不猜它是可自动继续的对象。
4. 真实删除完成后sync父，独占写自己的marker并sync；marker已有时完整校验format/id/类型/nlink与本请求关联。两个目录都缺且无合法marker直接报结果不明，不创建证明。
5. V21–V23包括两个remove不同id、交叉marker、非法marker、最后删除后kill、不同payload/new生命周期；区分B未提交与自己已提交的封装。
6. 旧`payload.deleted`与损坏记录原样保留并报告，不自动迁移为某个id.deleted；公共门禁与独立review后交T28。

**停止。** 凭目录缺失/marker.exists就成功、删除不同对象、结果不明变自动PASS、cleanup重跑旧remove，都停止并保留错误原文与原响应。

### C002-T28 pending安全清理与只读发现

Owner：pending/load/Workbook views；依赖T25–T27。完成现象：合法孤儿可清、已提交唯一原件不丢、pending对象可只读访问。

**定位。** pending.rs，Store请求引用查询，load::locate_committed_object，WorkbookRepo::load/list/verify，CLI workbook list/show/verify、Work status、start preflight。

1. 为全部请求建只读引用索引，解析/校验完成后才清任何对象；不只扫unpublished。Store效果解不开时停止，不能把目录误判无引用。
2. 按owner+引用+published+目录状态表处理孤儿、空容器、残片与异常树；每次只清自己的对象，永不按年龄删pending。完成后清理失败仅维护告警。
3. 提供同一只读locate接口：未完成/pending须合法侧车；已完成且清理元数据的当前final以对应业务行/成功请求/effect/内容核身份。历史完成快照重放不碰当前新生命周期。有限final→pending→final重读，读者不取引擎.lock、不恢复、不建库；SQLite控制文件按已采用例外处理。
4. 向list/show/verify/status输出pending_publish事实；verify合法pending为ok，不能missing；start只读preflight用pending冻结图，锁内恢复后从final重核。
5. 将实际定位的冻结目录传给图/资源/说明书load，登记ArtifactRef仍是final路径，不能把pending路径写回业务状态。
6. V24–V27逐项运行；再测published1+owner/container已清后status/show/verify及历史重放，避免自身cleanup破坏读取。捕获业务路径/bytes/mode、SQL、.lock前后，SQLite控制文件另记；读rename交错用同步事件。公共门禁后复核read/write caller。

**停止与交接。** 清被引用原件、忽略坏Store引用、读取任意另一版本、只读生成.lock、cleanup失败重做业务均停止。交T29同次只读Loaded状态与定位事实；交M1 pending剩余异常Owner/停止路线。

### C002-T29 stats与next一致快照

Owner：Work事实视图/CLI；依赖T25、T28。完成现象：一条stats回复内部同一state/revision，即使同时有写者。

**定位。** WorkService::stats/status，load.rs的Loaded，commands/work.rs::stats，core::render_stats/render_stats_json/legal_next。

1. runtime从同一次load返回stats文本/结构化事实及同state的next，不复制render规则；仅新增返回值/小结构，不加持久视图表。
2. CLI移除第二次svc.status，不在渲染后再查Store；同步所有函数签名caller与fixture。
3. V28在装入完成同步点阻住读者，让写者改变Work，然后放读者渲染；用手写原state期望断言stats和next匹配，之后的新查询可见新state。
4. 复验累计blocked、非零耗时、同node不同edge、fail_reason/完整ArtifactRef；文本/JSON期望不是相互生成。
5. 跑CLI work/core render相关用例、公共门禁；Reviewer核“同一次加载”注释与实现一致。

**停止与交接。** CLI仍二次load、从最新状态拼旧stats、以锁住全部读写掩盖快照问题都停止。交T31本响应的一致事实oracle。

### C002-T30 spec-dev重规划交接

Owner：Workbook作者/CLI-Git回归；依赖T18。完成现象：第二次plan的fresh worker只凭任务书就拿到旧整体基线和已完成任务。

**定位。** `workbooks/spec-dev/flows/default.toml` 的plan/plan-review inputs+outputs，instructions/plan-review.md/plan.md/implement.md/verify.md/fix.md，resources/templates/plan.md/tasks.md/report.md，CLI scenario_spec_dev，core examples。

1. plan-review新增必需reviewed-plan/reviewed-tasks输出，审核人按任务书原字节复制本次输入再交decision；批准/打回都保留被审副本。plan增optional previous_plan←plan-review.reviewed-plan、previous_tasks←plan-review.reviewed-tasks。保留core禁止自己作为来源的规则，运行现有compile确认显式back可达。
2. plan再绑定optional previous_verification←verify.report、previous_change←implement.change、previous_fix_change←fix.change。首次无旧计划才取HEAD，之后从被审副本复制原始基线；不从冻结tasks猜后来完成事实。
3. change记录继承verify输入路径并原样携带表；verify写本轮检查change/fix及继承来源，逐行比对并保留全部原行，独立过Git/范围/门禁才追加当前Task。fresh planner沿这些源路径核累计前缀和Git/审批/原始证据，漏行或改写停止；对尚未验证的下一任务，只保留其已继承事实，不标新Task完成。改变验收条件列重验项；表保留引用不复制全文，符合32768字节上限。
4. V29真实CLI first plan→review→back，核被审副本bytes与第二brief实际路径。V30完成并提交task1、独立verify追加累计表后replan；fresh worker只从新brief绑定文件和project Git重建整体基线及已验证Task/commit，不捕获baseline0/已完成列表到闭包给它。
5. 用独立git diff/file set证明task1与新变更都在最终范围；反例只重设新基线/丢一条完成任务/旧输入缺字段，worker按说明停止。
6. 同步Workbook README/模板、全部plan-review human输出fixture（新镜像文件须原字节复制）及源码静态图用例；修正强制tNN测试名前缀的旧指引；跑scenario_spec_dev/core examples/skill引用与公共门禁。Workbook新定义只装独立fresh home，已运行Work的冻结副本不原地改写。

**停止与交接。** 靠聊天/测试缓存补旧事实、引擎从文档推断批准、改图规则绕非法输入、改整体基线都停止。交T31真实fresh worker输入与临时Git oracle；真人质量仍留T16。

### C002-T31 全链验证、并发与突变处置

Owner：测试/平台操作者；Reviewer未参与T19–T30代码。依赖所有修复任务。完成现象：R17有实际交错/窗口/突变证据；M1可用同一候选复核全部R/O/N。

**定位。** runtime tests/service.rs假并发、selfmgmt purge等待者、schema2_replay/workbook_txn/crash，CLI共用binary helper，`.config/nextest.toml`、scripts/mutants.sh、必要CI文件。

1. 先做能力→真实入口→oracle→Owner→保留/替换矩阵。把lazy spawn/join改为先spawn集合、barrier/channel等待明确事件、最后join；purge等待者不以sleep代表已进入等待。
2. 将runtime中真正需要子进程CLI的crash/OS主体用例迁到sheltie-cli/tests（T30全仓验收已暴露测试内重建共享binary的ENOENT竞态，因此此迁移提前随T30落地；T31核迁移矩阵及当前候选覆盖），保留runtime的直接API用例；CLI集成测试用Cargo注入的CARGO_BIN_EXE_sheltie，不在测试体启动cargo、不猜target路径。CLI使用T29提供的failpoint feature映射runtime/failpoint，crash用例只在该feature下启用；全features门禁覆盖它。手工probe从Cargo JSON取executable。迁移前后能力/oracle/Owner一一对照，不能删掉故障窗口。
3. 为repair-validation §4逐窗口增加精确failpoint与子进程用例。API编译、模拟目录/SQL、exit70、实际kill分开存证；删除不明窗口的正确结论是停止。所有“存在”断言补文件bytes/Store归属。
4. 在macOS以同候选/lockfile/feature运行文件/删除/publish/锁相关用例。Linux运行按用户明确指示豁免，保留`not_run`并将最终结论限定为macOS。
5. 为core/runtime新候选生成完整mutant列表，按能力分片执行并记录每个存活体；保留安全/恢复/real-entry不同oracle测试，删除死代码只在consumer与义务已不存在的证据下进行。
6. 若替换或改归属触及legacy测试卡，保留归档完成事实，用明确replacement注记链接新覆盖，不能改历史PASS。任务脚本如需支持C002-M1，按现有Tnn的active-package路径路由扩展Mnn，验证它使用本package白名单/状态；不拿legacy表检查新里程碑。
7. 完成所有公共门禁、deny、MSRV、dist plan；固定candidate hash并交独立M1 reviewer。T31 done只证明验证任务自己的清单满足，M1由另一个门槛决定。

**停止与交接。** 存活体无处置、运行共享未变异binary、kill被模拟窗口代替、失败输出被覆盖、真实新反例FAIL，全部停止关闭对应义务。Linux已由用户豁免，记`not_run`并列为剩余平台风险。给M1完整修复commit与V/R/O/N矩阵、macOS raw runs、突变处置、实现者逐条答复。

## 5. 突变运行与长任务续接

先记录源码/lockfile/features/toolchain候选。为突变建立独立普通Git clone；linked worktree的.git文件不能直接复制，否则会指回原metadata。若T31候选仍有待提交文件，按显式清单复制候选bytes到clone并逐文件对sha，临时clone可提交测试候选，不能拿临时提交当产品完成。清除GIT_DIR/GIT_WORK_TREE/GIT_INDEX_FILE等继承变量，核clone自己的.git目录与source/input closure。

在该clone中运行scripts/mutants.sh；相对target让每个变异副本隔离，不能改回全局绝对target，也不能注入指向未变异binary的外部路径。CLI tests的Cargo注入binary必须属于当前变异副本并包含相同feature。每次运行前记录inventory：

```bash
cargo mutants -p sheltie-core -p sheltie-runtime --list --json --exclude 'crates/*/src/testkit.rs'
scripts/mutants.sh sheltie-core --all-features --test-workspace true --copy-vcs true --jobs 1 --shard 1/4
```

按1/4至4/4、core/runtime分别运行；可按实际机器能力调整片数，但必须证明集合无漏无重。每片结束先保存target/mutants.out的日志/JSON到本片evidence再运行下一片，避免覆盖。先验证未变异baseline绿与真实binary来自该副本；Git元数据/fixture指向不正确时停止这片。旧候选的1137只是历史列表，当前生成数以当前输入为准，不能据此挑少数当完整突变。

每个存活体记录函数/变异diff、影响能力、真实consumer、现有oracle为什么漏、补测或等价/无当前义务理由、Reviewer处置。等价或不适用不是测试PASS；超时/不可构建另记，不能静默当caught。中断后只从同输入闭包且raw run可追踪的片续接；改生产代码先重新核哪些片输入失效。

## 6. M1 最终关闭与后续

先确认主plan T18–T31均满足各自门槛，固定最终commit与输入闭包。Reviewer按照repair-validation的R01–R19矩阵及原O01–O13/N01–N14逐行复核；不把task done当证据，不把T31自己的自查当独立M1。

按原M1门禁在同候选跑Rust四门禁、deny、docs/specs/core-vocab/tests/skill、MSRV和 `dist plan --output-format=json`。已有同闭包raw run可复用时写清executed/reused及相同输入证据；有代码/fixture/feature/工具链变更就按影响链重验。完整窗口和mutant处置缺失时M1不能通过。

最终review.md只写通过/需修改/阻断，并链接本轮候选与原始输出；validation保留旧FAIL与新证据，progress写下一入口。M1通过只代表源码与离线可靠性闭环，T16真实Host/usage和T17发布授权/四平台资产必须分别完成。
