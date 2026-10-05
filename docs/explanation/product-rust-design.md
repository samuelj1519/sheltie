# 产品设计与 Rust 工程来源

核对日期：2026-10-03。本文保存一手资料及其适用范围，不具有产品或实施权威；方案选择仍由项目目标、真实需求和不变式决定。

## 用户问题与完整结果

[GOV.UK：Learning about users and their needs](https://www.gov.uk/service-manual/user-research/start-by-learning-user-needs)要求先理解可能的使用者、现行做法、困难和所需结果，并将未经用户验证的建议作为假设。其服务领域不同于本项目；可迁移的是问题先于解决方案、观察真实使用者、按结果验证需求的方法。

项目采用这一方法区分方法作者与任务使用者，先观察完整任务的首次使用、复用及中断成本，再决定通用增量。历史实验的样本范围是项目预算选择，不是该资料规定的统计样本量；有限样本不能证明普遍收益，评估方法见[效果评估](../how-to/evaluate-workflows.md)。

[GOV.UK：Measuring success](https://www.gov.uk/service-manual/measuring-success)提供服务绩效与收益测量入口。项目将总投入和交付质量分开报告，包含准备、核对、失败和未完成；不从工作流状态或局部省时推断用户价值。这是本项目对目标的推导，不是通用强制指标。

## Rust 的具体类型与真实消费者

[Rust API Guidelines checklist](https://rust-lang.github.io/api-guidelines/checklist.html)列出类型安全、文档、可靠性和未来演进的检查维度。对本项目有用的是校验类型、私有字段、可定位错误、明确合同和实际调用者；不因为存在指南就为每个模块增加 trait、公开构造器或抽象平台。

[The Rust Programming Language：Test Organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html)区分模块内部单元测试与从公开接口使用库的集成测试。项目以纯 core 规则、真实 runtime/SQLite/文件和 CLI 消费者分别验证；手写期望、来源/恢复 oracle、同闭包复用和验证预算由项目实际失败义务决定。

## 设计裁决

[cargo-dist 配置](https://axodotdev.github.io/cargo-dist/book/reference/config.html)允许 package 的 dist=false 排除分发，workspace packages 列表可以覆盖逐包选择。项目锁定 0.32.0 的本地 config/v0.rs 与 announce 实现也支持 Cargo 元数据位置。外围 `sheltie-export` 通过自身 Cargo 元数据排除分发；新增 binary 时须核实际 dist 计划，避免自动进入已配置发布集合；这不授予运行 release 的权限。

方案中采用的同步 core/runtime 分工、单一 SQLite 权威、单一目标格式、正常接口的幂等恢复、独立审查与数据保留来自项目宪章和合同，不声称四份资料规定了这些具体机制。最佳实践不等于机制清单；只有承担当前保证或实际需求的机制才进入产品。
