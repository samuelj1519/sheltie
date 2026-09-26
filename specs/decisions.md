# 设计决定记录

每一条记录一个有争议或有替代方案的设计决定：背景、选择、被否决的方案、后果。新决定往后追加，不改已有条目的原文；被推翻的决定加一行「被 D-nn 取代」。规格与合同只写结论，理由都在这里。

格式：`D-nn 标题`。日期为拍板日期。

## D-01 边显式声明，合法下一步就是出边

2026-09-24。

**背景。** 协调者要在「主干、打回、修复旁支、再审」之间选路。图里必须能写出这些边，引擎必须能机械判断哪些边合法。

**选择。** Flow 用 `[[edges]]` 表显式声明 `from / to / kind`，`kind` 取 `main | back | branch | re_review`。合法下一步等于当前节点的全部出边，再按 `max_visits` 过滤。没有隐式「往下走」。

**否决。** 用 `depends_on` 表达 DAG 加 `routes` 表达条件路由。条件路由需要一套表达式语言与结构化结果字段，而自然语言结论的节点用不上它；回边在这种形式下无法直接写出。

**后果。** 图可以有环，靠 `max_visits` 保证有限。`kind` 对引擎只是标签，四种边的合法性判断相同。机器路由若将来需要，以显式节点类型另立项（[路线图](roadmap.md)）。

## D-02 协调者选边，引擎不读内容

2026-09-24。

**背景。** 审查节点的结论「通过 / 不通过」决定走哪条边。谁来读这个结论。

**选择。** 结论是工作 agent 写进输出文档的自然语言，协调者读后在 `next` 里选一条。引擎只保证选的那条是声明过的边。`attempt begin <下一节点>` 这次调用就是选边，不另设推进操作。

**否决。** 引擎解析文档里的关键词自动推进。这违反 [宪章](constitution.md) `INV-1`、`INV-2`，且把业务判断锁死在引擎里。

**后果。** 引擎对任何业务领域零知识；换一个 Workbook 不改引擎。协调者选边不需说明理由，不被二次盘问。

## D-03 十一个 CLI 动词，每次响应带 `next`

2026-09-24。

**背景。** 协调者与引擎的接口要小到 skill 一页纸能教完，又要覆盖领取、执行、提交、门槛、取消。

**选择。** `workbook add|list|show|remove|verify`、`work start|list|status|cancel`、`attempt begin|submit|fail`、`gate approve`，加 `self` 组五条管二进制自身。领取与准备合并为 `attempt begin`，返回任务书。每个响应带可直接执行的 `next`。

**否决。** 把领取、准备调用、读输入、开始、提交拆成独立操作。对本地文件系统上的单用户工具，拆成这些分步没有独立价值，只会增加协调者出错的机会。

**后果。** MCP 接口若将来需要，只是同一组操作的薄封装。工具描述短，符合 Anthropic 关于工具接口的建议。

## D-04 `gate` 与 `executor` 正交

2026-09-24。

**选择。** `executor = human` 只表示谁干活，人的提交是一次普通 Attempt。`gate = true` 只表示 Attempt 成功后要 `gate approve` 才能离开节点。两者互不推断。

**否决。** 由 `executor = human` 推断门槛。人审节点不需要再批准一次自己的结论，而 agent 节点需要人把关时又没法表达。

**后果。** 人点头只出现在两处：人审节点的提交、门槛批准。其余不打扰人（[宪章 §5](constitution.md)）。

## D-05 整份状态存一列 JSON，乐观并发

2026-09-24。

**背景。** Work 状态怎么持久化。

**选择。** `works.state_json` 存整份 `WorkState`，`revision` 做 CAS，`audit` 表另记每个 Command，`requests` 表做重放。`decide` 是纯函数，在事务外算好，事务内只比较 `revision` 与写入。

**否决。** 事件溯源加 reducer；按实体编码的类型化变更集。两者都为「多 Work 并发、大规模审计回放」设计，而这里一次调用只改一个很小的 Work。

**后果。** 恢复只需读一行；崩溃语义只需分析 `COMMIT` 前后两个窗口；效果全部幂等。代价是每次写整份状态，对几 KB 的对象可忽略。

## D-06 `work_id` 用日期序号加名字

2026-09-24，用户决定。

**选择。** `work_id = <UTC 日期>-<当日序号 001..999>-<名字>`，目录名等于 `work_id`。序号在独立短事务里按日期递增，分配后不回收。身份以 SQLite 行为准，目录名只是投影。

**否决。** UUID v7。对人不可读，`work list` 与目录浏览体验差。项目定位为单机单进程，事务内分配序号的成本可接受。

**后果。** `start` 比其他写操作多一个前置小事务；失败会留空号。前缀查找要处理歧义。

## D-07 `INV-3` 的边界是宿主，不是「安装」这个词

2026-09-24。

**背景。** 引擎不做安装，那 `workbook add` 把目录复制进 `~/.sheltie/workbooks/` 算不算安装。

**选择。** `INV-3` 防的是写宿主配置、把资源装进 agent。引擎写自己的管理根不受此限。于是 Workbook 入库、二进制自更新都由 `sheltie` 自己做。

**否决。** 一切安装动作都归独立安装工具。这让 MVP 装一份 Workbook 都得先做第二个二进制，而它防的风险在 MVP 里根本不存在。

**后果。** 宿主资源安装（skill、subagent 进 Claude Code）仍然归独立工具，见 [路线图 GF-20](roadmap.md)。

## D-08 一个二进制，`self` 组管自己

2026-09-24，用户同意。

**选择。** `sheltie self install|update|rollback|uninstall|version`。发布用 `cargo-dist`，自更新用 `axoupdater` 读同一份发布清单。替换二进制走「下载到 `tmp/`、核对 sha256、挪 `.prev`、`rename` 到位」，保留一级回滚。默认不写 shell 配置。

**否决。** 两个二进制（要先解决谁装安装器、两处版本如何保持一致）；不做自更新（单机用户升级与回滚体验差）。

## D-09 每个 Work 持有 Workbook 的冻结副本

2026-09-24。

**背景。** `workbook remove` 与升级不能影响运行中的 Work；有人手改已装 Workbook 也不能影响。

**选择。** `work start` 把 Workbook 目录复制到 `works/<work_id>/workbook/`，之后该 Work 的一切加载只读副本。`remove` 只拦非终态引用作为安全网。

**否决。** 用引用计数阻止删除；或在删除时把副本迁给 Work。前者让终态 Work 永久锁住旧版本，后者把复制推迟到最不该出错的时刻。

**后果。** 每个 Work 多几十 KB。`GF-05` 的「冻结定义」从一句话变成一份文件。

## D-10 资源分两层：文件绑输入，宿主机制才声明

2026-09-24，用户同意。

**背景。** Workbook 需要 skill、subagent、reference 配合。全部打包会在多个 Workbook 之间、与宿主已有资源之间重复。

**选择。** 能让工作 agent「读文件」解决的放 `resources/`，用 `resource.<path>` 绑成节点输入，随版本冻结。只有需要宿主机制的（skill 自动触发或工具授权、命名 subagent、MCP）才在 `requires` 里按 `kind + name` 声明；引擎把声明列进任务书，不检查、不安装。同名同类在任何 Workbook 与宿主里指同一资源，这是安装工具去重的依据；`digest` 不同拒绝后装，不覆盖用户自己的资源，宿主落点是一个 Sheltie 托管 plugin。

**否决。** 把所有资源打包进 Workbook 并装进宿主。重复、撞名、且大部分资源根本不需要宿主机制。

**依据。** Anthropic Agent Skills 文档：skill 正文在触发后才从文件系统读进上下文。一份绑好路径的参考文件效果相同，只少了自动触发。

## D-11 执行失败与负面结论是两回事

2026-09-24。

**选择。** Attempt 只有 `running | succeeded | failed`。「审查完了，结论是不通过」是 `succeeded`，走边由协调者选。「崩了、超时、没交文件」是 `failed`，在 `max_retries` 内重试，耗尽则 `blocked(retries_exhausted)`，唯一出路 `work cancel`。

**否决。** 失败后也允许走修复旁支。旁支边是从成功节点出发的，失败节点没有可信的输出可供旁支消费。

## D-12 两个上限，不多不少

2026-09-24。

**选择。** `max_visits`（节点到达次数，含回环）与 `max_retries`（同一次到达内的失败重试）。到顶就从 `next` 里拿掉对应项；所有出边目标都到顶时 `blocked(no_legal_edge)`。

**否决。** 再加一个「修订次数」上限。它与 `max_visits` 语义交叠，而修订本来就是通过回边再次到达。

## D-13 对照 OpenAI 与 Anthropic 官方建议

2026-09-24。依据：Anthropic《Building effective agents》《Writing effective tools for AI agents》、Agent Skills 文档；OpenAI《A practical guide to building agents》、Agents SDK human-in-the-loop 文档。

| 建议 | 落点 |
| --- | --- |
| 先找最简单的方案；workflow 比 agent 可预测 | Sheltie 是 workflow 层，图由人写死，agent 只在每一步内部自由。MVP 不做动态模板、并行草稿、机器路由 |
| 工具少而清楚，返回高信号信息，大输出分页或拒绝 | 十一个动词；`next` 是可执行命令行；响应只回路径与摘要；摘要超限拒绝而不截断 |
| 让 agent 的计划可见 | 状态卡的 `done / current / next` |
| 人工介入只在高风险动作与失败超阈值 | `gate` 与 `retries_exhausted`，其余不打扰人 |
| 审批是持久状态上的暂停与恢复 | `blocked(gate)` 是持久状态，`gate approve` 从原状态继续 |
| 文件系统做外部记忆，按需读取 | `T-2` 路径引用、`T-3` 状态卡、`T-4` 有界摘要 |
| Skill 用 `SKILL.md` 与渐进披露 | sheltie skill 只教命令与循环，细节链接合同 |
| 幂等、可重试的工具 | 每个写操作带 `request_id` |

## D-14 Rust 工程取舍

2026-09-24。

| 取舍 | 选择 | 理由 |
| --- | --- | --- |
| crate 数 | 三个：core、runtime、cli | 每个都有真实职责；宿主客户端与安装器等立项再加 |
| core 的外部依赖 | 纯函数接收「已观察事实」作参数，不用 trait 抽象 I/O | 没有 mock 也能测每条规则；runtime 只测事务与文件 |
| 错误集 | 23 个，每个带「已发生什么、下一步」 | 协调者能从错误直接决定动作 |
| 并发模型 | 同步，无 `tokio` | 单用户 CLI，一次调用一个事务 |
| 存储 | `rusqlite` bundled，WAL，`synchronous = FULL` | 单文件、零运维、默认参数即最佳实践 |

## D-15 MVP 不做 MCP，只支持单人

2026-09-24，用户决定。

**选择。** Claude Code 与 Codex 都能跑 CLI，skill 教会协调者调 CLI 即可；MCP 是体验优化，见 [路线图 GF-23](roadmap.md)。`gate approve` 记操作系统用户名，够单人本地用；需要「独立真人」时的认证能力见路线图。

## D-16 输入可以声明为可选

2026-09-24。

**背景。** 回环里被打回的节点需要读审核意见，但第一次到达时还没有意见。若所有输入都必需，这种节点写不出来。

**选择。** `inputs[].required = false`，只允许 `<node>.<output>` 来源。上游没有成功 Attempt 时不绑定，任务书标「尚无」。上游有了就绑最新一次。

**否决。** 让协调者在追加上下文里手工把意见贴进任务书。这把一件机械事推给协调者，弱协调者会漏。

**后果。** `Attempt.inputs` 的值类型变成 `Option<ArtifactRef>`；`start` 与 `resource` 来源上写 `required = false` 编译拒绝。

## D-17 `spec-dev` Workbook 的结构

2026-09-24。

**背景。** 需要一份可分发的软件开发 Workbook，人少插手、弱模型能跑、token 省。

**选择。** 八个节点。规划三步（`spec`、`plan`、`plan-review`），其中只有 `plan-review` 是人，用人审节点而不是 `gate`，因为人的结论要能把流程送回两个不同的上游。实现两步回环（`implement` 一次一个任务，`verify` 新会话独立跑门禁），任务数由 `verify` 报告第一行推进，靠 `max_visits` 限界。收尾两步（`review` 看整体 diff 一次，`deliver`）。`fix` 节点同时服务验证与审查两个回环，只修报告里第一行为「不通过」的那份。每份报告第一行是协调者的选边协议。门禁命令由 `plan` 从项目里抄进方案，后续每步照跑。

