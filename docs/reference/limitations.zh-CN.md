# 支持、兼容性与信任边界

[English](limitations.md) | 简体中文

本页描述使用当前源码时需要考虑的范围。产品支持承诺来自[规格](../../specs/spec.zh-CN.md#当前产品环境)，历史与限定验收结论来自[验收边界](acceptance.zh-CN.md)和[发布记录](releases/README.zh-CN.md)。

## 版本与环境

当前产品定位是 macOS aarch64／APFS 本地场景。其他 OS、架构、非 APFS 与外置物理设备专项认证属于未来采用范围。当前物理 APFS 无法创建的非 UTF-8 名称没有现场遍历通过结论；字节接口拒绝测试不替代实际载体。

正式发布为 v0.2.0。当前开发源码是 0.3.0-rc.1，Store schema 4、cli-result/v4、work-result/v1、workbook-digest/v2；sheltie-export 和作者工具未发布。schema 1／2／3 的旧 Store 不自动迁移或清空，也不按 schema 4 解释。旧二进制与对应根保留用于历史查询，新开发线使用新的显式根。

## 操作与内容

| 事实 | 边界 |
| --- | --- |
| Attempt／Work succeeded | 表示执行或流程状态；报告内容、代码质量与用户接受分别判断 |
| gate 批准与 OS 主体 | 记录真实进程账户调用，不提供独立真人认证 |
| resume 与草稿路径 | 给当前接续指针；不证明草稿存在、完整或已封存 |
| work result 引用 | 绑定具体成功终点；普通查询不重新认证全部源字节 |
| attempt replace | 撤销正式提交资格，不停止进程、不隔离工作区、不撤销外部副作用 |
| 只读查询 | 不推进业务或恢复效果；SQLite 控制文件有 [D-039](../explanation/decisions/D-039-sqlite-read-control-files.zh-CN.md) 规定的例外 |
| 导出 complete | 完成规定的核验、整目录不覆盖发布与 OS 同步，不承诺物理断电持久或同权限隔离 |

接手者写共享工作区前，由操作者确认旧进程已停止或环境已隔离。导出的可编辑副本不是第二结果来源；编辑后的字节不再由初始 manifest 保证。错误和 publication_unconfirmed 的处理见[导出指南](../how-to/export-results.zh-CN.md)，接续及 pending 的处理见[接续指南](../how-to/resume-work.zh-CN.md)。

## 结论的使用范围

已实现、限定验收完成、实际用户接受、发布和可量化收益分别报告。历史 LEAK、费用、公平对照或旧取证缺口不会因新运行成功而自动关闭，详细结论见[验收边界](acceptance.zh-CN.md)。未知费用不填 0，Attempt 时长不换算为模型 usage。
