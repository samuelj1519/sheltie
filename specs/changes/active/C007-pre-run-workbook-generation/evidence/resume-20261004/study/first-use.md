# 当前agent试用入口

本入口只适用agent-doc-delivery-20261004。先T04独立准入并提交，再读freeze/actor-history/samples task/project/protocol、原完整method。不要把原experiments pending模板换成真人事实。

## 实际参数与命令入口

`STUDY` 为本文件目录的绝对路径。binary/methoddir/study_root从其`freeze.json`读，`HOME=<study_root>/formal-sheltie-home`；只有B arm使用，不能复用技术probe Home。任务参数分别为`STUDY/samples/<sample_id>/task.md`与`project.md`，project中的repo_by_arm和patch_rule.verify_repo_by_arm给真实独立路径。Root在开始run之前生成唯一run_id、run_dir、start/deadline、arm/order/localprior的run-binding.json；它是实验参数，不是引擎状态。sample与arm只能按freeze.order，未知/缺字段先停。

所有CLI用下表的参数数组，`BINARY/HOME/TASK/PROJECT/METHOD`都由上述实际JSON取得；ID、brief与输出只能取实际响应。数组展示不经shell。先独占写本动作`intent.json`保存UUID和argv，再调用capture，不把事后生成UUID叫先记录。固定args不得由报告正文添加参数。

| 操作 | 实际argv（前缀P） | 响应取值/下一动作 |
| --- | --- | --- |
| P | `[BINARY, "--home", HOME, "--json"]` | 每次相同真实Home |
| 首次登记 | `P + ["--request-id", UUID, "workbook", "add", METHOD]` | data.id/version/digest；0且ok，保存method身份，仅一次 |
| show/verify | `P + ["workbook", "show", "code-task-study@1.0.0"]`；verify同结构 | start_inputs task/project、每版本status=ok；readonly无UUID |
| start | `P + ["--request-id", UUID, "work", "start", "--workbook", "code-task-study@1.0.0", "--flow", "default", "--name", SAMPLE, "--input", "task=@"+TASK, "--input", "project=@"+PROJECT]` | data.work_id/work_dir/workbook.digest，存完整reply及next |
| begin | `P + ["--request-id", UUID, "attempt", "begin", WORK_ID, "--node", NODE]` | NODE只从reply.next；data.attempt/brief_path/inputs/outputs，全部读完整brief后委worker |
| submit | `P + ["--request-id", UUID, "attempt", "submit", WORK_ID, "--attempt", ATTEMPT_ID, "--summary", SUMMARY]` | 先核actualoutputs exist/bytes；summary真实<=4096，存revision/next |
| fail | `P + ["--request-id", UUID, "attempt", "fail", WORK_ID, "--attempt", ATTEMPT_ID, "--reason", REASON]` | 只实际执行失败，原件和投入留；不能代内容back |
| status/resume | `P + ["work", "status", WORK_ID]` | data.resume.attempt/brief_path/inputs/draft_outputs、revision/pending及next；冷续原running，不begin/replace |
| result | `P + ["work", "result", WORK_ID]` | data.final/succeeded/noeffects，keys恰change/checks/review/delivery/patch和source/bytes/SHA；实际读文件核同 |

调用例（具体label每次新且只含字母/数字/hyphen/underscore）：

```python
import json, sys, uuid
from pathlib import Path
sys.path.insert(0, str(STUDY))
from capture import execute
request = str(uuid.uuid4())
argv = [BINARY, "--home", HOME, "--json", "--request-id", request,
        "attempt", "begin", WORK_ID, "--node", NODE]
with (RUN_DIR / "begin-intent.json").open("x") as stream:
    json.dump({"request_id": request, "argv": argv}, stream)
record, stdout, stderr = execute(RUN_DIR / "cli", "begin-01", argv,
                                 REPO, 60, RUN_DEADLINE)
reply = json.loads(stdout)
if record["exit_code"] != 0 or not reply["ok"]:
    raise RuntimeError("Stop: original command record retained")
ATTEMPT_ID = reply["data"]["attempt"]
BRIEF = reply["data"]["brief_path"]
OUTPUTS = reply["data"]["outputs"]
```

上例变量全来自实际binding或此前response，不复制占位ID去跑。capture只收原件，不判next或内容；调用者必须核实际exit与ok，失败即按protocol停，不换label续同失败。合法单条件拒绝只在已结束的preparation，不算正式run。Native每检查也用同capture/120s和唯一记录，阶段报告位置由run binding实际paths给出。

