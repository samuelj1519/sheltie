# C006 验证

Candidate: `none`

状态：`active`。T01安全原语与阶段准备已验证；正常raw/export入口、真实收益及最终验收尚未完成。下表前半是计划oracle，执行事实见阶段记录。

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

## T01 原语与独立修复（待正式骨架）

当前全部37项原语实际green，run `ca409eaa-f821-474e-a5a4-5f00f07c9fca`；Source/Target/Report/目录前检原件保存。macOS原生目标发布、权限、整目录NOREPLACE、sync真实probe与17 Target用例通过，不当断电或跨设备保证。Source首次14run中的LEAK unknown，以及typed新测试诊断过约束的FAIL+LEAK原件保留，后续green不覆盖它们。

独立元数据反例原本接受非法版本、0Occurrence、混终点来源及succeeded多reason字段。shared model完整当前版本规则、正号/同binder、typed strict status presence/duplicate及portableleaf已修，真实Source::result caller8green。独立copy曾在反斜线leaf时创建staging后拒绝；元数据前检现在同Target，旧red不删除。目录授权先于Source启动的旧order red/新control+6拒绝例已实跑，library4green。

初次future19run2PASS/17FAIL，均是公共尚未开放路径；其中opaque-key新fixture错误用了受ID限制的outputs名、后又漏resource根路径，两个前提失败属于作者diagnostic，不是有效feature red。修为实际合法的terminal输入槽+资源文件后，单case已走到raw参数未知的真实red；后续完整future矩阵原件另保存，不改变key或产品规则。

初次完整工程run `10dba84f-91ac-4ebc-8c46-be97fbdc1de7` 为811/811 PASS（2slow、1LEAK unknown attempt_begin_returns_brief_path_that_exists），19phaseignore；fmt/check/clippy/cachedDeny/docs/spec/tests通过。运行期间仅修未启用的future key fixture和共享测试helper的错误诊断文本，Rust生产/源原语/Target/Report没有变化；原run不声称最新whole闭包一样，最终骨架将以修后输入完成必要验证和正式review。

本机cargo-dist未安装，原dist plan exit101无执行，stderr保留；Cargo metadata实际证明publish=[]、metadata.dist.dist=false、无runtime依赖/无workspace packages override。与[一手cargo-dist配置说明](https://github.com/axodotdev/cargo-dist/blob/main/CHANGELOG.md)一致，配置证明不是实际asset plan，后者按授权延期不记PASS。原nextest0.9.145仍exit92/not_run，0.9.140override是实际执行器。

## T01 正式冻结

开工完整基准 `78e6af69a5454e27325e4183d1167c2c3769c824`。最终[完整工程门禁](evidence/t01/gates-final-input.txt) exit0，run `8f9707f6-dd07-4fd1-a0e4-fb0b08cf5470`：811/811 PASS、2slow、19阶段ignore、本run无LEAK。fmt/check/clippy、缓存deny、docs145/specs8/1active/tests830/260cards通过。[Rust1.85 locked全targets/features检查](evidence/t01/msrv-check.txt) exit0。37原语run `ca409eaa-f821-474e-a5a4-5f00f07c9fca` 全部PASS。

最终[有效future-red](evidence/t01/future-red-corrected-fixture.txt) run `5e293231-4664-4685-9b42-4803ceb3f7f6`，19项实际执行：2PASS/17FAIL，exit100；所有前提fixture合法，失败来自未开放公共参数。之前两个错误fixture诊断不算有效red；不删除或替换原文。

[冻结consumer闭包](evidence/t01/input-closure-final.txt) `6712a54ead3b8aa1245227e17cc7beb421638729ec6bbcf4a660910e71751aae`，295files，域sheltie-consumer-input/v1；排除evidence/progress/review/validation及生成诊断snap.new。冻结后仅改阶段状态/验证文字，另检查治理，不把它们宣称成相同whole hash。正常raw/export参数尚未公开；T02只开放两份binary参数和分发，库内高风险政策已全部交付。

macOS arm64原生API/37原语通过；跨设备载体、其他OS、真实用途/用户/成本、cargo-dist实际asset plan、nextest要求版本0.9.145及在线fresh advisory仍授权延期。cached deny使用未改policy的独立db-path配置和RustSec `117edb3bed98e9be112f277b7615eea3252e7c43` 缓存，不能称最新在线审查。此前LEAK、FAIL及中间闭包保留各自因果边界。

T01全cached diff检查exit2，仅[清单](evidence/t01/diff-check.json)内不可变工具原文尾空格/空行；保留原字节。精确排除这11份已知raw后的作者文件检查exit0，未扩大忽略范围。

## C006-M1 冻结交接

独立阶段PASS，candidate `6e60faacbe039cd21c04e5aa4d514ea5765e2ba3`。精确[输入manifest](evidence/m1/t01-input-manifest.json)已由Reviewer从git candidate独立复算，代码与oracle没有修改。M1新增的是记录/白名单与状态，单独治理检查；T02冻结代码/测试完整基准为该T01SHA，允许范围仍仅main/CLI和删除19ignore。
