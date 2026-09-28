# C002-T04 证据

- Owner：Claude（glm 会话）。基准 `429cb1d`，待提交树见提交说明。
- 输入闭包：`Cargo.lock` 未变；特性全开；平台 macOS aarch64、`rustc 1.98.1`（unix 文件 API：`symlink_metadata`、`File::metadata`(fstat)、dev/ino 身份比对、`OpenOptions::create_new`(O_CREAT|O_EXCL)、`File::sync_all`、目录 fsync、`File::set_permissions`(fchmod)）。
- 改动：新增 `crates/sheltie-runtime/src/fsx.rs`（SafeFile、`ensure_dirs_under`、`write_exclusive_atomic`、`remove_tree_no_follow`、`copy_tree_confined`、`set_tree_readonly_confined`、`make_tree_writable`）；`home.rs` 根入口规范化；`observe.rs`、`service.rs`、`workbook_repo.rs`、`selfmgmt.rs` 全部改走 fsx；删除 service 的固定 `tmp-pending` 原子写、workbook_repo 的本地 copy/collect/readonly helper。测试新增 `crates/sheltie-runtime/tests/fs_boundary.rs`；按白名单勘误更新三个既有测试的期望（canonical 根与含根只读，见 tasks.toml 注记）。

## 正反例

| 例 | 入口 | 独立期望 | 结果 |
| --- | --- | --- | --- |
| works 父软链（反） | runtime `service.start` | `InvalidRequest` 点名软链；根外哨兵字节+权限不变；根外目录条目数不变 | PASS |
| bin 父软链（反） | `selfmgmt::install` | 同上；哨兵不动 | PASS |
| 叶软链输入（反） | `service.begin` 观察 | 拒绝点名软链；哨兵不动 | PASS |
| 固定名临时软链（反） | 预置 `status-card.md.tmp-pending` 软链后 `work cancel` | 哨兵不动；状态卡写到正确位置且内容正确 | PASS |
| 观察后替换（反/正） | `SafeFile` 路径换文件后旧句柄再读 | 旧句柄仍读到原对象（fd 钉住）；新开句柄拿到新内容，封存核对不通过 | PASS |
| 超限文件（反/边界） | `SafeFile::read_bounded` | 恰好 32 MiB 接受；多一字节读取前拒绝；service 层 33 MiB 输出在观察步报 `OUTPUT_TOO_LARGE` | PASS |
| COMMIT 前观察拒绝 | 软链输出后 `service.submit` | Store 状态不推进、无提交 | PASS |
| 原子写（正） | `write_exclusive_atomic` | 目标被 rename 替换、嵌套父目录创建、目录内无 `.tmp` 残留 | PASS |
| ensure_dirs 单元（反） | 软链段/文件占位段/根外目标 | 逐项拒绝；合法嵌套创建成功 | PASS |
| 嵌套输出与 @file（正） | 既有 370 测试回归（CLI scenario 覆盖 @file 与嵌套输出） | 全部通过 | PASS |

外部哨兵的比较用 `(全部字节, mode)` 快照，前后逐项相等。

## 门禁

| 命令 | 退出码 | 原始输出 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 0 | [fmt.txt](fmt.txt) |
| `cargo check --all-targets --all-features` | 0 | [check.txt](check.txt) |
| `cargo clippy --all-targets --all-features -- -D warnings` | 0 | [clippy.txt](clippy.txt) |
| `cargo nextest run --all-features --no-tests=pass` | 0；370 passed | [nextest.txt](nextest.txt) |
| `cargo test -p sheltie-core --doc` | 0（compile_fail 文档不变） | [doctest.txt](doctest.txt) |
| `scripts/check-docs.sh`、`scripts/check-specs.sh`、`git diff --check` | 0 | 提交前复跑 |

## 覆盖与边界

- 关闭：O01（管理根隔离贯通——根下软链段拒绝、固定临时名删除、句柄身份核对、含根只读、不跟随软链的清理、句柄复制与限额读取）与 N14 的读取部分（观察/资源索引/摘要先限额后读、流式计数）。
- `pending/` 目录尚不存在（T07 引入），其父软链保护由同一 `ensure_dirs_under` 代码路径承担；commit 后封存/投影失败的 `committed=true` 响应形状按协议 §5 归 T07，本任务的行为面是：封存前按记录摘要核对，不符不 chmod。
- 观察与打开之间用 dev/ino 身份比对关闭替换窗口（无 libc 依赖下的 std 能力）；stat→open 的纯竞态注入未做并发实验，M1 突变/审查覆盖。
- 独立 review：Owner 按任务卡自查；独立 Reviewer 审查由 M1 承担。
