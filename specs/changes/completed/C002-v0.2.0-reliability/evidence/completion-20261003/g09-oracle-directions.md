# G09 独立期望准备（15 个精确对应 ID）

只读当前源码、准确突变 diff、合同及现有测试入口；未编写/运行 oracle，未改 tracked。仅 macOS aarch64。JSON 保存 15 个旧/当前 ID、每项原 diff SHA、来源 SHA、caller 方向和静态边界。库存行号只作归档身份，当前插入 observer 后的行号以具体函数为准。

| 组 | 数量 | 共享能力与真实入口 |
|---|---:|---|
| W1 pending 登记闭包 | 4 | PendingReferenceIndex→cleanup/list/verify；效果删除及 Workbook 定位 |
| W2 owner 持久化顺序 | 1 | stage_pending→sync_dir_locked→container/payload |
| W3 新文件失败清理 | 4 | write_new_observed_unlocked 的真实写入失败分支 |
| W4 清理资格、告警与准备 | 5 | before_write→prepare_new_request；cleanup_pending/成功维护 |
| W5 恢复分派 | 1 | before_write 的 cached current 与逐一 pending request |

## W1：一份原始登记事实，整体索引先拒绝

`pending_internal_id` 的全部生产 caller 是 `PendingReferenceIndex::from_rows`、`effects::delete_dir`、`WorkbookRepo::publication_for_row`。后两类消费者的 CheckedEffects producer 已经完整核 pending 的 len/prefix/leaf/UUID；不能因此处分三个突变，因为全局 Index 的 producer 直接解 persisted effects，在本函数前只核 owner/final/digest，并未核 pending 形状。该路径覆盖 published 记录，任何坏登记必须让整个索引拒绝，不能据错误索引删除独立的合法未提交孤儿。

共享正例：实际 add/remove 产生的一份 pending UUID/owner 与单个登记，合法孤儿最终可清。负例仅改变一个 pending 事实：`other/<uuid>/payload`、`pending/<uuid>/other`、`pending/not-a-uuid/payload`、`pending/<uuid>/payload/extra`，并补过短 `pending`。用真实行保存后修改 effects_json，保留其他字段及一个合法邻居孤儿，调用 `cleanup_pending` 或全局 list。原实现必须 STORE_CORRUPT、无 panic、零孤儿删改，Store 行及孤儿完整 bytes/modes/inode 保持。

四谓词 A=len不为3、B=prefix错误、C=leaf错误、D=UUID错误。三个 connector 依次成为 `(A&&B)||C||D`、`A||(B&&C)||D`、`A||B||(C&&D)`。尾段事实可暴露首项，单独 prefix/leaf/UUID 可以共享暴露其余项；过短路径还可能因短路被破坏导致越界 panic，不能以“进程失败”统称正确拒绝。无须新增 observer。

from_rows :116 的限定静态候选见后文。重复目录效果不应编成镜像 guard 测试：由独立请求/内部 id 唯一关系核 whole-index 拒绝即可，且保留原诊断与成本差异。

## W2：owner 目录 fsync 的真实停止顺序

whole-crates caller 搜索：`sync_dir_locked` 唯一生产调用是 `service::stage_pending`；Work start/Workbook add/remove 都经该函数。流程为持锁建立 pending→CREATE|EXCL owner→write_all+file.sync_all→pending 目录 fsync→container mkdir→（add/start）payload mkdir。`pending_owner_synced_before_payload` 已在 container 创建后，不是本突变的 fsync 窗口。

正例走三种实际 stage/请求，核 canonical owner 与 op、只有规定 payload。拒绝例控制该 parent-directory fsync 的外部 I/O 失败：原实现允许留下 exact owner，但在创建 container/payload、任何新业务/audit/request 登记之前返回 IO；整个 sync 函数→Ok 可以继续 staging。优先使用 root-scoped sync_error 设施，若当前无精确 hook，作者可加一个窄的真实必需 sync 边界，不添加新的平台框架。有效锁/目录的 primitive 正反例可补接口资格，但不能代替 stage 顺序判据。不要在父 fsync 之后注入错误，或把后续 mkdir 的另一 fsync当作前序义务等价；这里承诺是操作顺序，不把进程实验写成断电物理持久性证明。

