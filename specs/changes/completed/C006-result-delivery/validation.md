# C006 验证

Candidate: `c62f14d7e28a2f63394a60d9b702cc03e85de6f1`

状态：`completed`。实现、两个精确窗口、指南与M2限定审阅完成。真实收益未执行。下表前半是计划oracle，执行事实见阶段记录。

## 1. 源与目标验收矩阵

| 要求 / 风险 | 合法例 | 拒绝例或故障例 | 归属 |
| --- | --- | --- | --- |
| 唯一结果选择 | succeeded/final/无待完成效果且选集非空 | 未完成、effects_pending、空选集、未知字段或格式，目标无成果目录 | T01 原语 / T02 真实链 |
| 精确 revision/key | 同一次结果 revision 的正式 Artifact | revision 不同、key 不存在、非法路径、草稿均拒绝 | T01 原语 / T02 真实链 |
| 实际源字节 | 同句柄 size/sha 正确，原字节 stdout | 替换、截断、增长、改写、非零退出、缺字节不发布 | T01 原语 / T02 真实链 |
| raw CLI | 二进制零字节/非 UTF-8 内容逐字节相同 | --json、--request-id、单独 artifact/revision 拒绝，不补换行 | T01 原语 / T02 真实链 |
| 格式与实际能力 | 同格式和读取能力、不要求 release 相同 | 旧或未知 mode 非零，私有暂存不得升级完成 | T01 原语 / T02 真实链 |
| 限额 | JSON 恰好 1 MiB；单文件32 MiB、总量256 MiB接受 | 超一、加法溢出或源实际输出超过声明拒绝 | T01 原语 / T02 真实链 |
| 授权父目录 | 已存在无链接目录，私有新对象 | 不存在、链接、管理根重叠、身份变化拒绝 | T01 原语 / T02 真实链 |
| 目标对象 | 独占普通文件、单硬链接、私有目录 | 软链、硬链、特殊文件、路径注入、名称碰撞拒绝 | T01 原语 / T02 真实链 |
| 全体发布 | 所有 bytes 与清单核验后一起出现 | 任一项错，最终目录不出现；不把暂存称完成 | T02 |
| 不覆盖 | 唯一目标名目录 NOREPLACE | 目标竞争创建，旧字节、inode、权限不变 | T01 原语 / T02 真实链 |
| 发布后未确认 | 移动和同步成功 complete | 移动后 sync/身份失败 publication_unconfirmed，不删除或回滚 | T01 原语 / T02 真实链 |
| 残留与重跑 | 每次重跑创建新副本 | 不复用旧暂存、不接管不明目录、不覆盖旧副本 | T02 |
| 跨设备 | 源目标不同设备仍逐字节复制 | 不通过源 rename/link/reflink 转移；载体不可用记 not_run | T01 原语 / T02 真实链 |
| 业务状态只读 | 导出前后 Work/request/audit 语义不变 | 不写回执、非final、不偷做效果恢复；SQLite控制文件另识别 | T01 原语 / T02 真实链 |
| 可编辑副本 | 发布字节核验且用户可编辑 | 后续编辑不改变 Work，不声称清单代表永久完整性 | T03 |

所有拒绝例核实际输出目录、源与根外哨兵字节、inode 和权限，不只核错误码。可重复竞争钩子仅放外部边界；不能 fake 直接返回成功。实际 platform NOREPLACE 失败阻断该平台支持。

开发包的 publish=false 和 package.metadata.dist.dist=false 必须在本地 dist plan 中证明没有 exporter 发布资产；workspace packages override 不能绕过排除。该检查不执行 release 构建、上传或外部发布。

## 2. 崩溃与竞态

必需 kill/sync 窗口：私有目录创建后、单文件接收中、文件同步后、全部成果读回前、清单写入中、暂存树同步后/移动前、整目录移动后/父同步前、父同步后/响应前。前六类没有完成目录；移动后可能有完整目录但缺确认。重跑只能新建副本，逐字节核已有对象不变。

