# MVP 独立综合审查

结论：**需修改**。产品定位与三个 crate 的基本分工成立；当前实现不能完整兑现请求重放、文件边界、冻结定义和崩溃恢复的承诺。v0.1.0 已发布且 MVP 已被项目接受，这是历史事实；本报告评价其当前缺陷与下一版方案，不重新授予或撤销历史验收。

日期：2026-09-27。代码候选：`a664e75a3ab3041d09cd0a1ab4d69336f2dcd055`。开始时工作区干净。主审 Codex；独立 Spec 轴 `spec_review`，Standards 轴 `standards_review`。审查者均未参与被审实现。范围是当前完整 MVP，不是 `v0.1.0...HEAD` 的文档差异。

## 1. 审查方法与覆盖

先阅读当前规格、合同、实现与测试，固定独立发现并运行真实 CLI 探针，再读取 C002 的六份旧材料。主审的前置记录保存在 [independent-before-c002.md](evidence/2026-09-27/independent-before-c002.md)。两个审查轴也先提交独立发现，再核对旧方案。

| 范围 | 阅读与验证 |
| --- | --- |
| 产品与规则 | `CONTEXT.md`、宪章、规格、架构、三份合同、路线图、工程规范、文档地图 |
| core | 全部生产模块、单元测试、testkit、集成测试与状态卡快照；解析、编译、状态转换、输入绑定、统计与渲染 |
| runtime | 全部生产模块及 6 份顶层集成测试与 common；Store、Service、Workbook、文件观察、自管理与故障点 |
| CLI | 全部生产模块及 11 份顶层集成测试与 common；命令参数、错误映射、响应、生命周期场景 |
| Workbook 与 skill | 三份样例；spec-dev 的全部 Flow、说明、模板、清单；sheltie skill 与安装方法 |
| 工程交付 | Cargo manifests/lock/toolchain/deny、CI/release、pre-commit、全部 scripts；C001/C003 治理入口、计划、审查和证据；C002 在独立阶段之后读取 |
| 动态证据 | 当前候选构建、315 个既有测试、隔离 CLI/SQLite/Git/浅克隆反例；详见 [validation.md](validation.md) |

未重做真实 Claude Code/Codex 宿主会话、四平台发布、联网安装升级、全量突变或断电测试。原始宿主 transcript 未取得；历史叙述不能被本次离线测试升级为直接观测。

## 2. 产品价值与定位

Sheltie 最清楚的定位是：**给现有协调者使用的本地、可恢复工作流程与产物记录工具**。用户已经有 Claude Code 或 Codex，已经能调用工具与委派工作；仍需稳定记录做到哪、哪份产物来自哪次尝试、下一步允许做什么。Workbook 把可重复的方法独立于模型和会话保存，SQLite 保存执行事实。这个需求真实且长期存在。

