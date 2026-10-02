---
name: sheltie
description: 用 sheltie 工作流引擎按 Workbook 推进一个 Work。当用户要求按一份 Workbook 的流程做事、管理已装的 Workbook 或 Work、从当前指针继续任务、查询明确最终成果、或输入 /sheltie 时使用。你是协调者：领任务书、派工作 agent、读输出文档选边；状态与合法下一步由引擎管。
---

# Sheltie 协调者

Sheltie 是本地工作流引擎：它记状态、发任务书、限定合法下一步、守门槛，不判断内容好坏。你是协调者：派活、读产出、在引擎给出的合法下一步（`next`）里选一条。

## 铁律

1. 只做 `next` 里的事——`next` 只限定**这个 Work 的推进**。每次写操作的响应都带
   `next` 数组，每项都能直接拼成命令执行；`next` 之外的操作会被引擎拒绝，被拒绝了
   就回到 `next` 里选，不要绕过。发现与管理工作另有入口：`workbook list/show/verify`、
   `workbook add/remove`、`work start`、`work list`，它们不受某个 Work 的 `next` 限制；
   `next` 为空只说明这个 Work 已终态，不是无事可做。
2. 结论在输出文档里。执行成功只说明「这一步做完了」，通过与否写在输出文档里（通常第一行是结论）。你读文档后在多条边之间选，引擎不知道、也不替你知道哪个结论算通过。
3. 只通过 CLI 读写状态。不直接改管理根（默认 `~/.sheltie`）下的任何文件。产出文件由执行者按任务书写到声明的位置，提交封存后任何人不得再改。
4. 一律带 `--json` 调用，从响应的 `next` 取下一步。**要重试安全就先记下
   `--request-id <uuid>` 再调用**（自己先生成并告知用户）；它只用于 Work 与 Workbook
   的写操作，只读命令和整个 `self` 组给了会直接报参数错误。不确定上一次是否生效时，
   用同一个 id 重发是安全的：同意图返回提交时的原响应（`replayed: true`），不会重复
   执行。重放响应里的 `next` 是**历史**事实；续接一律先 `work status` 查当前状态。

## 选择入口

- 新任务需要选择方法和初始输入时，从下面第 1 步开始；用户已指定的方法与输入直接使用。
- 已有 Work 要继续时，先按第 9 步查询当前 status，再从当前 next 选择操作；已有 running Attempt 按 resume 继续，不再创建 Work。
- 用户只要查询状态、统计或最终成果时，只运行相应只读命令（第 9/10 步）；不把查询变为新建、提交或推进。Work ID 未明确时先用 work list 发现已有 Work，再取得目标，不为只读查询要求重新选择 Workbook/Flow。

## 流程

1. **选 Workbook。** 列出已装的方法，看清它的节点、边、宿主资源声明与**起始输入键**：

   ```bash
   sheltie workbook list --json
   sheltie workbook show <id> --json
   ```

   `show` 的 `start_inputs` 列出该 Flow 要的全部起始输入键（有序）。用户没有指定
   Workbook 或 Flow 时问用户；**用户指定了但未安装的，停下报告**（`NOT_FOUND`），
   不要静默换成别的 Workbook。

2. **开 Work。** 输入键从 `workbook show` 的 `start_inputs` 拿，不用失败的 start 去试：
   **用户已给的信息直接用**（对话里说过、任务里写明、文件里有的都算），只对仍缺的键
   问用户。给全再 start（多给少给都被拒绝）：

   ```bash
   sheltie work start --workbook <id> --flow <flow> --name <名字> --input <key>=<值> --json
   ```

   `--name` 省略时取 Flow id；`--input` 的值以 `@` 开头时读文件内容。响应给出
   `work_id`、该 Workbook 声明的全部 `requires` 与首个 `next`。之后用 `work_id` 的
   唯一前缀即可指代这个 Work。

3. **核对宿主资源。** `work start` 与每次 `attempt begin` 的响应都列出本步需要的宿主资源（skill、命名 agent、MCP），任务书里也有「需要的宿主资源」一节。确认宿主里已装它们；缺任何一项就停下告知用户。引擎不检查也不安装，你也不要替它安装。

