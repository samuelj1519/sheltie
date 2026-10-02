# C002 独立审查

M1 结论：**通过**。此结论限定于用户授权的 macOS 源码与离线验收范围，包含两项实际平台暂停后的跳过；不是完整 Spec 批准、全部变异验证或安全验证通过。M1 覆盖 T01–T15、T18–T34；M1 完成时 Linux、真实 Host T16 和发布 T17 均为 `not_run`。后续实施进度只看 [plan.md](plan.md)。

本文合并最终审查；旧候选失败、Reviewer 原文及逐 ID 记录均保存在 [历史快照](validation.md#历史证据恢复)，不删除历史结论或改写原始结果。正文中原文链接统一指向该索引。

## 候选与审查职责

产品候选为 `e3eea899877165f8573befee3774555598ec92bd`，整体基准为 `a664e75a3ab3041d09cd0a1ab4d69336f2dcd055`。变异用普通 clone 的冻结提交为 `95d78e0677f1ffba6abe5ad9c13430ea53957101`，其 167 项源码、配置和 fixture 与产品候选一致；234 项治理文件保留当时输入，不用后续文档改动覆盖历史运行。[冻结输入](validation.md#历史证据恢复) SHA256 为 `6139c0648032211ef2a02bd60d4a05fd1e2898f80c7944868979f8769ea0aa13`。

Codex 负责修复、运行及最终记录。独立 Spec、Standards 和证据 Reviewer 未参与被审代码的编写。早期 Spec 发现及增量意见保留；最终 Spec 续审缺失按 SK01 记录。Standards 最终通过，证据 Reviewer 完成原文与逐 ID 核算；后两者不能代替 Spec 批准。原始复核报告及临时路径的持久副本索引见 [Reviewer 记录](validation.md#历史证据恢复)。

## 实现与工程质量

| 修复 | 提交 | 结果与真实调用验证 |
| --- | --- | --- |
| T32 精简 | `326cbbb` | 删除无消费者接口、转发与可选观察兜底；持 HomeLock 的 CAS 冲突直接传播，勘误重试合同。补 manifest/curl 精确限额和删除清理反例 |
| T33 根解析 | `06c3af3` | 路径解析只在 NotFound 时查祖先，其他 I/O 错误保留；拒绝非法环境值及不可表示的实际路径。真实 Home/CLI、权限和消失 cwd 回归；观察线程退出时释放并 join |
| T34 历史快照 | `e3eea89` | R22 核实 original/pending_original 的业务身份；R23 拒绝不可能历史状态；R24 将 requires 绑定冻结节点。复用 core 纯规则及可信装入，保留合法效果失败的原响应 |

R01–R24、O01–O13、N01–N14 的 51 行依据、提交、合法例、单条件反例、真实 caller 与独立 oracle 见 [验收矩阵](validation.md#验收矩阵)。R17/N13 的验证义务只在明示跳过范围内完成 M1 交接，215 项缺失不写成能力证明。

三个 crate 的职责与单一持久事实来源保留；没有增加第二套状态、兼容协议或推测性抽象。工程规范仅澄清私有纯身份投影与完整合同解码的区别：投影不批准完整载荷，其余字段在效果 I/O 前仍经严格解码。Standards 独立复核确认唯一调用链满足此条件，未新增未变异产品缺陷。产品价值与设计结论延续原审查：引擎守机械规则，协调者判断内容；OS 主体不提供独立真人认证。

## 正常门禁与真实窗口

[当前输入](validation.md#历史证据恢复) SHA256 为 `2390ad8b68cb86b9e33acf07b9172484d50871d92e990585f7d0af5806a2c1ce`。[门禁原文与结果](validation.md#历史证据恢复)记录 fmt、check、Clippy、nextest、MSRV 1.85 locked、docs、specs、core-vocab、tests、skill 和 dist plan 均 exit 0。Nextest run `2e126f6a-1496-4b0b-8d71-583e22d4065e`：699/699，通过、零跳过；1 项限额测试 slow。离线 deny exit 0，仅使用本地缓存公告，保留许可与重复依赖警告，不声称公告实时更新。

[当前崩溃窗口](validation.md#历史证据恢复)另核实 24/24 父测试、58 次真实终止（32 次 exit70、26 次 SIGKILL）。Start/Add 的 20 次显式序列化 oracle 单列，其余 38 次以父测试及源码断言核实；不扩大为全部独立序列化 oracle 或断电证明。

## 精简后的分组验证

保留完整首轮与已完成 workspace 运行，按真实 consumer 将未执行的 CLI 验证分组；已检测到差异的项不重复跑全部 699 项。共享直接 oracle 和受控观察副本各自绑定实际源码、控制例、单项 diff、产物、命令与输出。中断及取消的两批 workspace 原文只作 WIP，不进入完成计数。

[2514 项最终账本](validation.md#历史证据恢复)集合与完整 inventory 精确相等、无重无漏；[独立核算](validation.md#历史证据恢复)通过的是证据核算。

| 当前处分 | 项数 | 结论范围 |
| --- | ---: | --- |
| 首轮正式 CaughtMutant | 1812 | 原始构建及实际失败日志 |
| 编译器确认 Unviable | 328 | 原始 Rust 诊断，不是测试捕获 |
| 限定静态处分 | 74 | 精确当前 consumer/producer、平台或合同证明；不是动态 PASS |
| 完整 workspace 捕获 | 33 | 仅已完整结束阶段 |
| 能力组 CLI 捕获 | 30 | 181 项执行中 30 caught、151 missed；独立于原首轮标签 |
| 补充检测 | 22 | 16 项直接、6 项受控观察；不改写正式 MissedMutant |
| SK02 任务受影响的额外执行缺失 | 215 | 每项仍缺新 oracle，不是 caught、equivalent 或 PASS |
| 合计 | 2514 | 全部有记录，完整验证仍未通过 |

74 项静态处分保留各自限制。特别是输入解析的一项只证明接受集合、类型与结构化定位相同，错误文本不同；4 项非 Unix backend 仅当前平台不适用，不是 Windows 资格认证。同句柄与私有 producer 证明不扩展到任意外部写者。受控 source-alias 观察仅证明提前跟随差异，两次最终 CLI 均拒绝；tree-swap 未检测记录保留，剩余条件属于缺失范围。

原暂缓 269 项已逐项恢复追踪：[映射与独立审计](validation.md#历史证据恢复)核实 261 项当前相同片段、2 项无用接口退役、6 项旧实现替换。迁移映射本身不是执行证明；T31 当时历史豁免记录不改写。

任务记录门禁默认使用C002原始基准，误将前序任务已提交的6项快照变化归入本次文档记录，默认失败原文保留。本M1记录相对最终产品提交 `e3eea899` 没有源码、测试或快照改动；独立复核后使用脚本支持的显式任务基准通过。整体代码审查仍对原始 `a664e75`，详见[最终文档与任务门禁](validation.md#历史证据恢复)。

## 实际跳过与验收限制

[实际提示清单](validation.md#历史证据恢复)保留两次提示原文、阶段、执行者与未完成义务。用户授权仅跳过实际暂停的任务并继续其余工作，未授权把缺失写成验证通过。

| 项目 | 实际暂停 | 保留的缺失 |
| --- | --- | --- |
| SK01 | T34 独立 Spec 最终续审 | 未取得最终 Spec 批准；既有发现与方向意见仅为此前证据 |
| SK02 | M1 新共享事件补充 oracle 执行 | [215 个精确 ID](validation.md#历史证据恢复)映射受影响任务；不声称每个 ID 都单独触发提示 |

两项均未重试、改写提示或转交替代执行。既有 CLI 分组、已完成原文审计及普通门禁按原授权完成。[验收范围](validation.md#历史证据恢复)将 M1 在明确例外内的完成，与 `disposition_complete=false`、`full_validation_pass=false` 区分。

Linux 原生运行、非 UTF-8 物理目录 fixture（本机创建被 EPERM 拒绝）、真实 Host 与远端发布保持各自限制。Workbook 重读反例只证明已测 request_id/replayed 条件，不外推为任意外部 SQLite 并发编辑的完整保护；当前锁合同约束协作引擎。T16 下一步仍需真实宿主验收；本次没有授予发布或推送授权。


## 历史结论与后续

2026-09-27 的 MVP 审查（a664e75）与 2026-09-28 的 M1 预审（e1a8126）均为需修改；T31 原审查的七项问题、首次失败、后续答复及 T32–T34 增量修复在固定快照中保留。阶段 PASS 只覆盖当时输入，最终范围以本文和 validation.md 为准。

真实 Host、模型交付质量、usage/成本对照与四平台发布没有由本次离线审查证明。适用场景仍是同 OS 用户下的协调者工作流与可追溯产物，不是独立真人认证、多用户合规或工具沙箱。新机制须对应已出现的失败、当前合同或可验证近期用例；不因推测新增 MCP、认证层、动态模板或并行框架。


## 执行效率复盘

2026-10-02 用户要求分析近 70 小时的执行成本。本节是执行方法诊断，不改变 M1 结论、任务状态、源码或门禁规则。MIT 与文档收敛已分别提交 `7a584a3`、`a31824b`。通用流程见 [任务执行与验证预算](../../../guides/task-execution-efficiency.md)。

### 计时口径与已确认事实

重点窗口取 T30 提交 `b926789` 到 M1 记录 `9c933d8`：2026-09-29 19:47:15 至 2026-10-02 15:50:57（UTC+8），68 小时 03 分 42 秒，与用户的近 70 小时描述接近。它是日历跨度，不是主动工时；整个 C002 从采用到 M1 的跨度更长，不能混用起点。

| 提交区间 | 日历跨度 | 阶段 |
| --- | ---: | --- |
| T30 → T31 | 43小时36分56秒 | 综合验证、发现与修复持久事实问题、候选重跑 |
| T31 → T34 | 8小时30分22秒 | T32精简、T33根解析、T34快照绑定与回归 |
| T34 → M1记录 | 15小时56分24秒 | 新冻结验证、分组/补充验证、处分和审计收尾 |

从归档 `e54dcd41d8f1e186007b62b47583063cb19a4b66` 批量读取 JSON，去除同一 raw run 的重复副本。完整变异运行的 start_time/end_time 截取到上述窗口，再合并重叠区间，得到至少 35.35 小时的运行区间覆盖；11 份缺 end_time 的原文不计入。运行区间可以与审查/修复并行，因此它是已记录验证活动的下界，不是该窗口的完整耗时归因。

| 观察 | 实测值 | 解释 |
| --- | ---: | --- |
| 保存的变异结果 | 16861次 | 包含不同候选、复验和中断结果；不是16861个唯一ID或PASS |
| 变异命令超时 | 193次，均超过590秒 | 合计32.17命令小时；并行时间不能当32.17小时墙钟或全部浪费 |
| 带计时的阶段记录 | 139份，阶段墙钟之和39.36小时 | 含直接、workspace、focused等范围；阶段可能重叠，不与68小时相加 |
| 最终输入直接阶段 | 2514项，12511.238秒 | 约3.48小时；当前完整首轮有真实验证价值 |
| 最终完整workspace复验 | 152项，7851.011秒 | 约2.18小时；17 core加135 runtime |
| 后续CLI能力分组 | 181项，1555.196秒 | 约26分钟；与workspace样本不同，不作严格A/B加速比 |
| 普通gate-results中可计时Nextest | 去重22条，合计3272.84秒 | 约54.5分钟，仅此类记录；不含全部hook/基线/独立运行 |
| 本轮T36整体测试 | 699项，168.880秒 | `add_accepts_files_at_exact_limits`单项70.461秒，真实处理256MiB树 |
| 收敛前材料 | 4253 evidence文件、约392MiB | 另有8份阶段/repair文档；证明整理成本，不能量化为70小时主因 |

变异命令的 Build/Test elapsed 之和分别约46.90/196.63命令小时。这些命令可并行、等待和争用资源，不是CPU小时，也不能与阶段墙钟或日历跨度相加。缺主动工作、等待、模型和工具分段记录，无法把剩余时间精确分配成百分比。

### 根因与可避免工作

1. **昂贵验证启动得太早。** T31的七项实现审查修复、计数约束，以及后续R21–R24都发生在批量验证推进期间，输入变更使旧结果成为历史。重跑失效结果是正确做法；可以避免的是关键caller/原响应/失败停止语义尚未核清就启动完整流水线。仅T31“七项修复前”的记录已有5955次processed、14.11阶段小时，后来多个superseded目录继续积累开销；这些数不是全部无效工作。改法：先真实反例和关键语义审查，再冻结并批量跑；变更后按依赖关闭失效范围。
2. **验证范围按工具默认值而非问题选择。** 第一阶段只测变异crate漏CLI消费者，第二阶段又把存活项整体送全workspace，导致无关场景与大文件测试反复执行。当前152项workspace复验用2.18小时，而后续181项相关CLI分组只用约26分钟，体现范围选择的重要性。改法：能力→consumer→oracle→测试组先映射；影响不明再扩大，每个ID仍完整记账。
3. **超时没有及时改变调度策略。** 193次600秒级超时中可能既有真实挂起也有拥塞，不能全当产品缺陷或环境问题。一次jobs=8的首轮workspace尝试在16项中出现15个Timeout；没有足够预检就扩大并发/延长超时，会放大资源争用和等待。改法：代表性样本先测总吞吐与资源；同类超时集中时暂停新批，分类并缩小复现，而不是继续整批跑到timeout。
4. **门禁层级与复用规则不完整。** 工程规范对每次提交强制四条全仓命令；实现者、提交钩子、最终候选又可能各跑一次。同一源码上正常门禁确有重复，但上述普通计时样本仅约54.5分钟，不是主要35小时级验证负载。此次纯文档T36仍按现规则花168.880秒跑整体测试，是可复现的开销。改法：按语义风险分层、绑定真实输入闭包，钩子只补缺或拒绝失效记录；不能只按Git SHA复用。
5. **审查和证据收尾缺少成本边界。** 多轮完整清单、分散目录、逐ID人工证明、相同输入的重复原文复核扩大上下文和交接负担。预算、停止条件和单一执行/证据索引缺失，使“再多查一层”容易成为默认动作。改法：先确认验收义务，共享oracle和证明自动归组，只人工处理异常；修复后增量审查，满足必要项就结束，保留一次可恢复原文。

此前runtime测试内启动Cargo构建CLI，导致并发重链接/产物竞争；T30已将对应用例迁移到CLI并使用Cargo注入产物。它是整个C002的历史开销和有效修复，不能继续当T30之后68小时窗口的未修主因。当前应保留正确产物隔离，不为加速共享未变异binary或错误target。

真正的身份、计数、同步、原响应和历史状态缺陷需要修复；真实CLI、关键故障窗口、独立期望和最终整体基线也有价值。需要删减的是重复执行、不相关全量、无收益证明的扩展和过早启动的失效批次，不是把这些正确性义务省掉。实际SK01/SK02缺失继续保留，不能把此次提速诊断变成完整验证PASS。

### 后续方案与实施优先级

- 先在任务卡固定交付、影响链、正反例、验证命令、估计成本与完成条件；只有有新信息收益的检查才追加。
- 稳定候选前只跑短反馈与必要审查；长任务先少量预检，按实测吞吐估算并设停止扩张条件。
- 变异按真实消费者分组，重型边界用例在相关组和稳定候选保留；逐ID核算由共享规则驱动。
- 优先修改工程规范/模板及钩子：文档与元数据不默认全测试，代码按影响选检查，整体里程碑有一次有效全量记录。同闭包结果严格复用，失效、失败与零测试必须拒绝。
- 后续真实任务记录“总墙钟/主动执行/等待/重复及理由/有效缺陷/缺失”，再衡量改善。不承诺把所有类似任务压缩到某固定小时数，也不把并行命令小时当节省墙钟。

本轮已完成历史统计和通用方案文档，未修改验证脚本、钩子或强制门禁，也没有重跑变异。涉及减少既有必需门禁的建议尚未采用；先以小范围机制落地和真实任务验证效果，不建新的调度/缓存平台。

### 原文定位与复算

所有路径均位于归档快照的C002 package内，恢复方法见 [validation.md](validation.md#历史证据恢复)：

- `evidence/repairs/t31/mutants/`，特别是 `superseded-before-implementation-review-fixes/` 与 `workspace-overload-first-attempt/`：旧候选、600秒超时、jobs及实际phase argv。
- `evidence/m1-2026-10-01/mutants/stage*/metadata.json`、`raw/outcomes.json`：最终2514首轮和152完整workspace记录。
- `evidence/m1-2026-10-01/mutants/adaptive-cli-groups/G*/metadata.json`、`raw/outcomes.json`：181项能力分组。
- `**/gate-results.json`：有seconds的普通门禁；去重相同stdout/argv/seconds副本，缺计时项不推估。

复算运行成本时对同一原始run身份去重，不按mutation ID去重，因为相同ID的重跑本身有实际成本。完整run时间区间取start_time/end_time并裁剪到窗口后合并；部分无end记录保留但不计该下界。超时成本取Mutant场景内process_status=Timeout的真实phase duration。结果核算与计时统计分开，不能用计时统计改写原处分。

## C002-T38 产品与代码分析（2026-10-02）

本次基准为 `4b86279`，分析与精简由用户明确授权。此节独立于上面的历史 M1 结论：精简后源码是新候选，历史变异结果不转记为本次 PASS。code-simplifier 阅读三个 crate 的全部生产源码，Codex 对照当前权威规格、真实调用链及每处改动复核；测试阅读覆盖直接相关调用方与全仓用例索引，未把所有无关测试逐行通读。验证结果见 [T38 记录](validation.md#c002-t38-精简验证)。

### 产品方向与方案

方向符合定位。宪章的长期价值是外置事实、合法边、不可变输入产物、门槛与上限；根规格没有把内容判断、模型调度或宿主安装放进引擎。Workbook 保留业务方法，skill 只教协调者调用 CLI，三层职责明确。C002 的 request intent、历史响应快照、目录摘要 framing、句柄核验与持久效果针对已观察到的故障，属于当前可靠性义务。

SQLite 状态、请求快照、audit 和效果登记各有职责，不是四套推进状态：只有 WorkState 决定运行事实，audit 提供提交归属，snapshot 保留历史回复，effects 证明文件动作完成。删除标记解决无法从“文件已不存在”证明本请求删除成功的问题；根锁覆盖 SQLite 事务之外的文件生命周期。不能为缩短代码把这些证明合并成路径存在性检查。

产品价值尚有独立缺口。离线调用和结构校验不能证明真实协调者的交付质量、人工投入或 token 节省；T16 才提供真实宿主证据。C004–C008 与路线图是待采用方向，不能作为当前代码缺失项或本次重构依据。

### C002 实现覆盖

| 能力 | 真实实现入口与测试入口 | 判断 |
| --- | --- | --- |
| Workbook/Flow 解析与图约束 | core `workbook/manifest`、`flow/parse/compile/graph`；core examples/API tests | 已接入严格定义边界与显式边、输入引用、输出路径冲突校验 |
| start 输入发现、preflight 与单一布局 | core `work/start/layout`、runtime `service::start`、CLI workbook show；start_preflight/work/output_paths | 已接入；新请求预检与历史请求查重有不同义务，不能删除锁内重核 |
| 推进、重试、gate、上限与事实视图 | core `decide/next/state/render`；work/attempt/scenario/stats 相关用例 | 已接入；gate 只记 OS 主体，不证明独立真人 |
| 请求身份与原响应 | runtime `request/snapshot`、Store、service/repo；schema2_replay/CLI replay/implementation_repairs | 已接入目标绑定与历史回复，未发现本次改动改变响应事实 |
| 冻结、摘要、根内 I/O | runtime `fsx/workbook_digest/observe/load`；fs_boundary/workbook_digest/identity/node_inputs | 已接入受限树、同句柄观察及封存、冻结副本核验 |
| 发布、恢复、删除与只读 pending | runtime `effects/recovery/pending/session`；write_session/effect_contracts/crash/CLI reliability_crash | 已接入持锁生命周期、整组效果校验、必要 sync 与完成证明；现有平台和窗口证据范围仍有限 |
| 协调者与业务 Workbook 交接 | skills/sheltie、article-review、spec-dev；skill_delivery/scenario_article_review/scenario_spec_dev | 离线流程已接入；真实 agent 质量属于 T16 |
| self、MSRV 与发布链 | runtime selfmgmt、CI、skill-delivery；self_cmd/remote_update/release_governance | 源码与离线门禁已接入；实际四平台发布属于 T17 |

不能得出“C002 所有规划内容均已实现并完整验收”。除 M1 已记录的 SK01/SK02、Linux/T16/T17 限制外，本次新发现重复 Flow id 可造成提交后不可恢复阻断、`tmp/` 过期维护缺少 caller，以及嵌套 AttemptId/WorkStatus 未拒绝未知字段，见 [T38 新问题](findings.md#6-t38-新发现)。过期维护为静态确认，其余有真实 CLI 取证；三项是行为修复，不纳入本次等价精简，也不回写历史 M1 结果。

### Rust 工程实践与复杂度

总体符合项目需要：单向 crate 依赖、纯 core、私有 Raw DTO 与已校验 Graph、newtype、穷尽状态枚举、Result/thiserror 错误边界、借用与 RAII 句柄/锁、流式有界读取，以及 MSRV/Clippy/真实 CLI 和故障测试都有实际落点。Error 传播、借用已有对象和收口构造面分别符合 [Rust Book 的错误传播](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html)、[方法与借用](https://doc.rust-lang.org/book/ch05-03-method-syntax.html)及 [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/checklist.html)的原则；这些资料不是本产品正确性的证明。

仍有可改进点：完整载荷严格解码须深入嵌套类型；CLI 的全局 dead_code 许可掩盖了无调用者的错误 DTO；响应组装、冻结输入核验和审计行映射重复；部分骨架期注释已过时。不能把 `WorkState` 的公开事实字段说成所有非法状态都不可表示，也不需要为此增加 typestate 或普遍包装每个 String。

架构没有出现需退役的异步框架、事件溯源、业务判定层、动态路由 DSL、宿主适配框架或第二种持久格式。fsx/service/effects/pending 文件较大，阅读成本高，但主要复杂度来自对象身份、错误归属和故障窗口；只按行数拆文件会扩大内部接口。本次按重复义务合并具体函数，不新增通用 trait、配置、状态或依赖。后续拆模块应以稳定职责和实际消费者为边界。

### 已执行的精简与复核

| 改动 | 等价依据 |
| --- | --- |
| CLI 集中 `ok_response` 与 `replayed_data`，写命令复用只读响应 envelope | data 从同一提交快照取得；只有 replayed 被覆盖；request_id/revision/next 形状、字段省略及文本保持原样，不回读 Store |
| 删除 CLI `ErrEnvelope/ErrorBody` 和全局 dead_code 许可 | 全仓无消费者；错误仍由原 err/error_map 路径生成，编译器与 Clippy重新检查剩余项 |
| core 合并 start/node 冻结输入校验 | 缺观察或摘要不符仍返回相同 ArtifactModified 字段；路径和字节归属仍由原 runtime 装入/观察链核验；resource/stats/可选输入分支不变 |
| begin 移动 Attempt 并借用最新 Attempt 渲染 | 保留 stats 回填后的渲染顺序，消除两次整份 Attempt clone，状态/brief/stats 字节不变 |
| runtime 去掉永远 Some 的 preflight Store 包装 | NotFound/其他错误原样停止；成功分支始终有 Store，不改查重、输入读取与加锁顺序 |
| 审计脱敏按三个固定字段循环，审计行使用同一 decoder | 字段次序、UTF-8 字节长度、SQL 列序、逐列错误传播不变 |
| 目录集合从 `BTreeMap<String, ()>` 改为 `BTreeSet<String>` | 相同 String 排序及去重，父目录先于子目录的输出顺序不变 |

另勘误架构的 `uzers` 选型与状态校验表述，以及协议中的 `attempt`、WorkStatus 对象和 D-039 控制文件边界。依据是已有 C002 快照 DTO、序列化类型和真实调用方，不变更协议输出或持久数据。

本次生产 diff 由未编写该 diff 的 Codex 逐处复核。分析包含上述未修行为缺口，因此不作为新的完整 Spec 批准、完整变异验证或发布验收。测试、快照、依赖、fixture 和安全文件动作不修改；最终门禁和零改动核对见 validation。本次改动留在工作区，未提交、推送、安装宿主或发布。

## C002-T39 三项缺口修复（2026-10-02）

用户明确授权修复 F38-03/F38-02/F38-01。T38 的发现与运行保留为历史；本次在其未提交工作区上修复，不自动清除或改写既有坏 Store，不将新源码套用旧 M1 门禁或变异结果。

- 重复 Flow id 的检查放入 `WorkbookRepo::load_tree` 的完整定义装入，在私有副本 parse 后、COMMIT 前按 Workbook 范围去重，返回 `FLOW_INVALID` 的规则、文件和 id 定位。恢复端快照唯一性检查保留。合法两图可以 show/start；重复 id 拒绝后业务、request、audit、sequence 行与最终目录均无新增，独立合法写及修源后的同 request 重试成功。
- `AttemptId` 与 `WorkStatus` 的 serde 边界拒绝嵌套未知字段，合法 JSON 字节和状态形状不变。回归通过真实 CLI 的 state、历史 Reply/NextOp 与 audit Command 装入；坏载荷不修写、不释放成功 original，合法历史重放保持完整原响应。
- tmp 维护使用既有根锁和文件/目录句柄，在成功 CLI 写操作输出响应后独立执行。严格超过 24 小时才清理；普通文件与目录复核同一打开对象，链接仅 unlink 自身，特殊对象或身份不明保留并告警。只读、失败调用不执行；维护失败只写 stderr，不改变响应/退出码/历史。storage §3.3 固定了时间边界、调用时机和失败语义，pending 从未使用时间规则。

新增 tmp 回归覆盖文件与目录的精确 24 小时/+1 纳秒、未来时间、recent 对象、pending 保留、只读/失败、self 写、硬链接、FIFO、根外链接哨兵和真实目录替换窗口。目录替换使用已有 `delete_before_root_unlink` 同步点；原打开目录移名后放入带哨兵的替代目录，清理必须停止，成功重放回复保持原值，哨兵保留。未新增生产故障注入点、框架或持久状态。

fix_flow_ids 和 fix_nested_decode 分别实现前两项，Codex实现 tmp 整链；未参与三项编写的 review_repairs 负责独立规格、错误及文件边界审查。code-simplifier 只读复核认为 tmp 三种对象分支各有必要义务，无需再抽象。最终审查、红绿原文与普通门禁见 [T39 验证](validation.md#c002-t39-修复验证)。本次不重新作完整 M1/变异/Host/发布验收，改动仍留工作区。


## C002-T40 测试精简审查（2026-10-02）

用户明确要求彻底精简上一轮审计相关部分。保留T38/T39未提交工作区，以其实际文件字节为起点；本次只改测试及测试专用支持，默认生产行为、依赖、配置、业务Workbook和七份快照保持。core、runtime、CLI分别实施，未参与编写的`review_repairs`逐项审查结论均为“通过”。普通完整候选门禁另见 [T40验证](validation.md#c002-t40-测试精简验证)。

### 实际结果与覆盖

37个Rust文件净减少896行（含注释、空行和支持代码）：core 9文件292行，runtime 13文件433行，CLI 15文件171行。Nextest测试入口从717收敛到688；30个入口退休/合并，1个入口改名，另增1个独立schema2合法控制，因此31个旧名字消失、2个新名字出现。减少入口不代表减少产品条件：独有case/oracle先迁移，多状态、多输入、exact/+1和错误优先序仍逐项执行。

- core：合并engine.stats、blocked_count、终点及限额两端；删除被严格包含的smoke/日期前缀、只复演SHA wrapper的旧向量和重复manifest样例。实际runtime五向量、prefix/BE64/frame独立字节、七个快照、五个compile_fail、状态矛盾/overflow/日历/proptest/public legal_next均保留。复用已有Fixture构造，不引入I/O；testkit接口仅为真实测试caller公开，删除无consumer支持函数。新增resource拒绝和精确错误/roundtrip判据单独归T40。
- runtime：3条严格重复/旧tmp布局退休，9条先迁移独有oracle再合并；原新生命周期分别核相同与不同bytes，0/1历史终态查询仍两cases，多个错误字段仍单条件反例。共享literal/start、SQL行、哨兵、限额树、tar动作及11处现有同步生命周期，未改变write_session的特殊释放顺序。
- CLI：6条重复或同入口测试先移首轮@file字节、@summary正文、最新Occurrence路径及ghost身份判据再合并。共享原始SqlValue快照、Process的env清理/精确同步/timeout/Drop释放kill等待、目录复制和5处真实T01写入/门禁/提交动作；冷worker expected仍来自绑定文件、原字节和Git证据。不同目录软链复制策略的skill helper保留私有实现，未增模式框架。所有T39独立边界保留。

表中是全部消失的原测试名字；逐一保留入口、条件、原/现函数字节、oracle和故障窗口的完整机器映射为`/private/tmp/sheltie-t40-evidence/coverage-map.json`及三份scope dispositions。其他原名字逐项核对为保留或支持重构，717条原身份无遗漏。

| 原测试 | 当前保留能力入口 | 处分 |
| --- | --- | --- |
| `attempt_submit_summary_from_at_file` | `submit_request_replay_does_not_reread_a_deleted_summary_file` | merged_unique_oracle_first |
| `downstream_binds_latest_succeeded_occurrence_output` | `review_back_edge_creates_second_draft_occurrence` | merged_unique_oracle_first |
| `work_start_missing_input_exits_1_with_input_missing` | `start_deterministic_rejections_leave_home_unchanged_and_do_not_burn_seq` | retired_strict_subset |
| `work_start_accepts_at_file_input` | `start_request_replay_does_not_require_the_original_input_file` | merged_unique_oracle_first |
| `work_start_default_name_is_flow_id` | `work_start_creates_work_and_prints_next` | retired_strict_subset |
| `named_but_missing_workbook_is_not_silently_replaced` | `start_deterministic_rejections_leave_home_unchanged_and_do_not_burn_seq` | merged_unique_oracle_first |
| `digest_v2_framing_known_vectors_match_independent_hashes` | `of_bytes_matches_known_vector`、`digest_v2_prefix_is_domain_string_with_nul`、`be64_encodes_big_endian_bytes`、`digest_v2_file_frame_is_length_prefixed_path_and_content`、`digest_matches_independent_vectors_on_disk`、`framing_collision_pair_now_yields_two_different_digests` | retired |
| `node_with_out_edges_is_not_terminal` | `compiles_article_review_example` | merged |
| `begin_engine_stats_counts_current_attempt` | `begin_binds_engine_stats_and_emits_write_file` | merged |
| `submit_accepts_summary_of_exactly_4096_bytes` | `submit_rejects_summary_over_4096_bytes` | merged_and_strengthened |
| `submit_accepts_output_of_exactly_max_bytes` | `submit_rejects_output_over_max_bytes` | merged_and_strengthened |
| `blocked_count_accumulates_across_kinds_and_keeps_after_approve` | `stats_blocked_count_by_reason` | merged |
| `host_require_snapshot_decode_rejects_one_invalid_field` | `host_require_snapshot_decode_accepts_valid_fields`、`host_requirement_snapshots_accept_exact_byte_limits_and_reject_one_extra_byte` | merged_and_strengthened |
| `description_accepts_exactly_2048_bytes` | `rejects_description_over_2kib` | merged |
| `timestamp_day_is_date_prefix` | `timestamp_parse_accepts_utc_second_precision` | retired |
| `all_examples_compile` | `no_example_declares_requires` | retired |
| `spec_dev_optional_inputs_all_point_to_reachable_upstream` | `rejects_optional_input_on_start_or_resource_source`、`rejects_optional_engine_stats_input`、`spec_dev_replanning_uses_required_review_copies_and_reachable_optional_history`、`spec_dev_escalation_inputs_cover_return_edge_to_verify`、`spec_dev_binds_decision_into_scaffold_implement_verify` | retired_and_boundary_repaired |
| `mutated_two_step_manifest_with_extra_field_is_rejected` | `rejects_unknown_field` | merged_and_strengthened |
| `allocate_seq_is_not_reused_after_failed_start` | `allocate_seq_starts_at_1_per_day_and_increments` | retired_duplicate |
| `submit_replay_after_output_change_returns_original_snapshot` | `completed_submit_replay_does_not_seal_or_rewrite_the_output_again` | merged_oracles |
| `update_rejects_checksum_mismatch_and_leaves_binary_intact` | `update_failed_digest_leaves_old_binary_and_cleans_tmp` | retired_duplicate |
| `status_works_after_workbook_removed` | `work_readable_after_workbook_removed` | merged_cases |
| `malformed_started_workbook_ref_is_not_projected_as_success` | `malformed_started_identity_is_not_projected_as_success` | renamed_parameterized |
| `malformed_started_name_is_not_projected_as_success` | `malformed_started_identity_is_not_projected_as_success` | merged_cases |
| `unknown_nested_status_field_is_not_projected_as_success` | `malformed_cancelled_status_is_not_projected_as_success` | merged_cases |
| `self_install_on_new_home_creates_root_store_and_bin` | `install_copies_current_exe_and_is_idempotent` | merged_oracles |
| `remove_rejects_when_active_work_references_version` | `remove_rejects_active_reference_and_rolls_back_row` | merged_oracles |
| `remove_moves_dir_to_tmp_before_delete` | `remove_deletes_row_and_directory` | retired_obsolete_contract |
| `verify_reports_tampered_after_byte_change` | `load_rejects_tampered_registered_digest` | merged_oracles |
| `old_remove_replay_does_not_delete_readded_workbook` | `completed_old_remove_and_add_replays_preserve_new_lifecycle_bytes_and_row` | merged_oracles |
| `old_add_replay_does_not_bind_to_a_readded_workbook_lifecycle` | `completed_old_remove_and_add_replays_preserve_new_lifecycle_bytes_and_row` | merged_oracles |

### 修正弱判定依据

两条schema形状测试用独立手写schema2、合法只读/读写控制、单列缺失/类型变化、准确works定位和非空main/WAL逐字节不变；不再以版本1早退冒充表形状覆盖。提交前观察拒绝测试保持真实外部TempDir存活，并核全表revision/state/request/audit、哨兵字节/权限不变。CLI输出叶软链也核准确错误和同类完整事实；恢复正例从“文件存在”加强为登记历史字节/当前状态卡事实。

独立私有源码副本的正控全部通过；禁止表形状比较后两条schema测试失败，submit前注入仅revision违规写后观察测试失败，叶读取同时改成follow stat与open后CLI软链测试失败。均正常编译、真实执行后失败；这些是定向辨别力证据，不是完整变异验证或旧M1账本的新处分。

### 边界与治理

保留不同消费者、COMMIT/rename/delete标记/parent sync、观察句柄与锁替换、历史/当前响应、只读/恢复/清理、Exit70与SIGKILL的独立义务，未因相似名称或重型耗时删反例。MVP卡21条退休或加强归属只作原位Replacement历史注记；移除注记后与开工原文逐字节相同，checker未改。测试default之外的testkit支持接口变化只证明本仓库消费者闭合，不声明任意外部feature consumer兼容。

独立审查原文为`/private/tmp/sheltie-t40-independent-review.md`，本机临时证据不属于固定Git历史归档。当前完整普通门禁688/688与5/5 compile_fail均通过，7快照与默认生产AST保留；M1/SK、完整变异缺失、Linux/T16/T17状态保持原记录。本次不自动提交或发布。

## T16 部分证据与交接复核

Reviewer：`Codex /root/t16_evidence_review`，未参与本轮记录、运行或 worker 产物。初次结论为「需修改」：冻结 task-rules 明确允许纯文档例外，不能将 T04 仅因没有函数体判为违规。记录和用户审核问题已纠正，不新增产品 finding。

纠正后记录内容结论为「通过」，范围只含候选 `82c6c551` 的构建、工程原文、请求/响应、ArtifactRef/Finder oracle、未完成项及人工交接。Reviewer 独立核了 Nextest 首次 exit 92 与最终 688/688、5/5 compile-fail、8 个实际封存引用、安装 bytes 和档案原文。完整审查原文保存在 T16 归档的 `independent-review.md`；归档加入最终记录后另核文件字节与引用，不沿用旧 hash。

初次归档不是 T16 验收通过：当时真实人工门槛、定稿、方案审核与条件、back/重规划、重开会话、Host 版本及至少两个 spec-dev 已验证任务均缺证据。普通工程门禁通过；正式 `check-task C002-T16 82c6c55` exit 1（状态不是 done），完成门禁仍 FAIL，未提交或发布。后续已补项见下节；当前进度和原文索引见 [plan.md](plan.md)、[validation.md](validation.md#c002-t16-当前宿主回归)。

### 后续阶段复核

同一独立 Reviewer 的 `continuation-review.md` 与 `final-review.md` 单列后续真实批准、两次人工复制、受控 back、实际改方案与新版批准、四任务原始前缀及整体代码/CLI。最终任务报告和整体审查均实际 26/26、零 skip；环境限定当前空 venv 与继承沙箱闭包，不作全局 Host 信任结论。SDK 私有 policy 查询仅辅助观察，不以未知语义升级 PASS。

F16-01 修复复核通过：三个 Workbook 文件在先修订的 T16 白名单内，0.2.1 注册/副本/新 brief 与正确 WorkLayout 一致；原 0.2.0 冻结和历史 brief 未改。producer fixture 产物合成，不能替代真人或模型；反思建议只有一条有证据的问题，不凑数量。Reviewer 初次报告未预判尚运行的修复门禁；其后实际 688/688、5/5 与全工程门禁退出0，原文见 fix-gates。

最终仍不是 T16 完成：Work006 待真实重开会话继续，Work003 retro gate 待用户批准；usage 无可用计量，手动读取 skill 不证明自动发现。后续按当前 CLI 继续并复核这些必需项后，才可完成 T16 状态和提交。T17 发布仍未运行，旧 M1/SK/Linux 限制不改。

### 真实重开后的阶段复核

Reviewer：`Codex /root/t16_reopen_review`，未参与本轮 CLI、worker 产物或 package 修改。独立报告见重开续接档案的 `reopened-session/independent-review.md`，档案入口及摘要见 [README](README.md)。当前阶段结论为「通过」，T16 完成资格仍受 Work003 retro 的实际批准与 CLI gate 阻挡；前节未重开的结论保留为历史时点事实。

Reviewer 核新会话九次 CLI 原文与退出码/hash、Work006 running→succeeded 的 revision 2→5、当前 next、任务书绑定原提纲、417 字五段摘要质量及两份封存产物。当前 39 个唯一产物 bytes/hash/单链接/0444 全匹配；52 个重复引用位置不当作独立产物数。真实重开以用户陈述为来源，手动 skill 不证明自动发现；usage 缺失、Work/Attempt 耗时含等待，不推算成本。旧三阶段档案、修复后 688/688 与 5/5 原门禁及原始限制均保留。

docs/specs/diff-check 均 exit 0，正式 task gate exit 1（T16 状态不是 done），没有放宽 checker 或提前标完成。待具体批准后再核 CLI gate、当前终态及新阶段封存，才给 T16 最终结论；本轮未提交或发布。


### C002-T40 提交授权与证据保存

本任务的上述“未提交/不自动提交”是此前工作区交接事实。用户随后明确授权提交；本提交保留该任务实际源字节和对应原运行，原文已按README索引压缩归档，不以当前最终运行替代早期候选证据。

### 最终具体批准后的 T16 复核

同一独立 Reviewer 在 `reopened-session/final-review.md` 补核用户具体答复、批准前新 status、实际 approve revision 40/succeeded、最终 status/stats 及只读 audit/request 的一致性。结论「通过」，范围限定 plan §5 的本机真实场景；旧批准前报告和 pending 档案保持原样。结合既有真实交互、人工复制、受控 back、四任务/条件/重规划、Finder、错误恢复及本轮重开续接，必需项已齐，T16 可按该范围标 done。usage 缺失不是 0，手动 skill 不证明自动发现，0.1.0 二进制不写成新版 rc，旧 M1/SK/Linux/完整变异与 T17 限制不改。最终状态门禁与新档案仍按实际结果另行收尾，不把本次 gate 批准当提交或发布授权。

状态更新后实际 docs/specs/diff-check 及工作区 `check-task C002-T16 82c6c55` 均 exit 0，原文见 final-checks；批准前 exit 1 保留。本次未运行 staged 提交门禁、未暂存或提交，不声称提交完成。

独立 Reviewer 最后核最终 104 文件档案及批准前 69 文件档案逐字恢复、实际批准/CLI、六 Work 终态、277 项输入和原工程/最终工作区门禁。28 项核验均通过；派生 `C002-T16-reopen-final.audit.json` 在档案外保存，入口见 README，不修改两份报告或旧快照。本机范围的 T16 收尾通过，T17 not_run 及既有缺失保留。
