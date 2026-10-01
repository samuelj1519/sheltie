# T31 冻结输入与运行

普通clone：`target/t31-validation/final-source`；临时候选：`49d3a191aa4c918aab279617fa2bf7d9b3b36a0e`。这不是产品提交。167项源码、fixture、配置、脚本逐SHA核root/clone一致；164项治理输入在clone冻结，保留全部权威文件与evidence Markdown，原始非Markdown运行产物不作为治理测试输入。详见[source-input.json](source-input.json)。

[执行闭包](execution-closure.json)固定all-features、Nextest0.9.146、cargo-mutants27.1、opt1、debug assertions/overflow checks、每副本相对target与600秒timeout；[pipeline-final.py](pipeline-final.py)逐片核candidate、源码与治理SHA、baseline和清单集合，原始输出保存后才进入下一片。第一阶段core4片/runtime8片分别跑完整crate测试；全部missed/timeout进入workspace复验，不抽样。逐结果与结束状态见[progress.json](progress.json)、[results.json](results.json)。完整处分及独立复核前仍为WIP。

[上一输入](superseded-before-guard-oracles/superseded.json)的2501项第一阶段和中断workspace复验完整保留，因新增caller oracle和两个已确认退役点，不计本输入通过。更早的[cross-gate输入](superseded-before-cross-gate-oracle/superseded.json)和[实现审查前输入](superseded-before-implementation-review-fixes/superseded.json)同样保留。

workspace首片8-job并发产生15项Timeout（多数674项通过、剩长场景被600秒终止），完整原文在[workspace-overload-first-attempt](workspace-overload-first-attempt/README.txt)。不当caught，已从正式结果移出。保持49d源码/主测试/fixture/配置/工具/profile完全不变，workspace复验改为4个mutant并行、每个Nextest2线程，完整381项重新执行；执行身份分开记为[execution-closure-workspace.json](execution-closure-workspace.json)，不把原超时改写成PASS。低并发未变异baseline675/675、0skip、80.521秒，见[原文](workspace-low-concurrency-baseline.stdout.txt)。第一阶段和core原执行记录保留自己的闭包ID；auditor明确校验两执行闭包引用同一source SHA，分别核逐结果原始文件。

2026-10-01最终交接：原完整安全变异执行未结束，不能写PASS。按用户明确要求停止后续混合安全验证，完整2499项已作[逐ID登记](final-dispositions.json)，269项为[deferred_by_user](user-deferred-ids.json)。正式第二阶段保留245个终态结果；两次中断不捏造总exit或未完成项结果，只保留有完整build/test状态和日志的terminal entries。第二次为用户要求停止，见[停止记录](monitor-status.json)。本次T31完成采用主计划的明确豁免，不关闭M1。

已经独立接受的29个补充真实caller捕获使用各自额外测试闭包，3项也在正式第二阶段捕获，最终新增26项单独分类；不追改原Missed。当前Root治理输入另在最终验收时固定，不沿用clone旧164项治理树。压缩原文时逐成员核SHA和bytes并保留完整`raw.tar.gz`，outcomes/mutants索引保留在原路径；原日志路径可从压缩包内恢复。
