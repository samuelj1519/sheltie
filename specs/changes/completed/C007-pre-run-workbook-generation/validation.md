# C007 验证：质量与完整成本

Candidate: `45bf9feaf56a38a15e551a4a884b8e8cb833375f`

状态：`completed`；技术资产/scopedM1/M2完成；真实任务、原正式准入/结果与盲审not_run。文档静态检查不进入实验结果。

## 1. 正式运行前冻结

T01 在 protocol.md 固定三项任务和独立初始副本、两组文字一致的方法及 task/project、原生配置、模型/宿主/环境、六次顺序、actor × arm 的初始真实历史和任务分配、质量 oracle、重开位置、返工演练、最终交付与记录口径。至少同一使用者在每组完成三项任务。deliver 默认无 gate，真实不可替代授权例外和两组等价动作提前冻结；用户接受与独立盲审在流程之外单列。不得按已见结果改样本、分配或门槛。

| 预算或门槛 | 冻结值与用途 |
| --- | --- |
| 方法与配置准备活动分钟上限 | 采用时由任务提供者给实际预算；未填不开始准备 |
| 每次 run 的活动分钟与墙钟上限 | T01 在结果前定值；达到上限停止，保留已经投入的成本 |
| 总活动分钟、费用和截止范围 | 覆盖准备、六次运行、复核与分析；未知费用单列且不得当零 |
| 可接受质量 | 必需标准全部满足，无阻断缺陷；重大缺陷容忍度与严重性例子在运行前固定 |
| 有意义的改善 | 任务提供者预先固定可接受的准备回收范围和复用活动分钟改善；不在三项结果后选择有利阈值 |

这些是实验选择，不是已有产品事实。首次准备预算获人采用后才投入；若预算不足以完成必要独立审阅，缩小实验或停止，不删质量环节。

准备机制在 T01 使用临时合法/拒绝 CLI 和必要脚本测试完成，固定非零清单、原命令、退出与 raw run ID。M1 核准备可执行性，不替代真实试用；T02 只按 M1 审过的 first-use.md 保存原件；T03 分析，M2 核实际质量和完整成本。相同闭包下短审与 M 审引用相同证据，不重复执行无关检查。

## 2. 独立质量 oracle

每项任务冻结正向行为、必要拒绝/边界、已有检查、实际交付对象和人工接受条件。实现者看到要求，盲审者可另有独立检验材料；隐藏答案不能成为事后新标准。

审阅最终候选和 patch，重跑任务必需检查并核对实际结果。确认交付对应这一候选、说明足以使用、必须行为无缺失、无重大回归。由实际行为和代码判断质量，报告自称通过或引擎终态不算质量证据。盲审不能完整遮蔽时披露，独立审阅仍须保留。

| 规则 | 正例 | 反例与处置 |
| --- | --- | --- |
| 同任务配对 | 两组同一目标、初始候选、方法与标准 | 组间标准/工具变化，单列偏差，不作公平比较 |
| 当前接口 | 真实 add/start/begin/submit/status 路径可重现 | 使用提案字段或不存在命令，停止 |
| 完整质量 | 最终内容满足独立 oracle | 只评草稿或零退出，质量证据不足 |
| 真实续接 | 关闭旧会话，新会话根据自然材料继续 | 同会话「继续」，不计中断样本 |
| 首次读者 | 未写方法的人实际操作且帮助计时 | 作者代办关键步骤，披露并限制结论 |
| 真实复用 | 同一 actor 每组三次使用，exposure 据实际 prior_uses | 按 sample 编号或换人后仍称复用，拒绝复用结论 |
| 授权与质量 | 默认无 gate；必要授权两组等价，接受/盲审单列 | 人造门槛或 gate 代质量，比较不成立 |
| 返工来源 | 自然发现及演练明确分开 | 人造故障计入自然收益，分析失败 |
| 全部成本 | 准备、执行、核对、返工、续接和审阅均计入 | 删除失败时间或免费化配置，无法判断价值 |
| 原始记录 | 引用可逐条复算，未知为 null | 用 Attempt 时长推 token 或人工时间，拒绝结论 |

