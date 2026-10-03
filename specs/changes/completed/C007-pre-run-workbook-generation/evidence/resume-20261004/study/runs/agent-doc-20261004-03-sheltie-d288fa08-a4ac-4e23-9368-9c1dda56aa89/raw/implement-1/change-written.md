可审查：完整文档候选已提交，两项冻结检查成功；独立应用、日常审查和最终质量/接受尚未执行。

# 目标与实际变更

为首次接手 Work 的操作者提供 `specs/guides/continuity-choices.md`，区分同 running Attempt 的 status/resume 接续、真实执行 fail、内容 review 正常 submit 后沿 back 返工，以及确需行政资格撤销时 replace。冻结基线没有此指南；接手时已有 5874 字节未跟踪草稿，现完成为 13354 字节指南。

保留原选择表、同 Attempt 接续步骤、真实 failed 计数和显式内容回边说明；核其自然草稿 SHA 与 partial state 完全相同。补齐 replace 命令/前提、每 Occurrence 固定一次额度、冻结输入与 stats、旧 submit/fail 和终态错误优先级、历史 next 刷新；增加结构化 EFFECT_PENDING 中 A/B 请求身份恢复、unknown、human/gate/终态停止说明。未改产品标准、Rust、测试、方法或其他文件。

# 候选与授权

- 独立仓库：`/private/tmp/sheltie-completion-20261003/c007-agent-paired-study/repos/continuity/sheltie`；初始 HEAD `351feb7ac22c21317a686693b732d5ae0c4b4bcc`，初始 tree `80ea3046b1b3575f07e68313444c051ee7c5b7db`。
- 当前 candidate：`89c1b6022ced9cfe698e56a0f1c08bc65168cf46`；tree：`5640cd4993bcac032bc15fcdee17a9e05cf42eb5`。
- 唯一变更：`specs/guides/continuity-choices.md`，SHA-256 `a028106d07209ad3026af594e88004acda12e878f2a710f0402bd379fddcf890`。
- 完整基线→HEAD 路径集合等于唯一 allow_file，工作区干净；原件 `/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C007-pre-run-workbook-generation/evidence/resume-20261004/study/runs/agent-doc-20261004-03-sheltie-d288fa08-a4ac-4e23-9368-9c1dda56aa89/raw/implement-1/07-candidate-closure.{json,stdout,stderr}`。两项检查的 staged tree 与提交 tree 完全相同。
- 仓库历史入口显示无 active change，本次外部 C007 冻结 task/project 明确授权该独立样本，不从 proposed 自选工作。

# 事实依据与使用

先读选择表，按实际触发进入对应章节；将占位符替换为完整身份与本 Work 专用 Home。所有示例命令带显式 Home/JSON，只读不带 request-id，写意图先保存 UUID。本文是操作者选择指南，未执行其中 fail/replace/gate 示例以制造实验事实。

字段与命令以 `specs/contracts/protocol.md` §1/3/5/6/7 为依据；输入、请求身份与效果按 `specs/contracts/storage.md` §2.1/3.2/5.5/6；replace 边界按 C005 spec EX-01–EX-08 和 design §2–4；接续范围链接 C004 README。已读取 assigned repo CONTEXT、README、engineering 和完整实际 brief/task/project，并按共享中文 skill 与受控技术写作参考组织。相关 CLI 参数、调用者及 core next/decision 只读核对，原件在 `/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C007-pre-run-workbook-generation/evidence/resume-20261004/study/runs/agent-doc-20261004-03-sheltie-d288fa08-a4ac-4e23-9368-9c1dda56aa89/raw/cold-implement-1` 的 `01`–`07` 记录。协议宽读显示有截断，关键字段均在实际显示的 §3/5/6/7 中核对，不声称对未显示段落做完整内容审阅。

# 同 Attempt 冷接续与返工

实际只读 `work status` 在 2026-10-03T20:34:07.063716+00:00 开始，20:34:07.075161 完成，exit 0；仍为 running `implement#1.0`，resume 中 brief、inputs、draft_outputs 逐项与 stage 一致。输入 SHA/bytes 于候选闭包复核一致；previous-review 为 null，未收到自然审查返工。草稿位置经实际读取确认，不从路径推断存在或完整。

原 helper 正常 turn 结束事实来自 stage：`2026-10-03T20:32:57.454521+00:00`，其自启命令/turn 范围报告无后台，未声称全宿主停止。当前 helper 仅继续原 stage；new begin/start/fail/replace/submit 为 0，无直接 Store 写，不重计 method use。冷查找、核对、解释和保留/修改/重做依据在 `/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C007-pre-run-workbook-generation/evidence/resume-20261004/study/runs/agent-doc-20261004-03-sheltie-d288fa08-a4ac-4e23-9368-9c1dda56aa89/raw/cold-implement-1/cold-resume-activity.json`；继承草稿原件 `/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C007-pre-run-workbook-generation/evidence/resume-20261004/study/runs/agent-doc-20261004-03-sheltie-d288fa08-a4ac-4e23-9368-9c1dda56aa89/raw/cold-implement-1/inherited-draft.md`。

# 冻结完整 patch 与独立应用规则

完整 patch 原件：`/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C007-pre-run-workbook-generation/evidence/resume-20261004/study/runs/agent-doc-20261004-03-sheltie-d288fa08-a4ac-4e23-9368-9c1dda56aa89/raw/implement-1/06-full-patch.stdout`，13748 字节，SHA-256 `a10bef129228ddc38cb77f9fbbda4e4e21ad1d62b84fc5032e3b3171ef41d1ac`，小于 8 MiB。实际生成 argv：

```text
git diff --binary --full-index 351feb7ac22c21317a686693b732d5ae0c4b4bcc HEAD -- specs/guides/continuity-choices.md
```

新文件已 stage/commit；不是仅生成 unstaged diff。deliver 应再次核对 candidate 与完整路径集合，冻结 project 声明的授权独立应用副本为 `/private/tmp/sheltie-completion-20261003/c007-agent-paired-study/patch-check/continuity/sheltie`，从原 archive/full initial snapshot 建立，基线必须为上述初始 HEAD/tree；不得修改 source 或另一 arm。按冻结顺序执行：

```text
git apply --check <完整patch绝对路径>
git apply --index <完整patch绝对路径>
git write-tree
```

这些命令在授权检查副本的根执行；最终 tree 必须等于 `5640cd4993bcac032bc15fcdee17a9e05cf42eb5`，所有授权文件字节与 candidate 相同。本 helper 的 ownership 不含检查副本，独立 apply 为 `not_run`，留给 deliver 执行；未把可应用规则当作已验证事实。

# 限制与未完成

本候选只新增指南。真实撤销进程、宿主隔离、身份认证、Rust 回归和文档中的业务示例演练 `not_run`；检查成功不代替内容独审。日常独立 review、deliver 的独立 patch 应用、最终独立质量与代理接受均 `not_run`；安装、发布、push/merge 未执行。实际 usage、费用、API 模型/会话标识未知，冻结声明为 gpt-6.1-sol/high，Root 后续可核实际 metadata。

Actor：`/root/c007_study_coordinator/run03_cold_implement`；写报告 UTC：`2026-10-03T20:37:40.445230+00:00`。引擎本阶段提交仍由协调者执行。