**否决。** 每个任务都审查一遍（token 翻倍，验证已经保证任务完成）；`gate = true` 代替人审节点（只能批准或取消，不能送回具体上游）；把整份任务清单交给一次 `implement`（弱模型在大任务上失败率高，失败时丢全部进度）。

**后果。** 一个 N 任务的需求要跑 `implement` 与 `verify` 各 N 次，每次上下文只有一条任务与几个文件路径。`max_visits` 24 与 32 限制了单个 Work 的规模；更大的需求在 `spec` 阶段拆。

## D-18 `deliver` 带门槛，CI 留在流程外

2026-09-24，用户同意。

**背景。** 最后一步由谁验收：人看一眼，还是交给 pre-commit、lint、GitHub Actions。

**选择。** `deliver` 节点 `gate = true`。人读 `delivery.md`，自己运行里面列出的对外动作（push、开 PR），再 `gate approve`。流程内不 push、不发布。CI 是 push 后的冗余确认，留在流程外；`plan` 把 CI 命令抄进「门禁」，流程内每个任务已跑过三遍同样的检查。

**否决。** 不设门槛、靠 CI 验收。CI 只能验机器可判的性质，验不了「做的是不是想要的」，且只在 push 之后才跑，而 push 本身是不可逆的对外动作，按 OpenAI 与 Anthropic 的建议应有人放行。

**后果。** 人在流程里出现两次：开头审方向，结尾接收并放行。中间全自动。

## D-19 卡住走人审旁支，任务书带「来自」行

2026-09-24，用户提出，采纳并扩展。

**背景。** 工作 agent 会遇到三类靠自己解决不了的事：缺只有人知道的信息、要做需要授权的高风险动作、同一任务反复修不过。`max_retries` 与 `max_visits` 耗尽只给 `work cancel`，会丢掉全部进度。

**选择。** `spec-dev` 加一个 `executor = human` 的 `escalate` 节点，`implement`、`fix`、`verify` 都有 `branch` 边进去；人写四个词之一（`继续 / 跳过 / 改方案 / 止损`）加意见，四条出边分别回 `implement` 或 `fix`、`implement`、`plan`、`deliver`。「修两轮不过」由报告里手递手传的 `修复轮次` 数字触发，`verify` 在轮次到 2 时写 `不通过，需要人`。「需要授权」由工作 agent 自报 `卡住` 并写清需要什么，人在决定里写授权范围，这段话原样进任务书，给下一步当输入。

同时给引擎的任务书加「来自」行（上游 Occurrence 与边类型），`Attempt` 记 `entered_from`。原因：可选输入绑定后会一直带着上一次的内容，多入口节点必须知道自己是从哪条边来的才能决定读哪份。

**否决。** 靠引擎的 `blocked` 加 `cancel`（丢进度）；让协调者在聊天里问用户（不落文件，弱协调者会即兴处理）；用 `gate` 代替（只能批准或取消，不能带意见分四路）；由引擎在 `max_visits` 到顶时自动转人（引擎不知道该转到哪个节点，且违反「引擎不选路」）。

**越权。** 宿主层的工具权限提示（Claude Code 的 allow / deny）仍然是第一道拦截。Workbook 里的 `卡住` 约定处理的是任务范围外的动作，两者互补。引擎级的权限求值见 [路线图 GF-22](roadmap.md)。

**后果。** `spec-dev` 九节点二十边。人在流程里最多出现三次，其中 `escalate` 只在异常时出现。

## D-20 强模型搭骨架，初级实现者填空，机器当审查

2026-09-24。

**背景。** 实现者可能是首次接触项目的初级开发者或初级模型。要在低 token、少返工、高质量三者之间取综合成本最低。

**选择。** T01 由强模型一次性写出全部类型、签名、文档注释、`todo!()` 函数体、全部测试（禁用）、快照、脚本与样例。之后每个任务是「启用一组测试，填几个函数体，让它们变绿」。实现者不做设计、不写测试、不改签名。逐任务审查交给编译器、clippy、测试和 `scripts/check-task.sh`（改动文件白名单、无残留 `todo!()`、测试与快照未改）。模型审查只在三个里程碑做一次，配 `cargo mutants` 找没被测到的逻辑。

**为什么有效。** 初级实现者失败的地方集中在四处：发明类型与接口、读大量上下文、判断自己做完没有、处理事务顺序这类细微语义。骨架把前两项做掉，测试把第三项做掉，Rust 的类型系统与穷尽 `match` 把第四项的大部分交给编译器。剩下的工作是「给定签名、注释、失败的测试，写函数体」，这正是初级模型最稳的场景。每个任务的输入上下文压到 3k 到 8k token。

**否决。** 只把任务卡写得更细（散文再细也替代不了签名，而且比签名更长）；每个任务都做模型审查（贵，且审查者同样初级）；让实现者自己写测试（初级模型写的测试常常只验证自己的实现）；强弱模型逐任务配对（token 翻倍）。

**代价。** T01 是一次大投入，要求骨架作者对合同理解完整；骨架错了会连累多个任务。缓解：T01 的验收包括「`scripts/task.sh T02` 能跑出红」，M1 在 core 完成后立即做，错误不会拖到 runtime。

**后果。** `plan.md` 的每张任务卡从「先写的测试」改为「要变绿的测试」，多了「文件」白名单与十条实现者规则；新增 `tasks.toml`、`scripts/task.sh`、`scripts/check-task.sh`；新增 M1 到 M3 三个里程碑。

## D-21 `spec-dev` 采用同一套「骨架、填空、机器审查」；节点带 `tier` 标签

2026-09-24，用户提出，采纳。

**背景。** D-20 的做法只写在 `plan.md` 里，服务本仓库的开发。`spec-dev` Workbook 面向任意项目，实现者同样可能是初级模型，同样的问题会出现。

**选择。** `spec-dev` 在 `plan-review` 之后加 `scaffold` 节点（强模型）：把方案与任务清单变成一次骨架提交，全部类型、签名、注释、占位函数体、禁用的测试，并输出单任务测试命令。`implement` 与 `fix` 变成填空，附一份十条实现者规则当输入。`verify` 用 `git diff` 机械核对改动只在白名单里、测试与快照未改、无残留占位。`review` 可跑突变测试。任务清单模板改为「只改哪些文件、要变绿的测试」。

Flow 格式加节点字段 `tier = "strong" | "standard"`，默认 `standard`，`human` 节点不得声明。引擎只把它透传到 `next` 与任务书；协调者据此派模型。`spec-dev` 里 `spec`、`plan`、`scaffold`、`review` 为 `strong`，其余 `standard`。

**为什么 `scaffold` 放在人审之后。** 骨架是重活，人审看的是几页文字。人先审便宜的，通过了再花强模型的钱。方案被打回时骨架不用重做。

**否决。** 让 `plan` 顺手做骨架（人审前就花大钱，且打回要撤销提交）；不加 `tier`，靠 README 告诉协调者（弱协调者会忘，标签在 `next` 里随手可得）；引擎按 `tier` 选模型（引擎不知道宿主有什么模型，也不该知道）。

**后果。** `spec-dev` 十节点二十三边。一个需求里强模型被调用四到六次，标准模型跑几十次。Workbook 合同加一个字段与一条编译规则；`plan.md` T04、T05、T10、T11 各加一个测试。

## D-22 首次提交前的收口

2026-09-24，用户提出四项，采纳并调整。

**提交信息格式。** `type(scope): 中文摘要`，正文中文，末尾 git trailer `Task`、`Work`、`Agent`。类型与范围用英文是为了 `git-cliff` 按类型分组；摘要用中文是为了人读。这一格式同时写进 [engineering.md §4](engineering.md) 与 `spec-dev` 的实现者规则、骨架规则、交付说明书，两边一字不差。`cliff.toml` 里原有一条「含汉字的提交跳过」的规则被删除，否则中文提交进不了变更日志。

**AGENTS.md 与 CLAUDE.md。** AGENTS.md 是唯一入口，按 OpenAI 的 AGENTS.md 惯例与 Anthropic 的 CLAUDE.md 建议写：短、具体、命令可直接跑、边界明确、当前阶段一句话。`CLAUDE.md` 只有一行 `@AGENTS.md`，用 Claude Code 的导入语法引用，避免两份文件漂移。Cursor 与 Codex 直接读 AGENTS.md。

**非文档文件。** CI 分成 docs、build-rust（ubuntu 与 macos 矩阵，产品主目标是 Apple Silicon）、release 三个 job，去掉未使用的 `cargo-llvm-cov`，命令与 AGENTS.md 逐字一致，分支改为 `main`。pre-commit 用官方 `crate-ci/typos` 钩子，`cargo deny` 只在依赖文件变化时跑。`deny.toml` 去掉 Windows 目标，未知源从警告改为拒绝。`rust-toolchain.toml` 用 `minimal` profile。根 `Cargo.toml` 留给 T01 改成 workspace，避免首次提交夹带半个 T01。

**否决。** 现在就把根 `Cargo.toml` 改成 workspace（那是 T01 的一部分，拆开会让 T01 的提交不完整）；在 CLAUDE.md 里另写 Claude 专用内容（目前没有 Claude 专用的事，多一份就多一处漂移）。

## D-23 T02 复核后的三处迭代

2026-09-24。

**背景。** 第一个填空任务 T02 由强模型完成。它发现 `scripts/check-task.sh` 有三个缺陷会让按规则做的任务无法通过门禁，自行修复并单独提交，标 `Task: T01`。这违反了规则 8、9 的字面，但判断正确。代码本身通过复核，两项低严重度待修。

**选择。**

1. 规则 9 加唯一例外：工具缺陷可修，单独提交、`Task: T01`、逐条说明、不放松意图；里程碑复核每一次工具改动。把实现者做对的事变成规则，而不是留着「违规但正确」。
2. 新增 §0.2.1「实现者是强模型时」：可连做相邻任务；复核发现由实现者修，复核者只补测试。测试仍归骨架与复核者，这条不松。
3. 新增 §0.6 复核记录：每个任务卡一行「复核」，待修项编号 `Bn`，修复提交引用它。任务状态在有待修项时回到 `doing`。

**根因与补救。** T01 只用负例验证了门禁脚本，没有做过一次「按规则填满一个任务再跑门禁」的正例试跑。T01 的验收与 `spec-dev` 的骨架规则都补上这一条。

**否决。** 把工具缺陷也算「卡住」让人来修（人要看的是方向，不是 shell 脚本）；允许实现者改测试来配合工具（测试是合同，一旦可改就没有锚点）。

## D-24 反思放在 Workbook，引擎只给事实

2026-09-24，用户提出，采纳。

**背景。** T02 复核的价值大半来自审流程而不是审代码：发现工具缺陷、把它们变成规则、校准实现者档位。用户问要不要给引擎或 Workbook 加「反思与自我进化」。

**选择。** 分三层。引擎加一个只读投影 `work stats`（每节点到达、尝试、失败次数、平均耗时、进入来源；受阻与批准次数）与第四种输入来源 `engine.stats`，开工时把这份 JSON 写成 `stats.json` 按字节冻结绑给节点。`spec-dev` 加终点节点 `retro`（`standard`，带 `gate`）：读 `stats` 与各报告第一行，按六个闭集类别写 `lessons.md`，每条建议必须有证据（哪个 Attempt 的哪份文件哪一行）与落点（Workbook 内哪个文件哪一段），否则进「不建议改的」。人在最后一次批准时同时看交付说明与反思，采纳的改进下一版并在 README 修订记录里写「采纳 L1、L3，否决 L2」；下一次 `retro` 核对上一版建议是否见效。`gate` 从 `deliver` 挪到 `retro`。

**否决。** 引擎按统计自动调 `max_visits`、换 `tier`、跳节点（宪章 §7 否决的「更聪明的调度」，且同一版本行为不再一致，审计失效）；让 `retro` 直接改 Workbook（`INV-6`，Workbook 只由人冻结）；给引擎加「反思」命令（引擎没有可反思的东西，只有事实）；`retro` 用 `strong`（它做的是归类与定位，判断留给看 `lessons.md` 的人）。

