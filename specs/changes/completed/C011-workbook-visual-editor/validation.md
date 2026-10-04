# C011验证

Candidate: `011ec67375aed79e6bf17aab60c9607471982d57`

当前候选：`011ec67375aed79e6bf17aab60c9607471982d57`；原候选与历史表保留。最新结果见末尾「紧凑输入输出属性」。

| 开工时检查项（历史） | 执行方式 | 输入闭包 | 当时命令 | 当时结果 | 原件 |
| --- | --- | --- | --- | --- | --- |
| 作者工具范围/原输入保护 | not_run | 待冻结 | 未执行 | not_run | [开工状态](evidence/opening-state.json) |
| 真实CLI校验/完整副本/拒绝 | not_run | 待工具及oracle | 未执行 | not_run | 无 |
| 节点/表单/文件实际浏览器 | not_run | 待实现 | 未执行 | not_run | 无 |
| 真人新建/修改/接受/宿主重开 | not_run | 待实际当前Work | 未执行 | not_run | 无 |

当前已经确认用户没有真实Sheltie方法使用；本轮只验收可用性，不设省时收益阈值。个人120分钟/全任务60小时是本任务上限，不给每次重试加一份预算。原C007公平六run/原code、人类净收益与历史费用未知保持原状态。

## 首轮实现与自然返工

真实Work为 `2026-10-04-001-workbook-visual-editor`，初始 `implement#1.0`。工作agent在独立快照上交候选index tree `4250480fddcc7fdaff9d65b79e14e84586a1800a`，23/23测试与入口烟测原件已独立读取并核输入；其中一项布局断言没有实际调用布局操作，覆盖声明未通过。两次早期CLI响应/只读清理失败、网络拒绝、raw日志空白检查失败及首次完整patch超限全部保留，没有倒填成功。

首轮完整patch原为22661296字节，超过方法8MiB上限。仅在本package命令原件目录把JSON/stdout/stderr标作Git binary表示后，原文件字节/路径不变，完整patch7721970字节、SHA `52c58c7c6c922be3a1b6ce0ed25b82ec40053966cf7794399f870fdf03476037`；独立apply/index tree及全部变更字节吻合，源码/package/lock/docs仍文本。这是无损表示，不放宽限额。独审另核99份旧原件/19工具文件及仅属性改变的旧树关系。

Root在Chrome实际新建两节点/显式main边，native键盘修改ID/名称，真实检查通过；删边后的 `FLOW_INVALID` rule3 `nodes[1].id`拒绝也实际观察。GUI保存的ZIP确实落盘并读出同一ID/名称/两节点/main边。Playwright fill、零高度SVG和download事件等待的驱动边界单列，native输入/笔画点击/磁盘文件核对补实际观察，不冒称驱动成功。原件见 [浏览器索引](evidence/browser/initial-observations.json)。这些均由agent执行，真人操作尚未执行。

独立Standards轴1项必改：布局测试只改无关Map，不能授能力通过。Spec轴2项必改：已展开的当前TOML预览不随普通属性编辑同步；结构错误只显示generic与escaped process JSON，不满足易读定位。正式 `review#1.0`正常完成并沿显式back进入 `implement#2.0`，不是执行失败或replace；本次方法使用仍为1。旧已封存报告不改，新报告使用新Attempt路径，见 [当前返工](evidence/back-implementation.json)。完整产品/真人/宿主关闭重开仍未授PASS。

## 修后技术、浏览器与稳定交付

第二实现候选如首屏index tree；5项受影响检查与新28/28、真实入口/CLI/HTTP/完整ZIP烟测在新runtime/dependency/fixture/binary闭包执行，旧23不替newPASS。真实结构错误新增回归先红再绿；原124份旧evidence字节保持。独立增量Standards关闭R1、Spec关闭S1/S2，review#2.0建议交付；代码质量与人工接受分列。