竞态覆盖父路径打开后替换、暂存目录身份变化、文件软硬链接注入、最后移动前竞争目标名，以及两次独立导出。句柄操作不能重定向到替代路径，观察到异常即拒绝。测试不声称能隔离恶意同权限进程。支持平台必须真实运行，其他平台与缺失跨设备场景列明 not_run 及影响。

## 3. 真实副本价值

使用 C007 或真实任务已识别的副本用途。先固定相同结果、质量标准和记录口径，再记录手工与工具路径。找结果收益归 C004，复制与核对收益归 C006。

记录人工操作和核对分钟、选错文件/漏文件、总完成时间、失败时能否正确判断是否发布，以及用户是否需要副本。单次成功只证明该场景；无真实需求保持 not_run，不用示例输出或 kill fixture 制造价值。

## 4. 验证记录

采用后固定候选、工具链、features、平台、C004 DTO、fixtures、环境、两个 binary 摘要、过滤器与实际命令。复用须有相同闭包与原 run ID；执行模式不是 PASS。

| 原定验证义务 | 原计划模式 | 待固定输入 | 原计划命令 | 原状态 | 原计划证据 |
| --- | --- | --- | --- | --- | --- |
| C004 结果与可信源读取 | not_run | 待 T01 固定 | 未执行 | not_run | 无 |
| 目录句柄与 NOREPLACE | not_run | 待支持平台固定 | 未执行 | not_run | 无 |
| 整份复制与 raw CLI | not_run | 待 T02 固定 | 未执行 | not_run | 无 |
| 崩溃、sync、竞争与跨设备 | not_run | 待 T02 固定 | 未执行 | not_run | 无 |
| Work 业务状态不变 | not_run | 待固定前后快照 | 未执行 | not_run | 无 |
| 首次读者与真实副本收益 | not_run | 待真实任务 | 未执行 | not_run | 无 |
| T01 原语 green 与 T02 行为 red | not_run | 待 T01 阶段闭包 | 未执行 | not_run | 无 |
| C006-M1 安全与实现准备 | not_run | 待骨架完整候选 | 未执行 | not_run | 无 |
| 最终工程检查与 C006-M2 | not_run | 待最终候选 | 未执行 | not_run | 无 |

缺输入、超时、中断和未跑平台逐项记录，不写 PASS。静态方案通过、本地工具构建、产品正确性、真实收益和发布状态分别结论。

## 阶段验证与测试基准

T01 复杂作者创建 `verification/commands.sh`、所有阶段 tests/fixtures 与 `experiments/runbook.md`，实际名称、过滤器、测试归属、counts 和预算在交接时固定。原语测试归 C006-T01、实际 green；新完整行为测试归 C006-T02、初始 ignore，以 task.sh 显式运行。future-red 至少一项真实 CLI 或持久消费者的行为断言失败；编译/环境错误、零测试或仅占位 panic 不算有效 red。有效 red 是准备证据，不是产品 PASS。已有正确 caller 保持 green，无需全红。

M1 核原语、普通回归、有效 red、骨架可编译且没有未完成正常入口。T02 开工基准是 M1 审定骨架或最新独立测试修订的完整 SHA，allow_test_changes=false；只能删除归本任务 ignore。测试/fixture/合同缺口交复杂作者新增明确修复任务，独立复核后固定新基准，不隐式继承旧批准。正式 feature 必须非零实际执行、全部通过且无本任务 ignore。

M2 核完整用户链与实际使用。对 M1 已审且闭包未变化的内容引用原 closure 和 run ID，不重复全套；新增调用/配置/fixture/效果或异常分支按影响补验。每项任务仍有短语义复核，原输出只保存一次。T03 是手册和实际使用，不以无 Rust 用例的 task.sh 验收。

## C006-T00 采用入口

开工完整候选 `8d27348d995a434ae3dbf8255e1110973dec1381`。仅归档C005/采用C006/当前索引与链接、采用及授权例外记录；无Rust/Cargo/fixture/合同变更。docs142、specs8/1active、tests774/204cards、diff exit0。[原文保真清单](evidence/c005-raw-preservation.json)证明C005全部51个evidence逐字节保留。真实复制需求/成本与平台实测未执行，不因开发采用记为PASS。