## 3. 分析口径

人工活动分钟按所有参与者、作者和助手实际投入求和，同一人的重叠区间不重复。模型或工具等待不计人工；可观察模型耗时、usage、费用与墙钟单列。每个活动只属于一个 cost_bucket，公共准备不能又计入 run。注入演练单列，不计自然复用收益。

设 `P` 为公共方法准备成本，`S_a` 为组 a 专用配置，`M_a(k)` 为该组截至第 k 个 run 的维护成本，`R_a,i` 为第 i 个 run 的全部人工活动，包括各 actor/助手填写、执行、核对、返工、续接、交付、流程外用户接受和独立质量审阅。组别累计比较采用：

```text
T_a(k) = P + S_a + M_a(k) + sum(R_a,i, i = 1..k)
```

两组分别采用同一 P，才能公平呈现方法准备是否回收。它不是实验实际支出两次；实际总投入单独求：

```text
T_experiment = P
             + sum(S_a + M_a(3) + sum(R_a,i, i = 1..3), a = native, sheltie)
             + E
```

`E` 是协议制作、测量、盲审编号处理和分析等纯实验开销，不混入任一产品运行成本，但实际总投入不遗漏。共同人工活动和各组专用活动按发生目的唯一归集；缺成本项记 unknown，不当零。报告 P、S、M、每次 R、E 和各组逐次累计值，并分别展示首次与实际复用的分项。

使用历史按 actor × arm 核算：开始前记录该 actor 在 native/sheltie 各自的 prior_uses，本组序号为 prior_uses[arm] + 1；本组次数为零才标 first_use。实际练习、失败或停止但已进入方法执行的 run 计使用一次；同一 run 的节点重试、返工和中断续接不增加方法使用次数，not_run 不计。跨组熟悉度、助手代办和换人单列，不把其他操作者的历史或 sample_id 当本人的复用经历。

至少列出：准备与维护；逐次任务输入；执行与最终审阅；候选/状态核对；自然返工和重复工作；重开续接；最终交付；墙钟；可取得的模型 usage/费用及缺失来源。宿主原始 usage 的字段含义不确定时不自行合并。

产品价值判断先看质量门槛，再看同一 actor 各组真实使用历史、含准备成本的累计投入与实际复用是否达到冻结改善条件。跨组熟悉效应或顺序偏差明显时只报观察，不单独归因复用。三项配对只支持当前方法、模型、宿主和样本方向；没有显著改善、缺数据或质量不稳可以停止或只改说明。不得把整体差异分别归因于原生检查、预检、替换或导出。

## 4. 执行记录

| 原正式义务 | 原计划模式 | 待补输入 | 原计划命令 | 原状态 | 原计划证据 |
| --- | --- | --- | --- | --- | --- |
| 方法与协议准备 | not_run | 待 T01 固定 | 未执行 | not_run | 无 |
| 三项任务六次配对 | not_run | 待真实样本与顺序 | 未执行 | not_run | 无 |
| actor × arm 使用历史与真实复用 | not_run | 待参与者历史、分配与每次序号 | 未执行 | not_run | 无 |
| 首次读者使用 | not_run | 待参与者与冻结入口 | 未执行 | not_run | 无 |
| 真实关闭重开 | not_run | 待配对阶段和自然材料 | 未执行 | not_run | 无 |
| 自然返工 / 注入演练 | not_run | 待实际发现 / 冻结夹具 | 未执行 | not_run | 无 |
| 独立最终质量盲审 | not_run | 待最终候选与 oracle | 未执行 | not_run | 无 |
| 流程外用户接受 | not_run | 待实际最终交付与用户判断 | 未执行 | not_run | 无 |
| 总投入与根因分析 | not_run | 待全部原件 | 未执行 | not_run | 无 |
| C007-M1 准备与可执行性 review | not_run | 待 T01 方法/协议/oracle/脚本闭包 | 未执行 | not_run | 无 |
| C007-M2 真实结果 review | not_run | 待分析候选与全部 raw 闭包 | 未执行 | not_run | 无 |

