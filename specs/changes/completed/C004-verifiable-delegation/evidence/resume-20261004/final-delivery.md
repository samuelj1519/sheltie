C005 文档开工包及独立内容审查已整理；本 delivery 仍待协调者提交，最终成果封存、下一消费者实际使用、C005 真实撤销、真人接受与完整成本尚未在本交付说明中验收。

# C005 开工包交付说明

实际消费者是继续 C001–C008 请求的下一协调者。本次交付文档开工包，实际仓库代码未在本任务中修改，不宣称 codefix。当前 C004 active，C005 仍在 completed 保存历史限定验收；下一消费者先只读原件、定位首动作并整理本轮准备判断，C004 最终归档以后才另任务恢复 C005。

## 1. 取得本次三份成果

本 Work 为 `2026-10-03-001-c005-opening-package`。以下 `WorkRoot/` 均指绝对目录 `/private/tmp/sheltie-completion-20261003/c004-real-work-home/works/2026-10-03-001-c005-opening-package/`。本 delivery 的任务身份为 `deliver#1.0`，输入来自本次 `review#1` 的 main 边。

| 成果 | 本次具体位置 | 用法与已核绑定 |
| --- | --- | --- |
| change | `WorkRoot/attempts/implement/occurrence-001/attempt-000/outputs/change.md` | C005 开工包正文。27476 B，SHA-256 `e465b6cd1f45cd30fc2b88e798a58d0e0be9c62eff6e517c03b1eb725be93332`，与本 deliver brief 的冻结引用一致。 |
| review | `WorkRoot/attempts/review/occurrence-001/attempt-000/outputs/review.md` | 独立内容审查原文。9259 B，SHA-256 `3edd3209f279dc5cd72806dd8501718676846bbd1bbee02c0157063a26b22b93`，与本 deliver brief 的冻结引用一致。 |
| delivery | `WorkRoot/attempts/deliver/occurrence-001/attempt-000/outputs/delivery.md` | 本说明，当前是 running Attempt 的声明输出；不能提前称为已封存或可交付最终集合。最终 bytes/SHA 与具体终点引用由提交后的结果 oracle 核对。 |

绑定 task 为 `WorkRoot/start-inputs/task`，1670 B，SHA-256 `66feae2dda4b66a94aaad56ed9f34e67ff90903fb66f0a7d67174dbf503a760e`。已冻结 project 为 `WorkRoot/start-inputs/project`，1578 B，SHA-256 `8720c0940953935968d8b6a5c3925690f70718112282fcd9f6d1327684cb2e7b`。本执行者逐项重算上述输入，均与本次已绑定引用或事前冻结记录相符。

change 与 review 指向同一份文档候选：review §1 明确核对上述 change 路径、27476 B 与 SHA-256；review §2 按冻结 task 的全部质量条件查源。交付者未将另一 Attempt、较新文件或仓库最新草稿替换为本次输入。

冻结 Flow 的 deliver Node 明确选择本次绑定的 `change`、`review` 输入和本次 `delivery` 输出为 result。协调者提交后应通过当前冻结 CLI 的只读 `work result` 核对 `final/succeeded`、三条引用及其 source、bytes、SHA；查询不推进状态。流程成功只说明执行和输出合同成立，报告内容是否可接受由使用者核对。

## 2. 文档候选、原件与检查原文

仓库原件根为 `/Users/shushu/orca/workspaces/sheltie/codex`。以下 `C005/` 指该根下 `specs/changes/completed/C005-executor-continuity/`。本次没有新代码 commit 或 patch；实际文档候选就是 §1 的 change。相关源码候选与旧检查引用仍是报告内容，产物封存不证明外部 Git、工具过程或宿主事实。

