# spec-dev：规格驱动的软件开发

简体中文 | [English](../spec-dev/README.md)

从一句需求到一组已验证、已提交、已审查的小改动。人固定出现两处：开头拍板规格与方案，结尾读交付说明并批准对外。中间由 agent 自动推进，卡住时才找人。

## 流程

```text
spec ─▶ plan ─▶ plan-review(人) ─▶ scaffold ─▶ implement ─▶ verify ─▶ review ─▶ deliver ─▶ retro(gate)
 ▲        ▲        │  │               │            ▲ │          │ ▲ │       │ ▲        ▲
 └────────┴─back───┘  └─────main──────┘            │ │          │ │ │       │ │        │
                                                   │ │    branch│ │ │       │ │        │
                                                   │ │          ▼ │ │       ▼ │        │
                                                   │ │         fix ◀┼───────┘ │        │
                                                   │ │          │   │ branch  │        │
                                                   │ │          └───┼─re_review─▶ review
                                                   │ │              │                  │
              卡住 branch（scaffold/implement/fix/verify）─▶ escalate(人)              │
                                                       │                               │
              继续 → 回卡住的那步    跳过 → implement    改方案 → plan    止损 → deliver ┘
                （scaffold / implement / fix / verify）
```

十一个节点、二十五条边。人必经两处：`plan-review` 与 `retro` 的门槛（同时看交付说明与反思）；另有 `escalate` 一处，只在卡住时。

scaffold、implement、verify 从任务书的 `approval_rules` 输入读取共享审批更正规则，随本 Work 冻结。普通批准核对的位置仍在各节点开工前。

| 步骤 | 谁 | 产出 | 一句话 |
| --- | --- | --- | --- |
| `spec` | 强模型 | `spec.md` | 把需求写成可验收的规格；问题先给推荐答案 |
| `plan` | 强模型 | `plan.md`、`tasks.md` | 技术方案、门禁命令、三到二十个填空式任务 |
| `plan-review` | 人 | `decision.md`、被审方案与任务清单的原字节副本 | 第一行 `通过 / 修改规格 / 修改方案`，三份输出都要提交 |
| `scaffold` | 强模型 | `scaffold.md` 加一次骨架提交 | 全部类型、签名、注释、占位函数体、禁用的测试。最后一个做设计的步骤 |
| `implement` | 标准模型 | `change.md` | 一次一个任务：启用测试、填占位体、过门禁、提交。第一行 `完成 Tnn / 卡住 Tnn` |
| `verify` | 标准模型（新会话） | `report.md` | 自己跑单任务测试与门禁，按本任务的基线与候选核对改动、占位与批准版本；第一行 `通过，下一任务 Tnn / 通过，全部完成 / 不通过 / 不通过，需要人` |
| `fix` | 标准模型 | `change.md` | 只修报告里的发现，一次提交。第一行 `修复完成 / 卡住` |
| `escalate` | 人 | `decision.md` | 卡住、修两轮不过、需要授权时找人。第一行 `继续 / 跳过 / 改方案 / 止损` |
| `review` | 强模型（新会话） | `report.md` | 对原始基线到 HEAD 的整体 diff 审查，可跑突变测试；第一行 `通过 / 不通过` |
| `deliver` | 标准模型 | `delivery.md` | 做了什么、怎么验、还欠什么、对外动作的命令 |
| `retro` | 标准模型，带 `gate` | `lessons.md` | 读引擎的事实视图与各报告第一行，写出对本 Workbook 的具体修改建议。人批准后 Work 结束 |

「强模型」「标准模型」来自节点的 `tier` 字段，任务书与 `next` 里都会显示。协调者按它选派模型：`strong` 派最强的，`standard` 派便宜的。

## 给协调者

每份报告的**第一行**就是选边依据。对照表：