Root Chrome实际native名称/标题/内嵌说明修改后，当前TOML预览同值且保展开/焦点，实际新增未连节点直接显示FLOW_INVALID/rule3/nodes[1].id/原reason/处理提示，技术诊断折叠。实际节点拖动 `(0,0)→(140,165)`、平移/缩放/适应前后两次GUI ZIP的文件集合和每个文件全bytes相同；这关闭实际本场景的布局保真，不从伪Map断言推断。code-change三节点/显式返工边实际加载并check成功。原件见 [修后浏览器](evidence/browser/repaired-observations.json)、[完整布局bytes证明](evidence/browser/10-layout-bytes-proof.json)与样例AX/JPG。

目录代理上传因Chrome扩展file URL权限缺失未执行，原生app窗口绑定调用长时间未返回后选中了无关前台窗口，Root没有在该窗口行动；这些是host/driver边界，不是产品或真人PASS。手动目录选择留待用户正常选择器，不要求更改扩展设置。

完整patch7807709字节、SHA `3717cccd6d8862a6217e67655d61c9e30bdc2c4141b75705345957357f401adc`，独立应用188变更路径及全3469tracked文件bytes/mode同树。后续Root按不改index的git apply整合188文件，逐bytes核与已审副本一致，现有验收工作保留，原件见 [共享工作区整合](evidence/root-integration.json)及 [稳定草稿](evidence/delivery/draft-readback.json)。docs/specs/diff实际通过，Rust/既有fixture/构建输入未改，不重新授Rust test run。

真实Work当前revision10、deliver#1.0 running、无effects pending。交付agent全部命令结束，完整patch与delivery可核草稿已保留，未提交终点是为了观察实际用户操作与同Attempt宿主重开；不是交付失败。真人接受/新建编辑/目录选择/宿主关闭重开保持not_run，不判费用或净收益，操作入口见 [人工验收](human-acceptance.md)。

## 清晰显示与统一资料框

本次仍为同一用户可用性任务，不给每次返工增加预算。006 Work正常review1内容拒绝后back至implement2；当前review2稳定running，工程建议交付，未代用户接受。上方001历史指针不作为当前CLI状态。

| 消费者与边界 | 实际结果 | 原件 |
| --- | --- | --- |
| 新几何出口/入口、实际offset、局部绕行与已有消费者 | 35/35通过，原两项RED及后续诊断失败保留 | [最终检查](evidence/clear-graph-view/implementation-02/checks-obstacles-final.json) |
| 原完整工具组 | host60为59通过/1旧held-stream时窗失败；同源隔离1通过，分别资格，非全65单跑绿；失败因果未知 | [原完整检查](evidence/clear-graph-view/implementation/checks-all-host-02.json)、[限定](evidence/clear-graph-view/implementation/material-held-stream-limitation.json) |
| 最终Spec25路径/25标签/25列表真实鼠标 | 75/75选择、类型、两端与同d高亮通过 | [75次实际结果](evidence/clear-graph-view/spec-75-candidate2-final.json) |
| Code原生拖动S1、zoom/pan、实际native scroll x160/y668、fit/arrange | 12次选择通过，滚动重置0；最初PageDown/arrows为0未当PASS | [实际手势](evidence/clear-graph-view/gesture-final.json)、[S1修后位置](evidence/clear-graph-view/s1-real-drag-final.json) |
| 布局/模式/选择后的完整ZIP | Spec27与Code6逐路径原bytes同 | [Spec27](evidence/clear-graph-view/spec-view-only-candidate2-final.json)、[Code6](evidence/clear-graph-view/code-view-gestures-final.json) |
| 主流程、节点邻接与隐藏边列表定位 | 8/25、10/25、隐藏边实际对应；positions保持 | [子集与定位](evidence/clear-graph-view/main-focus-reveal-candidate2-final.json) |
| 统一多选、自定义内容、中文名及固定引用编辑 | UI/真实CLI通过；COW完整ZIP9→10，仅Flow与新ref变化，所有原ref及其它声明保留；7file闭包同 | [浏览器](evidence/clear-graph-view/unified-input-browser-final.json)、[字节](evidence/clear-graph-view/unified-input-final-cow-byte-proof.json)、[闭包](evidence/clear-graph-view/input-ui-source-closure.json) |
| 双轴复审 | Standards与Spec通过，S1关闭，不代真人 | [规范](evidence/clear-graph-view/standards-review-02.md)、[需求](evidence/clear-graph-view/spec-review-02.md) |
| 完整patch与Root整合 | 280485字节/SHA192b6bd…；独立tree同、全部3646 tracked bytes/mode同；Root73变更路径同且HEAD/refs/index保持 | [独立应用](evidence/clear-graph-view/independent-apply-final.json)、[Root整合](evidence/clear-graph-view/root-integration-final.json) |
| 本版目录浏览器自动重开 | 扩展file URL权限阻断，not_run；用户正常目录选择仍待实际复测 | [边界](evidence/clear-graph-view/browser-directory-reopen-block.json) |
| 真人整体接受、实际分钟/费用、真正宿主关闭重开 | 尚未确认/not_run/unknown，不判ROI | [人工操作](human-acceptance.md)、[稳定恢复卡](resume-card.md) |

