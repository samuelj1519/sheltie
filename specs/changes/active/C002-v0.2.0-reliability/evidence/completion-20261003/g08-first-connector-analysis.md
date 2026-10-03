# G08 三项首 kind/dev 连接符：独立限定分析

结论：3项均保留 **open**。当前实现没有 `tree.dev == root.dev` 约束，也没有拒绝submount的检查；完整caller闭包不能排除不同dev但数字inode相同的目录事实。另一方面，本轮未找到可在授权的native临时根内抵达精确差异窗口的实际fixture，因此不给“已验证反例”、等价或PASS。只读，没有假stat、mount、oracle编写/运行或tracked修改。来源SHA和精确旧/当前ID见JSON。

## 精确差异，不扩成普通 inode 替换

本次仅审归档当前ID：

- fsx.rs :658:13，ManagedFs::verify_tree_at；实际条件当前在 :678。
- fsx.rs :699:13，ManagedFs::rename_tree_new；实际条件当前在 :719。
- fsx.rs :1691:9，verify_tree_entry_at；实际条件当前在 :1743。

令A=当前名称kind非Directory、B=名称dev不同于持有目录FD的dev、C=名称数字inode不同于持有目录FD的inode。原谓词为 `A||B||C`，首连接符mutant为 `(A&&B)||C`。它们只在C=false且A/B恰有一项true时不同。普通同dev另一个目录的C=true，两者都拒绝；这不是这三项的counterexample。

所有实际持有FD的producer确实打开Directory。若采用native文件对象的dev+inode联合身份约定，同dev、同inode且持有FD仍存活的对象不能同时变为不同类型，A=true/B=false/C=false可以按该联合身份前提排除；这不是把数字inode当成全局唯一。剩余需要全caller排除的是A=false/B=true/C=false，即另一个设备上数字inode相同的目录。没有证明这个集合为空。

## producer 与前层到底约束了什么

`ManagedTree`唯一literal构造点是 `open_tree_locked`（fsx :656–669）。它核HomeLock，然后 `open_dir(Some(path))`，记录管理根字符串和root_ident；没有记录或校验 tree.file 的dev等于root_ident.0。

`open_dir`（:337）从root FD逐段调用 `open_directory_at`。后者（:2511）核名称kind、O_DIRECTORY/NOFOLLOW打开并核该段before stat和opened fstat的dev/inode；它证明这一段打开了观察到的对象，却没有比较该段dev与父/root dev。因此路径可合法跨既有设备边界；没有NO_XDEV、submount扫描或root-device断言。`ensure_dir_unlocked`对既有段也是相同before/opened绑定，对新mkdir也会在其实际父设备创建，不能以“引擎自己创建”反推任意既有祖先同root设备。

`check_tree_root`（:916）比较tree记录的管理根与当前ManagedFs根字符串/epoch，不比较子树dev。`HomeLock::matches_root`（home :249）进一步核实际根/锁身份仍有效，但只约束管理根，不禁止根内部不同dev子树。不会把Home规范字符串当epoch，也不会把合法锁当子树设备证明。

其他路径/owner/digest资格是ManagedRelPath、已登记归属和原件内容事实：它们没有dev==root的上游合同。storage §3.2/§3.3要求同一原件，§4/§5拒links/特殊对象，并未把submount声明为不可到达或引擎已经拒绝。此前正常mkdir产生同设备临时对象的个别测试不能覆盖所有恢复、维护和名称外部变化。

## 完整当前 caller 与后层支配

### ManagedFs::verify_tree_at

直接caller为：cleanup_expired_tmp两次（核tmp根）；managed_tree_is_empty_locked一次；rename_tree_new的rename后一次；make_managed_tree_writable的chmod前后两次；verify_managed_tree_at wrapper一次。wrapper在effects中由publish_dir七次、verify_publish_final_state一次、delete_dir六次调用。没有其他Tracked Rust caller或额外ManagedTree constructor。

每次目标stat来自当前父FD下的名称；先前verify及后续verify都重新读外部名称，不是同一局部stat不可变值支配。普通pending发布中，后层未突变rename_tree_new确实会再次完整核source，但它在目标父目录ensure_dirs之后：第一次应拒绝与较晚拒绝不是同停止点/诊断证明。make_writable有真实递归chmod夹在两次检查之间。final-only恢复分支也会在后续重新装入内容、同步和readonly后再核，不能静态视为“后层总会同样拒绝”而消除第一次绑定义务。

