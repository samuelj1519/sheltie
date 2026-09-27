# C001：Specs 与版本迭代文档治理

状态：`completed`
目标版本：`none`（文档治理）
兼容性：`none`
Owner：Codex
影响：文档结构、任务工具与 agent 入口；不改变产品行为
基线：`31d7ddee18921b4066c7433a752c2000e5110869`
当前事实：MVP、T26 与 `v0.1.0` 已完成；`v0.2.0` review 修复方案尚未实施

实施范围：G1–G3；只调整文档、检查脚本与 CI/pre-commit 接线，不修改产品代码、MVP 状态或 C002 产品实现状态。

本文设计 Sheltie 在 MVP 之后如何管理稳定规格、版本迭代、提案、计划、进度、决定、审查、验证和发布历史。D-032 已接受该设计；实施结果由本 package 的 validation 与 review 收口。来源笔记见 [文档治理一手来源](../../../research/2026-09-27-document-governance-sources.md)。

## 1. 方案结论

采用“**稳定权威文档 + 版本化 change package + release record**”三层结构：

1. 根 `specs/` 只保留当前目标与长期规则。
2. 每次具体迭代进入一个独立 change package。提案、设计、任务、进度和验证在同一目录闭合。
3. 发布后用 release record 固定 tag、候选、验收和已知限制；历史目标通过 Git tag 重建，不复制一套可能漂移的规格。
4. `AGENTS.md` 只做短入口、长期约束和命令地图，不写当前任务编号或临时结论。
5. 当前 `plan.md`、`decisions.md` 与 T25/T26 runbook 作为 MVP 历史关闭。后续版本不继续向这些大文件追加执行流水。

这个结构让“已完成 MVP”和“尚未采用的 review 修复”可以同时成立：前者进入 release record，后者放在 `changes/proposed/`，两者不争用同一个状态列。

## 2. 依据