所有代码/报告写者结束；仅Root归属的4311页面与临时校验服务运行，旧4311/4312实例已按精确归属停并观察exit0。旧用户试点27文件备份保留。Root备份指南移位后曾出现相对链接断链，原字节改以.txt归档后docs复核；浏览器读取scope不支持parseFloat、列表折叠与已选标签覆盖短线的初次oracle准备问题单列，未当产品RED。此前S1真实缺陷、标签hit与整段重合原件保持。

## 紧凑输入输出属性

007首轮候选85a在实际改名后直接点候选时吞首click，正式双轴review1需修改已封存并正常back。新011ec候选复用候选DOM并按实时资料投影更新状态，Root同一步骤首次click即产生request chip且名字保持，Spec/Standards复审关闭S1；新review2稳定running等真实用户判断，不为重开或复测另建Attempt。

- [工程原件](evidence/compact-node-properties/implementation/checks-staged-final.json)：48相关source/model/CLI消费者通过；最后只变app，48确切依赖保持，syntax/diff/静态HTTP与[真实S1复测](evidence/compact-node-properties/direct-candidate-click-final.json)另列资格，不称旧全65单跑绿。
- [最终浏览器](evidence/compact-node-properties/browser-final.json)：12输入/2输出页签、13已有候选真实勾选含别名、搜索1项、默认属性高度1141（旧4188）；[窄窗](evidence/compact-node-properties/narrow-final.json)1000/600窗口页签/编辑/搜索/属性重开可达。
- [完整视图副本](evidence/compact-node-properties/view-only-candidate2.json)：27路径与全部文件原bytes同；[单项编辑](evidence/compact-node-properties/edit-byte-proof-final.json)仅指定input name和output path变更，required=false与其它声明/26文件保持，真实CLI结构通过。
- [自然行切换](evidence/compact-node-properties/direct-row-click-final.json)、[暂选清理](evidence/compact-node-properties/pending-prune-final.json)有实际DOM正例；前版别名逐一/末次删除与暂选跨tabs的已验证投影、事务函数在新app保持，纯数据资格与最终DOM复测分开，不无条件继承整个旧候选。
- [正式规范复审](evidence/compact-node-properties/standards-review-02.md)与[需求复审](evidence/compact-node-properties/spec-review-02.md)通过；先前准备审查者只做独立apply辅助，不担任正式Standards轴，身份披露保留。
- [完整patch独立应用](evidence/compact-node-properties/independent-apply-02.json)：189457字节/SHA6ee6412735a35515fc33439341827dcd1a318e920259c55ee462f43b54ff1aa6，精确重建011ec tree并核全部3691tracked字节/mode；[Root整合](evidence/compact-node-properties/root-integration-final.json)49路径同，HEAD/refs/index不变。

