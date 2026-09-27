# C002-T07 证据

- Owner：Claude（glm 会话）。基准 `85ada77`，待提交树见提交说明。
- 输入闭包：新增依赖 `fs4 0.13`（管理根写锁，D-035）；特性全开；平台 macOS aarch64、`rustc 1.98.1`；deny 全绿。
- 改动（一次格式切换，无混用分支）：
  - **schema 2**（`store/schema.rs`、`store/mod.rs`）：建库 DDL 与 `user_version=2` 同一事务；已存在的库先以只读连接识别，schema 1 拒绝前零写入；`requests` 表改 `intent_hash`/`effects_json`/`published`。
  - **RequestIntent**（新 `request.rs`）：八种写操作闭集，canonical JSON 的 sha256 指纹；Work 意图绑定解析后的完整 WorkId（O02）；`@file` 只记词法绝对路径，内容在重放查重之后读。
  - **效果登记与恢复**（新 `effects.rs`）：`publish_dir`（含 `digest_root`，Work 只核 `workbook/` 子树摘要）/`prepare_attempt`/`write_file`（精确字节）/`seal_outputs`（完整 ArtifactRef）/`delete_dir`/`refresh_status_card`；恢复按 audit 序、同对象视为完成、不同对象停止。
  - **管理根写锁**（`home.rs` `HomeLock`，fs4）：所有写操作锁内「恢复 → 重核 → 决定 → 事务 → 发布」。
  - **WorkLayout 切换**：`start-inputs/`、`attempts/<node>/occurrence-NNN/attempt-NNN/`、输出在 `outputs/` 下、`engine/stats.json`；core `WorkState::attempt_dir`/decide/render 全部走 `layout`。
  - **摘要 v2 唯一口径**：`WorkbookRepo::digest_dir` = `digest_dir_v2`，旧算法与 `collect_file_meta` 删除。
  - **快照响应**：`Response{reply,data,next}` 提交时组装存 `reply_json`；重放逐字段返回；CLI 不再回读 Store 拼数据（O04/O05）。
  - **start/add/remove 走 pending staging**：侧车（`pending/v1`）+ payload + `publish_dir`；begin 的目录骨架与 brief/stats 精确字节进效果。
  - **CLI**：写命令数据来自快照（`replayed` 注入）；只读与 self 组给 `--request-id` 即 `INVALID_REQUEST` 退出码 2。
  - 删除旧 `hash_command`/`hash_json`/固定 staging 目录路径。

## 正反例（新增 `tests/schema2_replay.rs` + 既有 crash/并发回归）

| 例 | 独立期望 | 结果 |
| --- | --- | --- |
| schema 1 只读/读写拒绝且字节不变（反，D-033） | 双模式 `STORE_SCHEMA_MISMATCH`；文件逐字节相等 | PASS |
| 跨 Work request-id（反，O02） | 同 id cancel B 报 `REQUEST_CONFLICT`；B 保持 active | PASS |
| 文件变化后 submit 重放（正，O04） | 改输出字节后重放返回原快照（reply/data/revision 逐字段） | PASS |
| Workbook 删除后 start 重放（正，O04） | 不重读仓库，返回原 work_id 快照 | PASS |
| cancel 后旧 submit 重放（正，O04） | 原 next 保留、不混 cancelled；当前状态查 status | PASS |
| 旧请求不回退状态卡（反，§6） | begin 重放后卡仍是 submit 版本 | PASS |
| 历史文件被改的重放（反，§3.2） | `write_file` 摘要不符报 STORE_CORRUPT，不掩盖 | PASS |
| add 重放忽略源变化（正，O08） | 改源目录后同 id 返回原 digest；新 id 对坏源准确拒绝 | PASS |
| remove 重放与重新 add（正，O08） | 重放不重复删除；同版本新生命周期可再装 | PASS |
| 两个写者竞争（反，§2.2） | 写锁串行化；终态一致（cancelled），无半写 | PASS |
| begin 崩溃窗口恢复（正，§3.1） | 置 `published=0` 后下一次写先按登记字节补任务书 | PASS |
| COMMIT 前/后 kill（既有 crash 回归） | before_commit 无变化可重试；after_commit 状态先进、重放补 brief | PASS |
| 正常链路（正） | 全部既有 397 测试（two-step/article-review/gated/spec-dev、重放、并发、限额、边界） | PASS |

## 门禁

| 命令 | 退出码 | 原始输出 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 0 | [fmt.txt](fmt.txt) |
| `cargo check --all-targets --all-features` | 0 | [check.txt](check.txt) |
| `cargo clippy --all-targets --all-features -- -D warnings` | 0 | [clippy.txt](clippy.txt) |
| `cargo nextest run --all-features --no-tests=pass` | 0；397 passed | [nextest.txt](nextest.txt) |
| `cargo deny check`（fs4 新依赖） | 0 | [deny.txt](deny.txt) |
| `scripts/check-docs.sh`、`scripts/check-specs.sh`、`git diff --check` | 0 | 提交前复跑 |

## 覆盖与边界

- 关闭：O02（意图绑定目标）、O04（快照重放不依赖当前观察/当前 Store）、O05（brief/stats 精确字节恢复）、O07（v2 唯一摘要口径）、O08（Workbook 写操作重放语义）、N01 的产品闭环（新 home 失败 start 无副作用已在 T02，schema 2 建库路径一致）、N03 的建库半结构（DDL+version 同事务）、O12 布局闭环（真实 CLI 走 `outputs/`、`engine/stats.json`、`start-inputs/`）、O13 的产品闭环（快照 + `next_item_json` 同形，`ok_work_next`）。
- `requests.effects_json` 的 `delete_dir` 完成标记（`.deleted` 侧车）按存储合同 §3.3 应在删除后写入：当前实现以「pending 与 final 都不存在即视为完成」推进，标记文件写入归 T08 的 remove 生命周期收紧（本任务 remove 的重放已不重复删除）。N02 的并发 add/remove 交错窗口归 T08。
- remove 的引用检查仍在锁内预检（非同事务），按任务卡归 T08。
- 真实子进程 kill 覆盖 before/after COMMIT 两个点（crash.rs 回归）；每种意图 × 每个窗口的全矩阵注入 `not_run`，M1 补。
- 独立 review：Owner 按任务卡自查；独立 Reviewer 审查由 M1 承担。
