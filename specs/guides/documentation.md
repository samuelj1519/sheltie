# 文档维护与历史查阅

specs 面向项目使用者、维护者和想理解源码的读者。正文应回答当前行为、如何操作、为什么这样设计，以及维护时需要守住什么。任务执行期间产生的材料在自己的 change package 内保存，完成后只将长期有用的内容纳入阅读集合。

## 一份内容放在哪里

| 内容 | 位置 | 维护方式 |
| --- | --- | --- |
| 产品边界、不变式、行为 | constitution／spec | 保持单一当前定义 |
| 模块、字段、命令、持久格式 | architecture／contracts | 与全部真实消费者同步 |
| 操作步骤与源码解释 | guides | 保留前提、失败与恢复路径，删除过期时点事实 |
| 重要选择的理由 | decisions | 一个决定一个记录；替代时保持编号和互相链接 |
| 未来方向 | roadmap | 写需求触发与采用条件，不写成当前能力 |
| 外部参考 | research | 标明来源、时点和项目推论，不作为项目权威 |
| 未完成实施 | active change | plan 管进度，progress 管交接，validation／review 管资格与证据 |
| 已完成变更 | completed 摘要 | 保留问题、变化、理由、验证范围、限制和阅读入口 |
| 正式发布 | releases | 固定 tag、源码、验收闭包、实物与历史限制 |

入口按读者目的组织，避免在首页复制全部版本流水。一个事实只在一处定义，其他文件链接它。术语见 [CONTEXT](../../CONTEXT.md)。文案编辑不能改变字段、命令、版本、限制或原始结果。

## 完成后收敛

1. 在完整 completed package 中完成采用义务、必要门禁与独立审查，保存固定候选、输入闭包、原始结果及未知边界。
2. 将完整 package 提交进 Git，核对快照确实包含任务、验证、审查与原件。不能使用未包含材料的 SHA，也不能把另一份摘要当原件。
3. 将仍有效的行为并入上游规格／合同，设计理由并入 ADR，可重复使用的操作或评估方法提炼为指南。历史发布的范围与限制留在 release record。
4. 为 package 保留 README 参考摘要，写 `记录形式：reference` 和完整 `历史快照` SHA。删除完成后无长期用途的 plan、progress、阶段交接、重复报告、临时实验副本和原始运行输出。
5. 仍被工具使用的任务白名单、完成状态和测试声明只提取必要映射，放入 [scripts/task-history](../../scripts/task-history/README.md)。不搬运整份计划或证据，不把该映射用作新任务计划。
6. 修复所有入站链接，更新阅读地图与相关检查。check-specs 仍从固定快照核完成任务、最终验证表和独立结论；摘要不能绕过原完成门禁。

这些步骤是维护方法，不自行采用新方案、批准门槛或授权发布。既有 active plan 要求的原件保存与审查必须先满足。

## 查阅历史原件

本轮清理前的完整已提交文档快照为 `f38954d543ff01eb5a798be48f29060b80d5952c`。它包含 C001–C011 的完整 completed package 和 MVP 历史材料。更早被移出的 C002 原始证据位于 `e54dcd41d8f1e186007b62b47583063cb19a4b66` 的 `specs/changes/active/C002-v0.2.0-reliability/`；这是归档 commit，不是发布 tag。

在仓库根读单份历史文件：

```bash
git show f38954d543ff01eb5a798be48f29060b80d5952c:specs/changes/completed/C011-workbook-visual-editor/design.md
git show f38954d543ff01eb5a798be48f29060b80d5952c:specs/releases/v0.1.0/plan.md
```

需要完整目录时，解压到新的临时目录：

```bash
history_dir=$(mktemp -d /private/tmp/sheltie-history.XXXXXX)
git archive f38954d543ff01eb5a798be48f29060b80d5952c specs/changes/completed/C011-workbook-visual-editor | tar -x -C "$history_dir"
```

浅克隆可能没有所需历史。先用 `git cat-file -e <SHA>^{commit}` 核快照，缺失时取得包含该 commit 的完整历史；不能因取不到原件将结果默认为通过。CI 的治理检出要求 `fetch-depth: 0`。此清理不重写 Git 历史、不减少 .git 的历史体积。

发布时的规格直接从发布 tag 读取，例如 `git show v0.2.0:specs/contracts/storage.md`。复查历史任务门禁时应在对应历史 checkout 使用当时源码、计划、工具和输入闭包；新工作区的映射只保留任务归属与范围检查用途，不能重建过去执行环境。

## 修改后的检查

在仓库根运行：

```bash
scripts/check-docs.sh
scripts/check-specs.sh
scripts/check-tests.sh
git diff --check
```

改变治理脚本或任务映射时还须验证真实消费者和非法输入拒绝路径；纯文案不要求无关 Rust 全量重跑。命令块和源码链接需人工核对真实入口，机械链接通过不证明操作正确。
