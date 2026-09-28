# C002-T12 证据

- Owner：Claude（mimo 会话）。基准 `b3f138bdb81b415e2bf3d2970f34ecf32673f1a8`（T11 提交），待提交树见提交说明。工作区起点只有 T11 之后的未跟踪测试目录之外无无关改动（`git status --short` 逐项都落在 T12 白名单内）。
- 输入闭包：`Cargo.lock` 未变（不跑 deny）；特性全开（`--all-features`）；平台 macOS aarch64、`rustc 1.98.1`。全部场景用测试自建的临时管理根与临时项目 Git 仓库，不碰真实 `~/.sheltie`，也不在 sheltie 仓库里做 Git 操作。
- 改动：`workbooks/spec-dev/`（`flows/default.toml` 补 decision/spec/escalation 输入与 escalate→verify 回程边，instructions 九份、templates/plan.md、两份 checklist、workbook.toml 升 0.2.0、README 修订记录）；`crates/sheltie-core/tests/examples.rs` 三例静态图测试；`crates/sheltie-cli/tests/scenario_spec_dev.rs` 十例真实 CLI 场景（新文件）；`specs/contracts/workbook.md` §6 spec-dev 边数 24→25 事实同步；`specs/releases/v0.1.0/plan.md` T11 卡旧测试名改原位注记（与 C002-T03 同一处置）；`tasks.toml` 白名单与注记。

## 卡片第 1、2 条落在哪

约定都在 Workbook 层（说明书、模板、清单、Flow 输入绑定），引擎不记 Spec/Plan/Git 状态：

| 约定 | 落点 | 由谁证明 |
| --- | --- | --- |
| 整体原始基线只记一次，改方案原样抄、不重设 | templates/plan.md「基线」行；plan.md 说明书 | G5：`plan_baseline` 解析第 2 版方案，仍等于 Work 开始时的 HEAD |
| 每任务基线/提交，verify 只核当前任务改动（含修复轮次） | implement/fix/verify 说明书与 change.md 模板 | G1：T01、T02 的范围断言各自只含自己的文件；修复轮次并进同一任务范围 |
| 合法的未来任务占位留存不算残留，无编号占位不算合法 | verify 说明书第 6 步、scaffold/implementer 清单 | G2 正例：T01 填完、T02 占位原样留存仍通过；G2 两个反例：本任务占位残留恰一行判「不通过」，无编号占位（`unmarked_placeholder_is_flagged`）同样判「不通过」 |
| plan-review 条件绑到获批的 plan/spec 版本，随 Flow 输入交给 scaffold/implement/verify | plan-review 说明书的 decision 模板（`批准的规格:`、`批准的方案:` 两行摘要）；三节点的 decision 必需输入 | G3 正例逐节点核绑定路径后缀与 `shasum -a 256` 摘要；G3 反例旧摘要对不上即卡住 |
| 方案修订使旧批准失效，worker 停下升级给人 | scaffold/implement/verify 说明书的核对步骤 | G3 反例：plan#2 承载 plan#1 的摘要 → 卡住、项目零提交、下一步是 escalate |
| verify/scaffold 升级返回的边、输入、说明书三件套 | escalate→verify `kind = back` 边；scaffold/verify 的 escalation 可选输入；两处说明书 | G4a/G4b：`next` 里边类型是 `back`/`branch`，任务书「来自」行与输入表绑定 escalate 的 decision 文件，正文不进任务书 |

## 五组闭环（卡片第 3 条）

每组一个独立临时项目 Git 仓库 + 真实 `sheltie` 二进制（`assert_cmd`，临时管理根），文件集合与路径后缀是手写期望，提交哈希经独立 git CLI 与报告记录互核（不取生产 helper），并核任务书输入绑定。

