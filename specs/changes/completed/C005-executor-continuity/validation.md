# C005 验证

Candidate: `d8d8c20fc71e3bd8bb813afecc6a4aa1af4a63f4`

状态：`completed`。以下机制表为预定 oracle。T01 已有部分执行记录，正式骨架、M1、公开功能及真实使用尚未验收。

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

| 采用前预定义务 | 采用前状态 | 当时输入 | 当时预定执行 | 当时结果 | 说明 |
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

## T01 部分执行记录（尚未冻结）

[core 普通验证](evidence/t01/core-green.txt) 为 243 项 PASS（含 5 doctest）、2 项 C005-T02 ignore；[core clippy](evidence/t01/core-clippy.txt) exit 0。新增 number 载荷原始 red，以及未来 [next](evidence/t01/next-red.txt)、[状态卡](evidence/t01/status-red.txt) red 均保留，未来 red 不计产品 PASS。

[runtime 原语首轮](evidence/t01/runtime-primitives-first.txt) run `0a590d0c-1b53-45c2-bd77-f5b88054bee3` 为 11/11 PASS；[受影响消费者](evidence/t01/runtime-consumers.txt) run `a1295822-a8e8-488a-b5cc-2debf16ec6e5` 为 18/18 PASS，1 LEAK、111 未在过滤内。LEAK 的 `result_selects_terminal_bound_input_and_sealed_output_in_key_order` 是库用例，未定位原因，不声称零残留。实际 nextest 0.9.140 override；原 0.9.145 门禁仍 not_run。[runtime clippy](evidence/t01/runtime-clippy.txt) exit 0。

[未来提交错误 next](evidence/t01/runtime-next-red.txt) run `395939cb-a66e-42bc-bffc-4b8c78c3bb90` 为 1 FAIL、exit 100，缺预定 replace 项；不是编译或零测试失败。此时尚在补零输入进入来源拒绝例，以上局部原件不冒充最终闭包通过。

## T01 最终准备闭包

初次[完整门禁](evidence/t01/gates.txt) run `8ebd532a-e4ed-4d22-abc7-1cc81c6977a2` 为 763 PASS / 1 FAIL、10 阶段 ignore、exit 100。旧 Store 用例确实拒绝 STORE_CORRUPT，但新增泛化检查覆盖了原 running/current 具体诊断。[原 caller red](evidence/t01/store-current-red.txt) 保留；[最小修复](evidence/t01/current-guard-fix.diff) 只把新增守卫移到已有具体检查之后，不改旧测试。[caller green](evidence/t01/store-current-green.txt) 和两项 validator green 保留。

修后[完整门禁](evidence/t01/gates-repaired.txt) exit 0，run `c1f30428-2691-47cf-befc-3821fc4986e7`：764/764 PASS、2 slow、10 阶段 ignore、1 LEAK（`install_modify_path_flag_is_rejected`，cause unknown）。这条 LEAK 与之前局部 run 的两条分别保留，不称零残留，不调整容忍。fmt/check/clippy 全 targets/features、缓存 deny、docs141/specs8/1active/tests774/240cards/diff 均通过。实际 rustc 1.98.1 / aarch64-apple-darwin、nextest 0.9.140 override；配置要求的 0.9.145 原[task命令](evidence/t01/required-task.txt) exit 92、未执行测试，仍 not_run。

[26 项原语](evidence/t01/primitives.txt) run `d4b29e13-1a6e-4347-aeae-3e10d5eff871` 初轮 26/26 PASS；微修后这 26 项全部在最终 764 回归中实际通过。零输入来源的[实际 red](evidence/t01/zero-input-lineage-red.txt)、[runtime 最终 12 green](evidence/t01/runtime-primitives-final.txt) 保留。

修后[future-red](evidence/t01/future-red-repaired.txt) run `114656b7-e8da-46da-a0c7-2015d4358227`：10 实际失败、exit 100，CLI 未接受 replace、next 及 golden 缺预定 replace；编译成功、非零测试，不是产品通过。初轮 future-red 的 1 FAIL+LEAK unknown 仍在原文。被拒绝的 `.snap.new` [实际输出](evidence/t01/status-rejected-output.txt) 单独保存，只清理本轮已知生成诊断，不采用它作 golden。

