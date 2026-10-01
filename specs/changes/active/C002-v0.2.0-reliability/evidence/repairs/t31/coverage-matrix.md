# T31 保留与补强矩阵

实施基线：T30 `b926789`；最终源码输入`49d3a191`。下表的正常可靠性能力在最终675项运行覆盖；原始结果见[最终门禁](final-acceptance-gates/gate-results.json)。Linux为not_run。完整安全变异门槛按2026-10-01用户要求暂缓，单列处分，不从本表推成安全PASS或M1完成。

| 能力 | 真实入口 | 独立oracle | Owner | 保留或补强 |
| --- | --- | --- | --- | --- |
| 同Attempt竞争提交 | WorkService::submit两个线程 | 两参与者先到barrier；只有一成功，另一精确拒绝；Store终态 | 原T16，T31替换交错方式 | 旧lazy spawn/join改为先collect全部handles，再放行并join |
| add并发暂存与取消/提交 | WorkbookRepo::add；WorkService::cancel/submit | 同步起跑、各自结果、原字节与Store终态 | 原C002-T08/T07 | 保留API用例，加实际barrier，不删owner/oracle |
| 原子COMMIT/删除/update退出 | Cargo注入sheltie子进程 | exit70、requests与登记效果、原件/prev原字节 | 原T23/C002-T27 | T30已完整迁CLI crash7条；runtime留API2条 |
| 禁止修改宿主配置/OS身份 | Cargo注入sheltie子进程 | 退出码2、宿主rc不变；id -un独立主体 | 原C002-T15/T05 | T30已迁CLI os_process，无测试内Cargo构建 |
| Start/Add发布窗口 | CLI start/add，同请求及另一类写入口 | 每个精确窗口的Store提交/published、原件字节及原snapshot | T31 | 新增owner sync/私有副本/COMMIT/rename/mark前后exit70+真实kill |
| Attempt目录/历史文件/封存 | CLI begin/submit，恢复写命令 | 登记content逐字节、产物权限/ref、最新卡 | T31 | 新增COMMIT后/首个历史write之后的真实窗口，两种终止模式 |
| remove完成证明 | CLI remove及跨入口恢复 | 同一payload；部分删除保留；无marker准确停止；有效marker只证明本请求 | T31；保留T27 | 补精确移入/部分删除/末删/marker sync/mark前后kill；保留已有异常对象反例 |
| update和purge | 安装后的真实binary/prev；CLI self | prev rollback原字节、Store字节；根/.lock dev/ino、旧Work不复活 | T31；保留T23 | 真实kill与exit70分开；purge等待者先到try-lock失败事件，不用sleep猜窗口 |
| 锁外根/.lock替换后的等待者 | HomeLock真实句柄与身份复核 | 不接受旧对象、重试或明确拒绝，根外哨兵不动 | T19/T24/T31 | 保留已有API身份复核与并发初始化；补purge后合法初始化/旧Work等待链 |
| stats同次事实/只读pending | CLI stats及runtime readonly入口 | 原stats/next同revision、writer确实完成；只读不恢复 | T29/T28 | 保留真实同步测试与反实现失败证据 |
| 重规划交接 | CLI+临时Git与冷读Worker | 冻结镜像、递归累计前缀、审批/原始证据/原基线 | T30 | 保留全部9条、不得以任务卡或缓存替代 |
| 完整突变 | 隔离普通Git clone、当前变异副本binary | 完整inventory分片无漏无重，caught/missed/timeout/unviable分列 | T31、独立M1 | 第一阶段2499齐全；第二阶段245终态；[全部ID处分](mutants/final-dispositions.json)含269用户暂缓，本次不记完整安全PASS |

T30提交后check-task通过，T31最终167项源码sha一致。原始失败与审查保留；迁移未删除历史能力。最终窗口结果来自实际675项运行及各轮原文，不由任务状态代替。T31完成仅在主计划明确的本次豁免范围内成立。

## 实现审查追加闭环

七项逐条根因、真实回归与限制见 [修复答复](../../../review-response-implementation-2026-09-30.md)。补充CLI首次Store初始化SIGKILL、当前/历史批准记录删除；runtime文件/目录及已published历史补缺连续sync失败；纯core非gate Gate理由及合法取消；add结构粗检NoHome与私有副本内容失败零业务行同rid重试。原T24错误的内容拒绝边界保留失败证据后按上游勘误替换，不以删测试让代码通过。
