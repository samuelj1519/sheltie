# 真人试用准备与交接

本次只准备最新源码的可信二进制、独立管理根、固定方法和待填记录。原三项代码任务/六次正式运行尚未准入，没有正式run ID，没有创建试验Work，也没有新增参与者使用次数。原六次文档agent观察、失败、偏差和未知成本均不变。

## 当前可用材料

- [准入待填输入](admission-inputs.json)：三项未解决任务、参与者/history、模型/host、预算/阈值、顺序和质量标准。`null`是缺项，不是0。
- [活动记录模板](activity-template.csv)：按实际人和助手记录不重叠区间，保留失败/停止/核对/接受/盲审的投入。未填模板不是执行记录。
- [关闭重开卡模板](reopen-card-template.md)：到达正式协议指定阶段才填写；没有真实关闭重开不生成成功叙述。
- [文档接受模板](guide-acceptance-template.md)：现有三份指南的真人接受可以先做，不能追溯补六次代码试验。
- [完整剩余工作](../../../C008-dependency-readiness/remaining-acceptance.md)：责任、关闭条件和条件任务入口。
- [原正式手册](../../experiments/first-use.md)、[原协议](../../experiments/protocol.md)：保留原要求，不能由本次技术准备推断原M1准入或M2真实价值通过。

技术准备的实际binary路径/SHA、Home、方法摘要和CLI原件记录在 [prepared-environment.json](prepared-environment.json) 和 [prepare-cli.json](prepare-cli.json)；从Cargo JSON读取产物后复制到本次自有新目录，避免后续构建替换用户将使用的字节。所有操作都显式使用该Home；不替换用户安装、不改宿主配置或默认管理根。临时路径丢失或字节不符时停止，重建后重新核验，不从PATH取另一个版本。

本次自有目录是 `/private/tmp/sheltie-human-acceptance-20261004.xw8h_gyu`，Home在其 `home/`，固定二进制在 `bin/`。7条实际CLI均退出0：版本、自描述、方法add/show/verify、空work list及exporter help；schema4/0.3.0-rc.1、摘要 `0d66bee3391522be7858130951dd979e2a7ee69b4a435e8968ac676153b21d86`、task/project起始键、verify=ok均读回。方法原字节未变，Work列表为空。方法已登记一次，正式准入后只核show/verify，不重复add；读命令不传request-id。没有把工程准备算成真人首次使用。

## 人与agent的下一步

1. 操作者先按文档接受模板回复实际判断，并提供准入输入。正式比较缺任何必要条件就不开始。
2. agent核目标确实未解决，准备两组独立初始副本、完整同文字方法、各任务正反标准/必需检查、实际权限和原生持久记录；固定预算/阈值与配对顺序后独立审准入。收到信息后在本package恢复对应计划，不静默改变原门槛。
3. 正式运行中agent负责派活/检查/原件；操作者记录自己的实际活动、费用来源和接受判断。首次读者必须未写方法，所有助手和跨组熟悉披露。
4. 到达冻结的同一阶段后，agent保存本组恢复卡并确认执行者停止。操作者实际关闭旧会话、新开会话并记录身份/时间，再交恢复卡。新会话按当前status/resume和候选接续；普通接续不调用replace。
5. 最终candidate/完整patch/原检查供独立盲审，真人接受单列。成本按原P/S/M/R/E公式复算；未知usage/费用不当零。没有自然返工则保持该观察未执行。

该正式方法是私有 `code-task-study@1.0.0`，默认无gate；不把spec-dev 0.2.2换入其中改变原配对。C006副本观察另行冻结，不能悄悄增加到原C007比较。真撤销或必需宿主资源摩擦出现时，只记录触发材料并按对应package前提办理。
