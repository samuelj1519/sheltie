# T58 最小设计：历史响应的审计执行事实一致性

只读准备，未编写源码/oracle、未运行Cargo，未批准候选。F-M2-01原实物32c：真实gated-release已发布批准请求，只改audit.principal，same-rid replay成功而status报STORE_CORRUPT。原repro SHA、固定binary和五表/业务原件见JSON。操作仅限自有临时根的记录一致性，不建立/修改真实账户，不扩权限或外部系统。

## 一套事实，两个校验位置

最小生产范围只有service.rs。把既有validate_read_audit改为显式参数的私有执行事实helper，供read和load_checked_request共用；无需WorkReadRequest包装getter、trait、DTO、state或新observer。

纯层建议 `validate_audit_snapshot_fields(request_id,&AuditRow,&CheckedData)`：audit.principal非空；复用已有Timestamp::parse核audit.at规范格式（read的decode_read_request已parse row.at，Store bundle已核row.at==audit.at）；ApprovedData的by/at与audit.principal/at精确相同。ApprovedData.by/at已pub(crate)，不改snapshot API。不要trim/lowercase、不查询当前OS账户、不比较重放者身份；这是历史记账一致性，不是真人认证。格式规则仍storage既有RFC3339 UTC秒精度，不新增时间政策。

写路径在Command解码/check_data之后、self.load之前执行纯层，保留T46完整command/data在冻结IO前拒绝。读路径decode_read_request也调用同一纯层，不用另一套批准比较。批准snapshot.by合法非空只证明形状；未与audit同一事实绑定前不得成功。

完整执行事实层适配旧 `validate_read_audit` 为 `validate_audit_execution_facts(request_id,&AuditRow,&WorkState,&Reply)`。写在loaded state/graph与validate_command_owner_data之后、work_original_response生成之前执行；read按post-load资格循环共用。不要为提前全部时间检查增加第二次Store state查询、缓存或改load接口；纯层能提前的字段先提前，依业务上下文的事实在原响应资格前核。

## 完整 command/reply/producer 时间关系

| Reply / Command | core真实producer | 必须绑定audit.at的历史事实 |
|---|---|---|
| Started / Start | decide_start created_at=ctx.now | state.created_at |
| AttemptBegun / BeginAttempt | decide_begin started_at=ctx.now | 该Reply精确Attempt.started_at |
| AttemptReplaced / ReplaceAttempt | old.ended_at与new.started_at同ctx.now | 两者都相同；不是new后来ended_at |
| AttemptSubmitted / SubmitAttempt | 成功Attempt.ended_at=ctx.now | 对应Attempt.ended_at |
| AttemptFailed / FailAttempt | 失败Attempt.ended_at=ctx.now | 对应Attempt.ended_at |
| GateApproved / ApproveGate | append node/occurrence/by=ctx.principal/at=ctx.now | 同node/occurrence批准记录by/at，同时ApprovedData.by/at等于audit |
| Cancelled / Cancel | updated_at=ctx.now，terminal拒后续新命令 | Cancelled状态的updated_at |

runtime只在新请求建Context(now,principal)。core决定的同Context字段进入state，snapshot_data从该decision.state批准记录复制by/at，commit同ctx写audit principal/time及requests.at。请求重放不取新ctx作为旧审计标准。created/started/ended/approval历史事实后续保留；Cancelled terminal不能有合法后续新业务改updated_at，显式历史修补不改该字段。不能把全部Reply比较current.updated_at/current.status/current.principal，也不推断更强的单调时钟或所有历史相同状态。

当前validate_command_owner_data已核command/Reply身份、批准state与snapshot.by/at、历史status与冻结图；本修复把audit这一边接进同事实。现有read helper的逐Reply时间逻辑保留，不创历史重演/第二状态来源。

## 最小独立 oracle：3条共享能力 + 现有效果控制

推荐只在CLI snapshot_qualification.rs追加3条能力，复用private business_files（全works/workbooks/pending type/mode/bytes）与common::store::store_rows（五表每列SQLite类型与原JSON字节），不扩新框架。新任务测试注释归C002-T58；当前plan已允许该文件。

