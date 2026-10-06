# 文档维护与历史查阅

[English](../../en/how-to/maintain-docs.md) | 简体中文

使用者从 docs 学习、操作、查询和理解；维护者从 specs 确定行为依据、规范、验收与采用范围。文档变更先核当前源码能力和精确合同，再决定要修改哪些读者材料。

## 1. 按用途放置内容

| 内容 | 位置 | 维护边界 |
| --- | --- | --- |
| 产品不变式、已采用行为、架构约束 | specs/constitution、spec、architecture | 单一规范，不将实现缺陷反写为承诺 |
| 字段、错误、格式与持久规则 | specs/contracts | 精确定义只维护一次，docs/en/reference 通过摘要与链接查阅 |
| 实现定位与限定验收 | docs/en/reference/implementation、acceptance | 区分存在实现、已验证、已发布与用户收益 |
| 未来方向与未完成变更 | specs/roadmap、changes | 保留需求触发、采用与当前进度 |
| 设计理由 | docs/en/explanation/decisions | 按主题索引，明确全部或部分替代关系，保留原采用理由 |
| 已关闭变更与发布记录 | docs/en/history/changes；docs/en/reference/releases | 保存历史范围、限制和固定快照，不混入当前主题解释 |
| 固定学习路径 | docs/en/tutorials | 明确起点、可预期动作与学习结果，不夹入所有真实任务分支 |
| 完成一个实际目标 | docs/en/how-to | 写前提、操作、停止和恢复；概念解释链接出去 |
| 查命令、状态、文件或限制 | docs/en/reference | 按产品结构组织事实，避免复制完整合同 |
| 理解概念、源码与设计取舍 | docs/en/explanation | 讲关系与原因，正式决定链接 ADR，外部来源标时点 |

