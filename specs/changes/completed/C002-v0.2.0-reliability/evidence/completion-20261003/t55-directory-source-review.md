# C002-T55 directory 源码准备复核

**PASS，限定纯目录module/3接线/8个FS oracle；不批准T55整体或5结构处分最终关闭。** 新helper无I/O/state/public/trait，3调用点actual各取同一named stat、expected取同一current held fstat，未误用root_ident。独立把3新条件回替旧条件、删除新helper/observer/module后，整before-file bytes一致，故锁/原statat→fstat/error/副作用顺序保留。目标libc dev_t i32→u64对应转换保等、ino_t u64未缩窄；De Morgan与原拒绝式点值一致。当前SHA与8/8真实baseline见JSON；首次缺PermissionsExt编译失败保留，不算产品红。

8纯关系由literal expected手写，明确Dir+sameino/diffdev=false；不是fake fstat。native verify/epoch转植/健康rename与最后sourceStat前B/删除末root竞争者均真实发生。全树snapshot含完整keys/dev/ino/mode/nlink及file/linkbytes，不跟随leafsymlink。purge whole-root preflight先捕获readonly-root及early目录，after在RAII还原之前捕获，任何早chmod/unlink可见；hardlink外原件不变。合法symlinkpurge只unlink自身。delete-last窗口承认A child已合法删完，不称全删除前零写；lateB空root必须保留。recursive directory/file替换有真实stat/open窗口，当前B和retainedA保全。scope Release/join和PermissionRestore覆盖普通panic/回收，永久OS恢复错误不在此证明中。

原215全量行目标扫描：**仅5connector语句移除，另88个fsx起始目标行仍存在**；3函数中的verify_tree_at旧568 Fn→Ok仍live，要生成新current target执行，不是结构删除。6个源码语法连接符≠6个215欠账。旧1595原formal stage1实际Missed/317绿，随后旧final-ledger独立focusedCLI记Caught/已完成/no missing；在215之外，不造第34项，也不改正式Missed。原raw archive/log SHA与后续账本记录存t55-old-1595-original-stage.json。其余模块若后续变更须另扫，行存在亦不等于current inventory/diff可盲复用。

新tree_rename_before_source_stat确在root/锁/parents打开后、source stat前；default关闭无新增FS/SQL/注入env读取。8项尚未实际验证其marker Io错误分支，终审须有准确移动前原件保全证明。当前remove_empty_managed_tree/sync_directory_entry等完整真实caller、共享新机制日志、effects/locator与全工程尚未在本准备闭包审定；5结构项仍按待最终证据，不记old Caught/native反例/等价。
