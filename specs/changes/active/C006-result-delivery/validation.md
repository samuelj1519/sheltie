# C006 验证

Candidate: `none`

状态：`active`。以下为预定 oracle；源入口、导出器、支持平台、真实使用与实施审阅均为 `not_run`。

## 1. 源与目标验收矩阵

| 要求 / 风险 | 合法例 | 拒绝例或故障例 | 归属 |
| --- | --- | --- | --- |
| 唯一结果选择 | succeeded/final/无待完成效果且选集非空 | 未完成、effects_pending、空选集、未知字段或格式，目标无成果目录 | T01 原语 / T02 真实链 |
| 精确 revision/key | 同一次结果 revision 的正式 Artifact | revision 不同、key 不存在、非法路径、草稿均拒绝 | T01 原语 / T02 真实链 |
| 实际源字节 | 同句柄 size/sha 正确，原字节 stdout | 替换、截断、增长、改写、非零退出、缺字节不发布 | T01 原语 / T02 真实链 |
| raw CLI | 二进制零字节/非 UTF-8 内容逐字节相同 | --json、--request-id、单独 artifact/revision 拒绝，不补换行 | T01 原语 / T02 真实链 |
| 格式与实际能力 | 同格式和读取能力、不要求 release 相同 | 旧或未知 mode 非零，私有暂存不得升级完成 | T01 原语 / T02 真实链 |
| 限额 | JSON 恰好 1 MiB；单文件32 MiB、总量256 MiB接受 | 超一、加法溢出或源实际输出超过声明拒绝 | T01 原语 / T02 真实链 |
| 授权父目录 | 已存在无链接目录，私有新对象 | 不存在、链接、管理根重叠、身份变化拒绝 | T01 原语 / T02 真实链 |
| 目标对象 | 独占普通文件、单硬链接、私有目录 | 软链、硬链、特殊文件、路径注入、名称碰撞拒绝 | T01 原语 / T02 真实链 |
| 全体发布 | 所有 bytes 与清单核验后一起出现 | 任一项错，最终目录不出现；不把暂存称完成 | T02 |
| 不覆盖 | 唯一目标名目录 NOREPLACE | 目标竞争创建，旧字节、inode、权限不变 | T01 原语 / T02 真实链 |
| 发布后未确认 | 移动和同步成功 complete | 移动后 sync/身份失败 publication_unconfirmed，不删除或回滚 | T01 原语 / T02 真实链 |
| 残留与重跑 | 每次重跑创建新副本 | 不复用旧暂存、不接管不明目录、不覆盖旧副本 | T02 |
| 跨设备 | 源目标不同设备仍逐字节复制 | 不通过源 rename/link/reflink 转移；载体不可用记 not_run | T01 原语 / T02 真实链 |
| 业务状态只读 | 导出前后 Work/request/audit 语义不变 | 不写回执、非final、不偷做效果恢复；SQLite控制文件另识别 | T01 原语 / T02 真实链 |
| 可编辑副本 | 发布字节核验且用户可编辑 | 后续编辑不改变 Work，不声称清单代表永久完整性 | T03 |

所有拒绝例核实际输出目录、源与根外哨兵字节、inode 和权限，不只核错误码。可重复竞争钩子仅放外部边界；不能 fake 直接返回成功。实际 platform NOREPLACE 失败阻断该平台支持。

开发包的 publish=false 和 package.metadata.dist.dist=false 必须在本地 dist plan 中证明没有 exporter 发布资产；workspace packages override 不能绕过排除。该检查不执行 release 构建、上传或外部发布。

## 2. 崩溃与竞态

必需 kill/sync 窗口：私有目录创建后、单文件接收中、文件同步后、全部成果读回前、清单写入中、暂存树同步后/移动前、整目录移动后/父同步前、父同步后/响应前。前六类没有完成目录；移动后可能有完整目录但缺确认。重跑只能新建副本，逐字节核已有对象不变。

