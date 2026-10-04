# C010 实施计划

| ID | 状态 | Owner | 依赖 | 结果 |
| --- | --- | --- | --- | --- |
| C010-T01 | done | Root；独立 Reviewer | 用户采用 | 范围、合同和测试后继独立短审通过 |
| C010-T02 | done | Root；code-simplifier | T01 | 局部实现、完整测试后继与新方法工程闭包通过 |
| C010-T03 | todo | Root；独立 Reviewer | T02 | 固定候选、完整验证、独审与归档 |

### C010-T01 固定范围与方法目标

**入口。** 本 package、当前 specs 路由与 Workbook 合同。引擎保持 schema 4、cli-result/v4、work-result/v1；方法 0.2.2 使用已有终点选择和 resource 绑定能力。

**步骤。** 记录已证明的局部重复、测试后继与不采用的产品变更；同步方法目标，交未参与设计或实现的 Reviewer 核边界。

**验证。** docs/specs/tests、git diff --check、独立范围审查、check-task 的本任务开工基线。

**提交。** `docs(specs): 固定局部精简与方法交付范围`。

### C010-T02 收敛实现与完整方法

**生产入口。** runtime load/service/workbook_repo/fsx/effects；export error。删除恒假比较、无效赋值与未用映射；各入口仅计算一次不可变 intent 摘要；purge 三个原时点复用枚举，发布两分支仅复用相同顺序的收尾。保留公开 add/remove 快照类型，不为减少几行引入通用 trait。纯解码的大范围重构不采用。

**方法入口。** spec-dev manifest/Flow、scaffold/implement/verify 说明书、共享审批 resource、README、CHANGELOG。版本升 0.2.2。共享规则由三个 Node 的 approval_rules 必需输入绑定；普通审批核对的位置仍在各自开始执行前，人工更正引用规则只定义一次。retro 的 delivery 输入与 lessons 输出声明 result=true，其他槽不选。

**消费者补齐。** core/testkit 的静态内存文件表也登记共享 resource，仍由 include_str 和手写索引提供独立文件观察；不把 I/O 引入 core。首次消费者运行实际发现该表缺项，保留失败原文并在修复后验证全部消费者。

**分工。** code-simplifier 独占上述生产 Rust 文件；Root 独占方法、测试、规格与验证。混合源码只改生产区，保留内嵌测试。双方不覆盖他人编辑。Reviewer 不代写修复。

**正例。** 原状态/next/历史响应和原字节相同；同 Home 缺 tag 拒绝后成功更新且 executable；新方法规则路径随 Work 冻结；gate 批准后成果只有 delivery/lessons，分别来自 retro input/output。

**反例。** 非法持久路径、旧图/输入被改、未知字段、审批更正缺引用、未批准 gate、pending/sync 故障仍按原合同拒绝；不覆盖或误删其他对象。

**验收用例。** 在既有真实 CLI 支持上补一个完整方法结果用例，先在未改方法上观察批准后无明确成果的行为失败；同时验证三个规则绑定与原字节读取。新增测试归 C010-T02；自更新旧例只在完整后继运行通过后退休，历史任务卡记录后继。

**测试。** `spec_dev_final_results_are_frozen_and_wait_for_gate_approval`。

**停止条件。** 若局部抽取改变观察时点、诊断或 original 保留资格，修正抽取；若需要产品/Store 改义，保持当前行为并在本 package 记录原因。不得通过削弱断言使检查通过。

**验证。** 先运行新 CLI 用例的真实红/绿与受影响消费者；固定实现输入后运行 fmt/check/clippy、全 features nextest、doctest、MSRV 1.85 locked、deny、docs/specs/skill/tests/core-vocab 与 task 范围。nextest 至少 0.9.145；RUSTC_WRAPPER 为空，明确可写 target。错误、LEAK、skip 与执行数逐项保留。

**提交。** `refactor(runtime): 精简局部重复并补齐方法成果`。

### C010-T03 独立审查与归档

未参与设计/实现/oracle 的 Reviewer 核完整 diff、全部真实消费者、测试迁移与原始验证。Root 处理 finding，Reviewer 复核；没有输入变化不重复长验证。全部采用义务完成才移到 completed，更新当前入口。真实净收益、宿主、发布均不在本次范围。

**验证。** 采用闭包、独立 review、文档治理、最终范围与工作树；check-task 允许本 package 移动前后路径。

**提交。** `docs(specs): 归档局部精简与方法交付验证`。
