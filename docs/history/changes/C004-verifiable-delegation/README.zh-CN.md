# C004：明确成果与可靠接续

[English](README.md) | 简体中文

状态：`completed`
目标版本：`v0.3.0`（开发目标，尚未发布）
兼容性：不设计兼容层或数据迁移；仅实现采用后的单一合同
基线：`3692c4eed62e22f57a94573635890c8b61405cc4`
Owner：`Codex /root`
记录形式：`reference`
历史快照：`9d98bf8f15944b7bda4fbd4096e7771b09eab728`

## 变化与理由

增加终点 result 声明、work result 和当前 Attempt 的 resume 指针。最终成果绑定具体成功终点 Attempt 的输入或封存输出；不从目录最新文件、报告正文或摘要猜测成果。会话中断后使用当前 status 接续同一 running Attempt。

采用单一 Store 格式切换（schema 3、cli-result/v3、work-result/v1）；后续 C005 的 schema 4／cli-result/v4 是当前目标。

## 验证与限制

实际 agent 文档交付、同 Attempt 冷接续和新消费者读取已在限定范围核验。查询列举冻结引用，不证明内容质量或再次核验源字节。原真人对照、总费用、净收益与旧查询取证偏差不能由这些技术结果补证。

本页保留设计与结果摘要；当前行为以根规格和合同为准。历史验证不能直接复用为当前候选 PASS。

## 参考

[接续指南](../../../how-to/resume-work.zh-CN.md)、[协议合同](../../../../specs/contracts/protocol.zh-CN.md)、[格式选择](../../../explanation/decisions/D-040-result-resume-format.zh-CN.md)。完整任务、审查与运行原件按[历史查阅指南](../../../how-to/maintain-docs.zh-CN.md#查阅历史原件)从上述快照读取。
