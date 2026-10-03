# T57 最小实施卡：G13 五项

这是准备卡，不是实施或验收：未改tracked、未编写/运行oracle，全部目标not_run。仅macOS aarch64。作者由根agent实施；最终独立验收由oracle_review负责，本准备作者不自批。HEAD和来源SHA、5个准确旧/当前ID及原diff SHA在JSON。

## 最小范围

新增3条能力测试即可：fsx现有controlled_object_tests中两条解除配置测试，CLI reliability_crash中一条三配置作用域测试。第五个maybe_fail_cleanup目标优先使用两条已定义的CLI业务能力；短反馈证明不足才补该能力，不先新增镜像helper测试。推荐tracked范围为 `crates/sheltie-runtime/src/fsx.rs`（test-only）和 `crates/sheltie-cli/tests/reliability_crash.rs`。不需要改failpoint/pending生产逻辑、public API、observer或新harness。

归档目标语义：disarm_rendezvous→Ok须由“解除后同真实consumer不再激活”捕获；disarm_sync_error→Ok须解除尚未消费的root/name配置；两个归档rendezvous :170:34/:170:59分别改env name/id匹配，表达式当前在rendezvous_payload；pending :724 maybe_fail_cleanup→Ok由业务告警/prepare停止捕获。默认feature关闭的无影响证明不能代替本轮all-features。

## 现有入口，按实际可见性复用

- unit串行锁：failpoint.rs:50 `RENDEZVOUS_TEST_LOCK`，cfg(all(test,feature="failpoint"))。当前fsx、pending、effects unit故障配置测试都用这个共享锁；在两条新unit测试从配置开始到最后观察/清理一直持有。不同integration binary有独立进程静态，不拿implementation_repairs的SYNC_TEST_LOCK当全库锁。
- fsx.rs:3509 `controlled_object_tests::observe_change/observe_recorded_change`：已内部锁同一Mutex，thread::scope+release/disarm Drop。不能在持有同一非递归锁时再调用。其temp载体内部管理、返回后消失，不直接适合“旧reached删除后再次观察”的生命周期测试。
- runtime tests/common/mod.rs:220 `RendezvousWorker<T>::single/new/wait/finish`：wait10秒，finish先release再join/disarm，Drop也释放、join/disarm。用于真实pause备选路径；不为单纯解除测试把integration common全部导入生产unit模块。
- runtime tests/implementation_repairs.rs:12 `SYNC_TEST_LOCK/SyncGuard/sync_guard`：只在该integration进程串行，Drop disarm。现有 `matching_history_after_failed_parent_sync_stays_pending_until_sync_succeeds`、`existing_output_directory_after_failed_parent_sync_stays_pending_until_sync_succeeds`覆盖匹配消费和重放，可复用业务预期，但它们通常已消费故障后才disarm，不能替代未消费取消。
- CLI tests/common/process.rs:6 `Process::spawn/reached/release/finish`：spawn明确清并设置子进程env；reached/finish10秒；Drop release/kill/wait。reliability_crash已导入common::process::Process，复用其Env、store_rows、record/container/tree等现有夹具。

## T57-A：disarm_rendezvous，真实 write_atomic

建议测试名 `disarmed_rendezvous_does_not_reactivate_a_real_atomic_write_callback`，放fsx现有controlled_object_tests。具体消费链：`ManagedFs::write_atomic` (:1047)→write_atomic_unlocked→rendezvous_observed_path("atomic_write_before_open", target绝对路径, 实际随机tmp路径)。该callback在open/create前，scope已来自实际目标，不猜临时名。

最小有界反例沿已有T51测试 `write_observation_failures_preserve_exact_committed_bytes_or_refuse_before_creation` 的atomic-before场景：真实Home/lock/fs、根下单叶目标；独立carrier临时目录中建立普通文件并写手写sentinel，把该非目录路径作为配置carrier。arm匹配point/scope，实际write_atomic应IO、目标和自有tmp尚未创建、carrier字节未动；rendezvous只clone配置，失败不会消费它。随后disarm，再对同consumer/同目标写手写正常字节，必须成功且独立读回精确内容；重复disarm/合法写仍正常。mutant no-op仍触发原callback，第二写IO而非正常落位。全过程用真实写路径与OS错误，不读私有Option、不复述配置filter bool。

这条最小路径无需worker等待或新observer，坏carrier原本就是现有API可配置的外部I/O错误测试。若作者要用有效目录的pause生命周期备选：第一次匹配→reached精确payload→release完成，保留release并删除reached，disarm后再次同consumer必须不重建旧reached；必须用RAII release/join且明确payload是observed tmp路径，不能套Process.reached对literalpoint的断言。二者选一，别为同一能力写双份冗余harness。

## T57-B：disarm_sync_error，真实 write_new_atomic

建议测试名 `disarmed_sync_failure_does_not_reject_a_real_new_atomic_write`，同模块同共享锁。具体消费：`ManagedFs::write_new_atomic` (:1057，crate可见)→write_atomic_unlocked(NOREPLACE)→sync_error(root,"managed_file_parent_sync") (:1109)，实际rename后、parent fsync前。

两个真实Home/lock/fs A/B，所有目标用不同新leaf。arm A的managed_file_parent_sync；先B write_new_atomic成功，独立核B字节（异root不消费配置）；disarm；A write_new_atomic应正常成功并独立核精确A字节。mutant保留A配置，A返回IO，尽管rename已经留下正确原件。再用一份新的arm A正例：新目标实际命中一次IO，核其rename后原件正确；下一新目标成功，证明一次性消费，而不是先消费再验证disarm。

