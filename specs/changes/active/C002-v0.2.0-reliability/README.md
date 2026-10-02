# C002：v0.2.0 可靠性修复

状态：`active`
目标版本：`v0.2.0`
兼容性：`breaking`（schema 2 拒绝旧库；旧管理根和数据原样保留）
基线：`a664e75a3ab3041d09cd0a1ab4d69336f2dcd055`
Owner：Claude（原实施）、Codex（后续修复与收敛）；采用人：用户（2026-09-27）
权威性：产品语义以根规格、架构及合同为准；实施进度只看 [plan.md](plan.md)

## 范围与阅读入口

修复请求身份、根内文件、Workbook 摘要与冻结、事务发布与恢复、事实视图、协调者交接和发布门禁。保留三个 crate、同步处理和 SQLite 单一事实来源；不增加多用户认证、第二套状态或兼容格式。

| 文件 | 用途 |
| --- | --- |
| [spec.md](spec.md) | 产品变化、兼容性与信任边界 |
| [design.md](design.md) | 已采用的机制与取舍，含 M1 修复要点 |
| [findings.md](findings.md) | O01–O13、N01–N14 与 R01–R24 的问题索引 |
| [plan.md](plan.md) | 任务状态、依赖、责任、验证要求与下一步 |
| [progress.md](progress.md) | 当前跨会话交接 |
| [tasks.toml](tasks.toml) | 文件范围和测试权限 |
| [review.md](review.md) | M1 最终审查结论、范围和限制 |
| [validation.md](validation.md) | 候选、运行、51 行验收索引及历史证据恢复 |

T01–T15、T18–T34 和 M1 已按各自授权范围完成；T16 已按本机构建与实际场景完成，T17 发布候选准备为 `doing`，真实外部发布尚未执行。M1 包含实际 SK01/SK02 跳过：缺最终 Spec 批准及 215 项额外验证，不是完整变异或安全验证通过。

## 成功判据

- 请求绑定真实目标，重放保留提交时的响应和历史文件，当前状态卡不回退。
- 文件操作、Workbook 身份、摘要、发布与恢复遵守上游合同；确定性拒绝没有业务副作用。
- 状态与合法下一步可供跨会话继续；人工条件和 Workbook 交接保留适用版本。
- 固定候选通过必要审查、真实宿主回归及发布门禁；豁免和未执行项不能记 PASS。

## 文档收敛