Rust/fixture/命令冻结运行的 whole consumer 为 `f43d18f21c87b6c539379af3cb61363b93b165ef1de97d37e722976d6722cf61`（272 files）；domain `sheltie-consumer-input/v1`，按排序路径和内容长度/字节哈希，排除 evidence、progress/review/validation 与非输入 `.snap.new`。之后只改 plan/T02 scope/runbook 的交接说明，源码和全部测试 oracle 不变；[交接闭包](evidence/t01/input-closure-handoff.txt) `54bbe666019f0289f9c2fecf937e4f5ae26edebae39e4c278f1c319f5b88ab4e` 不是相同 whole hash，治理改动另跑检查。原首轮保守 hash 的域不同，不与本域混算。

[cached deny](evidence/t01/deny-cached.txt) exit 0，复用未改 policy 的私有 db-path 配置，锁定 Cargo/全部 features；RustSec 缓存 `117edb3bed98e9be112f277b7615eea3252e7c43` / `2026-10-02T10:58:33+02:00`。在线 fresh 获取仍延期，不能称最新在线审查。

独立审查的 core 14 项、runtime 恢复/历史/严格读取与顺序修复另有实际非零验证；维护性只读审认为无必要改变。正式 M1 仍待完整 T01 commit 与最终交接复核，不把短审记为全部产品完成。

## C005-M1 采用阶段证据

正式独立 Reviewer 审定 `8d00a29e10810c79bb5b44bdc006dc021e26637b`，阶段 PASS。复用同闭包的最终门禁与实际 red/green，未重复全工程运行。辅助原文 [core 11](evidence/m1/independent-core11.txt) run `f581a1f2-9e1c-46c6-8f6f-99b370a61c10`、[runtime 3](evidence/m1/independent-runtime3.txt) run `628ba848-1ff3-47de-afd0-1dc2d29487ac`；history run `7f28370d-f878-4f79-93a4-06cef4188d3c`、order run `2eafb8b2-0230-4f89-a797-cf6b3880c276`、core state run `92a210ee-d12f-483d-b549-9cd9af833447` 的原文已在 T01 evidence。阶段结论与原因见 review，不把 M1 记为最终产品完成。

## C005-T04 独立 oracle 修订

开工 `6925b73727f84871a02d1d8c33edaf2094e151fa`。公开 T02 原完整回归 run `4d3300ab-4da7-41c6-ba6c-19bc86f0b039` 为 774 执行、773 PASS / 1 FAIL、0 skip：旧卡片恢复手写 next 遗漏 replace。原草稿/全部 raw 保存在自有 stash `f3b93bf0e423316f0a6ce655a0e19c8c4917c64b`，不混进本修订。

复杂作者只改旧恢复场景的 Task 归属、T02 ignore 和手写 replace 一行。M1 未公开 next 的 [真实 red](evidence/t04/status-card-red.txt) run `5e8e29e8-f630-4d5a-a3dc-058c3a832d0a` 为 1 FAIL、1 filter 外未执行、exit 100。[独立字节差异](evidence/t04/status-card-byte-diff.txt) 证明只有该行不同，原输入摘要/23B/所有路径/完整 assert 保留。Reviewer 未编写 oracle，按 current running/未用一次额度的 protocol 资格独立确认该最小修正。

feature 现在为 11 项，不能把该 ignore/red 当 PASS。修订提交完整 SHA 是下一 T02 冻结基准；基础生产代码未改。

T04 全 cached `diff --check` exit 2，仅两份不可变原文的工具尾空格（status-card-red.txt:29、status-card-byte-diff.txt:4）；原字节保留。明确排除这两个已知 raw 文件的作者范围 cached 检查 exit 0，详见 [结构化原结果](evidence/t04/whitespace-check.json)。不把 unstaged 或作者范围通过记成全 cached 通过。

## C005-T02 正式接线与冻结验证

最新测试基准 `c9492f80969d59897480f138b780dc8c1d83ec62`。生产只改 next.rs 与 CLI 的 enum/dispatch/wrapper；T01 的状态、决定、strict snapshot、FD、事务和恢复基础未变。[自有草稿恢复证明](evidence/t02/draft-restore.json) 确认八个源码/测试文件及六份原 raw 共 14 项逐字节相同，唯一 plan 冲突保留双方意图与新基准，stash 未删除。

初轮 [10 feature](evidence/t02/feature-before-enable.txt) run `2c9655ad-fec2-4964-a6a0-e842468e4a55` 为 10/10 PASS。首次 [fmt](evidence/t02/gates.txt) 因删除 ignore 留下 fn 缩进 exit 1，恢复原缩进后，测试差异仍只有 ignore 删除。[旧完整回归](evidence/t02/gates-formatted.txt) run `4d3300ab-4da7-41c6-ba6c-19bc86f0b039` 为 773 PASS / 1 FAIL、0 skip，遗漏卡片预期的唯一实际失败交 T04 独立修订，未放宽生产或测试。

