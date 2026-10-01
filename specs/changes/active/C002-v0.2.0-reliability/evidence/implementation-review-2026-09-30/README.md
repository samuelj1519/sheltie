# C002 实现审查证据

审查入口：[review-implementation-2026-09-30.md](../../review-implementation-2026-09-30.md)。候选为 `b926789093c51c62ee742bfbe1cb0c3c43044035` 加本轮开始时全部未提交 T31 内容。`candidate.patch` 保存 tracked 差异；`input-closure.json` 同时记录 untracked 源码摘要、工具版本与治理夹具复制输入。本轮没有修改或提交实现。

## 本轮运行

Cargo 命令均在仓库根运行，使用 `RUSTC_WRAPPER=` 与 `CARGO_TARGET_DIR=/private/tmp/sheltie-c002-review-9NZouweW/target`。以下均为本轮执行，无历史结果复用。

| 命令或检查 | 退出码 | 结果与原始输出 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 0 | [fmt.log](fmt.log)，成功时无输出 |
| `cargo check --all-targets --all-features` | 0 | [check.log](check.log) |
| `cargo clippy --all-targets --all-features -- -D warnings` | 0 | [clippy.log](clippy.log) |
| `cargo nextest run --all-features --no-tests=pass` | 92 | [nextest.log](nextest.log)：本机 0.9.140，配置要求 0.9.145；正式门禁受工具版本阻断 |
| `cargo nextest run --override-version-check --all-features --no-tests=pass` | 0 | [nextest-override.log](nextest-override.log)：647 run，647 passed，0 skipped，24 slow、1 leaky；耗时 932.579 秒。这是补充运行 |
| `cargo build -p sheltie-cli --all-features --message-format=json` | 0 | [build.json](build.json)、[build.log](build.log)；供独立 probe 取得真实 executable |
| 两个真实 CLI 反例 | 0 | [probes.log](probes.log)、[probes-evidence.json](probes-evidence.json)；退出 0 表示两个错误均成功复现 |
| `scripts/check-docs.sh` | 0 | [docs.log](docs.log)：182 个 Markdown 文件通过 |
| `scripts/check-specs.sh` | 0 | [specs.log](specs.log)：8 个 change、1 个 active，通过 |

`leaky` 对应 `sheltie-cli::release_governance::check_specs_accepts_active_target_without_tag`。该警告与原始结果一起保留，本轮未定位其原因，不把它当作新的已验证代码缺陷。

源码、fixture 和合同的 273 项摘要在结束时一致。治理输入中的 `evidence/repairs/t31/mutants/stage2-sheltie-runtime-2-of-8/stdout.txt` 被其他运行追加；前后摘要与字节数保存在 [input-audit.json](input-audit.json)。本轮不改这个日志，不用本次补充测试声明完全固定的治理输入闭包，也不关闭 T31 mutation 义务。

Linux、MSRV、完整 mutation、真实 Host、发布、CR-S01 的动态同步失败注入：本轮均为 `not_run`。报告与新增证据的文档检查另见 `docs.log`、`specs.log`；它们发生在源码补充测试结束后。

## 可重跑反例

[probes.py](probes.py) 从 Cargo JSON 取得 executable，每次创建两个独立临时管理根。它只 SIGKILL 自己启动的初始化进程，只修改自己夹具中的 `state_json.approvals`，不碰真实用户管理根。

```bash
review_run=$(mktemp -d /private/tmp/sheltie-c002-reprobe-XXXXXXXX)
RUSTC_WRAPPER= CARGO_TARGET_DIR="$review_run/target" \
  cargo build -p sheltie-cli --all-features --message-format=json \
  > "$review_run/build.json"
python3 specs/changes/active/C002-v0.2.0-reliability/evidence/implementation-review-2026-09-30/probes.py \
  --cargo-json "$review_run/build.json" --repo "$PWD"
```

脚本断言的是被审候选的实际错误：初始化被杀后重试报 schema mismatch；删除唯一 gate approval 后 status/begin 仍成功。修复候选应改用拒绝损坏状态、成功恢复初始化的回归断言，不能把当前 probe 的退出 0 当作修复 PASS。

初次独立复现还分别保存在 [initialization-evidence.json](initialization-evidence.json) 与 [gate-evidence.json](gate-evidence.json)。CR-S01 的 sync 恢复缺口来自报告所列静态调用链，没有伪造对应运行输出。
