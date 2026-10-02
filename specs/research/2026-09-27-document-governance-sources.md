# 文档治理一手来源笔记

日期：2026-09-27

本文为 Sheltie 的 specs 管理方案提供来源依据。本文不是权威规格、实施计划或已采用决定，也不表示下列建议已经实现。

## 1. 仓库基线

先按仓库内的权威顺序核对当前文档：

- [文档地图](../README.md) 规定「宪章 → 规格 → 架构 → 合同 → 计划」的冲突裁决顺序，并把路线图、决策、时点性审查和修复提案放在不同位置。
- [MVP 实现计划](../releases/v0.1.0/plan.md) 已关闭；T01 至 T26、M1 至 M3 均为 `done`。
- [路线图](../roadmap.md) 保存 MVP 之后的产品愿景。条目只有满足立项条件后，才应长出独立 spec 与 plan。
- [MVP 决策记录](../releases/v0.1.0/decisions.md) 同时承载产品决定、里程碑复核记录与首次真实运行记录，现已归档。
- [T26 后复审](../changes/completed/C002-v0.2.0-reliability/findings.md) 固定候选与证据；[v0.2.0 根因修复方案](../changes/completed/C002-v0.2.0-reliability/README.md) 明确声明自己是未采用提案。二者都不是进度权威。

当前结构已经区分目标、进度、未来愿景和时点性证据。待设计的问题是：如何让长期稳定指令、按需文档、进行中状态、版本计划、提案、决定和归档各有单一职责，并让 agent 能低成本找到正确入口。

## 2. OpenAI 官方来源

### 2.1 Codex 的 `AGENTS.md` 加载方式

