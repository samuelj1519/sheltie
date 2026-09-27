# C002-T06 证据

- Owner：Claude（glm 会话）。基准 `d38b6e1`，待提交树见提交说明。
- 输入闭包：`Cargo.lock` 未变；特性全开；平台 macOS aarch64、`rustc 1.98.1`。
- 改动：core `work/state.rs` 新增 `blocked_count`（serde 必填字段，无默认值——旧 state 缺字段即拒绝，不猜历史）；`work/decide.rs` 三处状态转换递增（gate 提交成功、重试耗尽、`no_legal_edge` 发生，approve 后的 `no_legal_edge` 同计）；`work/render.rs` 新增 `StatusView` 与 `status_view`，文本卡与 JSON 都从同一视图渲染，JSON 补 `blocked`（无前缀说明串）、`last_attempt.reason`、`outputs` 完整 `{path, sha256, bytes}`，`next` 由 core `next_item_json` 装配成协议 §5 唯一形状；stats 的 `entered_via` 保留边类型（文本 `node(edge)×n`，结构化 `entered_via_json` `{from, edge, count}`），`blocked_count` 改读状态字段。CLI `work status/stats` 的封装 `next` 与状态卡同形（`ok_work_next`）。既有期望与快照按合同（协议 §6/§work stats）更新。

## 正反例

| 例 | 入口 | 独立期望 | 结果 |
| --- | --- | --- | --- |
| NoLegalEdge→cancel 计数不减少（反，N07） | 真实 decide 转换后 `render_stats_json` | blocked=1，cancel 后仍 1 | PASS |
| 三类受阻累计与批准保留（正） | 重试耗尽/gate/批准/普通成功 | 1/1/保留 1/0 | PASS |
| 失败原因与完整产物引用（正，O13） | `status_card_json` | `reason` 有值；outputs 的 path/sha256(64 位)/bytes 逐一等于提交记录；字段被删即断言失败 | PASS |
| blocked 说明串两格式同源（正） | gated 卡 | JSON `blocked` = `gate: notes#1 需要 gate approve`（无前缀）；文本行带 `blocked: ` 前缀 | PASS |
| next 唯一形状（正，O13） | review 完成后的卡 | 项为 `{op, args{work…}, edge, executor, tier}`；begin 项三附加键齐全 | PASS |
| 相同来源不同边可区分（反，N07） | back 后再进 draft | `entry×1, review(back)×1`；结构化两条 `{from:null,edge:null}` 与 `{from:review,edge:back}`；`draft(main)×1` | PASS |
| 手写状态两格式一致（正） | 手改时间/原因/计数的 WorkState | 文本 `reason: 超时退出`；`total=5400s`、`avg=900`、`blocked=2` 两处一致 | PASS |

不使用两个 render 函数互为 oracle：期望值来自手写字符串、提交记录与合同示例。

## 门禁

| 命令 | 退出码 | 原始输出 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 0 | [fmt.txt](fmt.txt) |
| `cargo check --all-targets --all-features` | 0 | [check.txt](check.txt) |
| `cargo clippy --all-targets --all-features -- -D warnings` | 0 | [clippy.txt](clippy.txt) |
| `cargo nextest run --all-features --no-tests=pass` | 0；386 passed | [nextest.txt](nextest.txt) |
| `scripts/check-docs.sh`、`scripts/check-specs.sh`、`git diff --check` | 0 | 提交前复跑 |

## 覆盖与边界

- 准备单元完成：O13 的字段面（reason、完整 ArtifactRef、blocked、next 同形）与 N07 的事实口径（边类型保留、累计受阻由转换记录且取消不减少）。O13/N07 的产品闭环（持久 caller、`cli-result/v2`、真实 CLI 终态/失败/NoLegalEdge→cancel）按 plan 留待 T07 后完成，此前不记关闭。
- `blocked_count` 使 `state_json` 增加必填字段：T07 切 schema 2 时的 fixture/重建随格式切换一次完成；本任务不用 `serde(default)`。
- CLI 仅做最小同形适配（`ok_work_next`），响应语义不变。
- 独立 review：Owner 按任务卡自查；独立 Reviewer 审查由 M1 承担。
