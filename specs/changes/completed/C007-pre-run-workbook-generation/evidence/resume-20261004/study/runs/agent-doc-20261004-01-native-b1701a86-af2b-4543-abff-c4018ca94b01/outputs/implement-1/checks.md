通过声明的两条必需检查：实际退出码均为 0，绑定 staged tree `3b9e14dfb50d3b577cc4caaacdf33e96082026fc`，提交 tree 同一；此结论仅覆盖机械检查。

## 原始检查记录

仓库 cwd 均为 `/private/tmp/sheltie-completion-20261003/c007-agent-paired-study/repos/source-start/native`。检查在两授权文件 stage 后、candidate commit 前运行；当时 HEAD 为初始 `351feb7ac22c21317a686693b732d5ae0c4b4bcc`，index tree 为 `3b9e14dfb50d3b577cc4caaacdf33e96082026fc`。提交后 HEAD `8366454ddbc2a07d25b2e5e949171cde5b5169ae` 的 tree 与文件 SHA/bytes 完全相同。没有候选内容漂移，因此沿用同一输入树的两条实际检查，不重复运行。

| 必需 argv | exit | UTC start / end | 原件 |
| --- | --- | --- | --- |
| `scripts/check-docs.sh README.md specs/guides/source-quick-start.md` | 0 | 2026-10-03T19:54:59.107362+00:00 / 2026-10-03T19:54:59.702257+00:00 | `/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C007-pre-run-workbook-generation/evidence/resume-20261004/study/runs/agent-doc-20261004-01-native-b1701a86-af2b-4543-abff-c4018ca94b01/raw/implement-1/check-docs.json`；同前缀 `.stdout` / `.stderr` |
| `git diff --cached --check` | 0 | 2026-10-03T19:54:59.702895+00:00 / 2026-10-03T19:54:59.715281+00:00 | `/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C007-pre-run-workbook-generation/evidence/resume-20261004/study/runs/agent-doc-20261004-01-native-b1701a86-af2b-4543-abff-c4018ca94b01/raw/implement-1/check-cached-diff.json`；同前缀 `.stdout` / `.stderr` |

每份 capture record 原样保存实际 argv、cwd、control_environment、stdout/stderr 绝对路径和 SHA256、退出码、是否超时及 deadline 状态。`check-docs.stdout` 为 `check-docs: OK (2 个文件)` 加换行，stderr 空；diff 检查 stdout/stderr 均空。两条均未 timeout 且 within_deadline=true，单 check 上限 120s，总 run deadline `2026-10-03T20:10:26.252142+00:00`。

补充实际环境在 `/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C007-pre-run-workbook-generation/evidence/resume-20261004/study/runs/agent-doc-20261004-01-native-b1701a86-af2b-4543-abff-c4018ca94b01/raw/implement-1/candidate-before-checks.json` 的 relevant_environment，含 PATH/LANG/LC_ALL/LC_CTYPE/SHELL 和影响 Git 的环境项；检查捕获时 LANG=en_US.UTF-8、LC_ALL/LC_CTYPE=C.UTF-8，GIT_INDEX_FILE/GIT_DIR/GIT_WORK_TREE/GIT_CONFIG_COUNT 均 null。capture 的 SHELTIE_* control 环境原值见对应原件，不自行归零。两条都是声明 argv，无替换、删除或重跑凑绿。

## 候选与附属原件

- `/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C007-pre-run-workbook-generation/evidence/resume-20261004/study/runs/agent-doc-20261004-01-native-b1701a86-af2b-4543-abff-c4018ca94b01/raw/implement-1/pre-stage.json`：初始 HEAD/tree、实际既有状态和 allow_files。
- `/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C007-pre-run-workbook-generation/evidence/resume-20261004/study/runs/agent-doc-20261004-01-native-b1701a86-af2b-4543-abff-c4018ca94b01/raw/implement-1/stage.json` 及同前缀 stdout/stderr：实际 `git add -- README.md specs/guides/source-quick-start.md`。
- `/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C007-pre-run-workbook-generation/evidence/resume-20261004/study/runs/agent-doc-20261004-01-native-b1701a86-af2b-4543-abff-c4018ca94b01/raw/implement-1/candidate-before-checks.json`：检查前 index tree、两项 SHA/bytes 和环境。
- `/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C007-pre-run-workbook-generation/evidence/resume-20261004/study/runs/agent-doc-20261004-01-native-b1701a86-af2b-4543-abff-c4018ca94b01/raw/implement-1/checks-bindings.json`：逐必需检查 record → staged tree 绑定。
- `/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C007-pre-run-workbook-generation/evidence/resume-20261004/study/runs/agent-doc-20261004-01-native-b1701a86-af2b-4543-abff-c4018ca94b01/raw/implement-1/candidate-commit.json` 及 stdout/stderr：真实本地 candidate commit argv、时间和 exit 0；正文原件 `/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C007-pre-run-workbook-generation/evidence/resume-20261004/study/runs/agent-doc-20261004-01-native-b1701a86-af2b-4543-abff-c4018ca94b01/raw/implement-1/commit-message.txt`。
- `/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C007-pre-run-workbook-generation/evidence/resume-20261004/study/runs/agent-doc-20261004-01-native-b1701a86-af2b-4543-abff-c4018ca94b01/raw/implement-1/candidate-after-commit.json`：commit HEAD/tree/parent、clean 状态、完整改变路径集合和两项 SHA/bytes，已断言与检查树相同。
- `/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C007-pre-run-workbook-generation/evidence/resume-20261004/study/runs/agent-doc-20261004-01-native-b1701a86-af2b-4543-abff-c4018ca94b01/raw/implement-1/actor.json`：真实 worker 身份、各可观测 UTC、模型/effort 声明来源、无 helpers、usage/fees unknown/null。
- `/Users/shushu/orca/workspaces/sheltie/codex/specs/changes/active/C007-pre-run-workbook-generation/evidence/resume-20261004/study/runs/agent-doc-20261004-01-native-b1701a86-af2b-4543-abff-c4018ca94b01/raw/implement-1/execute-checks.py`：本次 stage/检查捕获驱动源代码，调用冻结 capture.py.execute；不是 Flow 或 Store 解释器。

## 覆盖范围与剩余义务

check-docs 仅核两份文档的相对链接、禁用词和既有决策编号机械检查；diff --cached --check 仅核提交内容的空白错误。CLI 参数、字段、Flow 额度、任务报告路径与源合同/CLI/测试做静态对照，未执行测试场景或文档命令。

必需检查失败：无。额外只读诊断的路径搜索因 study-inputs.json 不存在退出 2，未重跑掩盖；原工具结果在会话记录，stdout/stderr 分离未提供，不编造分离原件。该诊断与必需检查无依赖，不构成 task 检查失败。

`not_run`：Rust 编译/测试、安装、Sheltie、文档演练、独立 review、完整 patch 独立应用、deliver、最终质量和接受。其余未声明工程检查未运行；本实验按冻结 project 仅要求以上两条。不能把退出 0、candidate commit 或报告封存表述成引擎证明内容正确。初始与当前候选完整绑定，后续内容变化须使用新原件验证新候选。
