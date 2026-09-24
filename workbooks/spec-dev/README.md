# spec-dev：规格驱动的软件开发

从一句需求到一组已验证、已提交、已审查的小改动。人在两处出现：开头拍板规格与方案，结尾读交付说明并批准对外。中间全部由 agent 自动推进。

## 流程

```text
spec ─▶ plan ─▶ plan-review(人) ─▶ scaffold ─▶ implement ─▶ verify ─▶ review ─▶ deliver(gate)
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
```

十个节点、二十三条边。人出现在三处：`plan-review`（必经）、`escalate`（只在卡住时）、`deliver` 的门槛（必经）。

| 步骤 | 谁 | 产出 | 一句话 |
| --- | --- | --- | --- |
| `spec` | 强模型 | `spec.md` | 把需求写成可验收的规格；问题先给推荐答案 |
| `plan` | 强模型 | `plan.md`、`tasks.md` | 技术方案、门禁命令、三到二十个填空式任务 |
| `plan-review` | 人 | `decision.md` | 第一行 `通过 / 修改规格 / 修改方案` |
| `scaffold` | 强模型 | `scaffold.md` 加一次骨架提交 | 全部类型、签名、注释、占位函数体、禁用的测试。最后一个做设计的步骤 |
| `implement` | 标准模型 | `change.md` | 一次一个任务：解开测试、填占位体、过门禁、提交。第一行 `完成 Tnn / 卡住 Tnn` |
| `verify` | 标准模型（新会话） | `report.md` | 自己跑单任务测试与门禁，核对没改白名单外的文件与测试；第一行 `通过，下一任务 Tnn / 通过，全部完成 / 不通过 / 不通过，需要人` |
| `fix` | 标准模型 | `change.md` | 只修报告里的发现，一次提交。第一行 `修复完成 / 卡住` |
| `escalate` | 人 | `decision.md` | 卡住、修两轮不过、需要授权时找人。第一行 `继续 / 跳过 / 改方案 / 止损` |
| `review` | 强模型（新会话） | `report.md` | 对基线到 HEAD 的整体 diff 审查，可跑突变测试；第一行 `通过 / 不通过` |
| `deliver` | 标准模型，带 `gate` | `delivery.md` | 做了什么、怎么验、还欠什么、对外动作的命令。人批准后 Work 结束 |

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
| `escalate` | `继续` | 回到进入 `escalate` 之前的节点：`attempt begin scaffold`、`implement` 或 `fix`（看状态卡 `done` 的倒数第二项） |
| `escalate` | `跳过` | `attempt begin implement` |
| `escalate` | `改方案` | `attempt begin plan` |
| `escalate` | `止损` | `attempt begin deliver` |
| `review` | `通过` | `attempt begin deliver` |
| `review` | `不通过` | `attempt begin fix` |
| `deliver` | 提交后 Work 为 `blocked(gate)` | 通知用户读 `delivery.md`；用户 `gate approve` |

`verify` 与 `review` 要派给**新的工作 agent**，不要复用实现者的会话。这是独立性的来源。

到 `escalate` 时通知用户：谁卡住了、卡在哪（读 `change.md` 或 `report.md` 的备注一句话转述）、决定写到哪。不要替用户决定。

`implement` 每次只做一个任务。任务清单有 N 个任务，`implement` 与 `verify` 就各跑 N 次。`max_visits` 上限是 24 与 32；更大的需求先拆。

## 给用户

```bash
sheltie workbook add workbooks/spec-dev
sheltie work start --workbook spec-dev --flow default \
  --name "导出为CSV" \
  --input request="在报表页加一个导出 CSV 按钮，导出当前筛选结果" \
  --input project=/abs/path/to/repo
```

然后在 Claude Code 里 `/sheltie`，选这个 Work。你一定会被找两次，偶尔第三次：

1. `plan-review`。读 `spec.md`、`plan.md`、`tasks.md`，写 `decision.md`，按任务书末尾的命令提交。
2. `escalate`，只在 agent 卡住、同一任务修两轮不过、或要做需要授权的事时。读它的报告，写四个词之一加意见。授权要写清范围。
3. `deliver` 之后。读 `delivery.md`，满意就运行里面列的对外动作（push、开 PR），再 `sheltie gate approve <work> --node deliver`。不满意就 `work cancel`，或者手工改完再批准。

流程本身不 push、不开 PR、不发布。项目自己的 CI 在 push 后照常运行，是流程外的第二道确认；`plan` 抄进「门禁」的命令应与 CI 一致，这样 CI 很少再报出新问题。

## 为什么这么设计

- **人只在两头。** 开头审方向，结尾接收并放行对外动作。中间每一步都有机械判据（门禁、验收标准、审查清单），agent 判得比人快且不累。这符合 OpenAI 与 Anthropic 的建议：人工介入放在高风险与不可逆的决定上，其余自动。
- **CI 在流程外。** 流程内已经跑了三遍门禁，CI 是 push 后换干净环境的再确认，不是验收。让 `plan` 抄 CI 的命令，两边就不会打架。
- **设计集中在强模型手里，实现者只填空。** 初级模型失败的地方集中在四处：发明类型与接口、读大量上下文、判断自己做完没有、处理细微语义。`scaffold` 把类型、签名、注释、测试一次写好，前两项就没了；测试预写，第三项没了；语言的类型系统与穷尽匹配接住第四项的大半。实现者拿到的是「签名、注释、失败的测试」，这是初级模型最稳的场景。
- **一次一个任务。** 每次 `implement` 只带一条任务、几个文件、十来个测试，上下文 3k 到 8k token，出错也只丢一个任务。
- **审查交给机器，模型审查只做一次。** 编译器、测试、门禁抓逐任务的错；`verify` 用 `git diff` 机械核对没改白名单外的文件、没动测试与快照。`review` 只在最后看一次整体 diff，可以跑突变测试找没被测到的逻辑。
- **按 `tier` 派模型。** 五个节点标 `strong`（`spec`、`plan`、`scaffold`、`review`，以及需要判断的 `escalate` 由人做），其余 `standard`。一个需求里强模型只被调用几次，便宜模型跑几十次。
- **验证者与实现者分开。** 实现者报告不可信，验证者自己跑命令。验证只判「做完没」，审查只在最后看整体，避免每个任务都审一遍的 token 开销。
- **报告第一行是协议。** 协调者不用读全文，看一行选边。修复者不用问，看「发现」动手。
- **卡住就找人，不硬做，不绕过。** 缺信息、需要授权、环境坏了、修两轮不过，四种情况都走 `escalate`。人的回答写成文件，作为下一步的输入，不是聊天里的一句话。两轮的上限靠报告里手递手传的 `修复轮次` 数字，简单到不会算错。
- **任务书的「来自」行决定读哪份输入。** 可选输入会一直保留上一次的内容，节点只读「来自」指向的那份，其余当过期忽略。这是防串台的机制，不靠模型自己判断新旧。
- **门禁由方案写死。** 命令从项目里抄，不由每个实现者现场猜。
- **一切走文件。** 规格、方案、任务、报告都是文件，任务书只给路径。没有任何一步需要把上一步的全文贴进对话。