| 当前节点 | 报告第一行 | 下一步 |
| --- | --- | --- |
| `plan-review` | `通过` | `attempt begin scaffold` |
| `scaffold` | `完成` | `attempt begin implement` |
| `scaffold` | `卡住` | `attempt begin escalate` |
| `plan-review` | `修改规格` | `attempt begin spec` |
| `plan-review` | `修改方案` | `attempt begin plan` |
| `verify` | `通过，下一任务 Tnn` | `attempt begin implement` |
| `verify` | `通过，全部完成` | `attempt begin review` |
| `verify` | `不通过` | `attempt begin fix` |
| `verify` | `不通过，需要人` | `attempt begin escalate` |
| `implement` | `完成 Tnn` | `attempt begin verify` |
| `implement` | `卡住 Tnn` | `attempt begin escalate` |
| `fix` | `修复完成`，针对验证报告 | `attempt begin verify` |
| `fix` | `修复完成`，针对审查报告 | `attempt begin review` |
| `fix` | `卡住` | `attempt begin escalate` |
| `escalate` | `继续` | 回到进入 `escalate` 之前的节点：`attempt begin scaffold`、`implement`、`fix` 或 `verify`（看状态卡 `done` 的倒数第二项） |
| `escalate` | `跳过` | `attempt begin implement` |
| `escalate` | `改方案` | `attempt begin plan` |
| `escalate` | `止损` | `attempt begin deliver` |
| `review` | `通过` | `attempt begin deliver` |
| `review` | `不通过` | `attempt begin fix` |
| `deliver` | 交付说明写好 | `attempt begin retro` |
| `retro` | 提交后 Work 为 `blocked(gate)` | 通知用户读 `delivery.md` 与 `lessons.md`；用户 `gate approve` |

`verify` 与 `review` 要派给**新的工作 agent**，不要复用实现者的会话。这样验证与审查才独立。

到 `escalate` 时通知用户：谁卡住了、卡在哪（读 `change.md` 或 `report.md` 的备注一句话转述）、决定写到哪。不要替用户决定。

`implement` 每次只做一个任务。任务清单有 N 个任务，`implement` 与 `verify` 就各跑 N 次。`max_visits` 上限是 24 与 32；更大的需求先拆。

## 重规划交接

plan-review 总是保存本次被审 plan/tasks 的原字节副本。下一次 plan 的任务书会绑定 previous_plan、previous_tasks，以及可用的 previous_verification、previous_change、previous_fix_change；新 worker 只需这些路径与 project Git 即可恢复整体原始基线和已验证前缀。

实现和修复记录原样携带已有验证行，独立 verify 在核 Git、批准版本、门禁和原始证据之后追加本任务。planner 沿引用逐行核对，缺字段、漏行或基线变化时停止，不能从旧任务清单或聊天猜完成事实。尚未验证的当前变更只交接旧前缀。

重规划保留原始基线和历史验证行。验收条件变化时列出需重验事项，经过新 plan-review 批准后执行重验。表只保存路径与哈希引用，不复制全部报告正文。测试名描述行为，任务归属单独记录，不强制 tNN 前缀。

本次定义用独立管理根回归。已安装的同 id/version 不被覆盖，已有 Work 继续使用自己的冻结 Workbook，不能原地改写其图或说明书。

## 给用户

```bash
sheltie workbook add workbooks/spec-dev-zh-CN
sheltie work start --workbook spec-dev-zh-cn --flow default \
  --name "导出为CSV" \
  --input request="在报表页加一个导出 CSV 按钮，导出当前筛选结果" \
  --input project=/abs/path/to/repo
```

然后在 Claude Code 里 `/sheltie`，选这个 Work。流程必经规划审核和最终批准两个位置；重规划会再次审核，卡住时另走升级：

1. `plan-review`。读 `spec.md`、`plan.md`、`tasks.md`，把实际被审 plan/tasks 原字节复制到 reviewed-plan/reviewed-tasks，核摘要相同，再写 decision.md。通过或打回都提交三份输出。
2. `escalate`，只在 agent 卡住、同一任务修两轮不过、或要做需要授权的事时。读它的报告，写四个词之一加意见。授权要写清范围。
3. `retro` 之后。读 `delivery.md`，满意就运行里面列的对外动作（push、开 PR），再 `sheltie gate approve <work> --node retro`。不满意就 `work cancel`，或者手工改完再批准。顺手读 `lessons.md`：每条建议指向这份 Workbook 的一个文件一段，采纳的就改进下一版，在 README 修订记录里写「采纳 L1、L3，否决 L2（原因）」。

流程本身不 push、不开 PR、不发布。项目自己的 CI 在 push 后照常运行，是流程外的第二道确认；`plan` 抄进「门禁」的命令应与 CI 一致，这样 CI 很少再报出新问题。

## 为什么这么设计

