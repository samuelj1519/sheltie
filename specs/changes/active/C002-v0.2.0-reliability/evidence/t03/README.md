# C002-T03 证据

- Owner：Claude（glm 会话）。基准 `df0e120`，待提交树见提交说明。
- 输入闭包：`Cargo.lock` 未变；特性全开；平台 macOS aarch64、`rustc 1.98.1`。工作区起点干净。测试全部使用临时管理根。
- 改动：`crates/sheltie-core/src/work/layout.rs`（新，纯函数 + 独立测试）、`work/mod.rs` 导出、`flow/parse.rs` 输出路径四条规则（删除「不得与 brief.md 相同」）；测试 `flow/parse.rs` 内嵌更新、`crates/sheltie-cli/tests/output_paths.rs`（新）。持久 caller（`WorkState::attempt_dir`、decide/render/service 的路径生成）按 plan 留待 T07 一次切换。

## 正反例

| 例 | 入口 | 独立期望 | 结果 |
| --- | --- | --- | --- |
| draft#2.1 映射稳定（正） | `layout::attempt_dir` 纯测试 | 手写 `/…/attempts/draft/occurrence-002/attempt-001`；首达首执行 `occurrence-001/attempt-000`；两位数 `occurrence-012` | PASS |
| engine 与 outputs 分离（正） | `layout` 纯测试 | 引擎 `brief.md`、`engine/stats.json` 在 Attempt 根；worker `outputs/stats.json`、`outputs/outputs/brief.md` 与它们路径不同；嵌套 `outputs/notes/sub/x.md` 保留子目录 | PASS |
| outputs/brief.md 声明合法并真实提交（正） | 真实 CLI：add → start → begin → 按 brief 写三个输出（`outputs/brief.md`、`stats.json`、`notes/sub/x.md`）→ submit | `work_status=succeeded`、三份输出封存、引擎任务书仍在且与 worker 的 brief 路径不同 | PASS |
| 重复路径（反） | core parse | `out/x.md` 重复 → `nodes[0].outputs[1].path` 拒绝 | PASS |
| 祖先冲突（反） | core parse 与真实 CLI `workbook add` | `out` 与 `out/sub`（双向）拒绝；CLI 报 `FLOW_INVALID`、detail.path 指到 `nodes[0].outputs[…]` | PASS |
| 大小写折叠别名（反） | core parse 与真实 CLI | `OUT.md`/`out.md`、`Out`/`out/x.md` 拒绝；正侧 `out/x.md`+`out/y.md`、`notes/a.md`+`notes/a/b.md` 接受 | PASS |
| 可移植字符集（反） | core parse | 汉字、全角斜杠、空格、`+` 逐项拒绝；段恰好 128 字节接受、129 拒绝 | PASS |
| 既有样例不受影响（回归） | examples 三份与 spec-dev 编译测试 | 347 全部通过 | PASS |

期望值全部手写（架构 §5 布局、workbook.md §3.2 规则），不调用被测函数生成。

## 门禁

| 命令 | 退出码 | 原始输出 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 0 | [fmt.txt](fmt.txt) |
| `cargo check --all-targets --all-features` | 0 | [check.txt](check.txt) |
| `cargo clippy --all-targets --all-features -- -D warnings` | 0 | [clippy.txt](clippy.txt) |
| `cargo nextest run --all-features --no-tests=pass` | 0；347 passed | [nextest.txt](nextest.txt) |
| `scripts/check-docs.sh`、`scripts/check-specs.sh`、`git diff --check` | 0 | 提交前复跑 |

## 覆盖与边界

- 关闭的准备单元：O12 的路径规则面（输出命名空间隔离、祖先与 ASCII 折叠别名、可移植字符集）与单一 `WorkLayout` 目标形状。产品持久路径仍走旧布局（`attempts/<node>/<n>/<retry>`、输出在 Attempt 根、engine stats 在 `<attempt>/stats.json`），切换与新 Store 一次完成，归 T07——`outputs/` 物理前缀、`engine/stats.json`、`start-inputs/` 的真实 CLI 闭环与「空执行不得把 engine.stats 当产出」在 T07 验证。纯路径测试不足以关闭 O12，本任务不记 O12 关闭。
- 未引入 LegacyV1、默认布局推断或第二套持久格式；`WorkLayout` 只服务本次已采用的格式切换。
- 独立 review：Owner 按任务卡自查；独立 Reviewer 审查由 M1 承担。
