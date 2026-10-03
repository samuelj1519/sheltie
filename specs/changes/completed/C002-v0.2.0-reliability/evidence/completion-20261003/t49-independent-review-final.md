# C002-T49 增量独立审查

结论：**通过（限定当前新增 oracle 与 F01–F03 修复）**。不作为 T49 完成、G02全部47处分或C002-M2验收。初审 `t49-independent-review.md` 需修改与原外层超时记录保留，不反写旧结果。

Reviewer：`Codex /root/oracle_review`，未参与设计、测试、生产实现或修正。只读审查代码/合同/原文，未构建、改仓库或编写 oracle；仅写报告。当前基准 `63b05ca8c4b6977677b01db957b550f83393926a`。snapshot_qualification SHA256 `50471c9c38e849c098653207f21a42c42b4fed32f6878b718917c7c83a680229`，workbook_txn SHA256 `f6b3c631bfae8d75fb196d81024f1c2ebc2b943d3876260e9a4e560047868c41`。源码仅测试追加，生产实现未改。

## F01–F03

F01 关闭：全部新 rendezvous 测试及 publication_tree 有 failpoint feature 门控。`t49-default-feature-check.txt` 实际默认配置编译完成；write_session/service既有unused warnings保留，不冒称默认Clippy -D warnings。该默认运行早于最后独立事实/新index测试追加；后续函数同样feature门控，静态确认，不假称它们重新执行默认配置。

F02 关闭（owner竞态）：stats 固定断言真实Start的WorkId、Active、total/blocked/approvals=0、outline visits1/attempts0、summary visits0/attempts0；list 固定真实WorkId、default、Active、outline#1。手写期望来自已采用two-step合同和真实Start身份；调用同一getter捕获的旧完整视图只作跨竞态不变性比较，不再作为业务事实的唯一来源。

五张表独立SELECT所有列，writer阶段仅手写允许目标requests.published从0→1。无writer的unpublished分支不允许该改变。works/workbooks全部路径、类型、bytes、mode与inode独立快照；仅真实Work writer恢复时可按既有RefreshStatusCard合同原子换该Work status-card inode，bytes/mode仍固定。unpublished分支不开放该例外。writer的pending允许完成元数据清理，目标owner消失单列，不把writer阶段排除pending的快照冒称该阶段全部pending精确不变。reader释放后五表与**含pending的完整业务树**均逐项相同。SQLite/WAL/SHM/lock控制载体不混入业务原件声明。

F03 关闭：workbook/stats/list三种reader各执行七条件，21个同步场景。合法真实writer完成、metadata逐字未变且published=true接受；owner消失而仍false拒绝；真实writer完成后intent_hash/reply_json/effects_json/work_id/at各单项漂移拒绝。JSON加尾空格保持载荷语义和解析形状，只改变原始immutable文本；避免仅靠解析失败检测。读线程错误明确STORE_CORRUPT/owner，五表与全树拒绝后不被修写。

正常writer是真实同id重放恢复和cleanup_pending；初始published=1→0仍是SQL重构rename后/mark前相容状态，不是实际COMMIT断点/持久崩溃资格。reader捕获owner stat后reached，主线程等待后disarm再恢复/清理，最后release；disarm不替代release。RendezvousWorker正常及断言unwind路径沿既有RAII释放/join，测试不承诺release写失败的严格墙钟上限或进程树隔离。

## 新观察窗口 oracle

- audit二次查询：真实add后在owner观察处停住，原audit已完成资格检查；只改单个work_id/revision/at，再释放。要求当前add审计不一致，排除第一次读取的早退；三项坏SQL记录和树原件不修写。
- Start/index后漂移：真实Start、真实另一Work的final目录、真实A→remove→B，共享捕获pending reference index后改一项effects。extra-effect、final与pending前缀/叶/UUID要求准确Start效果/pending拒绝；B借A的UUID要求引用索引拒绝。仅受控SQL观察条件，不声称引擎会合法改变immutable历史，也不推断任意外部写者安全。
- 新old-index/restored-effects：真实add后先把effect digest改为明确不同的合法64hex，让reader捕获该index；随后还原真实原effects，再释放。此时当前完整请求合法，stale index仍必须报pending引用索引不一致；独立五表和全树不变，新load成功是合法控制。不是调用被测helper构造期望或直接写成功。
- lifecycle坏snapshot id/version现在要求当前B的业务身份/效果资格诊断。三个pending lifecycle短样本变体仍拒绝，却错选旧R并报“最新生命周期是remove”；新判据区分**诊断来源**，不得记成这些变体接受非法对象或执行非法写入。audit_target一般拒例仍有前层intent摘要边界，不能归目标Remove OR等处分。

shape/lifecycle CLI仍使用原五表与业务树bytes/mode快照；lifecycle另核installed根inode。完整每对象inode证明来自新增runtime publication_tree，不能把CLI原helper外推为整树inode观察。

## 实际基线与短样本

`t49-owner-writer-oracle-corrected.txt` run `cb07c727-f806-4e73-b00f-3f95fe78280f`：owner 1函数/21场景PASS。`t49-query-race-corrected.txt` run `c3161250-9e58-46d6-b512-6c2b646a73cc`：audit/index两函数PASS。`t49-lifecycle-diagnostic-baseline.txt` run `ba566226-6cec-4346-8ce9-884bc2bc288c`：lifecycle PASS。最后补独立事实和restored-index的 `t49-independent-facts-baseline.txt` run `71131632-f8ed-48a2-b156-59fc04fdcc97`：2/2 PASS，347因filter未选。初次非法CLI参数、未列真实RefreshStatusCard inode变化的fixture失败原文保留，不记产品red。

`mutation-g02-query-sample/result.json` 实际exit2，197.585s，baseline Success1、CaughtMutant11、MissedMutant2，0timeout；该短样本不是PASS。逐原log区分：pending lifecycle3项只有diagnostic detection；Start effects四OR与二次audit四OR共8项实际接受坏观察状态，错误断言捕获。两项Missed是publication_for_row :785 final-path OR与:788 digest-root OR，仍需逐项精确处置，不改caught或equivalent。

短样本完成时source_unchanged=true，随后workbook_txn补独立事实和新restored-index。当前与该selection唯一源码差异为这一测试文件；原11/2只属于其冻结scratch输入，新baseline与它分列。不把样本结果赋给新整文件闭包。

原G02全组外层1200s截断的42/47（12Caught+30Missed）、两项build结束但test未跑、三项未派发仍见初审。完整47、稳定最终工程门禁/任务范围/提交/M2尚未由本报告关闭。用户范围继续为macOS aarch64全自动；其他平台和无人真实trial不转PASS。