- **人只在两头。** 开头审方向，结尾接收并放行对外动作。中间每一步都有机械判据（门禁、验收标准、审查清单），agent 判得比人快且不累。这符合 OpenAI 与 Anthropic 的建议：人工介入放在高风险与不可逆的决定上，其余自动。
- **Workbook 靠反思演进，不靠引擎自动调参。** `retro` 只写建议，每条有证据有落点；人决定采纳哪些，改出新版本，重新 `workbook add`，之后的 Work 用新版。引擎不会自己改 `max_visits`、换 `tier`、跳节点，同一版本在每次运行里行为相同。
- **CI 在流程外。** 流程内已经跑了三遍门禁，CI 是 push 后换干净环境的再确认，不是验收。让 `plan` 抄 CI 的命令，两边就不会打架。
- **设计集中在强模型手里，实现者只填空。** 初级模型失败的地方集中在四处：发明类型与接口、读大量上下文、判断自己做完没有、处理细微语义。`scaffold` 把类型、签名、注释、测试一次写好，前两项就没了；测试预写，第三项没了；语言的类型系统与穷尽匹配接住第四项的大半。实现者拿到的是「签名、注释、失败的测试」，这是初级模型最稳的场景。
- **一次一个任务。** 每次 `implement` 只带一条任务、几个文件、十来个测试，上下文 3k 到 8k token，出错也只丢一个任务。
- **审查交给机器，模型审查只做一次。** 编译器、测试、门禁抓逐任务的错；`verify` 按每任务的基线与候选用 `git diff` 机械核对没改白名单外的文件、没动测试与快照、没留本任务的占位。`review` 只在最后看一次原始基线到 `HEAD` 的整体 diff，可以跑突变测试找没被测到的逻辑。
- **按 `tier` 派模型。** 四个节点标 `strong`（`spec`、`plan`、`scaffold`、`review`），`escalate` 需要判断、由人做，其余 agent 节点 `standard`。一个需求里强模型只用几次，便宜模型跑几十次。
- **验证者与实现者分开。** 实现者报告不可信，验证者自己跑命令。验证只判「做完没」，审查只在最后看整体，避免每个任务都审一遍的 token 开销。
- **报告第一行是协议。** 协调者不用读全文，看一行选边。修复者不用问，看「发现」动手。
- **卡住就找人，不硬做，不绕过。** 缺信息、需要授权、环境坏了、修两轮不过，四种情况都走 `escalate`。人的回答写成文件，给下一步当输入，不是聊天里的一句话。两轮的上限靠报告里手递手传的 `修复轮次` 数字，简单到不会算错。
- **任务书的「来自」行决定读哪份过程输入。** 可选输入会一直保留上一次的内容，节点只读「来自」指向的那份，其余当过期忽略。这样防串台，不靠模型自己判断新旧。例外是 `decision`：它是批准记录，每一步都核它写明的获批版本（规格与方案的 sha256），不随「来自」变；`escalation` 只在「来自」是 `escalate` 时读。
- **门禁由方案写死。** 命令从项目里抄，不由每个实现者现场猜。
- **一切走文件。** 规格、方案、任务、报告都是文件，任务书只给路径。没有任何一步需要把上一步的全文贴进对话。

## 明确成果

最终门槛批准后运行 `sheltie work result <work> --json`，取得终点 retro 明确选择的 `delivery` 和 `lessons`。delivery 引用 retro 开工时冻结的交付说明输入，lessons 引用该 Attempt 封存的反思输出；门槛未批准时成果集合为空。取得文件不代表已执行交付说明中的对外动作。

## 修订记录

每次采纳 `retro` 的建议出新版本，在这里记一行：版本、采纳了哪些 `Ln`、否决了哪些与原因。`retro` 下次运行会读这一节核对效果。

- 0.2.2 共享审批更正规则由三个实际 Node 输入绑定；终点选择 delivery 与 lessons，批准后通过 work result 取得。已有 Work 的 0.2.1 冻结副本不变。
- 0.1.0 初版。
- 0.2.1 修 C002 真实宿主回归发现的反思报告定位：按当前 WorkLayout 的 `occurrence-<NNN>/attempt-<NNN>/outputs/` 读取历史报告，Occurrence 和 retry 分别补三位零。已运行 Work 的冻结副本保留原字节。
- 0.2.0 修 C002 审查确认的三处交付闭环缺口（O09/O10/N08）：任务验证改按「任务基线..候选」核对，占位体带任务编号、只查本任务；`plan-review` 决定写批准版本摘要并交给 `scaffold`/`implement`/`verify` 逐条核对；整体审查固定用 Work 原始基线，改方案不再重设。这一版不是 `retro` 建议，记在这里是为了版本可追溯。

## 语言版本

此目录是持续维护的简体中文方法；英文为默认语言，保留原 ID 并升级版本。中文方法 ID 追加 `-zh-cn`，可与英文版本并存。新的 Git 提交摘要和正文统一使用英文，中文说明语言不改变这项约定。已有 Work 保留其原始冻结字节；本目录不作为原始历史证据。

语言选择链接用于源码阅读；安装或冻结的说明与参考文件保持所选语言，未包含另一种语言的同级目录。
