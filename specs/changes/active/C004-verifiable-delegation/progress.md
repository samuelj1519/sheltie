# C004 交接

负责人 Codex /root；core 作者 /root/later_plans、runtime 作者 /root/code_state；局部简化作者 /root/simplify_c004；独立 Reviewer /root/independent_review，不参与合同、测试或实现。采用范围与环境跳过边界见 [adoption](adoption.md)，当前进度只看 [plan](plan.md)。

T00 提交：`3d6f294`；T01 开工基准为该提交。T01 当前创建了真实 core/result/resume、同快照读取和阶段用例，stage-1.md 列实际接口与命令。首轮独立审查抓到 schema fixture、Resource 引用完整性、完整载荷解码顺序三项，已交作者修复。三项初审缺口及后续卡片/实时投影职责缺口均修复并独立复核，最后完整现行回归 728/728 通过。M1 独立阶段准备通过，冻结基准为 `0cedb8c1f3ee7ba8d4041151a5382d3b9e8abb4e`。下一动作是 T02 接通公开命令；T02 的公开 result 尚未接通。

本机 nextest 0.9.140 与配置最低 0.9.145 不符；原 task 命令退出 92、没有测试执行。安装版本 override 实际跑 T01 原语 25/25；真实 T02 caller 2/2 预定失败。最终基础回归为 run `6acddb6a-86d1-44d9-a278-f785c715cf6a`，728/728 PASS、13 后续 ignore；前述准备过程 run 不覆盖修复后的闭包。具体原文与环境边界见 [validation](validation.md) 和 evidence。

T02 draft 已接公开接口；冻结用例路径别名失败由 T05 复杂作者修复，保持 8 个 CLI / 3 个 runtime ignore 的独立测试提交后，再恢复自己已保全的 T02 draft 并以新 SHA 继续。没有他人未归属变更。

最新独立测试修复基准：`2914e8564047809659c4610d151f9ea6ceef7476`。T02/T03 使用该完整 SHA；T02 9 个源码/版本/测试草稿逐字节恢复，plan/tasks 保留 T05 并与 T02 状态/范围合并。

T02 公开 result 与 status 全视图已接通，最新冻结 11/11 及完整工程 739/739 通过；一次旧 SIGKILL 用例 LEAK 的 cause unknown，单次诊断未确认缺陷，原文与边界已记录。下一动作是 T03 接通首次使用/协调者说明并启用已冻结方法场景；基准仍为2914e8564047809659c4610d151f9ea6ceef7476。
