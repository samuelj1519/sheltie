# C005 开工包实际消费与准备判断

消费者 `/root/c004_actual_consumer` 在新上下文实际读取 C004 已交付三份成果，通过观察器分别查询 status/result 各一次，执行交付指定的首个只读动作。本次取得足够的入口与限制，能够独立整理 C005 的准备判断；当前停在只读准备。真实撤销、真人接受和费用没有取得事实，本报告不作这些结论。

记录 UTC：`2026-10-03T18:15:45.974388+00:00`。范围仅 macOS aarch64 的自动 agent 文档消费；本消费者不是引擎的新 Node，没有执行状态写操作。

## 实际查询与只读观察

实际同步命令如下，均只执行一次，没有绕过观察器直接调用 CLI：

```bash
python3 specs/changes/active/C004-verifiable-delegation/evidence/resume-20261004/consumer-observer.py status
python3 specs/changes/active/C004-verifiable-delegation/evidence/resume-20261004/consumer-observer.py result
```

| 查询 | 实际 CLI exit | 实际 observer exit | 查询 UTC 起止 | 原件记录 |
| --- | --- | --- | --- | --- |
| status | 0 | 0 | `2026-10-03T18:14:07.482632+00:00` → `2026-10-03T18:14:07.493675+00:00` | `consumer-status-observation.json` |
| result | 0 | 0 | `2026-10-03T18:14:10.758293+00:00` → `2026-10-03T18:14:10.769680+00:00` | `consumer-result-observation.json` |

两份 observation 透传原 stdout/stderr，消费者按原 JSON 解析。每次都有独立 before/after：`workbooks=1`、`works=1`、`work_sequence=1`、`requests=8`、`audit=8`，47 个业务对象的身份/权限/链接数/原字节 SHA 保持。两次查询各自 `unchanged=true`、`within_deadline=true`；消费者另比对四份完整快照，全部相等。SQLite 数据库/WAL/SHM/锁载体字节不在观察对象内，五表行和业务对象已包括。本结论不证明宿主全进程树已关闭。

status 实际返回 Work `2026-10-03-001-c005-opening-package`、revision `7`、`succeeded`、current `deliver#1`、last_attempt `deliver#1.0/succeeded`、`next=[]`、pending `[]`、effects_pending/pending_publish 均 false。result 实际返回 `work-result/v1`、`final=true`、同一 Work/revision/Flow、`succeeded`、effects_pending=false、`next=[]`。这只确认本次成功终点及成果合同；内容能否用于准备由下面实际阅读和动作判定。

## 三份成果的实际读取与绑定核对

消费者由 result 的三条 ArtifactRef 读取原文件，以 `Path.read_bytes()` 独立计算实际 bytes 和 `hashlib.sha256`。三条大小/摘要全部相符，不用文档自报值生成期望。每条 source 指向具体成功终点 `deliver#1.0`：change/review 是该终点的冻结输入，delivery 是该终点的输出；生产文件路径仍分别属于 implement/review/deliver 的实际 Attempt。

| key | 实际 bytes | 实际 SHA-256 | result source | 实际读取路径 |
| --- | --- | --- | --- | --- |
| `change` | 27476 | `e465b6cd1f45cd30fc2b88e798a58d0e0be9c62eff6e517c03b1eb725be93332` | `input` / `change` / `deliver#1.0` | `/private/tmp/sheltie-completion-20261003/c004-real-work-home/works/2026-10-03-001-c005-opening-package/attempts/implement/occurrence-001/attempt-000/outputs/change.md` |
| `delivery` | 11870 | `7825e55ef8924d6af1513c08ade61c786eade0c52f71d113c09cf787720e26e2` | `output` / `delivery` / `deliver#1.0` | `/private/tmp/sheltie-completion-20261003/c004-real-work-home/works/2026-10-03-001-c005-opening-package/attempts/deliver/occurrence-001/attempt-000/outputs/delivery.md` |
| `review` | 9259 | `3edd3209f279dc5cd72806dd8501718676846bbd1bbee02c0157063a26b22b93` | `input` / `review` / `deliver#1.0` | `/private/tmp/sheltie-completion-20261003/c004-real-work-home/works/2026-10-03-001-c005-opening-package/attempts/review/occurrence-001/attempt-000/outputs/review.md` |

实际阅读用途：change §2–3 让我找到六项历史缺项和首动作原件；§4/4.1 提供恢复后的责任、真实变量、原约束比对和停止条件；§5–7区分机制判据、后续入口与 B 自述。review 第一行为「通过」，其文档范围、逐条件查源和历史缺项边界明确；我没有据此把 C005 真实试用判为通过。delivery §3 给出现在可执行的只读动作，§4要求消费者实际定位与准备判断。本消费确实使用这些入口，不只是列出 refs 或重述图状态。