**依据。** 宪章 `INV-1`、`INV-2`、`INV-6`、§7。「自我进化」去掉「自我」：Workbook 提出对自己的修改，人冻结，引擎装新版本。差的那一步是人，这一步是产品定位里不能省的。

**后果。** `spec-dev` 十一节点二十四边。core 加 `render_stats`、`render_stats_json`、`Effect::WriteFile`、`InputSource::EngineStats`，保留字加 `engine`；协议加 `work stats`；plan 相关任务各加测试；M1 到 M3 加「流程教训」一节。`retro` 每次运行多一次 standard 调用，两三千 token。

## D-25 复核也要被复核

2026-09-24。

**背景。** T02 的实现者对复核意见写了一份反驳。核实后，反驳全部成立：复核有两处事实不实（说「删除行只有 `allow`、`ignore`、`todo`」，实际 `text.rs` 换了一行 `use`；说「改动只增不删」）、三处漏检（`check-task.sh` 第 124 行 `"$residue："` 里的全角冒号被 bash 当作变量名的一部分，`set -u` 直接中止，负例输出里的报错行被复核者忽略；`manifest.rs:114` 有骨架残留的 `#[allow(unused_variables)]`，按新检查 2 的语义会让 T03 按规则做完也过不了；检查 1 在工作树脏与净两种情形下只看一边）。另有两条判断偏窄：B1 只点了前导 `+`，前导零同样破坏解析回环；B3 的根因是 `kebab_id!` 宏用类型名当 `field`，与 `work_name`、`work_id` 的词汇不一致。

**事实。** 复核声称「用 diff 核对了删除行只有 `#[allow]`、`#[ignore]` 与 `todo!`」，实际只查了一个文件，`text.rs` 还删了一行 `use`。复核声称「重跑负例仍能报错」，实际只 grep 了以 `check-task:` 开头的行，没看到脚本在 `set -u` 下因 `"$residue："`（全角冒号被 bash 吞进变量名）中止。复核没发现骨架在 `parse_manifest` 上留了一个没有 `todo!()` 的 `#[allow(unused_variables)]`，按新的检查 2 语义 T03 填完也过不了。复核把 `is_han` 缺区段的处理定为「改合同措辞」，实现者指出这样的措辞没法测，应列出码点区间。复核把 B3 当单点文档失配，根因是 `kebab_id!` 宏用类型名当错误 `field`，与 `work_name`、`work_id` 词汇不一致。

**根因。** 复核者只 grep 了输出里以 `check-task:` 开头的行，没读整段输出；只对一个文件做了 diff 核对，把结论推广到四个文件。两处都是「看了一部分，说成全部」。

**选择。**

1. 复核结论必须引用可重跑的命令与完整输出，不引用摘要。「我核对了」要写成「`git diff a b -- <文件> | grep '^-'` 的输出是 …」。
2. 反驳有效：实现者对复核的反驳与复核本身同等对待，逐条核实，成立的写进任务卡与本文。
3. 修法归属按引入者：骨架残留、宏词汇、合同措辞、`tasks.toml` 白名单由复核者（骨架作者）改；解析函数与 `check-task.sh` 的缺陷由实现者改。
4. 合同措辞要可测：B2 从「按 Unicode 汉字区段」改为逐区间列出十二个码点范围，测试对每个区间取一个码点。模糊措辞比近似实现更糟。
5. `tasks.toml` 加 `[T01]`：工具提交不得动 `crates/`，不再落在核对盲区。
6. 规则 3 明确：同文件私有辅助函数是实现细节，允许。原文没禁，实现者按最严解释自我批评，说明规则需要写清而不是靠猜。

**否决。** 撤回 T02 的「通过」（没有影响已实现行为的缺陷，十条测试绿，撤回没有依据）；把复核者的错误算成实现者的返工；因为复核不够严就取消逐任务复核（问题是复核的方法，不是复核本身）；让复核者顺手把实现者的缺陷一起修掉（归属混了，下次就分不清谁该对什么负责）。

**后果。** T02 保持 `doing`，五条新测试待实现者启用；`check-task.sh` 两处由实现者修，`Task: T01`。M1 检查表加一项：复核结论是否附带可重跑的命令与完整输出。任务卡多一行「复核的复核」。这一轮往返约两次强模型调用，提前消掉三处会在 T03 立刻爆炸的缺陷。（此条原先被写成两条同号的 D-25，M1 第二轮合并为一条，保留本条结构，并入另一条的两项否决、归属划分与「事实」段；合并哪一版原写「由人定」，2026-09-25 用户认可本次合并。`check-docs.sh` 加编号唯一检查。）

## D-26 输入名只查唯一，不走 ID 字符规则

2026-09-25，用户拍板。

**背景。** T04 的 `parse::convert` 对 `inputs[].name` 调了 `validate_id`；T07 复核时把 `workbooks/spec-dev` 与 `testkit` 夹具里带下划线的输入名（`plan_tpl`、`side_out` 等）改成 kebab 去迎合这道检查（00ca15a）。M1 后的对抗复核（verify-t05-compile，14 条发现、每条 3 票反驳制）唯独这一条三次反驳都没驳倒（0/3）：合同 §3.2 对 `inputs[].name` 只写「`name` 在节点内唯一」，ID 字符规则明文给了 `outputs[].name`、`start.<key>`、`requires[].name`，没给输入名。不对称是有意的：输入名不进 `from` 点语法，也不出现在 `inputs/<key>` 路径里，只作展示名与 JSON 键。

**选择。** 按合同字面办：删掉对 `inputs[].name` 的 `validate_id`，保留节点内唯一性检查；00ca15a 的改名全部回退（`side_out`、`plan_tpl`、`tasks_tpl`、`task_rules`、`fix_change`、`implement_change`、`review_report`、`scaffold_report`、`verify_report`），指令文档里的同名引用一并还原。资源文件路径（如 `resources/checklists/task-rules.md`）是 RelPath 不是输入名，不受影响。

**否决。** 改合同把 ID 字符规则扩到 `inputs[].name`（把期望弯向实现，workbook.md 明禁，且会把合同合法的写法判非法）；只回退夹具不改 `parse`（多检还在，下一个夹具还会再被拒一遍）；顺手给输入名补一条新的字符限制（合同没写的约束不发明）。

**后果。** `parse.rs` 的 `validate_id` 剩四个调用点：`start.<key>`、`from` 的输出名、`outputs[].name`、`requires[].name`，与合同逐条对应。`inputs[].name` 只受节点内唯一约束。样例与夹具恢复下划线写法。

## D-27 测试名写行为，任务归属写注释

2026-09-25，用户拍板。

**背景。** 此前测试名以任务编号开头（`t05_rejects_self_loop_edge`），任务卡里列的名字又省略前缀。前缀是把任务映射抄进交付物：任务重排、合并时名字变成化石，失败输出里读到的是排期号而不是哪条行为坏了。但前缀不只是装饰：`scripts/task.sh` 靠它选测试，而且带 `--run-ignored all`，好让实现者不删标记就看到红。按文件选跑行不通：`decide.rs` 装着 T06 到 T09 的测试，做 T08 时会连 T09 仍停在 `todo!` 上的禁用测试一起跑，`task.sh` 永远红。

**选择。** 测试名只写「条件 → 行为」。归属写成紧贴 `#[test]` 上方的一行 `// Task: Tnn`，它是任务卡映射在代码里的机器可读副本；`task.sh` 按它选测试。新增 `scripts/check-tests.sh`（pre-commit 与 CI）：名字不带前缀、每个测试恰好一行归属注释、`#[ignore]` 标签与之一致、名字全仓唯一、任务卡列的名字都存在且归属对。快照改显式名字，与函数名解耦。一次改完全部 279 个测试，7 个快照 `git mv`（字节不变），`tasks.toml` 删 `tests` 字段。改名提交打 tag `t11-baseline`，`check-task.sh` 以它为新基准。

**否决。** 按 `test_files` 选跑（上述原因）；`task.sh` 直接解析任务卡取名字（复核者补的测试都得回写任务卡，漏写无人发现，且 bash 解析 Markdown 太脆）；新测试执行新规、旧名渐进迁移（半新半旧比现状更乱，T11 起尚未填空，现在一次改最便宜）；用模块名命名（`optional_outputs_work` 是分类不是规格）。

**后果。** 两份映射（任务卡与注释）由 `check-tests.sh` 核对单向一致：卡上列的必须存在且归属相符；代码里多出的测试（复核者补的）只需挂被复核任务的编号。`unknown_subcommand_exits_2` 原名挂 `t01_`，任务卡列在 T17，按任务卡归 T17（T01 骨架里它已经是绿的）。

## D-28 宿主资源清单原样给声明；时间只有秒精度

2026-09-25，M1 第二轮，复核者定。

**背景。** M1 的 O1 修完后 `work start` 回了 Workbook 全量 `requires`，但每项是拼好的 `kind:name` 字符串：manifest 里写的 `version`、`digest`、`source` 到回复里就丢了，协调者开工前「自行确认宿主」时拿不到要确认的版本。`attempt begin` 同样。另一件：`Timestamp` 是 `pub struct Timestamp(pub String)`，谁都能塞任意字符串；`secs_between` 为此带着「解析不了当 0」的兜底，`day()` 取前 10 字节在短串上返回整串垃圾而不是日期，字符串排序也不等于时间序（`…05Z` 与 `…05.678Z`、`+08:00`）。

**选择。**

1. 两处回复的 `requires` 都是 `Vec<HostRequire>`，每项原样是 manifest 那条声明 `{ kind, name, version, digest, source }`，没写的字段为 `null`。`work start` 按 manifest 声明顺序给全量；`attempt begin` 按节点里的书写顺序给本节点引用的那几条。`digest` 用引擎统一的裸 64 位十六进制，`sha256:` 前缀只是 manifest 的书写格式。
2. `Timestamp` 只能经 `parse` 或 `from_unix_secs` 构造，反序列化也走 `parse`。格式固定为 `YYYY-MM-DDTHH:MM:SSZ`，校验日历（闰年、每月天数、`23:59:59` 为止）。runtime 的 `format_rfc3339` 与 `civil_from_days` 挪进 core 成 `from_unix_secs`（纯算法，不碰时钟，不违反 INV-1）；它对超出 9999 年的输入饱和到 `9999-12-31T23:59:59Z`，不产出 `parse` 会拒绝的五位年份。存储合同 §7 写明秒精度与这条饱和。
3. `Timestamp::unix_secs` 留给实现者（T10）：有了它，`secs_between` 不再需要私有解析与 0 兜底。

**否决。** 回复里继续用 `kind:name` 再另加一个 `versions` 映射（两份并行数据，协调者要自己拼）；`requires` 只给 `(kind, name, version)`（`digest` 与 `source` 是作者写给协调者核对与安装用的，砍掉就得再去读 manifest）；`Timestamp` 允许毫秒与时区偏移再在比较时归一（引擎自己产生时间，没有理由接受多种写法；固定格式换来字典序即时间序）。

**后果。** `Reply::Started` 与 `Reply::AttemptBegun` 的 `requires` 类型变了，协议 work start 第 8 步、attempt begin 第 5 步随之改写。`Timestamp` 的字段私有，runtime 与测试改用 `parse`；`observe::now()` 改调 `from_unix_secs`。新增测试见 M1 第二轮处置。

## D-29 `engine.stats` 的口径含本次 Attempt

2026-09-25，M2，复核者定。

**背景。** M2 给 T16 补重放测试时发现：提交时 `stats.json` 从 `begin` 之前的状态算（当前节点的 `visits` 还没增、本次 Attempt 还没进表），而崩溃后的重放只能拿到库里的提交后状态。从提交后状态反推提交前状态不可行：`visits`、`current`、`attempts` 可以借 `entered_from` 逆推，但 `updated_at` 在每个提交点都被刷新，跨秒时 `total_seconds` 就重建不出来，重放补写的 `stats.json` 与 `Attempt.inputs` 里记录的摘要必然对不上。

**选择。** 口径改为含本次 Attempt：`decide_begin` 先把 `visits`、`current`、新 Attempt、`updated_at` 推进到位，再用这份完整状态算 `stats.json`。提交后状态等于库里的状态，重放重算逐字节一致，效果真正幂等。

**否决。** 反推提交前状态（`updated_at` 不可恢复）；把 `stats.json` 内容存进库（内容可由状态导出，存两份必然漂移）；容忍重放内容不同（`Attempt.inputs` 里记的摘要与磁盘文件不符，「输入按字节冻结」名存实亡）。

