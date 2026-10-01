# T31 全链验证与修复证据

最终交接：2026-10-01，T31按用户明确安全验证豁免收尾；任务状态只看[plan.md](../../../plan.md)。Linux为not_run；独立M1、T16/T17保留后续门槛。暂缓项不构成安全PASS。

七项实现审查修复见[逐条答复](../../../review-response-implementation-2026-09-30.md)与[修复证据](implementation-review-fixes/README.md)。追加真实caller测试覆盖效果登记/快照/输入绑定/HTTP来源客户端；HTTP只替换curl传输边界，不访问外网。退役无人调用的optional helper和不可达marker分支；序号边界用合法997夹具验证998/999接受、1000拒绝与回滚；治理夹具保留全部Markdown，过滤原始运行产物的重复复制。

源码冻结输入为`49d3a191aa4c918aab279617fa2bf7d9b3b36a0e`，167项源码/主测试/fixture/配置保持一致，方法见[闭包](mutants/closure.md)。最终默认全仓675/675、零skip，fmt/check/Clippy、MSRV1.85 locked、离线deny、docs/specs/core-vocab/tests/skill及dist plan通过，原文见[最终门禁](final-acceptance-gates/gate-results.json)。最终治理树另行绑定，不把旧clone的164项治理输入当当前治理树。

完整2499项第一阶段已经执行；第二阶段保留245项终态结果，剩余混合安全验证按用户要求停止。完整[逐ID处分](mutants/final-dispositions.json)：1803第一阶段捕获、21第二阶段新增捕获、26仅补充捕获、311编译器确认不可构建、64独立审查等价、4本平台不适用、1无当前producer义务、269[用户暂缓](mutants/user-deferred-ids.json)。补充共29个唯一变异被真实caller捕获，其中3个亦被正式第二阶段捕获；输入和正式结果不混计。[独立收尾审查](final-review.md)认可本次豁免范围内提交。

[coverage-matrix.md](coverage-matrix.md)保留CLI/runtime迁移、确定性交错、真实exit70/SIGKILL、发布/删除窗口、字节/权限/Store归属及Root/.lock生命周期oracle。旧输入和失败输出保留，不重写历史PASS。

旧运行原文及最终原文采用无损压缩时，`raw-manifest.json`逐成员保存原始bytes与SHA；`raw.tar.gz`保留完整日志/diff，未压缩的outcomes/mutants索引供审查读取。压缩不改变运行结论。
