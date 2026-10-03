# 首次使用操作入口

当前仅完成技术准备，正式实验未准入。三项真实任务、参与者与使用历史、宿主/环境/权限、顺序、关闭重开、预算和逐任务质量标准均 pending；六个正式运行 `not_run`，不创建虚构 run_id。方法作者的技术自测不能充当首次读者、用户接受或成本结果。补齐下列入口并经独立正式准入审查后，才执行本手册的正式运行部分。

## 1. 先核准入与固定材料

| 核对对象 | 文件或来源 | 开始前必须确认 |
| --- | --- | --- |
| 本轮范围与完整准入 | [protocol](protocol.md)、[validation](../validation.md) | 所有 pending 已由实际任务提供者补齐；正式预算、停止、顺序和独立审阅已冻结 |
| 方法原件 | [confirmed-input/README](confirmed-input/README.md)、[shared-method](shared-method.md) | 完整 manifest、Flow、三个 instruction；两组同一文字，不只读 show 摘要 |
| 真实样本参数 | [samples/README](samples/README.md) | 补齐后建立 `samples/<actual-id>/task.md`、`project.md`；目前这些参数文件不存在，不执行 start |
| 质量与接受 | [oracle/quality](oracle/quality.md) | 逐任务正反标准、必需检查、完整 patch 方法、重大缺陷及实际接受人；不是通用模板即已确认 |
| 使用者与历史 | protocol 与实际宿主原件 | actor × arm 的真实 prior_uses、所有助手和首次读者；作者身份不能写成未经接触 |
| binary 与环境 | [preparation/README](preparation/README.md)，其实际 CLI 记录 `preparation/cli-mechanism.json` | 从 Cargo JSON 或可信发布取得的绝对 binary 路径/字节标识；实际 `--version`、`self version`、源码闭包及环境匹配 |
| 仓库与记录路径 | 真实 project、[record-template](preparation/record-template.md) | 两个独立初始副本、允许目录、唯一运行记录目录、检查原件位置与冻结的 patch 应用副本 |

当前采用源码版本为 `0.3.0-rc.1`、`cli-result/v4`、schema 4；核实际输出，不能自行改版本。binary 不从 PATH 或猜测的 target 路径取。把证据中的实际绝对路径记为 `SHELTIE_BINARY`，固定 confirmed-input 的绝对路径记为 `METHOD_DIR`。每次 Sheltie 调用都显式传当前配对实验专用的绝对 `EXPERIMENT_HOME`，不使用正式 `~/.sheltie`，不运行 self install。

正式准入后，为每次运行分配唯一 `RUN_DIR`；真实 task/project 绝对路径记为 `TASK_FILE`、`PROJECT_FILE`。这些变量是已冻结实际值的别名，未填时不能运行下面的命令。写操作前分配实际 UUID request-id，保存其值与完整 argv；同一操作的安全重试保留该 ID，不给只读操作传 request-id。

## 2. 保存每条命令原件

每条 CLI 单独记录真实 argv、工作目录、开始/结束时间、stdout、stderr、退出码；文件名用本运行内不重复的操作序号。例如第一条 add 使用如下捕获方式，`ADD_REQUEST_ID` 必须已记录，`RUN_DIR/cli` 必须是已授权的新记录目录：

```sh
"$SHELTIE_BINARY" --json --home "$EXPERIMENT_HOME" --request-id "$ADD_REQUEST_ID" workbook add "$METHOD_DIR" > "$RUN_DIR/cli/01-add.stdout" 2> "$RUN_DIR/cli/01-add.stderr"
cli_exit=$?
printf '%s\n' "$cli_exit" > "$RUN_DIR/cli/01-add.exit"
```

其他命令以同样方式保存自己的新文件，不能覆盖失败、重试或旧原件。记录模板字段由真实原件填写；当前未知 usage、费用和活动分钟保持 null，不把墙钟或 Work stats 当人工时间。实际运行时同时保存所有参与者与助手的活动、phase 和唯一 cost_bucket；失败、停止和流程外接受/盲审的投入也记录。

