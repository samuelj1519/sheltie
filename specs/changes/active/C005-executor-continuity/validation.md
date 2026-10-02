# C005 验证

Candidate: `none`

状态：`active`。以下是预定 oracle；实现、真实使用和实施审阅均为 `not_run`。

## 1. 机制验收

| 要求 / 风险 | 合法例 | 拒绝例或故障例 | 归属 |
| --- | --- | --- | --- |
| 一次原子替换 | 当前 latest running，旧 superseded、新 running | 非当前 / 非 running，业务状态无变化 | T02 |
| 固定一次额度 | 第一替换成功 | 第二次拒绝，当前仍可 submit/fail，无新增 blocked | T02 |
| 冻结输入 | 非 stats 引用逐项相同，可选 None 保持 None | 输入缺失、摘要或大小改变，整次拒绝 | T01 原语 / T02 真实链 |
| 统计输入与任务书 | stats 含新 Attempt，恢复逐字节相同 | 用最新状态重算旧 stats 被拒绝 | T01 原语 / T02 真实链 |
| 编号与失败独立 | replace 后 number=1、failed=0；max_retries=1 第一次 fail 仍 active | number 不能耗尽重试；max_retries=0 第一次真 fail 必须阻塞 | T01 原语 / T02 真实链 |
| 历史 fail caller | replace→fail→begin→fail 后重放首次 fail 返回原 active | 伪造历史状态、失败身份或前缀，StoreCorrupt | T01 原语 / T02 真实链 |
| 旧资格撤销 | 新 Attempt 正常提交 | 非终态旧 submit/fail 为 ATTEMPT_NOT_RUNNING；新尝试已使 Work 终态后旧 fail 为 WORK_TERMINAL | T02 |
| 并发 | 普通替换提交一个新 Attempt | submit/replace、两个 replace 竞争至多一个成功 | T02 |
| 请求意图 | 同 ID 同参数返回原响应，不读已删除理由文件 | 改理由或目标 REQUEST_CONFLICT | T01 原语 / T02 真实链 |
| 历史 begin / replace | 新尝试后来成功，原请求仍可重放 | 错身份、路径、requires 或输入绑定拒绝 | T01 原语 / T02 真实链 |
| 效果与卡 | 后续状态卡保持最新 revision | COMMIT 后发布失败恢复同一 brief，不再创建 Attempt | T01 原语 / T02 真实链 |
| COMMIT 前 | 正常事务完整提交 | 提交前 kill，无旧终止、新尝试或成功请求 | T01 原语 / T02 真实链 |
| 状态与版本 | 新格式严格读写，理由仅属 superseded | 未知字段、非法状态组合、第二 superseded、旧库均拒绝，旧字节保留 | T01 原语 / T02 真实链 |
| 身份与时间 | 真实 OS 边界、非默认时间和主体 | 改 USER 不改变审计，CLI 不接收注入时间/身份 | T01 原语 / T02 真实链 |
| 普通续接与门槛 | 重开继续原 Attempt，替换后标准仍在 | 不能强制替换、因等待阻塞或绕过 gate | T02 / T03 |

拒绝断言同时核业务状态、revision、audit、request 和历史文件，不只看错误码。全部限额含恰好上限与超一拒绝；顺序号含检查溢出。故障恢复必须核原 brief/stats 字节或独立摘要。

## 2. 真实使用

沿用 C007 或真实使用中已识别的撤销任务，不另做大样本比较。记录为什么普通续接不足、旧进程是否停止或工作区如何隔离、新 Attempt 如何使用冻结输入、旧正式提交如何被拒绝，以及完成原目标的总投入。

| 指标 | 口径 |
| --- | --- |
| 需求成立 | 哪个具体场景必须撤销旧资格；上下文缺失不计替换需求 |
| 约束保持 | 输入、说明、成果标准和 gate 逐项核对，关键丢失即失败 |
| 接续投入 | 用户操作、重新解释、必要重新验证与重做分别记录 |
| 保证范围 | 旧进程、共享目录和外部副作用属于人工或宿主事实，不宣称引擎隔离 |

单次使用只证明对应场景可用。真实样本缺失写 `not_run`；不把 fixture 故障、普通恢复成功或审核通过当产品收益。

## 3. 证据表

采用后执行前固定候选、工具链、格式、Workbook 摘要、fixtures、features、环境与过滤器。复用需要相同输入闭包和原 run ID；executed/reused/covered 是模式，不能充当 PASS。

| Requirement / risk | Mode | Input closure | Command / raw run ID | Result | Evidence |
| --- | --- | --- | --- | --- | --- |
| 状态、计数及历史 caller | not_run | 待采用固定 | 未执行 | not_run | 无 |
| 事务、重放与故障恢复 | not_run | 待 T02 固定 | 未执行 | not_run | 无 |
| CLI、普通续接与门槛 | not_run | 待 T02 固定 | 未执行 | not_run | 无 |
| 真实撤销与接续 | not_run | 待真实任务 | 未执行 | not_run | 无 |
| 最终工程门禁 | not_run | 待固定候选 | 未执行 | not_run | 无 |
| T01 原语 green 与 T02 行为 red | not_run | 待 T01 阶段闭包 | 未执行 | not_run | 无 |
| C005-M1 实现准备 | not_run | 待骨架完整候选 | 未执行 | not_run | 无 |
| C005-M2 完整审阅 | not_run | 待最终候选 | 未执行 | not_run | 无 |

超时、中断、缺输出和未跑平台逐项列明，不写 PASS。方案文档静态检查与产品验收分别记录。

## 阶段验证与测试基准

T01 复杂作者创建 `verification/commands.sh`、所有阶段 tests/fixtures 与 `experiments/runbook.md`，实际名称、过滤器、测试归属、counts 和预算在交接时固定。原语测试归 C005-T01、实际 green；新完整行为测试归 C005-T02、初始 ignore，以 task.sh 显式运行。future-red 至少一项真实 CLI 或持久消费者的行为断言失败；编译/环境错误、零测试或仅占位 panic 不算有效 red。有效 red 是准备证据，不是产品 PASS。已有正确 caller 保持 green，无需全红。

M1 核原语、普通回归、有效 red、骨架可编译且没有未完成正常入口。T02 开工基准是 M1 审定骨架或最新独立测试修订的完整 SHA，allow_test_changes=false；只能删除归本任务 ignore。测试/fixture/合同缺口交复杂作者新增明确修复任务，独立复核后固定新基准，不隐式继承旧批准。正式 feature 必须非零实际执行、全部通过且无本任务 ignore。

M2 核完整用户链与实际使用。对 M1 已审且闭包未变化的内容引用原 closure 和 run ID，不重复全套；新增调用/配置/fixture/效果或异常分支按影响补验。每项任务仍有短语义复核，原输出只保存一次。T03 是手册和实际使用，不以无 Rust 用例的 task.sh 验收。

## C005-T00 采用入口证据

开工Candidate `23932afc1577a0b20a6b1cf7ad23ab8ac6186571`。本轮仅C004归档、C005采用、状态/索引/引用文字，无code/fixture输入变更。docs139、spec8change1active、tests741/205cards及diff通过（exit0）。独立Reviewer核采用授权、真实需求/宿主处置不造事实、已有C004 scope PASS与缺项仍分开，并逐字节核132原文移动无漂移。初次target/Owner字段替换漏匹配的结构检查失败保留为作者诊断，修后通过，不算产品验证。