当前4311由Root PID47242/session19626提供；旧PID19053按精确归属停止并观察exit0，试点完整ZIP保持。本版整体真人接受、实际分钟/费用、目录手动重开与真正宿主关闭重开仍未确认/not_run/unknown。不重无关Rust/几何75，不关闭旧时间窗/平台/ROI边界。目录自动选择的扩展权限限制沿用，人用正常系统选择器完成。

## 2026-10-05 最终可用性接受与提交准备

用户已明确回复「已复测，接受这版界面」，见[evidence/completion-queue-20261005/human-acceptance.json](evidence/completion-queue-20261005/human-acceptance.json)。Root补齐相同lock和逐字节相同的两项依赖后，本机完整工具测试72/72通过，0失败、0跳过；新原件见[全量输出](evidence/completion-queue-20261005/npm-test-host.stdout.txt)与[检查结果](evidence/completion-queue-20261005/final-check-results.json)。依赖未安装的首次运行、离线缓存缺失、网络解析失败及沙箱listen EPERM分别保留，不当产品通过；旧006 held-stream失败因果仍unknown，新全量成功不抹去旧记录。Rust输入未改，未重跑无关测试。

独立review#2.0已按公开CLI封存，当前进入deliver#1.0。用户已表示准备真正关闭并重开宿主；该项尚未实际发生，继续记not_run。当前任务完成后提交C011代码已获用户授权，随后翻译与dsh依次执行；个人分钟、费用和其他平台结论保持unknown/not_run。任务提交拆分与旧工作保护见[提交准备](evidence/completion-queue-20261005/task-commit-plan.json)。

## 2026-10-05 真正宿主重开与最终成果

用户明确回复「已关闭并重开」。新会话公开 `work status` 返回相同 deliver#1.0/revision10；四份冻结输入、两份草稿及可信 binary SHA 全部一致。先记录并公布 request-id `bbdae341-0055-4a73-8fad-dbbb9b67cfd1`，再正常 submit；公开 `work result` 为 succeeded/revision11/final=true/effects_pending=false，仅含 change/checks/review/delivery/patch 五份成果。逐文件 bytes/SHA 通过。原件见 [readback](evidence/host-reopen-20261005/readback.json)、[submit](evidence/host-reopen-20261005/submit-response.json)、[result](evidence/host-reopen-20261005/result.json)、[result-readback](evidence/host-reopen-20261005/result-readback.json)。

29 个工具 tracked 文件仍与已独审 tree011ec673 完全相同，复用本机 72/72 全量测试资格；未修改 Rust 输入，未重跑无关 Rust 门禁。个人分钟/费用/ROI 未知，历史失败原因和平台限制保持。归档后文档、规格、任务范围门禁原件记于同目录。

## 最终采用范围验证

| Requirement / risk | Mode | Input closure | Command / raw run ID | Result | Evidence |
| --- | --- | --- | --- | --- | --- |
| 源码、真实 CLI/ZIP 与工程消费者 | reused | tree011ec673；Root 工具源与已审字节相同 | 本机 npm test，72/72，0 skip；此前实际 CLI/ZIP 证据保持 | PASS | [源码核验](evidence/host-reopen-20261005/tool-source-readback.json)、[全量原件](evidence/completion-queue-20261005/npm-test-host.stdout.txt) |
| 真人最终界面接受 | executed | 最终紧凑属性候选 | 用户「已复测，接受这版界面」；只授可用性 | PASS | [用户报告](evidence/completion-queue-20261005/human-acceptance.json) |
| 真正宿主关闭重开与同 Attempt 接续 | executed | deliver#1.0/revision10；四冻结输入与两草稿 | 用户「已关闭并重开」；公开 work status | PASS | [读回](evidence/host-reopen-20261005/readback.json) |
| 明确成果封存与公开消费者 | executed | deliver#1.0/revision11；五份成果 | 正常 submit 后 work result；逐 bytes/SHA | PASS | [结果](evidence/host-reopen-20261005/result-readback.json) |

归档治理和逐任务范围检查由收尾原件单列，不改变旧失败原件。费用/个人分钟/ROI 未知；其他平台不授通过。
