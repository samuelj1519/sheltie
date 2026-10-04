# C009 实施计划

| ID | 状态 | Owner | 依赖 | 结果 |
| --- | --- | --- | --- | --- |
| C009-T01 | done | Root；独立 Reviewer | 用户采用 | 范围、测试后继和当前文档语义明确 |
| C009-T02 | done | Root；code-simplifier；测试实现者 | T01 | 重复实现与测试收敛，消费者行为保持 |
| C009-T03 | doing | Root；未参与实现的 Reviewer | T02 | 固定候选、完整验证、独立审查与归档 |

### C009-T01 固定范围与修正文档

**入口。** specs 当前入口、GF-12、C006/C007 README、本 package。先明确 gate 的 approve/cancel，与架构和已有实现一致；原实验与新 agent 协议分开表述。旧测试引用按已有退休注解方式记录后继，不改历史验收原件。

**验证。** `scripts/check-docs.sh`、`scripts/check-specs.sh`、`scripts/check-tests.sh`、独立范围审阅、`scripts/check-task.sh C009-T01 <本任务基线> --staged`。

**提交。** `docs(specs): 明确项目精简范围与验证边界`。

### C009-T02 收敛实现与测试

**生产入口。** core 的 render/decide；runtime 的 service、store、pending、workbook_repo、home/observe；CLI 的 output/attempt。按具体类型共享组装，保留纯 core 与 I/O runtime 分工。只删除证明不可达或无真实调用者的代码，不新增通用执行框架。

**测试入口。** 见 test-disposition 与 tasks。core/runtime/CLI/export 的正反例、真实文件与进程、不同崩溃窗口和字节 oracle 保留。可共享复制、SQL/文件现场快照及 kill 夹具；快照字段不能减成最弱集合。spec-dev 保留代表性的完整 CLI 绑定闭环，其余检查器负例使用较小的独立文件/Git 夹具。

**分工。** code-simplifier 独占 core render/decide 和 runtime 生产重构；CLI/export 测试作者独占其测试文件；runtime 测试作者独占其测试及内嵌支持；Root 独占 CLI 生产、文档与验证。混合文件须按明确模块协调，任何跨所有权需求先交 Root。

**正例。** 相同输入产生相同状态、next、任务书、统计、成果和重放响应；原件保持相同字节与身份。

**反例。** 未批准 gate、未知字段、损坏 audit/effects、被替换文件、提交前后故障继续按既有合同拒绝；合法 original 不因效果错误丢失。

**停止条件。** 需要改变错误优先级、跨时间观察、文件效果或外部合同的候选退回本 package 另行分析；不以减少代码为由弱化断言。

**验证。** 先运行受影响消费者与任务测试，再固定输入运行 fmt/check/clippy、全 features nextest、doctest、deny 与 docs/specs/skill/tests/core-vocab；nextest 至少 0.9.145，不越过版本检查。所有构建清空 RUSTC_WRAPPER 并使用明确可写 target。

**测试。** `two_step_via_cli_reaches_succeeded`、`commit_workbook_on_readonly_store_preserves_sqlite_error_and_rows`、`handwritten_current_schema_control_accepts_both_open_modes`、`open_creates_schema_with_current_user_version`、`external_file_rejects_symlink_directory_and_missing`、`external_file_sha256_matches_known_vector`、`concurrent_initialization_fixture_joins_the_waiting_writer_after_failure`。其余消费者和迁移用例按 validation 的实际 inventory 核对。

**提交。** `refactor(runtime): 收敛重复投影与测试支持`。

### C009-T03 独立审查与收尾

未参与实现者核 T02 的完整 diff、测试处置、真实消费者和原始验证；Root 修复其 finding，Reviewer 复核。仅输入变化时补受影响验证。记录源码候选与输入闭包，不把计数下降、工程 PASS 或运行模式当用户净收益。任务完成后移动到 completed 并更新两个当前入口；不推送、发布或替换用户安装。

**验证。** 完整采用闭包、独立 review、文档治理、任务范围与最终工作树。

**提交。** `docs(specs): 归档项目精简验证与独立审查`。
