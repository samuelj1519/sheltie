# C008 目标前检原件

本目录只记录采用范围内的项目声明盘点和缺项；探针机制 `not_adopted`，原机制、真实观察与价值义务 `not_run`。当前文件路径以 C007 completed 目录为准，不把历史 active 路径当当前入口。

- [inventory.json](inventory.json)：6 份实际 manifest、6 个 Flow、24 个 Node 的原始声明值/字段缺省，以及输入文件字节标识。manifest 与 Node 的 `requires` 均为 0；`resource.*` 是 Workbook 内参考文件，共 15 个绑定、13 个独立文件。
- [preflight.json](preflight.json)：真实目标、`kind:name`、宿主/版本/规则/读取范围、作者必需确认、重复摩擦、实验预算与收益阈值全部为 null；列明原义务 `not_run`、精确原因和资料范围。
- [historical-index.json](historical-index.json)：前两份 `/private/tmp/` 盘点的逐字节历史副本。它们保留旧路径与 SHA，只用于历史保真，不证明当前文件身份。
- [byte-manifest.json](byte-manifest.json)：本目录其余文件的字节数和 SHA-256；不包含自身，避免自引用。

盘点只读仓库的 manifest、其选定 Flow、明确 `resource.*` 文件和 C007 方法文件；文件 SHA 不是 `workbook-digest/v2`，也不是引擎校验或宿主就绪证明。未扫描宿主/技能目录、Store 或进程环境，未添加依赖，未创建 probe、配置、fixtures 或测试骨架。保存本目录原件是本轮授权证据写入，不是 probe 的 no-write 验证。

当前没有选定必需宿主资源及重复人工核对事实。未观察宿主，不能从声明缺省推出资源缺失或 mismatch。实际条件补齐后仍按原采用门槛冻结目标、规则、授权、预算并独立审查，才开始原机制和真实观察。
