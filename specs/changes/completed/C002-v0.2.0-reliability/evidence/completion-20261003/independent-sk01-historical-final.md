历史补审已执行。**旧候选最终 Spec 结论：需修改；不批准旧输入。** 固定候选 `e3eea899877165f8573befee3774555598ec92bd`。本次完成原 SK01 的 T34 最后增量 Spec 及原 M1 整体最终判断；审查者 `/root/completion_spec_review` 未参与旧源码、规格、修复或 oracle 编写。本轮按固定候选的原上游合同/工程规范只读审查，没有重跑历史699项或变异，也没有覆盖旧暂停原件。

`review_execution_complete=true`；`old_candidate_final_spec_approval=false`。全部原文件 Git blob/字节数/SHA256 见 `independent-sk01-historical-inputs.json`。此处行号均指 e3eea89 中的文件，复演使用 `git show e3eea899877165f8573befee3774555598ec92bd:<path>`，不是当前工作区行号。

**旧候选的三条明确合同缺口。**

| 历史 finding | 原依据与真实调用链 | 对旧输入的判断 | 后续现行修复 |
| --- | --- | --- | --- |
| F38-03，P1：重复Flow id提交后阻断 | 原 workbook合同:70 明确Flow id在Workbook内唯一，:21要求提交前完整校验。workbook_repo.rs:876–910 的load_tree逐Flow解析/编译，但没有跨Flow去重；add:223装入后收集重复id，:285先commit，然后:289恢复调用load_checked_request，:386–400才拒绝重复flows。 | 两份不同合法路径/相同Flow id的Workbook可先登记业务行/request/audit，再因自身快照flows重复变成EFFECT_PENDING。恢复重新遇到同一错误；新写先恢复旧请求也被阻断。它违反提交前校验及GF-17/GF-31；不能因后端严格恢复正确拒绝就称初次add正确。旧M1不批准。 | T39 `84bafa37f57ac2fda6aa5c223f6930f4f0882a87` 在load_tree最终私有副本、COMMIT前跨Flow去重。坏旧Store保留，不自动迁移或清空。 |
| F38-02，P2：完整载荷嵌套未知字段接受 | 原engineering.md:39要求serde结构拒未知字段；原architecture序列化边界采用同一规则。ids.rs:295–300的AttemptId无deny；state.rs:242–249的WorkStatus无deny。store/read.rs:434完整WorkState解码不能由外层:278的deny替内层拒绝。PersistedResponse.reply/next及Command也持有AttemptId。 | 只增AttemptId或WorkStatus内部未知字段仍会被派生解码忽略，合法业务值随后照常通过。此问题影响当前state的完整装入，也影响T34所核原Reply/audit资格；“外层严格+业务身份正确”不等于完整载荷合格。T34最终增量不能无条件通过。snapshot.data的canonical_status有独立规范表示校验，不把该边界错误扩大为所有data状态都接受未知字段。 | 同一T39给AttemptId与WorkStatus加完整deny，并补合法形状/未知字段及真实CLI state/历史Reply/NextOp/audit拒绝链。现行schema4的number与replacement是之后C005合同，不反套旧schema2的retry。 |
| F38-01，P2：24小时tmp维护没有caller | 原storage.md:172要求写操作清理tmp中mtime超过24小时的条目、不得跟随链接或用于pending。固定旧生产树没有读取mtime/遍历过期tmp的caller；selfmgmt.rs:175只清当次自有tmp；CLI dispatch:67–75成功写后只cleanup_pending。 | 一次中断留下的下载/解包/store-init tmp不会被后来写操作按年龄维护，残留可持续占用空间。不能由“可手工清理”或只清当次tmp证明满足已写合同。旧M1不批准。该项为静态完整caller核验，本次未把它说成新动态执行。 | 同一T39在成功CLI写后持既有锁接Home::cleanup_tmp→ManagedFs::cleanup_expired_tmp，只维护tmp；其时点合同也明确错误stderr告警、不改成功JSON。 |

