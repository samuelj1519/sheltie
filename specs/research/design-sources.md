# 设计与维护阅读线索

这些链接来自项目既有来源笔记，供进一步阅读。来源核查时点为 2026-09-27／2026-10-03，外部文档可能更新；这里不把外部建议当作 Sheltie 的采用决定或已执行证据。

| 阅读主题 | 一手来源 | 在项目中接着读 |
| --- | --- | --- |
| 简单工作流、agent 内部自主与工具边界 | [Building effective agents](https://www.anthropic.com/engineering/building-effective-agents)、[Writing effective tools for agents](https://www.anthropic.com/engineering/writing-tools-for-agents) | 宪章、架构、源码导读 |
| 长任务的完成标准、接续与独立评价 | [Effective harnesses for long-running agents](https://www.anthropic.com/engineering/effective-harnesses-for-long-running-agents)、[Harness design for long-running apps](https://www.anthropic.com/engineering/harness-design-long-running-apps) | 实施指南、效果评估 |
| 教程、操作、参考与解释的职责 | [Diátaxis](https://diataxis.fr/) | 文档地图、文档维护 |
| 决定的背景、备选方案与后果 | [Documenting Architecture Decisions](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)、[ADR templates](https://adr.github.io/adr-templates/) | decisions |
| 发布兼容性与面向用户的变化 | [Semantic Versioning](https://semver.org/spec/v2.0.0.html)、[Keep a Changelog](https://keepachangelog.com/en/2.0.0/) | release records、CHANGELOG |
| Rust 接口、模块与验证 | [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)、[Cargo Book](https://doc.rust-lang.org/cargo/) | engineering、architecture |
| SQLite WAL 的控制文件与事务 | [SQLite WAL](https://sqlite.org/wal.html) | 存储合同、D-039 |

这些材料提供设计选项。项目是否采用仍由自己的用户需求、合同、真实消费者、质量与成本证据决定；有限实验不能证明一般收益或所有平台支持。旧提案逐项推导与来源覆盖矩阵从[历史快照](../guides/documentation.md#查阅历史原件)读取。
