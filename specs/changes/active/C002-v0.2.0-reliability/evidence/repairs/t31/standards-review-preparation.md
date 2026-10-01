# T31 Standards 独立审查准备

状态：**准备审查，非 M1 最终结论**。dangling 历史文件修复的 task-local Standards 结论为 **PASS**；本报告覆盖的生产增量未发现剩余必改项。完整变异及存活体处置尚未结束，不能据此关闭 T31、R17、N13 或 M1。Linux 按用户授权保持 `not_run`。

Reviewer：`/root/standards_review`，未参与产品代码、测试或方案实施。本轮读取源码、合同、调用者与已有原始输出，未运行 Cargo 或产品测试；只负责本独立报告。

## 审查输入与依据

产品基线为 `b926789`。最早已审的生产增量和 642 项门禁对应普通 clone 临时提交 `6baee53376f8ba631d5c92777881ec82692576de`，输入与日志见[归档目录](gate-source-6baee-before-public-contract-oracles/candidate-input.txt)。增加公共合同测试后的旧候选 `2cfe8027cf60c3a2ba3eaa6ef5f92d09b8b21f6b` 有 645 项通过，其输入与日志见[另一归档目录](gate-source-2cfe-before-counter-integrity/candidate-input.txt)。两者均已被 R20 修复及新增测试取代，不可算入当前变异结论。

当前普通 clone 临时提交为 `da2bd13979df88dd987596d7ed726ef99bb89651`，固定新增 R20 测试与修复后的执行输入。证据记载了 147 个源码/fixture/配置/脚本文件及 34 个改动文件在复制时的 SHA 校验，见 [candidate-input.txt](candidate-input.txt)、[候选元数据](mutants/superseded-before-implementation-review-fixes/current-candidate.json)与[逐文件清单](mutants/superseded-before-implementation-review-fixes/copied-candidate.json)。后续 progress/validation 写入仍可能使实时工作树不同于这个固定输入，最终产品 commit 必须重新逐文件核对。临时提交不是产品任务提交。

临时负控制不作为候选，运行结束须恢复并核 SHA。新增测试和 opt1 profile 形成新执行输入，不能沿用旧候选的门禁或变异结果；执行方法见 [mutants/superseded-before-implementation-review-fixes/closure.md](mutants/superseded-before-implementation-review-fixes/closure.md)。

标准来源为 `specs/engineering.md` §2、§3、§5，`specs/constitution.md` 的 INV-1 至 INV-7，架构、storage/protocol 合同，以及 C002 repair-plan/repair-validation。本报告不审 C004–C008 proposed，也不将逐任务完成状态当作能力证据。

## 工程、架构与产品价值

| 范围 | 独立判断与当前真实调用关系 |
| --- | --- |
| 三 crate 分工 | core 负责纯解析、校验、状态转换和渲染，未见产品 I/O；runtime 负责受管文件、SQLite 和恢复；CLI 保留解析、协议输出与退出码。Workbook 的自然语言审核规则及测试中的模拟 worker 没有进入引擎选路或系统事实推断。 |
| 唯一事实源 | Work 状态、请求与审计仍由 SQLite 提供。目录、状态卡和历史文件是登记效果或投影。恢复通过私有 `RecoveryAccess` 连接两个真实 provider，没有从目录名或报告文本反向推进状态。 |
| 写入入口 | Home-only Service/Repo 构造不做 I/O；私有 Store 与 WriteSession 限制 RW 构造。Work、Workbook 和 self 的实际写 caller 保持锁与根身份检查；原始效果不能绕过 CheckedEffects 直接执行。 |
| 文件与冻结边界 | 受管路径采用词法约束、父目录 fd、no-follow 和对象身份复核；SafeFile/ManagedTree 贯通观察、封存、移动和删除。外部 Workbook 装入先核限额，复用同次捕获的图、资源、说明书和摘要。恢复核登记引用，不重新解释当前源文件。 |
| 生命周期与协议 | 共同恢复在 mark 前完成必要效果与最新状态卡。same-RID 未提交残留的定向准备失败发生在新请求登记前；普通 maintenance 仍由 CLI 在成功输出后报告 stderr 警告，不改变成功 JSON 或退出码。purge 保留根及同一 `.lock`，旧 Work 写入口不重建已清 Store。 |
| Rust 质量 | newtype、严格 DTO/Serde、Result 和明确错误类别支持边界校验。恢复统一、共享原子写原语及私有 Store 收回了先前重复职责和旁路。本轮未发现引入通用框架、兼容层、第二状态源或 unsafe 的必要性。 |

