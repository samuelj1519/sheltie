# C002 当前交接

进度只看 [plan.md](plan.md)。本文件只保留下一入口与限制；原逐会话交接从 [历史快照](validation.md#历史证据恢复)恢复。

## 当前位置

M1 已按用户授权的 macOS 离线验收与实际 SK01/SK02 跳过范围完成。产品源码候选 `e3eea899877165f8573befee3774555598ec92bd`；M1 记录提交 `9c933d8da38b2e7b1968de55343bf27e0dc11c25`。699 项通过、零跳过；完整证据核算和限制见 [validation.md](validation.md)、[review.md](review.md)。验证队列已结束或停止，不续跑历史流水线。

SK01 缺最终 Spec 批准；SK02 缺 215 项额外执行，未重试或改记 PASS。Linux 原生运行仍 not_run。MIT许可已由独立T35提交 `7a584a3c1dc0a0589f6839eef23c755cf53320c6`，不属于历史M1输入；后续 rc 必须重新固定实际候选，不能沿用旧输入 hash 声称新候选已验证。

## T38 工作区交接（历史）

产品/全生产代码分析与等价精简已完成，改动留在未提交工作区。10 个 Rust 文件净减少 130 行；699/699 和工程/文档门禁结果绑定本次输入，详见 [T38 验证](validation.md#c002-t38-精简验证)。不复用历史 M1 变异 PASS，也不改写 SK01/SK02、Linux、T16/T17 状态。

T38发现的 [F38-03/F38-02/F38-01](findings.md#6-t38-新发现) 已由T39修复关闭，原始失败与历史状态仍保留。T38的旧输入/测试零改动结论不被T39新增回归改写。

## T39 工作区交接（历史）

三项缺口已修复并通过独立审查：重复Flow在COMMIT前拒绝；完整嵌套载荷拒绝未知字段且历史不修写；成功CLI写后安全维护过期tmp，异常只告警，pending不按年龄清理。最终Nextest 717/717、零跳过，其他工程/文档门禁通过；命令、输入、红绿与范围见 [T39验证](validation.md#c002-t39-修复验证)。

T38与T39改动都未提交。本次未处理真实用户坏Store、未自动迁移/清库、未续跑历史完整变异，也不新增M1/Host/发布PASS。后续rc固定当前实际输入，不沿用旧候选hash。

## T40 工作区交接（提交前记录）

审计相关测试精简已完成：37个Rust测试/support文件净少896行，717→688入口，独有case/oracle/窗口有逐项保留映射；5个compile_fail和7份快照保留。默认生产AST及fixture/config/依赖不变，schema/提交前观察/CLI软链三类定向负控证实新增判定有效。独立审查通过，完整688/688、5/5 doctest与工程/文档门禁通过，详见 [T40验证](validation.md#c002-t40-测试精简验证)。

MVP21条替代只记原位历史注记，checker未放宽。T38/T39/T40均留在未提交工作区；后续如提交须保留各任务实际范围和原始证据，不以新688运行改写旧717/699/M1记录。仍无完整变异、Linux、Host或发布新PASS。

## 下一入口

T16 已提交 `785552be4fddc8fbbbcbffef7bcf69e65588ff8f`；T17 从本文末尾「T17 本版本当前入口」继续。

M1/T16 已按各自范围完成；T17 已准备0.2.0本机候选，本版本两个Mac平台CI与外部发布还需实际证据和授权，当前发布仍为v0.1.0。

## 文档维护

当前 package 仅保留 9 个常规文件。机制只在 design.md 概述并链接上游合同；验证只在 validation.md 固定候选与原文索引；review.md 保存当前审查结论。阶段 review、repair 文档和 evidence 已入固定 Git 归档，不再恢复成活动入口。


## 提交交接

T38/T39分别已提交；T40精简结果确认到位并获提交授权，本源码绑定688/688与5/5compile_fail。三个任务的提交结果以git历史及trailer为准；原文见README三个压缩档案。Linux/Host/完整变异/发布限制仍保留。


## 本轮提交结果

T38已提交 `20aa655717aabad87289e70d4629c4188757470a`，T39已提交 `84bafa37f57ac2fda6aa5c223f6930f4f0882a87`；两者源码分别对应699/717原输入闭包，并在独立target补核699/717零跳过。T40本提交保留最终688/688和5/5compile_fail的原文与源字节；本提交hash由git历史和Task trailer定位。三个任务均已确认本轮范围到位，后续入口仍是T16，不扩为发布授权。

## T16 当前交接

源码候选 `82c6c55159592db5cdb485549e933960863de185`；独立目录 `/private/tmp/sheltie-c002-t16-82c6c55`，管理根为其 `home/`。本机 release 二进制、skill 与输入清单在 `delivery/`、`candidate.json`；仍显示 `0.1.0`。本轮 688/688、5/5 compile-fail 与工程门禁通过，原文及逐命令记录见 [T16 验证](validation.md#c002-t16-当前宿主回归)。未重启历史变异队列。

新会话先手动读取 `delivery/sheltie/SKILL.md`，再用 `delivery/sheltie-bin --home /private/tmp/sheltie-c002-t16-82c6c55/home --json work list` 发现 Work。每个 Work 先查询 `work status`，只按当前 `next` 继续；不要从历史重放的 `next` 推进，不把本文当引擎状态。

| Work | 本轮观察 | 后续动作 |
| --- | --- | --- |
| `2026-10-02-001-t16-two-step` | 两个真实 worker 完成；旧 submit 返回原响应，当前终态不回退 | 新会话只读查询，核对封存产物 |
| `2026-10-02-002-t16-gate` | 用户实际批准后已归档，Work succeeded | 只读核对当前状态及批准事实 |
| `2026-10-02-003-t16-spec-dev` | 四任务、重规划、整体审查、交付、反思均完成；本轮展示产物后取得具体批准，retro gate 成功，Work succeeded，revision 40，next 为空 | 只读核对终态、批准事实和封存产物 |
| `2026-10-02-004-t16-article` | 用户实际复制、核字节相同并提交，Work succeeded | 只读核对人工产物及当前终态 |
| `2026-10-02-005-t16-back` | 用户实际复制第二份修订稿后已提交，Work succeeded，revision 11 | 只读核对 back 和人工产物；不将受控反例当自然质量 |
| `2026-10-02-006-t16-续接` | 用户确认真实关闭并重开；新会话通过 CLI 查询、核原提纲后提交，真实 summary worker 完成并封存；Work succeeded，revision 5，next 为空 | 只读核对续接响应和封存产物 |

已收到实际 gate、两版方案审批、改方案决定及两次文章手动复制完成。四任务及 CLI 条件通过；新的 Workbook/Flow 和输入缺失交互也已由用户实际答复，数据给齐后直接启动，未重复追问。用户本轮确认真实关闭并重开，新会话手动读取交付 skill 后通过 CLI 发现并完成 Work006；Work003 也在展示 delivery/lessons 后获具体批准，最终 gate 成功，T16 必需场景最终独立复核通过，按本机范围收尾为 done。重开事实来自用户陈述，不声称窗口或进程自动认证。已记录的宿主 bundle 元数据为 `com.openai.codex`、`26.928.31416`、build `12553`（应用显示名 ChatGPT）；本宿主窗口访问被工具安全限制拒绝，证据只含运行应用标识与 Info.plist，不证明窗口匹配。usage 仍缺失，手动读取 skill 不证明自动发现。T16 完成不扩大为自动发现、全平台 Host 或发布通过。

F16-01 已修复源码 Workbook 0.2.1；真实 add/show/verify 与新 brief 的 producer fixture 通过，旧冻结 0.2.0 和历史 brief 保留。修复后的完整 688/688、5/5 compile-fail 与工程门禁通过；F16 的合成 producer 产物不算 Host/真人证明。当前真实开发 Work 仍按冻结 0.2.0 运行，不改已登记摘要。批准前正式门禁曾因 T16 未 done 而 FAIL，原文保留；本轮未提交或发布。

Work006 原 outline 的 SHA256 为 `db1dd9ca5730f74dd2329e6afd6fc0ebb509c3e251c2fb48798d90c3a19c1ca1`，983 bytes；重开后独立核字节未变，已按当前 CLI next 提交。summary 为 417 字、五段、1260 bytes，SHA256 `e1c7b576cf0cdec6657e0f6fc0c7f870b11d531681e9bd5fd05147201ab5c752`。`resume-checkpoint.json` 保留未提交的历史续接点，不改成第二套状态。Work003 的交付说明在 `attempts/deliver/occurrence-001/attempt-000/outputs/delivery.md`，反思在 `attempts/retro/occurrence-001/attempt-000/outputs/lessons.md`；本轮实际答复「批准 Work003 最终 retro gate」与来源保存在 `reopened-session/work003-gate-approval.json`。当前终态以批准后 CLI status 为准，不用整个 C002 发布授权替代本地演练批准。

管理根与演练项目先保留用于续接。清理时先归档证据并核对路径，再用这份候选的 `self uninstall --purge`，按其明确确认流程执行；不得操作真实 `~/.sheltie`。本轮未清理或发布。

T16 当前已按本机范围完成，正式工作区 task gate 与 docs/specs/diff-check 均 exit 0；独立报告和原文入口见 validation/README。保留批准前快照与 FAIL 原文，不把后续成功改写成旧阶段 PASS。本轮未暂存或提交；T17 仍 not_run，发布授权尚未取得。

## T17 当前交接（范围调整前）

用户要求继续直到C002全部完成。已准备0.2.0候选、PR四平台非发布构建与同SHA质量证据接线、精确治理表和自包含测试夹具；独立审查及本地700/700、5/5与工程门禁通过。T17目录 `/private/tmp/sheltie-c002-t17-785552b`，逐命令与本机安装/更新/rollback/旧schema保留证据见validation。源码候选提交与最终验收提交分开，当前task完成门禁仍因doing拒绝，不伪造done。

下一步固定候选提交，取得新验证分支推送与draft PR的具体授权，运行实际四平台/同SHA quality；材料齐备后单独取得v0.2.0外部发布批准，再核远端指定版本update/rollback与最终生命周期。旧M1/SK01/SK02、Linux历史、usage缺失保持各自范围，不启历史变异队列、不清理原home。

PR #1 已创建，原candidate7f42e3b的两个Mac资产成功，但Linux构建/quality/MSRV因三处RawMode位宽E0308失败；原文保存不改。三处权限转换已修、本机新700/700与5/5及辅助门禁/独立审查通过；下一步推送新候选到同一非发布验证分支并取完整四平台证据。验证分支已获具体授权，正式发布仍待四平台材料和单独批准。

## T17 本版本当前入口

用户2026-10-03明确可忽略Linux。本版本只发布macOS aarch64/x86_64；当前Cargo dist两个目标，quality/MSRV和产品工程矩阵改为原生Mac，源码摘要用portablePython。上方四平台步骤为范围调整前交接，原33b08bc确实构建四包但Linux质量失败，不能沿用为新输入PASS。

下一步提交当前Mac限定候选、更新已批准验证分支，取同SHA的两个Mac包与完整quality/MSRV原文；实物/manifest/checksum独立核通过后才请求外部发布批准。Linux失败保留并记excluded_by_user，不发布本版Linux包。原SK/usage/Host与源输入边界不改；尚未创建v0.2.0 tag或Release。

## T17 最终单平台入口

用户进一步限定macOS aarch64单平台，其余以后有需求再增加。当前仅一个dist目标与一个原生ARM工程runner；其他平台旧结果保存并excluded_by_user。接下来更新单目标候选至已批准PR，核单平台实物、同SHA完整quality/MSRV与实际安装，再取得正式发布批准；随后远端update/rollback和completed迁移。上方两目标/四目标均为此前输入，不改其SHA或原结果。

本轮「可以仅发布 macOS aarch64」同时明确了该版本单平台发布许可，保存于single-platform-authorization.json。当前仍须先完成实物、同SHA质量与独立核验；条件满足后按此已给许可发布，不重复请求相同授权。此前未授权描述保留为其时点事实。

## 最终交接

C002已按用户许可的macOS aarch64单平台发布v0.2.0；source/tag8455aed2bcd9ac1739852be3987091a0975e7ac3，真实tag质量700/700及MSRV通过，公开资产与skill核验、实际安装/默认远端指定update/rollback及独立发布审查通过。全部任务按原授权范围收尾done；原M1 SK/215/完整安全变异缺失、其他平台排除及usage缺失仍在release record，既有失败档案不改。

当前目录迁为completed，不从proposed自行开新任务。最终文档与task staged gate另存收尾audit后提交；临时演练数据保留，本轮不purge、不改真实home、不合并main。源码tag与后续验收提交分开固定，不能改写已发布tag。

最终独立audit26项与五门禁实际通过，全部41任务按范围done、0active；当前只剩验收记录提交循环，原始已发布source/tag保持8455。提交成功以git Task:C002-T17 trailer及实际返回为准，后续本change不再有实施入口。