交付原文仍保留写入时的「待提交/实际使用尚未观察」语境。消费者以本次原查询确认最终封存并另留此报告，没有回写原文或改写历史。change 的 B 会话与查询陈述不是本消费者取得的 B 历史快照；旧 B 的两次逐查询 before/after 仍缺失。

观察器执行时已逐项核冻结输入；消费者随后独立重算 `consumer-freeze.json` 的 84 项 SHA-256，`drift=[]`。包括观察器、三份成果、原输入及冻结 C005 原件。没有用新增事实材料替换这组判定来源。

## 首个只读动作和我的 C005 准备判断

从 delivery §3 定位并实际同步执行以下两条命令，均 exit `0`：

```bash
sed -n '143,160p' /Users/shushu/orca/workspaces/sheltie/codex/specs/changes/completed/C005-executor-continuity/validation.md
cat /Users/shushu/orca/workspaces/sheltie/codex/specs/changes/completed/C005-executor-continuity/evidence/t03/preflight.json
```

validation 原件保留六项：真实撤销/接手/原约束/完整投入/用户接受 `not_run / authorized_defer`；原 nextest `0.9.145` 门禁 `not_run`（原 exit `92`、实际 `0.9.140` override 分列）；online fresh advisory `not_run`；其他平台/Rust `1.85` 测试 `not_run`（编译不等于测试）；四条原 LEAK `unknown`；两次 cached whitespace `raw_diagnostic_exception`（作者范围通过不等于全 cached 通过）。M2 的历史限定 PASS 不覆盖这些缺项。

preflight 的 actual_trial 七字段全部为 `null`。我的本轮判断逐项如下：

| 真实试用所需事实 | 本次实际能提供什么 | 准备结论 |
| --- | --- | --- |
| 真实 Work / 旧 Attempt | C004 文档 Work 现已 succeeded，具体终点 `deliver#1.0`；不是待撤销的真实运行对象 | C005 真实 `work_id` / `old_attempt` 未取得，保持 null |
| 确需撤销的理由或事件 | 没有真实旧正式提交资格需要撤销的原件 | `real_revocation_event=null`；普通上下文续接不足以支持 replace |
| 旧执行者停止或新宿主隔离 | 仅本消费者自身同步命令结束声明；没有 C005 旧执行者/宿主处置原件 | `old_executor_stop_or_host_isolation_evidence=null`；不足以派 C005 新执行者 |
| 新执行者身份 | 当前只是文档实际消费者，没有 C005 真实试用执行者 | `new_executor=null` |
| 原目标和验收 | 已取得开工包及冻结原件入口；没有真实 C005 对象的原 brief/目标/质量标准 | `original_goal_and_acceptance=null` |
| 质量、总活动投入与接受 | 本次能完成只读准备判断；没有 C005 真实成果/质量检查/用户接受/完整成本原件 | `quality_and_total_activity_cost=null`；真实试用仍 `not_run` |

当前需求是普通冷上下文的文档接续，开工包足够支持这次准备动作。没有证据成立行政撤销需求，不调用 `attempt replace` 制造样本。当前 C004 尚 active，准备止于本报告；不写 C005、不激活下一 change。

后续由 Root 在 C004 最终归档后另任务明确 C005 本轮采用范围、责任人、独立 Reviewer、当前候选与输入闭包。真实执行前应取得同绝对 Home/完整 WorkId、最新 running Attempt/brief/冻结输入、确需撤销理由及原目标/验收；旧执行者停止或新宿主隔离证据足够后才派新执行者。普通恢复继续同一 Attempt；只有真实资格撤销需要成立且当前 next 允许时，才按开工包 §4.1 固定 request-id 执行 replace。旧新输入、标准、gate/requires 有丢失或漂移即停止，不把 C004 文档 Work、夹具或机制测试充作真实试用。

## 预算、命令与结束边界

本次时点距原 `2026-10-03T17:41:37.324097Z` 起点已连续经过 `2048.650` 秒，硬截止仍为 `2026-10-03T18:41:37.324097Z`，没有新开 60 分钟预算。该墙钟包括等待、阅读、治理和补取证，不折算为人工时间或费用。`usage=null`、`paid_cost=null`、`human_activity=null`、`user_acceptance=null`；没有真人宿主关闭重开、真人接受、paired 净收益或真实 C005 撤销结论。

消费者实际启动的 cat/sed、只读 Python 摘要/快照/JSON 核对及两次观察器命令均同步结束，实际 exit `0`；stdout/stderr 和退出状态保留于本消费者工具原记录。报告写入及最终读回的退出状态以紧接着的同步工具原记录为准，不预先据报告文字认证命令。没有启动后台/长驻进程，没有未完成 session，没有派其他 agent。此声明仅覆盖本消费者自己启动的命令。

消费者仅创建 `consumer-report.md`；观察器分别创建两份 observation。未修改三份成果、brief、Home、C005 或其他仓库文件，未回退他人工作。完成一次最终读回核对后停止写入，由独立 Reviewer 另做增量核验；本报告不代替 C004-M3。
