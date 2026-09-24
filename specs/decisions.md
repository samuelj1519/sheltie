# 设计决定记录

每一条记录一个有争议或有替代方案的设计决定：背景、选择、被否决的方案、后果。新决定往后追加，不改已有条目的原文；被推翻的决定加一行「被 D-nn 取代」。规格与合同只写结论，理由都在这里。

格式：`D-nn 标题`。日期为拍板日期。

## D-01 边显式声明，合法下一步就是出边

2026-09-24。

**背景。** 协调者要在「主干、打回、修复旁支、再审」之间选路。图里必须能写出这些边，引擎必须能机械判断哪些边合法。

**选择。** Flow 用 `[[edges]]` 表显式声明 `from / to / kind`，`kind` 取 `main | back | branch | re_review`。合法下一步等于当前节点的全部出边，再按 `max_visits` 过滤。没有隐式「往下走」。

**否决。** 用 `depends_on` 表达 DAG 加 `routes` 表达条件路由。条件路由需要一套表达式语言与结构化结果字段，而自然语言结论的节点用不上它；回边在这种形式下无法直接写出。

**后果。** 图可以有环，靠 `max_visits` 保证有限。`kind` 对引擎只是标签，四种边的合法性判断相同。机器路由若将来需要，作为显式节点类型另立项（[路线图](roadmap.md)）。

## D-02 协调者选边，引擎不读内容

2026-09-24。

**背景。** 审查节点的结论「通过 / 不通过」决定走哪条边。谁来读这个结论。

**选择。** 结论是工作 agent 写进输出文档的自然语言，协调者读后在 `next` 里选一条。引擎只保证选的那条是声明过的边。`attempt begin <下一节点>` 这次调用就是选边，不另设推进操作。

**否决。** 引擎解析文档里的关键词自动推进。这违反 [宪章](constitution.md) `INV-1`、`INV-2`，且把业务判断锁死在引擎里。

**后果。** 引擎对任何业务领域零知识；换一个 Workbook 不改引擎。协调者选边不需说明理由，不被二次盘问。

## D-03 十一个 CLI 动词，每次响应带 `next`

2026-09-24。

**背景。** 协调者与引擎的交互面要小到 skill 一页纸能教完，又要覆盖领取、执行、提交、门槛、取消。

**选择。** `workbook add|list|show|remove|verify`、`work start|list|status|cancel`、`attempt begin|submit|fail`、`gate approve`，加 `self` 组五条管二进制自身。领取与准备合并为 `attempt begin`，返回任务书。每个响应带可直接执行的 `next`。

**否决。** 把领取、准备调用、读输入、开始、提交拆成独立操作。对本地文件系统上的单用户工具，这些分步没有独立价值，只增加协调者出错的机会。

**后果。** MCP 面若将来需要，只是同一组操作的薄封装。工具描述短，符合 Anthropic 关于工具接口的建议。

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

**选择。** `work_id = <UTC 日期>-<当日序号 001..999>-<名字>`，目录名等于 `work_id`。序号在独立短事务里按日期递增，分配后不回收。身份权威是 SQLite 行，目录名只是投影。

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

**否决。** 两个二进制（要先解决谁装安装器、版本对齐）；不做自更新（单机用户升级与回滚体验差）。

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

**选择。** `max_visits`（节点被到达次数，含回环）与 `max_retries`（同一次到达内的失败重试）。到顶就从 `next` 里拿掉对应项；所有出边目标都到顶时 `blocked(no_legal_edge)`。

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

**背景。** 回环里被打回的节点需要读审核意见，但第一次到达时还没有意见。所有输入都必需的话，这种节点写不出来。

**选择。** `inputs[].required = false`，只允许 `<node>.<output>` 来源。上游没有成功 Attempt 时不绑定，任务书标「尚无」。上游有了就绑最新一次。

**否决。** 让协调者在追加上下文里手工把意见贴进任务书。这把一件机械事推给协调者，弱协调者会漏。

**后果。** `Attempt.inputs` 的值类型变成 `Option<ArtifactRef>`；`start` 与 `resource` 来源上写 `required = false` 编译拒绝。

## D-17 `spec-dev` Workbook 的形状

2026-09-24。

