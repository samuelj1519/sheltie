# C006：可编辑成果副本

状态：`completed`
目标版本：`v0.3.0`（尚未发布，外围工具不进入dist）
兼容性：`只采用 work-result/v1 与本方案的源读取合同；release 版本仅标识来源`
基线：`8d27348d995a434ae3dbf8255e1110973dec1381`
Owner：`Codex /root`
记录形式：`reference`
历史快照：`f38954d543ff01eb5a798be48f29060b80d5952c`

## 变化与理由

引擎提供同快照绑定的受限原字节读取；未发布外围 sheltie-export 只通过公开 CLI 取得最终选集，不打开 Store。导出先暂存、核字节与 manifest、同步，再以 NOREPLACE 发布新目录。完成后的可编辑副本不成为第二结果来源。

## 验证与限制

实际 agent 消费、旧编辑保留、再次新副本及跨 APFS 载体链已限定验收。虚拟 APFS 跨设备测试不等于外置物理设备认证。complete 表示合同规定的核验、发布与 OS 同步完成，不承诺物理断电持久或同权限隔离；工具尚未发布。

本页保留设计与结果摘要；当前行为以根规格和合同为准。历史验证不能直接复用为当前候选 PASS。

## 参考

[导出指南](../../../guides/result-export.md)、[D-042](../../../decisions/D-042-final-artifact-copy.md)、[源码入口](../../../../crates/sheltie-export/)。完整任务、审查与运行原件按[历史查阅指南](../../../guides/documentation.md#查阅历史原件)从上述快照读取。
