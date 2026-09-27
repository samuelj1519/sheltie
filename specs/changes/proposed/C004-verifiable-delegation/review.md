# C004 方向评估

结论：建议以“可验收委派”作为下一版的候选方向，采用前必须先完成 [validation.md](validation.md) 的实验。本文件记录方向讨论的依据。它不是实现后的独立 review；独立 review 在采用并实现后另行进行。

基准：`296313c6f9fcfd0d6348927b73e7a49578581b6c`。日期：2026-09-27。参与者：用户、Claude Code，以及两位外部讨论者（下称 A、B）的书面意见，共十一轮。第七轮把执行器从 CI 改为本地优先，见 §7；第八至十一轮讨论独立审查、执行者替换与额度接续，与验收相关的结论见 §8，执行者与接续部分拆到 [C005 review](../C005-executor-continuity/review.md)。§3 中被修订的条目已标注。

## 1. 起点

用户提出：模型会越来越强，固定 flow 的价值会下降；一个任务可能需要多个 agent 配合，agent 之间的组织调度和隔离会越来越重要，所以考虑把 Sheltie 的核心价值转向这两点。

## 2. 为什么不把多 agent 调度和隔离作为核心价值

- **成本。** Anthropic 的公开资料显示，多 agent 系统消耗的 token 是单 agent 的 3–10 倍，研究型多 agent 系统约为普通对话的 15 倍。只有任务可以高度并行、而且价值足以覆盖成本时才划算。多数代码库变更不属于这种情况。（第九轮限定：这是 Anthropic 对并行探索式方案的经验值，不适用于把工作分给便宜模型的分工；token 数也不等于付费或订阅额度占用。）
- **失败来源。** MAST 对 210 条多 agent 轨迹的分析显示，大部分失败来自规格不清、agent 之间目标不一致和验证缺失，而不是调度算法。Spark to Fire 显示错误会在 agent 之间传染；基于来源图的防御能把防御率从 0.32 提到 0.89，去掉拦截机制后防御就失效。这说明价值在“证据来源 + 拦截点”，而不在“更多 agent”。
- **宿主已在做调度。** Claude Code 的 subagent 和 Agent Teams、Codex 的 subagents、OpenAI Agents SDK 的编排都已提供调度能力。Sheltie 在这一层做不出差异。
- **隔离做不出可信版本。** Sheltie 是本地单用户工具，与 agent 以同一个 OS 用户运行。门槛批准人可被 `USER` 伪造（已复现，见 C002 N04），Store 文件对 agent 可写。在这个信任模型下宣称隔离是误导。OpenAI 的 sandbox 和宿主沙箱才是隔离的正确位置。
- **官方方向。** Anthropic 的 Managed Agents Outcomes 用独立 grader 按标准验收结果，报告的提升最多约 10 个百分点。验收和证据正在成为委派的核心，这与“可验收委派”一致。

## 3. 三方收敛的结论

1. 定位是“面向代码库变更的可验收自主委派层”，内部是“合同 + 账本 + 受控接收点”。
2. 静态政策、动态计划。政策只在一个授权版本内静态，放宽必须形成新的显式授权。
3. 单 agent 下也必须成立，作为近期筛选原则，不写成不变式。
4. 首版必须有返工闭环。只会拒收、不会让 agent 继续修的系统，用户价值不够。
5. 承诺逐条写信任前提，区分“正常接口会拒绝”和“执行者无法绕过”。
6. 证据用属性描述，不排高低等级。
7. 换一个模型审查不等于独立验证；写集不相交也可能有语义冲突。
8. 预算分三栏：能强制的、估算的、不可知的。
9. GF-25（并行）与 GF-26（动态展开）分开决策。
10. 要限制的是待审、待集成的在制品数量，而不是 agent 数量。
11. v0.3 维持单宿主、单 agent、单用户信任；沿用静态 Workbook；不做守护进程和 Touch ID。~~验收用项目已配置的 Git/CI~~（第七轮修订：改为本地快照验证器，CI 后置，见 §7）。
12. 宿主权限配置只算防误操作加固，待验证：沙箱禁写也会挡住 CLI 子进程，Bash 规则按命令文本匹配，不是安全边界。
13. ~~验证过程会执行项目代码，所以控制器与执行器分离。~~（第七轮撤回：本地验证下分离只剩进程级和目录级，没有信任级；承诺表第二行在本地路径下改为不承诺，见 §7。）
14. 评估不能用 Sheltie 自己的接收状态当真值；原生组要认真配置。~~两组共用同一套 CI~~（第七轮修订：两组拥有等价的本地工作区条件、验收标准、命令、环境与权限）。

## 4. 讨论中纠正的错误

讨论过程中我方的以下说法被指出有误，已更正，本 package 不沿用：

