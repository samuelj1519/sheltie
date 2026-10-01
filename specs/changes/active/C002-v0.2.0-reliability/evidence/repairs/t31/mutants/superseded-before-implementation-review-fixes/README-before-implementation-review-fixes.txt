# T31 全链验证与修复证据

状态：**WIP**。代码修复和 macOS 公共门禁已通过；完整变异验证、逐项存活体处置、独立 M1 尚未完成。Linux 按用户指示为 `not_run`。

产品基线 `b926789`。当前源代码/配置/fixture/脚本闭包见 [candidate-input.txt](candidate-input.txt)；变异普通 clone 临时测试提交为 `da2bd13979df88dd987596d7ed726ef99bb89651`，不是产品提交。测试默认公共门禁使用 all-features、Nextest 0.9.146、空 `RUSTC_WRAPPER`、隔离 target 与独立临时 home，不读取或修改真实用户管理根。

## 修复与独立 oracle

| 范围 | 修复及真实入口 | 结果与原始证据 |
| --- | --- | --- |
| 并发证据 | service/workbook_txn/schema2_replay 先启动所有线程，Barrier 后再 join；purge CLI 等待者发出 failed try-lock 事件后才放行 | 全仓与任务测试通过；[覆盖矩阵](coverage-matrix.md)、[去锁反实现](missing-home-kernel-lock.stdout.txt) |
| 发布和恢复窗口 | Start/Add 的 owner、私有副本、COMMIT、rename、mark/cleanup；Begin/Submit 的目录、历史 write 与 seal；Remove 的移入、部分删除、末删、marker、mark/cleanup | 精确同步事件后分别 exit70 与实际 SIGKILL，直连 SQLite 的 original/effects/published 和原文件 bytes 为 oracle；[任务原文](task.stdout.txt) |
| 跨类恢复 | Start 的未发布请求直接由 Add 恢复；Add 的未发布请求直接由 Start 恢复 | 新请求之前不重放旧请求；原件、响应、全部效果和最新卡独立核对 |
| 删除同步 | 已移入 pending 的恢复先同步源、目标父目录，再删除 payload | 两个 sync 失败分别停止、重复失败保留原 bytes；解除故障后恢复；[反实现](missing-delete-parent-sync.stdout.txt) |
| 同请求残留 | 新请求注册前，必须处理同 request-id 的未提交侧车；失败不能登记新的 request/audit | owner/无 request/原 bytes/解除故障后成功；[反实现](missing-same-rid-preparation.stdout.txt) |
| 已提交历史类型 | no-follow 区分缺失与 symlink、dangling symlink、hardlink、FIFO；历史补写使用 NOREPLACE | 异常停止并返回带原 snapshot 的 EFFECT_PENDING，cause=STORE_CORRUPT；保留 outside bytes/mode；[独立复现](r07-independent-probe.json)、[红](dangling-history-red.stdout.txt)/[绿](dangling-history-green.stdout.txt) |
| 系统路径别名 | 精确 `/tmp`、`/var` 和后代只接受经核验的系统链接 | 同一普通 UTF-8 文件 alias/private 正例；任意用户 symlink 反例；[独立探针](exact-alias-independent-probe.json) |
| SQLite/purge | 只允许迟到的安全 SHM 或零字节单链接普通 WAL；异常文件保留并报部分失败 | readonly 等待者不变 main/业务事实；late WAL/SHM 的 symlink、directory、hardlink、非空 WAL 反例；上游合同先澄清 |
| update/purge 进程终止 | 用真实安装 binary/prev 执行 update 和 rollback；purge Store 最后删除，保留 root/.lock inode | 分开 exit70/SIGKILL；Store/binary bytes、根锁身份与旧 Work 不复活 oracle |
| 测试资源回收 | 临时目录持原 root FD，仅 chmod 自有目录，不跟随外部链接、不 chmod 文件 | readonly 树删除成功；外部 symlink/hardlink 哨兵 bytes/mode 不变 |
| 公共 core 合同 | 非空 NodeDef.requires 与公开元数据；32/512 字节声明快照；实际 core 生命周期与单字段持久矛盾；公开 legal_next retry 边界；真实 gate 批准产生 NoLegalEdge | getter 反实现被捕获；合法 gate 累计 blocked_count 从 1 到 2 |
| M1 路由 | check-task 支持 active package 的 Mnn；M1 仅文档白名单，禁止改测试 | [路由反例](m1-routing-negative.stdout.txt)；tasks.toml 使用唯一 M1 table 并可由 tomllib 解析 |

原 CLI crash/OS 身份迁移在 T30 已完成，能力与 Owner 均保留；本任务不删除历史归属。开发中失败原文分别保留为 red/development 文件，不作为当前通过证据。

