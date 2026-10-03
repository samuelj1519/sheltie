# 正式run记录模板

这是待填模板，未填写时不是有效run.json、没有正式run ID。当前六规划槽位只在T02登记not_run。正式准入后每个run保存唯一目录，原stdout/stderr/exit、最终patch/candidate/检查/说明和所有活动不可覆盖。

| 字段 | 正式记录要求 |
| --- | --- |
| run_id/sample_id/arm | 实际分配非空ID；arm=native或sheltie |
| actor_id/assistant_ids | 真正稳定匿名使用者和所有助手 |
| prior_uses/actor_arm_use_index/exposure | 两组真实历史；本组prior+1，0才first_use |
| protocol_id/order_index/input_refs | 冻结协议/真实顺序；任务、候选、方法、模型/host/权限/binary全部原件 |
| started_at/ended_at/wall_seconds | 真实UTC时间和墙钟；未知/null，不当人工活动 |
| activity | 每项actor_id/phase/cost_bucket/start/end/minutes/evidence；所有作者/助手/维护/接受/盲审成本都覆盖 |
| interruptions/rework | 真关闭重开证据；natural与injected分开，未知原因明确unknown |
| usage | 宿主input/output/cache token/cost原值和来源；未知null |
| delivery_refs/acceptance_ref/quality_ref | 实际最终candidate/完整patch/说明/检查、外部接受、独立质量原件 |
| outcome/stop_reason | completed/failed/stopped/not_run及真实原因，不把not_run当使用次数 |

phase可为preparation/task_input/execution/verification/rework/resume/delivery/acceptance/final_review/maintenance；cost_bucket为shared_method/arm_setup/arm_maintenance/run/experiment_overhead。每活动只归一个桶，同人时间不重叠；失败和停止成本保留。原费用字段解释不明则不合并。
