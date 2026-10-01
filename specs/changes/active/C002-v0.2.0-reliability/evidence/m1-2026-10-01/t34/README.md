# T34 快照业务绑定修复

Owner：Codex。父提交T33 `06c3af3`；独立Reviewer：`/root/m1_spec`、`/root/m1_standards`，均未修改本任务代码。任务状态只看[plan](../../../plan.md)。

## 已确认的实现错误

- R22：合法类型的remove id/version与audit不符时，真实CLI已报完整性错误，却附带错误的成功original；add的id/version/digest也存在同一资格缺口。[原实现反例](r22-probe-red.json)和[独立finding](r22-independent-finding.md)保留首次提交回复、控制例、SQL单字段变化及全Store/业务文件前后oracle。
- R23：Fail历史回复可成功重放Succeeded/Cancelled等不可能状态。
- R24：Begin Reply/data互相一致时，可成功重放冻结节点没有声明的宿主资源。两项由原冻结候选真实CLI确认，[原文](r23-r24-probe-red.json)、[独立finding](r23-r24-independent-finding.md)保留，未用当前状态误验历史。

## 实现与资格边界

Store分别解码快照元数据和效果字段。Work/Workbook用同一装入函数核audit、业务事实和快照，成功后才提供原响应；后续效果错误携带该响应。Add还核唯一PublishDir业务目标、owner、digest；remove依据audit目标。目标检查在完整效果检查中复用，不复制规则。

RecoveryAccess返回已核请求或带资格信息的装入错误；恢复器集中包装本人和旧A阻新B的错误。删除任意raw JSON到成功原响应的回退、未消费的请求查询和冗余判断。装入错误的cause采用Box，修正Clippy实际发现的136-byte大Err，不屏蔽lint。

core生成和验证共享节点资源解析、成功状态规则及失败重试规则。历史状态只核冻结事实可决定的必要条件，不依据当前状态或后来的visits重建过去；失败回复的状态可由原retry/max_retries确定。仅跨模块使用的helper公开，内部失败状态函数保持私有。

快照绑定坏则无original/pending_original；合法快照遇坏effects BLOB、坏后续path、sync或mark错误仍保留原响应。Start/Add自身发布定位不可核时不猜快照、不改用当前已装Workbook。旧效果测试按此资格分组：Add目标锚点坏必须断无原响应；pending等后效果坏仍断原响应业务字段等于首次捕获值，未删除拒绝或零写断言。

## 真实caller验证

新增真实CLI覆盖本人original、旧A的pending_original、Add目标、Fail原retry及后继blocked/cancelled后的合法历史、Begin冻结资源。ManagedDir覆盖每次write/rename拒异根锁或替换后的旧锁、原件bytes/mode不变及恢复合法对照。

[r22-tests-red](r22-tests-red.txt)确认两个新CLI断言在原实现失败；[定向回归](semantic-controls.txt)覆盖崩溃窗口、metadata/效果错误和合法对照。该运行有过滤，不能代替最终全仓零skip门禁。只读fixture连接、编译调用点/借用错误、旧原响应断言边界与初始Clippy/docs失败原文分别保留，不把fixture问题当新的产品finding。独立seam说明仅调整一处仓库禁用措辞，事实与校验义务保留。

## M1输入边界

T33冻结clone `5344b265`的runtime第一阶段1773项完整：1205caught、209compiler unviable、359missed；core720原阶段及core4当前workspace原文保留。T34编辑后源码guard在第一个runtime workspace阶段创建前停止，exit1原文保留；这不是平台安全提示，也不是安全跳过。T34改变core与runtime，最终M1必须重新执行两者完整inventory，不把任何旧运行冒充新输入PASS。Linux仍用户豁免的not_run，T16/T17仍not_run；独立Spec后续复核实际被平台暂停并按授权跳过，原文与缺失义务见[安全跳过清单](../safety-skips.json)。该项不记Spec通过，不重试；其他已返回审查证据和未受影响门禁继续。
