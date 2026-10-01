# T31 跨门槛回归后的变异输入

固定普通clone为 `target/t31-validation/repaired-source-v2`，临时候选 `c31dcb390c15496f6e9586a92b3b73fa2d199061`。170项源码/fixture/配置/脚本逐SHA核root/clone一致，完整治理树由clone Git提交冻结；新日志只写产品树。本轮完整清单重新生成并执行，不复用旧输入任何结果。采用Nextest0.9.146/cargo-mutants27.1、all-features、相对target、opt1、debug assertions/overflow checks开启；默认门禁另跑。

此前3a9f输入的core724完整执行与workspace复验发现跨node同号Occurrence批准的测试缺口，新增真实CLI用例实际捕获精确AND→OR反实现（exit100），恢复源码SHA后通过；因测试输入改变，core全部片和中断runtime片在 [归档](../superseded-before-cross-gate-oracle/superseded.json)，不计本输入通过。更早da2bd结果在 [前一归档](../superseded-before-implementation-review-fixes/superseded.json)。

完整inventory、baseline、两阶段无漏无重、每个missed/timeout/unviable的准确分类及Reviewer处置完成前T31保持doing；Linuxnot_run、M1/T16/T17分别保留后继门槛。
