# Sheltie 文档

[English](../en/README.md) | 简体中文

Sheltie 是给协调者 agent 使用的本地工作流引擎。Workbook 保存做事方法，Work 保存一次运行；引擎记状态、生成任务书、计算合法下一步并守门槛，内容判断和派活由协调者完成。

本文档面向方法作者、任务协调者、操作者与维护者，按 [Diátaxis](https://diataxis.fr/) 分为教程、操作指南、参考和解释。

## 从当前目标开始

| 需要完成什么 | 阅读入口 |
| --- | --- |
| 第一次了解如何运行 | [构建源码](how-to/build-from-source.md) → [第一个 Work](tutorials/first-work.md) → [门槛与成果](tutorials/gate-and-result.md) |
| 执行已授权的仓库修改 | [code-change](how-to/run-code-change.md)；需要先规划再逐任务实施时用 [spec-dev](how-to/run-spec-dev.md) |
| 编写或编辑方法 | [编写 Workbook](how-to/write-workbook.md)／[画布编辑](how-to/edit-workbook.md) |
| 装入、核验或移除方法版本 | [管理 Workbook](how-to/manage-workbooks.md) |
| 重开会话、返工或更换执行资格 | [接续 Work](how-to/resume-work.md) |
| 读取和复制最终成果 | [导出成果](how-to/export-results.md) |
| 安装、更新或卸载引擎 | [管理安装](how-to/manage-installation.md) |
| 命令报错或流程受阻 | [排查错误](how-to/troubleshoot.md) |
| 查参数、状态、文件和工具接口 | [技术参考](reference/README.md) |
| 理解概念、模块、事务与恢复 | [原理与源码](explanation/README.md) |
| 修改代码或维护文档 | [实施变更](how-to/implement-change.md)／[验证变更](how-to/validate-change.md)／[维护文档](how-to/maintain-docs.md) |

## 版本与阅读范围

首个对外支持基线为 `v0.3.0`。本文的操作和格式面向此源码基线；发布状态与可用附件以[发布记录](reference/releases/README.md)为准。不支持旧版本兼容或迁移；首次使用应选择新的显式管理根，详见[支持与兼容性](reference/limitations.md)。

引擎不会安装宿主资源、调用模型、决定报告质量或自动发布成果。作者工具和导出器是从源码使用的外围工具，各有独立文件与权限边界。

## 四类文档与规范依据

- [教程](tutorials/README.md)：固定练习路径，帮助学习并得到可观察结果。
- [操作指南](how-to/README.md)：按实际目标给前提、步骤、结果与失败处理。
- [参考](reference/README.md)：按产品结构查命令、数据、文件、工具和支持范围。
- [解释](explanation/README.md)：说明概念关系、设计原因与源码调用链。

[specs](../../specs/README.md) 定义产品必须满足的规则，精确字段、限额、错误和持久格式只在 specs/contracts 维护。docs 提供查阅摘要与使用材料，不另立规范。统一术语见 [CONTEXT](../../CONTEXT.md)。

实现、验证、用户接受、发布与可量化收益分别判断；当前实现和证据入口见[实现定位](reference/implementation.md)与[验收边界](reference/acceptance.md)。需要追溯时查[历史档案](history/README.md)；历史记录不作为当前操作步骤。
