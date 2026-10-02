# C004–C007 一手来源核查：agent 工作流与 Rust 工程

本笔记对应 2026-09-27 的提案快照，供追溯来源与推论，不定义当前方案。最新范围与计划见 [change 索引](../changes/README.md)；不能沿用本文旧阶段编号、字段或实验形态作为实施依据。

获取日期：2026-09-27。本文为研究笔记，不是产品、机制或进度权威。核查对象是四个 `proposed` change 的 README、spec、design 与 validation；所有实验仍为 `not_run`。以下「来源事实」只陈述来源明确支持的内容；「对 Sheltie 的推论」用于审查，不能代替用户实验或采用决定。文中的 `[Sxx]` 指向末尾的一手来源。

## 共同判断

**来源事实。** OpenAI 建议从单 agent 加工具的可评估基线开始，按实际失败与收益增加编排；用任务级 eval 比较质量、成本和延迟，失败次数超限或高风险动作触发人工介入。[S01][S02] Anthropic 同样区分固定 workflow 与由模型动态决定步骤的 agent；评价器与生成者分离适合标准清楚、可测量改进的任务，运行中应从工具和环境取得事实，并设置停止条件。[S03] Anthropic 的 agent eval 指南要求按任务、试次和评分器评价，可组合代码、模型和人工评分；单次通过不能证明系统可靠。[S04]

**对 Sheltie 的推论。** C004–C007 把「验收」「接续」「交付」「生成」拆开，并保留单 agent 路径，符合增量引入机制的方向。它们的价值仍须用各自 validation 中预定的人工时间、质量和失败率判据检验。官方的工作流模式是设计选项，不构成 Sheltie 必须拥有多 agent、自动路由或新 runner 的证据。

## 近期社区讨论的适用边界

**来源事实。** MAST 作者在多 agent 框架的研究中，把观察到的失败归为规格与系统设计、agent 间目标不一致、验证与终止三组；论文并未测量 Sheltie 的单人本地代码库任务。[S24] 2026 年的 *From Spark to Fire* 在六个多 agent 框架上研究错误沿消息依赖传播，说明增加 agent 也会引入错误传递路径；它的防御数值不能外推成 Sheltie 的预期收益。[S25] METR 公开了模型在其软件开发和 AI 研发评测中修改测试或评分代码的案例，但明确说没有测出所有模型上的总体发生频率。[S26] OpenAI 的 Symphony 以任务系统承载长运行代码 agent 的编排，并把人为注意力和会话切换列为其团队的瓶颈；这是另一种组织规模与宿主条件。[S27]

**对 Sheltie 的推论。** 这些讨论支持把标准来源、候选输入、证据写入方式和错误传播路径纳入实验，而不是证明每个个人仓库都需要强制审查、保留样本或自动多宿主驱动。C004 的原生组对照与 C005 的第一阶段交接测量，是检验实际增量的必要步骤。[S24][S26][S27]

## C004：可验收委派

**来源事实。** OpenAI 的 agent 安全指南建议让不可信文本只经受控数据通道流动，节点间用固定 schema、枚举等结构约束；但也明确说结构化输出与 guardrail 不能消除提示注入风险。[S05] Anthropic 发现生成者自评可能偏宽松，分离评价者并给它可检验标准能改善反馈；评价者仍可能过宽。[S06] OpenAI 与 Anthropic 均强调环境权限、工具授权和人工关口是独立于模型判断的保护层。[S01][S05][S07]

**对 Sheltie 的推论。** 冻结验收规则、绑定候选及验证输入、区分命令失败与未执行、保留未验证条件、失败返回协调者，这些机制直接回应「agent 自报完成」的不可靠性。审查声明只能证明有人提交了结构化字段，不能证明审查内容正确；C004 对此已有明确限制。候选代码和本地验证器若同属一个 OS 权限域，快照工作目录只减少混淆，不能保证候选代码无法修改 `SHELTIE_HOME` 或伪造本地记录。建议在审查中重点核对「受控接收接口拒绝」与「同权限执行者无法绕过」是否始终分开表述；保留样本也不能宣称保密。[S07][S08]

**验证重点。** 以真实本地仓库做同任务对照，测量每个被接受任务的人工分钟数与质量；针对快照替换、额外文件变动、验证器故障、审查未完成、阻断项延续和越权分别保留反例。官方建议支持设评价和人工关口，不支持预言 C004 一定降低总投入。[S01][S04]

## C005：执行者替换与任务接续