竞态覆盖父路径打开后替换、暂存目录身份变化、文件软硬链接注入、最后移动前竞争目标名，以及两次独立导出。句柄操作不能重定向到替代路径，观察到异常即拒绝。测试不声称能隔离恶意同权限进程。支持平台必须真实运行，其他平台与缺失跨设备场景列明 not_run 及影响。

## 3. 真实副本价值

使用 C007 或真实任务已识别的副本用途。先固定相同结果、质量标准和记录口径，再记录手工与工具路径。找结果收益归 C004，复制与核对收益归 C006。

记录人工操作和核对分钟、选错文件/漏文件、总完成时间、失败时能否正确判断是否发布，以及用户是否需要副本。单次成功只证明该场景；无真实需求保持 not_run，不用示例输出或 kill fixture 制造价值。

## 4. 验证记录

采用后固定候选、工具链、features、平台、C004 DTO、fixtures、环境、两个 binary 摘要、过滤器与实际命令。复用须有相同闭包与原 run ID；执行模式不是 PASS。

| Requirement / risk | Mode | Input closure | Command / raw run ID | Result | Evidence |
| --- | --- | --- | --- | --- | --- |
| C004 结果与可信源读取 | not_run | 待 T01 固定 | 未执行 | not_run | 无 |
| 目录句柄与 NOREPLACE | not_run | 待支持平台固定 | 未执行 | not_run | 无 |
| 整份复制与 raw CLI | not_run | 待 T02 固定 | 未执行 | not_run | 无 |
| 崩溃、sync、竞争与跨设备 | not_run | 待 T02 固定 | 未执行 | not_run | 无 |
| Work 业务状态不变 | not_run | 待固定前后快照 | 未执行 | not_run | 无 |
| 首次读者与真实副本收益 | not_run | 待真实任务 | 未执行 | not_run | 无 |
| T01 原语 green 与 T02 行为 red | not_run | 待 T01 阶段闭包 | 未执行 | not_run | 无 |
| C006-M1 安全与实现准备 | not_run | 待骨架完整候选 | 未执行 | not_run | 无 |
| 最终工程检查与 C006-M2 | not_run | 待最终候选 | 未执行 | not_run | 无 |

缺输入、超时、中断和未跑平台逐项记录，不写 PASS。静态方案通过、本地工具构建、产品正确性、真实收益和发布状态分别结论。

## 阶段验证与测试基准

T01 复杂作者创建 `verification/commands.sh`、所有阶段 tests/fixtures 与 `experiments/runbook.md`，实际名称、过滤器、测试归属、counts 和预算在交接时固定。原语测试归 C006-T01、实际 green；新完整行为测试归 C006-T02、初始 ignore，以 task.sh 显式运行。future-red 至少一项真实 CLI 或持久消费者的行为断言失败；编译/环境错误、零测试或仅占位 panic 不算有效 red。有效 red 是准备证据，不是产品 PASS。已有正确 caller 保持 green，无需全红。

M1 核原语、普通回归、有效 red、骨架可编译且没有未完成正常入口。T02 开工基准是 M1 审定骨架或最新独立测试修订的完整 SHA，allow_test_changes=false；只能删除归本任务 ignore。测试/fixture/合同缺口交复杂作者新增明确修复任务，独立复核后固定新基准，不隐式继承旧批准。正式 feature 必须非零实际执行、全部通过且无本任务 ignore。

M2 核完整用户链与实际使用。对 M1 已审且闭包未变化的内容引用原 closure 和 run ID，不重复全套；新增调用/配置/fixture/效果或异常分支按影响补验。每项任务仍有短语义复核，原输出只保存一次。T03 是手册和实际使用，不以无 Rust 用例的 task.sh 验收。

## C006-T00 采用入口

开工完整候选 `8d27348d995a434ae3dbf8255e1110973dec1381`。仅归档C005/采用C006/当前索引与链接、采用及授权例外记录；无Rust/Cargo/fixture/合同变更。docs142、specs8/1active、tests774/204cards、diff exit0。[原文保真清单](evidence/c005-raw-preservation.json)证明C005全部51个evidence逐字节保留。真实复制需求/成本与平台实测未执行，不因开发采用记为PASS。
