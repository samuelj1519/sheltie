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

## 里程碑记录

待 M1、M2、M3 完成后填写。

## 首次真实运行

待 [计划](plan.md) T26 完成后填写：协调者是否只用了 `next` 里的命令、有没有试图绕过、任务书是否够用、宿主观测的 token 用量。
