# C002 交接

跨会话交接用。任务状态只看 [plan.md](plan.md) 状态列；本文件写「接下来从哪继续」与跨任务事实，不复制任务状态。

## 当前位置

- 2026-09-27 用户采用 C002，指示按计划执行到完成。2026-09-28 用户明确跳过Linux相关运行，完成其余修复和M1。
- 实施进度只看 [plan.md](plan.md) 状态列；历史逐任务原始运行在 [evidence/tNN/](evidence/)。候选e1a8126的 [M1独立审查](review-m1-2026-09-28.md) 为“需修改”；追加修复的开工入口是 [repair-plan.md](repair-plan.md) 的T18，先固定上游调整/API门槛，再按T19–T31依赖修复与验证。
- 修复后重新固定M1候选，Reviewer不得参与被审代码实施；T16/T17继续需要真实宿主操作者和发布授权。新任务证据写到各任务专属 `evidence/tNN-*`，保留旧M1失败原文，不覆盖历史运行。T19由`ee78118`、T20由`22942ee`、T21由`543f9d2`提交并通过钩子；T22双轴review及macOS门禁通过，证据见[evidence/repairs/t22](evidence/repairs/t22/README.md)，接下来进入T23。

## 跨任务事实

- 格式切换只有一次：T03/T06/T09 只交付纯实现与独立测试；T07 统一接入 schema 2、新布局、新摘要与 `cli-result/v2`，删除全部旧路径。T03–T09 期间产品行为不变。
- 当前写锁使用`fs4`（D-035），OS主体使用D-036勘误后的`uzers`。追加文件方案选当前依赖闭包已有rustix1.1.4的安全fs API，T18完成macOS/MSRV1.85探针；用户豁免Linux运行，Linux留`not_run`，T19才加runtime直接依赖。输出路径仍限定可移植ASCII，别名仅ASCII折叠。
- 用户于2026-09-28授权按修复方案执行，并明确豁免本修复线的Linux验证。Linux结果全程保留`not_run`且不作为跨平台PASS；其余门禁在macOS执行。purge保留根/.lock、remove不预建payload、维护告警写stderr、SQLite共享内存控制文件只读例外已写入上游合同。T18由`69710aa`完成，包含macOS arm64/Rust 1.85.0探针、完整仓库门禁和独立Spec/Standards复核；T19由`ee78118`提交。T20已通过信任闭包负例、双轴review和macOS全门禁，证据在[evidence/t20-trusted-load-2026-09-28](evidence/t20-trusted-load-2026-09-28/README.md)，待提交。保持schema2与已有Workbook audit JSON字节格式，不迁移或清空旧记录。spec-dev交接采用不同节点的被审副本和累计verify报告，不放宽core禁止自来源规则。
- Cargo 版本在 T17 发布前保持 `0.1.0`。check-specs 的版本规则（N11，T15 修复）分开验证：有 release record 的版本按已发布核 tag 与 Release commit；开发中的版本等于 active 目标版本或其 RC 就行，不要求已有 tag，CHANGELOG 允许先写 `[Unreleased]`。取不到 tag/commit 时浅克隆报「缺历史」，历史完整报「未创建或未推送」。
- release.yml 是 dist 生成文件：skill 打包一步与 `quality` 质量 job 都是自定义步骤，`dist init` 重新生成会丢掉，注释写明照此补回；`[workspace.metadata.dist]` 因此加了 `allow-dirty = ["ci"]`。announce 只在 `quality`（同一 SHA）成功时建 Release；MSRV 1.85 locked 门禁在 build.yml 的 `msrv` job 与 release.yml 的 `quality` job 各跑一份，stable 通过不能代替。
- 测试由各任务自己编写并挂 `// Task: C002-Tnn` 归属；期望值用独立 oracle（手工字节、合同数值、独立计算），不用生产 helper 生成。
- 提交用 `Change: C002`、`Task: C002-Tnn`、`Agent: <实际提交者>` trailer；通常一个任务一个提交。T01 首次提交审查失败，勘误另记纠正提交。提交前 fmt/check/clippy/nextest 与任务附加 gate 全绿，独立 review 通过。
- skill 交付是生成物：`scripts/skill-delivery.sh pack|tar|verify` 从 `skills/sheltie` 与 `specs/contracts/` 生成/校验自包含交付（发布资产名 `sheltie-skill.tar.gz`）。仓库内 `skills/sheltie/SKILL.md` 保留指向合同的链接；T16 准备「自包含 skill」用 `pack`，核已装副本用 `check-skill.sh --delivery <dir>`（校验对象要与当前树同源；`storage.md` 类文件不随包发布，是去链接后的文字提法）。
