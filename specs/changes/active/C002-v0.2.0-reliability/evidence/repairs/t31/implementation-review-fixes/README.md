# T31 实现审查修复证据

对应 [实现审查](../../../../review-implementation-2026-09-30.md) 的CR-S01–04与CR-P01–03。用户于2026-09-30授权全部修复并完成T31；没有删除原T31工作。

- `initialization-gate-red.log`：真实首次建库SIGKILL与缺失approval的两项失败。
- `initialization-green.log`、`initialization-gate-green.log`、`memory-initialization-focused.log`：阶段通过；最终init采用pinned rusqlite0.37的安全serialize API内存生成完整schema2 bytes，同SafeFile独占写与NOREPLACE发布，不再重开暂存SQLite路径。
- `sync-confirmed-red.log`、`sync-green.log`：已落位原件/目录二次sync失败被原实现忽略，补同步后保持pending。
- `completed-history-sync-red.log`、`completed-history-sync-green.log`：已published begin删brief后补回但sync失败，二次重放也必须重试sync；成功才返回原快照，原请求/bytes/revision不变。
- `non-gate-block-red.log`：纯core从合法NoLegalEdge仅改理由Gate的反例，冻结图必须拒绝非门槛Gate受阻；合法未批准取消仍接受。
- `add-preflight-oracles.log`：结构缺manifest不建Store/锁；私有副本内容非法无业务行/请求/审计/final，来源修复后同rid成功及重放。
- `sync-standard-test-concurrency.log`：Mutex/RAII隔离普通cargo test同进程故障槽，3项通过；nextest独立测试进程机制保留。

第一轮全门禁因旧T24矛盾断言失败在 `gates-before-add-preflight-contract-correction/` 保存；第二轮653/653与MSRV通过但deny公告缓存锁受sandbox限制的原文也保留，取得该缓存锁权限后的离线检查在 `deny-cache-lock-approved.log`，exit0。本地缓存检查不声称公告库实时更新。阶段编译失败原文以build-failure或snapshot-check文件名保留，不当行为红证据。

最终candidate/input、独立审查、默认门禁和完整mutation结果尚待固定；本文件不把阶段通过写成T31/M1完成。