成功预期是退出 0 且 JSON `ok=true`；stdout 为一行响应，维护诊断可以在 stderr。业务拒绝退出 1，参数或输入参数文件错误退出 2；任一非预期结果按 §7 停止，不继续下一阶段。这里的命令成功只证明该操作完成，不能证明代码质量。

## 3. 原生组执行

1. 在协议指定的独立新会话开始。核对真实 actor、两组 prior_uses、模型、推理档位、工具权限、已有 skill 和环境，以及本组独立初始副本；按真实历史计算 exposure。先完成并记录合理的原生配置，禁止故意削弱该组。
2. 把 [shared-method](shared-method.md) 和同一三个 instruction 文件完整提供给宿主。每阶段执行者读对应文件原文、同字 task/project 与上游报告；按 project 的路径保存 change/checks/review/delivery/patch 和检查原件。实现者与日常审查者独立，全部助手实际动作入记录。
3. 用协议冻结的已有宿主持久方式记录阶段、候选、输入/输出位置、3/3/1 访问与每到达 1 次失败重试。没有真实持久方式时停止交准备作者，不能让运行者临时设计另一套流程。原生组不调用 Sheltie。
4. 协调者读 review：建议交付且必需检查齐全才进入 deliver；需修改则按既定回环进入新的 implement/review；停止或次数/预算耗尽时停止。失败重试、自然返工和 injected 演练分别记录。
5. 按 §5 实际关闭重开，按 §6 整理交付和流程外接受/盲审。原生组保留同等自然材料，不能由 Sheltie 作者代办困难动作后声称首次使用顺畅。

## 4. Sheltie 组执行

全部命令都替换为 §1 冻结的实际参数，使用 §2 的独立原件捕获。技术装入可以在正式任务前单独验证，但这些原件属于 preparation，不计正式 run。

| 顺序 | 实际 CLI 形式 | 预期 stdout / 退出码 | 读取或保存位置；失败动作 |
| --- | --- | --- | --- |
| 核 binary | `"$SHELTIE_BINARY" --version`；`"$SHELTIE_BINARY" --json --home "$EXPERIMENT_HOME" self version` | 实际版本文本；版本与 schema JSON，退出 0 | 保存各自原件与 binary 标识；版本/路径不符停 |
| 装入一次 | `"$SHELTIE_BINARY" --json --home "$EXPERIMENT_HOME" --request-id "$ADD_REQUEST_ID" workbook add "$METHOD_DIR"` | ok=true，data.id/version/digest/flows/requires，退出 0 | 保存 `code-task-study@1.0.0`、真实 digest；已存在或拒绝停，不能升版本/换内容绕过 |
| 核摘要结构 | `"$SHELTIE_BINARY" --json --home "$EXPERIMENT_HOME" workbook show code-task-study@1.0.0` | ok=true，Flow、节点/边、start_inputs，退出 0 | 起始输入为 task/project；与完整材料核对；不符停 |
| 核已装原件 | `"$SHELTIE_BINARY" --json --home "$EXPERIMENT_HOME" workbook verify code-task-study@1.0.0` | ok=true、该版本 status=ok，退出 0 | 保原件；tampered/missing 停，不手修已装副本 |
| 创建本次 Work | `"$SHELTIE_BINARY" --json --home "$EXPERIMENT_HOME" --request-id "$START_REQUEST_ID" work start --workbook code-task-study@1.0.0 --flow default --input "task=@$TASK_FILE" --input "project=@$PROJECT_FILE"` | ok=true，data.work_id/workbook/flow/work_dir、next，退出 0 | 把实际完整 work_id 记为 WORK_ID；保存被冻结输入位置/身份与 next，不编造示例 ID |
| 领取当前阶段 | `"$SHELTIE_BINARY" --json --home "$EXPERIMENT_HOME" --request-id "$BEGIN_REQUEST_ID" attempt begin "$WORK_ID" --node "$NODE"` | ok=true，data.attempt/brief_path/inputs/outputs、next，退出 0 | NODE 只能是实时 next 的可选节点；保存真实 ATTEMPT_ID 与输出路径，完整任务书交给执行者；拒绝停 |
| 提交真实完成 | `"$SHELTIE_BINARY" --json --home "$EXPERIMENT_HOME" --request-id "$SUBMIT_REQUEST_ID" attempt submit "$WORK_ID" --attempt "$ATTEMPT_ID" --summary "$SUMMARY"` | ok=true，封存结果与 next，退出 0 | SUMMARY 是真实简短结论，至多 4096 字节；先核所有必需输出已写到响应目标；非预期结果停 |
| 标真实执行失败 | `"$SHELTIE_BINARY" --json --home "$EXPERIMENT_HOME" --request-id "$FAIL_REQUEST_ID" attempt fail "$WORK_ID" --attempt "$ATTEMPT_ID" --reason "$REASON"` | ok=true，失败事实与 next，退出 0 | REASON 是真实原因，至多 4096 字节；保失败成本；不能用 fail 代审查返工 |
| 读当前事实 | `"$SHELTIE_BINARY" --json --home "$EXPERIMENT_HOME" work status "$WORK_ID"` | ok=true，data.resume/revision/effects_pending/pending_publish 及 next，退出 0 | 保存完整响应；恢复与选边只用当前事实，不能把重放的历史 next 当实时事实 |
| 核终点成果 | `"$SHELTIE_BINARY" --json --home "$EXPERIMENT_HOME" work result "$WORK_ID"` | ok=true，明确选择的 Refs 与就绪事实，退出 0 | 按 §6 核五个 key、候选和真实文件；拒绝/未就绪停，不扫描目录替代 |

