# C002-T52 源码与 oracle 准备审查

**PASS，限定当前7项oracle与2个观察点的源码准备。** 无必须修正的源码/方法finding。Reviewer `/root/oracle_review` 未参与G05方向准备、实现或测试编写，只读复核；不改源码、oracle、plan，不并发构建。基准 `93766bbc`，当前fsx SHA256 `f45e10c32796297c1b6260d9d58a9b8970936cde637a1e54d09b114ca7bbd9f7`。实际前整理 `t52-positive-baseline.txt` 为7/7 PASS、384项过滤，原文SHA见JSON；不把过滤项计为已执行。

依据工程§3/§5、INV-3及已采用T52 plan。生产差异仅2个既有failpoint rendezvous：verified在SafeFile/源名称核验和目标prestat/check_regular后，regular在两端prestat/check_regular后，均在真实EXCHANGE前。EXCHANGE后身份/类型/nlink复查与fsync次序不变。默认关闭feature无新增FS/SQL/注入环境读取，额外字符串构造不称零成本；Core/public接口不变，无通用observer框架或private origin伪造。

合法交换先记录独立真实A/B的bytes/dev/ino/mode/nlink，再完整比较交换后两端反向对应；late源/目标替换在上述prestat之后才发生，保留旧A/B而独立记录C。普通exchange另建真实hardlink，使nlink2，并核两端/alias原件保留。RecoveryRequired明确承认已经交换，不期待回滚或丢弃未知对象；这不是通过提前拒绝来掩盖postexchange守卫。

root epoch反例真实改名旧根、同词法路径建立新根/新锁/新ManagedFs，证实同dev不同root ino，再把实际旧A移入新根保持bytes/dev/ino/mode/nlink。旧SafeFile的真实origin不改；调用使用新锁，因此check_lock和后层文件身份都能合法通过，拒绝针对旧epoch。API正例证明同root held对象允许chmod及按原件删除；删除后held FD nlink0仍可读原bytes。此为有界API事实，不扩为CLI清理全caller。

set_mode替换反例准确保留A held/B current：方法先chmod真实A，后名称检查拒绝；B的完整snapshot不变，A仅允许444/755变化。不能将此拒绝叙述为全部零副作用。rename正例真实常规文件/目录，负例真实hardlink/symlink/FIFO；FIFO仅mkfifo/元数据，不读取或靠超时。外sentinel bytes/dev/ino/mode不变，硬链接创建后的nlink2按fixture事实列明。目录正例证明顶层身份与child字节，未声称递归全树每个inode不变。

两个异步exchange反例复用已审scope/release helper，真实reached后才替换，先release/join再断言；worker提前结束返回实际结果，不能用未到点超时代替目标失败。marker-error分支同步调用：真实carrier普通文件导致观察IO错误、无等待worker，先disarm再断言两端精确未交换及carrierbytes。普通panic路径由scope/Release/OwnedFD和TempDir回收，永久OS恢复错误不在此证明内。

16项精确映射/最终mutant日志、完整工程/MSRV/default/治理与最终提交闭包在本报告均为 **not_run/未审**。后续须逐目标失败排除未达观察点/早期守卫/编译失败捕获，不能从7个合法基线提前认全G05或C002-M2完成。

已增量核post-simplify：仅新test module把5元组换具名FileSnapshot，metadata/read、各预期字段和断言保留；模块前（含2观察点）的全部源码逐字一致。报告SHA绑定当前post-simplify源；此前7/7与post-simplify基线分开，后者正在执行，不把前者移植为后者PASS。
