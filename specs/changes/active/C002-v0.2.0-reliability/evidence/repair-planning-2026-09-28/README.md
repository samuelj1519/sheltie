# M1 修复方案与计划的材料验证

运行标识：`repair-planning-2026-09-28`。被审实现基准：`e1a8126a432981df40023628dd06feec7884d458`。本轮只创建修复文档、补主plan/tasks/交接入口并做文档检查；未实施源码、未运行源码测试或T18平台探针。方案入口见 [执行手册](../../repair-plan.md)。

[results.json](results.json) 记录check-docs、check-specs、check-tests机械归属检查、diff及TOML/任务/覆盖静态核对的命令与exit；stdout/stderr分别保存。check-tests只检查已有464测试的命名/归属，未执行测试。输入摘要与模式边界见 [input-closure.json](input-closure.json)。

## 独立审查与处置

Reviewer为未参与文档撰写的`/root/standards_review`与`/root/spec_review`。初稿“需修改”；以下问题保留处置，不用后续材料通过覆盖旧问题。

| 初稿问题 | 修订 |
| --- | --- |
| plan自引用违反现有core规则 | plan-review保存原字节reviewed_plan/tasks，plan引用不同节点，不放宽core规则 |
| 冻结tasks没有后来完成事实 | implement/fix携带累计verify表；verify过独立验证再追加，记录检查/继承源；fresh planner核累计前缀、Git和原始证据，漏行停止 |
| 清owner后final仍要求owner | 区分未完成/pending与已完成对象；历史快照不碰新生命周期 |
| maintenance有data字段与stderr两种解释 | 统一独立stderr诊断，成功JSON/业务快照不变 |
| READ_ONLY不能证明文件系统零写 | SQLite控制文件例外待T18采用；真实bundled/两平台、关闭后main/WAL保留、NO_CKPT_ON_CLOSE，不用immutable/VFS假通过 |
| FIFO先open后stat可能挂住 | 先类型检查、NONBLOCK/NOFOLLOW打开、后fstat核身份；有期限拒绝例 |
| purge等待者把旧Work当初始化成功 | install/add可初始化，旧Workexisting/not_found，不复活 |
| 前置task要求后继完整oracle | 分子义务与最终Owner，保留后继FAIL/not_run，不提前关R |
| 所有COMMIT前失败都要求sequence不变 | preflight不烧号，分配后允许空号且不回收/复用 |
| 正常seal可能用cached meta | V07补COMMIT后同inode bytes/nlink改变的正常链反例，不用重启恢复代替 |

两位Reviewer复核修订后给出“方案材料通过”。最终delta确认累计交接、镜像字段、名字选择器/规范化产物分工、published事实、SQLite安全API与seal职责自洽；Standards建议的distinct正常链oracle已补V07/T22。

Spec原始结论摘要：“方案材料结论仍为通过；仅文档差异，不表示代码、门禁、API探针或M1通过”。Standards原始结论摘要：“静态API和责任链可行，未跑代码；新增正常链bytes/nlink回归应补”。这些结论只评价设计/执行材料，T18–T31仍not_run；采用、API、代码、M1、Host和发布均未由本材料判通过。