**背景。** 需要一份可分发的软件开发 Workbook，人少插手、弱模型能跑、token 省。

**选择。** 八个节点。规划三步（`spec`、`plan`、`plan-review`），其中只有 `plan-review` 是人，用人审节点而不是 `gate`，因为人的结论要能把流程送回两个不同的上游。实现两步回环（`implement` 一次一个任务，`verify` 新会话独立跑门禁），任务数由 `verify` 报告第一行推进，靠 `max_visits` 限界。收尾两步（`review` 看整体 diff 一次，`deliver`）。`fix` 节点同时服务验证与审查两个回环，只修报告里第一行为「不通过」的那份。每份报告第一行是协调者的选边协议。门禁命令由 `plan` 从项目里抄进方案，后续每步照跑。

**否决。** 每个任务都审查一遍（token 翻倍，验证已经保证任务完成）；`gate = true` 代替人审节点（只能批准或取消，不能送回具体上游）；把整份任务清单交给一次 `implement`（弱模型在大任务上失败率高，失败时丢全部进度）。

**后果。** 一个 N 任务的需求要跑 `implement` 与 `verify` 各 N 次，每次上下文只有一条任务与几个文件路径。`max_visits` 24 与 32 限制了单个 Work 的规模；更大的需求在 `spec` 阶段拆。

## D-18 `deliver` 带门槛，CI 留在流程外

2026-09-24，用户同意。

**背景。** 最后一步由谁验收：人看一眼，还是交给 pre-commit、lint、GitHub Actions。

**选择。** `deliver` 节点 `gate = true`。人读 `delivery.md`，自己运行里面列出的对外动作（push、开 PR），再 `gate approve`。流程内不 push、不发布。CI 作为 push 后的冗余确认留在流程外；`plan` 把 CI 命令抄进「门禁」，流程内每个任务已跑过三遍同样的检查。

**否决。** 不设门槛、靠 CI 验收。CI 只能验机器可判的性质，验不了「做的是不是想要的」，且只在 push 之后才跑，而 push 本身是不可逆的对外动作，按 OpenAI 与 Anthropic 的建议应有人放行。

**后果。** 人在流程里出现两次：开头审方向，结尾接收并放行。中间全自动。

## D-19 卡住走人审旁支，任务书带「来自」行

2026-09-24，用户提出，采纳并扩展。

**背景。** 工作 agent 会遇到三类靠自己解决不了的事：缺只有人知道的信息、要做需要授权的高风险动作、同一任务反复修不过。`max_retries` 与 `max_visits` 耗尽只给 `work cancel`，会丢掉全部进度。

**选择。** `spec-dev` 加一个 `executor = human` 的 `escalate` 节点，`implement`、`fix`、`verify` 都有 `branch` 边进去；人写四个词之一（`继续 / 跳过 / 改方案 / 止损`）加意见，四条出边分别回 `implement` 或 `fix`、`implement`、`plan`、`deliver`。「修两轮不过」由报告里手递手传的 `修复轮次` 数字触发，`verify` 在轮次到 2 时写 `不通过，需要人`。「需要授权」由工作 agent 自报 `卡住` 并写清需要什么，人在决定里写授权范围，这段话作为下一步输入原样进任务书。

同时给引擎的任务书加「来自」行（上游 Occurrence 与边类型），`Attempt` 记 `entered_from`。原因：可选输入绑定后会一直带着上一次的内容，多入口节点必须知道自己是从哪条边来的才能决定读哪份。

**否决。** 靠引擎的 `blocked` 加 `cancel`（丢进度）；让协调者在聊天里问用户（不落文件，弱协调者会即兴处理）；用 `gate` 代替（只能批准或取消，不能带意见分四路）；由引擎在 `max_visits` 到顶时自动转人（引擎不知道该转到哪个节点，且违反「引擎不选路」）。

**关于越权。** 宿主层的工具权限提示（Claude Code 的 allow / deny）仍然是第一道拦截。Workbook 里的 `卡住` 约定处理的是任务范围外的动作，两者互补。引擎级的权限求值见 [路线图 GF-22](roadmap.md)。

**后果。** `spec-dev` 九节点二十边。人在流程里最多出现三次，其中 `escalate` 只在异常时出现。