## T01 原语与独立修复（待正式骨架）

当前全部37项原语实际green，run `ca409eaa-f821-474e-a5a4-5f00f07c9fca`；Source/Target/Report/目录前检原件保存。macOS原生目标发布、权限、整目录NOREPLACE、sync真实probe与17 Target用例通过，不当断电或跨设备保证。Source首次14run中的LEAK unknown，以及typed新测试诊断过约束的FAIL+LEAK原件保留，后续green不覆盖它们。

独立元数据反例原本接受非法版本、0Occurrence、混终点来源及succeeded多reason字段。shared model完整当前版本规则、正号/同binder、typed strict status presence/duplicate及portableleaf已修，真实Source::result caller8green。独立copy曾在反斜线leaf时创建staging后拒绝；元数据前检现在同Target，旧red不删除。目录授权先于Source启动的旧order red/新control+6拒绝例已实跑，library4green。

初次future19run2PASS/17FAIL，均是公共尚未开放路径；其中opaque-key新fixture错误用了受ID限制的outputs名、后又漏resource根路径，两个前提失败属于作者diagnostic，不是有效feature red。修为实际合法的terminal输入槽+资源文件后，单case已走到raw参数未知的真实red；后续完整future矩阵原件另保存，不改变key或产品规则。

初次完整工程run `10dba84f-91ac-4ebc-8c46-be97fbdc1de7` 为811/811 PASS（2slow、1LEAK unknown attempt_begin_returns_brief_path_that_exists），19phaseignore；fmt/check/clippy/cachedDeny/docs/spec/tests通过。运行期间仅修未启用的future key fixture和共享测试helper的错误诊断文本，Rust生产/源原语/Target/Report没有变化；原run不声称最新whole闭包一样，最终骨架将以修后输入完成必要验证和正式review。

