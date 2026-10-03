# 当前验收与用户交付卡

这份卡给首次接手 Sheltie 的协调者和开发者，用于判断已有能力、证据边界与下一步入口。事实冻结在源码快照 `00e010ba8466b6ffc78ba603f5c923174b3d12af`；不包含此后实施或实验。本文索引已有验收，不新增产品标准，也不代替用户接受、发布批准或 active change 的计划。

## 先确认使用哪条版本线

| 对象 | 冻结时点的事实 | 使用入口 |
| --- | --- | --- |
| 当前源码候选 | `0.3.0-rc.1`，开发目标 `v0.3.0`；Store schema `4`、`cli-result/v4`、`work-result/v1`。尚未发布 | [仓库 README](../../README.md)、[当前规格入口](../README.md)、[CLI 合同](../contracts/protocol.md)、[Store 合同](../contracts/storage.md) |
| 已发布产物 | `v0.2.0`，发布目标仅 `aarch64-apple-darwin`（macOS ARM）；README 的远端安装取得这条已发布线 | [v0.2.0 release record](../releases/v0.2.0/README.md) |
| 当前外围工具 | `sheltie-export` 提供可编辑成果副本，未发布、未安装；只经公开 CLI 读成果 | [导出指南](result-export.md) |

使用当前开发能力时，从当前源码取得可信二进制，并使用新的显式管理根。schema 1/2/3 旧库保留且被 schema 4 拒绝，不自动迁移或清空；旧记录要用对应旧二进制和旧管理根读取。二进制 rollback 不等于 Store 降级。构建与导出命令按[导出指南](result-export.md)执行，不能由 `v0.2.0` 安装完成推断开发功能可用。

## C001–C006 实际闭合到哪里