## D-20 强模型搭骨架，初级实现者填空，机器当审查

2026-09-24。

**背景。** 实现者可能是首次接触项目的初级开发者或初级模型。要在低 token、少返工、高质量三者之间取综合成本最低。

**选择。** T01 由强模型一次性写出全部类型、签名、文档注释、`todo!()` 函数体、全部测试（禁用）、快照、脚本与样例。之后每个任务是「解开一组测试，填几个函数体，让它们变绿」。实现者不做设计、不写测试、不改签名。逐任务审查交给编译器、clippy、测试和 `scripts/check-task.sh`（改动文件白名单、无残留 `todo!()`、测试与快照未改）。模型审查只在三个里程碑做一次，配 `cargo mutants` 找没被测到的逻辑。

**为什么有效。** 初级实现者失败的地方集中在四处：发明类型与接口、读大量上下文、判断自己做完没有、处理事务顺序这类细微语义。骨架把前两项做掉，测试把第三项做掉，Rust 的类型系统与穷尽 `match` 把第四项的大部分交给编译器。剩下的工作是「给定签名、注释、失败的测试，写函数体」，这正是初级模型最稳的场景。每个任务的输入上下文压到 3k 到 8k token。

**否决。** 只把任务卡写得更细（散文再细也替代不了签名，而且比签名更长）；每个任务都做模型审查（贵，且审查者同样初级）；让实现者自己写测试（初级模型写的测试常常只验证自己的实现）；强弱模型逐任务配对（token 翻倍）。

**代价。** T01 是一次大投入，要求骨架作者对合同理解完整；骨架错了会连累多个任务。缓解：T01 的验收包括「`scripts/task.sh T02` 能跑出红」，M1 在 core 完成后立即做，错误不会拖到 runtime。

**后果。** `plan.md` 的每张任务卡从「先写的测试」改为「要变绿的测试」，多了「文件」白名单与十条实现者规则；新增 `tasks.toml`、`scripts/task.sh`、`scripts/check-task.sh`；新增 M1 到 M3 三个里程碑。

## D-21 `spec-dev` 采用同一套「骨架、填空、机器审查」；节点带 `tier` 标签

2026-09-24，用户提出，采纳。

**背景。** D-20 的做法只写在 `plan.md` 里，服务本仓库的开发。`spec-dev` Workbook 面向任意项目，实现者同样可能是初级模型，同样的问题会出现。

**选择。** `spec-dev` 在 `plan-review` 之后加 `scaffold` 节点（强模型）：把方案与任务清单变成一次骨架提交，全部类型、签名、注释、占位函数体、禁用的测试，并输出单任务测试命令。`implement` 与 `fix` 变成填空，附一份十条实现者规则作为输入。`verify` 用 `git diff` 机械核对改动只在白名单里、测试与快照未改、无残留占位。`review` 可跑突变测试。任务清单模板改为「只改哪些文件、要变绿的测试」。

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

**根因与补救。** T01 只用负例验证了门禁脚本，没有做过一次「按规则填满一个任务再跑门禁」的正例干跑。T01 的验收与 `spec-dev` 的骨架规则都补上这一条。

**否决。** 把工具缺陷也算「卡住」让人来修（人要看的是方向，不是 shell 脚本）；允许实现者改测试来配合工具（测试是合同，一旦可改就没有锚点）。

## D-24 反思放在 Workbook，引擎只给事实

2026-09-24，用户提出，采纳。

**背景。** T02 复核的价值大半来自审流程而不是审代码：发现工具缺陷、把它们变成规则、校准实现者档位。用户问要不要给引擎或 Workbook 加「反思与自我进化」。

**选择。** 分三层。引擎加一个只读投影 `work stats`（每节点到达、尝试、失败次数、平均耗时、进入来源；受阻与批准次数）与第四种输入来源 `engine.stats`，开工时把这份 JSON 写成 `stats.json` 按字节冻结绑给节点。`spec-dev` 加终点节点 `retro`（`standard`，带 `gate`）：读 `stats` 与各报告第一行，按六个闭集类别写 `lessons.md`，每条建议必须有证据（哪个 Attempt 的哪份文件哪一行）与落点（Workbook 内哪个文件哪一段），否则进「不建议改的」。人在最后一次批准时同时看交付说明与反思，采纳的改进下一版并在 README 修订记录里写「采纳 L1、L3，否决 L2」；下一次 `retro` 核对上一版建议是否见效。`gate` 从 `deliver` 挪到 `retro`。

