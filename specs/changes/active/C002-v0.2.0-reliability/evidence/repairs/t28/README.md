# C002-T28：pending 清理与只读发现

基线：`3dd224d`。平台：macOS arm64。Linux 原生运行按用户授权保留 `not_run`，本任务不声明跨平台 PASS。

## 实现与边界

清理先装入全部 requests.effects_json，校验所有路径和请求归属后才删除。合法 owner 且没有任何 Store 引用的私有孤儿树使用同一 ManagedTree 放开冻结权限并删除；published=0 原件始终保留。published=1 只清同对象空容器、owner 和合法 deleted marker，异常或非空对象保留并告警。空容器资格与删除保持同一目录句柄，删除过程不递归认领新内容。

对象身份核验与目录删除遵循 HomeLock 协作模型。stat 与 unlink 是不同系统调用，不能提供任意无锁外部写者下的原子同对象 unlink 保证；本任务验证观察到换绑或新内容时保留现场。

Work 和 Workbook 共用只读定位、有限重读。pending 必须有合法 owner；完成 final 由当前业务行、对应成功请求、审计、快照、effect 和内容共同核验。Workbook 当前生命周期以 audit.seq 排序，不能用秒精度 added_at 代替事件顺序，也不能预筛损坏审计或回退旧 Add。历史快照重放继续独立于当前新生命周期。

Workbook list/show/verify 和 Work status 返回 pending_publish，rename 完成但尚未 mark 仍为 true。合法 pending 的 verify 为 ok。start 先从同一 pending 图预检，写锁内恢复后再核 final。资源和说明书从实际冻结目录装入，持久 ArtifactRef 仍保持 final 路径。只读测试核主库字节不变、没有创建 .lock、没有执行恢复；SQLite 控制文件按 D-039 的既有例外处理。

成功 CLI 写命令随后执行持锁维护。告警仅输出 stderr，含 request_id、object 和 reason，归属未知时明确 unknown；stdout 成功 JSON、原快照和 exit0 不变。维护失败不重做已完成业务效果。

## 验证与审查

`task.stdout.txt` 保存最终 `scripts/task.sh C002-T28` 原始输出。V24–V27 覆盖合法 readonly 孤儿、无 owner/异常容器、published0、published1 非空树、坏 JSON 和合法 JSON 的非法路径、成功请求证明、真实 rename/mark-cleanup 交错、完成缺失分类和 stderr 载荷。首轮审查及后续修订见 [review-corrections.md](review-corrections.md)。

最终定向 run `348c5ee9-71fb-473e-920a-b5a53ec80d7e`：22 passed、593 task-filtered。最终全仓 run `6e159189-3ad3-42c1-a4c8-e41a0b85304f`：615 passed、0 skipped、1 slow、无 LEAK，exit0。`candidate-input.txt` 保存基线、特性、平台及所有变更 Cargo/Rust/测试文件的 SHA256。

Rust 四门禁、MSRV 1.85.0 locked、离线 deny 与 docs/specs/core-vocab/tests/skill 均保存本目录原始输出。Cargo 使用 `/private/tmp/sheltie-c002-t28-target`，禁用受限环境无法启动的 sccache。deny 使用临时 Cargo home 的本机 advisory DB，显式 `cargo deny --offline check`；原始输出压缩保存，文本副本仅去除尾随空白。

初始 fmt 检查指出新增测试尚未格式化，初始 Clippy 检查指出六处风格问题，均已修正。`fmt-initial.stdout.raw.gz` 保留原始 ANSI 输出；对应文本副本仅去除尾随空白，以通过仓库 whitespace gate。

```bash
cargo fmt --all -- --check
RUSTC_WRAPPER= CARGO_TARGET_DIR=/private/tmp/sheltie-c002-t28-target cargo check --all-targets --all-features
RUSTC_WRAPPER= CARGO_TARGET_DIR=/private/tmp/sheltie-c002-t28-target cargo clippy --all-targets --all-features -- -D warnings
RUSTC_WRAPPER= CARGO_TARGET_DIR=/private/tmp/sheltie-c002-t28-target scripts/task.sh C002-T28
RUSTC_WRAPPER= CARGO_TARGET_DIR=/private/tmp/sheltie-c002-t28-target cargo nextest run --all-features --no-tests=pass
RUSTC_WRAPPER= CARGO_TARGET_DIR=/private/tmp/sheltie-c002-t28-msrv-target cargo +1.85.0 check --locked --all-targets --all-features
CARGO_HOME=/private/tmp/sheltie-c002-t28-cargo-home2 RUSTC_WRAPPER= cargo deny --offline check
scripts/check-docs.sh
scripts/check-specs.sh
scripts/check-core-vocab.sh
scripts/check-tests.sh
scripts/check-skill.sh
scripts/check-task.sh C002-T28 --staged
```

`workspace-nextest-review-regressions.stdout.txt` 保留审查修订期间的四项回归失败：完成态缺失错误分类，以及两个旧 row-retention oracle。修订后的影响面 runtime 129 项通过；最终全仓运行另以 `workspace-nextest.stdout.txt` 为准。更早通过的 603/604/615 项运行不替代最终候选。

早期定向并行运行出现过 Nextest LEAK；隔离运行及后续运行通过，原因尚未确认，不能用重跑 PASS 关闭原因。该运行现象交 T31 综合验证记录。

## 后续

两位独立 Reviewer 对最终源码给出 Spec/Standards task-local PASS。T28 完成只关闭 pending 清理与只读发现；T29 同次事实快照、T30 spec-dev 接续、T31 综合故障窗口和突变处置、M1 全链审查由各自门槛决定。T16 真实 Host 与 T17 发布继续由计划规定的操作者和授权边界决定。