分类依据是读者需求，不是标题或文件名。混合正文按用途拆开，四个目录不要求相同篇数。维护理念来自 [Diátaxis](https://diataxis.fr/)，项目采用关系见 [D-045](../explanation/decisions/D-045-specs-and-diataxis.md)。

## 2. 对齐当前实现

1. 从[实现基线](../reference/implementation.md)定位公开入口、合同与回归消费者，核实际字段和错误；不能仅凭 completed、函数名或测试存在声称完整覆盖。
2. 保持支持环境、版本、信任与未知边界。源码已经实现但未发布的能力明确标开发线；未采用方向留在 roadmap。
3. 修改行为时先同步规范，再同步受影响教程、操作、参考与解释。若实现与规范不符，记录差异后修实现或明确采用变更，不静默消除规范要求。
4. 在真实新管理根验证新增命令路径。验证成功不替代新人实际阅读、独立质量与真人接受。
5. 核全部入站链接、README 与 AGENTS 路由、skill 生成输入、检查工具和历史快照；历史固定路径不改写为当前目录。

术语来自 [CONTEXT](../../../CONTEXT.md)。命令、字段、版本、拒绝条件和原始结果保真；不存在同一个字段在 specs 与 docs 各有一份独立完整定义。

## 3. 完成后收敛

1. 在完整 completed package 中完成采用义务、必要门禁与独立审查，保存固定候选、输入闭包、原始结果及未知边界。
2. 将完整 package 提交进 Git，核对快照确实包含任务、验证、审查与原件。不能使用未包含材料的 SHA，也不能把另一份摘要当原件。
3. 将仍有效的行为并入上游规格／合同，设计理由并入 ADR，可重复使用的操作或评估方法提炼到 docs 的对应类别。主题解释按当前设计组织，关联 ADR 按主题登记并注明适用范围；部分选择被替代时明确前后关系，保留原理由。历史发布的范围与限制留在 release record。
4. 将英文 README 参考记录放入 docs/en/history/changes，并将对应中文记录放入 docs/zh-CN/history/changes，保留 `记录形式：reference`、完整 `历史快照` SHA 与基线，更新历史索引。删除 specs 中已关闭 package 的副本、执行计划、交接与证据；新记录不能把另一份摘要当完整资格原件。
5. 仍被工具使用的任务白名单、完成状态和测试声明只提取必要映射，放入 [scripts/task-history](../../../scripts/task-history/README.md)。不搬运整份计划或证据，不把该映射用作新任务计划。
6. 修复所有入站链接，更新阅读地图与相关检查。check-specs 仍从固定快照核完成任务、最终验证表和独立结论；摘要不能绕过原完成门禁。

这些步骤是维护方法，不自行采用新方案、批准门槛或授权发布。既有 active plan 要求的原件保存与审查必须先满足。

## 查阅历史原件

过程产物清理前的完整已提交文档快照为 `9d98bf8f15944b7bda4fbd4096e7771b09eab728`。它包含 C001–C011 的完整 completed package 和 MVP 历史材料。更早被移出的 C002 原始证据位于 `9ca6714e17c12dbe3f0e81056a03a781dbbfc8fd` 的 `specs/changes/active/C002-v0.2.0-reliability/`；这是归档 commit，不是发布 tag。

在仓库根读单份历史文件：

```bash
git show 9d98bf8f15944b7bda4fbd4096e7771b09eab728:specs/changes/completed/C011-workbook-visual-editor/design.md
git show 9d98bf8f15944b7bda4fbd4096e7771b09eab728:specs/releases/v0.1.0/plan.md
```

需要完整目录时，解压到新的临时目录：

```bash
history_dir=$(mktemp -d /private/tmp/sheltie-history.XXXXXX)
git archive 9d98bf8f15944b7bda4fbd4096e7771b09eab728 specs/changes/completed/C011-workbook-visual-editor | tar -x -C "$history_dir"
```

浅克隆可能没有所需历史。先用 `git cat-file -e <SHA>^{commit}` 核快照，缺失时取得包含该 commit 的完整历史；不能因取不到原件将结果默认为通过。CI 的治理检出要求 `fetch-depth: 0`。文档收敛不重写 Git 历史、不减少 .git 的历史体积。显式授权的提交信息迁移另行执行：引用按已核验的新旧映射同步，原始历史记录字节保持。

发布时的规格直接从发布 tag 读取，例如 `git show v0.2.0:specs/contracts/storage.md`。复查历史任务门禁时应在对应历史 checkout 使用当时源码、计划、工具和输入闭包；新工作区的映射只保留任务归属与范围检查用途，不能重建过去执行环境。

## 英文默认与中文版本

文档、开发说明、注释、Rust API 文档、产品文案以及新提交摘要和正文以英文为默认语言。人用文档按相同路径分别维护在 `docs/en/<path>.md` 和 `docs/zh-CN/<path>.md`，并提供双向语言导航；根 README 保留 `.zh-CN.md` 对应版本。英文为默认入口和权威，docs/README.md 提供两个目录的阅读入口。其他规格、仓库指令、变更日志和工具说明仅维护英文。中文 Workbook 与 skill 指令作为明确的 agent 输入保留；中文 spec-dev README 的修订历史会被 retrospective 读取，因此也保留。两个维护中的人用版本保留所有要求、限制、排除项和证据状态。当前语义变化同时更新两个版本；历史原始日志不静默翻译。

中文链接优先指向已有中文版本。命令、代码路径、协议字段、ID、字节与原始结果保真。默认方法使用英文指令；显式 `*-zh-CN` 目录使用独立中文 Workbook ID。每个版本经真实 CLI 装入、检查和运行，比较图语义、门槛、成果和恢复行为。引擎场景证明操作行为；实际 agent 的指令效果另按预注册质量和成本标准评估，结构测试不证明模型效果。

保留验证文本、路径、参数与归一化的有效 Unicode fixture。移除用户内容或历史原证据前先列清单，只有用途冗余且可恢复性已核验的材料才删除。未来提交按工程规范 §4 使用英文；历史提交信息迁移另核映射与元数据保持。

Wiki 源文件布局、生成、准备与发布见[管理 Wiki](manage-wiki.md)。文档在主仓库编写，独立 Wiki checkout 仅维护生成的展示文件。

## 修改后的检查

在仓库根运行：

```bash
scripts/check-docs.sh
scripts/check-specs.sh
scripts/check-tests.sh
git diff --check
```

改变治理脚本或任务映射时还须验证真实消费者和非法输入拒绝路径；纯文案不要求无关 Rust 全量重跑。命令块和源码链接需人工核对真实入口，机械链接通过不证明操作正确。