- OpenAI Docs 说明 Codex 会按目录层级加载 `AGENTS.md`，近处规则覆盖上层，并有默认上下文大小限制。因此入口应短、稳定、就近分层，而不是总手册：[Custom instructions with AGENTS.md](https://learn.chatgpt.com/docs/agent-configuration/agents-md)。
- OpenAI 长任务实践把 spec、plan、runbook 与 live status 分开，避免从静态目标推断完成度：[Run long horizon tasks with Codex](https://developers.openai.com/blog/run-long-horizon-tasks-with-codex)。
- OpenAI 的 agent eval 指南要求先保存完整 trace，形成明确成功标准后再固化成可重复数据集与 eval；因此一次 review、稳定回归和发布证据要分开：[Evaluate agent workflows](https://developers.openai.com/api/docs/guides/agent-evals)。
- Anthropic 长任务实践使用 feature list、进度文件、Git 历史和逐项验证来支持跨会话续接：[Effective harnesses for long-running agents](https://www.anthropic.com/engineering/effective-harnesses-for-long-running-agents)。
- Anthropic Agent Skills 采用元数据、主说明和按需资源三级渐进披露；文档入口可以使用同一原则：[Agent Skills](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/overview)。
- [Diátaxis](https://www.diataxis.fr/) 区分 tutorial、how-to、reference、explanation，支持“一份文档只解决一种读者需求”。
- [Semantic Versioning 2.0.0](https://semver.org/) 要求先声明公共 API，且已发布版本不可静默改写。
- [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/) 把 changelog 定义为面向人的显著版本变化，不是 commit log 或任务看板。
- [Markdown Architectural Decision Records](https://adr.github.io/madr/) 用独立记录保存背景、选项、决定、后果和 superseded 关系。

## 3. 当前结构的问题

| 现状 | 影响 |
| --- | --- |
| `plan.md` 同时包含 MVP 方法、26 张任务卡、里程碑记录和当前进度 | MVP 完成后没有自然的下一版本入口；继续追加会让每次 agent 都读历史实现方法 |
| `decisions.md` 同时包含 ADR、复审报告、缺陷处置、流程教训和真实运行记录 | “为什么决定”与“某次运行发生什么”混在一起；文件已接近 700 行 |
| `t25-t26-runbook.md` 仍位于当前规范入口 | 已完成的一次性执行手册容易被未来 agent 当成当前步骤 |
| `AGENTS.md` 写着“T11 是下一步” | 始终加载的入口已经过期；agent 可能从错误任务开始 |
| 迁移前 review 与 repair 分别放在 `reviews/`、`repairs/` | 同一变更的问题、设计、计划和验证分散，采用状态不明显 |
| 根 `CHANGELOG.md`、Git log、plan 和 decisions 都记录“发生了什么” | 职责重复，读者难判断哪个是用户变化、哪个是内部任务或历史证据 |

## 4. 目标目录

```text
specs/
  README.md                         # 短地图、当前 release、active/proposed change
  constitution.md                   # 长期不变式
  spec.md                           # 当前开发线的产品目标与验收
  architecture.md                   # 当前目标架构
  engineering.md                    # 长期开发、验证、提交规则
  roadmap.md                        # 未立项方向与立项条件
  contracts/                        # 当前目标的字段、命令、存储 reference

  decisions/
    README.md                       # 决定索引和状态
    D-032-use-change-packages.md
    D-033-....md

  changes/
    README.md                       # change 索引与当前 active 指针
    proposed/
      C002-v0.2.0-reliability/
        README.md                   # 一屏摘要、范围、状态、入口
        findings.md                 # review finding 与根因追踪
        spec.md                     # 本 change 的产品 delta
        design.md                   # interface、数据、兼容策略
        plan.md                     # 任务、Owner、依赖、状态
        validation.md               # oracle、run ID、候选、结论
    active/
    completed/
    rejected/

  releases/
    README.md                       # release 索引
    v0.1.0.md                       # tag、release commit、验收闭包、已知限制

  guides/                           # how-to/runbook；只有仍可执行的操作手册
  research/                         # 一手来源笔记；无权威性
```

`CONTEXT.md` 继续留在仓库根，作为全仓词汇表。`README.md` 的快速开始继续承担 tutorial；contracts 是 reference；architecture/decisions 是 explanation；guides 是 how-to。

## 5. 单一事实来源

| 事实 | 唯一权威 | 不放在哪里 |
| --- | --- | --- |
| 永久不变式 | `constitution.md` | change plan、review |
| 当前目标产品行为 | `spec.md` | release note、ADR |
| 当前目标机制 | `architecture.md`、`contracts/` | task 卡、changelog |
| 未立项方向 | `roadmap.md` | active plan、Unreleased |
| 当前实施进度 | `changes/active/<id>/plan.md` | `AGENTS.md`、CHANGELOG、ADR |
| 当前跨会话交接 | active package 的 `progress.md` | spec、Git commit message |
| 重要设计原因 | `decisions/D-*.md` | architecture 正文、review 流水 |
| 一次审查发现 | change package 的 `findings.md` 或 `review.md` | ADR、产品 spec |
| 验证与候选结论 | change package 的 `validation.md` | plan 状态文字、CHANGELOG |
| 已发布事实 | `releases/<version>.md` 与 Git tag | active plan、roadmap |
| 用户可感知变化 | 根 `CHANGELOG.md` | commit log、finding 列表 |
| agent 长期规则 | `AGENTS.md` | 当前任务进度、历史复盘 |

其他文档只能链接这些事实，不重复定义。

## 6. Change package

### 6.1 命名与身份

change id 全局唯一、单调增加：`C001`、`C002`。目录名为 `<id>-<target-or-topic>`，例如：

```text
C002-v0.2.0-reliability
C003-workbook-bundle
```

任务 id 在 change 内局部编号，但提交和测试使用全局组合：`C002-T01`、`C002-M1`。既有 T01–T26 作为 MVP legacy id 保留，不重编号。

提交 trailer：

```text
Change: C002
Task: C002-T01
Agent: <实际执行者>
```

### 6.2 最小 README

每个 change package 的 README 第一屏必须包含：

```markdown
# C002：v0.2.0 可靠性修复

状态：proposed
目标版本：v0.2.0
基线：31d7dde...
Owner：待采用时指定
来源：review 链接
影响的 public API：protocol / storage / work directory

## 要解决的问题
## 成功判据
## 不做什么
## 文档入口
```

小改动可以把 spec、design、plan、validation 合并进 README。出现以下任一条件时拆文件：跨 crate、改变 public API/持久格式、超过三个任务、需要崩溃/真实宿主验证、正文超过约 300 行。

### 6.3 Package 文件职责

| 文件 | 回答什么 | 更新时机 |
| --- | --- | --- |
| `README.md` | 这是什么、状态、入口 | 状态转换时 |
| `findings.md` | 哪些问题已由什么证据确认 | review 后；只追加纠正 |
| `spec.md` | 这个 change 改变哪些产品行为与验收 | proposed 阶段 |
| `design.md` | interface、数据、顺序、兼容与否决方案 | 采用前完成 |
| `plan.md` | 谁按什么依赖完成哪些任务 | active 期间是进度权威 |
| `progress.md` | 当前候选、最后完成项、下一步、阻塞、失败尝试 | 每个工作 session 结束时重写 |
| `validation.md` | 输入闭包、命令/run ID、结果、未跑项 | 每个里程碑和最终验证 |
| `review.md` | 独立审查结论和 finding 关闭情况 | milestone/final review |

`progress.md` 不是第二份任务表。任务状态只在 `plan.md`；progress 只保存新会话快速接手所需的当前摘要。

## 7. 生命周期

```text
proposed ──采用──▶ active ──实现+验证+独立 review──▶ completed ──纳入──▶ release
    │                │
    └──否决──▶ rejected
                     active 也可回 proposed 重设计
```

目录位置就是状态；README 的状态行必须与父目录一致。

### proposed

- 可以有研究、review、spec、设计和计划。
- 不修改产品代码，不占用“当前计划”身份。
- 不进入 CHANGELOG `Unreleased`。
- 本阶段的 `PASS` 只表示方案可执行，不表示产品能力完成。

### active

只有人可以执行采用动作。采用提交必须同时：

1. 把 package 移到 `changes/active/`。
2. 指定 Owner、基线和目标版本。
3. 确认成功判据、范围和不做项。
4. 按 spec-first 规则更新根 `spec.md`、architecture/contracts 和必要 ADR。
5. 建立 `plan.md` 与任务白名单。
6. 更新 `changes/README.md` 的唯一 active 指针。

Sheltie 当前规模默认只允许一个 active change。必须并行时，两个 change 的文件 Owner、合同面和发布依赖必须互不冲突，并在索引中写明。

### completed

只有同时满足以下条件才移动：

- 所有 task 为 `done`，没有未解释的 `not_run`。
- 最终候选 hash 固定。
- validation 的必需门禁通过。
- 独立 review 为 PASS。
- 根权威文档与实现一致。
- progress 写明没有后续必做工作。

completed 只说明 change 已完成验证，不说明已经发布。

### rejected

保留问题、否决原因和替代 package 链接。不得删除后重新使用同一 change id。

## 8. 版本与发布

### 8.1 Sheltie 的 public API

版本判断至少覆盖：

- CLI 命令、参数、退出码与 `cli-result/v1`。
- `workbook/v1`、`flow/v1` 字段和校验。
- 错误码及其“已发生什么”。
- Store schema 与升级/回滚行为。
- skill 对协调者承诺的可观察流程。
- 返回给用户的 Work/Attempt 路径。内部辅助函数不属于 public API。

项目仍在 `0.y.z`：

- `PATCH`：保持上述合同兼容的缺陷修复和文档修正。
- `MINOR`：新增能力，或有意改变既有合同/持久格式/路径并提供清晰迁移说明。
- `-rc.N`：用于真实安装、升级、宿主和发布链验证。

每个 change README 必须写 `compatibility: patch | minor | breaking | none`。目标版本以合同影响决定，不按工作量决定。

### 8.2 Release record

`releases/v0.1.0.md` 只记录：

```text
状态：released + accepted
tag：v0.1.0
release commit：9f87188...
acceptance closure：31d7dde...
包含的 change：MVP legacy plan
门禁与真实宿主证据：链接
已知限制：链接
```

不复制一整套 spec/contracts。需要查看发布时规范时，直接打开 tag 中的文件。

### 8.3 Changelog

`CHANGELOG.md` 只记录用户可感知、已采用的显著变化。proposed finding、任务状态、测试数量和原始 commit 列表不进入 changelog。发布时把 `Unreleased` 内容移入版本节，并从 completed package 与 release record 校对。

## 9. Decision records

MVP 的 `specs/decisions.md` 标记为 closed legacy log，不再追加。新决定一项一文件：

```text
D-032-use-change-packages.md
D-033-version-work-directory-layout.md
```

最小结构：

```markdown
# D-032 使用 change package 管理迭代

状态：proposed | accepted | rejected | superseded by D-nnn
日期：YYYY-MM-DD
关联 change：C001

## 背景
## 选择
## 否决方案
## 后果
## 如何确认仍成立
```

ADR 解释“为什么”。当前 architecture/contracts 仍定义“现在是什么”。决定改变时新增 ADR 并互相链接，不覆写旧记录为新结论。

## 10. Agent 入口与渐进读取

`AGENTS.md` 保持一屏左右，只含：

1. 项目一句话。
2. 不变式与权限边界。
3. 常用门禁命令。
4. 文档路由：产品行为、机制、当前 change、工程规则分别去哪读。
5. 当前状态入口 `specs/README.md`，不写“Tnn 是下一步”。

建议开工顺序：

```text
AGENTS.md
→ specs/README.md
→ specs/changes/README.md
→ active package README
→ 当前 task 指定的少量权威文件
```

没有 active change 时，agent 应报告“当前无已采用实施计划”，不能自动从 proposed 中挑一项执行。

`CLAUDE.md` 继续只引用 `AGENTS.md`。局部 crate 只有出现真正不同的命令或约束时才增加嵌套 AGENTS；格式、lint 等机器可判规则继续交给 CI。

## 11. 进度、验证与证据

### plan 状态

任务只使用：`todo | doing | done | blocked`。`done` 必须有候选 commit 与当前输入闭包下的验证。`blocked` 必须写缺失输入或外部变化；困难、耗时和提案未采用不算 blocked。

### validation 模式

每条验证写：

```text
Requirement / risk
Mode: executed | reused | covered | not_applicable | not_run
Input closure / candidate hash
Command or raw run ID
Result: PASS | FAIL | BLOCKED
Evidence location
```

`executed/reused/covered` 是执行方式，不是 PASS。单次调试 trace 先进入 findings/validation；稳定行为再转成自动化测试或可重复 eval。大日志不直接塞进决策文件，只保留 run ID、摘要和可重跑命令。

### progress 模板

```markdown
# Current handoff

Candidate: <hash>
Current task: C002-T03
Last verified: <command/run id + result>
Next: <one concrete action>
Blocked: none | <condition>
Failed approaches: <做过什么、为什么不再试>
Dirty worktree: <paths or clean>
```

每次 session 结束时重写。Git 历史保存旧版本，active package 永远只有一个当前 handoff。

## 12. 机械检查

新增 `scripts/check-specs.sh`，由 pre-commit 与 CI 调用：

1. 每个 change id 唯一，目录状态与 README 状态一致。
2. 默认最多一个 active change；`changes/README.md` 指针存在。
3. active package 有 Owner、baseline、success criteria、plan、progress、validation。
4. completed package 所有任务 done，并有 candidate hash 与 final review PASS。
5. rejected/superseded 记录有原因和替代链接。
6. ADR 编号唯一，状态合法，superseded 链可解析。
7. release tag 与 commit 可解析，release record 链接到验证证据。
8. `AGENTS.md` 不出现当前任务编号、`todo/doing` 状态或已经关闭的阶段描述。
9. `CHANGELOG.md` 的版本与 Git tag、Cargo 版本一致。
10. 继续运行现有链接、测试归属、skill 与词汇检查。

脚本检查结构和闭包，不判断方案内容好坏。

## 13. 当前仓库的迁移方案

迁移拆成三个独立文档提交，不改产品代码。

### G1 建立治理骨架

- 接受本提案并新增 D-032。
- 建 `decisions/`、`changes/{proposed,active,completed,rejected}`、`releases/`、`guides/`、`research/` 索引。
- 新增 `check-specs.sh` 和模板。
- 更新 `AGENTS.md`，删除“T11 是下一步”，改为稳定路由。

### G2 收口 MVP

- 新建 `releases/v0.1.0.md`，记录 release/acceptance commit、T25/T26 和已知限制。
- 把 `plan.md` 标为“MVP legacy plan，closed”，停止追加；不在本次大规模移动，避免断链。
- 把 `decisions.md` 标为“MVP legacy decision/review log，closed”，新决定进入 `decisions/`。
- 把 `t25-t26-runbook.md` 标记 archived，并由 release record 链接。

### G3 归并未实施修复提案

- 创建 `changes/proposed/C002-v0.2.0-reliability/`。
- 将当前 T26 后 review 归为 `findings.md`，修复方案拆为 `README.md`、`design.md`、`plan.md`、`validation.md`。
- 状态保持 `proposed`；不修改产品代码，不把任务写成 doing/done，不写入 CHANGELOG `Unreleased`。
- 人决定采用时再移动到 active，并按 spec-first 规则更新根权威文档。

迁移完成后，根 `specs/README.md` 的当前状态应是：

```text
Released: v0.1.0（MVP complete，T26 complete）
Active change: none
Proposed: C002 v0.2.0 reliability
```

## 14. 完成判据

本治理方案实施完成当且仅当：

1. 新会话只读 `AGENTS.md`、specs 地图和 active README，就能准确回答当前 release、active change、下一任务和验证入口。
2. MVP 历史、当前目标和未实施 repair proposal 不共享同一状态表。
3. 每项事实在职责表中只有一个权威文件，其他位置只链接。
4. proposed package 不会被 agent 当成授权实施；没有 active change 时不会自行开工。
5. completed change 有候选、验证和独立 review；release record 能重建发布闭包。
6. `check-specs.sh`、`check-docs.sh` 与既有门禁全部通过。
7. `AGENTS.md` 不再因版本或任务推进而频繁修改。
