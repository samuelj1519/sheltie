# 原理与源码

[English](README.md) | 简体中文

解释帮助读者理解系统为什么这样工作；具体操作见[指南](../how-to/README.zh-CN.md)，字段与命令见[参考](../reference/README.zh-CN.md)。

- [架构与恢复原理](architecture.zh-CN.md)：三层职责、纯决策、事务与文件发布为什么分开。
- [沿源码阅读](source-tour.zh-CN.md)：从公开命令追踪真实调用链与测试。
- [方法、状态与内容判断](workflow-model.zh-CN.md)：Workbook、Work、Attempt 和 gate 的关系。
- [作者工具](workbook-editor.zh-CN.md)：为什么以完整草稿和公开 CLI 编辑方法；接口与限额见[参考](../reference/workbook-editor.zh-CN.md)。
- [设计来源](design-sources.zh-CN.md)与[产品／Rust 推论](product-rust-design.zh-CN.md)：外部阅读线索、时点与适用范围。

已采用取舍的正式理由见按主题组织的[设计决定](decisions/README.zh-CN.md)。需要追溯某次变更或历史验证时查[历史档案](../history/README.zh-CN.md)，无需按迭代顺序阅读当前原理。外部建议不直接构成产品承诺。
