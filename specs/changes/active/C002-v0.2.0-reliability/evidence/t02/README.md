# C002-T02 证据

- Owner：Claude（glm 会话）。基准 `63d4d48`，待提交树见提交说明。
- 输入闭包：`Cargo.lock` 未变；特性全开（`--all-features`）；平台 macOS aarch64、`rustc 1.98.1`。工作区起点干净（`git status --short` 为空）。全部测试使用各测试自建的临时管理根，不碰真实 `~/.sheltie`。
- 改动：`crates/sheltie-core/src/work/start.rs`（新）、`work/mod.rs`、`work/decide.rs`、`crates/sheltie-runtime/src/service.rs`、`crates/sheltie-cli/src/commands/{work,workbook}.rs`；测试 `crates/sheltie-core/src/work/start.rs` 内嵌、`crates/sheltie-runtime/tests/start_preflight.rs`（新）、`crates/sheltie-cli/tests/{work,workbook}.rs` 追加。

## 正反例

| 例 | 命令/入口 | 独立期望 | 结果 |
| --- | --- | --- | --- |
| show 有序键（正） | `workbook show article-review` | `flows[0].start_inputs == ["topic"]`（来自合同与手读样例 Flow）；文本含「起始输入: topic」 | PASS |
| show 无键（反例的正侧） | `workbook show nokeys`（临时无键 Workbook） | `start_inputs == []`、文本「起始输入: 无」 | PASS |
| 缺 topic（反） | `work start --workbook two-step --flow default` | `INPUT_MISSING`、detail.missing=["topic"]；home 树逐路径相同；随后成功 start 序号 `-001-` | PASS |
| 多 key（反） | 同上加 `--input topic=x --input bonus=y` | `INPUT_MISSING`、detail.extra=["bonus"]；home 不变；成功后 `-001-` | PASS |
| 非法名字（反） | 同上加 `--name a/b` | `INVALID_REQUEST`；home 不变；成功后 `-001-` | PASS |
| 缺 workbook（反） | `--workbook ghost` | `NOT_FOUND`；home 不变 | PASS |
| 缺 flow（反） | `--flow ghost` | `NOT_FOUND`；home 不变 | PASS |
| 新 home 失败 start（反） | 未 add 的空管理根上 start | `NOT_FOUND`；管理根零条目（不建 store.db）；补 add 后同参数成功 `-001-` | PASS |
| 拒绝后补条件同 request-id 重试（正） | runtime 级 `start_preflight::missing_start_input_rejected_before_seq_and_materialization` | 失败后 `works/work_sequence/requests` 行数不变（直连 SQLite 独立读数），同 request-id 补 topic 后成功 | PASS |
| 参数错误 exit 2（反） | `--input noequal`、`--input topic=@/definitely/not/here.txt`、缺 `--workbook` | 退出码 2；前两个输出 `INVALID_REQUEST` JSON；clap 缺参走 clap 自身 exit 2 | PASS |
| 同请求重放不烧号 | 既有 `start_replay_returns_same_work_id_without_new_seq`（回归） | 重放返回原 work_id | PASS |

home 相同的 oracle：CLI 级用递归路径快照（跳过 SQLite 连接关闭时回收的 `-wal/-shm`）；runtime 级用只读 rusqlite 直查三表行数。期望不来自被测代码。

## 门禁

| 命令 | 退出码 | 原始输出 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 0 | [fmt.txt](fmt.txt) |
| `cargo check --all-targets --all-features` | 0 | [check.txt](check.txt) |
| `cargo clippy --all-targets --all-features -- -D warnings` | 0 | [clippy.txt](clippy.txt) |
| `cargo nextest run --all-features --no-tests=pass` | 0；338 passed（1 leaky，为既有 selfmgmt 测试的进程残留，与本任务无关） | [nextest.txt](nextest.txt) |
| `scripts/check-docs.sh` | 0 | 复跑于提交前，无文档改动 |
| `scripts/check-specs.sh` | 0 | 同上 |
| `git diff --check` | 0 | 终端记录 |
| `scripts/check-task.sh C002-T02 --staged` | 见提交前运行 | 见 validation.md |

## 覆盖与边界

- 关闭：N01（start 副作用顺序——确定性拒绝全部移到序号分配与物化之前；新 home 失败 start 不建库）、GF-30 的 `start_requirements`/`validate_start_inputs` 共享与 `workbook show` 暴露。
- 不含：request-id 意图指纹与目标绑定（O02/O04 归 T07）；`start-inputs/` 目录与 `pending/` staging 布局（归 T03/T07，当前仍写 `inputs/`）；预检并发竞争（T07 写锁）。O12 的输出命名空间隔离已由 T14 前的合同修订覆盖路径规则，本任务未动。
- 独立 review：Owner 按任务卡自查（上表逐项）；未参与实施的 Reviewer 审查按 plan 由 M1 全链 review 承担，未执行前不在此记 PASS。
