# C002-T05 证据

- Owner：Claude（glm 会话）。基准 `5196cb1`，待提交树见提交说明。
- 输入闭包：新增依赖 `uzers 0.12`（`Cargo.lock` 相应更新，deny 全绿）；特性全开；平台 macOS aarch64、`rustc 1.98.1`。
- 改动：core `workbook/manifest.rs` version 保留名（`.`、`..`、`.staging`）；runtime `observe.rs` 真实 OS 主体（uzers，忽略 `USER`/`USERNAME`）；`store/mod.rs` 合法写操作建库前先建父目录（O06）；`fsx.rs`/`observe.rs` 宿主元数据 `.DS_Store` 准确拒绝；`workbook_repo.rs` load 核登记摘要（O03）、remove 前核归属；`service.rs` start 冻结副本复制后重新 parse/compile 并核对身份与摘要、图取自最终副本字节。测试新增 `crates/sheltie-runtime/tests/workbook_identity.rs`、core manifest 内嵌；更新 CLI gated-release 的批准人期望为 `id -un` 独立值并加伪造 USER 反例（归属注释保持 T22，正文按 D-036 更新）。
- 决策勘误：D-036 由 `users 0.11.0` 改为同 API 维护分支 `uzers`——`cargo deny` 拒绝 users 的 RUSTSEC-2023-0040/0059 等 advisory，工程规范禁止藏告警改规则；语义与确认方式不变（见 D-036「C002-T05 实施勘误」）。

## 正反例

| 例 | 入口 | 独立期望 | 结果 |
| --- | --- | --- | --- |
| verify tampered 后 start（反，O03） | 改已装文件后 `service.start` | `WORKBOOK_TAMPERED`；works/ 无物化 | PASS |
| 清理前核归属（反） | 改已装目录后 `workbook remove` | `WORKBOOK_TAMPERED`；行保留 | PASS |
| 复制核验（反，静态） | `start_write` 复制后 load_dir 重 parse/compile | manifest 身份不符或摘要不符报 `STORE_CORRUPT`；图取自副本字节。复制间源变化的动态竞态未注入，`not_run`（见边界） | 静态实现 |
| version 保留名（反） | core parse | `.`、`..`、`.staging` 拒绝且 field=version；`.1.0` 接受 | PASS |
| Finder 元数据（反，§5.3） | 源目录含 `.DS_Store` 后 `workbook add` | 拒绝并点名 `.DS_Store`；无行、无最终目录 | PASS |
| 新管理根 self install（正，O06） | 空根 `selfmgmt::install` | 建根、store.db、bin；幂等 already_installed | PASS |
| 只读不创建（反） | 缺库 `Store::open(ReadOnly)` | `NOT_FOUND`；管理根零条目 | PASS |
| 假 USER（反，N04） | 子进程 `USER=spoofed` 跑真实 CLI 开工 | audit 主体 ≠ 伪造值、= `id -un` 独立输出 | PASS |
| gate 批准人（反） | CLI `USER=spoofed` gate approve | 批准人 = `id -un`，≠ 伪造值 | PASS |
| 只读位边界（正/反，N10） | 冻结副本含根 0555；chmod 放开后改字节 | 0555 确认；改动使摘要变化（独立重算）且 begin 报 `STORE_CORRUPT`——权限只是减少误写，摘要是防线 | PASS |
| 删除后终态 Work 可读（正） | remove Workbook 后 `work status` | 状态卡与 JSON 完整 | PASS |

## 门禁

| 命令 | 退出码 | 原始输出 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 0 | [fmt.txt](fmt.txt) |
| `cargo check --all-targets --all-features` | 0 | [check.txt](check.txt) |
| `cargo clippy --all-targets --all-features -- -D warnings` | 0 | [clippy.txt](clippy.txt) |
| `cargo nextest run --all-features --no-tests=pass` | 0；380 passed | [nextest.txt](nextest.txt) |
| `cargo deny check`（新依赖触发） | 0 | [deny.txt](deny.txt) |
| `scripts/check-docs.sh`、`scripts/check-specs.sh`、`git diff --check` | 0 | 提交前复跑 |

## 覆盖与边界

- 关闭：O03（load 核登记摘要；复制核验为静态实现）、O06（新根 self install/add；add 由同一 Store 建库路径覆盖）、N04（真实 OS 主体，`USER` 失效，D-036 勘误 uzers）、N10（含根只读已在 T04 落地，本任务补「权限非防线、摘要是防线」的边界测试）与 §5.3 宿主元数据、version 保留名。
- 复制间源目录变化的**动态**竞态注入未执行（需要 failpoint 插桩 load 与 copy 之间）；静态实现是复制后对最终副本重新解析并比对登记摘要，任何窗口内的变化都会造成摘要不符而 `STORE_CORRUPT`。`not_run` 如实记录。
- uid 回退形式（`uid:<n>`）与 Windows 分支无测试环境，`not_run`。
- 独立 review：Owner 按任务卡自查；独立 Reviewer 审查由 M1 承担。
