# C002 交接

跨会话交接用。任务状态只看 [plan.md](plan.md) 状态列；本文件写「接下来从哪继续」与跨任务事实，不复制任务状态。

## 当前位置

- 2026-09-27 用户采用 C002，指示按计划执行到完成。
- 实施进度只看 [plan.md](plan.md) 状态列；逐任务原始运行在 [evidence/tNN/](evidence/)。T01–T13 与 T15 已提交；下一个实现入口是 M1（固定候选全链 review，Owner 是未参与 T02–T15 实施的独立 Reviewer），之后 T16/T17 需要用户参与。
- 执行顺序按依赖：T01 → T14 → T02 → T03 → T04 → T09 → T05 → T06 → T07 → T08 → T10 → T11 → T12 → T13 → T15 → M1；T16/T17 需要用户参与。

## 跨任务事实

- 格式切换只有一次：T03/T06/T09 只交付纯实现与独立测试；T07 统一接入 schema 2、新布局、新摘要与 `cli-result/v2`，删除全部旧路径。T03–T09 期间产品行为不变。
- 新依赖已在 T01 固定：`fs4`（管理根写锁，D-035）、`users`（安全 Rust API 取得 OS 主体，D-036）。实现前按 engineering §1.2 核对目标平台与 MSRV。输出路径限定可移植 ASCII（workbook.md §3.2），别名判定只需 ASCII 大小写折叠，不加 Unicode 归一化依赖。
- Cargo 版本在 T17 发布前保持 `0.1.0`。check-specs 的版本规则（N11，T15 修复）分开验证：有 release record 的版本按已发布核 tag 与 Release commit；开发中的版本等于 active 目标版本或其 RC 就行，不要求已有 tag，CHANGELOG 允许先写 `[Unreleased]`。取不到 tag/commit 时浅克隆报「缺历史」，历史完整报「未创建或未推送」。
- release.yml 是 dist 生成文件：skill 打包一步与 `quality` 质量 job 都是自定义步骤，`dist init` 重新生成会丢掉，注释写明照此补回；`[workspace.metadata.dist]` 因此加了 `allow-dirty = ["ci"]`。announce 只在 `quality`（同一 SHA）成功时建 Release；MSRV 1.85 locked 门禁在 build.yml 的 `msrv` job 与 release.yml 的 `quality` job 各跑一份，stable 通过不能代替。
- 测试由各任务自己编写并挂 `// Task: C002-Tnn` 归属；期望值用独立 oracle（手工字节、合同数值、独立计算），不用生产 helper 生成。
- 提交用 `Change: C002`、`Task: C002-Tnn`、`Agent: <实际提交者>` trailer；通常一个任务一个提交。T01 首次提交审查失败，勘误另记纠正提交。提交前 fmt/check/clippy/nextest 与任务附加 gate 全绿，独立 review 通过。
- skill 交付是生成物：`scripts/skill-delivery.sh pack|tar|verify` 从 `skills/sheltie` 与 `specs/contracts/` 生成/校验自包含交付（发布资产名 `sheltie-skill.tar.gz`）。仓库内 `skills/sheltie/SKILL.md` 保留指向合同的链接；T16 准备「自包含 skill」用 `pack`，核已装副本用 `check-skill.sh --delivery <dir>`（校验对象要与当前树同源；`storage.md` 类文件不随包发布，是去链接后的文字提法）。