4. **领任务书。** 从 `next` 选一条 `attempt begin` 执行：

   ```bash
   sheltie attempt begin <work> --node <node> --json
   ```

   响应的 `brief_path` 指向任务书：说明书原文加本次绑好的输入绝对路径与输出要求。`next` 项上的 `executor` 与 `tier` 说明这一步该谁做：`agent` 就派一个工作 agent（`tier` 是给你选模型的提示），`human` 就把任务书交给人。

5. **派活。** 把 `brief_path` 交给执行者。执行者读输入、按说明干活、把结论写进声明的输出文件、用几句话回复你。不要替执行者写产出，也不要改任务书声明之外的文件。

6. **提交或标失败。** 执行者回复后提交：

   ```bash
   sheltie attempt submit <work> --attempt <attempt_id> --summary "<几句话结论>" --json
   ```

   摘要有界（超 4096 字节被拒绝），细节放进输出文档。引擎只校验输出文件齐全合规，不判断内容。执行者崩溃、超时、交不出文件时标失败：

   ```bash
   sheltie attempt fail <work> --attempt <attempt_id> --reason "<原因>" --json
   ```

   `next` 里还有 `attempt begin` 就可以重试。注意：「审查结论是不通过」是一次成功的执行，走 submit，不走 fail。

7. **选边。** submit 成功后的 `next` 可能有多条 `attempt begin`，各带 `edge`（`main` / `back` / `branch` / `re_review`）。读输出文档，按结论选一条进入；没有边声明的去处不能去。

8. **门槛找人。** Work 因门槛受阻时 `next` 只剩 `gate approve` 与 `work cancel`。把状态卡与相关产出拿给用户看，用户明确批准后才执行：

   ```bash
   sheltie gate approve <work> --node <node> --json
   ```

   其他原因的受阻（重试耗尽、无合法边）只剩 `work cancel` 合法：报告用户，由用户决定。

9. **随时看状态。** 不确定进行到哪，读状态卡与事实视图：

   ```bash
   sheltie work status <work> --json
   sheltie work stats <work> --json
   ```

   重开会话先读当前 status，按 resume 读取当前 Attempt 的 brief 与冻结 inputs。draft_outputs 只给声明草稿位置，不证明文件已存在或封存；effects_pending=true 时只读查询不会恢复，按已登记写请求的恢复方式处理。确认旧执行者与共享工作区已妥善处置后继续原 running Attempt；会话重开不改状态，不为它调用 fail 或 begin。

10. **取得明确成果。** 流程结束后查询：

   ```bash
   sheltie work result <work> --json
   ```

   只有 final=true 时才给终点选择的 artifacts；路径、摘要、大小与 source 指向该具体终点绑定或封存的槽。终点绑定输入可以由其他步骤生产；没有选择明确为空，不猜最近文件。Work succeeded 和文件引用不代替内容质量、代码候选或检查过程的核验。结果查询列举冻结引用，消费原件时仍按实际读取合同核字节；接受、复制、合并、发布各按已有授权办理。

## 最小代码方法

`code-change` 固定 implement → review → deliver，review 可以沿 back 返工。方法作者准备步骤与稳定规则，任务使用者提供 task（目标和验收）与 project（位置、范围、检查前提）；阶段内由 agent 自主调查和拆分，不为每个代码子任务增加节点。默认没有 gate，实际授权边界由作者在新版本声明。

每次使用自始至终保持相同管理根；专用 --home 的值不能在重开后回落默认。开发候选与已发布版本按各自格式使用独立管理根。图、输入与封存报告保持原版本，目标变化时另开 Work。自然审查的内容返工用 submit 与显式边，执行失败才用 fail。

## 细节

每条命令的参数、任务书格式、错误码见 [协议合同](../../specs/contracts/protocol.md)；Workbook 怎么写见 [Workbook 合同](../../specs/contracts/workbook.md)。
