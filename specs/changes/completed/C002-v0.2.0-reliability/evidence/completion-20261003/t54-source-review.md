# C002-T54 源码/oracle 准备审查

**PASS，限定当前5项新增oracle、1项未改复用oracle与1个观察点的准备。** 无剩余必须源码/方法修正项。Reviewer `/root/oracle_review` 未参与G07方向、源码或测试编写，只读。当前SHA见JSON，真实wire-corrected baseline6/6 PASS、397过滤。revision编译101、错误SQL列和original/内部Response形状fixture失败均保留，不当产品红；wire original现在按公共合同完全手写，不调用生产投影helper或直接拿reply_json当答案。

seal观察点在真实fstat/check_regular之后、旧before身份/长度比较之前，随后才hash read；默认feature关闭无新增FS/SQL/注入环境读取，Core/public接口不变。现有SafeFile元数据并不能支配之后live before长度，窗口准确。

同步来源正例真实同root/原handle。负例真实改名旧根、同词法路径新建新根/锁/ManagedFs，再移入实际A保持dev/ino/mode/nlink/bytes，旧SafeFile origin不变，新API用新锁。后层路径匹配能合法通过，拒绝针对旧epoch；不是伪造private origin或借旧锁绕guard。限于crate API，不称真实CLI所有epoch场景。

输出cap使用真实Repo.add/start/begin且Flow明确max_bytes33554432。恰好cap真实submit完成，Ref bytes/独立zeroSHA、revision3、单请求audit/published1和mode0444均核；多1字节精确OutputTooLarge max/actual，5表完整不变、无请求登记、output原mode/dev/ino/nlink/size保留。不会被较小core声明上限提前挡住。Runtime integration并非实际CLI进程，范围如实列明。

封存反例真实submit COMMIT后才grow4→5；真实before fstat捕获5后reached，再同inode恢复data4，释放/join完成后校EffectPending committed/request/causeStoreCorrupt、手写完整original、5表与published0、原字节/身份/未封mode。旧before坏长度仍必须拒，后续正确SHA/长度不能擦除已捕获事实；5表比较基准是COMMIT后的真实行，不能叙述为未提交。

marker失败分支真实普通carrier导致Io，先disarm再assert原件完整未封，无worker。integration复用已存在RendezvousWorker，两release路径在首次wait前持有；任何普通panic Drop释放两点、join并disarm，显式finish先join再assert。wait缺少early-worker-result分支，但10秒deadline有界且Drop回收；最终mutation判定必须拒绝任何未达点/timeout，不能当caught。

旧C005input swap未改且会在实际filter执行：A捕获后保留，换成同bytes新inode B；因A仍保留inode不能复用，实际identity漂移。它核core替换前拒绝/5表与两份bytes，不声称不同bytes或新worker内容差异，也不复制新镜像测试。完整业务文件tree逐字证明、全部CLI链、6最终处分和工程闭包均未在此批准。

整理增量复核通过：模块前fsx完全不变，既有integration前缀及两条新result tests逐字不变；2个TOML字符串独立解码同bytes，3身份tuple只提取std字段。post-simplified6/6真实基线已核，当前SHA见JSON。