| 对象 | 实际位置或身份 | 范围 |
| --- | --- | --- |
| 当前冻结源码候选 | `ccf7a0cc4d4c4a3a1b3c040473ab2cbc069a1982` | 本次 C004 事前固定候选，非本任务新实现。 |
| 冻结 binary | `/private/tmp/sheltie-completion-20261003/t58-final-frozen-binaries/sheltie`，SHA-256 `7c961834888b085c1bd54a0302b83354c11eed480ee0941e5866b665f273e88c` | 交付者只读重算摘要相符，未调用状态写命令。 |
| C005 历史限定验收候选 | `d8d8c20fc71e3bd8bb813afecc6a4aa1af4a63f4` | 原验收范围见 `C005/validation.md` §C005-M2 与 `C005/review.md`；旧 PASS 不自动适用于当前源码或真实使用。 |
| 本次冻结来源 | `specs/changes/active/C004-verifiable-delegation/evidence/resume-20261004/freeze.json` | method_files 6、source_files 18、task_project_protocol_sha256 3、完整 C005 原件 62 项分别重算，均 `drift=[]`；清单有重叠，不累计为独立文件数。 |
| 本次执行边界 | 同目录 `protocol.md`；冻结 task/project | 只交文档；当前消费者只读；需要新来源或发现源漂移即停止本 run，先独立重审与重新 freeze。 |

正文查阅顺序：change §2 了解六项原义务，§3 找原件与首动作，§4/4.1 找责任、前提、真实命令和停止条件，§5 找正反判据，§6 找后续机制/环境入口，§7 区分实际检查与未执行。review §2 是逐质量条件审阅，§3 是原日志复核，§4 是结论范围。

原检查和缺项须查原字节，不能只看摘要：

| 义务或信号 | 保留结论 | 精确原件入口 |
| --- | --- | --- |
| 真实撤销、接手、原约束、完整投入及用户接受 | `not_run / authorized_defer` | `C005/validation.md` §C005-T03、§原义务（行143–154）；`C005/plan.md` T03（行116–126、160–162）；`C005/evidence/t03/preflight.json` 的 actual_trial 七字段均 null。 |
| 原 nextest 0.9.145 门禁 | `not_run`，原命令 exit 92；历史实际 0.9.140 override 单列 | `C005/evidence/t01/required-task.txt` 行1–2；`C005/validation.md` 行83、148。 |
| 在线 fresh advisory | `not_run`，固定缓存结果有界 | `C005/evidence/t01/deny-cached.txt`、`C005/evidence/t02/gates-revised.txt`；`C005/validation.md` 行91、139、149。 |
| 其他平台与 Rust 1.85 测试 | `not_run`；1.85 仅本机 macOS arm64 编译 | `C005/evidence/m2/msrv-check.txt`；`C005/validation.md` 行141、150、160。 |
| 四条原 run/case 的 LEAK | `unknown`，原因未定位 | `C005/evidence/t01/runtime-consumers.txt` 行5、17；`t01/gates-repaired.txt` 行13、18；`t01/future-red.txt` 行5、67（行269重复）；`t03/skill-consumers-revised.txt` 行4、6。具体 run ID/case 见 change §2、review §3。 |
| 两次全 cached whitespace | `raw_diagnostic_exception`，full exit 2 与 authoring exit 0 分列 | `C005/evidence/t04/whitespace-check.json`、`C005/evidence/t02/whitespace-check.json`；`C005/validation.md` 行107、119、152。 |

早期 validation §3 和 review 的阶段 `not_run` 与后续 M1/M2 限定证据分列。不得将阶段表改写，或把作者范围 PASS 称为全 cached PASS、后续干净运行称为旧 LEAK 已解释、1.85 编译称为测试、macOS 结果称为跨平台完成。旧真人配对、真人宿主重开和人类净收益义务也未因本次自动 agent 文档审阅完成。

当前开发格式为 `0.3.0-rc.1`、Store schema `4`、`cli-result/v4`，保持 `work-result/v1`、`workbook-digest/v2`；已发布 `v0.2.0` 另按 release 原件核对。没有 push、merge、release、部署或宿主安装授权。

## 3. 下一消费者的首动作与使用方法

下一协调者现在先只读以下原件。这就是开工包的第一项合法准备动作，不启动新 Work，不调用 replace，不提前 activate 或写 C005。

```bash
sed -n '143,160p' /Users/shushu/orca/workspaces/sheltie/codex/specs/changes/completed/C005-executor-continuity/validation.md
cat /Users/shushu/orca/workspaces/sheltie/codex/specs/changes/completed/C005-executor-continuity/evidence/t03/preflight.json
```

读取后由下一协调者整理自己的本轮准备判断：实际能提供哪些真实 Work、旧 Attempt、撤销事件、旧新执行者、宿主处置、原目标/验收及成本证据；当前需要是否只是普通上下文恢复。能逐项指向原件才称已取得；缺项保持 null/not_run。仅有演示、夹具或 C004 同 Attempt 冷接续时，不能认定真实撤销需求成立。

