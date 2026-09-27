# C002-T08 证据

- Owner：Claude（glm 会话）。基准 `bcfa0e6`，待提交树见提交说明。
- 输入闭包：`Cargo.lock` 未变；特性全开；平台 macOS aarch64、`rustc 1.98.1`。
- 改动：`store/commit.rs` 新增 `workbook_in_use_check`——remove 的引用检查（逐行解码校验后查非终态引用，损坏行 STORE_CORRUPT）与删行、审计、请求登记同一事务；`effects.rs` 的 `delete_dir` 核对删除对象的登记摘要（不符即别人的新生命周期对象，不删、视为完成）并写 `pending/<id>.deleted` 持久标记（§3.3）；`store/mod.rs` 识别连接与 `connect` 的 busy_timeout 先于任何会取写锁的 PRAGMA，识别连接不再用 `READ_ONLY` 旗标（WAL 库只读连接在写者活动时直接失败）；删除锁外预检的 `works_referencing`。测试新增 `tests/workbook_txn.rs`（8 例）。

## 正反例

| 例 | 独立期望 | 结果 |
| --- | --- | --- |
| 有引用时 remove（反） | `WORKBOOK_IN_USE`，行回滚未删，Work 仍可推进 | PASS |
| 损坏引用行（反，N03） | 直写坏 `state_json` 后 remove 报 `STORE_CORRUPT`，行保留 | PASS |
| 并行 add 不删对方 staging（反，N02） | 串行建库后两线程 add 不同 Workbook，两行两目录都在 | PASS |
| add 发布窗口（正，N02） | 行已提交、目录撤回 pending、`published=0` → 下一个写操作恢复发布、行置 1、load 核摘要可用 | PASS |
| 旧 remove 不删新对象（反，§5.2 第 4 条） | remove → 同版本重新 add → 重放旧 remove/旧 add 均只回快照，新目录仍在、行数 1 | PASS |
| 删除完成标记（正，§3.3） | remove 后恰一个 `pending/<id>.deleted`，内容含 `delete-complete/v1` | PASS |
| 请求全局去重（反） | 同 request-id 先 add 后 start → `REQUEST_CONFLICT`，无 Work | PASS |
| add/remove 重放快照（正，O08） | data/request_id 逐字段等于首次 | PASS |

实现期间并行测试暴露一次真实缺陷：识别连接用 `READ_ONLY` 旗标在 WAL 写者活动时直接 `SQLITE_BUSY`/`database is locked`，且 `busy_timeout` 晚于 `journal_mode` PRAGMA 设置——已改为识别连接普通打开（零写入操作）并先设等待，复跑 5 次并行用例稳定通过。

## 门禁

| 命令 | 退出码 | 原始输出 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 0 | [fmt.txt](fmt.txt) |
| `cargo check --all-targets --all-features` | 0 | [check.txt](check.txt) |
| `cargo clippy --all-targets --all-features -- -D warnings` | 0 | [clippy.txt](clippy.txt) |
| `cargo nextest run --all-features --no-tests=pass` | 0；405 passed | [nextest.txt](nextest.txt) |
| `scripts/check-docs.sh`、`scripts/check-specs.sh`、`git diff --check` | 0 | 提交前复跑 |

## 覆盖与边界

- 关闭：N02（发布失败残留行的恢复、每次 add 只操作自己的 pending、并发互不干扰）、O08 的产品闭环（add/remove 重放语义）、remove 引用检查同事务与损坏行停止（GF-16）。
- 并发 add 的**杀进程**注入（kill 于复制/事务/rename 各窗口）未做动态实验；发布窗口的静态恢复路径已由「撤回目录 + 置 published=0」的等价注入覆盖，`not_run` 如实记录，M1 突变补。
- 独立 review：Owner 按任务卡自查；独立 Reviewer 审查由 M1 承担。
