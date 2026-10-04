# 已完成的恢复记录

2026-10-05 用户明确确认真正关闭并重开。新会话已核验同一 deliver#1.0 的全部输入与草稿，正常提交后 Work007 succeeded/revision11，五份公开最终成果 bytes/SHA 一致。以下内容是提交前历史恢复卡，不再据其 running 状态操作。当前收尾见[closeout.md](evidence/host-reopen-20261005/closeout.md)。

# 当前稳定恢复卡

本卡不证明已经真正关闭并重开宿主；用户实际操作后才记录该观察。

2026-10-05新增用户授权：当前任务完成后提交代码，随后按顺序继续英文Markdown翻译和dsh Mac客户端。C011逐任务提交只收本任务范围，保留旧C007/C008/C010未提交工作；具体授权与条件见evidence/completion-queue-20261005/user-authorization.json。不要再次询问是否允许提交，但未实际完成的人工验收不因此改为通过。

## 新会话入口

仓库为`/Users/shushu/orca/workspaces/sheltie/codex`。读AGENTS、CONTEXT、specs入口与active C011 plan，然后查询：

```bash
/private/tmp/sheltie-human-acceptance-20261004.xw8h_gyu/bin/sheltie \
  --home /private/tmp/sheltie-human-acceptance-20261004.xw8h_gyu/home \
  --json work status 2026-10-04-007-workbook-compact-properties
```

记录时deliver#1.0 running、revision10；review#2.0已封存。代码与报告写者已全部结束，两个草稿已核bytes/SHA，当前可以真正关闭宿主；以新status/resume/next为准，不按旧001–006卡推进，不为重开begin/fail/replace。binary SHA04dba0274004bd46b35e68a2e5fc8f26387cf009b07c33f2e38e3caf75fb45d8，schema4/0.3.0-rc.1，同一Home不回落默认。

## 候选和材料

候选tree为011ec67375aed79e6bf17aab60c9607471982d57，独立源码/private/tmp/sheltie-c011-compact-properties-20261004，baseline602d8ea1dbf136a5e962bfaef48bf7493646eb2c/tree6e6c25。完整累计patch /private/tmp/sheltie-c011-compact-frozen-02-_ko1erjt/change.patch，189457字节，SHA6ee6412735a35515fc33439341827dcd1a318e920259c55ee462f43b54ff1aa6。

review2已封存4523字节，SHA2efcdf8c0d7123297a66e83f9525844a9cde994ff2b5dcc402c3104f46abef09。用户2026-10-05已实际复测并回复「已复测，接受这版界面」，原件见evidence/completion-queue-20261005/human-acceptance.json。当前稳定deliver暂不提交终点，供真正宿主重开后读取同一Attempt；新草稿逐字节核验见evidence/completion-queue-20261005/delivery-draft-readback.json。不要把草稿或最近文件当final成果。旧85a首候选/S1拒绝及全部旧Work封件保持。

Root无index整合49路径逐bytes/mode同；独立新副本重建精确tree，全3691tracked文件字节/mode同，index/refs不变。证据在[evidence/compact-node-properties](evidence/compact-node-properties/)；旧C007/C008/C010验收和本轮上游文档保持，不清理或回退。

## 验证与边界

Root2026-10-05已补齐same-lock/same-byte两项依赖并在本机运行完整工具测试72/72通过、0失败、0跳过；原件evidence/completion-queue-20261005/npm-test-host.stdout.txt。此前本轮48相关source/model/真实CLI消费者通过；最后只变app DOM复用，48依赖闭包原样，syntax/diff/static1及Root真实同步骤native S1红→绿单列资格。新72全量运行有单独资格；旧006 host60的59通过/1旧held-stream时间窗失败及隔离1成功仍留原因未知限定，不以本轮绿覆盖旧因果。Rust/图/引擎不改未重跑。

输入输出页签、紧凑分组/搜索/单项编辑、已有from及别名勾选、名称/输出路径保存CLI、27文件view-only原字节及实际编辑仅两字段改变、1000/600窗口可达/属性关闭重开已实际核。自然名称修改后一次点另一项或一次点checkbox均通过；暂选A转已用只清A、保B，取消B保A只读。模型数据事务/COW/图均保持；布局不重排声明。

真实人整体可用性已接受；个人分钟/费用未知，真正宿主关闭重开仍待实际操作。浏览器目录自动选择受Chrome扩展file URL权限阻断，正常系统选择器需人重开，不自动改权限。新的人工步骤见[human-acceptance.md](human-acceptance.md)。

## 当前页面

唯一URL [127.0.0.1:4311](http://127.0.0.1:4311/)；Chrome browser1/tab682934293，服务PID47242/session19626，归属evidence/compact-node-properties/server-owner.json。旧PID19053/session47700已只按核定归属停止并观察exit0，试点27文件ZIP备份trial-before-update.zip保留。若宿主关闭导致服务停止，先核端口归属，再按工具README使用Root已整合源码和显式可信binary重启，不恢复旧版或杀未知进程。

## 后续原目标

本轮只判断可用性，不判断ROI。C011仍active，用户已接受并表示准备重开；真正宿主接续后收尾、提交三个任务commit（方案见evidence/completion-queue-20261005/task-commit-plan.json），然后归档；翻译/Users/shushu/.agents英文Markdown为同级.zh-CN.*及dsh简易Mac客户端仍后继未实施，一次只一个active change。原预算不按返工重复增加，实际分钟/费用未知。Task2候选73/已有译文5；Task3source0.1.0-rc.5与已装runtime0.1.0-rc.7不混认。旧公平/取证/费用/平台限制不由本轮关闭。未push/merge/release或替换安装。

## 新会话必须完成的收尾顺序

1. 请用户如实报告这次是否真正关闭并重新打开了Codex宿主；新会话本身不自动证明关闭动作。若用户确认，只读查询当前status/resume，核同一deliver#1.0、所有冻结输入和两个草稿的bytes/SHA，记录实际接续；不调用replace/fail。
2. 从当前next按用户已接受结论提交deliver，随后公开CLI读取work result并逐文件核明确五份成果。不动旧001–006草稿、封件或状态。提交前先保存并告知request-id。
3. T03记录done，completed归档C011并修当前路由与所有因移动产生的相对链接；运行docs/specs和适用任务门禁。提交用户已授权的C011三任务范围。T02初版tree4bc46的23文件逐byte和T03最终diff已准备，不改当前源码来制造历史中间态；按合法Git索引分组，核每次git show范围。specs/README.md此前C008剩余验收段落不要混入；C007/C008/C010原未提交状态保留。
4. 随后启动英文Markdown翻译，再dsh客户端。两项预算文件已核：各120分钟人工/60小时总等待，费用unknown；用户选择核对并补齐已有译文，不新增ROI配对要求。正式采纳和完整源快照后分别执行，遵守同一方法与单active。

Root当前HEAD100a6f845e619f95d2f112d2adc5637fdf00801f、index空，无Git提交。Node_modules仅工具内ignore，不纳入。原网络npm ci会话66388已结束exit1/ENOTFOUND；测试会话11267沙箱失败、8837本机测试exit0均结束。4311页面服务可继续运行，归属已核，宿主重开不杀未知进程。

当前delivery草稿9548字节/SHA52a13edfc0e4abd9f7cf47e2daad41147448d47b21de51b4f96d5b44d1e4383f；patch189457字节/SHA6ee6412735a35515fc33439341827dcd1a318e920259c55ee462f43b54ff1aa6。稳定事实见[evidence/completion-queue-20261005/stable-handoff.json](evidence/completion-queue-20261005/stable-handoff.json)。