每次执行保存实际命令、raw run ID、stdout/stderr/退出码和输入标识。相同输入复用保留原 run ID；方法、任务、模型或环境改变后不能复用为同一实验。PASS 只表示该条已执行义务达标，不表示产品已采用、发布或各候选能力独立有效。

## C007-T00 采用事实

基线6614e42；C006全部101raw字节保真，实验依赖数据缺失且保留not_run。本轮只准备资产、技术CLI和补验；无Rust/Cargo/fixture或既有产品合同修改。正式预算/角色/标准未填不开始正式runs。

## C007-T01 技术资产与正式准入缺项

基准 `33352619a919b7f6142debb14a9822a4464fe336`。确认私有code-task-study@1.0.0：implement/review/deliver、3/3/1visits、retries1、gatefalse、无requires、256KiB报告/8MiB完整patch、终点3input+2output精确5key。Native与Workbook直接读同三instruction及完整策略，不另做简化基线。

[实际CLI原件](experiments/preparation/cli-mechanism.json)共13条外层调用，合法add/show/verify均0、起始键task/project；只追加unknown字段的临时invalid manifest实际WORKBOOK_INVALID/1，无最终Workbook目录，confirmed-source与根外哨兵字节不变。临时fixture三阶段begin/submit/status/result完整，resume保持原任务书，5结果有真实原件；没有求解真实任务、重开、自然返工或内容质量证据。

[实际binary](experiments/preparation/binary.json)从Cargo JSON取路径，源码0.3.0-rc.1、default/locked、SHA与环境有原件；[准备资产闭包](experiments/preparation/asset-closure.json)固定路径/尺寸/每文件SHA/帧算法。技术fixture不分配正式run ID/actual actor/exposure；人工分钟/usage/cost null。无重复正式记录摩擦，不创建collector/脚本测试或第二parser，task_files为空且不跑零Rust测试task.sh。

protocol/quality/templates记录实际任务、连续使用者/history、模型/host、真正会话、预算/阈值、质量/接受与顺序均pending。正式六run未开始；本轮M1只能独审资产与缺项，不批准原正式准入。后续供真实数据再冻结补验，保留原门槛。

独立补核first-use实际字段：status.data含resume/revision/effects_pending/pending_publish，resume含attempt/brief_path/draft_outputs/inputs；额外self version退出0，schema_version4/version0.3.0-rc.1，原件preparation/self-version.json。前13条原件不修改，不把单次机制续接查询算真实关闭重开。

T01作者文件的8处多余EOF空行在cached diff检查被发现，仅规范结尾单换行，未改方法词句。原CLI/闭包保真存preparation/prior-authoring，因Workbook字节摘要改变，另用新Home按规范后实际输入重跑13条CLI全部PASS，并登记新asset-closure；不把旧digest当新输入。

C007-M1本轮限定PASS，candidate53ceed8d完整SHA；原正式准入与六run不变，未重跑同输入CLI/引擎。docs/spec/tests及scope另验。

## C007-T02 授权未执行记录

开工bf6b26e1cabe61b6e621a91b5455b3f036f43075。按本轮范围登记[runs规划表](experiments/runs/README.md)六位：真实sample/run/actor ID均null、outcome not_run、原因逐项列明。没有正式Work、candidate/patch/checks/接受/盲审、真实重开或naturalrework，也没有人工P/S/M/R/E和usage数据。不用机制fixture代正式run，not_run不增使用历史。无准入不能开始计时/分配ID，按原标准补真实输入后独立正式M1再追加原件。

## C007-T03 证据边界分析

开工ed85bfaa167533ae898fa1e40aefc18c2d572227。[report](experiments/report.md)只描述11核心资产、13条技术CLI与六planning slots的实际范围。没有正式质量/人工P/S/M/R/E/累计/usage/费用/真实历史/重开/返工/接受原件，全部unknown/not_run，不估算差值或0收益，不认定产品无价值，也不提出无证据平台增量。保留原输入/标准/原件，逐项写正式补验门槛；方法生成与CLI成功不代实际使用。