装入只需在同一 Home 登记一次。既有已核登记在后续 run 使用 show/verify，不能每次 add 然后把 WORKBOOK_EXISTS 当成功。每个新写动作使用新 UUID，原动作重试使用原 UUID；保留每次响应的实际 request_id/revision。

节点执行只写 begin 返回的 `outputs` 路径，完整读取 `brief_path` 和已绑定 `inputs`。首次 implement 的 previous-review 是 null；返工必须读其当前绑定来源。输入、Workbook 与封存输出不修改。执行者可修改 project 授权仓库，检查原件写入预定记录位置，报告给出具体引用。引擎封存报告不核实 Git 候选或检查执行。

提交 implement 后由协调者从实时 next 选 review；review 正常完成先提交报告，再根据“需修改”选 implement 的 back，或根据“建议交付”选 deliver 的 main。每次 begin 都取新响应中的实际 Attempt 身份，不能手拼 occurrence/number；达到上限不能绕开继续。默认没有 gate；若未来真实样本有不可替代授权边界，必须先修改并冻结两组等价协议和方法，再独立准入，不在本轮临时执行 gate approve。

## 5. 真正关闭重开与接续

只在正式协议冻结的配对任务、同一阶段执行关闭重开。保存旧会话及执行者停止的真实证据，再关闭旧会话；新会话是实际新开的宿主会话，不能把同一对话发送“继续”计作重开。新会话获得本组自然拥有的持久材料和同等权限，找回/核对/重做及所有帮助实际计时。

原生组读协议冻结的宿主持久阶段记录、相同方法、task/project、候选和报告后续接。Sheltie 组先运行 §4 的只读 status，保存新原件并核：

