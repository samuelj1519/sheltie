# C012：英文默认项目迁移

[English](../../../../en/history/changes/C012-english-default/README.md) | 简体中文

状态：`completed`
目标版本：`v0.3.0`
兼容性：`minor；保持产品语义、协议、持久化格式和历史文件树`
基线：`206a944491acfa9fa4843ced06103427e13054aa`
负责人：Codex 协调者、实施 agent、独立 Reviewer
记录形式：`reference`
历史快照：`eec267b7b54b1e18e75578f7b5e269e7ddf73df8`

## 变更与理由

文档、开发说明、agent 指令、代码注释、Rust API 文档、CLI 文案、状态卡和 Workbook 编辑器以英文为默认语言。保留明确可选的中文文档与方法版本，并提供语言导航。有行为用途的 Unicode fixture、用户内容、原始证据和持久化审计标记保持原样；[语言清单](inventory.md)记录保留内容及用途。

翻译 260 个历史提交信息并映射 43 个本地引用，精确核验文件树、非父节点元数据、身份、时间戳及映射后的有序父节点。同步当前引用，保留原始发布身份和远端跟踪引用；[迁移映射](../../git-message-migration.md)及已核验的本地恢复 bundle 保留追溯入口。未来提交使用英文。原 MIT 条款与所有权在标准根 LICENSE 中保持不变，并补充 GitHub 贡献、支持、安全、issue 和 pull request 入口。

## 验证与限制

已记录的工程检查通过：956/956 Rust 测试、5 个 doctest、格式检查、全 target／全 feature 编译与 Clippy、锁定依赖的 Rust 1.85 编译、72/72 编辑器测试、独立编辑器与 ZIP 完整性检查、10 个双语 CLI 场景、语言与 skill 拒绝控制及独立审查。一次刷新后的依赖审计通过；之后一次公告库获取因 TLS 失败，最终提交门禁使用此前已审计、固定在 `ef6173cbc5c50ec8166f9a5b28f07834144373ee` 的公告库通过。该获取失败仍为 FAIL。

agent 语言效果、真人接受、对外发布、远端历史推送及其他平台验证仍为 `not_run`。CLI 场景使用合成产物和测试批准。已有受管用户数据、已安装方法及冻结副本保持原样。外部批量翻译路线已获授权，但翻译由并行 agent 完成，未执行该外部批处理。本参考记录保存历史工程结果，不为后续候选提供验证资格，也不扩大产品验收范围。

## 参考

- [语言清单与保留数据](inventory.md)。
- [Git 提交信息迁移与恢复](../../git-message-migration.md)。
- [当前语言维护规则](../../../how-to/maintain-docs.md#英文默认与中文版本)。
- [历史任务映射](../../../../../scripts/task-history/C012/ledger.md)。

完整计划、验证、独立审查、对象比较和原始日志保存在上述快照的 `specs/changes/completed/C012-english-default/`。按[历史查阅指南](../../../how-to/maintain-docs.md#查阅历史原件)读取，例如：

```bash
git show eec267b7b54b1e18e75578f7b5e269e7ddf73df8:specs/changes/completed/C012-english-default/validation.md
```
