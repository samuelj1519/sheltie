# C004 实施计划

状态：`draft`；package 为 proposed，全部任务 todo。采用后本表才是实施进度权威。本计划先评估摩擦，再实施工具事实，最后完成结果与真实使用；M1 未通过时停止产品代码阶段。

## 1. 首次开工

按[分阶段搭建与实现](../../../guides/proposal-implementation.md)完成阅读、骨架、填空、停止、提交和审查。先看 README/spec/design，再读自己的卡；原生工具与结果接口的唯一合同是 design。初级实现者不负责选择持久格式、系统调用、程序控制或信任政策。

实际导航：`crates/sheltie-core/src/flow/{def,parse,compile,graph}.rs`；`src/work/{command,decide,state,next,layout,render}.rs`；runtime `service.rs::WorkService`、`store/commit.rs::CommitInput/Store::commit`、`request.rs::RequestIntent`、`snapshot.rs::PersistedResponse`、`session.rs::WriteSession`、`fsx.rs::SafeFile`；CLI `commands/{work,attempt}.rs`、`cli.rs/output.rs/error_map.rs`。`sheltie-verify`、`runtime/observation.rs`、`core/work/result.rs` 和相应 tests 是拟新增路径。

计划按完整行为划分，不按 crate 逐层交付。T01 写总体设计与核心 oracle，T03/T06 写当阶段骨架和测试；每个阶段结束有独立强模型 review。骨架将拟新增测试写成真实用例并归属实现任务，未搭阶段用「验收用例」标识，不触发 check-tests 的不存在测试检查。

## 2. 任务表

| ID | 状态 | Owner | 依赖 | 可观察结果 |
| --- | --- | --- | --- | --- |
| C004-T01 | todo | 强模型 | 人采用实验范围 | 总体接口、3任务探针、等价基线、独立评分与实验脚本准备 |
| C004-T02 | todo | 简单模型操作；人处理实际授权 | T01 | 当前引擎上的3任务摩擦与续接探针，真实结果而非门控证明 |
| C004-M1 | todo | 独立强模型 | T02 | 净收益方向、主要根因和最小架构审查，决定是否值得实施 |
| C004-T03 | todo | 强模型 | M1通过且采用范围含产品实施 | 上游合同、工具/观察完整接口、阶段骨架和风险原语 |
| C004-T04 | todo | 简单模型 | T03 | 固定commit、命令与实际工具报告完整运行 |
| C004-T05 | todo | 简单模型 | T04 | 原生入口、短事务、正式工具输出与故障停止走真实CLI |
| C004-M2 | todo | 独立强模型 | T05 | 工具→Store→历史响应→输入绑定的整链与必要故障窗口 |
| C004-T06 | todo | 强模型 | M2 | 结果/交接/可信字节读取及固定Workbook的阶段骨架和oracle |
| C004-T07 | todo | 简单模型 | T06 | 终点结果声明、只读handoff/result、可信原件读取接通 |
| C004-T08 | todo | 简单模型 | T07 | 固定代码Workbook与skill走完实现、返工、审查和结果 |
| C004-T09 | todo | 简单模型操作；人处理实际授权 | T08 | 配对实验、真实宿主回归和候选交接 |
| C004-M3 | todo | 独立强模型 | T09 | 完整产品闭环、质量/成本、已知限制与发布准备结论 |

M1 的用户价值证据不采用未实现的接收机制作真值。采用范围仅包含实验时，M1 后由人决定产品实施；已经明确授权全部条件计划时，M1 通过后按计划继续，无须额外重复审批。Owner 在采用时写实际负责人姓名，阶段表中的模型类型是所需能力。

## 3. 任务卡

### C004-T01 总体设计与探针骨架

**Owner/依赖。** 强模型；依赖实验采用。输入是本package和C002当前合同；输出是总体接口草图、真实任务登记、基线与可直接执行的探针材料。

