# 实现与测试定位

本文将当前采用能力与公开入口、实现和回归测试相连，供维护者定位调用链。当前基础版本与 active change 只在[规范入口](../../specs/README.md)定义；修改后的检查以实际候选与输入闭包为准。

下表列出已存在的完整实现路径，不将“有函数／有测试”单独当作验收通过。历史资格及未执行边界见[验收依据](acceptance.md)和对应变更记录，改动后的验证按实际输入闭包重新执行。

| 采用要求 | 当前实现 | 回归入口 |
| --- | --- | --- |
| GF-01–GF-05：业务无关、显式图、定义与运行分离 | [解析与编译](../../crates/sheltie-core/src/flow)、[状态决策](../../crates/sheltie-core/src/work/decide.rs)、[Workbook 仓库](../../crates/sheltie-runtime/src/workbook_repo.rs) | [code-change](../../crates/sheltie-cli/tests/scenario_code_change.rs)、[article-review](../../crates/sheltie-cli/tests/scenario_article_review.rs)、core 编译测试 |
| GF-06–GF-09：任务书、冻结输入、输出封存与有界回复 | [输入与输出观察](../../crates/sheltie-runtime/src/observe.rs)、[文件句柄](../../crates/sheltie-runtime/src/fsx.rs)、[渲染](../../crates/sheltie-core/src/work/render.rs) | [输出路径](../../crates/sheltie-cli/tests/output_paths.rs)、[成果](../../crates/sheltie-cli/tests/scenario_artifacts.rs)、[文件边界](../../crates/sheltie-runtime/tests/fs_boundary.rs) |
| GF-10：同快照状态与 resume | WorkService::status_read、Store::read_work_bundle、StatusReadView | [状态查询](../../crates/sheltie-cli/tests/work.rs)、[严格快照](../../crates/sheltie-cli/tests/snapshot_qualification.rs) |
| GF-11–GF-14：失败、撤销、门槛、额度、终态与成果选择 | core 的 decide／legal_next／result_view，runtime 的 replace／result | [替换](../../crates/sheltie-cli/tests/attempt_replace.rs)、[门槛](../../crates/sheltie-cli/tests/scenario_gated_release.rs)、[最终成果](../../crates/sheltie-cli/tests/work_result.rs) |
| GF-15–GF-16、GF-30–GF-32：请求身份、拒旧、预检与可信恢复 | [request](../../crates/sheltie-runtime/src/request.rs)、[load](../../crates/sheltie-runtime/src/load.rs)、[Store](../../crates/sheltie-runtime/src/store)、[recovery](../../crates/sheltie-runtime/src/recovery.rs) | [重放](../../crates/sheltie-cli/tests/replay.rs)、[效果](../../crates/sheltie-runtime/tests/effect_contracts.rs)、[崩溃恢复](../../crates/sheltie-cli/tests/reliability_crash.rs)、[start 预检](../../crates/sheltie-runtime/tests/start_preflight.rs) |
| GF-17、GF-28：方法版本、冻结副本与 requires 声明 | WorkbookRepo、workbook-digest/v2、WorkService::start；requires 只传递声明 | [方法生命周期](../../crates/sheltie-cli/tests/scenario_workbook_lifecycle.rs)、[Workbook](../../crates/sheltie-cli/tests/workbook.rs) |
| GF-18–GF-19：协调者说明与诚实交付 | [Sheltie skill](../../skills/sheltie/SKILL.md)、单一合同生成与人工／工程边界 | [skill 交付](../../crates/sheltie-cli/tests/skill_delivery.rs)、[skill](../../crates/sheltie-cli/tests/skill.rs)、验收记录 |
| GF-27：self 安装、更新、回滚与 purge | [selfmgmt](../../crates/sheltie-runtime/src/selfmgmt.rs) | [self](../../crates/sheltie-cli/tests/self_cmd.rs)、[远端更新](../../crates/sheltie-cli/tests/remote_update.rs)、runtime selfmgmt 测试 |
| GF-29：事实统计与 Workbook 反思 | [统计渲染](../../crates/sheltie-core/src/work/render.rs)、[spec-dev](../../workbooks/spec-dev) | [spec-dev 场景](../../crates/sheltie-cli/tests/scenario_spec_dev.rs)与 core 渲染测试 |
| GF-33：原字节读取与可编辑新副本 | WorkService::write_result_artifact、[CLI raw 入口](../../crates/sheltie-cli/src/commands/work.rs)、[sheltie-export](../../crates/sheltie-export/src) | [raw 成果](../../crates/sheltie-cli/tests/result_artifact.rs)、[外围导出测试](../../crates/sheltie-export/tests) |
| GF-34：本地可视化作者工具 | [工具](../../tools/workbook-editor)通过公开 CLI 核草稿并输出 ZIP | [工具测试](../../tools/workbook-editor/test)、C011 限定可用性与实际用户记录 |

## 未实现或未采用的方向

GF-20–GF-26 的宿主安装与就绪机制、跨版本延续、宿主隔离、多宿主 MCP、自动成本门禁、并行草稿与动态展开不作为当前已实现能力。C008 完成的是声明前检，探针未采用。详细触发条件见[路线图](../../specs/roadmap.md)，不在 docs 中提供占位运行命令。

## 使用此基线

维护一个行为时，从要求找到公开入口、完整调用链、正反 oracle 和恢复窗口，再决定受影响验证。若源码与规范不符，记录准确差异；不能把未执行、不知道或被排除的环境覆盖改成通过。docs 的教程和操作应使用表中已有公开能力，未发布格式使用新的显式管理根。
