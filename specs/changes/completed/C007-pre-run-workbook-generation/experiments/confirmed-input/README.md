# 固定代码任务 Workbook

身份：`code-task-study@1.0.0`，Flow `default`。这是 C007 私有技术资产，未发布、未正式试用；实际 CLI 装入原件见 [preparation](../preparation/README.md)。装入可用不代表正式实验准入、真实质量或价值已通过。

完整审阅必须读取 [workbook.toml](workbook.toml)、[default.toml](flows/default.toml) 和三个说明：[implement](instructions/implement.md)、[review](instructions/review.md)、[deliver](instructions/deliver.md)。`workbook show` 的摘要不代替这些文件。两组直接读取同一三份说明及 [shared-method](../shared-method.md)，共有策略逐字相同。

| 节点 | 输入 | 必需输出 | 访问 / 失败重试上限 |
| --- | --- | --- | --- |
| implement | task、project、可选 previous-review | change.md、checks.md | 3 / 1 |
| review | task、project、change、checks | review.md | 3 / 1 |
| deliver | task、change、checks、review | delivery.md、change.patch | 1 / 1 |

入口 implement；implement → review 为 main，review → implement 为 back，review → deliver 为 main。全部 `gate = false`，没有 `requires`。报告各 262144 字节，patch 8388608 字节；达到预算或次数边界就停止记录。

deliver 明确选择五个唯一成果：输入 change/checks/review、输出 delivery/patch。输入绑定终点开工时的真实引用，输出绑定其封存引用；其他草稿或历史不自动列入成果。patch 必须按真实 project 预声明方式包含未跟踪文件并核可应用，不猜项目命令。

真实 task/project、初始副本、参与者与历史、宿主、预算和质量仍 pending。运行入口及停止条件见 [first-use](../first-use.md)，正式准入见 [protocol](../protocol.md)。