上表通过固定旧源码与原合同直接核验，不只是引用后来findings状态。T39的完整diff也已只读复核：三条实现修复分别命中上述原落点；修复提交证明当前变化，不改变旧e3eea89字节或旧run结果。

**T34 原请求范围的最后增量判断。**

原SK01原文从固定归档 `e54dcd41d8f1e186007b62b47583063cb19a4b66` 的 `.../evidence/m1-2026-10-01/safety-skips.json` 读取。原范围为快照业务绑定、checked original/pending_original、Add发布目标、历史状态及节点requires。逐项静态复核如下：

- R22的业务身份分层正确：remove从唯一audit target与完整RemovedSnapshotData核id/version；add除metadata/audit/DTO还核提交PublishDir的final/owner/digest。workbook_repo.rs:478完成add target资格后，:483才构造original；:514–539完整解码/闭包验证effect，效果错误仍携带已核original。Work侧service.rs:975完整业务绑定后，:983才构造original，整组effect另核。recovery.rs:258–277把本人original和旧A的pending_original归属正确分开。该机制不读自然语言，不新增业务判断或第二状态。
- R23在旧schema2下正确使用原Attempt.retry与冻结max_retries。core/decide.rs:507–534不以当前visits或后来WorkStatus替换历史状态，submit/gate只接受可证明的冻结图状态约束；runtime另核对应Attempt存在且状态为Failed/Succeeded。后来C005引入superseded/number后改成failed前缀，是新语义下的必要变化，不能倒判旧retry逻辑错误。
- R24核Begin的reply与data一致只是第一层；service.rs:1223–1229还核真实Attempt、固定paths/inputs/outputs及冻结graph.node_requires。共同伪造reply/data中资源不能越过冻结节点比较。
- 但是F38-02使完整state、历史Reply/NextOp/audit的嵌套载荷不满足原严格解码义务；业务条件成立不能授权忽略未知字段。因此原T34最终增量结论为**需修改**。它不是否认R22–R24主修复条件，而是拒绝把资格基础不完整的旧候选作为最终闭包批准。

原engineering.md:39的“所有serde结构”也没有给effects.rs:403–414私有PublicationTarget投影写明例外；完整effects确在任何效果动作前严格验证，投影本身不是完整payload入口。原代码与宽泛工程表述应勘误，不应为了给投影加deny而丢合法效果错误original。后续M1文档提交 `02046665faae3d071446b5a6e5fa5a8d45797b01` 澄清完整载荷/只读身份投影；当前T46进一步精确区分冻结读取与效果动作。旧e3eea89的原规范没有后来明确的“Command/data先于冻结读”句子，本报告不把T46新表述倒用为唯一旧候选否决依据；上述三项已有当时明确合同足够形成需修改判断。

**原M1整体最终判断与当前修复分开。**

旧plan §4要求固定候选全链核不变式、合同、真实caller与完整窗口/存活项处置；task done或699测试数不能代替最终Spec。三条原合同缺口已足以否决旧候选整体最终批准。SK02原215项执行缺失及原平台边界继续保留，本次只完成Spec审核，不执行它们，也不将原授权例外改成完整安全/变异通过。原宪章INV-1到INV-7及三crate方向未因这些修复引入新的业务判断、core I/O或宿主安装权力；这个限定静态结论不抵消具体缺口。

当前修复路径：T39的三项修复在当前源码仍存在；T46 `06e5a9108be9afdfb9b5b68fc112ad08a600af8d` 已把当前Command/data纯严格解码移至冻结读前，并经独立正反例、effects资格与843项稳定输入回归。当前关键Spec PASS仍引用 `independent-t46-review.md` 与 `independent-environment-review.md` 的确切范围/run，不能写成旧e3eea89通过。

因此可以如实记录“SK01此次历史补审已执行，旧候选结论需修改；问题已由后续具体提交在当前候选修复，当前受审合同另行通过”。不能记录“旧输入最终批准已补齐”，也不能改写2026-10-01原平台暂停、原final_spec_approval=false或原SK02未执行。审核任务完成与旧输入批准是两个独立事实。