**后果。** 工作 agent 读 `stats.json` 时看到自己这次 Attempt 已计入。`bind_inputs` 不再带 `attempt_id`，`EngineStats` 条目由 `decide_begin` 推进状态后回填。协议 `work stats` 一节已写明口径。

## D-30 `self update` 不引 `axoupdater`，直接读 `dist-manifest.json`

2026-09-25，T20，实现者定。

**背景。** 存储合同 §9 原写「`self update` 用 `axoupdater` 库读同一份清单」。核对 axoupdater 0.10.2 的公开 API：清单获取（`fetch_release`）是私有的，公开的 `Asset` 不带 sha256，唯一的完整更新入口 `run_sync` 会下载并执行 cargo-dist 的 `installer.sh`（由安装脚本自己决定装到哪、是否改 PATH）。这与本节的五步（下载到 `tmp/`、核对 sha256、两级 rename）不一致，安装脚本还可能写 shell rc 文件，踩 `INV-3` 的边界；依赖树还要带进 reqwest、tokio、miette。

**选择。** 不引 `axoupdater`。`selfmgmt` 直接读发布清单 `dist-manifest.json`：瘦格式 `{ version, assets: [{ platform, name, sha256 }] }` 是本地发布目录与测试的合同（`SHELTIE_RELEASE_BASE`）；真实 GitHub Releases 的完整清单在 `selfmgmt` 里适配成同一形状，下载用系统 `curl`。替换顺序仍是存储合同 §9 的五步。平台串用 Rust target triple（`aarch64-apple-darwin` 等，与发布包命名一致），不是 `<os>-<arch>`。

**否决。** 用 `run_sync` 走安装脚本（不受控、可能写 rc、无 sha256 核对）；把 `axoupdater` 仅当版本探针再自己下载（同一个清单要两套读法，且省不掉网络栈）；引入 `ureq` 等内嵌 HTTP 客户端（多一棵 TLS 依赖树，换来与 `curl` 相同的一次 GET）。

**后果。** `self update` 依赖系统 `curl`（`install.sh` 本来就要求）。真实清单的适配分支没有自动化测试，T25 从 `v0.1.0-rc` 升 `v0.1.0` 时人工核对，字段对不上就改 `selfmgmt` 的适配层，测试清单格式不变。存储合同 §9 已同步改写。

## D-31 `self install` 的幂等口径是字节相同

2026-09-26，M3 复审，复核者定。

**背景。** 协议 §3 `self install` 写「已存在同版本直接返回」，存储合同 §9 写「已存在且字节相同则不动」，实现与 M3 补测（fe91704）钉的是字节口径。两份合同同层冲突：同版本不同字节的场合（手工替换二进制），按协议不动、按 §9 重装。

**选择。** 统一为字节口径，协议 §3 改写。理由：install 没有版本来源可比对（运行中的二进制不知道自己对应哪次发布），字节是它唯一能核对的事实（INV-6）；同版本同字节是正常路径，行为不变。

**否决。** 把 §9 改成「同版本」（install 拿不到发布版本号，只能靠文件内容判断）；两个口径都留（同层冲突留着，下游永远不知道听谁的）。

**后果。** 协议 §3 与存储合同 §9 一致；实现与测试不动。`self update` 的版本比较仍是字面相等（允许降级），协议未定义排序语义，维持 M3 观察。

## 里程碑记录

### M1 core（2026-09-25）

**范围。** `git diff t01-skeleton..8af5f94 -- crates/sheltie-core`（T02 到 T10 的实现提交与其间的复核提交），`git log t01-skeleton..HEAD -- scripts/ tasks.toml` 的三次工具修复。

**结论：需修改。** 一条待修（B1，退回 T05），其余七条遗留（O1 到 O7）不影响已实现行为，归 M2 前的骨架补全或人拍板。实现质量整体扎实：逻辑与合同逐步对得上，错误路径精确，提交说明清楚；主要毛病是合同没写的地方自己补了约定（O1、O5、O6），而不是按规则 10 停下。

**检查表。**

| 项 | 结果 | 依据 |
| --- | --- | --- |
| 依据 | 需修改 | B1：合同 §3.2「可选输出下游不得当必需输入」在 §4 与骨架注释里都漏了，没有规则、没有测试。实现者自补、合同未写的约定：`decide_start` 的 `requires` 只取节点并集并注明「由 runtime 补」（O1）；`blocked` 计数的定义（O5）；`engine.stats` 序列化失败时写 `{"nodes":[]}`（O6）。骨架自带、合同未写：规则 5「来源不得是自己」 |
| 不变式 | 通过 | `cargo tree -p sheltie-core -e normal` 无 I/O crate（`libc` 只经 `sha2 → cpufeatures` 做 CPU 特性探测）；状态转移只看结构与计数，不读摘要或输出内容（`INV-1`、`INV-2`）；`INV-3`、`INV-4` 在 core 不适用 |
| 正反例 | 已补 | 九条规则各有拒绝例；缺「恰好上限」一侧与 B1。M1 补测见下 |
| 真实链 | 不适用 | core 没有 CLI 入口，归 T19 到 T21 |
| 崩溃 | 不适用 | core 纯函数；效果幂等归 M2 |
| 边界 | 已补，另有遗留 | 大小上限原先只有远超上限的拒绝例，版本、名称、`requires` 个数与字段、标题、说明文本、节点数、边数、说明文件与资源文件大小共十余处「恰好上限 / 多一个」没有测试，M1 补齐；未知字段拒绝有测试；`confine()` 归 runtime。O4：`BoundedText` 的 `#[serde(transparent)]`、`Graph` 的 `Deserialize` 绕过构造校验 |
| 文档 | 需人拍板 | `check-docs.sh` 过。协议 §4 示例与 T01 快照三处措辞不一致（O7）；本文有两个 `D-25` 标题 |
| 提交 | 通过，有瑕疵 | 一任务一提交。T07 发现夹具依赖 T08 到 T10 的函数，在 9044612 里一并填了并改了 `plan.md`、`tasks.toml`，说明写清，但 T08 到 T10 因此没有「先看到红」；T07 到 T10 的 `Task`、`Agent` 与 `Co-Authored-By` 之间空了一行，`git log --format='%(trailers)'` 取不到 |
| 突变 | 已处置 | 首轮 524 个：281 杀死、145 存活、98 不可编译。处置后 511 个：409 杀死、5 存活、97 不可编译。5 个都有理由，见下 |
| 证据 | 附命令 | 下文每项带可重跑命令；首轮突变结果因目标目录共用作废过一次，已写成工具（L1） |

**存活的突变体处置（首轮 145 个）。**

- 补测试杀死 134 个。`render.rs` 的时间换算、平均耗时与 `blocked` 计数 80 个：夹具时钟固定，快照里全是 `0s`、`blocked: 0`。`manifest.rs`、`parse.rs`、`compile.rs`、`ids.rs` 的上限与格式 38 个。`decide.rs` 的摘要、失败原因、输出大小边界 4 个。`blocked` 行的 `retries_exhausted` 与 `no_legal_edge` 两个分支。骨架里给 runtime 用的字面形式（`ErrorCode::as_str`、`Executor::as_str`、`Command::name`、`Timestamp::day`、各 newtype 的 `Display` 与 `From<_> for String`）。
- 删死代码消掉 6 个。`status_after_success` 里的 `!state.current_approved()`：批准只发生在门槛 Occurrence 的 Attempt 成功之后，此后该 Occurrence 不会再有 `Running` 的 Attempt，提交时这个条件恒真；`WorkState::current_approved` 随之删除。`next_attempt_id` 重复了 `decide_begin` 的编号逻辑，根因是骨架的 `bind_inputs` 签名拿不到 attempt id（文档注释却要求它算 `stats.json` 路径）；签名加 `attempt_id` 参数后删除。规则 5 的「来源节点不得是保留字」同样删除：解析层从不产生保留字来源的 `Node`，没有突变但属同类死代码。`lib.rs` 的 `#![allow(dead_code)]` 删除，clippy 无告警。
- 等价突变 4 个，保留。`next.rs:104` 的 `<` 换 `<=`：`Active` 且最新 Attempt `Failed` 时必有 `retry < max_retries`，否则已是 `Blocked(RetriesExhausted)`。`parse.rs:140` 的 `||` 换 `&&`：漏掉的不合规项随后被 `OutputName` 校验拒绝，报错规则相同。`render.rs:520` 的 `day - 1` 两个：常数偏移在 `secs_between` 的差里抵消。
- 测试夹具 1 个（`testkit.rs:273`），`scripts/mutants.sh` 起排除。

复跑：`scripts/mutants.sh sheltie-core`，输出末行 `mutants tested …: 4 missed`（上面四个等价突变）。

**其他核对。**

- `rg -n 'todo!\(|allow\(unused_variables\)' crates/sheltie-core/src` 零命中。core 里的 `#[ignore` 只剩 T11 的十一条与 B1 的一条，runtime、cli 的都属 T12 以后的任务。
- 工具改动三次：8b33c48（bash 3.2 兼容）、deb31ed（混合文件只比对测试模块、检查 2 收窄为只清 `todo!("Tnn")`、检查时机）、904a54c（范围取并集、全角冒号、空数组）。每次说明逐条写了原错与改法；deb31ed 收窄检查 2 的意图由里程碑的全仓零占位核对兜住，没有放松。

**待修。**

- **B1（T05，退回 `doing`）。** 被引用输出 `required = false` 而输入默认必需时，编译放行；上游不写该文件，`next` 仍给出 `attempt begin`，开工必报 `INPUT_UNAVAILABLE`，Work 只能取消。合同 §4 规则 5 与 `check_rule_5` 注释已补这一句；复核者补 `rejects_required_input_on_optional_output`（禁用）与 `accepts_optional_input_on_optional_output`。三份样例与 `spec-dev` 没有可选输出，不受影响。实现者启用测试、修，提交 `fix(core): 规则 5 拒绝把可选输出当必需输入`，M1 再复核这一条。

**遗留（M2 前由骨架作者补签名与测试，或由人拍板）。**

- **O1 `work start` 的 `requires`。** 协议第 8 步要 Workbook 声明的全部宿主资源，`Command::Start` 不带 manifest，core 只能给节点并集。选一：`Command::Start` 带 manifest 的 `requires`，或在 T16 骨架里写明由 runtime 用 manifest 覆盖。落点 T16。
- **O2 `resource.<path>` 的篡改检测。** 协议第 3 步说「重算 sha256 与已记录值核对」，但 `WorkState` 不记每个资源的摘要，`bind_inputs` 以观察为准，资源上的 `ARTIFACT_MODIFIED` 不可达。冻结副本只读，风险低；要么 `start` 时记下资源摘要，要么协议改成「冻结副本目录摘要与 `workbook.digest` 核对」。落点协议 §3 与 T16。
- **O3 任务书宿主资源表的「版本」列恒为 `-`。** `render_brief` 看不到 manifest，协议示例是 `^1`。`Graph` 编译时可把 manifest 的版本带进节点的 `requires`。落点 T05 骨架与快照。
- **O4 反序列化绕过构造校验。** `BoundedText` 是 `#[serde(transparent)]`，从库里读出超长摘要不会报错；`Graph` 可被反序列化出来，与「只能由 `compile` 构造」矛盾；`WorkName` 反序列化时静默规范化。runtime 从 `store.db` 读 `WorkState` 时 `STORE_CORRUPT` 因此漏检。落点 T13、T16 骨架。
- **O5 `work stats` 的 `blocked` 定义。** 协议只给了示例 `blocked: 1`。实现是「成功过的门槛 Occurrence 数 + 重试耗尽的 Occurrence 数 + 当前是否 `no_legal_edge`」，M1 已用测试钉住。需要人确认后写进协议 §3。
- **O6 `engine.stats` 序列化失败时写 `{"nodes":[]}`。** 实际不会失败，但失败时伪造内容与「引擎只记事实」相悖；应让它不可失败（手写 JSON）或把错误传出去。低优先，随 T16 一起改。
- **O7 协议 §4 与快照的措辞。** 「尚无（上游 X 还没有产出）」对快照「尚无」；「此 skill」对「此资源」；说明「逐字」对 `trim_end`。快照是 T10 的标准答案，但以合同为准；两者须一致，改哪边由人定。

**流程教训。**