**文件与入口。** 本 active package `experiments/`、spec/design/validation/progress；现有 `workbooks/spec-dev`、`examples` 和 skill 只读。用当前 `work start/attempt begin/submit/fail/status/stats`，不新增产品入口。

**步骤。** 登记3个真实任务的仓库、完整基线、目标、验收命令、标准和授权；准备同等原生流程与固定阶段Workbook说明。只在实验目录写外部快照脚本，按 design 的快照、argv、限额、日志和来源规则实现必要原语；报告明确标外部实验记录。写独立手算候选/退出码期望及人工时间记录模板，先跑最小离线准备例。

**正反例。** 一个既存commit检查成功；固定候选后移动源HEAD仍记录旧commit。不存在commit、改变冻结脚本或未知结果不得伪装为成功。

**验收用例。** 临时仓库独立 `git rev-parse` 与文件字节对照；脚本检查实际退出码，不用脚本自身的摘要函数计算期望。不创建虚构Rust测试。

**停止条件。** 真实任务缺工具/授权、外部快照无法闭合或对照条件不等价。收窄样本，不让产品先替实验搭环境。

**验证/交接。** 本任务脚本语法与实际正反例、docs/specs/diff检查；task.sh不适用。保存任务登记与原文命令，T02拿路径即可执行。提交 `docs(specs): 准备 C004 总体接口和摩擦探针`；现行提交门禁仍按工程规范执行。

### C004-T02 三任务摩擦探针

**Owner/依赖。** 简单模型负责既定命令与记录；人只负责实际宿主权限和评分输入；依赖T01。输入是冻结任务和脚本，输出是逐任务原始运行、人工分钟与问题归因。

**文件与入口。** 本 active package `experiments/`、validation/progress。任务实验使用独立临时SHELTIE_HOME及授权仓库，源产品源码不改。

**步骤。** 按validation完成bug、小功能和至少一次返工；停止并重开会话，从当前status/brief/产物继续原Attempt。记录每次人工介入、重复解释、重做、命令未执行与环境故障；自然事件与注入分列。若无摩擦也如实报告，不为方案制造失败。

**正反例。** 重开后从冻结材料完成同一Work；错误候选或缺证据被独立评分指出，不能靠Work succeeded作为真值。

**验收用例。** 每任务的外部diff审阅、真实命令与日志；当前已有机制场景不要求先红。task.sh不适用。

**停止条件。** 标准变化、权限缺失、样本明显偏向某组，或外部脚本被用作正式来源。记录缺失，不补写未执行结果。

**验证/交接。** docs/specs/diff及3条任务原文索引；提交 `docs(specs): 记录 C004 摩擦与续接探针`。handoff含输入闭包、事实与未解决问题，M1独立判方向。

### C004-T03 工具事实的合同与阶段骨架

**Owner/依赖。** 强模型；依赖M1和实际产品实施授权。输出是采用后的精确合同、可编译接口、风险原语和T04/T05测试。

**文件与入口。** 上游spec/architecture/engineering/contracts/CONTEXT；拟 `crates/sheltie-verify/{Cargo.toml,src,tests}`、`core/work/observation.rs`、`runtime/observation.rs`、CLI `commands/verify.rs`；现有command/state/decide/next/layout、service/Store/request/snapshot/effects与failpoint。按tasks精确范围开工。

**步骤。** 固定一个持久版本、CLI版本、DTO和错误码；写定producer=engine、open/completed/interrupted、两个内部短请求与public run不支持request-id。核安全Rust Git/进程组/目录句柄API在macOSaarch64与MSRV可行；完全实现子进程控制、句柄观察、准备与发布原语，不留初级猜系统调用。创建阶段接口、占位、独立夹具和真实CLI故障注入点；compiler限制每Node最多一个必需engine输出，profile_input必须是本节点required输入，日志只附属于该报告；按design写CollectOutput next及唯一candidate待填参数的完整协议；T04/T05新增测试归对应任务。未完成入口不对用户激活。

