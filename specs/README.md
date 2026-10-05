# Sheltie 文档

Released：[`v0.2.0`](releases/v0.2.0/README.md)，发布目标为 `aarch64-apple-darwin`
开发目标：`v0.3.0`
Active change：无

这里收集 Sheltie 的产品定义、实现原理、精确合同和维护方法。Sheltie 是给协调者使用的本地工作流引擎：人把方法写成 Workbook，引擎记状态、发任务书、限定合法下一步、守门槛，内容判断由人或 agent 完成。

当前源码包含尚未发布的开发能力。产品定位与支持范围见[产品规格](spec.md#当前产品环境)；验收与价值边界见[当前限制](limitations.md)。已完成变更不表示版本已发布。

## 从哪里开始

| 读者与目的 | 建议阅读路径 |
| --- | --- |
| 第一次了解项目 | [项目 README](../README.md) → [领域词汇](../CONTEXT.md) → [产品规格](spec.md) |
| 想从源码运行一次任务 | [源码快速开始](guides/source-quick-start.md) → [接续与错误处理](guides/continuity-choices.md) |
| 想了解源码如何工作 | [源码导读](guides/source-tour.md) → [架构](architecture.md) → 对应合同与测试 |
| 想编写或编辑方法 | [Workbook 合同](contracts/workbook.md) → [样例](../examples/)／[spec-dev](../workbooks/spec-dev/README.md) → [可视化作者工具](../tools/workbook-editor/README.md) |
| 维护代码或准备变更 | [工程规范](engineering.md) → [change 索引](changes/README.md) → [完整行为实施指南](guides/proposal-implementation.md) |
| 理解设计取舍或旧版本 | [设计决定](decisions/README.md) → [变更摘要](changes/README.md)／[发布记录](releases/README.md) |

## 当前权威

| 文档 | 职责 |
| --- | --- |
| [constitution.md](constitution.md) | 产品存在的理由与不可破坏的不变式 |
| [../CONTEXT.md](../CONTEXT.md) | 全仓库统一的领域词汇 |
| [spec.md](spec.md) | 当前开发线的产品行为、环境和验收要求 |
| [architecture.md](architecture.md) | crate 边界、类型、状态机与文件布局 |
| [contracts/workbook.md](contracts/workbook.md) | Workbook／Flow 格式、引用与编译规则 |
| [contracts/protocol.md](contracts/protocol.md) | CLI、响应、错误与协调者操作合同 |
| [contracts/storage.md](contracts/storage.md) | Store、请求身份、事务和文件恢复合同 |
| [engineering.md](engineering.md) | 编码、测试、验证、提交与审查规则 |

冲突时按「宪章 → 规格 → 架构 → 合同」裁决。实施前先同步上游定义，active change 的 plan 只规定执行顺序与进度。没有 active change 时，不从 proposed 自行选择方案实施。

开发目标描述当前源码候选的基础版本；数值产品 active 目标必须与它一致。发布事实始终按 release record、tag 和验收闭包核对。

## 操作、解释与历史

- [guides/](guides/README.md)：可执行教程、操作指南和维护方法。
- [decisions/](decisions/README.md)：重要选择的背景、备选方案与后果。
- [roadmap.md](roadmap.md)：未来方向及其需求条件。
- [limitations.md](limitations.md)：支持、信任、兼容性与仍未知的价值边界。
- [changes/](changes/README.md)：提案／实施入口和已完成变更的参考摘要。
- [releases/](releases/README.md)：已发布版本、固定源码、实物与已知限制。
- [research/](research/README.md)：外部来源与阅读线索，不具有项目权威性。

文档维护、完成后收敛与历史原件读取见[文档维护指南](guides/documentation.md)。修改后运行 `scripts/check-docs.sh` 和 `scripts/check-specs.sh`。