| # | 类别 | 证据 | 改哪 | 改成什么 | 处置 |
| --- | --- | --- | --- | --- | --- |
| L1 | 工具 | M1 第一次 `cargo mutants`（作废，不计入上面的首轮）：全局 `~/.cargo/config.toml` 设了 `target-dir`，并行副本共用产物，175 个「存活」里多数是测试跑了未突变的二进制 | `scripts/mutants.sh`（新）；`plan.md` §0.4、M1、M2 卡 | 固定 `CARGO_TARGET_DIR=target`，用 nextest，排除 `testkit.rs` | 采纳，已改 |
| L2 | 骨架 | 9044612：`bind_inputs` 的注释要求算 `attempt_dir/stats.json`，签名却没有 attempt id，实现者只好写 `next_attempt_id` 重复编号逻辑 | `engineering.md` §3「写新测试的人」段 | 骨架注释要用到的值都必须能从参数得到，做不到改签名 | 采纳，已改 |
| L3 | 骨架 | B1：合同 §3.2 的「不得」没进 §4 清单，骨架与测试都跟着漏 | `engineering.md` §3 同段；§5「正反例」行 | 合同里每句「不得」「必须」都有一条拒绝例 | 采纳，已改 |
| L4 | 测试 | 首轮存活里 `render.rs` 占 80 个：`stats_table_mid_flow` 快照全是 `0s`、`blocked: 0`，改错公式快照也不变 | `engineering.md` §3 同段 | 快照与断言里的数值字段至少一条非零、非默认值的断言；固定时钟下时间差单独造数据 | 采纳，已改 |
| L5 | 测试 | 首轮存活里上限类 38 个同时存活 `>`→`==` 与 `>`→`>=`，说明连「多一个」的拒绝例都没有 | `engineering.md` §3 同段；§5「边界」行 | 每个上限一对：恰好上限接受、多一个拒绝 | 采纳，已改 |
| L6 | 实现者 | 9044612 `decide_start` 注释「未用到的 manifest 条目由 runtime 在返回前补」；`count_blocks` 的定义；`engine_stats_artifact` 的兜底串 | `plan.md` §0.2 规则 10 | 把「由 runtime 补」「解析不了当 0」「出错用默认内容」写进注释也算发明，同样要停 | 采纳，已改 |
| L7 | 顺序 | 9044612：T07 的夹具要用 submit、fail、render，实现者在 T07 里填了 T08 到 T10 的函数，T08 到 T10 只剩验收，没有「先看到红」 | `plan.md` T01「验证」段 | 骨架作者对每个任务核对：只启用本任务测试时 panic 的都是本任务的 `todo!`。M2 开工前对 T12 到 T16 做一次 | 采纳，已改 |
| L8 | 提交 | 9044612、4334925、810730f、8af5f94、43ec6bf：`Task`、`Agent` 与 `Co-Authored-By` 之间空行，git 不认作 trailer | `engineering.md` §4 | 所有 trailer 同一段，不空行 | 采纳，已改；`check-task.sh` 仍按文本 grep，不改（改成严格解析会让历史提交全部不合格，收益小） |
| L9 | 文档 | 本文 294 行与 315 行两个 `D-25`，写的是同一件事的两个版本 | 本文 | 合并为一条 | 否决自动处理：决策记录是历史，合并哪一版由人定；`check-docs.sh` 暂不加唯一性检查，等合并后再加。第二轮已合并（保留第一条的结构，并入第二条的两项否决与成本），`check-docs.sh` 加编号唯一检查 |
| L10 | 审查 | M1 卡要求「全仓零 `#[ignore`」，但 T11 以后的测试按设计仍禁用，复核者新加的待修测试也必须禁用 | `plan.md` M1 卡 | 改为「本里程碑覆盖的任务标签为零」 | 采纳，已改 |

**修复提交。** 本条记录所在的 M1 提交（补测试、删死代码、合同与流程修订，打 tag `t05-review-3`）；待 T05 的 `fix(core)`。

**第二轮处置（2026-09-25）。** 实现者在 `t05-review-3` 之后交了七个修复提交（5f4b6d6 到 6290c61），附六条疑问。逐条核对后：

- 已修，复核通过：B1（5f4b6d6）；O1 的「全量」（51eda62）；O3（51eda62）；O4（f47af48）；O5 写进协议（ac17547）；O6（03a9be9）；O7 第一处「尚无（上游 X 还没有产出）」（496a182）。
- 实现者的疑问里成立的：O2 复核者原建议报 `WORKBOOK_TAMPERED`，存储合同 §5.1 明文要求冻结副本不符报 `STORE_CORRUPT`，复核者撤回原建议。不成立的：「样例说明文件没有末尾换行」，实际全部以 `0a` 结尾，`end-of-file-fixer` 也强制这一点。
- 复核者本轮改的（签名、测试、合同，实现者不能动的部分）：
  - O1 的改法：`requires` 从 `kind:name` 字符串改为原样的 manifest 声明，见 D-28。`Reply` 两处改类型，`decide` 两处随之改，新增 `start_requires_carry_manifest_declaration_as_is`（T06）。
  - `Timestamp` 收紧，见 D-28。新增 `timestamp_parse_accepts_utc_second_precision`、`timestamp_parse_rejects_offsets_fractions_and_impossible_dates`、`timestamp_deserialize_validates_and_serialize_is_plain_string`、`timestamp_from_unix_secs_matches_known_dates`（T01），以及禁用的 `timestamp_unix_secs_matches_independent_calendar_math`（T10）。runtime 的日历算法挪进 core，`format_rfc3339_epoch_and_known_date` 随之删除，覆盖由 `timestamp_from_unix_secs_matches_known_dates` 接替。
  - O7 第二处定为「此 <kind>」：协议 §4 补说明；快照 `brief_for_node_with_requires` 改成「此 skill」；`brief_require_version_column_comes_from_manifest` 改名 `brief_require_row_takes_version_from_manifest_and_names_kind` 并加一条 `mcp` 声明。两条都禁用为 T10。已用正确实现临时验证两条会变绿，然后撤回。
  - O7 第三处定为「逐字，去掉末尾换行」：协议 §4 模板改写，与实现一致，不动代码。
  - O2 落到 T16：协议 attempt begin 第 3 步写明 `resource.<path>` 输入由副本整体摘要覆盖，不符报 `STORE_CORRUPT`；`tests/service.rs` 补三条禁用测试，挂 T16。
  - D-25 合并（L9）。
- 流程：实现者的修复提交都写了 `Task: M1`，`check-task.sh` 因此按 M1 的白名单核对，而 M1 没有白名单，等于没核。`plan.md` §0.6 补一条：里程碑复核退回的修复用被退回任务的编号。
- 交还实现者：T10 退回 `doing`（「此 <kind>」；`Timestamp::unix_secs`，`secs_between` 改用它）；O2 的实现随 T16。完成后 M1 复跑突变并复核实现者本轮自写的测试。`render.rs:520` 的两个等价突变会随 `rfc3339_secs` 删除而消失。
- 不做：`BoundedText` 的 `Deserialize` 改为调用 `new`。重复只有一个长度比较，而且反序列化拿不到 `field` 名，两处报错本来就不同，改了没有收益。`render_brief` 输入表的 `_ => "尚无"` 分支按编译规则 5 不可达，但删掉只能换成 panic 或静默少一行，都比现在带注释的兜底差，保留。

**第三轮：复审（2026-09-25）。** 核对第二轮退回项的修复，复跑突变，结论：**通过，M1 关闭**。

- B1（5f4b6d6，T05）：`check_rule_5` 的 Node 分支拒绝「可选输出配必需输入」，位置在输出存在性检查之后；`rejects_required_input_on_optional_output`、`accepts_optional_input_on_optional_output` 已启用且绿。改动只有 `compile.rs` 与 `plan.md`。
- T10 两项（bc14d36）：宿主资源表说明写「此 <kind>」（`render_brief` 用 `kind.as_str()`，格式注释同步）；`Timestamp::unix_secs` 是 `from_unix_secs` 的逆运算（days_from_civil），`secs_between` 改用它，`rfc3339_secs` 与「解析不了当 0」兜底已删。三条禁用测试已启用。期望值抽查与 Python `datetime` 独立核算一致（`2026-09-24T03:04:05Z` → 1790219045 等五个点）。
- 实现者在修复轮自写的两条测试（51eda62 的版本列、03a9be9 的双 `engine.stats` 输入）逐条读过：断言的是行为（版本取自 manifest、两个输入共享同一份 `stats.json`），不是实现的镜像；当前形式（第二轮改过名与夹具）正确。
- 快照零改动（`git diff t10-review..HEAD -- '**/*.snap'` 为空）；`next` 的 executor/tier 透传、状态卡 blocked 行、`work stats` 口径与协议一致。
- 机械核对（命令均可重跑）：`rg 'todo!\(|allow\(unused_variables\)' crates/sheltie-core` 零命中；`#[ignore]` 全仓只剩 T11 到 T24 的标签；core 无 `std::fs`、时钟、随机数；`cargo tree -p sheltie-core -e normal` 无 I/O crate（`libc` 只经 `sha2 → cpufeatures`；`zmij` 是 `serde_json` 的 JSON 解析器）；`check-docs.sh`（47 个文件）、`check-core-vocab.sh`、`check-tests.sh`（289 个测试）、`cargo deny check` 与四条门禁全绿。
- 工具改动复核（`git log t01-skeleton..HEAD -- scripts/ tasks.toml`）：除第一、二轮已核的三次外，本轮新增 dc44577、18c65fa 两个措辞清扫提交碰过 `scripts/`——逐行核过 diff，只动注释与报错文案，检查意图未放松；fa4710b 碰了 cli/runtime 源码与 cli 测试共用模块，全是文档注释，产品字符串、快照、断言零改动。
- 提交纪律：修复提交已按 §0.6 新规写被退回任务的编号（5f4b6d6 写 `Task: T05`，bc14d36 写 `Task: T10`）；trailer 连排，`git log --format='%(trailers)'` 能解析出 `Task` 与 `Agent`（L8 的修法生效）。

**突变复跑。** `scripts/mutants.sh sheltie-core`：595 个突变体，519 杀死、7 存活、69 不可编译、0 超时。7 个存活逐条处置：

- `parse.rs:140`（`||`→`&&`）与 `next.rs:101`（`<`→`<=`）：第一轮已论证的等价突变，行号随重构移动，结论不变。
- `render.rs:520` 的两个等价突变随 `rfc3339_secs` 删除而消失，与第二轮预期一致。
- `decide.rs:252` 与 `render.rs:124`（`&&`→`||`，O1/O3 修复轮引入的 requires 查找）：**真缺口**。manifest 里出现 kind 与 name 交叉的声明（如 `skill:db` 与 `mcp:db` 并存）时，突变体会拿错声明、任务书版本列与 `attempt begin` 回复出错。补两条测试：`begin_reply_requires_ignore_crossed_kind_name_pairs`（T07）、`brief_require_version_ignores_crossed_kind_name_pairs`（T10）。手工把两处 `&&` 注入成 `||`，两条测试都红；还原后回绿。
- `state.rs:71` 三个（`from_unix_secs` 的 yoe 世纪修正项 `/1460`、`/36524`、`/146096`）：**真缺口**，既有用例全在 1970 到 2100 年，修正项在这些区段不改变整数除法结果。穷举全部 293 万个可达日（`secs ≤ 253_402_300_799` 饱和域）找到判定日：三个突变都从 1970-03-01 起算错一年，末项修正只在 era 末日（doe = 146096，如 2399-12-31）取值 1。这些日期与 2200/2400/2500/4000/9999 各一个点补进 `timestamp_from_unix_secs_matches_known_dates` 与 `timestamp_unix_secs_matches_independent_calendar_math`，三个突变逐一注入验证杀死。定向复跑（`-F 'replace && with \|\||Timestamp::from_unix_secs'`，79 个突变体）：78 杀死、1 不可编译、0 存活。

**本轮发现与处置。**

