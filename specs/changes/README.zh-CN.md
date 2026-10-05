# 变更入口

简体中文 | [English](README.md)

Active change：无

C012 已完成用户明确授权的英文默认迁移、中文文档与方法保留，以及本地历史提交信息翻译；目前无 proposed、active 或 rejected change。人采用提案后才执行 plan，一次只允许一个 active change。

```text
proposed ──人采用──▶ active ──实现、验证、独立审查──▶ completed
    └──人否决──▶ rejected
```

模板见 [templates](templates/README.zh-CN.md)，实施方法见[指南](../../docs/how-to/implement-change.zh-CN.md)。当前进度只看 active plan；completed 不等于已发布或所有实验通过。

完成资格先保存在完整 package。固定 Git 快照后，将长期参考移入 [历史变更](../../docs/history/changes/README.zh-CN.md)，按[维护指南](../../docs/how-to/maintain-docs.zh-CN.md#3-完成后收敛)清理实施目录。历史快照仍接受完成门禁核验；specs 不重复保存已关闭任务摘要。

[C012](completed/C012-english-default/README.md) 已完成授权范围内的英文默认迁移与本地提交信息翻译；未推送或发布，模型效果与真人接受未执行。
