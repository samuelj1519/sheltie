# C002-T11 证据

- Owner：Claude（mimo 会话）。基准 `a5b2a05fe26c32a462d1e9aa24848f9c0f65c0f5`，待提交树见提交说明。工作区起点干净（`git status --short` 为空）。
- 输入闭包：`Cargo.lock` 未变；特性全开（`--all-features`）；平台 macOS aarch64、`rustc 1.98.1`。全部测试使用各测试自建的临时管理根，不碰真实 `~/.sheltie`。
- 改动：`examples/article-review/{flows/default.toml,instructions/draft.md}`——draft 增加 `required = false` 的 `review.verdict` 输入，说明书改为按绑定路径读最近一次成功审查的意见文档；`specs/contracts/workbook.md` §3 同步同一张图、§6 样例行补可选输入回环；`specs/contracts/protocol.md` attempt begin 返回补「未绑定的可选输入值是 `null`」；`crates/sheltie-core/src/flow/compile.rs` 一条夹具按样例字节同步替换目标。测试 `crates/sheltie-core/tests/examples.rs` 与 `crates/sheltie-cli/tests/scenario_article_review.rs` 各增若干（`// Task: C002-T11`）。

## 正反例

真实 CLI（`assert_cmd` 起 `sheltie` 二进制、独立临时管理根）走首次 draft→review→back→第二次 draft；期望路径、任务书表行与「尚无」文案按合同 §4 模板与手写字符串，不从引擎布局 helper 生成。

| 例 | 唯一改变的条件 | 独立期望 | 结果 |
| --- | --- | --- | --- |
| 首次 draft 标尚无（正） | 无（首次到达） | `inputs.review == null`；brief 输入表 `\| review \| 尚无（上游 review 还没有产出） \| \|`，「来自: 入口」；说明书含「首次开工它标『尚无』」；全文不含 `attempts/review/` | PASS |
| back 后第二次 draft 绑定（正） | 从入口变为 back 边 | `inputs.review` 以 `/attempts/review/occurrence-001/attempt-000/outputs/review.md` 结尾；brief「来自: review#1（back 边）」且输入表给出同一路径；路径处文件字节等于提交时写入的 `output for review#1.0\n`；brief 不含该正文，也不含 review 摘要正文（只传路径） | PASS |
| 第三轮绑定最近成功 review（反，单条件=轮次） | 只多跑一轮 review | draft#3.0 绑 `.../review/occurrence-002/.../outputs/review.md`，brief「来自: review#2（back 边）」，文件字节 `output for review#2.0\n`——不是第一轮的产物 | PASS |
| `required = false` 是首稿能开工的条件（反，单条件） | 样例副本只去掉 `required = false` | 首次 `attempt begin draft` 退出码 1、`INPUT_UNAVAILABLE`、detail `input=review`、`node=review` | PASS |
| 静态图（正） | 无 | 编译后的 draft 输入 `review` 来源 `review.verdict` 且 `required() == false` | PASS |
| 回归 | 无 | 样例编译、back 回环、`max_visits` 耗尽、human 提交、下游绑定最新成功产物、checklist 资源路径等既有 6 例全部 PASS | PASS |

「聊天补传意见」路径没有接口可走：意见只以绑定文件路径进入任务书，说明书与任务书均不内联历史正文；若只能靠聊天补传，第二例的路径与来源 Occurrence 断言会失败。

## 门禁

| 命令 | 退出码 | 原始输出 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 0 | [fmt.txt](fmt.txt) |
| `cargo check --all-targets --all-features` | 0 | [check.txt](check.txt) |
| `cargo clippy --all-targets --all-features -- -D warnings` | 0 | [clippy.txt](clippy.txt) |
| `cargo nextest run --all-features --no-tests=pass` | 0；413 passed、0 skipped | [nextest.txt](nextest.txt) |
| `scripts/check-docs.sh` | 0 | [docs.txt](docs.txt) |
| `scripts/check-specs.sh` | 0 | [specs.txt](specs.txt) |
| `scripts/check-tests.sh` | 0 | [tests.txt](tests.txt) |
| `scripts/check-core-vocab.sh` | 0 | [core-vocab.txt](core-vocab.txt) |
| `git diff --check` | 0 | 终端记录 |
| `scripts/check-task.sh C002-T11 --staged` | 0 | 提交前在最终候选树实跑，输出 `check-task: C002-T11 OK` |

实现期间 full-suite 首跑发现 `flow::compile::tests::rejects_input_from_node_that_cannot_reach_consumer` 的字符串夹具匹配不到新字节而假红（替换落空后 `unwrap_err` panic）；处置是按样例字节同步替换目标，断言语义不变（tasks.toml 已注记），不是改期望迎合实现。

## 独立审查

Reviewer：未参与本任务修改的通用 agent（独立会话）。首轮结论「需修改」，两条必改：

| 问题 | 处置 |
| --- | --- |
| `inputs.review` 只用 `is_null` 断言：serde_json 的 Index 对缺失键同样返回 Null，合同新句「占一行、值是 `null`」的 key 存在性没被固定 | 改为 `as_object().unwrap().get("review") == Some(&Value::Null)`；复跑该例 PASS |
| 证据把 `scripts/check-task.sh C002-T11 --staged` 预填退出码 0，当时并未实跑 | 删除预填；plan 状态翻 `done` 后实跑，门禁表写真实退出码 |

可选建议采纳两条：测试注释引用改精确到「attempt begin 的返回说明」；协议 §7 `INPUT_UNAVAILABLE` 行补 `detail.input` 与 `detail.node`，测试断言有合同依据。未采纳一条：反例 detail 两键都叫 `review`（输入名与来源节点同名）鉴别力有限——样例字节固定该命名，换名会改产品示例，不为测试方便改。

## 覆盖与边界

- 关闭：N09 的 Flow 与说明书静态面 + 绑定上一轮文档后的「第一次尚无、第二次正确来源」验证。打回意见从任务书绑定的文件路径进入，不再靠聊天补传。
- 不含：真实宿主交互（协调者是否真的读路径、不再追问）归 T16；`workbooks/spec-dev` 的可选输入行为未改动（既有 core 用例覆盖）。
- 独立 review：未参与实施的 Reviewer 首轮「需修改」，两处修复后复核「通过」（见上节）；M1 的全链独立 review 另做。