| Change | 已实际闭合的范围 | 仍需保留的边界与来源 |
| --- | --- | --- |
| C001 | 建立稳定规格、change package、release record 的文档治理与检查入口 | 文档治理不改变产品行为，见[C001](../changes/completed/C001-specs-governance/README.md) |
| C002 | 已发布 `v0.2.0` 的历史独立保留；当前恢复完成可靠性、准确 215 项处置和 macOS aarch64/APFS 可构造输入的限定验收。T58 修复审计事实资格后，完整 Spec 与工程重新独审通过 | 当前候选不是新发行；物理非法字节名称遍历、旧运行限制及原失败留原，见[C002 验证](../changes/completed/C002-v0.2.0-reliability/validation.md#c002-t58-m2审计事实资格修复)、[修后 Spec 审查](../changes/completed/C002-v0.2.0-reliability/evidence/completion-20261003/m2-repaired-spec-review.md)与[工程审查](../changes/completed/C002-v0.2.0-reliability/evidence/completion-20261003/m2-repaired-engineering-review.md) |
| C003 | 将 v0.1.0 MVP 的 release record、plan、决定、runbook 与任务表统一归档 | 只调整历史路径和入口，不改产品代码、MVP 结论，见[C003](../changes/completed/C003-archive-v0.1.0/README.md) |
| C004 | 当前自动 agent 的有用文档交付、同 Attempt 冷接续、明确终点三引用及新真实消费者的逐查询观察和使用已验收 | 原真人试用、宿主关闭重开、配对净收益与费用未补成 PASS；原 B 两次查询缺独立前后快照的偏差永久留原，见下节与[C004 M3](../changes/completed/C004-verifiable-delegation/validation.md#c004-m3-当前采用范围收尾) |
| C005 | 原子替换机制、冻结输入、历史重放、额度与普通续接规则已实现；当前技术与真实需求负前检独审完成，新增实际 Rust 1.85 测试 | 七项真实撤销前提仍为 `null`，真实撤销使用 `not_run`；不能制造替换需求，见[C005 M3](../changes/completed/C005-executor-continuity/validation.md#c005-m3-当前技术完整限定验收) |
| C006 | 真实 agent 核对并编辑导出副本；同质量 CLI/script 与工具副本对照、再次导出保留旧编辑、不同 `st_dev` 的自有虚拟 APFS 卷全体复制与卸载已验收 | 只证明该自动 agent 场景与载体，不证明真人净收益、外置物理盘、其他 OS 或断电持久，见[C006 T08/M3](../changes/completed/C006-result-delivery/validation.md#c006-t08-当前实际副本与跨apfs) |

冻结时点 C007 只有技术准备与缺项交接，原正式真实实验未执行；C008 只有条件前检与缺项交接，probe 未采用、真实价值 `not_run`。两者即使在 `completed/` 中，也不能被读成正式试用或用户价值通过。当前无 active change；后续采用和任务开始仍由人决定，见[当前状态入口](../README.md#当前状态)。

## 技术证据按各自口径读取

下面的运行、旧目标处置与文件输入集合不是同一个计数域，不相加成新的总数，也不把同源复用写成再次执行。

| 证据域 | 实际结果与候选关系 | 原件入口 |
| --- | --- | --- |
| 当前原生完整工程 | Rust 1.98.1 / macOS arm64，nextest `0.9.145`；run `424c53bc-4a8a-4bf1-8c26-f4b600335a83` 为 `951/951` PASS、零 skip、零 LEAK；另有 5 个 compile-fail doctest，fmt/check/Clippy 全部退出 0。T58 新 source、实际 Cargo 产物与独立冻结二进制绑定 | [T58 最终工程原件](../changes/completed/C002-v0.2.0-reliability/evidence/completion-20261003/t58-final-gates.json)、[T58 验证说明](../changes/completed/C002-v0.2.0-reliability/validation.md#c002-t58-m2审计事实资格修复) |
| 当前实际 MSRV 测试 | Rust `1.85.0` / macOS arm64，nextest `0.9.145`；run `8d822614-fca3-453f-b28e-2d8345e810e4` 为 `951/951` PASS、零 skip、零 LEAK，另 5 个 compile-fail doctest 全通过。六步骤退出 0、无 timeout，连续总投入 `160.4732879s`；使用实际 1.85 Cargo 产物，不借旧 1.98 二进制。无 doctest 的两个 crate 的零样本不计额外用例 | [MSRV 执行原件](../changes/completed/C005-executor-continuity/evidence/resume-20261004/msrv-execution.json)、[C005 T06](../changes/completed/C005-executor-continuity/validation.md#c005-t06-当前原生rust185实际测试) |
| 旧 215 项欠项处置 | 恰好 215 个唯一旧 ID：185 项有动态结果、21 项限定静态处分、9 项结构验证。T43 时其中 37 个动态 ID 实际在其最终候选运行，148 个沿分组完整冻结结果并经当前 caller/生产差异复核。原 stage-1 `Missed`、原 `Caught=false` 和 9 个旧 native `not_run` 保留；9 个新结构 target 不冒充旧 target 动态 Caught。T58 两个新 helper 检测另列，不并入 215 | [215 账本](../changes/completed/C002-v0.2.0-reliability/evidence/completion-20261003/t43-215-dispositions.json)、[验收绑定](../changes/completed/C002-v0.2.0-reliability/evidence/completion-20261003/t43-acceptance-binding.json)、[T43 说明](../changes/completed/C002-v0.2.0-reliability/validation.md#c002-t43-全部215精确欠项汇总) |
| 输入闭包 | C002 的 190 个源码/变异输入与 134 个工程输入子集分别记录；后续 C004/C005/C006 的 192 个 tracked 输入域包含自身范围，Git symlink literal 与普通文件字节分开核。数量不代表测试数，也不能把 190 与 192 称同一集合 | [C002 修后资格](../changes/completed/C002-v0.2.0-reliability/validation.md#c002-t58-m2审计事实资格修复)、[C004 当前域说明](../changes/completed/C004-verifiable-delegation/validation.md#c004-t07-实际文档交付与同attempt冷接续)、[C005 当前资格](../changes/completed/C005-executor-continuity/validation.md#c005-m3-当前技术完整限定验收) |
| 依赖与资产计划 | RustSec `ef6173cbc5c50ec8166f9a5b28f07834144373ee` 实际取数后，按不变政策离线检查 advisories/bans/licenses/sources 四类通过；是该取数时点的资格。实际 dist plan 只有六项引擎资产，唯一原生 binary target 为 `aarch64-apple-darwin`，exporter 无发布资产；计划没有构建或发布资产 | [deny 原件](../changes/completed/C002-v0.2.0-reliability/evidence/completion-20261003/m2-deny-execution.json)、[资产核对](../changes/completed/C002-v0.2.0-reliability/evidence/completion-20261003/m2-dist-assets-verification.json) |

当前 C004–C006 按各自相同输入闭包引用原生 951、MSRV 951 和依赖原件，只证明对应技术资格；不是新的测试 run，也不是此刻重新获取 advisory。旧要求 nextest `0.9.145` 的 exit `92`、实际 `0.9.140` override、旧 1.85 仅编译、缓存与 fetch 中断事实不被后来结果倒填。

## 实际交付与未完成使用义务

### C004：接续和成果引用

真实 Work `2026-10-03-001-c005-opening-package` 使用冻结方法 `code-change1.0.0`。A 留下有用草稿后结束；B 是不继承 A 会话的新实例，只从 Home、Work ID、binary 与规程读取 `work status` 的 `resume` 接续指针，在同一 `implement#1.0`、revision `2` 接续，随后独立 content-review 和新 deliver 完成。最终 Work `Succeeded`、revision `7`；`change/review/delivery` 三项引用来自具体终点 `deliver#1.0` 的输入/输出槽，逐字节、size、SHA 已独立核对，不取目录最新文件。见[实际交付](../changes/completed/C004-verifiable-delegation/validation.md#c004-t07-实际文档交付与同attempt冷接续)与[三引用原件](../changes/completed/C004-verifiable-delegation/evidence/resume-20261004/final-three-reference-check.json)。

下一 C005 协调者真实读取成果并执行首项只读准备；另一个新消费者实际 `status/result`、读取三成果并作准备判断。新消费者每条查询五表全部行与 47 个业务对象前后相同，SQLite 控制载体例外明确。新观察补当前能力，不补造旧 B 查询缺失的历史快照。见[新消费者](../changes/completed/C004-verifiable-delegation/validation.md#c004-t10-新消费者实际逐查询与使用)与[准备判断](../changes/completed/C004-verifiable-delegation/evidence/resume-20261004/consumer-report.md)。代理接受仅该文档包；真人接受、原宿主关闭重开和人类配对净收益仍 `not_run`，usage、付费与人工活动未知。

### C005：没有撤销事件就继续原 Attempt

两个 C004 消费者只有普通接续，不是撤销旧资格的真实需求。七项前提仍为 `null`：`work_id`、`old_attempt`、`real_revocation_event`、`old_executor_stop_or_host_isolation_evidence`、`new_executor`、`original_goal_and_acceptance`、`quality_and_total_activity_cost`。见[原前提记录](../changes/completed/C005-executor-continuity/evidence/t03/preflight.json)及[当前负前检](../changes/completed/C005-executor-continuity/validation.md#c005-m3-当前技术完整限定验收)。

上下文缺失时按普通接续读取当前 brief/input/next。只有实际需要撤销资格、前提齐全时，才按[C005 runbook](../changes/completed/C005-executor-continuity/experiments/runbook.md)处理；新执行者开始前由操作者确认旧进程已停止或宿主环境已隔离。引擎替换不停止旧进程、不证明宿主隔离。真实撤销、真人接受、完整成本与净收益 `not_run`；原四条不同 run/case 的 LEAK 因果 `unknown`。

### C006：副本可以编辑，源结果仍冻结

本轮源为 C004 的三份真实成果。实际 agent 核对 CLI/script 与工具副本达到同一质量，再读写 tool 副本以准备后续资料；原源、manual 副本和原 manifest 不改。再次导出生成新目录，保留旧编辑，新副本仍是原成果字节。编辑后的文件不再由初始清单保证，不能把副本变成第二结果来源。见[C006 实际使用](../changes/completed/C006-result-delivery/validation.md#c006-t08-当前实际副本与跨apfs)。

自有 ASIF/APFS 镜像挂载后，源 `st_dev=16777234`、目标 `st_dev=16777243`，三项成果、manifest、权限与源业务状态全部核对，再只卸载本次 owned 载体。它是实际跨设备的虚拟 APFS 验收，不是外置物理盘或其他 OS。所有 7 条外层 engine/export 查询均有五表/47 对象前后相同的观察；外层查询数不混同内部调用推导。原 842 run 的 4 条 LEAK 因果仍 `unknown`，新 951 无 LEAK 不注销旧信号。

成本只有自动 agent/script 单 pair：原 CLI/script `0.315966s` 与工具 `0.116367s` 均含 observer 快照，不能推人类净收益或一般节省比例。真人手工、接受、人工费用未执行或未知，usage/付费为未知。`complete` 只表示合同规定的核验、整目录不替换发布和 OS 同步完成；不保证物理断电持久或恶意同权限进程隔离。见[C006 M3 边界](../changes/completed/C006-result-delivery/validation.md#c006-m3-当前采用完整限定验收)与[导出状态及失败现场](result-export.md#失败现场)。

## 首次接手的最少下一步

1. 先读[当前 specs 入口](../README.md)：冻结时点无 active change，`completed` 只表示其中限定范围归档，不能自行启动新方案、授用户接受或发布。
2. 要运行已发布线，按[README 快速开始](../../README.md#快速开始)并核对 [v0.2.0 release record](../releases/v0.2.0/README.md)。要使用开发能力，核当前源码候选与新管理根，按[导出指南的构建与输入](result-export.md#构建与输入)取得实际 Cargo `executable` 和 SHA；任一步非零停止。
3. 要交接或继续现有 Work，读[C004 验证与真实消费者](../changes/completed/C004-verifiable-delegation/validation.md#c004-t10-新消费者实际逐查询与使用)，用可信当前二进制查询 `work status` 与 `work result`，读取 status 中的 `resume` 接续指针，据当前 `next` 和绑定输入继续。不要由摘要、草稿路径或目录存在推断已封存或可推进。
4. 要取得可编辑文件，按[导出指南](result-export.md)：先核同一次结果 `final=true`、`status.kind=succeeded`、无待完成 effects 且选集非空，明确授权目标父目录；只把 `complete` 且退出 `0` 的新目录当规定完成。失败或 `publication_unconfirmed` 保留现场按指南核身份、manifest、字节与退出码。
5. 要判断真实价值或恢复 C007/C008，先补真实任务、实际 actor、固定质量与方法、活动/成本原件、独立质量与接受条件。冻结时点正式试用 `not_run`；由人决定采用后，按新的 active package plan 执行，不能用这份卡降低合同或把技术 PASS 升级为所有安全、所有平台、真人价值 PASS。

macOS aarch64/APFS 之外的 Linux/Intel 按用户范围排除，不记 PASS。APFS 不允许构造物理 `0xFF` 名称（`errno92/EILSEQ`），引擎对此物理名称的遍历仍 `not_run/environment_blocked`；纯 bytes 名称测试不替代现场执行。依据见[C002 原生适用性](../changes/completed/C002-v0.2.0-reliability/validation.md#c002-t45-当前macos-aarch64原生适用性)及[新 binary 原生补验](../changes/completed/C002-v0.2.0-reliability/evidence/completion-20261003/m2-repaired-native-probe.json)。产品行为、字段与合法操作最终按[规格](../spec.md)和[合同](../contracts/)判断。
