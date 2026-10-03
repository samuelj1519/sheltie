可审查：已在 Native 独立副本完成两份授权文档和本地候选；两条声明检查退出 0，尚待独立 review 与 deliver。

## 任务与实际变更

目标是补齐当前源码 `0.3.0-rc.1` / schema 4 的首次使用入口，保留 v0.2.0 远端安装历史。初始 guide 不存在；现在 README 只增加一个入口段落，新 guide 给出当前源码构建、Store 隔离、真实输入和完整 code-change 操作路径。

- `README.md`：在快速开始首部添加一个指向新 guide 的当前源码入口；原有远端安装和其他内容未修改。
- `specs/guides/source-quick-start.md`：从 Cargo JSON 的 `compiler-artifact.executable` 定位二进制，空 `RUSTC_WRAPPER` 和独立 `CARGO_TARGET_DIR`；用 mktemp 私有目录中的新显式 Home，所有命令经显式 binary/Home 函数调用。
- guide 使用新练习仓库的 `greeting.py` 拼写修复，实际生成 task/project，固定允许文件、初始 HEAD 和两条验收命令；给 add/verify/show/start/begin、委派工作 agent、按声明路径写 change/review/delivery、submit、显式返工、status/resume 和 result 全入口。
- guide 保留默认无 gate、协调者内容判断、冻结输入、旧 schema 整体拒绝原件保留、EFFECT_PENDING 已提交/未提交区别、当前查询优先、次数/预算/未知/终态停止边界。它不授予安装、合并或发布权限。

依据已完整读取 task/project/confirmed-input implement.md，以及 assigned repo 的 AGENTS、CONTEXT、specs/README、engineering 和 project 全部 source materials；另静态核 CLI 命令树、self version 实际字段、code-change 指令/Flow 和已有场景测试。应用共享 tech-doc-style-chinese 及受控中文操作写作参考。未选择 proposed；授权来自外部 C007 本样本卡。

## 候选与范围

- 仓库：`/private/tmp/sheltie-completion-20261003/c007-agent-paired-study/repos/source-start/native`。
- 初始 HEAD：`351feb7ac22c21317a686693b732d5ae0c4b4bcc`；初始 tree：`80ea3046b1b3575f07e68313444c051ee7c5b7db`，本 worker 核 clean；协调者 preflight 核 192 source inputs matched。
- 当前 HEAD：`8366454ddbc2a07d25b2e5e949171cde5b5169ae`；tree：`3b9e14dfb50d3b577cc4caaacdf33e96082026fc`；parent 恰为初始 HEAD。工作区 clean，完整改变路径集合恰为两项 allow_files。
- 两条检查绑定的 staged tree 与 commit tree 一致，提交后两文件 SHA/bytes 未漂移；原件见 raw/implement-1 的 candidate-before-checks.json、checks-bindings.json、candidate-after-commit.json。
- 本地提交：`docs(study): 补齐当前源码快速开始入口`，按 project 使用 hooksPath=/dev/null、commit.gpgsign=false 和 Change/Task/Sample/Arm/真实 Agent trailers；不含 Work trailer，Native 无 Sheltie 调用。

| 文件 | SHA256 | bytes |
| --- | --- | --- |
| README.md | 56b6e924f8cdd784a60523d56f583f02fed439e5bf47dd872c3d0fe564dffc89 | 4372 |
| specs/guides/source-quick-start.md | b053c64c3a3076532c767e05f71ce7a12c7e0b097e3c928716afab09ca5937da | 13527 |

## 使用、返工与限制

首次 implement，无 previous-review 绑定。使用者从 README 新入口阅读 guide，在自己的 Bash 会话中按段操作，领取任务书后等待工作 agent。本文只静态写作；未实际编译、安装、执行 Sheltie 或演练文档，无 Rust 测试。没有修改 Root 源仓库、另一 arm、方法、标准、stage.json 或 run.json。

独立 review、deliver、完整 patch 生成/独立应用、最终质量、代理接受、人类接受均 `not_run`，由后续真实参与者完成。机械检查只核链接/禁用词和暂存 diff 格式，不证明操作成功或质量合格。额外只读路径查找尝试包含不存在的 study-inputs.json，rg 退出 2；该诊断未改变输入或候选，不算必需检查失败，原始工具输出保留在会话工具记录。

## 供 deliver 使用的冻结 patch 规则

以下是 project 预冻操作，当前 implement 未执行，不能记成 patch 应用通过。patch 最大 8388608 bytes，报告各最大 262144 bytes；超限或 deadline 到达停止，不截断。

在授权 candidate repo 中生成完整 patch，包含已提交的新 guide，禁止使用 plain unstaged diff：

```bash
git diff --binary --full-index 351feb7ac22c21317a686693b732d5ae0c4b4bcc HEAD -- README.md specs/guides/source-quick-start.md
```

将 stdout 原字节保存到本 run 的 deliver 原件。先核 HEAD 仍为 `8366454ddbc2a07d25b2e5e949171cde5b5169ae`，完整初始 HEAD→HEAD 改变路径集合恰等于 `README.md`、`specs/guides/source-quick-start.md`，无未跟踪必需文件；这两项已经 stage/commit，不能漏新文件。

唯一授权独立检查副本：`/private/tmp/sheltie-completion-20261003/c007-agent-paired-study/patch-check/source-start/native`。从原 archive/完整初始快照建立或核对该副本，禁止操作 Root 或另一 arm。如果副本尚不存在，可从 candidate repo 读取完整初始 Git 快照建立：

```bash
git clone --no-hardlinks --no-checkout /private/tmp/sheltie-completion-20261003/c007-agent-paired-study/repos/source-start/native /private/tmp/sheltie-completion-20261003/c007-agent-paired-study/patch-check/source-start/native
git -C /private/tmp/sheltie-completion-20261003/c007-agent-paired-study/patch-check/source-start/native -c core.hooksPath=/dev/null checkout --detach 351feb7ac22c21317a686693b732d5ae0c4b4bcc
```

副本已存在时不覆盖或重建；先核 `git rev-parse HEAD`、`HEAD^{tree}` 恰为初始身份且 `git status --porcelain` 为空。冻结应用命令在该副本 cwd 执行，`patch_file` 为实际完整 patch 绝对路径：

```bash
git apply --check "$patch_file"
git apply --index "$patch_file"
git write-tree
```

最后 `git write-tree` 必须等于 candidate `HEAD^{tree}` 即 `3b9e14dfb50d3b577cc4caaacdf33e96082026fc`，并逐项核两授权文件字节/SHA 与 candidate 相同；任一失败保留原件停止，不反复凑绿。每个实际 argv/cwd/env/stdout/stderr/exit 用 capture.py.execute 保存，单 check 上限 120s、总 deadline `2026-10-03T20:10:26.252142+00:00` 不重置。完整 patch 与 source/opposite arm 保持只读。

## 执行身份与时间

真实 worker：`/root/c007_study_coordinator/run01_implement`；模型/effort 按继承声明 `gpt-6.1-sol/high`，无 override、无额外 helpers。worker 无可取得的 turn_context metadata 引用，实际 session 级独立证明为 unknown；没有观察到与声明不同的证据。第一可观测 UTC 19:51:52，完整必需输入读取观测完成 19:52:23；checks/commit 精确 UTC 见 actor.json 与逐命令记录。报告完成 UTC：`2026-10-03T19:56:39.251570+00:00`。精确 turn 收到起始 UTC unknown，使用量和费用 unknown/null，未从 Attempt 统计推算。
