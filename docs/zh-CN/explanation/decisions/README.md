# 设计决定

[English](../../../en/explanation/decisions/README.md) | 简体中文

本目录保存重要设计选择的背景、备选方案和后果，帮助维护者理解当前系统为什么这样工作。当前字段和行为以[规格与合同](../../../../specs/README.md)为准；按迭代追溯结果见[历史档案](../../history/README.md)。

按下面的主题选择记录，无需按编号通读。`accepted` 表示曾被采用；具体选择的当前适用范围见表格和各记录，不能由该状态推断所有历史格式仍有效。

## 存储、格式与恢复

| 决定 | 状态 | 当前适用范围 |
| --- | --- | --- |
| [D-033：Store schema 2 明确拒旧](D-033-store-schema-2.md) | accepted | 单一格式、保留旧数据并明确拒绝的原则继续适用；schema 2／cli-result/v2 已由 D-040、D-041 的格式选择替代 |
| [D-034：目录摘要的带长度编码](D-034-workbook-digest-v2.md) | accepted | workbook-digest/v2 与输出路径别名约束 |
| [D-035：管理根写锁](D-035-root-write-lock-fs4.md) | accepted | 写操作串行化、锁对象身份与 revision 校验 |
| [D-037：受管文件与安全 API](D-037-managed-file-handles.md) | accepted | 目录句柄、文件对象与字节核验；平台范围见 D-044 |
| [D-038：purge 保留根锁](D-038-purge-lock-lifecycle.md) | accepted | 清空数据时保留根与原锁，准确报告部分清理 |
| [D-039：只读 SQLite 控制文件边界](D-039-sqlite-read-control-files.md) | accepted | 业务只读与 SQLite 控制文件允许变化的区别 |
| [D-040：成果与接续的单一格式](D-040-result-resume-format.md) | accepted | 冻结结果与同快照接续原则继续适用；schema 3／cli-result/v3 已由 D-041 替代 |
| [D-041：创建顺序号与行政替换](D-041-attempt-number-and-replacement.md) | accepted | 当前 schema 4／cli-result/v4、number 与一次替换资格 |

## 身份、授权与成果交付

| 决定 | 状态 | 当前适用范围 |
| --- | --- | --- |
| [D-036：真实 OS 操作主体](D-036-os-principal-from-uid.md) | accepted | 审计与批准的账户记账，不提供独立真人认证 |
| [D-042：原字节与外围副本](D-042-final-artifact-copy.md) | accepted | 公开成果读取、独立导出、不覆盖发布与失败边界 |
| [D-044：macOS／APFS 产品范围](D-044-macos-apfs-product-scope.md) | accepted | 当前支持与验收范围；不追溯补证历史覆盖或收益 |

## 工程与文档治理

| 决定 | 状态 | 当前适用范围 |
| --- | --- | --- |
| [D-032：change package](D-032-use-change-packages.md) | accepted | 采用范围、active plan、提交归属和完成资格；文档分类由 D-045 细化 |
| [D-043：开发目标权威](D-043-development-target-authority.md) | accepted | 未发布候选的基础版本与实施进度、发布身份分别管理 |
| [D-045：规范与技术文档分工](D-045-specs-and-diataxis.md) | accepted | specs 保存规范，docs 按读者用途组织，闭合变更归历史档案 |

## 阅读和维护记录

MVP 的 D-01–D-31 设计理由见[MVP 摘要](mvp.md)，其中已标出部分后续替代关系。里程碑复核、首次运行和逐轮审查的完整原件从[固定历史快照](../../how-to/maintain-docs.md#查阅历史原件)读取。

一个重要决定使用一个 Markdown 文件，保存状态、采用日期、关联 change、背景、选择、备选方案、后果和确认方式。编号与原采用理由保留，当前适用范围在首屏说明。

决定整体被替代时，状态改为 `superseded by D-nnn` 并双向链接。只有部分选择被替代时，保留原采用状态，逐项说明失效部分、仍有效原则和后续决定，不将整份记录判作失效。历史格式与当前规范之间的差异应明确，不能悄悄改写原采用事实。