## W3：只删除本次创建对象的失败补偿

`unlink_created_at` 只有一个 caller：`write_new_observed_unlocked` 中 `write_all(bytes).and_then(file.sync_all)` 的错误分支。created metadata 来自真实 CREATE|EXCL FD，在失败前捕获；失败时该 FD 仍持有。成功后 `new_file_before_identity_check` 永远不会进入这条补偿，不能以这个成功观察点验证失败清理。

共享能力正例包含正常成功创建的精确字节，以及在真实写入/同步失败时、名称仍绑定本次 A 的补偿：仅 A 被 unlink，原写入 IO 返回，父目录同步。整函数→Ok 会留下部分 A；两个 !=→== 会错误拒绝同一 A、返回 RecoveryRequired并留下原件。这三个 ID 可由同一失败条件与独立 inventory/error oracle 覆盖。

AND 的独立拒绝例：在同一失败路径、进入 unlink_created_at 前，将仍被 FD 持有的部分 A 改名保存，原 leaf 建同设备普通单链接 B（明确不同 inode），B 的 bytes 手写并独立核定。原实现必须保留 B/A，返回 RecoveryRequired；mutant 可误删 B 并只返回原 IO。`check_regular_stat` 通过不等于 inode 相同，不能套前层 type/nlink 守卫。

当前缺这个失败路径 observer。作者只需失败已发生且元数据已捕获后的窄 rendezvous；进入失败路径须是外部 write/sync 故障模型或独立确认的真实 OS 失败，不伪造 Metadata，不直接调用 helper 填一份假 created 身份。若选择实际 OS 限制，必须局限专属子进程、安全公开 API、独立先验确认，不填盘、不改变全局进程限额、不引入 unsafe/libc。若当前许可 API 不足，复用窄的 feature 故障注入设施模型化外部 I/O 边界，明确其测试范围；不得把无法进入失败路径写成静态等价。

## W4：清理权、告警与新请求门槛

`prepare_new_request` 唯一 caller 为 `before_write` 的 !committed 分支；读取同 request_id 的合法 owner，先安全清掉未提交准备，再允许登记新请求。现有 CLI `same_request_retry_stops_before_registration_if_its_uncommitted_original_cannot_be_cleaned` 已有实际 start/add 的 `pending_owner_synced_before_payload`/`before_commit` 中断、同 rid 重试及 `pending_cleanup_before_remove` 失败。优先复用其夹具能力：原件内容保留、无新 requests/audit/额外容器、准确 IO；解除清理故障后同 rid 只提交一次并清旧原件。权限可能被合法清理尝试先放开，不能给此入口添加源码并未承诺的“所有 mode 零修改”断言。

`valid_internal_id→true` 有真正名字边界：cleanup 的 entries 三分支及 prepare 的 .owner 筛选均使用它；`read_owner_file` 只核 format、internal_id与路径相同、op合法，并未重新校 UUID。保存一个合法未提交孤儿，耦合改变目录名/侧车名及 owner.internal_id为 `not-a-uuid`，保持同一字节内容、未提交 request、合法 op。原实现必须将不合法名字作为 unknown 保留并告警；mutant 可把这份非UUID声明误当合法孤儿、chmod并删除。正例真实UUID孤儿仍清理。该 guard不是重复 UUID核验，不用 age 或名称臆测所有权。

cleanup :356 delete !：建立实际已完成 B，再建立 after_commit_before_effects 中断的 A，将整个 pending 目录移入本次测试私有保存处，根内 pending缺失。cleanup不得删任何保存原件或业务行，告警仅归 A 的 unpublished登记及其内部 id，不能错误告警B已完成记录。保留完整 Store快照及 A/B 原件，避免只检测 warnings非空而错过反向筛选。