**否决。** 引擎按统计自动调 `max_visits`、换 `tier`、跳节点（宪章 §7 否决的「更聪明的调度」，且同一版本行为不再一致，审计失效）；让 `retro` 直接改 Workbook（`INV-6`，权威来自人冻结）；给引擎加「反思」命令（引擎没有可反思的东西，只有事实）；`retro` 用 `strong`（它做的是归类与定位，判断留给看 `lessons.md` 的人）。

**依据。** 宪章 `INV-1`、`INV-2`、`INV-6`、§7。「自我进化」去掉「自我」：Workbook 提出对自己的修改，人冻结，引擎装新版本。差的那一步是人，这一步是产品定位里不能省的。

**后果。** `spec-dev` 十一节点二十四边。core 加 `render_stats`、`render_stats_json`、`Effect::WriteFile`、`InputSource::EngineStats`，保留字加 `engine`；协议加 `work stats`；plan 相关任务各加测试；M1 到 M3 加「流程教训」一节。`retro` 每次运行多一次 standard 调用，两三千 token。

## D-25 复核也要被复核

2026-09-24。

**背景。** T02 的实现者对复核意见写了一份反驳。核实后，反驳全部成立：复核有两处事实不实（说「删除行只有 `allow`、`ignore`、`todo`」，实际 `text.rs` 换了一行 `use`；说「改动只增不删」）、三处漏检（`check-task.sh` 第 124 行 `"$residue："` 里的全角冒号被 bash 当作变量名的一部分，`set -u` 直接中止，负例输出里的报错行被复核者忽略；`manifest.rs:114` 有骨架残留的 `#[allow(unused_variables)]`，按新检查 2 的语义会让 T03 按规则做完也过不了；检查 1 在工作树脏与净两种情形下只看一边）。另有两条判断偏窄：B1 只点了前导 `+`，前导零同样破坏解析回环；B3 的根因是 `kebab_id!` 宏用类型名当 `field`，与 `work_name`、`work_id` 的词汇不一致。

**根因。** 复核者只 grep 了输出里以 `check-task:` 开头的行，没读整段输出；只对一个文件做了 diff 核对，把结论推广到四个文件。两处都是「看了一部分，说成全部」。

**选择。**

1. 复核结论必须引用可重跑的命令与完整输出，不引用摘要。「我核对了」要写成「`git diff a b -- <文件> | grep '^-'` 的输出是 …」。
2. 反驳有效：实现者对复核的反驳与复核本身同等对待，逐条核实，成立的写进任务卡与本文。
3. 修法归属按引入者：骨架残留、宏词汇、合同措辞、`tasks.toml` 白名单由复核者（骨架作者）改；解析函数与 `check-task.sh` 的缺陷由实现者改。
4. 合同措辞要可测：B2 从「按 Unicode 汉字区段」改为逐区间列出十二个码点范围，测试对每个区间取一个码点。模糊措辞比近似实现更糟。
5. `tasks.toml` 加 `[T01]`：工具提交不得动 `crates/`，不再落在核对盲区。
6. 规则 3 明确：同文件私有辅助函数是实现细节，允许。原文没禁，实现者按最严解释自我批评，说明规则需要写清而不是靠猜。

**否决。** 撤回 T02 的「通过」（没有影响已实现行为的缺陷，十条测试绿，撤回没有依据）；把复核者的错误算成实现者的返工。

**后果。** T02 保持 `doing`，五条新测试待实现者解开；`check-task.sh` 两处由实现者修，`Task: T01`。M1 检查表加一项：复核结论是否附带可重跑的命令与完整输出。

## D-25 复核也要被复核：T02 的两轮往返

2026-09-24。

**背景。** 复核者（强模型）审 T02，结论「通过，两项待修」。实现者回复：结论可维持，但复核过程有两处事实不实、三处漏检，其中一处会让 T03 必然过不了门禁。逐条核实后全部成立。

