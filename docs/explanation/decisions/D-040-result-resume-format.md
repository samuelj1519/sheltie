# D-040 成果与接续采用单一 Store 格式

状态：`accepted`
日期：2026-10-03
关联 change：[C004](../../history/changes/C004-verifiable-delegation/README.md)

当前适用范围：终点明确选择、冻结引用、同快照读取和单一严格格式的原则继续适用。本文延续 [D-033](D-033-store-schema-2.md) 的拒旧原则，同时将其 schema 2／cli-result/v2 替代为 schema 3／cli-result/v3；后续 [D-041](D-041-attempt-number-and-replacement.md) 只替代这里的格式版本，采用当前 schema 4／cli-result/v4。work-result/v1 与 workbook-digest/v2 保持，当前字段见[存储合同](../../../specs/contracts/storage.md)。

## 背景

C004 新增终点 result 声明、当前 Attempt 接续指针及严格同快照读取。用户明确当前无实际用户。此次采用单一新格式且不设计迁移；这是一项工程决定，不推断原件不存在，仍必须保留已有原件。已有严格 Store 与冻结定义不能在同一格式内默默赋予多种响应语义。

## 选择

Store 升到 schema 3，公开封装文档为 cli-result/v3，结果 DTO 为 work-result/v1。Workbook/Flow 保持 v1 并增加默认 false 的可选 result 声明；旧 Store 整体拒绝，不迁移、不清空。目录摘要保持 workbook-digest/v2。只维护单一严格读写闭包。

准备作者与实现者由同等能力负责人承担，保留 T01/M1/T02 边界、独立 oracle 和未参与编写的独立审查。完成耦合的纯类型与读取原语后才接公开命令，不制造正常入口的占位或假成功。

## 否决方案

- 同 schema 双解释或缺字段默认迁移：模糊冻结定义/响应合同。
- 状态卡保存实时效果完成：refresh 发生在 mark_published 之前，会留下陈旧就绪事实。
- 各连接分别拼 state、revision 与效果：并发发布时可混合两版事实。

## 后果与确认

旧 schema 1/2 被准确拒绝并保留。status/result 对关联 requests/audit/效果做同读快照的严格校验；不同 Work 未完成效果不影响结果。真实 CLI、SQLite 和故障窗口用例及独立审查核同源投影和只读边界。