每run在实际开始后建立run.json，字段必须至少包含：run_id/sample_id/arm/actor_id/assistant_ids、prior_uses（两arm实际计数）/actor_arm_use_index/exposure、protocol_id/order_index/input_refs、started_at/ended_at/wall_seconds、activity（自动agentwallclock事件单列，不冒humanminutes）/interruptions/rework、usage原字段或null、delivery_refs/acceptance_ref/quality_ref、outcome/stop_reason。以下是结构示例，pending/null不是成功值：

```json
{"run_id":"由Root实际开始前分配","sample_id":"source-start","arm":"native",
 "actor_id":"/root/c007_study_coordinator","assistant_ids":[],
 "prior_uses":{"native":0,"sheltie":0},"actor_arm_use_index":1,"exposure":"first_use_local",
 "protocol_id":"agent-doc-delivery-20261004","order_index":1,"input_refs":{},
 "started_at":null,"ended_at":null,"wall_seconds":null,
 "activity":[],"agent_activity":[],"interruptions":[],"rework":[],"usage":null,
 "delivery_refs":{},"acceptance_ref":null,"quality_ref":null,
 "outcome":"not_run","stop_reason":"not_started"}
```

实际run不能原样提交上例。结束时outcome按真实completed/failed/stopped，quality/acceptance未发生仍null；原始命令/报告不覆盖，run索引可追加新事实但不改已发生历史。完整原字段定义见原design/record-template，当前新protocol将global模型/真人历史及人类metrics保持unknown。

Root为每个run给实际run binding：sample_id/arm/order/actorprior、repo/patchcheck/run_dir、common task/project paths、methoddir、binary/Home和输出路径。两arm先核Git HEAD/tree和允许files、methodSHA、source/candidate/环境，不符停；不得在Root源仓库或另一arm修改。每动作保存新raw argv/cwd/env/UTC/stdout/stderr/actualexit与对应candidate，不覆盖首次失败。

Native：认真以本run的stage.json保存stage、task/project SHA、repo/candidate、reports路径、visits/retries、当前partialhelper/停写事实。向implement worker交完整implement instruction和common task/project/明确outputs，只写allowed repo files及change/checks；独立review worker读相同task/project/change/checks与真实repo，仅写review；根据实际第一行需修改走back或建议交付进入deliver，deliver worker核同candidate和完整patch应用、写delivery/patch。协调者不写节点内容、不把报告PASS当末质量。

Sheltie：每command显式binary --home专用Home --json；写前记唯一UUID，readonly不加。首次安装同method一次add/show/verify，之后show/verify不重复add；合法/拒绝准备测试不作为正式run。work start的task/project从同common文件@读取；从实际next begin并保存brief/inputs/outputs/Attempt；派worker只写actual outputs。implement完成submit、beginreview、submitreview后按正文选back/main；deliver实际submit后status/result核final/succeeded/noeffects/五refs与具体终点source、实际bytes/SHA。不能直接改Store、猜路径/历史next或批准不存在gate。本方法不用export或replace。

continuity两arm：先partialimplement worker正常结束，无后台/no submit；保存actual session task_complete、自启动commands与草稿。新fork-none worker仅本armbinding/task/project/method/持久阶段；Sheltie自行status/resume原running，不begin/fail/replace；Native自行stage/inputs/currentcandidate核定位。读取自然持久资料并完成原阶段，记录查找/解释/重做/助手变更；两arm都不传前worker会话正文。仅新agent上下文，不说关闭主App/真人会话。

每run交付五reports/完整patch/candidate与raw。patch验证只能该run预授权检查repo，初始HEAD须等base，实际git apply --check/--index后write-tree等candidate。把doc内容、任务、全patch和必要raw交Root代理内容接受；之后统一中性末审另一动作，quality待真实独审不预写。按design模板写run.json，每方法run真实prior/序号/停止、助手/冷续/费用null清楚。

提交正文保持中文规范，新guide未来RootT07应用前只存在sample副本；不用额外Rust/fulltests/磁盘镜像/安装/发行。工具不确定、超时/输入漂移/超方法边界停并保原件，不更改协议或以新的名偷偷重跑。日常自然返工依原method，真实执行失败才fail；未知不填0。每run结束coordinator final简报actual artifacts/sha/time/助手/局限，不透露另一arm产物给worker。