[修后 11 feature](evidence/t02/feature-revised.txt) run `94fb155a-080a-44a5-b6ef-1fe3632ab473` 为 11/11 PASS、763 filter 外未执行。删除全部 11 个 ignore 后，[完整门禁](evidence/t02/gates-revised.txt) exit 0，run `4b8d6190-d811-4235-9faf-c5ff24f4b84f` 为 774/774 PASS、2 slow、0 skip、本 run 无 LEAK。旧三条 LEAK unknown 不因这次未报告而撤销。fmt/check/clippy all targets/features、cached deny、docs141/specs8/1active/tests774/241cards/diff通过。实际 nextest 0.9.140 override、rustc 1.98.1/macOS arm64；原要求的 0.9.145 和 online fresh 仍 not_run。

最终 code/test/fixture/命令 consumer hash `d638dc62a9399d0efed1842a74aedf09dd49a6694034a9405f60f22eecfa836a`（272 files，同 consumer 域，排除记录/诊断）。只删除六份测试文件中的 11 ignore，所有断言、helpers、fixtures 与 snapshots 和新基准一致。独立短审通过，[关键 CLI 3 项](evidence/t02/independent-cli.txt) run `30a7e9f0-c04a-4855-a8b9-3819e901c595` 另实际 PASS；维护性短审认为无必要重构。该功能完成不代表 T03 真实收益或 M2 完成。

T02 全 cached whitespace 检查 exit2，仅 gates.txt:3 的原 fmt 空上下文与 gates-revised.txt:796/800 的原 cargo-deny 诊断尾空格；原字节不裁剪。明确排除这两份已知raw的作者范围检查exit0，[结构化结果](evidence/t02/whitespace-check.json)保存；不声称全cached通过。

## C005-T03 说明与授权延期交接

开工候选 `668f2220ac19cc29548d1c19feac4849b6128114`。只改技能与 package 状态元文案/记录，生产代码和 oracle 未变。[前提盘点](evidence/t03/preflight.json) 的真实 Work、撤销事件、执行者、宿主处置与接受/成本全为 null；没有制造事故或替代样本。原真实使用仍 not_run，按已授权无法执行项延期；runbook保留具体触发条件与步骤。

技能区分普通继续与撤销资格，规定新执行者开始前由操作者确认旧已停止或新环境已隔离；可以先撤销正式资格，不升级为引擎停止/隔离保证。独立 Reviewer 限定范围 PASS；不覆盖真实产品净收益。

[首次技能消费者](evidence/t03/skill-consumers.txt) run `d54560df-a0a3-4f1d-8009-714fe09e4f27` 为 9/9 PASS；文案两处修正后 [最终消费者](evidence/t03/skill-consumers-revised.txt) run `e2e42412-cead-461e-a4fc-d11cbf0acf4c` 为 9/9 PASS、1 LEAK unknown（generated_reference_keeps_authority_prose_verbatim）。原文分别保留，不重复计成不同用例或洗掉信号。check-skill/docs/specs/diff通过；无所属Rust测试，不运行零测试task.sh，也不重跑未受影响134秒引擎全门禁。原工具版本与fresh缓存边界保持。

## 最终限定实现验收表

候选 d8d8c20。与 T02 提交668f222相比，Rust code/tests、Cargo/lock、fixtures、配置、编译时include与工具环境均未变；only skills文案和package记录变化。按实际消费者分组：引擎/CLI/恢复/格式原run完全引用；被改变的技能另跑9项当前消费者；当前治理文件另检查。不是同一whole consumer hash，不能把T02全体输入宣称与T03完全相同。此前272file hash包含plan/skill等，域和分组区别保持。

