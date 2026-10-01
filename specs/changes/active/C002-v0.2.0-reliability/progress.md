# C002 交接

跨会话交接用。任务状态只看 [plan.md](plan.md) 状态列；本文件写「接下来从哪继续」与跨任务事实，不复制任务状态。

## 当前位置

- 2026-09-27 用户采用 C002，指示按计划执行到完成。2026-09-28 用户明确跳过Linux相关运行，完成其余修复和M1。
- 实施进度只看 [plan.md](plan.md) 状态列；历史逐任务原始运行在 [evidence/tNN/](evidence/)。候选e1a8126的 [M1独立审查](review-m1-2026-09-28.md) 为“需修改”；追加修复的开工入口是 [repair-plan.md](repair-plan.md) 的T18，先固定上游调整/API门槛，再按T19–T31依赖修复与验证。
- 修复后重新固定M1候选，Reviewer不得参与被审代码实施；T16/T17继续需要真实宿主操作者和发布授权。新任务证据写到各任务专属 `evidence/tNN-*`，保留旧M1失败原文，不覆盖历史运行。T19由`ee78118`、T20由`22942ee`、T21由`543f9d2`、T22由`fd21a63`、T23由`7e9178e`、T24由`5d2d051`提交并通过钩子；另针对初始化并发竞态追加T24 follow-up提交`edb86f1`。T23/T24证据见[evidence/repairs/t23](evidence/repairs/t23/README.md)、[evidence/repairs/t24](evidence/repairs/t24/README.md)与[T24并发初始化follow-up](evidence/repairs/t24-followup/README.md)。T25已提交`1d92bcd`，通过双轴独立Review、macOS全门禁与提交后任务门禁；cleanup诊断按计划交T28/V25，由T28关闭。T26已提交`5a9d430`，通过Spec/Standards独立Review、macOS Rust/MSRV/deny/文档门禁、提交钩子和提交后任务门禁；Linux保持`not_run`。T27已提交`3dd224d`，通过Spec/Standards独立Review、macOS Rust/MSRV/deny/文档门禁、提交钩子和提交后任务门禁；全仓Nextest 593项通过、0跳过、1 slow，1项leaky隔离复跑通过但原因未确认。stat→unlink边界按HomeLock协作模型记录，不声称能原子约束不遵守锁的外部写者。T28已提交`4526b7e`，通过双轴独立Review、macOS全仓615项测试、MSRV1.85与deny/文档门禁及提交钩子、提交后任务门禁，证据见[evidence/repairs/t28](evidence/repairs/t28/README.md)。T29完成真实CLI装入后writer交错修复，通过双轴独立Review、macOS全仓616项测试及MSRV/deny/文档门禁，证据见[evidence/repairs/t29](evidence/repairs/t29/README.md)；提交后进入T30。全仓纯静态测试的Nextest LEAK原因未确认，交T31综合验证。Linux保持`not_run`。

## 跨任务事实

- 格式切换只有一次：T03/T06/T09 只交付纯实现与独立测试；T07 统一接入 schema 2、新布局、新摘要与 `cli-result/v2`，删除全部旧路径。T03–T09 期间产品行为不变。
- 当前写锁使用`fs4`（D-035），OS主体使用D-036勘误后的`uzers`。追加文件方案选当前依赖闭包已有rustix1.1.4的安全fs API，T18完成macOS/MSRV1.85探针；用户豁免Linux运行，Linux留`not_run`，T19才加runtime直接依赖。输出路径仍限定可移植ASCII，别名仅ASCII折叠。
- 用户于2026-09-28授权按修复方案执行，并明确豁免本修复线的Linux验证。Linux结果全程保留`not_run`且不作为跨平台PASS；其余门禁在macOS执行。purge保留根/.lock、remove不预建payload、维护告警写stderr、SQLite共享内存控制文件只读例外已写入上游合同。T18由`69710aa`完成，包含macOS arm64/Rust 1.85.0探针、完整仓库门禁和独立Spec/Standards复核；T19由`ee78118`提交。T20由`22942ee`提交并通过信任闭包负例、双轴review和macOS全门禁，证据在[evidence/t20-trusted-load-2026-09-28](evidence/t20-trusted-load-2026-09-28/README.md)。保持schema2与已有Workbook audit JSON字节格式，不迁移或清空旧记录。spec-dev交接采用不同节点的被审副本和累计verify报告，不放宽core禁止自来源规则。
- Cargo 版本在 T17 发布前保持 `0.1.0`。check-specs 的版本规则（N11，T15 修复）分开验证：有 release record 的版本按已发布核 tag 与 Release commit；开发中的版本等于 active 目标版本或其 RC 就行，不要求已有 tag，CHANGELOG 允许先写 `[Unreleased]`。取不到 tag/commit 时浅克隆报「缺历史」，历史完整报「未创建或未推送」。
- release.yml 是 dist 生成文件：skill 打包一步与 `quality` 质量 job 都是自定义步骤，`dist init` 重新生成会丢掉，注释写明照此补回；`[workspace.metadata.dist]` 因此加了 `allow-dirty = ["ci"]`。announce 只在 `quality`（同一 SHA）成功时建 Release；MSRV 1.85 locked 门禁在 build.yml 的 `msrv` job 与 release.yml 的 `quality` job 各跑一份，stable 通过不能代替。
- 测试由各任务自己编写并挂 `// Task: C002-Tnn` 归属；期望值用独立 oracle（手工字节、合同数值、独立计算），不用生产 helper 生成。
- 提交用 `Change: C002`、`Task: C002-Tnn`、`Agent: <实际提交者>` trailer；通常一个任务一个提交。T01 首次提交审查失败，勘误另记纠正提交。提交前 fmt/check/clippy/nextest 与任务附加 gate 全绿，独立 review 通过。
- skill 交付是生成物：`scripts/skill-delivery.sh pack|tar|verify` 从 `skills/sheltie` 与 `specs/contracts/` 生成/校验自包含交付（发布资产名 `sheltie-skill.tar.gz`）。仓库内 `skills/sheltie/SKILL.md` 保留指向合同的链接；T16 准备「自包含 skill」用 `pack`，核已装副本用 `check-skill.sh --delivery <dir>`（校验对象要与当前树同源；`storage.md` 类文件不随包发布，是去链接后的文字提法）。