**正反例。** 两个内部响应不可互相覆盖；未完成批次不能第二次启动。既有Work/Workbook请求重放仍返回原字节；旧格式准确拒绝且不写。

**验收用例。** 预写T04/T05下列用例，阶段完成时更新为真实测试名。独立oracle取手写状态、Git命令、OS退出码、Store提交前后快照。

**停止条件。** API不支持进程组停止或安全文件合同、长运行仍持HomeLock、新接口要求业务判断进入core/runtime。修改设计并独立复核，不写降级假保证。

**验证/交接。** 工程四门禁、deny、docs/specs/skill/tests/core词汇；本阶段红例与既有回归。task.sh不为骨架虚构所属测试。记录骨架完整commit与各实现卡实际symbols。提交 `chore(workspace): 搭建 C004 原生检查阶段骨架`。

### C004-T04 固定候选和实际命令

**Owner/依赖。** 简单模型；依赖T03。输入是冻结profile、完整commit和受管scratch/日志句柄；输出是工具实际观察报告，不能写Store。

**文件与入口。** 拟 `sheltie-verify/src/{profile,snapshot,run,report}.rs`；只填T03已写定的私有接口。使用骨架已实现的进程/文件原语，不写新的系统调用。

**步骤。** 严格解析profile，固定候选并复制不共享对象的快照；按规则核受保护字节和树类型；执行argv/env与有限超时；生成逐项实际结果、未运行项与环境限制。记录报告再清理自有scratch；清理失败不删除结果。

**正反例。** 完整命令零与非零均准确记录；移动源HEAD不改变候选。不存在对象、类型不支持、保护文件改动、原仓库/外部脚本参数、超时、日志超限均不输出全通过。

**验收用例。** `fixed_commit_survives_source_head_change`、`unsupported_tree_is_not_checked`、`protected_file_change_prevents_command_start`、`source_repository_script_is_not_executed`、`nonzero_exit_does_not_claim_candidate_root_cause`、`timeout_and_log_limit_never_complete_successfully`（拟新增，归T04；不能复用被测摘要作期望）。

**停止条件。** 必须复制未跟踪文件、需要安装工具或业务根因解析，回T03收窄或改合同。

**验证/交接。** T04定向测试、受影响caller和工程门禁；check-task显式T03骨架commit。交接实际报告shape和原始Git/进程oracle。提交 `feat(verify): 固定候选并运行冻结本地检查`。

### C004-T05 原生入口和持久观察

**Owner/依赖。** 简单模型；依赖T04。输入实际工具接口和阶段骨架；输出从CLI到Store、正式工具输出及重开查询的完整行为。

**文件与入口。** core observation/command/state/decide/next/layout，runtime observation/service/request/snapshot/effects/load/store，CLI verify/cli/output/error_map；只组合T03风险原语。

**步骤。** 接通短登记、准备发布、释放全部锁Arc、实际运行和短完成；同一个Attempt仅一次。无记录next提供原生收集入口和candidate待填字段，open不放submit、completed发布后才放submit；普通next保持原规则。完成核当前归属而非旧全局revision；取消/fail关闭open，旧完成拒绝。engine输出只由实际报告构造，普通submit只合并封存引用。内部提交响应按历史快照恢复，恢复器不运行项目命令。

**正反例。** 长检查时另一个Work可写；同Attempt两个进程至多启动一个；当前Attempt结束后旧runner不能写新状态。开始前/后、工具退出后/完成提交前、完成提交后/发布前被杀，分别有可核状态和原字节。

**验收用例。** `duplicate_native_run_starts_at_most_one_process`、`other_work_write_during_native_check_succeeds`、`late_native_completion_is_rejected_after_fail`、`native_output_cannot_be_submitted_from_worker_file`、`node_with_two_engine_outputs_is_rejected`、`native_next_exposes_collection_then_submit_after_publication`、`recovery_publishes_recorded_bytes_without_running_command`、`native_start_and_finish_keep_distinct_historical_replies`（拟新增，归T05）。