**事实。** 复核声称「用 diff 核对了删除行只有 `#[allow]`、`#[ignore]` 与 `todo!`」，实际只查了一个文件，`text.rs` 还删了一行 `use`。复核声称「重跑负例仍能报错」，实际只 grep 了以 `check-task:` 开头的行，没看到脚本在 `set -u` 下因 `"$residue："`（全角冒号被 bash 吞进变量名）中止。复核没发现骨架在 `parse_manifest` 上留了一个没有 `todo!()` 的 `#[allow(unused_variables)]`，按新的检查 2 语义 T03 填完也过不了。复核把 `is_han` 缺区段的处理定为「改合同措辞」，实现者指出模糊措辞让合同失去可测性，应列出码点区间。复核把 B3 当单点文档失配，根因是 `kebab_id!` 宏用类型名当错误 `field`，与 `work_name`、`work_id` 词汇不一致。

**选择。**

1. 复核的验证义务写成硬规则：读工具的完整输出而不是 grep 一行；对「零改动」类主张用覆盖全部文件的机械 diff，并把命令贴进复核记录。
2. 合同不用模糊词换可测性：汉字区段在协议里列出十二个码点区间，实现逐区间一致，测试每区间取一个码点。
3. `tasks.toml` 加 `[T01]`：工具修复提交有自己的白名单，只准动脚本、计划、配置，不准动 `crates/`。此前标 `Task: T01` 的提交在机器核对的盲区里。
4. 规则 3 明确：同文件私有辅助函数允许。拆分是骨架的职责，但不禁止实现者为可读性加非 `pub` 的小函数。
5. 归属不变：实现者引入的（deb31ed 的全角冒号、检查 1 的互斥路径）由实现者修；骨架引入的（残留 `allow`、宏字段词汇、缺 `[T01]`）由骨架作者修。

**否决。** 因为复核不够严就取消逐任务复核（问题是复核的方法，不是复核本身）；让复核者顺手把实现者的缺陷一起修掉（归属混了，下次就分不清谁该对什么负责）。

**后果。** 任务卡多一行「复核的复核」。这一轮往返的成本约两次强模型调用，换来三处会在 T03 立刻爆炸的缺陷提前消除。

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
| 突变 | 已处置 | 首轮 524 个：281 杀死、145 幸存、98 不可编译。处置后 511 个：409 杀死、5 幸存、97 不可编译。5 个都有理由，见下 |
| 证据 | 附命令 | 下文每项带可重跑命令；首轮突变结果因目标目录共用作废过一次，已写成工具（L1） |

**幸存突变处置（首轮 145 个）。**

- 补测试杀死 134 个。`render.rs` 的时间换算、平均耗时与 `blocked` 计数 80 个：夹具时钟固定，快照里全是 `0s`、`blocked: 0`。`manifest.rs`、`parse.rs`、`compile.rs`、`ids.rs` 的上限与形状 38 个。`decide.rs` 的摘要、失败原因、输出大小边界 4 个。`blocked` 行的 `retries_exhausted` 与 `no_legal_edge` 两个分支。骨架里给 runtime 用的字面形式（`ErrorCode::as_str`、`Executor::as_str`、`Command::name`、`Timestamp::day`、各 newtype 的 `Display` 与 `From<_> for String`）。
- 删死代码消掉 6 个。`status_after_success` 里的 `!state.current_approved()`：批准只发生在门槛 Occurrence 的 Attempt 成功之后，此后该 Occurrence 不会再有 `Running` 的 Attempt，提交时这个条件恒真；`WorkState::current_approved` 随之删除。`next_attempt_id` 重复了 `decide_begin` 的编号逻辑，根因是骨架的 `bind_inputs` 签名拿不到 attempt id（文档注释却要求它算 `stats.json` 路径）；签名加 `attempt_id` 参数后删除。规则 5 的「来源节点不得是保留字」同样删除：解析层从不产生保留字来源的 `Node`，没有突变但属同类死代码。`lib.rs` 的 `#![allow(dead_code)]` 删除，clippy 无告警。
- 等价突变 4 个，保留。`next.rs:104` 的 `<` 换 `<=`：`Active` 且最新 Attempt `Failed` 时必有 `retry < max_retries`，否则已是 `Blocked(RetriesExhausted)`。`parse.rs:140` 的 `||` 换 `&&`：漏掉的形状随后被 `OutputName` 校验拒绝，报错规则相同。`render.rs:520` 的 `day - 1` 两个：常数偏移在 `secs_between` 的差里抵消。
- 测试夹具 1 个（`testkit.rs:273`），`scripts/mutants.sh` 起排除。