T29已由 `a4f1968` 提交并通过钩子与提交后任务门禁。T30完成required被审镜像、递归累计交接与真实CLI/Git冷读正反例；全仓验收暴露测试内Cargo重建共享CLI的ENOENT，已提前完整迁入T31原定binary注入步骤，Owner与断言保留。T30双轴独立Review及macOS门禁通过，详见 [T30证据](evidence/repairs/t30/README.md)。下一入口为repair-plan的T31：确定性交错、完整窗口、完整mutant inventory及存活处置，再固定M1候选。Linux仍为用户豁免的not_run。

2026-09-30 T31仍为WIP。macOS默认profile全仓 `6e03fa98-0535-4d68-bc1f-f4412a609ac8` 647/647，T31 `27ffc52b-c5a0-472b-ac1e-07c262c3abb0` 22/22；同源码优化测试profile普通clone全仓 `36cf2302-efc8-48f8-a1a4-1d5eb1770eb9` 647/647。新增R20的独立CLI探针确认不可能的blocked_count会使gate approve panic，现用已有Attempt/Approval与当前Blocked事实的必要界在可信装入时拒绝，core三处增量checked，结构化错误/零业务写与纯core回归已通过两位独立Reviewer增量复核。旧候选的partial mutants均归档、不复用；当前持久普通clone临时输入`da2bd13979df88dd987596d7ed726ef99bb89651`的2573个变异待新647项baseline通过后完整重跑；先前0aee的core结果归档、不复用，存活体处置和独立M1仍未完成。详见[evidence/repairs/t31](evidence/repairs/t31/README.md)。Linux按用户要求`not_run`，T16真实Host与T17发布仍`not_run`。

2026-09-30 用户授权修复实现审查全部七项并继续完成T31，Codex接续实施。逐条答复见 [review-response-implementation-2026-09-30.md](review-response-implementation-2026-09-30.md)。最终654项默认门禁及MSRV/deny/规范/dist plan均通过，两位未参与实施的Reviewer复核七项通过；已published历史补缺sync的追加反例已红→绿。此前da2bd变异原始结果归档为superseded，不计新输入；新普通clone临时提交3a9f689固定170项源码/fixture/配置，完整mutation与存活体处置执行中，T31仍以plan的doing为准。Linux仍not_run，M1/T16/T17未关闭。

## 2026-10-01 续接

当前源码已冻结为普通clone临时候选`49d3a191aa4c918aab279617fa2bf7d9b3b36a0e`，167项源码/fixture/配置/脚本与root一致、164项治理输入在clone冻结。默认675项测试及Rust/MSRV/离线deny/规范/dist门禁通过，Spec/Standards增量审查通过；完整mutation与处分仍在运行，不能关闭T31。接续入口为[evidence/repairs/t31/mutants/closure.md](evidence/repairs/t31/mutants/closure.md)：完成runtime片和全部workspace复验，逐missed/timeout/unviable建立准确处分与独立复核，再运行任务门禁并提交。新输入完整inventory为2499（core724/runtime1775），旧c31的2501项结果完整归档、不复用为最终PASS。Linux/M1/T16/T17边界不变。

## 2026-10-01 本次收尾

用户明确要求暂缓可能触发额外安全检查的相关任务并完成T31，主计划已记录豁免。混合runtime变异流水线精确停止并核进程结束；完整2499项第一阶段与第二阶段245项终态原文保留，全部ID分类守恒，269项为deferred_by_user而非PASS。正常675项与Rust/治理/MSRV/离线deny/dist门禁通过，独立Spec/Standards认可七项修复及本次豁免范围内提交。最终证据见[evidence/repairs/t31/README.md](evidence/repairs/t31/README.md)；任务/staged门禁与治理树绑定随提交收尾。T31主表done仅针对本次授权范围，M1/T16/T17及Linuxnot_run不变。