可靠性方案符合产品价值：幂等请求、冻结字节、恢复、准确提交状态和合法下一步，都是协调者反复承担的机械责任。引擎接管这些责任，Workbook 与人继续判断内容质量和审批。该判断支持方案方向，不证明真人使用质量、token 成本或宿主集成收益；这些仍需 T16 的真实证据。

权限位只降低误写风险。stat 与 unlink 等操作仍受已采用的 HomeLock 协作模型约束，不声称抵抗同账户任意不遵守锁的外部写者。SQLite 的 NOFOLLOW 也不等同目录 fd 锚定；已采用的控制文件例外、main/已有 WAL 字节不变和 schema 拒绝要求必须继续保留。

非阻断品味建议：单次 Workbook list 可构建并复用本次读取的审计索引，减少每行重复扫描全历史。应先证明查询成本，再局部改进；不需要长期缓存或另一份权威数据。本建议不是已证明的性能回退，也不阻断本批修复。

## dangling 历史文件修复

原缺陷是 `Path::exists()` 跟随 dangling symlink 后将异常叶判为缺失，普通原子 rename 随后覆盖该链接并错误恢复成功。现 `effects.rs` 先调用 `open_managed_optional`，以 no-follow 读取区分真正缺失与异常类型；缺失历史文件经 `write_new_atomic_file` 落位。

`fsx.rs` 复用原子写原语，将历史创建限定为 NOREPLACE，保留投影替换的既有语义。失败时尝试清理独占临时对象，已有普通叶或 dangling 叶不被替换。落位冲突归 `STORE_CORRUPT`，真正 chmod、sync、读取等 I/O 失败仍保留其 I/O 类别。

真实 CLI 的 dangling 变体保留原链接与 Store snapshot；fsx 用例检查已有普通叶、dangling 叶和临时残留。原反实现先失败，修复后通过：[红 run 172d76ee](dangling-history-red.stdout.txt)、[绿 run c32677e3](dangling-history-green.stdout.txt)。这项局部修复为 PASS，不等于完整 M1 通过。

## 窗口与 oracle 核对

| 范围 | 当前证据判断 |
| --- | --- |
| 真实终止 | exit70 和 SIGKILL 分开运行；kill 前等待精确 checkpoint，核真实 signal 9。Process 的 checkpoint/finish 各有 10 秒期限，提前 panic 的 Drop 执行 release、kill、wait。测试用 Cargo 注入本候选 binary，未在测试体内重建共享 CLI。 |
| 发布与跨类恢复 | Start/Add 的 owner、私有副本、COMMIT、rename、mark/cleanup 窗口保持独立；新增 fresh fixture 在旧请求 COMMIT 后直接由另一类写入口恢复，未先重放旧请求。Store 的 original/effects/published 与原文件 bytes 提供独立依据。 |
| 历史文件与封存 | Begin/Submit 核登记 content 的恢复字节、原响应、产物 bytes/只读权限和最新状态；历史 symlink、dangling symlink、hardlink、FIFO、PrepareAttempt 类型冲突均停止并保留对象。 |
| 删除与 self | Remove 覆盖移入、部分删除、末删无 marker、合法 marker 和 mark 窗口；结果不明时准确停止，不伪造证明。已移入恢复的源/目标父 sync 失败分别在删除前停止。update 用真实安装 binary/prev 恢复原字节；purge 等待者先到 failed try-lock 事件，再验证合法初始化与旧 Work 拒绝。 |
| 测试环境 | 并发测试先完整 spawn，再 barrier 放行和 join。临时目录保留原根 fd，仅 chmod 目录；外部目录 symlink 和文件 hardlink 的 bytes/mode 反例保留。late WAL/SHM 的类型、nlink、WAL 零长度及异常保留有单条件边界用例。 |