| Requirement / risk | Mode | Input closure | Command / raw run ID | Result | Evidence |
| --- | --- | --- | --- | --- | --- |
| 原子替换、一次额度、真实失败及历史前缀 | executed | T02代码/测试/fixtures/config/Cargo未变，26基础与11feature均含最终回归 | 4b8d6190-d811-4235-9faf-c5ff24f4b84f；94fb155a-080a-44a5-b6ef-1fe3632ab473 | PASS | [774原文](evidence/t02/gates-revised.txt)、[11feature](evidence/t02/feature-revised.txt) |
| 冻结输入/FD/严格归属/重放/并发/原字节恢复 | executed | 同一Runtime/core code、fixture、features和环境；无后续source变化 | 同774原run；独立M1及T02定向原run | PASS | [验证原文](evidence/t02/gates-revised.txt)、[独立历史](evidence/t01/independent-runtime-history.txt) |
| CLI/next/错误优先级及所有冻结oracle | executed | c949新基准，仅删11ignore，六份tests/assert/fixtures/snapshots未变 | 4b8d6190-d811-4235-9faf-c5ff24f4b84f；scope前后 | PASS | [新冻结修订](evidence/t04/status-card-red.txt)、[恢复证明](evidence/t02/draft-restore.json)、[完整门禁](evidence/t02/gates-revised.txt) |
| 工程fmt/check/clippy与普通功能 | reused | Rust/Cargo/lock/config/features/toolchain/include输入未变；技能消费单独替换新证据 | T02同原run/fullgates，不重复134秒 | PASS | [工程原文](evidence/t02/gates-revised.txt) |
| 缓存依赖风险/许可/来源 | reused | deny policy/Cargo graph/全部features/缓存快照与db-path无变更 | 同T02 cached deny，RustSec117edb3缓存 | PASS | [缓存结果](evidence/t02/gates-revised.txt) |
| 技能、首次读者说明及前提/延期交接 | executed | T03新skill、preflight、同CLI和工具环境 | e2e42412-cead-461e-a4fc-d11cbf0acf4c，9/9；check-skill/docs/spec | PASS | [新消费者](evidence/t03/skill-consumers-revised.txt)、[前提](evidence/t03/preflight.json)、[runbook](experiments/runbook.md) |
| 声明MSRV编译兼容 | executed | 本机1.85.0/macOS arm64、同lockedCargo全部targets/features；隔离target与wrapper | cargo +1.85.0 check --locked --all-targets --all-features，exit0 | PASS | [MSRV原文](evidence/m2/msrv-check.txt) |

## 原义务、未执行与未知（不进入PASS表）

| 义务 / 信号 | 当前结果 | 原因与补全条件 | 保证边界 |
| --- | --- | --- | --- |
| 真实撤销/接手/约束/完整投入与用户接受 | not_run / authorized_defer | 未提供真实Work、撤销事件、旧新actor/宿主处置/接受原件；补后按runbook执行 | 夹具、说明和机制不等于价值 |
| nextest0.9.145要求的原版本门禁 | not_run | 原命令exit92；实际0.9.140override已执行 | 不称正确版本已跑 |
| online fresh advisory | not_run | 复用已有缓存快照，不伪称在线最新 | 当前cached结果仅对应固定HEAD |
| 其他平台及Rust1.85测试执行 | not_run | 已补macOS arm64/Rust1.85编译；测试实际由1.98.1执行，其他平台延期 | 不称跨平台或1.85测试已跑 |
| 局部与T01/T03的LEAK | unknown | 4条不同原run/case记录保留，未定位；最终T02干净不能证明过去无残留 | 测试断言PASS不等于永久无泄漏 |
| 两次cached原文whitespace检查 | raw_diagnostic_exception | 原工具上下文/尾空格保留，明确knownraw例外后的作者范围PASS | 不把作者范围称全cached通过 |

真实使用延期仅来自本轮用户授权，不删原产品实验义务。M2独立限定结论已追加；完成记录不授权发布、安装或宿主变更。

## C005-M2 最终独立限定验收

Reviewer `/root/independent_review` 未参与实现/oracle，固定完整候选 `d8d8c20fc71e3bd8bb813afecc6a4aa1af4a63f4`：限定 `PASS`。EX01–08、完整caller、计数/历史、同FD输入、事务恢复、冻结oracle与准确说明已核。按消费者复用同源T02的774原run与T03新9技能run，不称同whole闭包重跑。真实撤销价值、原nextest版本、onlinefresh、其他平台与永久无leak不在PASS范围，全部原失败和raw空格例外保留。

补充本机实际已安装Rust1.85.0，执行隔离target/wrapper的 [locked全targets/features编译](evidence/m2/msrv-check.txt) exit0、6.14s；不是1.85上的测试执行，也没有改源码或依赖。当前docs/specs/tests/skill及作者diff/scope另行检查，M2仅记录。

## 2026-10-04 C005-T05 恢复当前验收准备

C004当前agent质量/冷接续/三refs/逐查询新消费者使用已归档9862083，最终275blob含21日志读回同值、连续2655.603s；输入资格原件已存input-C004-final-qualification.json。C005旧62文件物理移动全SHA同值，旧限定候选、四LEAK、nextest92、1.85仅编译及raw空白例外留原，不从新目录推出PASS。