| 组 | 测试（`// Task: C002-T12`） | 唯一改变的条件 | 独立 oracle | 结果 |
| --- | --- | --- | --- | --- |
| 1 两任务改不同文件 | `two_task_loops_scope_each_task_diff_to_its_own_files` | 无（T01 干净 + T02 带一轮修复） | `git diff <本任务基线>..<本任务提交> --name-only` 逐任务等于手写两文件集；对照组把范围换成骨架提交..修复提交（O09 失败形态）断言会含别人的文件；brief「来自: scaffold#1（main 边）」「来自: verify#2（branch 边）」「来自: fix#1（re_review 边）」，change（verify 的输入）与 report（implement#2 的输入）绑定路径后缀 `.../implement/occurrence-001/attempt-000/outputs/change.md`、`.../verify/occurrence-001/attempt-000/outputs/report.md` | PASS |
| 1 反例 | `out_of_whitelist_file_in_second_task_is_flagged` | 只多改一个白名单外文件（T01 的 `src/export.py`） | verify 报告第一行 `不通过`（读回输出文件），「发现」点名 `改了不该改的文件：src/export.py` | PASS |
| 2 共用文件留未来占位 | `shared_file_keeps_future_task_placeholders` | 无（共享 `src/feature.py`，两个测试文件分开） | T01 只填自己的占位体，`# T02` 占位按字节留存；本任务占位残留清单为空；brief 输入绑定同组核对（decision 后缀 `.../plan-review/occurrence-001/.../outputs/decision.md` 及表行、change 后缀及表行） | PASS |
| 2 反例 | `own_task_placeholder_left_behind_is_flagged` | 只把填充目标换成别人的占位体（自己的没填） | 本任务残留恰一行 `raise NotImplementedError  # T01`，报告第一行 `不通过`（读回输出文件） | PASS |
| 2 反例（无编号占位） | `unmarked_placeholder_is_flagged` | 只去掉残留占位体的任务编号 | 本任务占位清单为空、无编号占位恰一行 `raise NotImplementedError`，报告第一行 `不通过`、「发现」写「占位体没标任务编号」 | PASS |
| 3 附条件批准进三节点 | `conditional_approval_binds_decision_into_scaffold_implement_verify` | 无（批准带条件「验收时要跑门禁」） | 三个节点的 decision 路径后缀都是 `/attempts/plan-review/occurrence-001/attempt-000/outputs/decision.md`，plan/spec 后缀各自 `/attempts/plan|spec/occurrence-001/...`；`shasum -a 256` 摘要逐字等于绑定文件实测值；条件既在 decision 也在 verify 报告的验收行 | PASS |
| 3 反例 | `stale_approval_hash_makes_worker_stop` | 只让 plan#2 的批准记录载 plan#1 的摘要 | 摘要比对 Err（返回文件侧真实摘要）；卡住记录点名两个摘要与绑定文件摘要；`proj.head()` 不变（零骨架提交）；下一步含 escalate | PASS |
| 4 verify 升级后继续 | `verify_escalation_continue_binds_opinion_on_return` | 无（验证要授权，报告「不通过，需要人」） | `next` 中 verify 的边类型 `back`；verify#2 brief「来自: escalate#1（back 边）」；escalation 绑定 `/attempts/escalate/occurrence-001/attempt-000/outputs/decision.md` 且等于提交字节；意见正文不进任务书 | PASS |
| 4 scaffold 升级后继续 | `scaffold_escalation_continue_binds_opinion_on_return` | 无（方案与现有接口冲突，卡住） | 卡住时 `proj.head()` 仍等于基线；进 escalate 的边 `branch`、返回 scaffold 的边 `back`；scaffold#2 brief「来自: escalate#1（back 边）」+ escalation 绑定同上 | PASS |
| 5 改方案后最终审查 | `replan_after_first_task_keeps_original_baseline_for_final_review` | 无（任务 1 完成 → 任务 2 卡住 → 人改方案） | plan#2 的「基线」行解析值 == Work 开始时 HEAD == 根提交；review 绑 plan#2（`/attempts/plan/occurrence-002/...`）；整体范围 `原始基线..HEAD` 等于手写四文件集且含任务 1 的 `src/export.py`；反例范围（改方案时点..HEAD）等于手写两文件集，缺 `src/export.py` | PASS |

静态图由 `crates/sheltie-core/tests/examples.rs` 三例补证：`spec_dev_compiles_with_eleven_nodes_twenty_five_edges`（11 节点 25 边）、`spec_dev_binds_decision_into_scaffold_implement_verify`、`spec_dev_escalation_inputs_cover_return_edge_to_verify`（escalate→verify 为 `Back`）。

首次提交尝试在仓库 pre-commit 钩子里跑了全量 nextest，暴露出一处夹具不隔离：钩子进程给子孙留下 `GIT_DIR`/`GIT_INDEX_FILE` 之类的环境变量后，夹具里的 `git init/add/commit` 不再认 cwd 的临时仓库，把临时项目暂存到了宿主仓库的暂存区（`git status` 出现整仓暂存删除）。处置：`git reset` 恢复暂存区（仅索引级；工作树逐文件核对完好、stash 为空、HEAD 未动，未丢任何东西）；夹具 `Proj::git` 摘掉整组 git 环境变量并把 `core.hooksPath` 指到空目录，宿主钩子与宿主仓库都进不来。修复后故意带 `GIT_DIR` 跑同一用例 PASS，断言语义未动。这次失败没有产生提交。