当前未发现调用生产 helper 生成同一预期值的假成功 oracle，也没有因迁移删除故障窗口或原 Owner。SQL snapshot 保留测试以提交时登记值核历史一致性；内容正确性仍依赖 core 的直接合同例及独立字节向量，不能由 snapshot 自洽单独证明。

最终矩阵应将「mark 前完成最新物理状态卡」关联到 [T25 原始证据](../t25/README.md)及 `workbook_write_recovers_work_status_card`、`workbook_remove_recovers_latest_work_status_card` 的卡字节/Store oracle。新跨类例中的 `work status` JSON 证明当前事实视图，不能单独证明物理卡的完成顺序。最终 R/O/N 矩阵仍需逐项列真实 consumer、单条件反例和同候选原始 run，不以本表合并代替。

## 已核证据与未完成门槛

已读取旧候选 6baee533 的 T31 run `8ebd2e2f-96cf-4df1-b631-83f7ea756a1e` 的 17/17 与 workspace run `7c61730e-2eb5-4ea5-9642-1c8113b86e80` 的 642/642、0 skipped；见归档的[任务原文](gate-source-6baee-before-public-contract-oracles/task.stdout.txt)、[全仓原文](gate-source-6baee-before-public-contract-oracles/nextest.stdout.txt)。旧候选 2cfe8027 的 workspace run `f6318505-5e6f-4ed8-9c38-dffca69476c4` 为 645/645，见[归档原文](gate-source-2cfe-before-counter-integrity/nextest.stdout.txt)。这些结果只证明各自输入。[dist plan 元数据](dist-plan.metadata.json)记录的候选仍是 6baee533，只支持当时本地发布形状，不证明当前候选的四平台资产或发布执行。

新增 `core/tests/persisted_contracts.rs` 的 task-local Standards 结论为 PASS。元数据期望来自手写 manifest/Flow 字面值；合法 WorkState 由纯 core 的 Start、Begin、Fail、Cancel、Submit、Approve 产生，拒绝例先核合法基线，再只改目标字段或集合。两个 Gate 正例分别保留「另一 node 的相同 occurrence」和「同 node 的旧 occurrence」，避免过严校验。Fixture 仅复用公开的 graph/manifest/instructions 及命令 runner，未修改 testkit 或开放私有 API。

HostRequire.source 使用合法 URL `https://example.com/` 加 492 个 ASCII 字节，正好 512 字节；拒绝例仅 append 1 字节到 513，version 仍为 32/33 字节配对。此次 URL 修订后 3/3 的原始 run 为 `4332d69c-94f4-4442-9ddd-725a9d7509a5`，见 [public-contract-development-4.stdout.txt](public-contract-development-4.stdout.txt)。此前 UTF-8 夹具的审查与开发失败保留为历史，不替代新合法 URL 用例。

当前 da2bd139 输入的默认 profile workspace run `6e03fa98-0535-4d68-bc1f-f4412a609ac8` 为 647/647、0 skipped，见[当前默认原文](nextest.stdout.txt)。同一普通 clone、opt1 profile 的 baseline run `32ac9a76-3eeb-4e8e-a938-d7ed9b370680` 为 647/647、0 skipped，见[当前优化原文](mutants/superseded-before-implementation-review-fixes/unmutated-workspace-durable.stdout.txt)。优化 profile 不替代默认公共门禁。误选全局 Nextest 0.9.140 的最低版本拒绝保留在 `environment-global-nextest-rejected/`，是工具输入不符，不能计作产品失败或测试通过。