来源：[Custom instructions with AGENTS.md](https://learn.chatgpt.com/docs/agent-configuration/agents-md)

来源直接说明：Codex CLI 会从全局位置以及仓库根到当前工作目录逐层发现指令文件，按由宽到窄的顺序注入上下文；越靠近当前目录的内容越晚出现。由此可得：

- **稳定指令：** `AGENTS.md` 适合存放每次工作都要遵守的仓库规则、真实命令和文档路由。目录级规则可以缩小适用范围。
- **按需文档：** 长规格不应复制进多层 `AGENTS.md`。入口只需说明什么任务去读哪份权威文档。
- **归档：** 已失效内容不能继续留在自动加载路径中。仅移动到另一个仍会自动加载的位置，不能降低它对 agent 的影响。

这支持保留短入口并链接权威文档，不支持把全部 specs 或当前任务流水塞入 `AGENTS.md`。

### 2.2 重新审视 Skills、prompts 与 `AGENTS.md`

来源：[Rethinking skills and prompts for GPT-6 Astra](https://developers.openai.com/blog/rethinking-skills-and-prompts-for-gpt-6-astra)

来源直接建议定期检查始终加载的 `AGENTS.md` 指令是否仍有必要；任务特定材料应按任务读取。Skill 的根文件应是最小路由，支持按需加载参考资料和脚本。由此可得：

- **稳定指令：** 只保留长期有效、适用于广泛任务的边界。模型升级或工作方式变化时也要复审这些指令。
- **按需文档：** 使用简短描述进行发现，再加载与当前任务有关的正文、参考和脚本。
- **当前进度：** 高频变化的状态不适合放进始终加载的入口；入口应指向单一状态源。

### 2.3 Codex 长任务的持久项目记忆

来源：[Run long horizon tasks with Codex](https://developers.openai.com/blog/run-long-horizon-tasks-with-codex)

这份实践将长任务状态外置到仓库文件，并把不同职责分开：目标与完成定义、里程碑与验证、执行规则、当前状态与决定。它还要求每个里程碑验证失败后先修复，再继续。由此可得：

- **当前进度：** 进度需要可重复读取的持久状态，至少回答「已完成什么、下一步是什么、怎么验证」。
- **版本规划：** 计划应由可完成、可验收的里程碑组成，并为每个里程碑附验证命令和失败停止规则。
- **决策记录：** 计划中的简短决定笔记可以防止 agent 在长任务中反复改变方向；重要决定仍应进入独立决定记录。
- **稳定指令与状态分离：** 执行规则告诉 agent 怎么工作，状态文件说明本次工作走到哪里；两者不应混写。

### 2.4 agent 评估的声明边界

来源：[Evaluate agent workflows](https://developers.openai.com/api/docs/guides/agent-evals)

来源建议在调试阶段检查包含模型调用、工具调用、guardrail 与 handoff 的完整 trace，并用结构化 grader 找到回归与失败模式；完成标准明确后，再把它固化为 dataset 和可重复 eval run。由此可得：

- **当前进度：** `done`、`PASS` 或审查结论必须绑定本次端到端 trace、判定标准和结果，不能从旧运行外推。
- **变更提案：** prompt、工具、路由或 guardrail 改动要用同一 dataset 和 eval 口径比较，避免只凭单次演示判断改进。
- **归档：** 调试 trace 与稳定回归集职责不同；前者保留单次诊断证据，后者支持跨版本重复比较。

## 3. Anthropic 官方来源

### 3.1 长任务 harness 的进度文件与增量交付

来源：[Effective harnesses for long-running agents](https://www.anthropic.com/engineering/effective-harnesses-for-long-running-agents)

来源识别了两类失败：agent 一次做太多，留下半成品；agent 看到已有成果后过早宣布完成。它用完整 feature list、进度文件、Git 历史、一次一个 feature、验证后再改状态来缓解。由此可得：

- **当前进度：** 状态必须覆盖完整目标集合；所有条目初始为未通过，只能在验证后改变状态。
- **版本规划：** 大目标应拆成一次可以完成并留下干净状态的增量。
- **归档：** Git 记录变更事实，进度文件记录交接所需的当前摘要。二者互补，不能互相替代。
- **稳定指令：** 每轮都要执行的「先读状态、选一个未完成项、验证、更新进度」属于工作协议；具体 feature 内容属于状态数据。

### 3.2 Claude Code 的项目记忆

来源：[How Claude remembers your project](https://code.claude.com/docs/en/memory)

来源区分了人维护的持久指令与 agent 自己积累的记忆；说明二者都是上下文，不是硬性执行机制。它建议 `CLAUDE.md` 保持具体、简短，并把多步骤流程或局部规则移到 Skill 或路径限定规则。由此可得：

- **稳定指令：** 入口文件保存构建命令、约定、项目布局和「始终执行」的规则。硬性阻止动作仍要靠确定性检查或 hook。
- **按需文档：** 多步骤流程、局部规则和长参考资料不进入始终加载的入口。
- **当前进度：** agent 自动记忆不能成为团队共享的进度权威；权威状态必须进入版本控制并由明确规则更新。
- **归档：** 要定期清除冲突、过期的始终加载指令；历史原因放到可追溯记录中，不继续作为当前命令生效。

### 3.3 Agent Skills 的渐进式披露

来源：[Agent Skills overview](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/overview)

来源把内容分成三层：始终加载的名称与描述、触发后加载的 `SKILL.md`、按需读取的参考资料与脚本。由此可得：

- **按需文档：** 文档地图可以采用同一结构：短元数据帮助选择，入口页负责路由，详细资料只在任务需要时读取。
- **稳定指令：** 始终加载层只承担发现，不承载完整流程。
- **归档：** 归档资料应能由当前入口追溯，但不应继续出现在默认任务路由中。

### 3.4 长任务中的 sprint contract 与独立评价

来源：[Harness design for long-running application development](https://www.anthropic.com/engineering/harness-design-long-running-apps)

来源在每个 sprint 实施前，用文件记录生成者与评价者共同确认的完成合同；实施后再交给评价环节。这样可以在不提前规定全部实现细节的前提下，保持结果忠于上层规格。由此可得：

- **变更提案：** 提案应先明确问题、范围、成功判据和验证方式，经采用后才转成实施任务。
- **当前进度：** 「提案已写」「提案已采用」「实施中」「已验证」是不同状态，不能压缩成一个 `done`。
- **决策记录：** 对完成定义的协商结果可以成为后续决定的输入；正式采用的跨任务选择再进入决定记录。

## 4. 通用文档治理来源

### 4.1 Diátaxis：按读者需要区分文档职责

来源：[Diátaxis](https://www.diataxis.fr/)

Diátaxis 区分 tutorial、how-to guide、technical reference 和 explanation，并要求文档围绕不同读者需要组织。由此可得：

- **按需文档：** 同一文件不要同时承担教学、操作步骤、精确合同和原理解释。
- **版本规划：** 路线图与实施计划不是 how-to 或 reference；它们需要独立的生命周期文档类型。
- **归档：** 归档应按原文档类型保留语境，不能把历史说明混进当前 reference。

对 Sheltie 而言，spec 与 contracts 接近 reference，architecture 与 constitution 更接近 explanation，engineering 与 runbook 更接近 how-to；快速开始或完整样例才承担 tutorial。这个映射是本文对来源的应用，不是 Diátaxis 对 Sheltie 的原话。

### 4.2 Semantic Versioning：版本号表达兼容性

来源：[Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html)

SemVer 要求先定义清晰的 public API；发布后的版本内容不得修改，任何修改都必须作为新版本发布。`MAJOR.MINOR.PATCH` 分别表达不兼容变更、向后兼容功能和向后兼容修复；`0.y.z` 表示初始开发期。由此可得：

- **版本规划：** Sheltie 必须先声明哪些合同、命令、格式和可观察行为构成 public API，再据此决定版本增量。
- **变更提案：** 提案要标出对 public API 的兼容性影响；目标版本不能只按工作量或日期命名。
- **归档：** 已发布规格和合同应保持可重建。发现错误时通过后续版本纠正，并保留原版本记录。

SemVer 管产品发布和 public API 兼容性，不直接规定内部文档目录的版本号。把每次文档编辑都解释成产品版本变化，会扩大来源的适用范围。

### 4.3 Keep a Changelog：面向使用者记录重要变化

来源：[Keep a Changelog 2.0.0](https://keepachangelog.com/en/2.0.0/)

来源要求维护 `Unreleased`，发布时转成带日期的版本节，再新建空的 `Unreleased`；changelog 只记录对读者重要的变化，不是 commit 日志。它还要求弃用先于删除，并建议大型历史只在稳定后归档，主文件与归档双向链接。由此可得：

- **版本规划：** 已采用且预计进入下一版的用户可见变化进入 `Unreleased`；路线图中的愿望和未采用提案不进入 changelog。
- **变更提案：** 提案记录为什么和怎么改；changelog 记录最终对使用者产生了什么变化。二者不能互相替代。
- **归档：** 发布历史不得删除。需要拆档时，只归档不再编辑的旧版本，并保持双向链接。
- **当前进度：** changelog 不是任务看板，不能用 `Unreleased` 代替 `plan.md` 的任务状态。

### 4.4 ADR：保留重要决定的背景与后果

来源：[Michael Nygard：Documenting Architecture Decisions](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)、[ADR Templates](https://adr.github.io/adr-templates/)

原始 ADR 提案要求每份短记录只描述一个重要决定，并包含 title、status、context、decision、consequences。编号单调增加且不复用；决定被反转后保留旧记录，标为 `deprecated` 或 `superseded` 并指向替代记录。ADR 社区站点进一步列出最小模板与包含备选方案权衡的 MADR 模板。由此可得：

- **变更提案：** 尚未同意的选择标为 `proposed`，不能直接写成当前架构事实。
- **决策记录：** 正式记录应写明背景、已选方案、重要备选方案、后果和状态，而不只是复盘流水。
- **归档：** 已被替代的决定保留原编号与原因，双向链接新决定；不覆写成仿佛从未发生。
- **按需文档：** 决定记录解释「为什么」，当前 spec、architecture 和 contracts 继续定义「现在是什么」。读当前合同不需要先重放全部历史。

## 5. 来源覆盖矩阵

`●` 表示来源对该治理面给出直接依据；`○` 表示只能作为间接推导，不能单独支撑规则。

| 来源 | 稳定指令 | 按需文档 | 当前进度 | 版本规划 | 变更提案 | 决策记录 | 归档 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Codex `AGENTS.md` | ● | ● | ○ |  |  |  | ○ |
| OpenAI Skills / prompts | ● | ● | ● |  |  |  |  |
| OpenAI 长任务 | ● | ● | ● | ● | ○ | ● | ○ |
| OpenAI agent 评估 |  |  | ● |  | ● | ○ | ● |
| Anthropic 长任务 harness | ● | ○ | ● | ● |  | ○ | ● |
| Claude Code memory | ● | ● | ○ |  |  | ○ | ● |
| Agent Skills | ● | ● |  |  |  |  | ○ |
| Anthropic sprint contract |  | ● | ● | ● | ● | ● | ○ |
| Diátaxis | ○ | ● |  | ○ | ○ | ○ | ● |
| Semantic Versioning |  |  |  | ● | ● |  | ● |
| Keep a Changelog |  |  | ● | ● | ● |  | ● |
| ADR |  | ● |  |  | ● | ● | ● |

## 6. 适用于 Sheltie 的设计约束

以下是基于来源和仓库基线提出的约束，供后续方案采用或否决。

1. **`AGENTS.md` 只保留长期规则和路由。** 包含不变式、真实门禁命令、权限边界和「什么任务读什么文档」。当前版本、当前任务、临时 review 结论和未采用提案不得进入始终加载层。
2. **每类文档只有一个职责和一个权威入口。** 产品目标、机制合同、实施进度、未来愿景、变更提案、决定、审查证据和发布变更分别存放；入口页只做地图和权威关系说明。
3. **按任务渐进读取。** 默认入口提供短描述、适用条件和链接；实现者只读取任务所需的权威正文。不能用「开工前通读全部 specs」作为所有改动的固定前置条件。
4. **进度由单一、可验证的状态源表达。** 任务状态至少区分 `todo`、`doing`、`blocked`、`done`；`done` 必须绑定完成判据和证据。review、提案、提交存在或 changelog 条目都不能替代进度状态。
5. **路线图、提案与实施计划分开。** 路线图说明将来可能做什么和立项条件；提案说明具体问题、候选方案、影响和验收；只有提案被采用并同步上游权威后，实施任务才进入进度计划。
6. **提案采用状态显式化。** 至少区分 `proposed`、`accepted`、`rejected`、`superseded`；提案必须标明 owner、目标版本、依赖、受影响权威文件、成功判据和替代提案链接。
7. **决定记录保持追加式历史。** 一个编号只记录一个重要决定；编号不复用。被替代记录保留背景与后果，标记状态并链接新决定。当前合同只表达有效规则，不承载完整争论历史。
8. **产品版本以公开合同的兼容性为依据。** 先列出 Sheltie 的 public API 边界，再判断 major、minor、patch 或预发布版本。已发布规格快照不可静默改写；纠正通过新版本和 changelog 说明。
9. **changelog 只记录已采用、用户可感知的重要变化。** 未采用提案、内部任务流水和原始 commit 列表不进入 changelog。发布时从 `Unreleased` 生成版本节；弃用先记录，删除后记录。
10. **归档保留可追溯性并退出默认路由。** 只归档不再更新的时点性材料；当前入口与归档双向链接，保留日期、候选、版本、状态和替代物。归档内容不再被 agent 当作当前规则自动加载。

## 7. 方案设计时需要明确的边界

这些问题不能由上述通用来源替 Sheltie 决定，后续方案必须显式选择：

- MVP decision log 已归档；新决定由 `specs/decisions/` 的独立 ADR 管理。
- repair proposal 的正式采用者是谁，采用动作由哪个可审计状态或提交表示。
- 产品 `CHANGELOG.md`、规格快照与 Git tag 之间谁生成谁，以及发布失败后的修订规则。
- `plan.md` 在 MVP 完成后是封存的历史计划、持续任务表，还是按版本拆分；无论选择哪一种，只能有一个当前进度权威。
- 哪些 `spec.md`、contracts 和 CLI 行为构成 SemVer 意义上的 public API；`0.y.z` 阶段的兼容承诺也要写清。
