# C004事前规程：agent真实交付与冷接续

本次为真实下一C005协调者生成新开工包。不是旧机制fixture或人类配对收益。冻结源/方法/输入/标准/预算后才启动。Root协调者应用仓库skills/sheltie/SKILL.md，写状态只通过CLI；使用专用Home与当前冻结binary，自始至终不切换默认Home。

A接implement任务书，独立读取源，写有用部分到声明change.md，然后正常结束turn，不submit、不创建后台进程；报告自己启动命令均退出、不再写。Root观察agent completed并核其启动进程/命令原文，不把它称宿主/全进程树关闭。B新fork-none上下文，只拿binary、Home/workid、规程路径；自己CLIstatus查resume，读brief/冻结inputs/草稿，继续同running Attempt。不新start/begin/fail/replace；Root核before/after原AttemptId/revision、输入bytes、read-only五表和业务原件保持。B不得得到A会话正文或已有结论。

A+B完成内容后Root submit并从next进review；Reviewer未编写change，按task必需清单独立查源。无法人类arm盲审时说明是独立内容审查，不叫原真人盲审。需修内容时正常submit审查并back最多一次；execute失败才fail。通过后deliver仅整理所绑定三文档的路径/来源/用法与限制。Root只协调选边，不代writer写产出。

结果oracle：running时result nonfinal/artifacts空；终点后final/succeeded恰好change、review、delivery引用，与实际文件bytes/size/sha/source从该具体终点匹配；查询不改5表/revision/业务对象，SQLite控制载体例外单列。Root实际读取交接包并依其首动作准备C005，记录真实使用；代理接受由用户自动授权，原真人接受没有发生。

预算：准备30分钟、实际run60分钟（A/B/review/deliver/记录合计墙钟），最多一次自然返工；方法本身硬上限仍遵守。墙钟不同于人工活动/费用，不把等待消去成本；usage、付费、未知人体活动null。时间到限、源漂移、writer停写不明或质量降时停止该run，保存已发生结果，不造成功。

只当前自动agent/macARM可构造使用；真人first/reuse/宿主重开/人类paired净收益以及旧LEAK原因仍not_run/unknown，不改写过去。

## 明确时点与文档任务

prepare从freeze.adoption_timestamp_utc开始，硬deadline为start+30min，已发生准备/review/等待均计入，不重置时钟。run_start_utc在第一条add或任何Work CLI调用前实际记录，run_deadline=start+60min，A/B/review/deliver/记录与下一消费者只读使用均包括；到限即停止/留原，不追写超限为通过。

当前只生成文档交接包，不对实际仓库codefix作宣称。code-change图/说明版本保持，task授权限定为文档工作；不写C005、源码或管理元数据。当前下一首动作必须是只读的原件阅读/范围判断；C005激活在C004最终归档以后另任务执行。完整C005 package文件已逐SHA冻结，需新增来源或任一冻结输入漂移就停止并独立重审后再用。