- N1（流程）：dc44577、18c65fa、fa4710b 三个措辞清扫改了 T02/T05/T10 测试模块里的五行注释与断言提示文字，`check-task.sh` 检查 4 因此报「测试代码被改动」。逐处核实全是文字、无行为改动（断言、期望值、快照不变）。这是用户拍板的全仓清扫，不是实现者越界；但基准 tag 因此失效。处置：核实后以本轮提交重打 `t02-review-3`、`t05-review-5`、`t10-review-2`。
- N2（工具）：`cargo mutants` 默认把结果写进未忽略的 `./mutants.out`，跑突变期间工作树变脏，`check-task.sh` 检查 6（只在工作树干净时查提交信息）静默跳过，也有误提交风险。已修（82d870b，`--output target`，`Task: T01` 单独提交）。
- N3（文档）：合同 §3 的 article-review 样例里 `draft` 节点缺 `max_visits = 3`。缺了它，§3 自己的 `back` 边在默认上限 1 下永远不可用（`draft` 在 start 时已用掉唯一一次到达）。`examples/article-review/` 的实际样例有这一行。已把 §3 改成与样例一致。
- 观察，不处置：`Graph` 仍 derive `Serialize` 但没有序列化调用者（Deserialize 已按 O4 拆掉）；`NextOp` 的 `edge`、`tier` 序列化成 `null` 而不是省略该键，协议 §5「只在…出现」的措辞与之有出入，CLI 落地时（T17 起）若有人咬文再定；`render_brief` 在节点不在图里时返回空串，`decide` 路径不可达，属防御性兜底；nextest 把 `node_count_limit_is_64` 标 `leaky`（64 节点夹具的线程标记，与正确性无关）。

**流程教训（本轮新增）。**

| # | 类别 | 证据 | 改哪 | 改成什么 | 处置 |
| --- | --- | --- | --- | --- | --- |
| L11 | 流程 | N1：措辞清扫改了测试模块注释，三个任务的 `check-task.sh` 基准当场失效 | `engineering.md` §5「文档」行；或清扫规则 | 全仓措辞清扫不得动 `#[cfg(test)]` 模块；非动不可时，由复核者当场逐处核实并重打 `tNN-review` 基准 | 采纳：本轮已重打三个 tag；规则写进 engineering.md §5 |
| L12 | 工具 | N2：`./mutants.out` 未忽略，check-task 检查 6 在突变测试期间静默跳过 | `scripts/mutants.sh` | 输出收进已忽略的 `target/` | 采纳，已改（82d870b） |
| L13 | 测试 | state.rs:71 三个突变存活：日期换算的既有用例全挤在 1970 到 2100 年，世纪修正项摸不到 | `engineering.md` §3「写新测试的人」段 | 查表/换算类函数，用例要覆盖修正项生效的每个区段；找不到时穷举可达定义域找第一个判定点 | 采纳，已改 |

**M1 关闭。** T01 到 T10 全部 done；O1 到 O7 的落点：O1、O3 到 O7 已修并复核通过，O2 随 T16（测试已挂 T16 禁用）。下一次模型审查是 M2（runtime）。

### M2 runtime（2026-09-25）

**范围。** `git diff 94519d7..2344894 -- crates/sheltie-runtime`（T12 到 T16 的实现提交 9b2d00f、a860311、00bd835、fc4f9f9、2344894，与复核者的夹具修复 fe7f5fe），`git log 94519d7..HEAD -- scripts/ tasks.toml` 的两次 `tasks.toml` 白名单修订（T13 加 `failpoint.rs`、T15 加 `service.rs`，均为按依赖提前填入的登记，提交说明写清了原因，检查意图未动）。T16 同时验收了 M1 遗留 O2（冻结副本核对）。

**结论：需修改 → 复核者本轮直接修复，通过，M2 关闭。** 两条实质缺陷（B1 注入点位置、B2 重放口径）都源于骨架与语义缺口，按 [plan.md](plan.md) §0.6 由复核者修，不退回实现者：B1 不改测试只挪一行调用；B2 动了 `bind_inputs` 签名与 stats 口径，记为 D-29。实现质量整体扎实：事务顺序与合同逐行对得上，错误归类精确（`WORKBOOK_EXISTS` 只认主键冲突），提交说明完整。

**检查表。**

| 项 | 结果 | 依据（命令均可重跑） |
| --- | --- | --- |
| 依据 | 通过 | 每个行为能指回存储合同或协议：staging 四步（§5）、`commit()` 事务顺序（§2，逐行对照 `store/commit.rs:52-144`）、序号分配（§7.1，`store/mod.rs:119` SQL 与合同原文一致）、冻结副本（§5.1，`service.rs:306` `load` 核对摘要）、`resource.<path>` 由副本整体摘要覆盖（协议 attempt begin 第 3 步，O2 落地）。无文档外发明行为 |
| 不变式 | 通过 | runtime 无业务判断：规则判断全在 core 的 `decide`/`compile`，runtime 只做观察、事务、效果；`INV-3`：写入全部在管理根之下，外部键先经 `confine`（`service.rs:122`）；`INV-7`：状态只从 `works.state_json` 读，目录是投影。`cargo tree -p sheltie-runtime -e normal`：rusqlite、serde、serde_json、sha2、thiserror、uuid、camino、sheltie-core，无越界依赖 |
| 正反例 | 已补 | 存储合同 §5「拒绝符号链接、硬链接、非普通文件、单文件超 32 MiB、总量超 256 MiB」原先缺硬链接与非普通文件的拒绝例、缺两个上限的「恰好接受」一侧，本轮补齐（见补测试清单） |
| 真实链 | 通过（本层） | runtime 集成测试全部用 `tempfile` 独立管理根走公开 API；CLI 端到端归 T17 到 T23 |
| 崩溃 | 通过（走查） | §3 三行逐行走查见下；T23 的子进程测试仍禁用（依赖 T17 到 T19 的 CLI），B1 修复后它们才可能绿 |
| 边界 | 已补 | `confine` 三条拒绝例加不可读祖先一条；32 MiB 与 256 MiB 都有恰好上限接受、多一字节拒绝；`user_version` 与同空白异类型拒绝；只读库插入归类。`deny_unknown_fields` 补齐见下 |
| 文档 | 通过 | `check-docs.sh`（48 个文件）绿；协议 `work stats` 一节补了 stats 口径一句（D-29）；T23 任务卡补复核记录 |
| 提交 | 通过 | 一任务一提交，trailer 连排可解析（`git log --format='%(trailers)'` 核对 T11 到 T16 每条都有 `Task` 与 `Agent`）；T13、T15 提前填入的跨界都改了任务卡与 `tasks.toml` 并在说明里写清（T07 先例）；fe7f5fe 夹具修复打 `t15-review` 基准，与卡上记录一致 |
| 突变 | 已处置 | 首轮 177 个：78 杀死、36 存活、63 不可编译；处置后复跑 176 个：106 杀死、7 存活、63 不可编译，7 个都有归属（见下）。core 改动定向复跑 `scripts/mutants.sh sheltie-core -F 'work/decide.rs'`：56 个，40 杀死、0 存活、16 不可编译 |
| 证据 | 附命令 | 上文与下文每条带文件行号或可重跑命令 |

**崩溃窗口人工走查（存储合同 §3，逐行对代码）。**

| 时刻 | 保证 | 代码位置 | 结论 |
| --- | --- | --- | --- |
| `COMMIT` 前 | 库无变化，无观察副产物 | 注入点 `store/commit.rs:53`（`BEGIN IMMEDIATE` 在 :55）；效果只在 `store.commit` 返回 `Committed` 后执行（`service.rs:458-462`）；观察全程只读（`observe.rs` 无写） | 成立 |
| `COMMIT` 后、效果前 | 状态已推进；`status` 正确；重放补写任务书与状态卡 | 注入点 `service.rs:461`（B1 修复后）；重放预检 `service.rs:385-401` 命中后走 `replay`（:480）→ `replay_effects`（:537）补写 `brief.md`、`stats.json`（缺失才写）、状态卡 | 成立 |
| 效果中 | 效果幂等，重放补齐 | `write_atomic`（`service.rs:591`）临时文件再 rename；`SealOutputs` 置只读可重复（`apply_effects`，:511）；状态卡整份重写（同上） | 成立 |

`workbook add` 的窗口按 §5：staging 写在事务前，崩溃残留由下次 `add` 顺手清（`workbook_repo.rs:98`）；行已插入而目录未 rename 时 `verify` 报 `missing`（:291）。`work start` 在事务前写冻结副本与起始输入，是 §5.1 与协议 `work start` 第 5、6 步的明文顺序；事务没提交时库无变化，重放同 `request_id` 重新分配序号（烧掉的号按 §7.1 不回收）。

**存活的突变体处置（首轮 36 个）。**

- 补测试杀死 25 个。`confine` 不可读祖先（home.rs:93）；只读打开已存在的库（store/mod.rs:75）；同空白异类型的结构比对（schema.rs:63，原有负例的 SQL 连空白都不同，「只比空白」的突变体漏网）；只读库插入的归类（read.rs:132）；显式版本加载（workbook_repo.rs:183）；硬链接（:353）与单文件上限 `>=`（:359）、总量上限 `==`/`>=`/`*=`（:366、:375）；`remove` 的同号他 Workbook 牵连（:265）；`resolve_work` 两个 match 臂与 `find_works_by_prefix`（service.rs:289、:292，read.rs:77）；重放预检比较（service.rs:395）、两个载荷哈希函数（:614、:623）、审计脱敏（:632）、响应 revision 递增（:448 两个）、`replay_effects` 整体与 stats 条件四个（:545、:565、:566 两个、:569）。
- 重构消掉 1 个。`observe_optional`（observe.rs:47）原先零调用，`begin`/`submit` 各抄了一份它的错误分支；改为调用它，突变体随真实调用被既有测试杀死。
- 修复带出 2 个。B2 修复后 `replay_effects` 的 `retain` 反推被删除，service.rs:569 一类突变体随之消失；D-29 的 core 改动定向复跑零存活。
- 等价突变 1 个，保留。`home.rs:97` 的 `p != base.as_path()` 换 `true`：走到 `base` 本身时 `canonicalize(base)` 必以 `base_canon` 为前缀，结论相同；`base` 不可读时突变体报错、原实现放行，是 fail-closed 方向。
- 归后续任务 6 个。`failpoint.rs` 两个只在 T23 的子进程崩溃测试里被测（测试仍在等 CLI）；`selfmgmt.rs` 四个是 T20 的 `todo!()` 骨架与其 `platform()`（唯一调用方是禁用的 T20/T23 测试）。M3 复跑时应只剩 `home.rs:97` 一个。

**发现与处置。**

- **B1（骨架缺陷，复核者已修）。** `after_commit_before_effects` 注入点被骨架留在 `run_command` 入口，进程在加载前就退出，与 `before_commit` 效果相同，T23 的 `kill_after_commit_leaves_state_advanced_and_replay_returns_original_reply_and_rewrites_brief` 任何实现都不可能通过。已挪到 `commit_one` 的 COMMIT 之后、效果之前（1a5d7a7），T23 任务卡补复核记录。属 M1 L7「依赖顺序核对」没覆盖到的一类：注入点的位置语义。
- **B2（语义缺口，复核者已修，D-29）。** 提交时 `stats.json` 从 begin 之前的状态算，崩溃重放只有提交后状态，`updated_at` 不可反推，跨秒就重建不出同一份文件。口径改为含本次 Attempt，core（00eaeeb）与 runtime（1a5d7a7）一起改，补测试 `begin_replay_regenerates_missing_stats_json` 断言重放重算与提交时逐字节一致，`begin_engine_stats_counts_current_attempt` 钉住口径。
- **serde 边界收紧（复核者已修）。** engineering §2.2 要求所有 serde 结构 `deny_unknown_fields`，存储合同 §1.2 对 `state_json` 点名；work 模块七个结构与 `HostRequire`、`Response` 缺，库里多余字段会被静默吞掉（M1 O4 收构造校验，漏了这一层）。已补（c5d567c、1a5d7a7）。限制：内部标签枚举（`Command`/`Effect`/`Reply`/`NextOp`）serde 不支持该属性，变体内多余字段仍被忽略，记为观察。
- **骨架豁免删除（复核者已修）。** runtime `lib.rs` 的 `#![allow(dead_code)]` 按自身注释在 M2 删除，clippy 无告警。

**观察（不处置，记录在案）。**