关键可见性/调用差异：**public write_atomic使用RenameFlags::empty，不到managed_file_parent_sync这条NOREPLACE分支**，不能误选它然后得到零命中。该consumer只能在当前unit内部直接用write_new_atomic，或通过现有历史文件恢复的真实业务调用；不为测试公开新方法。可复用 `new_atomic_history_publish_never_replaces_an_existing_regular_or_dangling_leaf` 的NOREPLACE夹具及implementation_repairs的后rename错误预期。每次操作核目标精确字节、原IO路径；不要要求故障时目标一定不存在。

## T57-C：env fallback，三配置一条真实child能力

建议测试名 `subprocess_rendezvous_reaches_only_the_configured_name_and_scope`，放CLI reliability_crash。这里已导入Process、Env和store_rows；不要选crash.rs后再引第二份common，它目前用runtime_common而非CLI Env。复用本文件stop helper的Kill分支及before_commit crash场景API。

三份独立Env/carrier/rid，以真实Workbook add触发 `Store::commit→maybe_exit("before_commit")`。此point位于SQLite BEGIN之前，name和scope都为literalbefore_commit；不要把它写成“所有Home控制对象也没创建”。新CLI进程的in-process RENDEZVOUS为空，必经env fallback；如果同进程arm会覆盖env分支，两个目标可能根本未测。

1. 配置name=id=before_commit：Process.reached核literalpayload与真实暂停，查询本rid request/audit/本次Workbook行尚无；release后Process.finish成功，JSON/rid与最终精确副本有效，恰好一份请求/审计。
2. 只改name为另一字符串、id仍before_commit：carrier预建release，正常完成后必须无reached。
3. name仍before_commit、只改id：同样预建release，正常完成后无reached。

预建release仅用于两个不匹配例，确保mutant误激活也有界完成并留下错误reached，而不靠timeout判断。两个==→!=各会损坏匹配正例并激活自己的单项失配；业务副本/JSON仍需核，不能只测私有布尔。每份child环境由Process管理，不用共享进程set_var，不造新observer，失败RAII必须释放/kill/wait。

## T57-D：maybe_fail_cleanup，复用完整业务能力

准确复用：

- CLI workbook.rs:161 `cleanup_warning_goes_to_stderr_without_changing_success_json_or_replaying_business_effects`：真实add成功但cleanup_error，stdoutJSON/exit仍成功，stderr含实际rid/object/reason；解除后replay仅1份请求且最终Workbook原件在。
- CLI reliability_crash.rs:709 `same_request_retry_stops_before_registration_if_its_uncommitted_original_cannot_be_cleaned`：真实start/add在pending_owner_synced_before_payload或before_commit中断，同rid重试故障必须IO、旧原件字节保持、requests/audit不追加；解除后安全清旧残留且只提交一次。

四个生产caller都在cleanup_selected（当前:604/:629/:650/:672），同一literalpending_cleanup_before_remove；callee由feature+专属child env控制。whole function→Ok会删掉上述告警与准备门槛。若正式mutation仍Missed，先核filter是否确实包含这2条、feature和实际CLI artifact，不立即为wrapper写镜像is_err测试。注意孤儿权限可能在cleanup失败点之前合法放开，不加“所有mode不变”的伪预期。

## 短反馈、冻结与收尾

最小焦点闭包是3新能力+2复用能力，新增生产observer数量0。原G13 filter不一定选择CLI workbook和reliability_crash的复用test；`binary(crash)` **不是** `binary(reliability_crash)`。新增作用域名含rendezvous/disarm可兼容旧test正则，复用两条必须按准确test名/实际binary显式纳入。执行者先用nextest JSON核所有5条存在、feature启用、非零筛选，再冻结候选/source/config/Cargo-injected CLI artifact；这张卡没替你运行list或baseline。

先短正例基线，再对5个准确语义patch分别正式执行，保留当前原diff映射、raw outcome与完整input closure。不以默认关闭feature的静态无影响处分all-features旧ID；零测试、未到窗口、错误consumer、source漂移、timeout保持open并停止。T57源/测试完成后按任务实际闭包验证与提交，不把这组harness测试上升为全产品/全平台验证；oracle_review独立核正式结果，本准备作者不参与自批。

## T56 库存对应修正

原准备稿把早期库存名称写在current_inventory_id字段，未表示当时新库存的真实行号/函数名。现已按 `t56-current-mutants.json` 更新JSON全部5项当前ID及diff SHA，历史old_id/old_diff_sha256不变。原稿完整保留为 `t57-implementation-directions-before-t56-remap.{md,json}`；JSON记录原稿与库存SHA。

| 历史目标/早期库存 | T56准确当前对应 |
|---|---|
| failpoint :77:5 disarm_rendezvous→Ok | :77:5 disarm_rendezvous→Ok |
| failpoint :108:5 disarm_sync_error→Ok | :108:5 disarm_sync_error→Ok |
| failpoint :170:34 rendezvous env name ==→!= | :182:34 rendezvous_payload env name ==→!= |
| failpoint :170:59 rendezvous env id ==→!= | :182:59 rendezvous_payload env id ==→!= |
| pending :724:5 maybe_fail_cleanup→Ok | :730:5 maybe_fail_cleanup→Ok |

函数重命名对应为旧rendezvous→新rendezvous_payload中的同一env fallback表达式；不对应新函数 :177 的in-process .filter判断。五项去掉diff文件/描述header与hunk坐标后的完整patch payload逐一相同。这只修正身份对应，不是执行/等价/PASS；实施源变化后根agent还需再次fresh map。