cleanup :664 AND→OR：前层在 marker=RegularFile时已确保 reference=published delete，所以这一方向会同时为 true。另一方向并不成立：published delete允许初始 record.marker=None（合法 cleanup已完成），reference.delete=true。OR此时新增 maybe_fail_cleanup及第二次read_deleted_marker。共享正例“已清完删除的重复 cleanup”在 `pending_cleanup_before_remove` 故障下原实现不应额外告警，因为没有实际 metadata需删；mutant会出现marker删除失败告警。优先专属 CLI进程重放一个已完成且其metadata已清理的请求，并配置该现有故障，避免新请求C的正常清理失败混入告警；核本删除id不应额外出现marker告警，保持成功JSON/退出码与Store行。不要全进程环境污染并行测试。

若需要原件副作用进一步分辨，观察窗口应为 entries/record捕获 marker不存在之后、最终marker分支前，在本id名下创建合法新marker；原实现本轮不读/删它，mutant额外重新读取并可能删掉。可证明新读取事实存在，不能把 index/entries 二次读取视为不可变；也不能无条件声称删除这份合法同id marker本身构成破坏所有权。现有 `pending_cleanup_after_empty_check` 要有真实container，通常太早或不命中 marker-none入口，须先核caller/窗口。:469 的整个死分支见后文。

## W5：别把 current B 的 CheckedRequest 消费给 pending A

`before_write` 由 WorkService与WorkbookRepo写路径调用，先按当前rid装可信current，再按 Store::unpublished_request_ids（audit.seq顺序）装各 pending。cache只能在 pending_id==request_id时take。同一CheckedRequest含 row.published/effects/original，`finish_checked_request`却使用外层pending_id作为 mark/error归属，错配有可观察结果。

最小真实反例无需新observer：先完成请求B（如Workbook add）并保存原响应；随后请求A在 `after_commit_before_effects` 中断、保留unpublished登记，将A实际目录的已登记内容改坏而不破坏快照资格；重放B。原实现应先恢复A并被其效果错误阻断：顶层 committed=true、rid B、original B、pending_request_id A、pending_original A、准确 cause，A published仍0；B原响应/审计和全部竞争现场保持。mutant对 pending A错误 take cached B；由于B.row.published=true，complete=false，B目录效果跳过，可能静默返回B成功而A未恢复。

正例用同一场景健康A：A先完成，B按原响应replayed返回，两条请求分别只有一个审计/提交；另核仅本人未完成B的正常恢复。若追加“current B也未完成且更早A”可观察错误mark A或错误原响应，但必须先独立证明两个真实提交的夹具来源，不用任意把两份状态写成成功。现有old A阻断new B只能覆盖current=None分支，不能替代本次缓存分派反例。

## 两个限定静态候选

1. `cleanup_selected :469 OR→AND`：此处 reference=None、owner_file=None 的外层分支根本不可达。上面的 match 对同一对局部 Option将 owner_matches_reference置false；随后的 `if !owner_matches_reference` 已发告警并continue。两值未赋值、未重新读取。全部prepare/cleanup caller都经这份函数。只证明这个死分支，不推广为其他metadata或告警判断等价。
2. `PendingReferenceIndex::from_rows :116 OR→AND`：到达者只有 PublishDir/DeleteDir。两者之前的纯 reference校验都以 `valid_digest=Sha256Hex::new(...).is_ok()` 核过同一String，摘要格式谓词在此恒false。重复id在mutant下多append一个同key引用；同一纯 invocation末尾每个 references.len()!=1会拒绝。没有 Store/FS二次查询，索引不会返回给caller，接受/拒绝集合、STORE_CORRUPT和cleanup首个I/O前停止相同。原诊断从“请求内重复id”变“pending被多个效果引用”，也可能因后续坏行改变首个错误/执行成本；不标完整JSON、早期工作预算或错误文本等价。

全部15项仍记 not_run_by_this_preparation；2项只供限定静态复核，其余open。作者应先确认观察点/实际consumer与基线，短反馈后再扩。零测试、错误ID、失败路径未进入、前层遮蔽、source漂移、超时均保留未完成，不改为caught/equivalent/PASS。不扩新平台、宿主机制或发布。