复跑：`scripts/mutants.sh sheltie-core`，输出末行 `mutants tested …: 4 missed`（上面四个等价突变）。

**其他核对。**

- `rg -n 'todo!\(|allow\(unused_variables\)' crates/sheltie-core/src` 零命中。core 里的 `#[ignore` 只剩 T11 的十一条与 B1 的一条，runtime、cli 的都属 T12 以后的任务。
- 工具改动三次：8b33c48（bash 3.2 兼容）、deb31ed（混合文件只比对测试模块、检查 2 收窄为只清 `todo!("Tnn")`、检查时机）、904a54c（范围取并集、全角冒号、空数组）。每次说明逐条写了原错与改法；deb31ed 收窄检查 2 的意图由里程碑的全仓零占位核对兜住，没有放松。

**待修。**

- **B1（T05，退回 `doing`）。** 被引用输出 `required = false` 而输入默认必需时，编译放行；上游不写该文件，`next` 仍给出 `attempt begin`，开工必报 `INPUT_UNAVAILABLE`，Work 只能取消。合同 §4 规则 5 与 `check_rule_5` 注释已补这一句；复核者补 `t05_rejects_required_input_on_optional_output`（禁用）与 `t05_accepts_optional_input_on_optional_output`。三份样例与 `spec-dev` 没有可选输出，不受影响。实现者解开测试、修，提交 `fix(core): 规则 5 拒绝把可选输出当必需输入`，M1 再复核这一条。

**遗留（M2 前由骨架作者补签名与测试，或由人拍板）。**

- **O1 `work start` 的 `requires`。** 协议第 8 步要 Workbook 声明的全部宿主资源，`Command::Start` 不带 manifest，core 只能给节点并集。选一：`Command::Start` 带 manifest 的 `requires`，或在 T16 骨架里写明由 runtime 用 manifest 覆盖。落点 T16。
- **O2 `resource.<path>` 的篡改检测。** 协议第 3 步说「重算 sha256 与已记录值核对」，但 `WorkState` 不记每个资源的摘要，`bind_inputs` 以观察为准，资源上的 `ARTIFACT_MODIFIED` 不可达。冻结副本只读，风险低；要么 `start` 时记下资源摘要，要么协议改成「冻结副本目录摘要与 `workbook.digest` 核对」。落点协议 §3 与 T16。
- **O3 任务书宿主资源表的「版本」列恒为 `-`。** `render_brief` 看不到 manifest，协议示例是 `^1`。`Graph` 编译时可把 manifest 的版本带进节点的 `requires`。落点 T05 骨架与快照。
- **O4 反序列化绕过构造校验。** `BoundedText` 是 `#[serde(transparent)]`，从库里读回超长摘要不会报错；`Graph` 可被反序列化出来，与「只能由 `compile` 构造」矛盾；`WorkName` 反序列化时静默规范化。runtime 从 `store.db` 读 `WorkState` 时 `STORE_CORRUPT` 因此漏检。落点 T13、T16 骨架。
- **O5 `work stats` 的 `blocked` 定义。** 协议只给了示例 `blocked: 1`。实现是「成功过的门槛 Occurrence 数 + 重试耗尽的 Occurrence 数 + 当前是否 `no_legal_edge`」，M1 已用测试钉住。需要人确认后写进协议 §3。
- **O6 `engine.stats` 序列化失败时写 `{"nodes":[]}`。** 实际不会失败，但失败时伪造内容与「引擎只记事实」相悖；应让它不可失败（手写 JSON）或把错误传出去。低优先，随 T16 一起改。
- **O7 协议 §4 与快照的措辞。** 「尚无（上游 X 还没有产出）」对快照「尚无」；「此 skill」对「此资源」；说明「逐字」对 `trim_end`。快照是 T10 的标准答案、合同权威更高，两者须对齐，改哪边由人定。

