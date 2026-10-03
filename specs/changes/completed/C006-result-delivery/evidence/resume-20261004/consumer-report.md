# C006 实际资料消费者报告

消费者：`/root/c006_actual_consumer`。观察时间：`2026-10-03T19:04:14.418813+00:00`。本次实际用途是为后续 C007 准备阅读既有文档成果并注释其当前语境。不是原真人手工对照、原真人接受、费用或净收益观察。

## 1. 绑定与实际两组核对

先读取 C006 `evidence/resume-20261004/protocol.md` 和 `consumer-binding.json`，再按后者逐一读取 manual/tool 两组的三份实际文件，独立用 `Path.read_bytes()` 与 `hashlib.sha256` 核 bytes/SHA，用 `lstat` 核普通文件、0600、单链接，逐文件比较两组完整字节。随后完整阅读三个相同文本的内容。没有用 exporter 代码生成期望值。Work 为 `2026-10-03-001-c005-opening-package`，绑定 result 是 `work-result/v1`、revision 7、final=true、succeeded、effects_pending=false；本消费者没有自行调用 CLI 重查 result 或管理状态。

| key | 两组原始 bytes | 两组独立 SHA-256 | 观察 |
| --- | ---: | --- | --- |
| `change` | 27476 | `e465b6cd1f45cd30fc2b88e798a58d0e0be9c62eff6e517c03b1eb725be93332` | 两组实际字节相同；0600、普通单链接 |
| `delivery` | 11870 | `7825e55ef8924d6af1513c08ade61c786eade0c52f71d113c09cf787720e26e2` | 两组实际字节相同；0600、普通单链接 |
| `review` | 9259 | `3edd3209f279dc5cd72806dd8501718676846bbd1bbee02c0157063a26b22b93` | 两组实际字节相同；0600、普通单链接 |

内容判断：两组提供同一份开工包、同一份独立内容审查与同一份交付说明，质量相同。change §2 保留六类历史未执行/未知义务，§3 给只读首动作和来源，§4 P1–P8 给责任、输入、正判据与停止，§4.1 给实际变量命令，§5 给正反判据；review 逐条件审原任务，delivery 给三成果绑定和消费顺序。这些材料能帮助我定位已有约束和缺项。review 的「通过」仅是原 C005 文档任务的自动 agent 内容结论；本次没有重复完成其全部原件审查、C005 真实撤销试用或原检查执行。

原报告含「C004 仍 active」「C005 尚待恢复」以及草稿/未封存时态，是当时写入语境。当前根入口明确 C006 active、C005 completed；C006 `input-C005-final-qualification.json` 给出已归档 C005 commit `a394009f419475cc1718892ba8e889feb95d031a` 的限定技术/负前检范围。本消费者读取该资格记录，没有独立重复其整个提交核验。原报告列出的旧工具/MSRV/fresh 状态也保留其历史语境，不据此推断当前补验未发生或抹掉旧未执行记录。

## 2. 我实际完成的副本编辑

只在 binding 指定第一 tool target `/private/tmp/sheltie-completion-20261003/c006-real-copy-validation/native-parent/2026-10-03-001-c005-opening-package-01a10324-c59b-72c4-afaf-d1000e4bee5c` 的三个 artifact 末尾追加相同注释：

> C005当前已归档a394009；真实撤销七前提null；当前副本用于C007准备。原报告状态按写入时语境理解。

每份完整原始字节都是编辑后文件的完整前缀，断言 `after == original + suffix` 成立。没有改正文、其他历史状态、manual arm 或第二副本。当前独立读 C005 `evidence/t03/preflight.json`：七字段 `work_id`、`old_attempt`、`real_revocation_event`、`old_executor_stop_or_host_isolation_evidence`、`new_executor`、`original_goal_and_acceptance`、`quality_and_total_activity_cost` 均为 null，result=not_run。C005 归档不提供真实撤销事实。