**H1 合法历史后来状态。** 一份two-step真实链覆盖Start、Begin、Replace、Fail、Submit、Cancel：outline#1.0→replace #1.1→fail→begin #1.2→submit→begin summary→cancel。default max_retries=1，替换不耗失败额度，故这条链真实合法。另一份gated-release：notes begin/submit/approve后，archive begin再cancel或成功。保存每个实际成功原响应，后续动作后同rid重放应仅data.replayed=true，其余原revision/data/next/by/at保持；五表和原件不新增/改写。至少一次后续操作实际跨UTC秒（有界等下一秒，不改系统时钟/Store事实），避免错误地一律拿current.updated_at比较仍偶然绿。approved原Active/currentCancelled与替换后newAttempt后来结束必须合法。

**H2 单事实不一致。** 实际完全published批准请求，只改audit.principal为另一个非空记录串，或只改为空；普通Begin只改principal为空，保证新规则覆盖非批准请求。时间反例选普通Begin对应state_json唯一started_at字段改为另一个规范Timestamp，其他身份/类型/状态不变，使新增共用执行事实判定被真实消费。原read早已拒，修前replay会暴露差异。audit.at单列与requests.at不一致已被旧元数据检查抓住，不能拿它当新增时间predicate证据；若采用保持row/audit时刻相等而与state不一致的反例，应明确是同一个时间事实在两处的耦合改动，不能声称只改一列。

readonly status/result应STORE_CORRUPT/exit1；same-rid A写重放应EFFECT_PENDING/exit1、committed=true、rid A、cause STORE_CORRUPT，**无original/revision/pending_original**。完整五表原值与业务原件type/mode/bytes在失败后保持。不要只断言is_err，或将普通记录串改动称新账户/真人批准。

**H3 纯层优先序。** 上述实际批准记录principal不一致/空，再把frozen workbook移到Work内retained-workbook。原源码缺纯校验时可能先读missing freeze；修后必须给audit明确原因而非generic freeze原因，无original/revision，保留所有Store与retained原件。无需新point；现有start_snapshot_identity_mismatch…已用同样真实move与完整inventory形式。健康audit而freeze缺失仍按既有业务资格失败，不能为证明pure层而过早伪造original。

**H4 复用效果例外与阻断正反。** 元数据/审计/业务图都合法时，effects未知字段/批次路径/sync/marker错误仍保留已核original。现有invalid_later_effect_is_rejected_before_any_effect_runs、old_effect_blocks_new_request_with_distinct_identities及effect_contracts的original-data断言可复用；它们有真实COMMIT后未完成A的producer。若需新B阻断判据，在同一个真实pending A现场只破坏audit.principal：B committed=false/rid B、pending_request_id=A、causeSTORE_CORRUPT且无pending_original，B尚未登记。健康A只有效果错误则保留A pending_original。不要把本finding完全published A直接改published=0来制造新缺项source。

## 错误边界与最小白名单

audit或其执行事实不合法，在生成original之前返回RequestLoadError（不with_original）；own replay保留已确认的提交身份，缺原响应也缺revision。合法审计/快照及state/graph资格完成后，existing effects_result.map_err(with_original)继续生效，不能为了audit修复把效果解码移到原响应资格前丢合法original。read没有EFFECT_PENDING封装而直接STORE_CORRUPT，两入口结果一致的是资格与事实，不是强行同JSON。

生产仅service.rs（纯字段helper、共享旧执行helper、两caller接线）。新测试优先snapshot_qualification.rs，回归沿replay/scenario_gated_release/effect_contracts/implementation_repairs。当前tasks列出的gate.rs尚不存在；它可以是作者后续新建目标，但不是可直接调用的入口，最小方案无需新建该文件。snapshot.rs字段已足够；不改core/store schema、principal来源、状态机、账号权限、公开字段或新增observer。package finding/计划/验证/审查/raw及必要protocol§5澄清在T58范围。新oracle/源码完成后的最终独审由oracle_review，不由本准备作者自批。

先冻结32c修前CLI/fixture，H2/H3实际red应是成功重放或错误来源，编译错误/零测试不替代；再最小实现、同正反green。按计划重跑新source的CLI/gate/history/effects资格与完整工程/MSRV/default/治理，保存实际二进制和新closure；旧190的948或同命令不能覆盖新生产资格层。current-binding重点复核G01历史业务/审计与效果original消费者；新更早拒绝可能改变旧oracle路径，需明确对应观察窗口，不机械将旧mutation全文授新PASS。M2仍待修后完整Spec/工程独立复审。
