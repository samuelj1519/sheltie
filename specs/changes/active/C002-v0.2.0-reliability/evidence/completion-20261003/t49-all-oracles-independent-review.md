# C002-T49 全部新增 oracle 独立审查

结论：**通过（限定 oracle、期望来源与消费者阶段）**。当前10条新增测试未发现剩余必须修改项。T49进度、35个动态ID、12项限定静态处分、完整稳定工程门禁与M2均需后续实际证据，本报告不预判PASS。初审和此前短样本原文保留。

Reviewer：`Codex /root/oracle_review`，未编写测试或生产实现。只读核合同、准确caller/停点、原输出、简化前后diff和最终文件；未修改仓库或重跑共享target。基准 `63b05ca8c4b6977677b01db957b550f83393926a`。最终runtime workbook_txn SHA256 `d57879b54fe1ccf7ac3dafe49ac3121f572d9c19e4a0df253480d67fdaeeae13`；CLI snapshot_qualification SHA256 `0f78ee613a5677a855a3d84dd993ffa463462e11cc0ee8be5db459335fc3d268`。

## 最终范围与简化

8条runtime integration、2条CLI integration，共10条新增test函数；循环内多个条件不按函数数冒称端到端场景数。两文件原内容与基准逐字相同，新增范围只在T49；无生产源码/AST/依赖/协议行为扩展。

独立对照 `/private/tmp/{workbook_txn,snapshot_qualification}.rs.t49-presimplify` 与最终文件：PublicationTree type alias、pending排除与writer树判据小helper、手写digest私有helper提取、SQL/JSON展开、non-Workbook audit真实producer移到唯一分支均保留原条件、握手顺序和断言。status-card inode例外仍只针对真实writer恢复且目标是同一Work卡，bytes/mode不放宽。root之后增加copy source bytes/inode和旧完整表断言是新的oracle增强，不能冒称纯整理；本审查另逐处核。

## 四组新增真实消费者

1. **old index / restored effects**：真实add，合法完整effect先记录。独立SQL只改digest为不同合法64hex，让reader在pending_after_reference_index捕获该旧index，再还原真实完整effect后释放。要求STORE_CORRUPT/索引不一致，五表及全部业务树不修写；fresh load接受是合法控制。属于受控外部记录变化，不声称合法引擎会修改immutable历史。
2. **物理manifest身份**：真实Repo.add并合法load。私有independent_workbook_digest按storage §5.1手写 `sheltie-workbook-digest/v2\0` domain、BE64数量/路径长度/内容长度、原bytes；Vec<u8>键按规范UTF8路径字节排序，空目录不入流。只使用普通文件独立读取和底层SHA256原语，不调用生产digest/helper生成期望。先对真实add摘要作合法校验，再把物理manifest有效id或version改错，workbooks行、snapshot、effect三份digest同步到该独立帧摘要。其他登记身份不变，故不会靠前层checksum错误提前拒绝。最终load必须在实体manifest与Store行身份处拒绝；原SQL/全树bytes/mode/inode不修写。此helper只处理本可信普通文件fixture的帧，不代替链接/大小上限/Unicode路径的通用资格测试。
3. **physical missing manifest**：真实Work.start，published=true保留final；false场景把真实完整目录移回所属pending并SQL回写false，owner和全部原记录保持合法。先合法stats，随后在真实存在的frozen目录只删除workbook.toml。两个状态都STORE_CORRUPT，但期待准确NotFound转换来源（已发布文件缺失 / pending缺声明），五表和全树不恢复、不重造。属于SQL重构的阶段fixture，不冒称真实崩溃窗口；其相关突变若仅改变最终诊断须单列diagnostic detection，不能宣传为“突变接受了坏Work”。
4. **copy身份复核**：真实已登记two-step；Start先停 `pending_owner_synced_before_payload`。实际service调用顺序已核：锁内Workbook资格完成 → owner/container持久化停点 → seq分配 → copy。主线程随后改配manifest-specific `external_tree_after_stat`，释放第一点；第二点仅在本次source-copy读取该manifest时触发。此后以同路径、相同长度写有效但不同id/version，source inode最后独立核保持。复制新bytes后必须报copied manifest身份错误，早于笼统digest错误。拒绝后source保留人为changed bytes、inode不被修写；完整workbooks/works/requests/audit旧行相等，works最终目录空。work_sequence及自有staging允许留下，不把这次锁后并发变化误写成锁前确定性拒绝或零控制I/O。

两同步点由RendezvousWorker持有release路径；主线程提前失败/10秒就绪预算触发panic会沿Drop释放两点并join，临时同步目录在guard后才销毁。正常finish也释放/join/disarm。没有新增生产观察配置，沿既有failpoint feature；不承诺release文件写失败的严格墙钟回收或进程树隔离。

copy最初停点过早导致WorkbookTampered的失败原文保留。修正到锁内资格后的owner点，再到source-copy点，最后冻结基线确实命中copied identity诊断；原失败不记产品red。id与version在OR→AND突变下可能仍被后续digest拒绝，此时只能登记准确diagnostic检测，不能假称非法副本被提交。

## 其余既有新增组

F01 cfg门控、F02独立stats/list事实及五表writer允许集/全树对象边界、F03三reader×七同窗口结果继续按 `t49-independent-review-final.md` 通过。生命周期加入真实Work Start产生的非Workbook audit值；它被放到当前B的Workbook audit后必须报当前B类型资格错误，不能跳过B借旧R。stale-row用例真实remove后手动重新插入旧登记行，并只改该最新remove snapshot request_id，必须先准确核最新remove资格；是受控损坏记录与diagnostic oracle，不是真实删除后引擎自行复活行。

三个pending lifecycle方向、非Workbook audit分类、latest-remove、copy/missing-manifest的准确错误来源若检测的是“仍拒绝但诊断阶段错”，均保持诊断检测与实际坏状态接受分开。audit_target原case的前层intent摘要拒绝边界继续列明，不能归为对应publication选择guard已被动态触达。

CLI业务树helper仍核paths/type/bytes/mode；lifecycle另核installed根inode。每个对象inode的完整证明来自runtime PublicationTree；不外推CLI旧helper观察范围。

## 完成基线与后续执行边界

最终统一冻结oracle基线 `t49-frozen-oracle-baseline.txt`：Nextest run `0ab9e52e-288f-4cab-8a1c-89b5967a934d`，10/10 PASS，564项因filter未选，test阶段5.293s。原文SHA256 `5e6107da3ab91da706cd7c5345cccdca12954210cc819b011684019247d478e8`，包含最终简化和copy保全增强；不是复用修改前单项结果。

此前physical manifest `8f3f9872...`、missing manifest `946dabef...`、corrected copy `d3eeab28...`、latest-remove/非Workbook audit `0dd01ece...` 原文各自保留输入和结果，不取代冻结10条整体基线。此前G02全组42/47截断，以及13短样本11Caught/2Missed和诊断检测分类仍保留，不反写旧Missed/timeout。

计划中的12项限定静态与35动态精确ID须分开审计；12/12/11三个执行批600秒预算不等于已执行或通过。完整工程门禁和任务范围/提交检查当前不由本报告关闭；后续按真实原文与输入closure核验，不以oracle审查PASS或10条基线宣告产品/全部215/M2完成。