2026-10-02 用户授权将本目录收敛为以上 9 个常规文件。修复设计、任务要点和最终验收摘要并入对应文件，阶段 review 与原始 evidence 移出工作树；完整原文保存在固定 Git 快照中，见 [历史证据恢复](validation.md#历史证据恢复)。本次不改产品实现、任务状态、已记录结果或发布授权。


## 本轮源码与验证提交

用户于2026-10-02明确授权在确认精简到位后提交。主文档保持9份，另保存按任务压缩的提交证据；档案只用于恢复原文，不恢复成阶段执行入口，也不改写旧M1/SK、Linux、Host或发布状态。

| Task | 原文档案 | SHA256 |
| --- | --- | --- |
| C002-T38 | [`C002-T38.tar.gz`](evidence/submissions/C002-T38.tar.gz) | `44f580d08ca1d6485d68b12d1ea0a7f0c4193cd0d80c6c961299006dfc680686` |
| C002-T39 | [`C002-T39.tar.gz`](evidence/submissions/C002-T39.tar.gz) | `b7e06a8ae7fbb49e0212f70e35b9770f02ba019ff07613baefee0625c1467649` |
| C002-T40 | [`C002-T40.tar.gz`](evidence/submissions/C002-T40.tar.gz) | `e169239ca600e6a6bb2237fdffa76fb0020a3f8a22aac974a46b0fccf940bc79` |

T16 部分回归与人工交接原文见 [`C002-T16-preparation.tar.gz`](evidence/submissions/C002-T16-preparation.tar.gz)，SHA256 为 `f83bd4549b4e99592f4f14885c44db715891c5f913926c48502969dcb9146173`。档案含请求/响应、门禁原文、worker 产物、临时 Store、演练 Git、独立 oracle 和纠正前后复核；构建二进制保留在本机临时目录，不重复收进档案。此档案不是 T16 完成或发布资产。

| T16 阶段 | 原文档案 | SHA256 |
| --- | --- | --- |
| 实际批准后、前两任务与受控 back 复审 | [`C002-T16-continuation.tar.gz`](evidence/submissions/C002-T16-continuation.tar.gz) | `35ab6352c08a3bca0140d5e44ee2509dcc6aab7fe1cc1355ffbada309a5e5565` |
| 四任务、重规划、整体审查、F16 修复与续接点 | [`C002-T16-spec-dev.tar.gz`](evidence/submissions/C002-T16-spec-dev.tar.gz) | `5facbe81ddf214fa85a7db3a2d88647d34516584b73d955d0f973ccca0de67f9` |

后一档案保留实际运行、四行验证、源输入、Git 和原始复核，排除可再生成 cache/bytecode、binary 与执行链链接；不恢复成可直接运行的新 home。Work006 尚待真实重开后推进，Work003 等最终批准；T16 仍 `doing`，不把档案、26/26 或 688/688 扩为完成。

收尾独立核验见 [`C002-T16-spec-dev.audit.json`](evidence/submissions/C002-T16-spec-dev.audit.json)：2597 文件逐字匹配、三个阶段摘要与引用一致，当前白名单、九份常规文档和任务完成门禁 FAIL 均保留。派生 audit 不放回 tar，避免摘要自引用。

真实重开后的续接快照见 [`C002-T16-reopen-pending.tar.gz`](evidence/submissions/C002-T16-reopen-pending.tar.gz)，SHA256 `f7dae68baa8ea8e1c9acc5ad003f8187e84b5c334f2ae7e14bccebec7f2bf72d`。69 个普通文件逐字回读一致，保存用户重开陈述、当前 CLI 查询与推进、Work006 产物和独立阶段审查；Work006 已 succeeded，Work003 仍待最终 gate 的具体批准。此档案不包含二进制或完整管理根，只用于核证据，不作为运行 home；T16 doing、T17 not_run 和正式完成门禁 FAIL 保留。

实际批准后的最终快照见 [`C002-T16-reopen-final.tar.gz`](evidence/submissions/C002-T16-reopen-final.tar.gz)，SHA256 `e9de0e4097eb2b14b6a8f286a6ad0b26e0dd96eaeff892f33f0421abc7a5fa83`。104 个普通文件逐字回读一致，包含用户批准、Work003 gate/终态、六 Work 最终查询、277 项输入复核、最终独立报告及工作区完成门禁原文。T16 已按本机范围 done，T17 仍 not_run；前一档案的待批准状态与 FAIL 是其固定时点事实。最终档案仍只供核证据，不包含完整可运行 home；本轮未提交或发布。

最终封存和记录的独立收尾见 [`C002-T16-reopen-final.audit.json`](evidence/submissions/C002-T16-reopen-final.audit.json)：28 项检查均通过，包含两个新档案逐字恢复、真实批准/CLI、六 Work、277 项输入与门禁、白名单及限制。派生 audit 保存在 tar 外，避免自引用。

T17 本地候选准备原文见 [`C002-T17-preparation.tar.gz`](evidence/submissions/C002-T17-preparation.tar.gz)，SHA256 `29ed28e382a81c1d6f88b841ca4d120607945a4a68052d3b30cdbb0669b892b3`。155 个文件逐字回读一致，包含新版本工程门禁、治理红绿与独立审查、本机 dist 和隔离安装/更新/rollback/schema 反例；排除二进制与可再生成 cache。四平台 CI、真实远端与发布尚未发生，此档案不是 T17 完成或正式发布资产。

首次CI失败与三处Linux权限位宽修复原文见 [`C002-T17-linux-mode.tar.gz`](evidence/submissions/C002-T17-linux-mode.tar.gz)，SHA256 `3c431a61ba73a0bbd1690b65d4d1bccbe7db2dbe328b75f1902034e8758d5829`。37文件逐字回读一致，包含原失败、批准/推送/PR事实、源码patch、独立内容审查及修复后本机700/700与5/5；此时新Linux CI仍待执行，不宣称四平台完成。
