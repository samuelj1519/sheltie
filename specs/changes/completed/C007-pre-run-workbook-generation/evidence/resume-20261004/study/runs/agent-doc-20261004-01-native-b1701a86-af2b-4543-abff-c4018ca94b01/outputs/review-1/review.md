建议交付：对冻结 candidate 的独立日常静态审查未发现需返工缺陷；完整 patch 的独立应用仍由 deliver 执行，操作演练、最终质量及接受未执行。

## 候选、独立性与证据

本次仅 review-1，审查者 `/root/c007_study_coordinator/run01_review` 未参与 candidate 实现，无额外 helpers。已完整读取 run-binding、task、project、固定 review 方法的共有策略和全部操作、change/checks 报告；按 assigned repo 的 CONTEXT、specs/README、engineering 及共同中文 skill 和受控操作写作参考审阅。历史无 active clone 由外部样本卡明确授权本任务，不采用 proposed。

- 实际仓库：`/private/tmp/sheltie-completion-20261003/c007-agent-paired-study/repos/source-start/native`。
- 初始 HEAD：`351feb7ac22c21317a686693b732d5ae0c4b4bcc`，其 tree 为 `80ea3046b1b3575f07e68313444c051ee7c5b7db`。
- 所审 HEAD：`8366454ddbc2a07d25b2e5e949171cde5b5169ae`，tree：`3b9e14dfb50d3b577cc4caaacdf33e96082026fc`，parent 恰为初始 HEAD；实际工作区 clean。
- 全范围初始 HEAD→HEAD 的差异路径恰为 `README.md` 和 `specs/guides/source-quick-start.md`。前者只在快速开始首部新增一个入口段落，其他历史内容原字节不变；新 guide 已提交，不是被 plain unstaged diff 漏掉的未跟踪文件。全部其他产品、方法、标准、测试和受保护内容无 candidate 差异。

以上直接读取和断言原件在 `raw/review-1/candidate-identity.*`、`candidate-commit.*`、`candidate-tree.*`、`full-patch.*`、`guide-lines.*` 和 `verify-bindings.*`。独立全量 diff 是审查原件，不冒充 deliver 的最终 patch 应用证据。

| 文件 | 当前 SHA256 | bytes |
| --- | --- | --- |
| README.md | 56b6e924f8cdd784a60523d56f583f02fed439e5bf47dd872c3d0fe564dffc89 | 4372 |
| specs/guides/source-quick-start.md | b053c64c3a3076532c767e05f71ce7a12c7e0b097e3c928716afab09ca5937da | 13527 |

## 内容审查

未发现重大或需要返工的具体缺陷；没有为实验制造缺陷或自然 back。逐项依据如下，行号指 candidate guide。

| 冻结验收条件 | 独立核对结果与依据 |
| --- | --- |
| 当前源码、版本及 schema 首次入口 | guide 3–5、46–50 行明确未发布 `0.3.0-rc.1` / schema 4 / cli-result/v4，区分 v0.2.0；与 Cargo.toml workspace version、协议 §1/§7、CLI self version 实际 data 字段一致。README 只新增一个入口。 |
| 实际 Cargo executable、空 wrapper、隔离 target/Home | 12–43 行的 Cargo JSON 选择 compiler-artifact 且 target.name=sheltie、单 executable，核真实文件及执行位；设置空 RUSTC_WRAPPER、独立 CARGO_TARGET_DIR。mktemp 私有新目录内尚不存在的 Home 及 engine 函数使全部文中引擎调用显式携带同一 Home；50 行保留接续路径。没有安装/更新/rollback/发布动作。 |
| 实际 task/project 和完整执行入口 | 54–112 行生成真实 greeting.py、Git 初始候选与普通 UTF-8 task/project，检查与权限明确。97–166 行的 add/verify/show/start/begin/submit/status/result 参数在 CLI command tree 和对应 consumers 存在；task/project 与 Flow default 的 start_inputs 相同。126–160 行按实际 brief、inputs、outputs、AttemptId 委派 implement/review/deliver；previous-review、显式 back、报告独立性和默认无 gate 与真实 Flow/说明书一致。引擎不判断质量。 |
| 当前 next/brief/resume 与停止边界 | 115、119、136、151、153、175–200 行要求当前响应查 next、接续读取 resume，不按历史 next、最新目录或重开本身 fail/begin 推进。协议 §3/§5/§6/§7 支持所述字段、提交封存、EFFECT_PENDING 已提交/未提交区别、旧 schema 整体拒绝和原件保留；retries、visits、replace 额度与 Flow/合同相符。未知、预算、权限、资源、终态都有停止界限。 |
| 中文、链接、权限与 README scope | 正文执行者、对象、先决条件和失败边界明确；真实任务限新练习仓库 greeting.py，合并/部署/发布保持独立授权。所有相对链接目标实际存在并已读取：根 README、specs README、result-export、protocol、code-change README。README 原安装历史及其他文字未改。 |

