# D-044：当前产品定位为 macOS／APFS 本地场景

状态：`accepted`
日期：2026-10-05
关联 change：[C002](../changes/completed/C002-v0.2.0-reliability/README.md)、[C006](../changes/completed/C006-result-delivery/README.md)、[C008](../changes/completed/C008-dependency-readiness/README.md)

## 背景与选择

用户明确确认「产品定位就是当前 macOS／APFS 使用场景」。此前剩余清单将不可在当前APFS构造的物理非UTF-8名称、外置物理盘专项验收列为待提供环境，容易与当前交付所需验收混淆。采用原话和原件保持范围见 [固定历史快照](../guides/documentation.md#查阅历史原件)中的 C008 采用记录。

产品环境的单一权威在 [产品规格](../spec.md#当前产品环境)。现有macOS aarch64范围保持；非APFS、其他OS/架构和外置物理设备专项认证不属于当前交付的必需验收。E01/E02按当前范围记为无需执行，历史not_run/environment_blocked仍保留。以后出现真实应用需求时另行采用对应目标和载体。

## 后果与确认方式

当前APFS已在创建阶段拒绝物理异常名称；引擎非法名称/参数的准确拒绝合同和已有字节接口及实际argv测试保持。导出的完整性、权限、原子不覆盖、同步和错误状态义务仍适用于当前范围，原本断电持久性非承诺不扩大。

本决定只限定支持承诺与验收范围。代码、CLI/Store/Workbook格式、历史Work和测试不改，不增加文件系统白名单；不由范围调整推出历史公平、费用、15%净收益或新release已通过。更新清单后通过独立事实审查与文档/规格门禁确认。
