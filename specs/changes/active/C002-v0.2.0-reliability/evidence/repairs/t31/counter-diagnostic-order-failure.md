# 首轮计数校验的诊断顺序回归

647 项候选首轮全仓 run 退出 100：406 项已运行，405 passed、1 failed，241 项尚未运行。失败为 runtime `load_rejects_invalid_state_combination`：它要求先报告“status 与当前 Attempt”，新增计数校验先报告了另一个同样真实的内部矛盾。没有放宽断言或移除样例；改为先核原结构一致性，再核必要计数界，定向复验 run `1062e606-36f3-4753-98e9-d161d6ab6bdc` 两项通过。

[原始门禁驱动记录](counter-diagnostic-order-failure.driver.txt) 保留退出码和耗时。该轮完整 Nextest stdout 在重跑时被同名输出覆盖，不能据此提供未保存的原文或称为通过；这里明确保留失败事实，最终候选另需完整原始通过记录。后续门禁脚本会在开跑前自动归档同名输出。
