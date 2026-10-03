可审查：同一 implement-1 的冷接续已补齐授权指南，两项既定检查退出 0；独立 review、完整 patch 应用和用户接受尚未执行。

# 任务与候选

目标：为首次接触项目的操作者新增接续与资格撤销选择指南，区分同 running Attempt 接续、真实执行 fail、内容 review back 与行政 replace。只修改 `specs/guides/continuity-choices.md`；无其他产品文件、标准、测试或方法改动。

- 仓库：`/private/tmp/sheltie-completion-20261003/c007-agent-paired-study/repos/continuity/native`。
- 初始 HEAD：`351feb7ac22c21317a686693b732d5ae0c4b4bcc`，tree：`80ea3046b1b3575f07e68313444c051ee7c5b7db`。
- 当前候选：`c9823575920795db14dfd378d77c8b311ce71d87`，tree：`404c3004895a9451d7cfc88b0c5a04400a4a0463`。
- 指南 SHA-256：`d7f53c99fae26656972b941b604cb2547fe5461790a973a7cb3dffed8cbc91aa`，13959 字节。
- 检查时的 staged tree 与提交后的 tree 完全相同；提交后工作区干净，初始基线到候选仅包含允许文件。身份与范围原件：`/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C007-pre-run-workbook-generation/evidence/resume-20261004/study/runs/agent-doc-20261004-04-native-73d637d1-86ee-4c61-b2af-5c1f04479728/raw/implement-1/candidate-identity.json`、`/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C007-pre-run-workbook-generation/evidence/resume-20261004/study/runs/agent-doc-20261004-04-native-73d637d1-86ee-4c61-b2af-5c1f04479728/raw/implement-1/candidate-head.json`、`/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C007-pre-run-workbook-generation/evidence/resume-20261004/study/runs/agent-doc-20261004-04-native-73d637d1-86ee-4c61-b2af-5c1f04479728/raw/implement-1/candidate-status.json`、`/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C007-pre-run-workbook-generation/evidence/resume-20261004/study/runs/agent-doc-20261004-04-native-73d637d1-86ee-4c61-b2af-5c1f04479728/raw/implement-1/candidate-paths.json`、`/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C007-pre-run-workbook-generation/evidence/resume-20261004/study/runs/agent-doc-20261004-04-native-73d637d1-86ee-4c61-b2af-5c1f04479728/raw/implement-1/candidate-show.json`。

# 实际变更与依据

初始冻结基线没有本指南。前 helper 留下 5465 字节未跟踪草稿，已完成状态查询、同 Attempt 接续、真实 fail 与内容返工的解释。本 helper 从本 run binding/stage/partial-state 自行恢复，核对同仓库、初始 HEAD/tree、dirty 和草稿 SHA `693cef118e615f1c10d2cc15e9d197a6380d531bd26827ab5dd1bfe6446418e6`。保留原有这些段落，扩展标题并补上全部未完成章节；未造返工事件、未重复访问阶段或重复计方法使用。

指南现在说明 replace 的真实理由、资格、一次/Occurrence 额度、原子旧 superseded/新 running、number 与失败数、entered_from 与 optional null/冻结非统计输入、提交后 engine.stats、旧草稿边界、旧 submit/fail 的拒绝及终态优先级；同时说明 EFFECT_PENDING 的 committed true/false、A/B request ID、same-intent 恢复、original 缺省与 unknown 停止，最后补齐只读、human、gate、终态与最终成果条件。所有示例使用显式 Home/JSON，只在写命令使用 request ID；重放历史 next 后必须刷新 status。指南不称引擎停止进程、认证执行者或隔离宿主。

依据来自本 assigned repo 的 CONTEXT、specs README/engineering、protocol §§1/2/3/5/6/7、storage §§2.1/3/5.5/6、C004 README、C005 spec/design、既有 sheltie skill，以及 CLI `commands/attempt.rs` 和 core `work/next.rs` 的实际调用与分支。中文写作应用共享 `tech-doc-style-chinese` 及受控中文参考。作者事实/链接复核、修订和 scope 记录：`/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C007-pre-run-workbook-generation/evidence/resume-20261004/study/runs/agent-doc-20261004-04-native-73d637d1-86ee-4c61-b2af-5c1f04479728/raw/cold-implement-1/factual-review.json`、`/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C007-pre-run-workbook-generation/evidence/resume-20261004/study/runs/agent-doc-20261004-04-native-73d637d1-86ee-4c61-b2af-5c1f04479728/raw/cold-implement-1/edit-record.json`。这不是独立审查。