C004两个真实消费者明确当前只是普通接续，无真实running撤销目标/事件/宿主处置原件；七前提继续null，负前检不造样本。当前technical-freeze.json固定134工程/192完整输入逐SHA同值和37 C005注册用例，原C002 run424c的951/951、0.9.145/无skip/LEAK及freshdeny ef6173仅同闭包资格引用，不称旧C005要求版本已发生；数据时点是原C002取数，并不伪称本时刻在线刷新。

原1.85只有check，当前新增实际1.85全feature测试和5doc。执行前固定runner/12min连续上限、失败/漂移即停、所有原argv/stdoutstderr/exit、实际Cargo JSON的1.85两个产物及专用target/frozen路径。CLI测试使用本nextest原Cargo产物，export测试绑定实际1.85新副本，旧1.98冻结binary仅资格引用。准备独立审正在进行，未运行T06/引擎替换、未接受原真实价值或发布。

T05事前审查单一timeout处置finding已在实际运行前修复：每次启动预留8s，owned process group TERM后KILL不依赖leader是否已退出、实际leader exit与timed_out分列、全部成功前核连续总耗时<720。初版finding留原；Reviewer已核原62路径集合、旧四LEAK真实case/run、134与原C002工程字典逐同及37全部原PASS用例，不授未运行MSRV/M3。

## C005-T06 当前原生Rust1.85实际测试

T05提交5ccf9d4后启动真实runner，初实际rustc1.85.0/Cargo/nextest0.9.145输出与host aarch64-apple-darwin保存。MSRV Cargo build7.50s取得两个真实executable后复制到独立frozen目录，engine SHA-256 `5f5194f1bf345b4028b89e1670e98cee5b2ab929a9a205c460e1322dc64edce4`，export SHA-256 `17876365dbab3031e942cc151c3ac4512bdd4778aa31daa99facbe44c3c08832`；不是旧1.98二进制。

实际nextest run `8d822614-fca3-453f-b28e-2d8345e810e4` 为951/951 PASS、2slow、0skip/LEAK，测试139.594s；5个compile-fail doc全PASS，两个无doc crate的0样本不计额外用例。全部6步骤exit0/无timeout，总构建/版本/测试160.4732879s，原12min内。37 C005用例逐PASS原行核，全192源前后SHA无漂移；argv/env/真实Cargo JSON/各stdoutstderr摘要与binary SHA全存msrv-execution.json和t06-result/source核对。初runid解析器期待冒号但真实header无冒号，从原UUID匹配修正，仅记录解析不变原raw。

新MSRV测试填当前1.85实际执行缺口；原历史1.85只有check事实留原，原0.9.140/exit92/缓存/fresh时点/四LEAK cause unknown不追改。当前1.98 run424c与四类freshdeny按134/192同源输入精确引用，不说本轮重跑全部1.98或本刻刷新数据库；其他平台按用户范围排除。七真实撤销前提仍null、真实试用not_run，负前检不造样本。独立T06/M3审核正在进行，不以本actualtest推真实价值/接受或发布。

## C005-M3 当前技术完整限定验收

T06提交73520b2全原件及hooks已实际读回；独立Reviewer完整EX01–08及当前技术/负前检范围PASS，无必修。新实际Rust1.85/0.9.145六步/951/37/5doc/160.473s与134/192同源资格闭合。原native1.98全部四工程门禁和freshdeny按原run/数据时点精确引用，不把源未变当新命令执行。没有产品新源码或状态/持久格式，未额外突变或故障演示。

真实C005撤销七前提仍null，两真实C004消费者证明当前只普通续接，负前检有效；没有资格撤销需求就不执行替换样本。原真实试用/真人接受/成本/净收益not_run，四旧LEAK因果unknown、原次工具92与历史MSRV编译/原缓存/作者空白例外留原；新技术成功不注销过去。其他平台按用户范围排除。当前技术能力可收尾，原用户价值未执行不得写成全产品PASS；后续C006按顺序另任务。

归档只移动当前package并同步入口；所有实际文件精确stage、核index=physical，并保留原字节。最终治理/范围/提交与全commit blob集合逐SHA读回另实际核，不由review预授PASS。没有push/merge/release或宿主安装。

## 2026-10-05 当前采用范围收尾

真实撤销条件前检已完成，没有实际需撤销事件或同目标处置闭包，不执行replace；普通重开和历史active指针不充当撤销需求。 实际原件与范围见 [本次记录](evidence/acceptance-20261005/current-revocation-preflight.json)。旧验证和原失败不改。