实际相关来源已读：`examples/code-change/README.md`、manifest、default Flow 和全部三个 instructions；`specs/contracts/protocol.md`；`skills/sheltie/SKILL.md`（仅作为项目来源阅读，没有按该 skill 操作 Sheltie）；`specs/guides/result-export.md`；CLI 的 `cli.rs`、self/work/attempt/workbook 命令与 output 封装。原件为 `raw/review-1/method-source.*`、`protocol.*`、`cli-consumers.*`。Flow deliver 的三个 result 槽及来源、只读 status 的 data.revision/effects_pending/next/resume、self version data.schema_version/home、写响应 data.attempt/brief_path/inputs/outputs 均直接与合同及实际 consumers 对照，未仅凭实现报告宣布合格。

## 必需检查和完整 patch 规则

复用 implement 的两条必需检查，同一输入闭包没有漂移：`scripts/check-docs.sh README.md specs/guides/source-quick-start.md` 与 `git diff --cached --check` 原实际退出码均为 0；两者绑定 staged tree `3b9e14dfb50d3b577cc4caaacdf33e96082026fc`，candidate commit tree 和文件 SHA/bytes 完全相同。全量 diff 证明 checker、合同、方法等消费者未变化；原 cwd、环境、argv、UTC、stdout/stderr/hash、timeout/deadline 可取得并已核原件。独立 `verify-bindings.py` 核了真实 stdout/stderr 的 SHA，前者 stdout 为 `check-docs: OK (2 个文件)`，其 stderr 空；diff 检查两者均空。

原记录在 `raw/implement-1/check-docs.*`、`check-cached-diff.*`、`checks-bindings.json`、`candidate-before-checks.json`、`candidate-after-commit.json`。独立 raw/review-1/verify-bindings.* 实际 exit 0。未重复运行两条机械检查；这些只覆盖文档机械和空白规则，不证明操作演练、内容质量或接受。

change 报告完整保留 project 冻结的 patch 规则：`git diff --binary --full-index 351feb7ac22c21317a686693b732d5ae0c4b4bcc HEAD -- README.md specs/guides/source-quick-start.md`，新文件 stage/commit，先核全量路径集与候选；只在声明的 Native patch-check 副本从完整初始快照按初始 HEAD/tree/clean 检查，然后 `git apply --check`、`git apply --index`、`git write-tree`，要求完整 tree 等于 candidate tree 且两项授权文件字节/SHA 相同。不能对 Root、另一 arm 或已有非初始副本覆盖重建。该声明符合冻结 project；review 没有执行应用。

## 交付前限制、失败和未核项

deliver 必须使用同一候选，实际保存完整 patch 和逐命令原件，完成独立同基线应用、全 tree/文件字节一致核对；失败、超限、deadline 或候选漂移即停止。patch 上限 8388608 bytes，报告各 262144 bytes，run deadline 固定 `2026-10-03T20:10:26.252142+00:00`，不得重置。该 review 结论只建议进入既定 deliver，不自行推进。

`not_run`：guide 命令实际演练、Cargo/Rust 构建测试、Sheltie 调用、安装、完整 patch 独立应用、deliver、最终独立质量盲审、代理/人类接受、发布及其他未声明工程门禁。Native 约束不执行 Sheltie；本阶段静态文档检查不能替代上述证据。Work 成功、提交或检查退出 0 均不能作最终质量/接受结论。

一次额外只读 source-map 诊断使用不存在的 `crates/sheltie-cli/src/args.rs`，rg 返回 2；真实工具输出 `rg: crates/sheltie-cli/src/args.rs: No such file or directory (os error 2)` 已保留在会话工具记录 chunk ebdd93，另见 raw/review-1/diagnostic-observation.json。它不是必需检查，未改变候选；随后通过实际 rg --files 定位 cli.rs 并读其原字节，不冒充原命令成功，不编造原输出的独立 stdout/stderr。其他有 capture 原件的审查读取/断言均 exit 0，无 timeout。

## 身份与时间

实际审查者 `/root/c007_study_coordinator/run01_review`；模型/effort 按父任务及 project 声明继承 `gpt-6.1-sol/high`，未 override；可独立取得的 session 级模型 metadata 为 unknown，没有观察到声明不一致。usage/fees 为 unknown/null，不从流程统计推算。首次可观测 UTC `2026-10-03T19:57:22+00:00`，精确任务收到起始时间 unknown；工具实际 UTC 见 capture 原件，完整结束 UTC：2026-10-03T20:00:06.982738+00:00。actor.json 保存身份、输入哈希、工具证据列表与未知费用。只写本 run outputs/review-1/review.md 和 raw/review-1；未改 candidate、Root、另 arm、既有报告、method/standards 或 stage/run metadata。
