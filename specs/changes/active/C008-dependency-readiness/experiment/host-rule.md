# 单宿主规则边界（未绑定实际host）

当前host/version、目标、获准root/选择机制与身份解释均null，没有实际观测。以下是原未来探针必须兑现的规则，不是已实现或已测试能力。完整机制合同见[design](../design.md)，实际缺项见[protocol](protocol.md)。

- presence、selection、声明身份字段分别判断。存在同名候选不证明host会选择它；不能解释优先级则selection unknown。
- 只有获准且完整的搜索闭包都无目标才mismatch；部分位置未找到是unknown，不把scope不足当缺失。
- version比较须有可靠来源及预声明语义；digest须覆盖指定完整内容闭包；source须有可靠来源标识，路径/显示名不是身份。读取漂移、未知解释或不完整关联内容unknown，可靠同闭包不符才mismatch。
- 目标项组合按原规则：确定mismatch优先，其余有未确认依据unknown；全部可靠相符才matches。三态结果与证据对应，不把退出0当资源或执行持续就绪。
- source只是数据，不执行skill/MCP/程序或远端地址，不联网/下载/安装，不修改host/Work/Store/任务目录。
- 非法/未知配置字段或无声明目标拒绝，原预定exit2且不输出有效观测；合法unknown输出完整依据与exit0。当前没有配置/脚本，不能以文档代这些测试。

未来只读fixtures与probe调用区间byte/tree快照核可观察范围；保存证据本身由外部记录者做，与probe no-write边界分开。实际host不可人为删除/移动/安装资源制造收益。声明和host变化的观察追加，不覆盖旧记录，不承诺TTL/缓存/运行期持续可用。
