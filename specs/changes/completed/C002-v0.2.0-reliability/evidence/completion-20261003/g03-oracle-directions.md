G03 26 个精确旧→当前ID的只读分组方向。当前仅macOS aarch64；本报告不写测试、不运行突变、不做PASS批注。逐ID/原diff/当前diff SHA、源码SHA见同名JSON。每组先跑实际非零baseline与少量代表，原Missed/timeout/未执行保留；只有真实red或精确限定静态证明才处分。

| 组 | 数量 | 真实consumer与共享独立oracle |
| --- | ---: | --- |
| P1 根/目录 | 5 | Home.resolve→ManagedFs.open/create；真实CLI add/start/install及ManagedFs.ensure_dir/directory_exists。合法目录可用；普通文件、链接、特殊对象占位必须在动作前拒，原因给完整路径，业务row与根外sentinel不变。 |
| P2 锁对象 | 6 | Home.acquire_lock与schema retry的acquire_existing_lock。缺锁返回None且不建；普通单链接成功；硬链接/FIFO/替换inode必须准确拒绝，不能以最终重试16次的IO错误代替早期InvalidRequest。 |
| P3 路径/错误 | 4 | Home.confine、ExternalReadFile/Tree打开与真实managed错误。合法不存在尾部允许词法路径；权限/NotADirectory不能当NotFound；完整绝对path必须保留，不能只is_err。 |
| P4 锁建立重试 | 3 | Home.acquire_lock→is_lock_setup_path_missing。只有root内的结构化NotFound能整体重试；root外或PermissionDenied不能被吞。核实际尝试/停止、原因、root/.lock身份与不建立业务Store。 |
| P5 新文件FD | 2 | write_new_observed/write_new用于pending owner、start-input、Store暂存。原bytes/同创建FD/dev/ino/mode核对；写后增加硬链接或unlink导致nlink≠1必须RecoveryRequired，保留原对象、不越界删除或假成功。 |
| P6 Store控制边界 | 3 | WriteSession.open_or_create/open_existing与Store.connect。RO预检后锁内仍核main及wal/shm/journal类型/链接/归属；无main却有sidecar必须拒且不创建新Store，不能被预检遮住。 |
| P7 schema等待 | 2 | Store.open_for_home的readonly mismatch→existing lock等待→重验；RW不套只读等待。实际并发/旧库拒绝与原bytes不变，不能仅测稳定错误码。 |
| P8 purge最终复扫 | 1 | self uninstall --purge真实临时Home。最后remaining扫描只排除同locked inode；晚到未知文件保留并报告partial，根/.lock同dev/ino，不能把所有stat成功项都忽略。 |

P1/P2需要分清**原stat与后来open不是同一个事实**。`open_directory_at`自己严格核目录，不证明调用者较早stat的类型/身份不变。`ensure_dir_unlocked:407`比较外层stat与FD，内层helper比较另一次stat与FD；两次stat之间真实替换仍有意义。现有directory_open_after_stat/after_open只能说明命中内层窗口，没命中外层窗口不能记caught。`open_existing_lock_file`的两次metadata来自路径与后来FD，不是同FD不变量。当前没有适合精确窗口的握手时，把缺项交原语作者补一个必要failpoint观察点；不用sleep/概率race或mock直接返回成功。

P5的两个||→&&都不能因**同FD dev/ino恒定**静态关闭。原式为 `type_bad || nlink_bad || identity_bad`；创建FD的type_bad/identity_bad通常恒假，但nlink在持有FD期间可变。两处&&都可能按优先级吞掉nlink_bad。需要写完、第二次metadata前的真实对象窗口；在函数已经返回SafeFile之后加hardlink摸不到这个guard。同理P2后三项||→&&可能吞nlink或inode，不能只看被替换字段的名称。

P6用一个真实锁等待fixture避免RO预检提前挡住：actor先持HomeLock，worker正常add/install完成RO无main预检后，确切停在home_lock_waiting；actor在本临时root加入格式合法的孤儿SQLite sidecar，然后放锁。worker锁内必须STORE_CORRUPT且main不存在，原sidecar bytes不变。该点同时区分validate_store_files_locked→Ok和sidecar_exists的|=→&=；后者从false开始永远false，逐sidecar是否合法的验证仍不等于孤儿判据。另核Store.connect的标准错误分类，需要真正到达后一次symlink_metadata，而不是前层managed验证已拒就记该guard被测。所有sqlite前后主库/WAL事实单列，D-039允许控制文件例外不当业务修改。

P3的display_path不是“纯展示可以忽略”：它进入结构化Error.path、cause_detail，也用于准确root内重试判断和现有握手scope。用普通文件占位或真实权限错误核绝对root/相对leaf和准确cause，不让缺path变成毫无定位的错误；不要仅用握手scope消失引起等待超时证明产品行为。Home.confine的parent guard变true会多读base甚至以上祖先；base_canon是已捕获值，base当前对象是活文件系统，静态不应当它们相同。

P7优先复用实际schema等待/初始化消费者。原子Store初始化现在不会把半结构main发布出来，因此不能只拿旧测试名声称命中过mismatch retry。可以用**真实显式purge旧schema临时Home**的合法交错：writer持原HomeLock、在删除数据树后/删除Store前暂停；readonly reader识别旧schema mismatch后必须等待existing lock，writer完成删除保留原root/.lock后reader重验应得NOT_FOUND，不能返回过时mismatch或新建锁/Store。要确认writer暂停时旧main还存在，reader确实到home_lock_waiting，再放行并join；不得在fake中写成功状态。稳定旧schema对照仍准确拒且原bytes保持。

P8在purge最后remaining扫描前后必须有非锁对象：现有purge_before_final_rescan可用于允许的late SHM/empty WAL与未知文件对照，但三次extras扫描会更早拒未知文件，不能据此证明**最后**守卫被触达。目标是extras结束后才到达的未知条目；合法.lock之外没有对象才成功。缺精确末窗口时保留open，不能改整个purge为宽松清理或按“残留可手工删除”判等价。

可减少无收益执行的一项**当前caller静态候选**：traverse_root:198的dot/dotdot OR→AND。所有Home路径先经Home.resolve的最深真实祖先+合法尾部；ExternalReadFile普通入口先std canonicalize父目录，no-follow入口及ExternalReadTree先canonical_external_root→lexical_abs；create_root只接受Home.root。因此当前实际producer送入traverse_root的根没有`.`/`..`段，两个谓词恒假。AbsPath::new自身只核absolute，不能单凭AbsPath类型作证明；要冻结全部当前caller和这些producer，不能扩大到将来未规范化的内部入口。

其它重复检查没有自动静态处分。观察路径、第二次stat、lock等待与SQLite打开会读取新的外部事实；只有完整caller证明同一不可变局部值的保护才可限定记录。每组保留准确EOF/退出码、完整SQL行、原bytes/mode/dev/ino、关闭FD/线程join和实际观察点。