**来源事实。** OpenAI 的托管 Agents API 已提供会话恢复、上下文压缩和委派能力；Claude Code 子 agent 可独立拥有上下文并恢复指定实例，但其请求仍计入主会话相同的使用额度。[S09][S10] Claude Code 的模型 fallback 仅覆盖某些服务端不可用情形；认证、账单和额度等错误不会触发 fallback，切换也只作用于当前 turn。[S11] OpenAI 与 Anthropic 都提供上下文压缩能力；Claude Code 文档说明早期详细指令可能在压缩后丢失。[S12][S13] Anthropic 的长任务实验用结构化交接产物连接会话，并报告仅靠压缩不足以保证长任务质量。[S06][S14]

**对 Sheltie 的推论。** C005 的第一阶段应证明的是跨宿主、跨会话仍能保留授权和证据要求，而非重复实现宿主内部上下文压缩。交接包中的「未完成、哪些证据有效、哪些限制仍生效」比完整转录更适合稳定接续；换执行者的执行代次和迟到提交拒绝属于引擎外部可观察的状态约束。第二阶段的自动等待、额度判定与跨宿主驱动不由上述官方能力直接推出；必须逐宿主验证状态与错误分类，未知时停下，不能把 usage 缺失解释为零。[S15]

**验证重点。** 先做人工跨宿主接续，记录重新解释、约束丢失、重复劳动及人等待时间，再决定驱动层。额度耗尽、审查者替换、旧代次迟到提交和人工门槛必须各有真实入口的反例；低成本模型分工要与强模型基线比较质量和完整任务成本。[S01][S15]

## C006：成果交付与依赖就绪

**来源事实。** Claude Code 的同名 skill 有来源优先级，个人、项目、插件来源不一定选择 Workbook 期待的那一份；同名 subagent 也有来源优先级。OpenAI Skills 文档说明 skill 是可复用指令文件，实际可用性受运行环境中的目录与加载机制约束。[S16][S10][S17] Rust 标准库的 `rename` 不能跨挂载点，`fs::copy` 会逐步写目标；`File::sync_all` 才尝试把文件内容和元数据同步到磁盘。SQLite 的事务保证数据库内的原子提交，但不自动把外部文件复制纳入同一事务。[S18][S19][S20][S21]

**对 Sheltie 的推论。** 单元一要把「已接收」「正在交付」「目标核对后已送达」区分，先写临时目标、核对字节，再决定回执与恢复步骤；不能把 SQLite 提交或一次文件复制成功直接等同用户目标处的持久交付。目标已被用户修改时应停止自动覆盖。单元二的宿主资源身份至少要考虑**实际解析到的来源**与可比对版本/摘要；只看到 `kind + name` 或文件存在不足以证明实际加载。技能内容的安全与质量属于另一项判断。[S16][S17]

**验证重点。** 单元一模拟复制中断、跨文件系统、回执前后崩溃和目标被编辑；单元二测试同名遮蔽、未知解析来源、版本不符及可选依赖。Rust 的 `Result`、单元/集成测试可表达这些失败路径，但编译通过或类型检查不会证明 I/O 恢复语义。[S22]

## C007：运行前生成 Workbook

**来源事实。** Anthropic 建议长任务把高信号上下文保存在结构化笔记/文件引用中，并按需加载；OpenAI 与 Claude Code 的 skill 都是可复用工作指令，不能自动成为可信状态权威。[S23][S17][S16] Anthropic 的长任务实验显示规划和执行可以通过交接产物配合，但自评偏差仍需独立评价。[S06][S14]

**对 Sheltie 的推论。** 先在现有 `workbook add` 路径上试「规划者生成、外部检查器核对、人确认」，比先改引擎合同更适合验证一次性任务的 authoring 成本。检查器可在有限 Flow 图上核对必需节点的路径覆盖、门槛和上限，这是**结构性质**；它无法从说明书自然语言证明验收标准充分、授权正确、预算实际不超、产物质量合格。规划者之外的生效约束快照是关键输入，但「另一个 agent 生成」也不自动提高其授权等级。[S05][S07]

**验证重点。** 比较手写与生成方案的人工时间、计划缺陷和最终质量；用绕过必需审查的路径及返工回环构造负例。`plan.lock.toml` 的摘要核对只说明文件未变，不说明规划者从正确约束出发，更不说明用户已经授权。[S05]

## 来源（一手资料，均于 2026-09-27 获取）