R20 的独立 [blocked_count 探针](blocked-count-independent-probe.json) 从真实 Gate 状态仅把计数改为 `u32::MAX`，旧状态查询接受、批准命令溢出 panic；该探针的二进制身份已记录，不能单独证明整个当前候选。修复只加入 `approvals.len() + 当前是否 Blocked <= blocked_count <= attempts.len() + approvals.len()` 的必要界，保留显式累计事实，不从审计或报告重算。core 的 Submit/Fail/Approve 计数增量用 `checked_add`，失败不改变传入状态；Store 装入和提交都把非法持久状态映射为 `STORE_CORRUPT`。真实 CLI 的 MAX、零值、上界加一分别核 status/stats/list/approve 的结构化拒绝、原 SQL/文件与无新请求。合法取消、Gate 批准后推进和重访仍满足该界。本增量的 Standards 结论为 task-local PASS；红绿和后续完整门禁分别见 [blocked-count-red.stdout.txt](blocked-count-red.stdout.txt)、[blocked-count-green.stdout.txt](blocked-count-green.stdout.txt)与当前 647 项 run。必要界不是任意持久值篡改的完整检测承诺。

三条独立负控制删除 same-RID 准备、删除父 sync 或内核锁后均被真实测试捕获；原始失败和恢复 SHA 见对应 metadata/stdout。最新候选的最终证据应核这些控制是否已绑定最终输入，不能无条件复用前一候选。新 Nextest 没有报告 LEAK，不足以逐一证明所有历史 LEAK 的根因。

完整清单为 2573：core 709、runtime 1864。两阶段全量方法可接受：逐个运行所属 crate 的完整测试，全部 missed 和不明 timeout 再逐 ID 运行全 workspace；独立直接 oracle 已 caught 的项目注明阶段，unviable 与 timeout 不算测试 PASS。优化 test profile 采用 opt-level=1、debug=0，并显式保留 debug assertions 与 overflow checks；不能减少 oracle、fixture 或清单。默认 profile 门禁另行保留。旧候选及默认 profile 变异已归档，当前 da2bd139 输入须完整重跑。

当前完整执行和逐项存活体处置仍 WIP。必须核清单并集无漏无重、各阶段候选/feature/profile/工具一致，以及变异副本实际使用自己的 binary。正常候选的 `--list` 启动超时应按环境或未判定结果处理，不能计为 caught 或等价变异。

对[执行脚本](mutants/superseded-before-implementation-review-fixes/pipeline.py)的首轮审查发现输入漂移误复用、不完整片覆盖旧 stdout、共享输出并发三项风险。当前 v6 脚本已加入单实例 flock、clone HEAD/dirty 与 147 项逐文件 SHA 检查、profile/timeout/工具及自身脚本指纹、已有闭包不同时拒覆盖、已存在不完整片目录的失败停止、未知结果拒绝及完整片之后的进度写入；这些修改已只读复核。147 项执行输入不含运行中更新的 progress/validation；[34 项快照](mutants/superseded-before-implementation-review-fixes/copied-candidate.json)仅记 clone 创建时的完整改动，不能当作持续与工作树一致的声明。

运行中的首轮程序仍是归档的 [pipeline-at-launch.py](mutants/superseded-before-implementation-review-fixes/pipeline-at-launch.py)（v5），不能把 v6 的保护追认到其原始结果；该轮须逐片核实际 argv、baseline、处理数、状态、原始 SHA 及所选测试是否真正失败。若 v5 中断，保存该轮已生成的 raw，并按同输入的剩余精确 ID 继续，不把部分结果计为完成。

v6 的完成片复用又补上四类 summary 白名单、分类计数重算、`outcomes.json` 与 `mutants.json` 的 SHA 比对；若已有 `raw-manifest.json`，还逐字节对照归档摘要。这关闭了事后改写 `MissedMutant` 分类而漏送第二阶段的误计路径。v6 尚未用于运行中的首轮；其静态检查通过，不能把保护追认给 v5。首轮仍须按归档 v5 的实际 raw、全清单对账及存活体处置判定，不能仅凭进度文件宣称 2573 项已全部验证。

最终关闭还需要固定产品 commit 与输入、核公共门禁及 MSRV/deny/dist 的同候选依据、完成存活处置，并由未参与实施的 Reviewer 作 M1 独立结论。SIGKILL、exit70 与 sync 故障分别证明不同边界，不替代断电持久性实测。Linux `not_run`、T16 真人/Host/usage 和 T17 发布均保持原边界。