| key | 注释后 bytes | 注释后 SHA-256 | 保留前缀 bytes / SHA-256 |
| --- | ---: | --- | --- |
| `change` | 27654 | `a24ced1e3fd5705456e657cd9929a2ae9eb3b91f24107a81fc6b51cdb4201025` | 27476 / `e465b6cd1f45cd30fc2b88e798a58d0e0be9c62eff6e517c03b1eb725be93332` |
| `delivery` | 12048 | `b4c47f22d02ee61ab7c4ce029a40baedb24d475bc9abaaf7c64c1d1e751f8259` | 11870 / `7825e55ef8924d6af1513c08ade61c786eade0c52f71d113c09cf787720e26e2` |
| `review` | 9437 | `61651a04f55ae2275efd6d44e2ea0c51eb688c6b63de36da21b382070e51874a` | 9259 / `3edd3209f279dc5cd72806dd8501718676846bbd1bbee02c0157063a26b22b93` |

第一 tool target 非 artifact 文件的逐文件 SHA 前后相同，包括 `manifest.json`（`9bde9de34c55263c101b0a16e8f655eb6abef9ae65b77c141227dfc101c8399c`）。manifest 保留发布时原 SHA，注释后完整文件 SHA 与之不同是可编辑副本的实际消费事实，不把注释文件登记成引擎新成果。三个绑定源 artifact 本次读取的 bytes/SHA 与原绑定一致，追加后再读仍相同；这只覆盖源三文件，不宣称本消费者核了完整 Home。manual 三文件前后完整字节相同。

## 3. C007 准备判断和用途

本消费者实际只读下列 C007 plan 和准入入口，没有写 C007、激活 change、建立 Work 或正式 run：

| 实际读取入口 | 读取时 SHA-256 |
| --- | --- |
| `/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/completed/C007-pre-run-workbook-generation/plan.md` | `bf5b967439d0647a2ff2c2f964d6a96f9466dca8935186249d9625846e0c8c52` |
| `/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/completed/C007-pre-run-workbook-generation/experiments/protocol.md` | `0ad83ea40f5603818c57c79e414abd5157eb1697895e0197b770ec571d190fe9` |
| `/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/completed/C007-pre-run-workbook-generation/experiments/first-use.md` | `c54cf1467741f7cb2914e9b0e0d81fe83eb38f85947132be64286f526831aad2` |

C007 plan 仍为 completed 的技术准备与缺项交接；原 M1 正式准入、六个正式 run、M2 真实结果仍 not_run。protocol 的正式准入九项当前为 null：三项近期任务；连续使用者/首次读者；actor × arm 实际历史；模型/宿主/环境/权限；真实关闭重开/自然返工；活动/墙钟/费用/准备回收预算；逐任务质量 oracle/接受；有意义改善阈值；配对顺序/配置/初始副本身份。first-use §1 要求先补齐并冻结正式材料，独立准入后才分配正式 run_id。

我的准备判断：这三份可编辑资料足以帮助整理历史约束、来源与未获事实，但不足以开始 C007 正式试用。副本的具体用途已经发生：我保留原件全文，在实际工作副本标注 C005 已归档、撤销七前提仍 null 与 C007 准备语境，使下一准备者能读历史成果又不把旧阶段状态当当前状态。正式下一步由协调者在 C006 完成后的另任务核真实输入，按 C007 plan/准入入口冻结；本报告不替代该采用/准入决定，不把本次三份 C005 文档成果当作 C007 三个真实代码任务或五类正式终点成果，也不把 C006 manual/tool 对照当 C007 的 native/Sheltie 配对。

本次读取和编辑证明的是自动 agent 实际使用新副本。原真人手工、原真人接受、usage、paid_cost、human_activity 与人类净收益没有新事实，未知仍 null；旧 LEAK 因果 unknown 保留。没有必要内容返工，没有人为制造失败、竞争或撤销样本。

## 4. 写入范围与停止

本消费者唯一写入是 binding 列明第一 tool copy 的三个 artifact 追加与 binding 的本报告路径。未写 manifest、manual、Home、源、C007、其他仓库内容或宿主配置；未运行状态写命令、安装、发布或无关 Rust/全仓库验证。C007 只读准备没有正式 run_id。

自己启动的 `cat`、`rg`、`sed`、只读 Python bytes/SHA/权限核对均为同步命令，已有工具记录均 exit 0；本报告与三项追加也在一个同步 Python 命令内完成并核前缀/manifest/manual/源三文件。本消费者没有启动后台、长驻进程、其他 agent 或未完成 session。最终回复前只读回查本报告和三项追加；之后停止写入。停止声明仅覆盖自己启动的命令，不是全宿主进程树结论。