1. data.work_id、workbook/flow、revision 与实际独立仓库和上次记录可对应，effects_pending/pending_publish 不阻断所需材料；不能用磁盘状态卡替代实时结果。
2. data.resume 给真实 attempt、brief_path、完整 inputs 引用和 draft_outputs。草稿路径不证明文件存在或完整；逐份读取绑定输入/任务书，并核当前候选与草稿。可选输入 null 就记未绑定，不猜历史文件。
3. 若 next 含对该 resume.attempt 的 submit/fail，接续原 running Attempt，在 draft_outputs 指定路径完成工作；不再 begin、不调用 replace、不重复建立本次使用。否则按当前 next 和状态处理：已完成就进入合法下一阶段，失败才走既定重试，终态不重新执行。
4. 草稿缺失、输入被改、候选不符、旧执行者仍活动、效果未就绪或恢复信息不完整时停止。记录实际查找/重复工作及原因，交准备作者；不自动重建状态或把卡点抹去。

一次 run 内的阶段重试、返工或关闭重开不增加方法使用次数。未发生真实重开就保留该义务 not_run，不补写成功叙述。

## 6. 交付、用户接受与独立质量

两组按同一 deliver instruction 完成最终候选、完整 patch、说明与原始检查。patch 生成和独立应用命令来自正式 project；必须覆盖授权未跟踪文件，不能只交一份漏文件的 diff。交付执行者在独立授权检查副本保存真实应用与候选核对原件，不在另一组仓库试应用。方法/副本/权限缺失或 patch 无法完整表达就停止，不临时决定。

Sheltie result 必须 `final=true`、status.kind 为 succeeded 且 `effects_pending=false`，然后核 artifacts 恰含 change/checks/review/delivery/patch 五个唯一 key，保存每项 source/path/sha256/bytes 以及实际文件。前三项是终点绑定输入，后两项是终点输出；其他历史材料不能冒充成果。原生组交付同一五类材料及代码候选与检查原件。本实验不自动调用 sheltie-export；若后续要观察导出，先另冻协议变量，不并入原配对。

把双方最终 candidate、完整 patch、任务标准、检查原件和说明交实际用户接受，并单独保存其结论/理由/活动。再按 oracle 交未参与方法、标准或成果编写的独立复杂模型审阅者；使用中性编号和冻结盲审规则，组别映射仅分析者持有。盲审者核实际代码/patch、按原标准必要复核，不只读报告。无法完全遮蔽时披露。两项结论和投入分别保存，任何缺项保持 not_run，不从 Work 终态、gate 或零退出推断。

## 7. 停止与交回

| 条件 | 运行者动作 |
| --- | --- |
| 缺真实 task/project、角色/历史、权限、标准或预算 | 不开正式 run，不分配假 ID；保 pending，交任务提供者补齐并重新正式准入 |
| 协议/方法/参数/binary/模型/环境漂移 | 停止并保原件；交准备作者声明修复或新协议，旧记录不并入新配对 |
| 非预期 CLI 退出、参数错误、结构拒绝、输入/封存文件不符 | 保存原命令、stdout/stderr/exit 和当前事实；停止，不改 Store 或放宽限额 |
| EFFECT_PENDING | 按真实 committed/request_id/original 或 pending_request_id 区分本请求与旧请求；停止正常推进，交准备作者核原因及恢复。需要重试时复用被核准的原请求 ID/原参数；原响应不是实时 next，恢复后重新 status |
| 检查非零、缺必要原件、review 阻断 | 如实保存失败；按固定方法真实返工或停止，不改验收、删检查、重跑抹掉首次失败 |
| 预算/访问/重试上限，或完整 patch 无法生成/核应用 | 立即停止，保已投入成本与未完成项；不加次数、不截断交付 |
| 缺实际关闭重开、用户接受或独立质量 | 对应义务 not_run；不把技术自测填成真实运行 |

运行者只追加 runs/ 原件与记录，不改方法、protocol、oracle、脚本或标准，不修产品代码。把真实 run ID（若已准入并开始）、停止原因、当前材料位置和全部缺项交 T01 作者；修复取得新冻结闭包并独立复核后才能继续。当前本手册仍是技术准备交接，不能授权真实六次实验。