| 编号 | 来源与适用点 |
| --- | --- |
| S01 | OpenAI，[A practical guide to building agents](https://openai.com/business/guides-and-resources/a-practical-guide-to-building-ai-agents/)：增量编排、基线、模型选择、人工介入。 |
| S02 | OpenAI，[Evaluation best practices](https://developers.openai.com/api/docs/guides/evaluation-best-practices)：按任务建立评价。 |
| S03 | Anthropic，[Building effective agents](https://www.anthropic.com/engineering/building-effective-agents)：workflow/agent 模式、环境反馈、停止条件。 |
| S04 | Anthropic，[Demystifying evals for AI agents](https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents)：任务、试次、评分器与多层评价。 |
| S05 | OpenAI，[Safety in building agents](https://developers.openai.com/api/docs/guides/agent-builder-safety)：不可信输入、结构化通道、授权与提示注入限制。 |
| S06 | Anthropic，[Harness design for long-running application development](https://www.anthropic.com/engineering/harness-design-long-running-apps)：交接产物、自评偏差、分离评价。 |
| S07 | Anthropic，[How we contain Claude across products](https://www.anthropic.com/engineering/how-we-contain-claude)：模型、环境、外部内容的分层保护；同权限与子 agent 信任问题。 |
| S08 | Anthropic，[Beyond permission prompts: making Claude Code more secure and autonomous](https://www.anthropic.com/engineering/claude-code-sandboxing)：文件系统和网络隔离的边界。 |
| S09 | OpenAI，[Agents API](https://developers.openai.com/api/docs/guides/agents-api/overview)：托管 harness、会话、压缩和恢复的产品能力。 |
| S10 | Anthropic，[Create custom subagents](https://code.claude.com/docs/en/sub-agents)：子 agent 上下文、额度、恢复及定义优先级。 |
| S11 | Anthropic，[Model configuration](https://code.claude.com/docs/en/model-config)：fallback 触发条件和作用范围。 |
| S12 | OpenAI，[Compaction](https://developers.openai.com/api/docs/guides/compaction)：长对话的上下文压缩。 |
| S13 | Anthropic，[How Claude Code works](https://code.claude.com/docs/en/how-claude-code-works)：压缩后早期细节可能丢失。 |
| S14 | Anthropic，[Effective harnesses for long-running agents](https://www.anthropic.com/engineering/effective-harnesses-for-long-running-agents)：跨上下文的增量工作与交接。 |
| S15 | OpenAI，[Observability and usage](https://developers.openai.com/api/docs/guides/agents-api/observability)：usage 是尽力记录，可未知、延迟或改变；任务总成本比较。 |
| S16 | Anthropic，[Extend Claude with skills](https://code.claude.com/docs/en/skills)：skill 来源与同名优先级。 |
| S17 | OpenAI，[Skills](https://developers.openai.com/api/docs/guides/tools-skills)：skill 文件与运行环境发现机制。 |
| S18 | Rust project，[`std::fs::rename`](https://doc.rust-lang.org/std/fs/fn.rename.html)：跨挂载点限制。 |
| S19 | Rust project，[`std::fs::copy`](https://doc.rust-lang.org/std/fs/fn.copy.html)：目标文件复制语义。 |
| S20 | Rust project，[`File::sync_all`](https://doc.rust-lang.org/std/fs/struct.File.html#method.sync_all)：落盘同步。 |
| S21 | SQLite project，[Transactions](https://www.sqlite.org/lang_transaction.html) 与 [Atomic Commit](https://www.sqlite.org/atomiccommit.html)：数据库事务范围与持久性。 |
| S22 | Rust project，[Writing Automated Tests](https://doc.rust-lang.org/book/ch11-00-testing.html) 与 [Recoverable Errors with `Result`](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html)：错误与测试边界。 |
| S23 | Anthropic，[Effective context engineering for AI agents](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents)：按需检索与结构化笔记。 |
| S24 | Cemri 等，[Why Do Multi-Agent LLM Systems Fail?](https://arxiv.org/abs/2503.13657)：多 agent 失败分类与研究样本。 |
| S25 | Xie 等，[From Spark to Fire](https://arxiv.org/abs/2603.04474)：多 agent 消息依赖中的错误传播。 |
| S26 | METR，[Recent Frontier Models Are Reward Hacking](https://metr.org/blog/2025-06-05-recent-reward-hacking/)：评测中的投机案例及频率未知的限制。 |
| S27 | OpenAI，[Symphony](https://openai.com/index/open-source-codex-orchestration-symphony/)：任务系统编排与人工注意力瓶颈的团队案例。 |

来源是检索当日的官方页面；宿主版本和产品行为可能继续变化。实施前须针对计划支持的具体宿主版本复核，不能把此笔记视为运行验证。
