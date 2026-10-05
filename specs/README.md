# 当前开发规范

Released：[`v0.2.0`](../docs/reference/releases/v0.2.0/README.md)
开发目标：`v0.3.0`
Active change：无

specs 只保存当前规范与变更入口。学习、操作、源码定位和历史参考见 [docs](../docs/README.md)。

| 依据 | 内容 |
| --- | --- |
| [constitution.md](constitution.md) | 产品边界与不变式 |
| [spec.md](spec.md) | 当前采用行为、支持环境与验收要求 |
| [architecture.md](architecture.md) | 分层、状态、文件与恢复约束 |
| [contracts/](contracts/README.md) | Workbook、CLI 与 Store 的精确定义 |
| [engineering.md](engineering.md) | 编码、测试、验证、提交与审查规则 |
| [changes/](changes/README.md) | 提案、实施与模板；active plan 是进度唯一权威 |
| [roadmap.md](roadmap.md) | 未采用方向及需求条件 |

统一术语见 [CONTEXT](../CONTEXT.md)。冲突按「宪章 → 规格 → 架构 → 合同」裁决；实现不符时修实现或明确采用上游变更，不把缺陷自动写成规范。没有 active change 时不自行实施 proposed。

当前源码使用 0.3.0-rc.1，产品环境为 macOS aarch64／APFS；已实现、已验证、已发布和已证明收益分别判断。查[实现定位](../docs/reference/implementation.md)、[验收边界](../docs/reference/acceptance.md)和[发布记录](../docs/reference/releases/README.md)。开发目标规定源码基础版本，数值产品 active 目标必须一致，不替代发布 tag 与实物。

修改后运行 `scripts/check-docs.sh`、`scripts/check-specs.sh`；涉及测试声明时加 `scripts/check-tests.sh`。
