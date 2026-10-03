# C008 实施计划

状态：`active`；T00已采用本轮条件前检/缺项交接，后续任务按下表。probe机制未采用，原机制与价值not_run。

每个任务和审查的范围基准都是本任务开工完整提交；提交前加 `--staged`，提交后用同一完整提交再核，不采用 README 总基线或随意 HEAD。准备/操作任务无 Rust 所属测试时不调用 task.sh。

## 1. 首次阅读与职责

先读 [README 采用条件](README.md#采用条件)、[规格 §2–4](spec.md#2-声明来源)、[设计 §2–3](design.md#2-输入与读取顺序) 和 [验证](validation.md)，再读自己的任务卡。当前没有已选真实声明或重复摩擦，不启动开发。人采用真实目标并授权只读范围后，本表是唯一进度来源。

T01 由复杂模型或有经验作者完整制作小 probe、必要 fixtures/tests、协议与 `experiment/runbook.md`，宿主解析、路径边界和未知语义不能交给简单执行者即兴设计。M1 独立复杂模型审准备与机制；T02 简单模型或初级开发者仅按手册做真实人工/脚本对照并记录；T03 复杂模型分析，M2 独立复杂模型核实际收益。

| 实施入口 | 职责 |
| --- | --- |
| 当前 Workbook 的 workbook.toml、manifest 指向的 Flow、选定 Node | 固定资源声明来源；已有真实引擎校验证据由 T01 核，probe 不复刻编译器 |
| experiment/probe.toml、host-rule.md | 一个宿主版本、一个目标、获准范围和每字段解释 |
| experiment/probe.py | T01 完整制作的配置校验、只读观察与 JSON；当前尚不存在 |
| experiment/test_probe.py、fixtures/ | T01 独立手工正反 oracle，机制不替真实样本 |
| experiment/runbook.md | T02 唯一运行入口：具体命令、参数、预期、退出与停止交接 |

本实验不改 Rust crate，不向 core/runtime 加宿主业务，不建 adapter trait 或第二套状态。产品工程边界、声明与公开操作见上游合同；不因短期探针成功直接提出准入。

| ID | 状态 | Owner | 交付 | 依赖 |
| --- | --- | --- | --- | --- |
| C008-T00 | done | root；独立Reviewer | 保真归档/唯一active与条件前检scope | 用户顺序授权 |
| C008-T01 | done | 复杂模型 / 有经验实验作者 | 声明/条件盘点、规则边界、补验手册；probe未采用 | 满足采用门槛，人授权只读范围 |
| C008-M1 | done | 独立复杂模型 Reviewer | 前检/缺项保真独审；原机制准入not_run | T01 |
| C008-T02 | todo | 简单模型 / 初级开发者；真实观察者执行真实动作 | 原机制/真实对照的具体未执行记录 | M1 |
| C008-T03 | todo | 复杂模型产品分析者 | 未知成本/覆盖边界和原条件补验报告 | T02 |
| C008-M2 | todo | 独立复杂模型 Reviewer | 本轮前检/延期交接独审；原真实价值not_run | T03 |

M1/M2 Reviewer 均未参与被审脚本、fixtures、tests、oracle、实现或分析。任务短审和里程碑在同一闭包、义务下引用同一证据，不双跑；输入变化只核受影响义务。提交一个任务一个提交，按工程规范和 check-task 核范围。无 Rust 归属测试，不运行 scripts/task.sh；测试归 T01，发现数必须非零，import 错误不算机制拒绝通过。

## 2. 阶段一：C008-T01 完整准备小探针

**输入与范围。** C007 自然事件或近期真实任务、作者确认的必需资源及重复摩擦、已校验 Workbook 固定副本、宿主版本、授权读取范围与预算。只制作本 package 的 experiment/ 资产及目标/机制证据，不改真实 requires、宿主配置、工作区或 Store。

**步骤与预期交付。**

1. 定位 manifest/Flow/Node 同一 kind:name，确认真实必需与重复摩擦。只有 resource.*、自然语言提及、没有授权或近期需要时停止，不制造声明。
2. 核一个实际宿主版本的一手搜索/选择/身份规则，固定 host-rule.md 与 probe.toml。部分范围未找到、选择不明、摘要闭包或版本语义不足预定 unknown；不能为减少 unknown 扩大读取。
3. 在结果前冻结人工对照、顺序与熟悉度、真实事件、各成本项、预算、最低有意义收益、停止和敏感内容边界。建立独立正反 oracle。
4. 完整实现 probe.py、test_probe.py 与最小 fixtures。高级路径检查、宿主解析、配置拒绝、无写入及 source 不执行由复杂模型负责，不留占位、假成功或通用适配层。
5. 在临时 fixtures 验证匹配、完整缺失、部分搜索未知、同名选择未知、版本/摘要不足、读取失败、输入变化、非法配置、source 不执行和 no-write。预期用手工身份/已知字节，不调用被测 helper 算答案。
6. 完整制作 runbook.md，给固定 Python、实际配置、允许读取路径、完整命令、唯一证据目录、JSON 示例、计时和失败停止；只做机制验证，不执行正式真实配对实验。

**runbook.md 必须交付的步骤表。**

| 步骤 | 参数处与预期输出 |
| --- | --- |
| 核起点 | T01/M1 commit、probe.toml、host-rule、脚本标识、宿主版本与授权；不符停下 |
| 人工核对 | 同一声明、规则、范围、预定顺序；原始观察与活动分钟，不接触额外内容 |
| 脚本观察 | 固定 `python3 -B <absolute-probe.py> --config <absolute-probe.toml>`；stdout 单份 JSON、stderr/exit 原件 |
| 解释输出 | 0 仅表示有效观察，逐项 matches/mismatch/unknown 由真实协调者解释；2 为配置/参数错误，停止交作者 |
| 保存证据 | 已授权唯一 experiment/evidence/real/<run-id>/；不覆盖旧 raw，输入变化另记观察 |
| 停止交接 | 明确不符/未知、预算或权限不足、漂移、异常写入/输出，保存事实并交固定 Owner |

**验收与停止。** 测试清单非零且全过，合法与拒绝例、只读边界独立可核，手册无需初级开发者设计政策。不存在可靠宿主解析或读取需要执行资源、联网、凭据、安装或越权时停止/unknown，不增加机制。有效报告退出 0 不能当资源合格；非预期非零或缺 JSON 是机制错误，不允许目录存在替代。

**验证与交接。** 运行 `python3 -B -m unittest discover -s <active-package>/experiment -p 'test_*.py'`，核预登记非零清单；跑临时合法/拒绝调用和 no-write，再跑文档检查。T01 把 active-package 占位替成真实路径写 runbook。交接 T01 commit、脚本/config/rule/test/fixture 闭包、正反 raw run ID、实际命令/Python、授权范围、预算与未知；不执行正式真实价值实验。

## 3. C008-M1：独立复杂模型审准备与机制

从真实声明和一手宿主规则核有限读取、路径边界、存在/选择/身份区别、unknown、source 不执行和 no-write。复核临时 fixtures、实际测试清单及准备成本，首次读者逐步核 runbook 的参数、退出、记录、停止与交接。

M1 只判断该探针和手册是否可以进入正式真实观测，不宣布任何真实资源当前可用，不认定价值。问题交 T01 作者修复；闭包改变后复核影响义务，原 raw 保留。Reviewer 不审自己编写的修复。

## 4. 阶段二：C008-T02 按手册做真实对照

**入口与范围。** M1 通过的 T01 commit 与 experiment/runbook.md。只新增 experiment/evidence/real/ 和 review/validation/progress 索引；不改 probe、test、fixtures、protocol、host-rule、config 或 runbook，不改宿主、任务仓库或 Store。

**步骤与预期。**

1. 核闭包、实际宿主版本、声明和授权相同，按 protocol 的真实事件、顺序和允许范围执行。
2. 真实观察者按冻结规则人工核对并计时，脚本按固定命令输出 JSON；记录先后熟悉效应与全部解释/复核成本，不把脚本报告当人工观察。
3. 保存 JSON、stdout/stderr/exit、配置/环境引用、逐项结果、时间及 probe 区间 no-write 证据。正常 Work 推进和外部写证据另列。
4. 匹配只说明这次有限观察；不符或 unknown 原样交协调者，不强行执行任务制造失败，不扩范围或安装。

**验收、停止与交接。** 同一声明/环境/资料的真实对照及全部 unknown 可复核；0 的有效不符/未知报告保留，2 或其他异常退出/缺 JSON 停止。版本/声明漂移、授权不足、预算耗尽、异常写入或脚本缺陷时保存原件并交 T01 作者；T02 不临时修脚本/政策。修复由 T01 作者在修复任务产出新闭包，经独立受影响义务复核后才续跑，新观察不覆盖原 raw。交接实际 run ID、声明/脚本闭包、全部结果、成本、偏差、缺项、允许范围和 T02 commit。只跑记录所需文档/范围检查，不重复跑相同机制测试证明价值。

## 5. C008-T03：复杂模型分析是否保留

**输入与范围。** T01/M1 机制闭包、T02 全部原件、准备维护成本和预算。只写 experiment/report.md 及 review/validation/progress 索引，不改原始结果、规则、声明或标准。

分别列机制结果、逐项可观察范围、真实重复事件、人工/脚本全部成本、误报和 unknown 复核；计算含准备维护的累计净成本与近期复用回收条件。没有足够真实事件、成本不划算或规则太重时可以停止/只改善说明。

**验收与交接。** 成本可复算，夹具不算真实收益，未执行失败不算避免损失，选择未知不算匹配；结论限这一个宿主、版本与范围。交接 T03 commit、报告/观察闭包、原审查引用、未知及待人决定范围，跑文档/范围检查。分析不自动授权安装、持续扫描、缓存、内核合同或发布。

## 6. C008-M2：独立复杂模型核真实收益与边界

核真实声明、逐项原始观察、人工顺序偏差、范围、unknown、误报、完整准备维护成本和结论，机制与价值分开。相同闭包下直接引用 M1 机制证据；脚本或环境变化只核影响义务，不双跑无关检查。

fixture PASS、静态审查或退出 0 不替真实收益。缺项保留 not_run，问题交 T01/T02/T03 Owner，Reviewer 不审自己修复。findings 写 [review.md](review.md)，命令与证据索引写 [validation.md](validation.md)。

### C008-T00：采用前提盘点与原条件机制延期交接

基线75b81ebed655693844ae417bb7d501a1b12a4387。用户授权顺序执行并允许缺环境/数据操作记录跳过；当前没有真实必需kind:name、固定host/version/rules、读取范围、重复摩擦或预算。本轮只完成T01声明/条件盘点与规则边界/runbook，M1审前检保真；T02登记原观察/探针机制not_run；T03报告未知成本与补验；M2审本轮交接。原probe not_adopted及机制/真实验收not_run完整保留，不添加requires或扫host制造目标。

原T01开发probe/test、原M1机制准入、原T02人工/script对照、原M2价值结论都在实际条件补齐后再按原标准执行。本轮task done仅可执行前检与授权延期交接，不表示原conditional实验完成。各CLI/声明读取30秒反馈预算，不作为人工实验预算或安装授权。
