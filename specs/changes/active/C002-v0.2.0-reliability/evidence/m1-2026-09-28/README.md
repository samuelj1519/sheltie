# M1 审查原始证据

运行标识：`m1-2026-09-28-e1a8126`。候选、基线、平台、Cargo.lock、全部被审文件摘要与 probe binary sha256 见 [input-closure.json](input-closure.json)。结论与 27 项关闭矩阵见 [审查报告](../../review-m1-2026-09-28.md)。本目录仅保存审查材料，不改变任务状态。

## 门禁

[results.json](results.json) 是按顺序执行的原始命令/exit/耗时。每条命令分别有同名 `.stdout` / `.stderr`；空文件表示该流为空。`cargo deny check` 首轮因 Cargo advisory 缓存锁 sandbox 权限失败，获准缓存访问重跑 exit 0；两份原文均保留。[gate-supplement.json](gate-supplement.json) 记录该补充与未完成模式。全量 nextest 464/464；MSRV 1.85.0 locked、dist plan 和治理脚本实跑。未复用先前任务的 PASS。

## 独立正反例

| 脚本 / 证据 | 内容 |
| --- | --- |
| [controls.py](controls.py)、[controls.json](controls.json)、[controls.stdout](controls.stdout) | Python独立摘要、跨Work冲突、历史快照完整比较、合法pending恢复、pending Work标记 |
| [extra_controls.py](extra_controls.py)、[extra-controls.json](extra-controls.json)、[extra-controls.stdout](extra-controls.stdout) | 状态推进后旧stats逐字节恢复且状态卡不回退；真实登记begin效果的合法path与单条件根外path配对 |
| [probes.py](probes.py)、[probes.json](probes.json)、[probes.stdout](probes.stdout) | @summary文件删除、COMMIT后状态卡失败、自己的pending重放/旧请求阻断新请求、历史父目录缺失、删除后无标记 |
| [recovery_probes.py](recovery_probes.py)、[recovery-probes.json](recovery-probes.json)、[recovery-probes.stdout](recovery-probes.stdout) | 真实COMMIT前/后退出、跨Workbook恢复Work、单独改输入/单独改owner、前缀变歧义、@start文件删除 |
| [init_lock_probe.py](init_lock_probe.py)、[init-lock-probe.json](init-lock-probe.json) | 父进程持本根锁时真实CLI仍建schema2库 |
| [Standards 报告](standards/report.md) | self路径、purge、效果路径、句柄封存、并发测试源码与独立探针 |
| [Spec 报告](spec/report.md) | state根外路径、Workbook pending/重放、摘要根软链、第二次plan任务书 |

探针在仓库根执行，临时管理根均在 `/private/tmp`。脚本保留本轮使用的绝对 binary、仓库和证据路径，重跑到其他机器时需对应替换路径；不应把输出中的旧 temp 根当当前事实。需要 failpoint 的 binary 已按 Cargo JSON 返回的 executable 定位；实际构建输出见 [build-metadata.jsonl](build-metadata.jsonl)。`safe_file_seal_probe.rs` 用本候选 runtime rlib 直接链接，未另跑 cargo；脚本源和原始输出保留，临时 fixture 树及可执行 probe 未纳入仓库。

Workbook发布与删除完成标记反例用临时目录/SQLite构造可观察窗口；它们不是实际kill。start和Workbook提交前反例是真实feature注入exit70。进程kill也不等同断电。预期来自合同、原始响应/文件字节、Pythonhashlib和哨兵状态，不由生产helper计算自身答案。

## 未完成边界

[mutants-list.json.gz](mutants-list.json.gz) 是 `cargo mutants -p sheltie-core -p sheltie-runtime --list --json --exclude 'crates/*/src/testkit.rs'` 的无损压缩原文（1137个候选）。只有枚举，没有执行或存活体处置，保持 `not_run`。完整kill矩阵、Linux文件API、断电模拟、远端CI、Host/usage与发布同样 `not_run`。本轮已确认需修改行为，M1门槛未关闭。