- 博客日期写错；“Codex 默认并发 3”没有文档依据（文档写的是未设置时由 Codex 决定）；
- Spark to Fire 的指标是感染率，不是失败率；
- 以为 C002 遗漏了 `USER` 身份问题，实际 N04 已记录；
- 把 GF-17 的限制解读为“引擎不执行任何脚本”，实际它只约束 `workbook add` 的校验；
- 说新方向“与宪章高度兼容”过于乐观，实际涉及 §5、§7、INV-6 和 GF-12 的措辞调整；
- 把证据排成高低等级阶梯；
- 示例的写集包含 `tests/`，测试可被篡改；
- 宿主加固方案自相矛盾（禁写 Store 会挡住 CLI 自己）；
- 执行器选择在 CI 和本地 runner 之间反复：第六轮用户选定 CI，第七轮根据 B 的意见和个人仓库下 CI 信任前提不成立的分析，由用户确认改为本地优先；
- 把 METR 的“43 倍”说成“看得到计分函数时投机多 43 倍”。原文比较的是 RE-Bench 与 HCAST 两套基准，可见计分函数只是作者列出的可能原因之一；
- 引用 Anthropic 的投机研究时没有说明限定：Hacker-Opus 是专门在可投机环境中训练的模型，网络攻击场景的工具结果是模拟生成的；
- 说单 agent 的流程、信息、上下文三种独立性“已挡住大部分偷懒和投机”，没有实测依据；
- 说“验收结果与执行者无关”，过强；改为 §8 的共同基础；
- 说可信执行身份“只能来自 Sheltie 驱动层”，过窄；外部受信平台也能提供，驱动层单独也不充分；
- 设计审查否决时写“同一候选不能换人重审”，这会让审查者额度耗尽时只能等待或制造无意义的新候选；
- 把 GF-23 引作“引擎不长出宿主专用逻辑”，实际 GF-23 是后置项“MCP 接口与多宿主”。

## 5. 被否决的方向

| 方向 | 否决原因 |
| --- | --- |
| 多 agent 调度作为核心价值 | 成本高；失败主要不来自调度；宿主已提供 |
| 隔离作为核心价值 | 单用户信任下做不出可信版本；隔离属于宿主和沙箱 |
| 受保护控制面（守护进程 + 真人认证） | 会把产品拉向安全平台，偏离用户价值；复杂度高 |
| 内核直接运行验收测试 | 内核会出现业务词汇（GF-01）并变胖；第七轮改为内核之外的本地快照验证器，见 §7 |
| v0.3 优先接入 CI | 个人仓库中 CI 的信任前提基本不成立，每轮返工还要推送和等待；后置到团队仓库需求出现时 |
| 20–30 个任务、四组对照的大实验 | 在摩擦未暴露之前成本过高；改为 3 + 5–8 两阶段 |
| 模型审查永久排除在接收条件之外 | 架构、需求完整性等条件难以全部写成测试，会把语义判断全推回给用户；改为合同冻结的三种审查作用 |
| 同一候选不能换人重审 | 与额度备选冲突；改为允许接替、发现不因替换消失 |
| 隐藏验收作为所有任务的默认防线 | 证据不足以支持；本地未隔离时也藏不住；改为可选模板 |

## 6. 参考来源

讨论中已核实：

- Claude 博客，Building multi-agent systems: when and how to use them（2026-01-23）：<https://claude.com/blog/building-multi-agent-systems-when-and-how-to-use-them>
- Anthropic，How we built our multi-agent research system：<https://www.anthropic.com/engineering/multi-agent-research-system>
- Claude Code Agent Teams：<https://code.claude.com/docs/en/agent-teams>
- Claude Managed Agents，Define outcomes：<https://platform.claude.com/docs/en/managed-agents/define-outcomes>
- OpenAI Agents SDK，Sandboxes：<https://developers.openai.com/api/docs/guides/agents/sandboxes>
- OpenAI Agents SDK，Orchestration：<https://developers.openai.com/api/docs/guides/agents/orchestration>
- Codex multi-agent：<https://developers.openai.com/codex/multi-agent/>
- MAST，Why Do Multi-Agent LLM Systems Fail?：<https://arxiv.org/abs/2503.13657>
- From Spark to Fire：<https://arxiv.org/abs/2603.04474>

由讨论者提供、采用前需复核：

- GitHub 分支保护与必需检查（来源限定、skipped/neutral 满足必需检查的行为）：<https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches>
- OWASP Transaction Authorization Cheat Sheet：<https://cheatsheetseries.owasp.org/cheatsheets/Transaction_Authorization_Cheat_Sheet.html>

由讨论者在第七轮提供、采用前需复核：

- Cargo `--offline` 与 `--frozen`：<https://doc.rust-lang.org/cargo/commands/cargo-build.html>
- `gh run view`：<https://cli.github.com/manual/gh_run_view>
- `git worktree`（linked worktree 共享仓库数据与配置）：<https://git-scm.com/docs/git-worktree>
- Codex 的 worktree 与本地环境 setup scripts：讨论者提供，未给出可核对链接

第八至十一轮已核实：