若未来原件fixture能在 `publish_after_tree_sync` 后给出B-only事实，当前下一操作正是verify wrapper；未mutant应以STORE_CORRUPT停止，mutant可能到达ensure最终父目录及rename阶段、或更晚其他诊断。这是条件性的源码窗口，不是本轮已经建立的native反例。

### ManagedFs::rename_tree_new

只有rename_managed_tree_new wrapper；两个实际effect caller为pending发布和remove final移入pending。它们此前有verify，但source stat属于本函数中新查询，不能用先前正确tuple冻结名字。guard之后立刻NOREPLACE rename，随后verify_tree_at目标；后层错误被包装RecoveryRequired，因为rename可能已经发生。原实现是移动前STORE_CORRUPT，两者拒绝时间、错误类别、原件位置没有全等证明。

原 `publish_after_tree_sync` 位于前一轮verify之前，会被那一层正常拒绝掩盖；若需要分辨本guard必须在最后前层verify完成后、本source stat/rename阶段给出真实变化。假设叶目录是另一设备的既有mount，OS rename也可能EBUSY/EXDEV，不得事先宣称一定成功或一定移动竞争者；若实际出该错误，须独立核准确cause及端点。此轮没构造这种设备/数字inode事实，不提出mount实施。

### verify_tree_entry_at（有额外PrepareAttempt消费者）

完整直接caller有三个方法，各调用两次：

1. remove_managed_tree：删除持有FD下内容前、根unlink前；producer是open_tree_locked。实际调用是effects delete、合法孤儿cleanup、过期tmp维护。
2. remove_empty_managed_tree：枚举空性前、unlink前；producer同上。实际消费者是已完成pending空容器metadata，以及WriteSession新Store发布后的空tmp/staging清理。
3. **sync_directory_entry_locked**：sync目录后、sync父目录后；持有FD直接来自open_directory_at，不是ManagedTree。真实入口为effects PrepareAttempt→sync_managed_directory_entry。此前G08概要以目录删除为主，不能遗漏这个完整caller。

这些方法都没有子树dev==root保护。前后helper调用各重新stat/fstat，且同一个mutant作用于两个调用。remove方法中间会删持有原树内容；empty方法中间读目录并可能unlink；sync方法中间会父目录fsync并触发准确sync故障边界。后层没有同一个已捕获entry值可作为支配证明。

`delete_before_root_unlink`是实际删除末段窗口，但普通同dev不同inode仍C=true，两者都拒绝。若未来存在B-only真正设备fixture，原末段应RecoveryRequired且保留变化后的名称；mutant可能尝试unlink（OS具体结果另核）。若B-only在首检查前，原实现尚未删持有A内容，mutant可能先删除A再在OS unlink失败，终局is_err不能恢复首层原件保全。sync-directory的前一次检查则能与后面的父sync诊断或额外I/O相区别。均为条件源码差异，不等于已证明native可达。

## 本轮只读 native 事实与边界

只读读取5个已经存在目录的metadata，没有创建fixture或调用被测API：`/`、`/System/Volumes/Data`、`/private/tmp`、本workspace均dev=16777234但inode不同；`/dev`为dev=1642716082、inode=333。精确值写入JSON。这个小样本没有不同dev且相同inode的目录对，也没有Home内对应的合法producer→变化→consumer窗口。它不能证明整个macOS inode全局唯一；根/tmp/workspace同dev也不能证明所有支持的Home/恢复事实均同dev。

从native闭包更改名称而不改变设备的现有observer，只能得到C=true反例，不能捕获这三项。Root改名/重建同dev导致root epoch失配的反例属于别的guard，会在check_tree_root/check_lock前层停止；不能拿它处分首connector。没有本轮允许的真实设备事实时，不假stat、不挂载、不引入设备模拟/新平台，不为把3项做完而改上游产品域或生产代码。

因此每ID的正式限定结论都是open：完整producer证明“持有FD是Directory”成立；“子树设备=根设备”证明不成立；“后层同不可变值支配”不成立；真正native反例尚未建立。若根agent选择当前具体临时fixture的更窄同设备范围，只能记录该fixture闭包内观察不到差异，不能升级为整个当前caller等价或旧SK02已完成。