**流程教训。**

| # | 类别 | 证据 | 改哪 | 改成什么 | 处置 |
| --- | --- | --- | --- | --- | --- |
| L1 | 工具 | M1 第一次 `cargo mutants`（作废，不计入上面的首轮）：全局 `~/.cargo/config.toml` 设了 `target-dir`，并行副本共用产物，175 个「幸存」里多数是测试跑了未突变的二进制 | `scripts/mutants.sh`（新）；`plan.md` §0.4、M1、M2 卡 | 固定 `CARGO_TARGET_DIR=target`，用 nextest，排除 `testkit.rs` | 采纳，已改 |
| L2 | 骨架 | 9044612：`bind_inputs` 的注释要求算 `attempt_dir/stats.json`，签名却没有 attempt id，实现者只好写 `next_attempt_id` 重复编号逻辑 | `engineering.md` §3「写新测试的人」段 | 骨架注释要用到的值都必须能从参数得到，做不到改签名 | 采纳，已改 |
| L3 | 骨架 | B1：合同 §3.2 的「不得」没进 §4 清单，骨架与测试都跟着漏 | `engineering.md` §3 同段；§5「正反例」行 | 合同里每句「不得」「必须」都有一条拒绝例 | 采纳，已改 |
| L4 | 测试 | 首轮幸存里 `render.rs` 占 80 个：`stats_table_mid_flow` 快照全是 `0s`、`blocked: 0`，改错公式快照也不变 | `engineering.md` §3 同段 | 快照与断言里的数值字段至少一条非零、非默认值的断言；固定时钟下时间差单独造数据 | 采纳，已改 |
| L5 | 测试 | 首轮幸存里上限类 38 个同时存活 `>`→`==` 与 `>`→`>=`，说明连「多一个」的拒绝例都没有 | `engineering.md` §3 同段；§5「边界」行 | 每个上限一对：恰好上限接受、多一个拒绝 | 采纳，已改 |
| L6 | 实现者 | 9044612 `decide_start` 注释「未用到的 manifest 条目由 runtime 在返回前补」；`count_blocks` 的定义；`engine_stats_artifact` 的兜底串 | `plan.md` §0.2 规则 10 | 把「由 runtime 补」「解析不了当 0」「出错用默认内容」写进注释也算发明，同样要停 | 采纳，已改 |
| L7 | 顺序 | 9044612：T07 的夹具要用 submit、fail、render，实现者在 T07 里填了 T08 到 T10 的函数，T08 到 T10 只剩验收，没有「先看到红」 | `plan.md` T01「验证」段 | 骨架作者对每个任务核对：只解开本任务测试时 panic 的都是本任务的 `todo!`。M2 开工前对 T12 到 T16 做一次 | 采纳，已改 |
| L8 | 提交 | 9044612、4334925、810730f、8af5f94、43ec6bf：`Task`、`Agent` 与 `Co-Authored-By` 之间空行，git 不认作 trailer | `engineering.md` §4 | 所有 trailer 同一段，不空行 | 采纳，已改；`check-task.sh` 仍按文本 grep，不改（改成严格解析会让历史提交全部不合格，收益小） |
| L9 | 文档 | 本文 294 行与 315 行两个 `D-25`，写的是同一件事的两个版本 | 本文 | 合并为一条 | 否决自动处理：决策记录是历史，合并哪一版由人定；`check-docs.sh` 暂不加唯一性检查，等合并后再加 |
| L10 | 审查 | M1 卡要求「全仓零 `#[ignore`」，但 T11 以后的测试按设计仍禁用，复核者新加的待修测试也必须禁用 | `plan.md` M1 卡 | 改为「本里程碑覆盖的任务标签为零」 | 采纳，已改 |

**修复提交。** 本条记录所在的 M1 提交（补测试、删死代码、合同与流程修订，打 tag `t05-review-3`）；待 T05 的 `fix(core)`。

## 首次真实运行

待 [计划](plan.md) T26 完成后填写：协调者是否只用了 `next` 里的命令、有没有试图绕过、任务书是否够用、宿主观测的 token 用量。
