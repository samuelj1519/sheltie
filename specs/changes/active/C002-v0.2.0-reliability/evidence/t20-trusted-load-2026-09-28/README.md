# C002-T20 持久路径与效果可信装入证据

候选基线：`ee78118`（C002-T19）。实现者：Codex。平台：macOS arm64。Linux 按用户明确授权豁免，状态仍为 `not_run`；本记录不构成跨平台 PASS。

## 行为边界

- 装入 Work 前先要求 `state_json.work_dir == Home.work_dir(work_id)`，冻结副本只能来自该 Work 的 `workbook/` 或经严格 owner/request/audit/snapshot 归属核验的 `pending/<uuid>/payload/workbook/`。
- `load.rs` 从冻结副本解析 manifest、Flow 与资源索引，再按图定义逐项核 WorkStart 输入、历史 Attempt 输入、声明输出名/路径和 engine.stats 路径；错 WorkId、节点、key 或路径返回 `STORE_CORRUPT`。
- `requests` 未完成项先完整枚举，不能靠 audit INNER JOIN 漏掉缺审计的请求。Work 请求必须且只能有一条 audit，WorkId、revision、timestamp、strict snapshot、Command、RequestIntent hash 形状与当前已校验状态互相一致。
- effects 在成为 `CheckedEffects` 前核对命令所需完整列表、数量、顺序、唯一性、owner、Attempt、目录骨架、所有目标路径、SHA-256、精确历史字节和同次 engine.stats ArtifactRef。原始 `EffectOp` 没有执行入口。一个坏的后续 WriteFile 会让整个批次在第一个效果前停止。
- persisted `Response` 外壳、`Reply`、`NextOp` 与 Workbook Added/Removed snapshots 均拒未知或缺失字段；`snapshot.data` 对每种 Reply 有精确key集合，机器路径字段与typed Reply/工作状态逐项相同。
- Workbook 行核验 `id/version/digest/added_at/dir`，并要求装入 manifest 的最终 id/version 与登记行一致。remove 也先按行身份校验；行缺失且目录仍存在时不把该目录当作可删对象。
- `PendingOwner` 侧车写入与读取均用 `deny_unknown_fields` Serde DTO，Work start 与 Workbook add/remove 恢复都必须核同一个 owner 侧车；stage 也用 serde 序列化，opaque request-id 中的引号/换行能无损往返。schema 2 原 audit 线格式保持：add 的 `source` 仍是 request intent hash；remove 保持 `target = "id@version"`。
- T20 未关闭 T21 安全目录枚举/摘要完整 caller、T22 同句柄封存、T23 self/purge、T24 请求初始化、T25–T28 完整恢复/发布/删除/pending、T31 故障窗口/突变或最终 M1。

正反例覆盖：state.work_dir 改根外；Attempt产物路径改成指向根外且字节相同的文件；Begin snapshot brief_path 改根外、删除 data、给 Reply/NextOp 加未知字段；后续 WriteFile 路径损坏、效果批次清空、audit 缺失/重复、published 改为 2；engine.stats ArtifactRef bytes 单字段篡改；Workbook 行 dir/version 变成越界路径；Workbook remove digest 清空；Workbook publish 删除必需 `digest_root`；pending request-id含引号与换行。所有损坏持久数据均在首个效果写入前拒绝，哨兵 bytes/mode 保持不变。

## 门禁

以下每个 gate 的原始 stdout、stderr 和退出码独立保存为 `<gate>.*`。

| Gate | 命令 | Exit | 结果 |
| --- | --- | ---: | --- |
| fmt | `cargo fmt --all -- --check` | 0 | 格式一致 |
| check | `cargo check --offline --all-targets --all-features` | 0 | 全部 crate 与 target 检查通过 |
| Clippy | `cargo clippy --offline --all-targets --all-features -- -D warnings` | 0 | 无 warning |
| 全仓 Nextest | `cargo nextest run --offline --workspace --all-features --no-tests=pass --no-fail-fast` | 0 | 最终run `39ad8a9c-a292-4a00-9f68-cc9d1535c6e1`；492 passed、0 skipped、0 leaky |
| T20 定向 | `scripts/task.sh C002-T20` | 0 | 最终run `19522a54-4921-44e6-9e00-121ea4395661`；17 passed、2 leaky（用例见`task.stderr.txt`）、475 skipped，无失败 |
| 暂存区白名单 | `scripts/check-task.sh C002-T20 --staged` | 0 | `check-task.stdout.txt` |
| 暂存区白名单 | `scripts/check-task.sh C002-T20 --staged` | 0 | `check-task.stdout.txt` |
| MSRV | `cargo +1.85.0 check --offline --workspace --all-targets --all-features --locked` | 0 | Rust 1.85.0 检查通过 |
| deny | `cargo deny --offline check` | 0 | advisories/bans/licenses/sources通过；deny配置的未命中许可白名单与winnow多版本只产生warning |
| 文档与测试治理 | `scripts/check-docs.sh`; `scripts/check-specs.sh`; `scripts/check-core-vocab.sh`; `scripts/check-tests.sh`; `scripts/check-skill.sh`; `git diff --check` | 0 | 全部通过 |

`cargo deny --offline check` 使用 `/private/tmp/sheltie-cargo-home3` 中本机缓存 advisory DB，避免网络访问；stderr 的表格空格以 `deny.stderr.raw.b64` 无损保存，可用 `base64 -D < deny.stderr.raw.b64` 还原。

## 独立审查

- Spec reviewer 最终通过：审查范围限于 T20 的 trusted loading、路径与owner关联、效果批次完整性和schema 2格式保持；T21之后的职责不提前关闭。
- Standards reviewer 最终通过：Raw effects 不可直达执行，批次、strict snapshot、request/audit/hash 与 pending owner均有校验和反例。保留一项非阻断建议：未来可让 Workbook version 的纯规则复用 core 入口；当前实现没有为此加框架。
- 所有结论只覆盖 macOS arm64。本机 cargo-deny缓存是本次offline门禁输入；Linux 原生执行仍 `not_run`。
