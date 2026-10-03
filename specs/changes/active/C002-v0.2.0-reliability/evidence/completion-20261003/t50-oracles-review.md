# C002-T50 oracle 准备独立审查

结论：**通过，限定当前17条oracle及6个观察点准备范围。** schema RW控制持锁死锁与权限恢复问题已修；新marker错误分支有实际保全测试。26个精确ID最终处分及完整工程门禁仍须后续原文，不由本报告预判。

Reviewer：`Codex /root/oracle_review`，未参与G03方向、测试或实现编写。只读审查，不改源码/oracle，不构建。基准 `44d6d1b4ed05c13637122047d552e1e4fa6781cd`；当前fsx SHA256 `1010aa21a78170d9b92a91048f8d4ee4d04798befd91e991c114af1da20807bf`，home `cf6cb4470000003c412f5b8523853ba4b3840e9d828a1f5831182900ed90529b`，store/mod `fe3654a0baec00afc8e67b638529714c27fbee0468cedf4a1692277588d07534`。

## 6个实际观察间隙

| hook | 源码事实与错误边界 |
| --- | --- |
| existing_lock_after_stat | 普通单链接旧stat已捕获、openat/fstat尚未发生；置换与硬链接由真实对象触发后检拒绝 |
| ensure_directory_after_stat | 缓存首次stat Result后、match类型守卫前；目录置换和捕获普通文件随后被移走可区分原kind与晚查询，不把两次stat当同值 |
| new_file_before_identity_check | 内容write+file sync已成功、较晚metadata及parent sync前；hook载体I/O失败为RecoveryRequired，保留已创建文件，不以普通Io/写失败清理掩盖 |
| purge_after_control_rescan | 受控SQLite晚sidecar重扫后、最终unknown-entry扫描前；hook失败沿partial_purge_error报告真实已删除roots，保留root/同lock |
| store_before_metadata | root/lock/SQLite控制预检已过，Store path的std symlink_metadata尚未读取；真实chmod拒绝不得归NotFound |
| confine_after_base_canonicalize | base_canon已捕获，但不是证明base还存在的FD；base消失后返回词法target，不向上越界查ancestor，不声称Work.start后续写入成功 |

默认无failpoint时调用已有rendezvous no-op，新增调用不做FS/SQL动作、不读取注入环境，不新增默认错误结果或生产API。ensure-dir把原statat Result提到局部值再match，读取次数/顺序仍一致。并非默认AST逐字不变或零分配成本声明；新错误映射只在启用feature且配置实际marker失败时可达。

## helper、权限和对象保全

observe_change的泛型scoped worker支持返回真实Result与借用参数。worker在reached前已结束时直接join并返回真实结果，不能等待10秒把失去窗口伪装捕获。正常change后release再join；body panic时Release守卫写release/disarm，随后thread::scope保证join，临时同步目录仍存在。fixture不能达到点且worker未结束的预算失败/timeout须记失败或未完成；早返回的捕获必须按真实API/等待义务判据归类，不宣称发生了未执行change。

schema原先在持held_lock时同步RW控制，mode==ReadOnly→!=会等待自己的锁而死锁。现在稳定RW控制在无持锁状态拒旧schema并核原bytes，随后独立held lock让RO等待真实ManagedFs.purge、释放锁后重查NotFound。schema mutant若RO提前返Mismatch，helper返回该真实旧结果而非timeout；实际purge未发生时不得说“purge后返回旧Mismatch”。

PermissionRestore在chmod前取得原permissions并安装RAII，FS锁、Home setup和Store后预检三处正常返回前显式drop；operation/observer panic也尝试还原，不会仅因控制流跳过恢复。Drop不再panic以免双重unwind；还原是OS调用best effort，不宣传永久I/O失败下必然成功。权限denial来自实际root0o0而非fake Error；纯Home classifier的手写Error参数只证明其分类合同。

同FD的dev/ino固定不代表nlink固定。new-file合法控制单链接，较晚FD实际变nlink2/0要求RecoveryRequired；已有alias或held FD读精确原bytes、mode/dev/ino保持。unlink case数据保全仅证明held FD生命周期内仍可读，不能说最后FD关闭后无名字的inode持久存在。lock、目录置换、late sidecar均保存原bytes/对象identity；不存在Store的路径测试用主库absence证明无业务行，不能冒称执行过五张不存在表的SQL读取。已有Store权限case核main原bytes与对象，SQLite控制文件例外不混为“全管理根物理无写”。

marker失败用真实非目录sync载体，未注入fake成功/错误。新文件原精确bytes、普通类型/nlink1/0600/device保持，载体不被改写；此时parent sync尚未执行，不证明断电持久性。purge真实删除works后失败，准确报告removed roots、原root和lock dev/ino/mode保留，载体不动。显式purge造成的删除是合法fixture操作，不记读取侧无副作用。

## 实际准备基线与停止边界

原11/13/14/17基线按各输入保留。最后missing-base补正后的17基线 `t50-missing-base-baseline.txt` 全PASS；函数内多个条件不按函数数冒称端到端覆盖。pure classifier、私有FS原语、真实Repo.add和Store/schema真实SQL各按实际消费者分层，不互相充当完整CLI链。

第一轮最终A12Caught/B11Caught+1Missed/C未派发保存于mutation-g03-first-final。Missed是Home::confine初始base canonicalize NotFound守卫→false：原测试base都存在/消失发生在canonicalize之后，现补原本缺base合法公共API控制，手写绝对target且无ancestor创建；旧Missed不改Caught。

仅改变测试fixture/helper/identity判据提取与SQL排版的简化不改变生产或握手；from_mode(0)→0o0的Clippy修正是测试字面规范，原preclippy exit101保留。最终25动态+1traverse根规范限定静态、闭包和工程结果由后续t50-final-review核，不给完整T50、其他SK02组或M2预先PASS。