- `work start` 被 `decide` 拒绝（如起始输入缺键）或崩溃时，`works/<work_id>/` 下已写的冻结副本与起始输入成为残留，库无行；§8 的「手工删」是唯一出路。不影响正确性。
- `works_referencing` 对解不开的 `state_json` 静默跳过（`workbook_repo.rs:263`）：损坏行可能被 `remove` 放行。该行之后任何读写都会报 `STORE_CORRUPT`，风险有限。
- 合同 §5 说 `remove` 删目录失败「只记日志」，runtime 没有日志设施，当前静默忽略（`workbook_repo.rs:240`）。T17 落地 CLI 时若有人咬文再定。
- `verify`/`digest_dir` 跳过非普通文件：往已装目录里塞一个 fifo 不会触发 `tampered`。普通文件的新增会被发现。

**流程教训。**

| # | 类别 | 证据 | 改哪 | 改成什么 | 处置 |
| --- | --- | --- | --- | --- | --- |
| L14 | 骨架 | B1：骨架放的注入点位置与名字语义相反，T13 到 T16 按规则不动它，缺陷一直留到 M2 | `plan.md` T23 卡；骨架自查清单 | 骨架放置的注入点、钩子逐个按名字核对语义位置（「X 之后」要在 X 真的发生之后）；T23 卡已记实际位置 | 采纳，已改 |
| L15 | 测试 | B2：T23 的崩溃测试只断言「brief.md 存在」，`stats.json` 重建口径与提交口径不一致因此漏网 | `engineering.md` §3「写新测试的人」段 | 重放与恢复类测试断言重建内容与提交时逐字节一致（或摘要相等），不只断言存在 | 采纳，已改 |
| L16 | 测试 | schema.rs:63：旧结构负例的 SQL 连空白排版都与常量不同，把比对函数换成「只比空白」也能过 | 同上（边界类用例的构造） | 白盒比对类函数的反例要控制变量：只改一个语义字符，其余（含空白）逐字相同 | 采纳，已在本轮测试落实；不另改文档，归入「只改一个条件」的既有规则 |

**修复提交。** 00eaeeb（core，D-29 口径，tag `t07-review-3`）、1a5d7a7（runtime 修复与十三条补测试，tag `t12-review`、`t13-review`、`t14-review`、`t15-review-2`、`t16-review`）、c5d567c（core serde 边界）、本条记录所在的文档提交。

**M2 关闭。** T11 到 T16 全部 done；无退回实现者的待修项。下一次模型审查是 M3（端到端），复跑 `scripts/mutants.sh sheltie-runtime` 时存活应只剩 `home.rs:97` 一个等价突变（`failpoint` 与 `selfmgmt` 的六个由 T20、T23 的测试杀死）。

### M3 端到端（2026-09-25）

**范围。** `git diff 268ec6d..7008803`（T17 到 T23 的实现提交 71930b6、63dea79、72794ea、420157e、96c79f9、3c62ead、64c34ad，工具修复 767084b，审查提交 dc05b1d、67332b6、fe91704、7008803），`git log t16-review..HEAD -- tasks.toml` 的白名单修订（T18 加 `attempt.rs` 的 begin 分支、T20 加两份 specs 文档），tag `t20-review` 重设 T20 测试基准。实现者与复核者是同一会话的强模型（plan.md §0.2.1 的连做模式）。

**结论：需修改 → 本轮直接修复，通过，M3 关闭。** 两条实质缺陷（B1、B2）是一行级失配，当场修复；两条骨架与工具缺陷（B3、B4）按规则 9 例外处理并记录。T21 六条、T22 十三条场景测试启用即绿，未暴露实现缺陷——T06 到 T16 的分层测试与场景测试口径一致。CLI 层整体扎实：23 个错误码的 detail 一次对齐协议 §7，next 项按 §5 组装 `args`，文本与 JSON 双模式的输出口径统一（错误 JSON 走 stdout 供脚本解析，文本错误走 stderr）。

**检查表。**

| 项 | 结果 | 依据（命令均可重跑） |
| --- | --- | --- |
| 依据 | 通过 | CLI 行为逐条指回协议：响应封装与退出码（§5）、`work start` 第 8 步与 `attempt begin` 第 5 步的 data 字段（§3）、错误码与 `detail.*`（§7 全表）、`self` 组（§3 与存储合同 §9，axoupdater 不可行改走 D-30 并先改合同再写代码）。无文档外发明行为；`verify` 给半截引用按 §5「参数解析错误 2」拒绝，协议未定义半截语义 |
| 不变式 | 通过 | `INV-1/2`：CLI 只渲染 reply 与 `next`，不读输出文档、不替协调者选边（`two_step_via_cli_reaches_succeeded` 每步只从 `next` 取命令）；`INV-3`：写入全部在管理根之下，唯 `self install --modify-path` 按 §3 明文经用户主动要求写 rc（子进程临时 `$HOME` 测试钉住）；`INV-6`：批准人与时间取库里的记录（`gate.rs`），CLI 参数里没有任何摘要字段；`INV-7`：状态只从 `store.db` 读。`cargo tree -p sheltie-core -e normal --depth 1`：camino、serde、serde_json、sha2、thiserror、toml，无 I/O crate（T25 收口提交再贴全文） |
| 正反例 | 通过 | 23 个错误码每个都有触发与拒绝例（`workbook_add_invalid_dir…`、`work_start_missing_input…`、`begin_next_node_before_approve…`、`update_rejects_checksum_mismatch…`、`workbook_remove_without_version_exits_2` 等）；合法例与拒绝例成对（`same_request_id` 两条、`add_second_version` 的 latest 与默认版本） |
| 真实链 | 通过 | T17 到 T23 的 cli 测试全部经 `assert_cmd` 走真实二进制与临时目录；三个场景文件用 `examples/` 样例走全链；崩溃测试跑带 `failpoint` 特性的子进程 |
| 崩溃 | 通过 | T23 六条：`before_commit`、`after_commit_before_effects`、状态卡重建、`update_between_renames` + `rollback` 恢复；存储合同 §3 走查结论沿用 M2，runtime 未改语义（仅 selfmgmt 新增） |
| 边界 | 通过 | 参数边界各有拒绝例：`--input` 缺等号与键、`@file` 读不了、`<id>@<version>` 两侧空、`work` 前缀零匹配与多匹配（列候选）；路径与大小上限沿用 T12 到 T14 测试；未知字段拒绝沿用 M2 serde 收紧（内部标签枚举的限制照旧观察） |
| 文档 | 通过 | `check-docs.sh`（47 文件）绿；storage §9 与 D-30、plan T18/T20 卡、tasks.toml 四处同步改；M2 遗留的「`remove` 删目录失败只记日志」维持静默观察 |
| 提交 | 通过 | 一任务一提交，trailer 连排可解析（T17 到 T23 与修复提交逐条核对）；三次白名单修订与工具修复（767084b）意图未放松——`.rs`-only 让占位检查只落在 Rust 源码，白名单与测试零改动检查不变 |
| 突变 | 已处置 | 全量 `MUTANTS_TIMEOUT=300 scripts/mutants.sh sheltie-runtime`：216 个，128 杀死、21 存活、67 不可编译、0 超时。存活里除 M2 已判等价的 `home.rs:97` 外全在 T20 新增的 selfmgmt。补测八条（fe91704、7008803）后定向复跑 `-f selfmgmt.rs`：40 个，32 杀死、8 存活——7 个是联网分支（curl、is_local 的远端侧）归 T25 真实升级，1 个 `fsync → Ok(())` 在可写普通文件上无失败路径，黑盒不可测，靠 §9 的 fsync 明文背书 |
| 证据 | 附命令 | 全量 `cargo nextest run --all-features --no-tests=pass`：314 条通过、1 条跳过（T24），约 11.6 秒；每个场景下表带测试名 |

**规格 §7 场景对照（十三项逐行）。**

| 场景 | 测试（全部通过） |
| --- | --- |
| 无审查两步 Flow | `two_step_via_cli_reaches_succeeded`（走 `next` 链）；`two_step_runs_to_succeeded`（runtime 层） |
| 审查通过 | `human_executor_node_is_begun_and_submitted_like_agent`（提交「通过」后协调者选 `main` 边）；`next_after_review_offers_both_main_and_back_with_kinds`（两条边并列由协调者选） |
| 审查不通过 | `review_back_edge_creates_second_draft_occurrence`（`back` 边、`draft#2`、`review#2`）；`downstream_binds_latest_succeeded_occurrence_output`（旧产出留在 `attempts/draft/1/0` 不被覆盖） |
| 回环超限 | `max_visits_exhaustion_blocks_with_no_legal_edge`（`next` 不再含该边）；`submit_when_every_out_edge_target_hit_max_visits_blocks_no_legal_edge` |
| 人工门槛 | `gate_node_success_blocks_work_and_next_has_only_approve_and_cancel`、`begin_next_node_before_approve_is_illegal_next`、`approve_records_os_user_and_unblocks`、`approve_on_terminal_gate_node_succeeds_work` |
| 输入被改 | `modifying_upstream_output_makes_downstream_begin_fail_with_artifact_modified`；`begin_rejects_modified_upstream_artifact`（core 层） |
| 输出缺失 | `submit_without_required_output_is_output_missing_and_attempt_stays_running` |
| 重放 | `same_request_id_same_payload_returns_replayed_true`、`same_request_id_different_payload_is_request_conflict`；`start_replay_returns_same_work_id_without_new_seq`、`commit_replays_same_request_id_and_payload` |
| 中途被杀 | `kill_before_commit_leaves_state_unchanged_and_replay_succeeds`、`kill_after_commit_leaves_state_advanced_and_replay_returns_original_reply_and_rewrites_brief`、`status_card_missing_is_regenerated_on_next_write` |
| 删除被引用的 Workbook | `remove_in_use_workbook_is_rejected_with_work_list`、`remove_after_work_succeeds_then_status_still_renders`、`remove_allows_when_only_terminal_works_reference_version`；`begin_loads_graph_from_frozen_copy_not_repository`、`status_works_after_workbook_removed` |
| 参考文件作输入 | `review_brief_lists_checklist_resource_with_frozen_path`、`editing_repository_copy_does_not_change_running_work_brief`、`verify_detects_the_edit` |
| 升级后回滚 | `update_rejects_checksum_mismatch_and_leaves_binary_intact`、`rollback_swaps_prev_back`（回滚后 `.prev` 不在）、`kill_between_update_renames_leaves_prev_and_rollback_recovers` |
| 事实视图 | `stats_json_counts_visits_failures_and_entered_via`（`draft` 经 `entry×1, review×1`）、`work_stats_prints_table_and_json`、`begin_replay_regenerates_missing_stats_json`（`engine.stats` 文件与库中摘要逐字节一致，D-29） |
| 新人上手 | 人工，归 T25 |

**发现与处置。**

- **B1（T19，已修 dc05b1d）。** `gate approve` 的 data 缺 `at`：协议 §3 写明记录 `{ node, occurrence, by, at }`，实现只回前三个。`at` 与 `by` 一样从库里的批准记录取。
- **B2（T20，已修 67332b6）。** `self install` 把「建 store.db」放在幂等短路之后：`bin/` 里已有同字节二进制但库没建的场合（手工复制、半途删除）不会补库，与协议的顺序句相悖。`Store::open` 挪到短路之前。
- **B3（工具缺陷，已修 767084b，`Task: T01`）。** `check-task.sh` 检查 2 对目录条目只取 `.rs`、对文件条目却整文件 grep：T20 按规则把两份 specs 文档加进白名单后，decisions.md 里程碑记录里的 `todo!()` 字样被误判为占位。改成文件条目也只查 `.rs`，占位宏只可能出现在 Rust 源码里，检查意图不变。
- **B4（骨架顺序缺陷，T18 卡补记）。** T18 的 `work_cancel_then_any_write_is_work_terminal` 要走 `attempt begin` 拿 `WORK_TERMINAL` 封装，不先实现它只能撞 `todo!("T19")` 的 panic。按 T07、T13、T15 先例把 `attempt begin` 与共用的 `read_text_arg` 拉进 T18，T19 剩 `submit`、`fail` 与 `gate`。与 B1 类的注入点位置问题同根：T01 的依赖顺序核对没有覆盖 CLI 层任务。
- **D-30（合同修订先行）。** 存储合同 §9 原定用 `axoupdater` 读清单；核对其 0.10.2 公开 API（不暴露清单与 sha256、唯一完整入口执行安装脚本）后不可行，先改合同与决策记录再实现，不引新依赖。
- **夹具修复（T23，`allow_test_changes = true` 卡上预判）。** 全局 `~/.cargo/config.toml` 把 `target-dir` 指到 `~/.rust/target`，crash.rs 按 `../../target` 找二进制必然落空；改为在测试里 `cargo build --message-format=json` 直接问 cargo 要路径。
- **testkit.rs 过时注释（T21 提交内）。** 「T06 之前所有动作都会 `todo!()`」让目录白名单的占位扫描误命中且已不成立，改写。

