# C002-T09 证据

- Owner：Claude（glm 会话）。基准 `6b40abd98d46b878286bdc531021c6c4bd9dce50`，待提交树见提交说明。
- 输入闭包：`Cargo.lock` 未变；特性全开；平台 macOS aarch64、`rustc 1.98.1`。工作区起点干净。
- 改动：`crates/sheltie-core/src/digest.rs`（`WORKBOOK_DIGEST_V2_PREFIX`、`be64`、`digest_v2_file_frame`、`from_sha256` 单次哈希收口）、`crates/sheltie-runtime/src/workbook_digest.rs`（新，流式 `digest_dir_v2`，尚未接生产 caller）、`runtime/src/lib.rs` 导出；测试在两个文件内嵌与 `crates/sheltie-runtime/tests/workbook_digest.rs`（新）。生产 `WorkbookRepo::digest_dir` 仍为旧算法，T07 一次切换。

## 独立 oracle

期望摘要值全部由 `python3 hashlib` 对**手工拼出的字节流**（手写 BE64，不经 Rust helper）独立算出；core 测试里的流也逐字节手写。生产 `digest_dir_v2` 不参与期望值生成。

实现期间独立向量抓住一次真实缺陷：初版用 `Sha256Hex::of_bytes(&hasher.finalize())` 收口，等于把摘要再哈希一次（旧双重哈希模式），空目录向量立刻不符；已改为 `Sha256Hex::from_sha256`（单次哈希直接转十六进制）。

## 正反例

| 例 | 输入 | 独立期望 | 结果 |
| --- | --- | --- | --- |
| 已知向量（正） | 空目录 / `a.txt=hello` / 嵌套多文件 + 空目录 | `fcf10ccb…`、`c760ce85…`、`fdd25e35…`（python3 独立算出） | PASS |
| O07 碰撞对（反） | `za="zb\0X"` 与 `za=""+zb="X"` 两棵目录 | 两种摘要不同且分别等于 `3947b322…` 与 `e850aeab…` | PASS |
| 顺序无关（正） | 同一组文件按不同次序创建 | 摘要相同（按路径字节序排序） | PASS |
| 一字节之差（反） | 路径或内容各改一个字符 | 摘要变化且等于独立常量 `98be5ca8…` / `ee29a7d5…` | PASS |
| 字节序排序（正） | `B.md` 与 `a.md` | `B(0x42)` 排在 `a(0x61)` 前，等于 `31d5fff6…` | PASS |
| 非法文件（反） | 符号链接、硬链接 | `InvalidRequest`，理由点名链接类型 | PASS |
| 单文件上限（反/边界） | 稀疏文件恰好 32 MiB / 多一字节 | 接受 / 读取前拒绝 | PASS |
| 总量上限（反/边界） | 8×32 MiB=256 MiB / 再加 1 字节文件 | 接受 / 读取前按总量拒绝 | PASS |
| 帧构造（core） | prefix 27 字节、BE64 大端、帧头逐字节 | 手写字节向量 | PASS |

## 门禁

| 命令 | 退出码 | 原始输出 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 0 | [fmt.txt](fmt.txt) |
| `cargo check --all-targets --all-features` | 0 | [check.txt](check.txt) |
| `cargo clippy --all-targets --all-features -- -D warnings` | 0 | [clippy.txt](clippy.txt) |
| `cargo nextest run --all-features --no-tests=pass` | 0；359 passed | [nextest.txt](nextest.txt) |
| `scripts/check-docs.sh`、`scripts/check-specs.sh`、`git diff --check` | 0 | 提交前复跑 |

## 覆盖与边界

- 关闭的准备单元：O07 的 `workbook-digest/v2` 算法本身（单 SHA256、BE64 定界、字节序排序、限额前置、流式计数）。O07 的产品闭环（add/load/verify/冻结副本改用 v2、schema 2 唯一摘要语义）按 plan 归 T07；旧算法在此之前仍是 schema 1 生产路径，不与新算法并存于同一字段，也不保留为 fallback。
- 读取间增长的动态竞态未做并发实验：帧头长度与实际计数核对是静态实现 + 代码审查，竞态注入留 M1 突变/审查。
- 独立 review：Owner 按任务卡自查；独立 Reviewer 审查由 M1 承担。
