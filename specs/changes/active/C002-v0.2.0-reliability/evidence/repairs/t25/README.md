# C002-T25：统一恢复与提交错误

候选基线 `5d2d051`（T24 follow-up `edb86f1` 已单独提交）。macOS arm64；Linux 按用户授权保持 `not_run`，不作跨平台结论。

## 实现

- 新增 `recovery.rs` 作为统一效果恢复与提交错误编排器。它只依赖 `RecoveryAccess` 能力接口，不引用 Service/Repo 具体类型；WorkService 与 WorkbookRepo 提供各自的完整持久行/效果校验及最新状态读取。`load.rs` 实际执行按最新 Store WorkState 重绘状态卡。
- 写请求按 `audit.seq` 顺序恢复。先用 hash-only 查询判断同 request-id 意图是否冲突；确认已提交后再读 reply/effects。旧 A 阻断 B 时报告 B 的 request-id、A 的 pending_request_id 与可验证的 A 原响应；当前请求自身失败时保留自己的提交身份。
- Work 原响应从无默认值的 `PersistedResponse` 生成协议成功封装；Workbook snapshot 通过严格 Add/Remove DTO 核验后生成封装。work/data/next 字段与持久 Reply 一致时才提供 `original`；若快照字段、payload 或嵌套 WorkStatus 损坏，则保留 `committed/request_id/cause`，`original` 留空，不伪造 `ok:true`。
- Workbook add/remove 重放先取得写锁并恢复未完成效果；已完成的旧 add 按自己的历史请求快照验证，不绑定当前 workbooks 行或新生命周期。所有 Work/Workbook 写入口执行整组 checked effects、刷新最新卡片、再要求 `mark_published` 恰好更新一行。
- SQLite BUSY/LOCKED、FULL、IO、权限及打开失败映射为 `IO` cause；schema 版本或表结构不符保留 `STORE_SCHEMA_MISMATCH`，持久数据/效果结构损坏保留 `STORE_CORRUPT`。维护清理诊断未在当前 T25 实现：执行清理、stderr/exit 0 与业务快照分离明确交 T28，V25 在 T28 完成前仍未关闭。

## 验证

- `task.stdout.txt`：`scripts/task.sh C002-T25`，27 passed、550 task-filtered，run `b959050f-291f-4c4e-b1bb-b1c545fcfaba`。
- `workspace-nextest.stdout.txt`：`cargo nextest run --all-features --no-tests=pass --no-fail-fast`，577 passed、0 skipped，run `2de4518b-59bd-45d2-87c2-79c9422f4127`。
- `fmt.stdout.txt`、`cargo-check.stdout.txt`、`clippy.stdout.txt`、`msrv-check.stdout.txt`、`check-docs.stdout.txt`、`check-specs.stdout.txt`、`check-core-vocab.stdout.txt`、`check-tests.stdout.txt`、`check-skill.stdout.txt` 均通过；Rust 1.85.0 使用 `--locked`。
- Spec 与 Standards 第三轮独立复核均为 PASS。T24 后续 Store 初始化竞态独立提交 `edb86f1`，真实 add/install 交错、锁移除、旧 schema/main/WAL 保留 oracle 见 [T24 follow-up](../t24-followup/README.md)。

## 保留的 LEAK 观察与边界

`task-pre-strict-snapshot.stdout.txt` 保留 Nextest run `7599a341-821f-4538-aa57-de30b0a7d803` 的历史 `LEAK`：纯 JSON `error_map` 测试在与多个任务测试并行时被标记 leaky。对应测试的隔离 run `leak-isolation-error-map.stdout.txt`（`403cae6b-2e49-4ac0-b264-479a9babb510`）通过且无 LEAK；这不能证明原并行 LEAK 的根因已关闭。另一次未保存原始stdout的中间任务过滤运行曾标记两个 Workbook 损坏用例LEAK；`leak-isolation-publish.stdout.txt` 和 `leak-isolation-remove.stdout.txt` 是之后各自单测的独立通过记录，不代替原运行证据。该观察缺少原始run id与完整输出，原因保持未确认。最终任务过滤 27/27 与全仓 Nextest 577/577 均未报告 LEAK。

T25 不替代后续 T26 发布完整归属与 sync、T27 删除完成证明、T28 pending 清理/只读发现、T31 全链交错与突变验证或 C002-M1。Linux 运行均为 `not_run`。