## 本轮技术准备与延期交接验收

本表只核T00采用的可执行范围；原正式六run及准入/价值义务全部not_run。方法/协议/模板自T01未变，原13CLI与资产独审按同消费者输入引用，不称same whole package hash。没有产品源代码或宿主变化，不重跑未受影响842引擎测试。

| Requirement / risk | Mode | Input closure | Command / raw run ID | Result | Evidence |
| --- | --- | --- | --- | --- | --- |
| 私有有界方法、五个结果与Native相同完整文字 | executed | T01完整SHA，11核心asset闭包25fef64b…、template同commit冻结 | 独立方法/CLI审查、真实add/show/verify | PASS | [method](experiments/shared-method.md)、[闭包](experiments/preparation/asset-closure.json) |
| 现有CLI、合法/单条件拒绝及副作用范围 | executed | normalized真实Workbook、default/locked binary SHA及专用Home | cli-check-command.py，13外层caller；额外self version | PASS | [原件](experiments/preparation/cli-mechanism.json)、[binary](experiments/preparation/binary.json) |
| 完整first-use、成本/质量模板及真实准入缺项 | executed | T01模板/protocol/oracle，M1完整53ceed8d冻结且未变 | 独立T01/M1技术资产审阅 | PASS | [操作](experiments/first-use.md)、[protocol](experiments/protocol.md)、[oracle](experiments/oracle/quality.md) |
| 六个planning slot未执行记录，禁止虚构正式ID与结果 | executed | T02新增table，原样本/actor/真实事实缺失不补造 | 记录/字段/原条件复核 | PASS | [六位表](experiments/runs/README.md) |
| 不估算未知成本/质量/真实使用或各增量收益 | executed | T03只分析准备原件与缺项，标准/原件/历史未改 | 独立M2限定分析复核 | PASS | [report](experiments/report.md) |
| 当前文档/规格/范围与保真交接 | executed | 本轮author records与未改源码/原产品格式 | docs/spec/tests/diff/task scope | PASS | [plan](plan.md)、[review](review.md)、[archive保真](evidence/archive-c006-raw-preservation.json) |

## 原正式实验延期

| 原义务 | 结果 | 缺项与补验 |
| --- | --- | --- |
| 三真任务、两独立副本、六正式run | not_run | 提供未解决目标/验收/初始状态及真正配对输入，重新冻结正式协议 |
| 连续actor每组三次、真实历史/首次读者 | not_run | 实际人/模型host/助手/练习失败历史，不能据slot推使用次数 |
| 真实关闭重开、自然返工 | not_run | 实际会话关闭/新会话及自然发现原件，机制查询不代替 |
| 用户接受/独立最终质量盲审 | not_run | 真候选/完整patch/检查/标准/主体，不能由Work终态或报告推断 |
| P/S/M/R/E、usage/cost/改善阈值及真实价值 | not_run | 正式预算/阈值/实际活动原件缺失；未知null，不当0或无价值 |
| 原M1正式试用准入、M2真实结果验收 | not_run | 补齐全部真实输入、独立批准原准入并执行/分析后核原M2 |

本轮task done/限定review PASS仅表示技术资产与授权延期交接完成，原实验没有完成。后续补真实数据按原要求追加，不清洗本轮缺项。

## C007-M2 本轮限定结论

独立审定candidate `45bf9feaf56a38a15e551a4a884b8e8cb833375f`，技术准备/真实缺项/授权跳过交接PASS，无必改。53ceed8d到candidate的方法/模板/protocol/oracle/13原件完全无diff；六slot及分析没有造ID/使用次数/质量/成本，没有省时或无价值推断。仅收口E措辞为实验整体开销，未知值未变。原formalM1/six trial/formalM2真实结果不改not_run，不授权新机制或发布。