本机cargo-dist未安装，原dist plan exit101无执行，stderr保留；Cargo metadata实际证明publish=[]、metadata.dist.dist=false、无runtime依赖/无workspace packages override。与[一手cargo-dist配置说明](https://github.com/axodotdev/cargo-dist/blob/main/CHANGELOG.md)一致，配置证明不是实际asset plan，后者按授权延期不记PASS。原nextest0.9.145仍exit92/not_run，0.9.140override是实际执行器。

## T01 正式冻结

开工完整基准 `78e6af69a5454e27325e4183d1167c2c3769c824`。最终[完整工程门禁](evidence/t01/gates-final-input.txt) exit0，run `8f9707f6-dd07-4fd1-a0e4-fb0b08cf5470`：811/811 PASS、2slow、19阶段ignore、本run无LEAK。fmt/check/clippy、缓存deny、docs145/specs8/1active/tests830/260cards通过。[Rust1.85 locked全targets/features检查](evidence/t01/msrv-check.txt) exit0。37原语run `ca409eaa-f821-474e-a5a4-5f00f07c9fca` 全部PASS。

最终[有效future-red](evidence/t01/future-red-corrected-fixture.txt) run `5e293231-4664-4685-9b42-4803ceb3f7f6`，19项实际执行：2PASS/17FAIL，exit100；所有前提fixture合法，失败来自未开放公共参数。之前两个错误fixture诊断不算有效red；不删除或替换原文。

[冻结consumer闭包](evidence/t01/input-closure-final.txt) `6712a54ead3b8aa1245227e17cc7beb421638729ec6bbcf4a660910e71751aae`，295files，域sheltie-consumer-input/v1；排除evidence/progress/review/validation及生成诊断snap.new。冻结后仅改阶段状态/验证文字，另检查治理，不把它们宣称成相同whole hash。正常raw/export参数尚未公开；T02只开放两份binary参数和分发，库内高风险政策已全部交付。

macOS arm64原生API/37原语通过；跨设备载体、其他OS、真实用途/用户/成本、cargo-dist实际asset plan、nextest要求版本0.9.145及在线fresh advisory仍授权延期。cached deny使用未改policy的独立db-path配置和RustSec `117edb3bed98e9be112f277b7615eea3252e7c43` 缓存，不能称最新在线审查。此前LEAK、FAIL及中间闭包保留各自因果边界。

T01全cached diff检查exit2，仅[清单](evidence/t01/diff-check.json)内不可变工具原文尾空格/空行；保留原字节。精确排除这11份已知raw后的作者文件检查exit0，未扩大忽略范围。

## C006-M1 冻结交接

独立阶段PASS，candidate `6e60faacbe039cd21c04e5aa4d514ea5765e2ba3`。精确[输入manifest](evidence/m1/t01-input-manifest.json)已由Reviewer从git candidate独立复算，代码与oracle没有修改。M1新增的是记录/白名单与状态，单独治理检查；T02冻结代码/测试完整基准为该T01SHA，允许范围仍仅main/CLI和删除19ignore。

## C006-T04 checkpoint握手修复

T02 first feature run `48ed88a3-1e54-48c7-bc8d-9634a43b8c65` 19/19PASS，但whole `4954967d-d5fb-4e4e-b50c-8018086598a1` 829PASS/1FAIL/0skip，exit100；后续deny/docs未执行。失败原件保留，不记全门禁PASS。reached先create_new再write_all，reader见名字即断言全文，实际读到空内容；无证据指向源/目标业务错误。

自有草稿stash `ef7d9bdfe49697a1aa858935c4ac0040ab28b117` 及[六文件字节](evidence/t04/draft-preservation-before.json)保留。修复仅测试握手消费，不动fault producer或原成果/kill/权限/重跑断言。真实空/部分标记和活直接child的确定性red `a896fcdd-4079-4de2-afb1-18c9a1737f4b` 1FAIL；完整bytes才返回，合法prefix等待、错误点拒绝，原12秒/退出检查/RAII保持。green `c8ef4066-23bc-4983-a62a-532f3924df04` 2/2，clippy0。

Cargo nextest会在开始执行前重建同executable路径，pre-build SHA不能证明之后执行的字节；helper仅核路径存在。[post采样](evidence/t04/post-failed-gates-binaries.json)只证明采样时身份。T02恢复后另以Cargo JSON真实build复制到独占私有binary路径，前后SHA相同的19真实公共consumer验证解决该证据缺口；不改冻结命令或helper，不以旧prehash代新实际身份。

T04独审发现初次positive在wait返回后writer.join，会放过提前返回变体；原初次2green保留但不是最终oracle充分性证据。修订改为真实fs::read后的观察回调：first read是固定空/部分字节，再写完整真实文件；返回时必须观察[initial,full]，没有调度/等待时长假设。正常10caller同一body的noop callback，生产代码未改。具体prefixassert后return变体run `d9683d94-91e6-4f03-b68f-3f623a88a06c` 实际1FAIL；最终 `6a65908d-fe53-46ed-a53d-a9465fd1dc49` 2/2PASS、clippy0。错误点必须特定通知拒绝，timeout或无关panic不能通过。独立Reviewer逐字节核原command/helper外的10场景和全部业务断言未变，T04短审PASS；仅握手/冻结修订，不当832全回归PASS。

## C006-T02 完整公开链

新冻结基准 `3a718cd9cec09404cec1a9e026edb9cc42cd449f`。恢复自有草稿后五文件与原稿SHA相同，crash.rs相对新基准仅删10ignore；全部四tests仅删19ignore，原expect/helper/fixture不变，[恢复证明](evidence/t02/draft-restore.json)。两份保留stash分别为 `ef7d9bdfe49697a1aa858935c4ac0040ab28b117`、`e7e9f4d18a8744f962d3a1eef9a6a6c5e2d4d423`。T04首次post范围检查误在T02已恢复上下文运行而拒绝T02越界；隔离同一草稿后同基准实际PASS，再恢复，未豁免scope。

[修后完整门禁](evidence/t02/gates-repaired.txt)实际exit0，run `4776c786-9f94-452d-8ae5-8ca380f50038`：832/832 PASS、0skip、0LEAK、2slow。fmt/check/clippy、缓存deny四类policy、docs145/specs8/1active/tests832/262cards全部通过；原失败whole仍保留。nextest实际0.9.140显式override，要求0.9.145继续not_run；缓存不是fresh审查。

为解决nextest重建同路径executable的身份记录缺口，[真实Cargo JSON构建](evidence/t02/capsule-build.jsonl)的两binary逐字节复制到独占私有路径、前后SHA完全相同；[19真实公开consumer](evidence/t02/feature-frozen-binaries.txt) run `4febada7-365c-4cf2-8014-78905900ccfa` 19/19 PASS，exit0。[capsule记录](evidence/t02/binary-capsule.json)证明该组执行身份，live pre-build/post SHA只表示各采样，不当历史执行锁定。未改commands/helper、政策或oracle。

当前whole consumer `f01613c4a052d7b2cd89b9870674b979c69c576696970548e4a0ea2be1ad55d5`，295files，同域/算法/排除，见[input记录](evidence/t02/input-closure-gate.txt)。它是修订后的新输入；之后只改plan状态与验证记录，治理单独检查，不复用旧whole hash。业务状态、整份字节/manifest、不覆盖、残留/重跑、kill/sync/链接和opaque keys均已实际经过公开链。真实副本需求/人工成本、跨设备、其他OS与dist实际asset plan保持延期。

T02首次staged范围检查拒绝main：任务表files数组左侧多一个空格，现check-task简单字段解析保留该前导空格，没识别main的files/test_files重叠。仅规范任务表空格以兑现原已审main实现范围，未改checker、保护范围、测试或代码；治理和同基准scope重验。全cached diff原文尾空格例外见[evidence清单](evidence/t02/diff-check.json)，精确排除单份immutable gates raw后的作者检查PASS。

## C006-T05 RC生命周期治理

开工 `5648958d4c6a8ed690a6ea074b59f9bc221ef972`。先建8个独立oracle，旧checker全30用例实际22旧PASS/8新FAIL、exit101；两个合法RC场景因没有数值active目标被拒，authority缺失/重复/非法/前导零被错误放行。所有早期red原件保存。修复后[同30真实governance消费者](evidence/t05/governance-final.txt)30/30PASS，旧22断言未变。

唯一首屏开发目标来自specs/README；数值active必须一致（released分支前也核），明确非产品实验/无active仍核同一权威目标。未知active目标拒绝；原tag/历史/CHANGELOG检查保持。CLI全targets/features check/clippy、fmt、docs146/specs8/1active/tests840以及作者diff全部exit0。最终[Rust1.85 locked全workspace检查](evidence/t05/msrv-final.txt)exit0。

引擎/导出source、Cargo、fixtures、原冻结public tests未变；832原run的其他消费者按相同source/config/features引用，改变的governance helper/script与其30消费者已换新证据，不称same whole hash或840单run。D-043与工程规范先固定版本权威。开发目标不会发布或创建tag。

T05全cached diff exit2仅两份工具原文EOF空行，见[evidence清单](evidence/t05/diff-check.json)；原字节保留。精确排除这两份raw后的作者检查exit0。

## C006-T03 手册与用途边界

开工候选 `b37938930487beb70ae1e0aacfbd18646e4baf88`。指南补真实Cargo JSON路径、构建失败立即停止/两binary齐全、新raw私有暂存文件与退出码、四状态和现场处置。字面默认features/locked构建片段实际exit0，[构建命令](evidence/t03/default-build-command.sh)、[Cargo JSON](evidence/t03/default-build.jsonl)、[编译/两SHA输出](evidence/t03/default-build.txt)保留。先前allfeatures capsule run单列，不冒充默认构建。

[默认binary实际手册机制](evidence/t03/default-mechanism.json)14条外层CLI均exit0：三选定报告的手工副本与工具副本bytes/size/SHA完全相同，manifest.result同一次结果，三次business status DTO相同；模拟编辑旧副本后重跑生成新副本且原编辑保留。都是明确fixture文字/临时目录，不是真实代码质量、首次用户试用或净收益。独立Reviewer已核前一capsule机制同样14calls及实际文件，默认新原件交M2。

[真实用途前检](evidence/t03/preflight.json)缺用户/目的/同质量对照及人工投入，原真实义务授权延期，minutes/usage为null而非0。未进行正式安装或发布。CHANGELOG仅未发布段改schema4/v4并补已实现替换/raw/export，历史release不改。无所属Rust用例，不运行零测试task.sh；源码、832/30消费者及MSRV输入按未变分组引用，指南/治理另验。

## 最终限定实现验收表

最终T06修订后以新的完整842单run、固定新双binary的21公共consumer、默认构建/14条外层CLI及Rust1.85检查替换改变组证据；原832/19/30记录保留历史范围。只改记录的后续M2单独治理，不宣称相同whole hash；所有原失败/未执行项保留。

| Requirement / risk | Mode | Input closure | Command / raw run ID | Result | Evidence |
| --- | --- | --- | --- | --- | --- |
| 同快照资格/revision/key及受限FD原字节读取 | reused | 最终source/全部原语/fixture/features，37原语含842回归 | 36297026-2963-426c-8a6b-d6ebde77078e | PASS | [完整工程](evidence/t06/gates-final.txt) |
| macOS目录句柄、私有对象、NOREPLACE/权限/链接/限额/读回 | reused | Target17原语及原native probe，T06两个精确窗口含最终842 | 同842；ca409eaa-f821-474e-a5a4-5f00f07c9fca | PASS | [native](evidence/t01/target-platform.txt)、[原语](evidence/t01/primitives-final.txt) |
| raw/整份成果/manifest/Store业务不变/重跑保留编辑 | executed | 实际Cargo JSON构建的私有双binary，SHA前后相同，原19冻结oracle+2新增真实窗口 | bddfa327-3e62-4c1d-afa7-c599c94e7515 | PASS | [21consumer](evidence/t06/feature-frozen-binaries.txt)、[binary身份](evidence/t06/binary-capsule.json) |
| 实际kill/sync窗口与发布后未确认、残留保真 | executed | 同21新binary/fixture；T04完整通知、T06前读回/partialmanifest独立预期 | 同21与842；6a65908d-fe53-46ed-a53d-a9465fd1dc49 | PASS | [完整链](evidence/t06/feature-frozen-binaries.txt)、[握手](evidence/t04/marker-ready-final-green.txt) |
| 引擎/导出Rust工程与普通消费者 | reused | 最终source/Cargo/全部features/840原tests+2新tests与相同环境 | 36297026-2963-426c-8a6b-d6ebde77078e，842/842 | PASS | [fmt/check/clippy/842](evidence/t06/gates-final.txt) |
| RC开发权威、release/tag/历史/CHANGELOG门禁 | executed | T05新script/fixture+原22断言不变，8新oracle | cargo test release_governance，30/30 | PASS | [30原文](evidence/t05/governance-final.txt)、[旧8red](evidence/t05/old-governance-red.txt) |
| Rust1.85最终候选编译 | executed | 最终T06 source/tests/Cargo/config/features；locked | cargo +1.85.0 check --locked --all-targets --all-features | PASS | [MSRV](evidence/t06/msrv-final.txt) |
| 缓存依赖风险/许可/来源 | reused | Cargo graph/policy/features/缓存117edb3未变；不是在线fresh | 同T06cached deny | PASS | [deny](evidence/t06/gates-final.txt) |
| 默认构建与准确首次读者指南、临时复制机制 | executed | 字面default/locked Cargo JSON；真实双binary SHA；14CLI/实际3文件 | default build及mechanism命令exit0 | PASS | [构建](evidence/t06/default-build.txt)、[实际机制](evidence/t06/default-mechanism.json)、[指南](../../../guides/result-export.md) |
| 原始失败、环境/真实价值延期与可执行补验入口 | executed | 原文保真及当前preflight，无伪造用户/成本事实 | 独立M2+当前docs/spec/tests/scope | PASS | [前提](evidence/t03/preflight.json)、[runbook](experiments/runbook.md) |

## 原义务的授权延期

| 义务 | 结果 | 具体缺项与补验 |
| --- | --- | --- |
| 真正用户、可编辑副本用途、首次使用及同质量人工对照收益 | not_run | 目的/使用者/质量标准/成本缺失；按runbook补，usage与分钟null |
| 其他OS及跨设备实机载体 | not_run | 当前只实测macOS arm64同设备；提供目标载体再核 |
| 实际cargo-dist asset plan | not_run | 本机无dist命令exit101；静态publish=false/dist=false不当实际计划 |
| 配置要求nextest0.9.145 | not_run | 已装0.9.140；override实际run单列，原exit92零执行 |
| 在线fresh advisory | not_run | 使用117edb3缓存；fresh获取需后续环境补全 |

历史LEAK unknown分别保留；历史832/19无LEAK与当前842的4LEAK各自保留，不证明旧run已定位或全宿主零残留。complete只承诺规定OS同步，不是物理断电保证或恶意同权限隔离。未执行安装/发布/push/merge。

T06精确补验：原19点包括AfterReadback/AfterManifestWrite，未直接覆盖全体发布前复核开始前/部分manifest写入；不当相邻等价。旧binary真实2caller red留在evidence/t06，T06补齐后另用新binary闭包验，不复用旧19作为新Target source的同闭包。

## C006-T06 精确窗口收口

两个实际旧binary caller red `55172769-3110-4529-9406-44987e4d4e32` 2FAIL、exit100：合法Work已完成但没有新checkpoint。新Target在publish全体最终readback开始前暂停；manifest真实前半bytes写后暂停，复核父/stage/held leaf身份/0600再写余半。旧point/原10场景/T04helper/common完全未改，新的2oracle核partial/no-manifest、真实SIGKILL、3实际Artifact/业务不变及重跑场景保真，独立短审PASS。

新固定Cargo JSON binary [21公共consumer](evidence/t06/feature-frozen-binaries.txt) run `bddfa327-3e62-4c1d-afa7-c599c94e7515` 21/21 PASS，前后SHA相同；[完整门禁](evidence/t06/gates-final.txt) run `36297026-2963-426c-8a6b-d6ebde77078e` 842/842 PASS、0skip、2slow、4LEAK、exit0，fmt/check/clippy/缓存deny/docs146/specs8/1active/tests842/272cards全部通过。4LEAK分别为delete_refuses_root_replaced_before_unlink、两duplicate_flow_id场景、replacement_refuses_modified_frozen_input_without_revoking_the_running_attempt；cause unknown。它们是旧CLI消费者，未定位原因，不能以新21无LEAK洗掉，也不调容忍或重复到绿。

最终[Rust1.85](evidence/t06/msrv-final.txt)全targets/features locked check0；新source的[默认构建](evidence/t06/default-build.txt)与[14条外层CLI机制](evidence/t06/default-mechanism.json)均exit0，三实际报告的手工/tool副本字节/size/SHA/manifest/业务status和模拟编辑后重跑保留一致。Input closure见[evidence](evidence/t06/input-closure-gate.txt)，状态记录在run后另改，源码/所有oracle保持冻结。实际价值和环境延期不变；T06是覆盖修复，不是新收益或发布。

T06全cached diff exit2仅两份immutable工具原文尾空格/EOF空行，见[evidence清单](evidence/t06/diff-check.json)；精确排除后作者检查exit0，原字节保留。

## C006-M2 正式限定验收

独立Reviewer审定 `c62f14d7e28a2f63394a60d9b702cc03e85de6f1`，实现/指南/前提及授权延期交接范围PASS，无剩余生产或oracle必改。从git candidate独立复算296file gate闭包bf8ed08b…精确一致，只在内存还原T06 done→doing；独立核当前默认14现场的三source/manual/newcopy bytes/size/SHA/manifest、模拟编辑保留与binarySHA。复用原独审及全部实际原run，不重复长验证。4旧CLI LEAK cause unknown和全部not_run保持，未部署/安装/发布。