- METR，Recent Frontier Models Are Reward Hacking（2025-06-05）：<https://metr.org/blog/2025-06-05-recent-reward-hacking/>
- Anthropic Alignment Science，Training a Misaligned Reward Seeker：<https://alignment.anthropic.com/2026/reward-seeker/>

由讨论者在第八至十一轮提供、采用前需复核：

- GitHub Actions OIDC：<https://docs.github.com/en/actions/security-for-github-actions/security-hardening-your-deployments/about-security-hardening-with-openid-connect>
- SLSA v1.0：<https://slsa.dev/spec/v1.0/>

## 7. 第七轮：执行器改为本地优先

B 在第七轮建议 v0.3 优先采用本地验证、CI 接入后置，用户确认采纳，首批用户确认为个人开发者的本地代码库。

**采纳的理由：**

1. **个人仓库中 CI 的可信度并不高于本地验证。** agent 使用用户本人的 git 和 `gh` 凭据，能改 workflow 文件；CODEOWNERS 的审批人就是用户自己；token 权限足够时甚至能改分支保护。CI 的来源限定优势只在团队仓库成立。
2. **可信度相同时，本地验证摩擦更低。** 不需要为每轮返工推送远端、等待排队，这正好落在返工闭环最敏感的环节。
3. **证据接收合同不绑定执行器。** 改方向只影响验证器和 ADR-A 的候选排序，不影响 VD-01 到 VD-07 的主体，以后接 CI 不用返工。

**采纳的 B 的具体意见：** “不需要网络和凭据”要附条件；接 CI 不要求引擎保存凭据；验证器按固定输入、执行检查、记录结果、结束与恢复四个环节定义；只冻结命令字符串不算冻结检查；检查失败、命令未启动、超时分开记录；worktree 不等于隔离；对照实验改为本地条件对等，原生组可用 worktree 和 setup scripts；前 3 个任务覆盖返工和中断接续；两种诊断；C002 先于 C004 代码；首页范围约束句。

**本方补充的边界（B 未提及）：**

- 在用户仓库里建 worktree 或替 agent 提交 commit 会违反 INV-3，所以临时工作区建在 `~/.sheltie/tmp`，只接受已存在的 commit；
- 验收命令的副作用（构建缓存、依赖缓存、项目代码的外部访问）需要在宪章中补充 INV-3 的解释，不能默认成立；
- worktree、commit 是 Git 词汇，验证器必须在内核之外（GF-01）；
- 记录规则版本只能发现篡改，挡不住；规则必须取自冻结合同或基线，候选改动受保护路径即判 `failed`；
- 不用 `git archive` 导出快照，因为它受 agent 可修改的 `.gitattributes` 影响；
- 超时要终止整个进程组；冷构建耗时是阶段 1 的重点测量项；
- 阶段 1 不先造验证器，用外部脚本实现，证据标为“外部回填”。

**因此撤回或修订的结论：** §3 第 11、13、14 条；承诺表第二行在本地路径下改为“不承诺”。

## 8. 第八至十一轮：独立审查与执行身份

用户补充了最初的需求：严格任务中，模型可能偷懒甚至作恶，希望由独立 agent 协调流程和审查；必须支持全程一个 agent；简单部分交给便宜模型；订阅有 5 小时和每周额度，需要备选 agent。讨论据此把问题拆成两份提案：本 package 负责“凭什么接收”，C005 负责“由谁继续”。

**共同基础：** 更换执行者，不得自动放宽任务标准、权限范围和证据要求；最终接收依据来自预先授权的验证机制，不能仅凭执行者自报。执行者可以替换，已登记的审查发现不因替换而消失。

**收敛的结论（写入 spec.md VD-09 到 VD-11）：**

1. 协调者看到“不通过”的审查报告后仍可选择继续，这对严格任务是真实缺口。审查作用在合同中三选一并冻结：提供建议、构成必要条件、触发升级。
2. 否决必须同时处理来源、误判和解除：未完成不等于通过；审查者可以接替；发现不因替换或新候选而消失；误判由用户按三种处置类型显式处理，原报告不删除。
3. 这需要正式修订 INV-1（解释）、INV-5 与 INV-6，不能靠改名规避。INV-6 的修订原则是区分报告事实、来源事实与内容判断。
4. 执行身份分声明、观测、强制三栏。严格合同缺观测证据时条件未满足，不能标“自报”后照样放行。
5. 协调者可以推进状态、提交带来源的声明，但不能把自己的内容标成验证器观测；“验证器取得”只能由验证器命令自己运行检查产生。单用户下这仍是接口约束。
6. 保留样本对应公开要求、不作为任务输入交出；本地未隔离时不保证保密；是否减少过度适配是实验问题。
7. 研究证据足以把投机纳入威胁模型，不足以证明日常任务风险单调上升；隐藏验收只作为可选模板和实验变量。

**先测量再实现：** VD-09 的价值取决于协调者是否真的会无视审查、误判阻断的代价有多大。阶段 1 记录这两项，结果不支持时，VD-09 推迟，只保留 VD-10。