随后按 change §3 的来源索引读 adoption、README、plan、runbook、现行 protocol/storage，再按 §4 P1–P8 准备。当前 C004 尚 active，合法准备止于只读判断。C004 最终归档以后，由下一任务固定 C005 本轮采用范围、责任人、独立 Reviewer、真实输入与验证闭包，再按唯一 active 的规则恢复；本说明不代替采用或激活操作。

普通 resume 使用同一 running Attempt。`attempt replace` 用于确需撤销旧正式提交资格，原子形成旧 superseded 与同 Occurrence 新 running，每 Occurrence 最多一次；number 是顺序号，superseded 不消耗业务失败额度。引擎不停止旧进程、不认证接手者、不隔离宿主。新执行者开始前须由操作者确认旧执行者停止，或有隔离的新宿主工作区；停止/隔离证据不足时不派新执行者。

未来真实操作必须使用 change §4.1 的实际变量模板：固定绝对 Home、完整 WorkId、候选 binary/hash、当前 Attempt、唯一写请求 UUID、原质量标准；先 status 并核 next，再决定普通 resume 或有理由的 replace。发生 EFFECT_PENDING 时按 committed/pending_request_id 处理，并以同一 request-id、同一参数恢复原已提交请求。继续前再查 status，不能用历史响应 next 猜当前权限。原输入、说明、标准、gate 或 requires 丢失/漂移时停止，先交作者和独立 Reviewer 判断。

未来机制检查入口位于 `C005/verification/commands.sh` 的 primitives、feature、regression、gates；治理及平台/工具检查按恢复后的本轮 plan 固定范围。以上均是后续入口，本 delivery 没有执行 C005 试用、Rust 测试、nextest、cargo deny 或全仓库门禁，也不以无关门禁替代本外部交付文档的内容核对。

## 4. 审查结论与接受前核对

绑定 review 第一行原文为「通过」。其结论只覆盖冻结 task 的 C005 文档开工包；没有必须修改项或阻断项，不要求人为制造返工。Reviewer `/root/c004_content_review` 未参与 change 编写，核对原件、命令及六项缺项。这是自动 agent 独立内容审阅，不是原真人盲审、真人接受、宿主认证或 C005 真实撤销验收。

接受前由协调者/使用者核对：

1. 本次三份成果来自本次具体终点 Attempt，绑定路径、字节、大小和 SHA 与 result 引用一致；非最终时 artifacts 应为空。协调者负责提交并检查 oracle，本执行者不提前断言该检查已通过。
2. 当前 consumer 的首动作仍为 §3 的只读原件检查；记录消费者实际如何定位入口、准备判断及仍缺输入。实际使用尚未在本 delivery 中观察，不能由图 done 或报告自称推断价值。
3. 后续恢复 C005 前核最新唯一 active、当前候选和 dirty 内容；本次 frozen 来源若漂移或需要新来源，停止并先独立重审/重新 freeze，不能结果后换材料。
4. 真实撤销对象、停止/隔离和原目标/标准有事实支撑；质量、接受、完整投入各有独立原件。缺实际对象时不制造事故或替换样本。
5. `usage=null`、`paid_cost=null`、`human_activity=null`、`user_acceptance=null`。墙钟不折算人工活动或费用，不将等待排除后称总成本，也不将未知付费写为 0。

本交付者的实际检查为：读取冻结 task、project、change、review、冻结 Flow、规程及 C005 最终义务原件；用 Python hashlib.sha256 对 §1 输入和 freeze 四组来源逐项重算，无漂移；核冻结 binary 摘要相符。对应 stdout/stderr/exit 见本执行者同步工具调用原记录，命令 exit 0；未据此认证外部仓库过程或全宿主状态。

本执行者只写本任务书声明的 `outputs/delivery.md`，未改 brief、绑定/封存输入、仓库、C005 或管理元数据，未调用 begin/submit/fail/replace 或其他状态写命令。所有自己启动的命令均同步结束，未创建后台或长驻进程；此停止说明只覆盖本执行者自己启动的命令，不是全宿主进程树关闭结论。完成回复后停止写入，由 Root 按当前 next 提交。