**停止条件。** 发现原子性缺口、API不足或必须改oracle，回骨架作者。不能接受CLI自报exit或从日志猜完整执行。

**验证/交接。** T05定向与受影响CLI回归、工程门禁、必要子进程故障窗口；check-task显式骨架commit。交接稳定候选和故障对照，M2审实际锁释放与历史数据可信校验。提交 `feat(runtime): 接通原生检查记录与故障停止`。

### C004-T06 结果和固定方法的阶段骨架

**Owner/依赖。** 强模型；依赖M2。输出是T07/T08的完整接口、模板与独立验收测试。

**文件与入口。** 先同步 `specs/contracts/protocol.md` 的结果DTO/schema/编码表、`specs/contracts/workbook.md` 的终点选择、`specs/architecture.md` 与根spec/CONTEXT的相应目标；再处理core flow def/parse/compile及拟work/result.rs；runtime拟result.rs及service/fsx；CLI work/cli/output；examples/code-change、skills/sheltie与实际CLI测试。Result字段和字节入口以design §6为唯一合同。

**步骤。** 写终点result选择器与严格DTO骨架、规范字节golden、同句柄原件流、失败前缀处理oracle；搭handoff投影。准备四到五个固定阶段的代码Workbook，verify输入配置，engine输出check，终点绑定同一个check报告和交付文档。预写无需审查与普通审查/返工/门槛的完整场景，T08可在现有已完成行为上实现材料而不造红。

**正反例。** retro输入也可交付delivery；缺结果声明不猜产物。不同candidate或输入的原生报告不能拼成一个最终候选；普通worker报告不能产生原生来源。

**验收用例。** 预写T07/T08用例，归属对应实现任务；stdout二进制、digest golden与外部哨兵独立给定。

**停止条件。** 为结果选择引入第二图、自然语言解析、终态写入或CandidateAccepted状态。回设计，不绕过单一状态。

**验证/交接。** 工程及文档/skill门禁，T07红例与既有回归；记录阶段骨架commit。提交 `chore(workspace): 搭建 C004 结果与方法阶段骨架`。

### C004-T07 明确结果与可信字节

**Owner/依赖。** 简单模型；依赖T06。输入成功终点的冻结引用；输出只读handoff、结构结果和原件字节。

**文件与入口。** core flow/result/render；runtime result/service；CLI work/cli/output/error_map。原生报告解码由工具crate，runtime不理解Git。

**步骤。** 接通终点result字段拒绝规则；从明确终点Attempt取inputs/outputs；生成规范结果摘要并匹配原生记录。实现final=false、空选择、candidate=null和冲突拒绝。原件流只用同一受限句柄，失败exit不被消费成成功；handoff单列草稿和未完成观察。

**正反例。** 后续用户HEAD变化不改变交付commit；当前结果digest不匹配拒读。门槛未批准、取消、可选结果、混合候选和源字节修改均准确拒绝或标非final。

**验收用例。** `terminal_frozen_inputs_select_exact_result_version`、`unselected_native_history_does_not_change_result`、`mixed_native_subjects_reject_single_candidate`、`result_digest_matches_independent_golden_bytes`、`failed_artifact_stream_is_never_published_by_consumer`、`handoff_keeps_drafts_distinct_from_artifacts`（拟新增，归T07）。

**停止条件。** 只能查询最新某节点而不能证明具体Attempt、源流无法限制或DTO未定，回T06。

**验证/交接。** T07定向/CLI场景、工程门禁和实际原件字节oracle；check-task显式T06骨架commit。提交 `feat(work): 提供明确结果与只读接续材料`。

### C004-T08 固定代码Workbook与协调者说明

**Owner/依赖。** 简单模型；依赖T07。输入已批准模板与T06场景；输出可运行方法和自包含skill。

