# C005 交接

状态：`proposed`。尚未采用；实施与真实使用均为 `not_run`。任务状态只在 [plan](plan.md) 维护。

| 字段 | 当前值 |
| --- | --- |
| 源码基线 | 见 README |
| 实施候选与 Owner | 无；采用时指定 |
| 当前任务 / 下一任务 | 未开始 |
| 已确定的接口 / oracle | 候选方案见 spec/design/validation；未写入上游 |
| 阶段 / M1/M2 结论 | 未进入；全部 not_run |
| 骨架 / 最新独立测试修订完整 SHA | 无 |
| 冻结命令 / runbook | 待 T01 创建 verification/commands.sh、experiments/runbook.md |
| T01 原语 green / T02 行为 red | 未执行 |
| 原始命令与 run ID | 无 |
| 未完成验证 | 全部实施、真实使用与 M1/M2 |
| 采用输入 | 真实撤销需求、旧进程停止或工作区隔离条件 |
| 下一个动作 | 人决定采用；不自行实施 proposed |

采用后交接保存具体 Work、Attempt、当前任务书指针、候选、实际范围和未跑项。当前合法操作以新的 `work status` 为准，不能照历史请求的 next 推进。人工进程检查不写成引擎已核实事实。

简单实现者开工必须核 M1、骨架或独立测试修订完整 SHA、冻结接口与 oracle、允许文件、非零有效 red 和停止条件。遇合同/原语/测试缺口，把真实 caller、最小复现和输出交复杂作者；测试修订独立复核后更新 SHA，不能沿用已漂移的测试基准。