**观察（不处置，记录在案）。**

- `self update` 的联网分支（`curl`、`is_local` 的远端侧、cargo-dist 完整清单的真实字段）离线测不了：定向复跑存活 7 个突变体，归 T25 从 `v0.1.0-rc` 真实升级核对，字段对不上改 `selfmgmt` 适配层。
- `output::ok` 走 `OkEnvelope` 的平铺 `next`，实际调用方（workbook 组、`work list`）都传空；Work 命令一律走带 `args` 的 `ok_work`。内部死路径，MVP 后可合并。
- `work stats` 为拿封装层的 `next` 再读一次状态卡：只读操作，多一次冻结副本加载，不影响正确性。
- 文本模式下 `workbook verify` 失败只打结果表，`WORKBOOK_TAMPERED` 的 message 只在 JSON 里；JSON 是协调者的主接口，可接受。
- `update` 的版本比较是字面相等：允许降级（`from > to` 也换），协议未定义排序语义，T25 若要禁再补合同。
- M2 遗留观察全部维持：`works_referencing` 跳过解不开的行、`verify` 不看非普通文件、内部标签枚举变体多余字段仍被忽略。

**流程教训。**

| # | 类别 | 证据 | 改哪 | 改成什么 | 处置 |
| --- | --- | --- | --- | --- | --- |
| L17 | 骨架 | B4：T18 测试要撞 `todo!("T19")`，T01 的「依赖顺序逐任务核对」没有覆盖 CLI 层任务 | `plan.md` T01 验证段已有明文 | 该核对按任务卡机械执行（启用本任务测试、确认 panic 都是本任务占位）；T18 卡已补记拉入关系。不再写自动化脚本：`check-tests.sh` 已知归属，脚本收益低于成本 | 采纳（T18 卡已改）；脚本化否决 |
| L18 | 合同 | D-30：合同把 `axoupdater` 写成既成事实，核对公开 API 后不可行，靠实现者停下来改合同才没硬塞 | `engineering.md` §1 | 合同与任务卡点名第三方库时，先核对其公开 API 能支撑合同的每一步再写入；写不进任务的先改合同 | 采纳，已补进 §1 |
| L19 | 测试 | T23：全局 `~/.cargo/config.toml` 的 `target-dir` 让按相对路径找构建产物的夹具全数落空 | `engineering.md` §3.2 | 测试要构建产物时问 cargo 要路径（`cargo build --message-format=json` 的 `executable`），不要猜 target 目录 | 采纳，已补进 §3.2 |

**修复提交。** dc05b1d（B1，`Task: T19`）、67332b6（B2，`Task: T20`）、fe91704（补测六条，`Task: M3`，tag `t20-review`）、7008803（`.tgz` 区段补测，`Task: M3`）、767084b（B3，`Task: T01`）、本条记录所在的文档提交。

**M3 关闭。** T17 到 T23 全部 done，规格 §7 十四行场景里十三项有通过的自动化测试、`新人上手` 归 T25（复审更正计数，见 M3 复审 F1）；全量 314 条测试约 11.6 秒全绿；仓库零 `todo!()`，`#[ignore` 只剩 T24 一处。剩 T24（skill）、T25（收口，含 `self update` 联网分支与 cargo-dist 真实清单的人工核对、`cargo tree -p sheltie-core` 贴进提交）、T26（真实宿主实测）。

### M3 复审（2026-09-26）

**范围。** 对 M3 报告的再审查，复审者未参与 T17 到 T23 与 M3。报告的每条结论独立取证、命令全部重跑，不采用自述。

**结论：通过，M3 维持关闭。** 十项检查表逐条核实成立；新发现五条（F1–F5），均为低危文档与残留问题，不影响关闭结论。F1、F2、F4 改文档，F3、F5 修复提交见末节。

**核实记录（均可重跑）。**

| 项 | 结果 | 证据 |
| --- | --- | --- |
| 场景对照 | 成立 | 报告对照表 36 个测试名逐一 grep 核实存在且分层正确；spec §7 实为 14 行（13 个自动化场景 + `新人上手`）。抽查六条关键测试的断言本体与场景口径一致 |
| 全量测试 | 成立 | 复审机上 `cargo nextest run --all-features --no-tests=pass`：314 通过、1 跳过（T24），20.2 秒（报告记 11.6 秒，机器差异） |
| 门禁 | 成立 | fmt、clippy `-D warnings`、`cargo deny`、check-docs（47 文件）、check-core-vocab、check-tests（315 测试）现跑现绿；零 `todo!()`、`#[ignore` 只剩 T24、无 `.snap.new` |
| 不变式 | 成立 | `cargo tree -p sheltie-core` 与报告逐行一致；CLI 层零 `std::fs` 写（INV-3）；`follow_begin` 实证测试只从 `next` 取命令（INV-1/2）；`gate.rs` 的 `by`/`at` 取库中记录（INV-6） |
| 错误码 | 成立 | 协议 §7 恰 23 码；core 与 runtime 的 `code()` 两个 `match` 无通配臂，`error_map` 穷尽 |
| 崩溃窗口 | 成立 | 三个故障点位置与 T23 卡一致：`commit.rs:53`、`service.rs:461`（确在 COMMIT 后、效果前）、`selfmgmt.rs:191`（两 rename 之间） |
| B1/B2 修复 | 成立 | dc05b1d 的 `at` 取自 `approvals.last()`；67332b6 的 `Store::open` 确在幂等短路之前 |
| 提交纪律 | 成立 | 十个提交 trailer 逐条在；tasks.toml 白名单修订恰为报告所述两处；767084b diff 最小、意图未放松；测试 diff 只有删 `#[ignore]` 行（T17 计 9 条、T18 计 12 条，与任务卡一致） |
| 突变 | 抽查一致 | 定向复跑 `mutants.sh sheltie-runtime -f selfmgmt.rs`：49 个，32 杀死、8 存活、9 不可编译；8 个存活逐条为 `is_local`×2、`curl`×3、`curl_to`×2（联网归 T25）与 `fsync`×1，与处置清单完全一致。全量 216 个未重跑，以定向复跑代替 |

**新发现与处置。**

- **F1（文档计数错，已修，本提交）。** spec §7 有 14 行，13 个自动化场景全部对上测试；M3 提交标题与关闭句的「十二」是数错（实质完整，报告自己的表就是 13 行全覆盖）。同段「T21、T22 的十三条」改为「T21 六条、T22 十三条」。历史提交信息不改。
- **F2（合同同层冲突，已修，本提交，D-31）。** 协议 §3 `self install` 的「同版本」与存储合同 §9 的「字节相同」冲突；实现与 M3 补测钉的是字节口径。统一为字节口径，协议改写，实现不动。
- **F3（残留文件，已修 8c5c7cf，`Task: T20`）。** `rollback` 把被丢弃的当前二进制 rename 成 `tmp/<uuid>` 后用 `remove_dir_all` 删；rustc 实证 `remove_dir_all` 对文件报 ENOTDIR，文件残留。改 `remove_file`。无正确性影响，属泄漏清理。
- **F4（注释陈旧，已修 dbebdcd，`Task: T20`）。** B2 之后 `self_cmd.rs` 头注「不打开 store.db」不再准确，更正为「除 install 建库外不打开」。
- **F5（静默兜底，已修 36e8577，`Task: T18`）。** `next_ops_of` 的 JSON 往返是恒等操作（`StatusCardJson.next` 本就是 `Vec<NextOp>`），`unwrap_or_default` 在不可能失败的路径上放了「出错给空 next」的兜底，规则 10 点名的模式。删函数，调用点直接取 `card.next`。

**维持观察（与 M3 一致，不处置）。** `self update` 联网分支归 T25；`output::ok` 的平铺 `next` 是内部死路径；`update` 版本比较字面相等允许降级；M2 各遗留观察。

## 首次真实运行

2026-09-26 至 27，宿主 Claude Code（配置目录 `~/.claude-glm/`），skill 手工装在 `~/.claude-glm/skills/sheltie/`，引擎 v0.1.0 经 install.sh 装入 `~/.sheltie/bin`。共三轮 Work，协调者都是新开的会话，只按 skill 与引擎响应驱动。记录前全部结论都在本机用引擎复核过（状态、review.md 首行、sha256 对照、stats）。

**Run 1（two-step，2026-09-26-003-local-engine，succeeded）。** prompt 指定了 article-review 并给了样例路径，协调者发现它没装后没有 `workbook add`，改用已装的 two-step 且未向用户声明替换——skill 的「用户没有指定 Workbook 时问用户」没覆盖「指定了但没装」。它还用一次必败的 `work start` 探测起始输入键，烧掉当日序号 002（留下空目录；序号不回收是存储合同 §7.1 接受的代价，但协调者注释「给少会被拒绝，不会产生副作用」与合同不符——`workbook show` 就能查输入键）。

**Run 2（article-review，2026-09-26-004-why-local-engine，succeeded）。** prompt 被宿主客户端拆条：「从严标准审」一句未送达，第一轮审查「通过」，无打回。收到明确指令后协调者正确执行了 `workbook add`；draft 与 review 派了独立 agent；读 `review.md` 第一行后选 `main` 边。publish（human 节点）的结论「确认发布」是用户给的，但复制 final.md 与跑 submit 由协调者代劳——任务书约定人自己提交，引擎按 D-04 不区分执行者，协调者越过的是任务书字面，不是引擎规则。

**Run 3（article-review，2026-09-26-005-local-workflow-engine-article，succeeded，含打回）。** 单行 prompt 完整送达。draft#1 → review#1「不通过」（用户要求第一轮加严）→ `back` 边 → draft#2（任务书带「来自： review#1（back 边）」行）→ review#2「通过」→ `main` → publish → succeeded。visits：draft 2/3、review 2/3、publish 1/1。协调者把「审查不通过」正确当作成功执行走 submit 而非 fail；收尾主动调了 `work stats`（`entered_via` 显示 draft 经 `review×1` 进入）。注意：article-review 样例的 draft 没有声明来自 review 的可选输入，打回意见是协调者按协议 §4 追加给改稿 agent 的，不是输入绑定——想让意见传递机制化的 Workbook 应给被打回节点声明 `required = false` 的 `<review>.<output>` 输入。

**四条观测结论。**

1. **是否只用了 `next` 里的命令：是。** 三轮全部写操作都在引擎给出的合法集合内；写之前的只读探查（list、show、status）不计。
2. **有没有试图绕过：引擎层面没有。** 不直接改 `~/.sheltie`、不跳过 submit、被拒后回到 `next`。两次越界都在引擎之外：run 1 替换用户指定的 Workbook 未声明；run 2、3 代劳 human 节点的机械动作（run 3 是用户在选项里明确选「由协调者代执行」）。
3. **任务书是否够用：够。** 三轮的工作 agent 都只靠任务书与绑定输入完成，没有回头要上下文；「来自」行在回环里被实际读到并引用。
4. **token 用量：宿主的 `/cost` 读数三轮都未采集成功。** 可记的代理指标：run 3 端到端约 22 分钟，`work stats` 的平均耗时 draft 136s、review 209s、publish 590s（publish 含人工确认等待）。

**完整性实证。** Run 2 撞见 003 报 `STORE_CORRUPT`：冻结副本目录被 Finder 落了一个 `.DS_Store`，摘要核对按存储合同 §5.1 拒绝加载——「输入按字节冻结」在真实环境按设计工作。副本顶层目录可写是注释明说的取舍（macOS 挪动/删除目录需要父目录可写，防篡改靠摘要核对），这次事件正是这条设计的实证；代价是副本被碰后该 Work 的 status 也读不了，记录为已知行为。

**路由（engineering §7）。** 改进项两条，都不阻塞 MVP，随下个版本：skill 补两句——「指定的 Workbook 没装时先 `workbook add`（素材路径由用户给），不得自行换用其他 Workbook」「探测输入键用 `workbook show`，不用必败的 `work start`」。`PATH` 在新会话三次都不可用、协调者三次都自行找到 `~/.sheltie/bin/sheltie` 恢复——宿主 shell 配置的传播问题，记录不改。