**文件与入口。** examples/code-change、skills/sheltie、core examples与CLI场景/skill测试；现有spec-dev只在实际受影响的结果引用与交付说明处同步。

**步骤。** 按冻结模板填写任务书，明确原生run、报告阅读、fail和back的区别。保留普通审查与最终gate；结果引用完整commit，交付节点不得把后续新HEAD冒充被验证版本。所有示例只调用真实已实现命令；新定义用新Workbook版本，不覆盖旧安装副本。

**正反例。** 失败→修复→新commit→复检→普通审查→结果；关闭重开继续原Attempt。只有报告文件而无原生记录不显示机器已验证，skip/gate/取消仍保持原语义。

**验收用例。** `fixed_code_workbook_returns_checked_commit_after_rework`、`ordinary_review_rework_needs_no_per_finding_human_approval`、`reopened_coordinator_continues_same_attempt`（拟新增，归T08；已有机制通过时不强制先红）。

**停止条件。** 固定模板需要自由图、严格身份或发现处置库才能使用，回M1范围，不现场增功能。

**验证/交接。** 真实CLI场景、examples、check-skill/skill-delivery及工程门禁；check-task显式T06骨架commit。提交 `feat(workbook): 交付固定代码变更方法`。

### C004-T09 配对实验与真实宿主交接

**Owner/依赖。** 简单模型按冻结规程操作；人提供实际权限和盲审；依赖T08。输出真实5–8任务对照、质量/成本、支持平台与限制。

**文件与入口。** 本 active package experiments/validation/progress；真实宿主和当前候选产物，源码不改。由Cargo JSON的executable定位CLI，不猜target路径。

**步骤。** 完成validation配对条件；记录首次/复用人工分钟、墙钟、未完成与实际usage，缺usage保留null。真实走至少一次返工、gate和重开；验收真值来自独立标准。冻结最终候选，汇总机制测试与用户证据。

**正反例。** 每组均用等价本地环境；Work成功而外部审阅不接受必须计入质量失败，不修改标准。未取得平台/usage证据不能记PASS。

**验收用例。** 真实宿主trace与盲审评分；task.sh不适用，不能添加模拟测试冒充真实效果。

**停止条件。** 质量下降、额外配置抵消收益、样本无完整闭包，报告并交M3决定收窄，不为了完成产品化而挑好结果。

**验证/交接。** docs/specs/diff及完整运行索引，工程提交门禁；提交 `docs(specs): 记录 C004 用户价值和真实宿主验证`。交接完整候选、限制、实际产物和发布待办；不执行push/release。

## 4. 独立里程碑

| 里程碑 | 固定范围 | 通过条件与停止 |
| --- | --- | --- |
| M1 | T01–T02、产品spec/design、等价基线 | 3任务解释了主要摩擦，冻结候选/检查/交接比原流程有实际用途；不将外部实验来源当原生证据。没有收益则重写或结束实验，T03不得开始 |
| M2 | T03–T05全部真实调用链 | 普通请求不回归、长命令不持根锁、重复启动和迟到完成拒绝、文件效果按已登记字节恢复；定向突变只覆盖新状态与归属判断及真实消费者 |
| M3 | T06–T09及整个候选 | 终点引用/工具事实/结果字节闭合，普通审查不新增逐项人工审批，质量与成本分开；完整工程门禁、必要影响链突变、实际macOSaarch64与真实宿主证据 |

Reviewer 未参与被审阶段骨架和实现。结论只用通过/需修改/阻断；方案审查、实验、机制、真实宿主和发布分开记录。补测或修复按共同指南新增明确任务，不让Reviewer自审。没有新输入变化，不重复全量和不相关长验证。

## 5. 完成

全部必需任务与M1–M3按固定闭包通过，产品合同与例子一致，所有C004占位/ignore清零，结果与缺失如实记录，才可完成采用范围。实验不支持产品化时记录其结果并由人决定rejected或收窄；不以代码量或文档审查代替收益。发布不在本表内，另由实际release任务与授权处理。