实现与验证期间的几处处置记在这里，不改期望迎合实现：`specs/contracts/workbook.md` §6 的边数随图改动 24→25（事实同步）；legacy 归档计划里 T11 卡的旧测试名按 C002-T03 先例改原位注记（check-tests 的括号剥离规则下合法），不改归档计划其他内容；`specs/releases/v0.1.0/decisions.md` 里「十一节点二十四边」是 D-24 当时的后果陈述（v0.1.0 属实），不追改，现行计数在 workbook.md §6。check-task 规则 2 会 grep `todo!()` 字面量，测试源里用 `concat!("todo", "!()")` 与 Python 风格夹具标记（`raise NotImplementedError  # Tnn`）避免假红。

## 门禁

| 命令 | 退出码 | 原始输出 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 0 | [fmt.txt](fmt.txt) |
| `cargo check --all-targets --all-features` | 0 | [check.txt](check.txt) |
| `cargo clippy --all-targets --all-features -- -D warnings` | 0 | [clippy.txt](clippy.txt) |
| `cargo nextest run --all-features --no-tests=pass` | 0；425 passed、0 skipped；原始输出标 1 项 leaky（子进程线程存活计时，复跑在 0–2 项间漂移，均无失败） | [nextest.txt](nextest.txt) |
| `scripts/check-docs.sh` | 0 | [docs.txt](docs.txt) |
| `scripts/check-specs.sh` | 0 | [specs.txt](specs.txt) |
| `scripts/check-tests.sh` | 0；425 个测试、任务卡 226 条 | [tests.txt](tests.txt) |
| `scripts/check-core-vocab.sh` | 0 | [core-vocab.txt](core-vocab.txt) |
| `git diff --check` | 0 | [diff-check.txt](diff-check.txt) |
| `scripts/check-task.sh C002-T12 --staged` | 0；输出 `check-task: C002-T12 OK` | [check-task.txt](check-task.txt) |

## 独立审查

Reviewer：未参与本任务修改的通用 agent（独立会话，只审不改）。两轮结论都是「需修改」，逐条处置：

| 轮次 | 问题 | 处置 |
| --- | --- | --- |
| 首轮必改 1 | G2 两例没有任务书输入绑定断言（其余四组都有，卡片要求每组） | 正例补 decision/change 路径后缀与表行断言，反例补 decision 后缀 |
| 首轮必改 2 | 证据表 G1 行写「change/report 绑定」，测试只断了 change | G1 的 implement#2 补 report 输入后缀与表行断言，两处落证 |
| 首轮必改 3 | 证据表第 3 行占位反例指代错指 G1；「无编号占位不算合法」无测试落证 | 指代改指 G2；新增 `unmarked_placeholder_is_flagged`（单条件：残留占位体去掉编号，`own_left` 空、`unmarked` 恰一行、报告不通过） |
| 二轮必改 | 证据「改动」摘要仍写「九例」（补反例后是十例） | 改「十例」 |
| 可选 1/2/3/5/6 | 哈希「手写期望」措辞过头；G2n 发现未读回；plan-review 批准记录名实不符；verify 升级返回读哪份记录不明确；escalate→implement 边 kind 遗留 | 逐条采纳（措辞精确化、G2n 读回第一行、plan-review 改「版本记录/四种决定都写」、verify 按 fix_change 绑定判定、README 边界段注记）；二轮顺手项 G2u 发现读回断言一并采纳 |
| 可选 4 | v0.1.0 decisions.md「十一节点二十四边」 | 不采纳：那是 D-24 当时的后果陈述，v0.1.0 属实，追改会 falsify 历史；现行计数在 workbook.md §6（处置段已记） |

二轮放行条件只有证据计数一处（已改），并确认三处首轮必改真实闭合、新反例单条件且非同义反复（`line_task` 的无编号分支首次被走到）、两处说明书措辞与流程其余部分自洽。

## 覆盖与边界

- 关闭：卡片第 1、2 条的 Workbook 约定 + 第 3 条五组闭环（每组独立临时 Git、真实 CLI、手写文件集合/路径后缀 oracle、任务书输入绑定核对）。
- 不含：真实 agent 是否照说明书交付（写文档的质量、是否真读绑定路径）归 T16；引擎侧没有任何 Spec/Plan/Git 状态改动（这张证据表里的核对逻辑都在测试与说明书里）。escalate→implement 的边 kind 仍是 `main`（与「跳过」共用一条边），implement 卡住「继续」回来时任务书显示「main 边」；旧设计遗留，本任务未动（回程边补的是 escalate→scaffold 与 escalate→verify）。
