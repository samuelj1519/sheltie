# T31 实现修复后变异输入

本轮因七项实现审查修复形成新输入，不复用此前变异结果。旧候选da2bd的全部完成/中断片在 [归档](../superseded-before-implementation-review-fixes/superseded.json)，旧产品基线仍为b926789。

普通Git clone位于 `target/t31-validation/repaired-source`，临时提交只固定测试输入，不是产品提交。源码、fixture、Cargo/锁/Nextest/脚本及治理输入在复制时固定；完整manifest和工具/feature/profile参数在本目录JSON记录。执行采用Nextest 0.9.146、cargo-mutants 27.1.0、all-features、相对target，opt-level=1、debug=0、debug-assertions/overflow-checks开启，默认门禁另跑。baseline、完整core/runtime inventory、所有第一阶段missed/timeout的完整workspace复验、逐项存活处置及独立审查未闭合前T31保持doing。Linux按用户授权not_run。

治理fixture使用本clone冻结的specs树；新的原始运行日志写在产品树evidence，不回写clone。源码及fixture若改变，本轮结果归档并生成新输入，不把旧片计为新候选通过。