R20 的原始 CLI panic 与独立固定二进制复验见 [probe](blocked-count-independent-probe.json)。修复只用当前 Attempt、Approval 和 Blocked 状态的必要计数界，三处增量统一 checked arithmetic，不重算持久事实；非法 MAX/0/超界时状态查询和新 gate 命令返回 STORE_CORRUPT，原 Store 全表和业务文件不变。Core 独立输入的 submit/fail/approve 溢出返回既有错误，原 state 不变；[红](blocked-count-red.stdout.txt)、[绿](blocked-count-green.stdout.txt)、[Core绿](counter-core-green.stdout.txt)、[独立复核](spec-review-preparation.md)。旧全仓首轮的诊断优先级失败及原文缺口按实写在 [记录](counter-diagnostic-order-failure.md)；修正后同条件两项[定向复验](counter-diagnostic-order-green.stdout.txt)，新完整候选另跑全仓。

## 当前门禁

[gate-results.json](gate-results.json) 记录完整 argv、退出码和耗时。fmt、check、Clippy、MSRV 1.85.0 locked、deny offline、docs/specs/core-vocab/tests/skill 均 exit 0。deny 使用本地 advisory 缓存，不表示实时安全公告查询。docs 的首次运行因审查报告链接移动而失败，修正链接后的 [重跑原文](docs-rerun.stdout.txt) 和 [specs 原文](specs-rerun.stdout.txt) 均 exit 0；gate-results.json 保留首次失败。

- 全仓 Nextest `6e03fa98-0535-4d68-bc1f-f4412a609ac8`：647 passed、0 skipped、2 slow、无 LEAK，158.063 秒；[原文](nextest.stdout.txt)。
- T31 Nextest `27ffc52b-c5a0-472b-ac1e-07c262c3abb0`：22 passed、625 task-filtered；[原文](task.stdout.txt)。
- `dist plan --output-format=json`：exit 0；[JSON](dist-plan-r20.json)、[stderr](dist-plan-r20.stderr.txt)、[argv/exit](dist-plan-r20.metadata.json)，只核发布形状，不发布或证明四平台资产。

旧候选的三项同步/残留/写锁负控制均 exit 100，原文保留但不冒充当前输入通过。当前计数修复的 `+=` 反实现 exit 100，恢复源码 SHA 后通过当前候选全仓门禁；[元数据](unchecked-counter-increment.metadata.json)、[原文](unchecked-counter-increment.stdout.txt)。

Nextest >=0.9.145 修复了 macOS 子进程继承其他测试 capture pipe 的机制（[上游 changelog](https://nexte.st/changelog/)）；本次使用经官方 SHA 校验的隔离 0.9.146，保留全局版本与旧 LEAK 原文。新候选无 LEAK，不声称每次历史 LEAK 都已逐一定位。

优化 profile 的当前普通 clone 全仓 baseline `32ac9a76-3eeb-4e8e-a938-d7ed9b370680`：647 passed、0 skipped，70.381 秒；[原文](mutants/unmutated-workspace-durable.stdout.txt)、[元数据](mutants/unmutated-workspace-durable.metadata.json)。profile opt-level=1、debug=0，debug assertions/overflow checks 保持开启，具体编译参数和官方依据见 [变异闭包](mutants/closure.md)。它不替代默认门禁。

旧 642 源码基线原文固定在 [旧门禁目录](gate-source-6baee-before-public-contract-oracles/nextest.stdout.txt)。误用全局 Nextest 0.9.140 的工具拒绝原文在 [工具版本拒绝](environment-global-nextest-rejected/nextest.stdout.txt)，修正后完整重跑通过；没有跳过版本检查。

此前0aee临时输入的core分片与跨workspace复验已完成，runtime第一片在临时目录被环境清理前未产出完整结果；全部留在 [归档](mutants/superseded-ephemeral-input/status.json)，不计当前候选。新普通clone放在被Git忽略的`target/t31-validation/source`，按[新闭包](mutants/closure.md)重建完整baseline后从全部分片重跑。

## 完整变异与审查边界

[变异输入与执行方法](mutants/closure.md) 说明完整 2573 项清单、零起点分片与两阶段全量执行；所有 missed/timeout 均送 workspace，逐项处置未完成前保持 WIP。早期候选/错误分片保留为 superseded 或 INVALID，不计当前通过。

独立 Spec 和 Standards Reviewer 只读复核，不参与修复；[Spec 准备报告](spec-review-preparation.md) 含完整 R/O/N 矩阵，[Standards 准备报告](standards-review-preparation.md) 核工程、架构、价值与窗口 oracle。二者增量通过不等于 M1 通过；最终结论与逐项答复待完整结果。当前CLI集成测试的清理helper与示例Workbook来自workspace，独立`.crate`解包后的测试自包含性未验收；`cargo package --list`只核源码包文件集合，不能当包内`cargo test`通过。本轮二进制/源码可靠性验证仍按仓库及普通Git clone执行；如未来要求发布包内复测，需连同examples/workbooks fixture一起解决。

SIGKILL、exit70、sync 故障是不同证据，不能据此声称断电持久性通过。真实 Host/agent 质量、usage、四平台发布及 T16/T17 均不由这些离线测试代替。
