# Git 提交信息迁移参考

[English](git-message-migration.md) | 简体中文

本次经授权的英文默认迁移翻译了 260 个可达本地提交的信息。每个提交的文件树、所有非父提交头字段（包括作者、提交者和时间）、父提交顺序与拓扑，以及机器 trailer 均保持。父提交 ID 和提交信息中可解析的提交引用按[新旧映射](git-message-migration.tsv)更新。

映射保留原身份供追溯。历史文件树和原始证据未被翻译或改写；当前文档与工具引用使用映射后的本地 ID，冻结历史文件内的原 ID 可查映射解析。实物 SHA256 保持不变。

本地分支、轻量发布与审查 tag、提交归档引用及 stash 通过一次带旧值守卫的事务更新，共 43 个引用。远端跟踪引用保留实际已观察的上游身份，原映射 blob 保持字节。已发布远端 tag、Release、工作流与 GitHub 提交 URL 仍使用原身份；发布记录明确区分本地映射提交与原发布提交。本次未推送或新增发布。

历史查询先在映射首列查原 ID，取第二列执行 `git show <mapped-id>:<path>` 或 `git archive <mapped-id>`。映射提交保存完全相同的原文件树，其中的历史中文仍保留。参见[历史查询](../how-to/maintain-docs.zh-CN.md#查阅历史原件)。

原历史 bundle 及原引用、消息快照保存在被忽略的本地目录 `output/C012-english-default-20261005/`。bundle SHA256 为 `3013ae9b61f68098526542f012cbaf1cdce0f20a5e83b2da08477ebd4863e294`。以后清理历史前保留这份备份；它是本地产物，不随源码克隆交付。

采用任务、精确对象核验、应用记录与最终检查保存在 [C012 package](../../specs/changes/completed/C012-english-default/README.md)；本次范围不扩大历史产品验收或发布证据。