价值集中在三处：跨会话继续工作；重复使用一套方法；让每次尝试、输入摘要、输出和批准记录可以追溯。完成与结论分开、引擎不读自然语言裁决、有限显式边与访问上限，都有清楚的职责边界。OpenAI 对编排区分集中协调与控制权交接，Anthropic 建议按任务特点选择确定流程或动态 agent，并从简单方案开始。这些建议支持当前“协调者判断、代码守机械规则”的分工；并不要求每一步都创建新 agent。[OpenAI 编排](https://developers.openai.com/api/docs/guides/agents/orchestration)、[Anthropic 工作流与 agent](https://www.anthropic.com/engineering/building-effective-agents)。

| 场景 | 适配度与原因 |
| --- | --- |
| 多轮文稿、研究资料整理、规范化审查 | 高：步骤与文件合同稳定，需要打回、复查、跨会话续接 |
| 有明确任务边界的软件开发 | 有条件：spec-dev 必须修好任务基线、人工条件与最终整体审查；代码仓库中的副作用仍由宿主控制 |
| 简单一次性问答或几分钟小修改 | 低：编写 Workbook、begin/submit 和额外 agent 的成本可能超过收益 |
| 开放式探索、步骤不可预知的研究 | 部分适配：在较粗节点内保留探索空间；不宜为了图完整而制造大量小节点 |
| 强隔离、高风险审批、多用户合规 | 当前不适配：同一个 OS 用户可调 approve、改文件和数据库，现有 gate 不提供独立认证或宿主工具隔离 |

主要代价是 Workbook 编写与维护成本、重复读文件的 token、步骤间交接延迟和固定图的僵硬性。路径引用减少了协调者重复拼接正文，但工作 agent 读文件仍消耗上下文；状态卡有长度上界也不等于足够紧凑。没有同任务、同模型、同质量标准的对照运行，不能声称已经节省 token 或提高正确率。

## 3. Agent 接口与信任边界

CLI 的操作词汇较少，`next`、任务书和文件输出适合作为 agent 的接口。但接口应让协调者第一次就能正确调用：show 必须公开起始输入；JSON 必须带失败原因、完整产物引用和一致的 next；安装后的 skill 必须能独立使用；重试必须返回可解释且稳定的结果。Anthropic 对工具设计强调有用上下文、清楚错误与真实任务评估，而不是单纯增加工具数量。[工具设计](https://www.anthropic.com/engineering/writing-tools-for-agents)。

MVP 能强制“必须先有批准命令才能沿出边推进”，不能证明“真人确实阅读并批准”。当前 `USER` 环境变量甚至不能可靠代表 OS 主体。建议下一版用实际 OS 身份记账，同时把单用户协作信任边界写清；是否自动调用 approve 仍须由宿主授权机制与 skill 约束。不要为掩盖这条边界仓促加一套自制认证系统。OpenAI 同样把自动校验与人工批准分别处理；这不是对本地用户名认证强度的背书。[人工审查与 guardrails](https://developers.openai.com/api/docs/guides/agents/guardrails-approvals)。

Workbook 的自然语言、resource 与产物均可能影响模型判断；哈希只能证明字节身份，不能证明内容可信。Sheltie 不运行节点工具，因此不能阻止节点内的外部副作用。C002 应修正承诺和宿主回归，GF-22 真正实施时才增加可验证的隔离能力。

## 4. Rust 架构与实现质量

保留 `cli → runtime → core`、同步执行、SQLite 单一状态、闭集命令和 `thiserror`。纯 core 让图与状态规则可用独立输入验证；`Graph` 私有构造、ID/path/digest newtype、禁止 unsafe、显式 Result 都是合适选择。spec-dev 中“一次预写全部测试、后续只填空、审查只读 diff”的效率假设也没有被当前离线图测试证明。任务验证必须能补看受影响 caller 与语义上下文，不能把骨架限制当作产品效果证据。

没有证据支持增加异步运行时、依赖注入框架、trait 仓库层、通用事件总线或第二套状态机。

缺陷集中在边界组合：核心规则正确，不等于 I/O 时机、事务、响应和恢复也正确。runtime 的用户意图、即时观察和持久响应被混在 Command 序列化里；Store 与文件发布之间缺少完整生命周期；CLI 提交后再读状态拼回复；同一路径被多处手工构造。这些应通过有具体职责的小模块和真实调用链修复，不能只补几个局部 if。

类型封装也没有达到文档所说的“非法状态不可表示”：Flow/Manifest 可公开修改和直接反序列化，WorkState 的状态与字段组合由普通结构体表达。建议收紧已校验定义的构造面，持久状态在装入时校验必要的一致性；不必对每个字符串新增类型或把整个状态机改成复杂 typestate。Rust 官方建议用类型保护真实约束，用 Result 报告可恢复外部错误。[类型安全](https://rust-lang.github.io/api-guidelines/type-safety.html)、[可靠性](https://rust-lang.github.io/api-guidelines/dependability.html)、[错误处理](https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html)。

其他改进随对应修复完成：去掉重复 load/observe，状态和 next 从同一快照产生；大文件先限额再流式读；SQLite 错误不要全部叫数据损坏；删过期骨架注释、无真实调用者的 public helper 和无必要的 dead_code 放行。现有文件行数包含大量测试，不应仅按行数拆模块。MSRV 1.85 声明应实际测试；stable 通过不能证明最低版本通过。[Cargo MSRV](https://doc.rust-lang.org/cargo/reference/rust-version.html)。

## 5. Standards 轴

独立轴的主要结论是：路径写入违反管理根约束；目录摘要缺少独立 oracle；Workbook 发布与重放没有守住持久规则；部分并发测试实际串行；发布与文档检查未形成可执行的 CI 闭环。对应 [findings.md](findings.md) 的 O01、O03、O06–O08、N02–N06、N11–N14。代码风格工具全部通过，不能替代这些行为证据。

C003 的当前文档/历史分层合理，本地结构检查通过；但是其继承的 `check-specs` 与浅克隆 CI 配置组合失败。完成记录不等于 CI 的真实运行已经被本次确认。应修检查器的运行前提和 active/RC 版本模型，不撤回文档组织方向。

## 6. Spec 轴

独立轴确认 spec-dev 的单任务检查、人工条件交接、升级返回、重规划后最终审查存在缺口；引擎输出与输入文件可能碰撞；JSON 状态卡和 stats 不能完整表达约定事实。对应 O09–O13、N07–N09。主审补证 O02、O04、O05、N01 等请求与恢复缺陷。

因此“主要 happy path 已实现”成立；“所有 MVP 合同完整实现”不成立。完整问题、严重度、依据、反例和修复任务只在 [findings.md](findings.md) 维护。

## 7. 对旧 C002 的评价

原 O01–O13 均有事实基础，本次没有发现需要整体撤销的条目。O12 的表述需要准确：exact `brief.md` 已拒绝，漏的是 stats、祖先冲突和平台别名等条件。原 S/P 条目更多是在评价 T26 历史归因，不能与当前产品缺陷重复计数。原文可用 `git show a664e75:specs/changes/proposed/C002-v0.2.0-reliability/findings.md` 重建；其中真实用户目录和 prompt 的叙述本次未重新核验。

| 旧方案 | 本次处理与理由 |
| --- | --- |
| StartRequirements、目标绑定 RequestIntent、提交时响应 | 保留；对应已复现的真实缺陷 |
| WorkLayout、engine/outputs 分离 | 保留集中路径与职责隔离；取消双布局常驻分支，目录标签属于一次性新布局选择 |
| LegacyV1、默认缺字段兼容、不升 schema 混合 requests | 删除默认兼容方案；会增加双解释路径，且不能解决新摘要格式。改为新 Store 版本拒绝旧格式，旧数据原样保留 |
| 把双 SHA256 固化成 digest/v1 | 撤销；无内容长度的编码已经有确定性碰撞，必须改算法输入格式 |
| exact bytes 重放全部文件 | 修正：brief/stats 是历史事实，status-card 是当前投影；后者不能回退到旧 revision |
| committed start 放可随时清理的 tmp | 修正为 Store 拥有的 pending 文件；完成发布前不可清理 |
| Workbook DurableEffect | 补事务归属、完成标记、删除/重加和崩溃竞争；仅命名一个效果类型不能证明恢复 |
| 修改只读根后认定 Finder 不再破坏 | 保留降低误写的用途；chmod 不是同用户防篡改认证，平台上须实际验证 |
| spec-dev 四项修改 | 扩展到固定整体基线、每任务基线、未来占位体、人工条件有效版本及完整返回路径 |
| CI/MSRV/Host 后验收 | 保留并补浅克隆、未发布版本检查、发布质量依赖；真实宿主结果独立记录 |

## 8. 迭代顺序与取舍

先关闭 C002 中已复现的数据、边界、请求与恢复问题，再修协调者可用性和 Workbook 完整流程，最后用真实宿主验收。首个价值实验只需固定少量代表任务，比较直接 agent、skill、skill + Sheltie；记录交付质量、违规推进、恢复成功率、人工干预、耗时和宿主实测 usage。重复采样后再决定更复杂的编排是否值得。OpenAI 建议先从轨迹定位行为问题，再用可重复数据集做回归；本项目可先用现有文件与脚本完成，无需建设评估平台。[Agent evals](https://developers.openai.com/api/docs/guides/agent-evals)。

MCP 在 CLI 摩擦被真实观测时增加；安装器在真实资源声明需求出现后设计；并行、动态模板、沙箱和多用户认证各自立项。路线图里 L1–L3 是假设，不应成为当前工程任务。没有实证，不因模型能力想象提前建设“清算平面”。

对用户提出的五条原则，本次采用的判断是：每项新机制都必须对应一个已出现的失败、一个当前合同或一个可验证的近期用例。修复请求重放、持久发布和路径边界属于兑现现有目标；为了浏览改名保留永久双格式、为假想宿主建立适配体系、用兼容分支掩盖错误摘要，都不值得。削减的是没有收益证据的复杂度，同时保留数据完整性所需的检查。

## 9. M1 预审与修复材料（2026-09-28）

候选e1a8126的[独立审查](review-m1-2026-09-28.md)为“需修改”，不改变本文前面的MVP历史结论。R01–R19的修复方案和新人执行材料见[repair-design](repair-design.md)、[repair-plan](repair-plan.md)、[repair-validation](repair-validation.md)。两位未参与材料撰写的Reviewer复核后给出“方案材料通过”，具体初稿问题与修订见[材料证据](evidence/repair-planning-2026-09-28/README.md)。该结论不表示代码、API探针、M1、Host或发布通过；实际进度仍只看plan.md。

T18合同/API探针delta经独立Standards Reviewer和Spec Reviewer复核通过。复核范围是用户采用后的合同文档和macOS arm64探针：恢复了pending owner精确格式与self update/schema边界；保留self install替换分叉二进制的既有行为；区分正常purge沿同一根锁与锁外意外替换；按bundled SQLite实测收窄短main/有效WAL与坏WAL表述；补充`@file`错误边界、RequestIntent名字顺序与SHM例外；T18验证操作职责/API，T19确定私有签名并迁移caller。Linux与真实runtime caller未由Reviewer验证；用户明确豁免Linux，Linux仍`not_run`。
