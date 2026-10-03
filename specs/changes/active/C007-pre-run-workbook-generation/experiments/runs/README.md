# 六个规划槽位的未执行记录

这张表是计划位，未分配实际sample_id/run_id/actor_id，没有正式run.json或模型求解原件。本轮用户允许缺输入时记录后跳过；原六次对照、连续使用、首次读者、真实关闭重开、自然返工、接受与盲审义务全部保留not_run。M1技术资产审阅不能替代正式准入。

| 规划位 | 预期任务类型 | arm | 正式run_id | 结果 | 缺项 |
| --- | --- | --- | --- | --- | --- |
| 1A | 小bug | native | null | not_run | 真实任务/独立副本/actor历史/模型host/质量/预算缺失 |
| 1B | 同一小bug | sheltie | null | not_run | 同上；未启动正式Work，不把机制fixture计入 |
| 2A | 小功能 | native | null | not_run | 真实目标/检查/patch方法/actor与投入预算缺失 |
| 2B | 同一小功能 | sheltie | null | not_run | 同上；无组间配对输入闭包 |
| 3A | 自然返工情境 | native | null | not_run | 真自然发现/会话关闭重开/使用历史/独立质量缺失 |
| 3B | 同一返工情境 | sheltie | null | not_run | 同上；不注入缺陷制造自然收益 |

两组P/S/M/R及实验整体E、wall_seconds、usage/cost、人工分钟、接受与质量均unknown/null，不写0、completed或失败数量。not_run不增加prior_uses，slot编号不推first_use/reuse。作者与技术机制接触必须在未来实际参与时披露，不能当未经接触的首次读者。

补验：提供三项尚未解决真实目标/验收/初始身份和独立副本、实际连续使用者与每组历史、同模型/host配置/权限、正式预算/阈值/顺序及质量/接受主体；冻结protocol/method/input/binary，独立正式准入通过后按first-use执行。在新目录追加真正原件，不覆盖本表或技术准备。预算/输入漂移/非预期退出停止，失败时间保留。