未绑定 previous-review；本次是正常冷接续，非审查返工。旧 raw/partial-state 与旧草稿 SHA 事实保留；没有覆盖旧检查或旧报告。冷接续入口与核对原件：`/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C007-pre-run-workbook-generation/evidence/resume-20261004/study/runs/agent-doc-20261004-04-native-73d637d1-86ee-4c61-b2af-5c1f04479728/raw/cold-implement-1/resume-activity-start.json`、`/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C007-pre-run-workbook-generation/evidence/resume-20261004/study/runs/agent-doc-20261004-04-native-73d637d1-86ee-4c61-b2af-5c1f04479728/raw/cold-implement-1/resume-head.json`、`/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C007-pre-run-workbook-generation/evidence/resume-20261004/study/runs/agent-doc-20261004-04-native-73d637d1-86ee-4c61-b2af-5c1f04479728/raw/cold-implement-1/resume-status.json`、`/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C007-pre-run-workbook-generation/evidence/resume-20261004/study/runs/agent-doc-20261004-04-native-73d637d1-86ee-4c61-b2af-5c1f04479728/raw/cold-implement-1/resume-top.json`。

# 使用与限制

操作者从指南第一条 status 查询开始，用现有 Work 的同一个管理根、完整 Work ID 和当前 next 选择；新试用使用未使用的独立 Home。指南中的尖括号参数需替换，命令仅为说明，本 run 没有执行 Sheltie、写 Store、安装或发布。未知结果按指南保留记录，不为流程制造失败/撤销事实。

本阶段两项既定机械检查实际成功，候选可交独立 review。Rust/build/tests、CLI 演练、其他文档全仓检查、独立 review、完整 patch 生成/应用、最终盲审、用户接受、费用与收益评价均未由本 helper 执行（not_run）；usage/fees unknown。当前成功不代表这些范围通过。失败检查为无；没有重跑凑绿。模型按绑定声明继承 gpt-6.1-sol/high，无 override；provider 实际身份未独立暴露，由 Root 另核。所有自启动命令已 await，无后台任务、无额外 helper；未改 stage/run 索引或共同方法。

# 已冻结完整 patch 与独立应用方法

由后续 deliver 使用 project 的原规则，基线固定为 `351feb7ac22c21317a686693b732d5ae0c4b4bcc`，在上述仓库执行：

```bash
git diff --binary --full-index 351feb7ac22c21317a686693b732d5ae0c4b4bcc HEAD -- specs/guides/continuity-choices.md
```

将完整 stdout 保存到 deliver 指定 `change.patch`。新增文件必须先 stage/commit；本次已提交该新增指南，不能用普通 unstaged diff 替代。核完整 diff 的文件集合恰为 allow_files，候选仍为上述 HEAD/tree，patch 上限 8388608 字节。

只在预授权 Native 独立副本 `/private/tmp/sheltie-completion-20261003/c007-agent-paired-study/patch-check/continuity/native` 应用。先从原始 archive/full initial snapshot 建立该副本，核 HEAD/tree 与上述初始基线相同、worktree/index 干净；不得用已改 candidate 工作区充当原基线。依序执行，参数替换为实际完整 patch 路径：

```bash
git -C "/private/tmp/sheltie-completion-20261003/c007-agent-paired-study/patch-check/continuity/native" apply --check "<deliver 的完整 change.patch 绝对路径>"
git -C "/private/tmp/sheltie-completion-20261003/c007-agent-paired-study/patch-check/continuity/native" apply --index "<deliver 的完整 change.patch 绝对路径>"
git -C "/private/tmp/sheltie-completion-20261003/c007-agent-paired-study/patch-check/continuity/native" write-tree
```

所得 tree 必须等于 `404c3004895a9451d7cfc88b0c5a04400a4a0463`；授权文件集合及全部字节须与候选一致。保留实际 argv/cwd/env/stdout/stderr/exit 和身份原件。不得在源仓库、另一组或宿主试应用。本 helper 的应用状态为 not_run；此处只复述已冻结方法，没有提前进入 deliver。

报告写成时间：`2026-10-03T20:56:11.039977+00:00`；仍在原 run hard deadline `2026-10-03T21:08:27.513464+00:00` 内。
