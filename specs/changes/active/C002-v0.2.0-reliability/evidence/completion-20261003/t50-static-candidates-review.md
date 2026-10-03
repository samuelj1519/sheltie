T50 两个限定处分候选与traverse-root前置覆盖的独立只读复核。未参与oracle/生产修改，未跑新探针，未写PASS批注。HEAD及当前T50工作区源码SHA见同名JSON；ID沿原inventory，实际行号已因观察点插入漂移，不能只按旧行号执行。

**ensure_dir_unlocked 原:398→false：稳定对象下重复拒绝成立，不能据此给无条件全观察等价。** 外层 `statat` 的S0与 `open_directory_at` 内层 `statat` 的S1是两次真实查询。稳定普通文件占位时二者都给相同InvalidRequest和完整「不是目录」路径；稳定合法目录也走相同打开流程。但S0不是pin对象的FD。

不必假设inode复用就有诊断反例方向：S0已经观察到普通文件后，将文件移走；原guard依据捕获的S0立即InvalidRequest，变体进入内层查询得到NotFound/I/O。内层成功打开后的dev/ino复核以及外层再fstat，在**内层已失败**的路径根本没有机会支配此差异。另一种file→directory更换通常会被外层identity比较拒绝；本报告没有证据证明macOS会复用同inode跨类型，因此不将它说成已存在的成功绕过，也不以“绝不复用”做等价假设。

当前 `ensure_directory_after_stat` 点在match的Ok fallback arm：原bad-kind会在此前返回，它不是两实现共享的S0观察点。若要精确同一事实的反例，必要观察点应置于第一次stat结果取得后、match guards之前；仅在failpoint feature启用，用真实文件rename、reached/release及RAII join。若保留当前点，须明确原分支提前结束/变体到点的行为和控制例，不把wait timeout当业务断言失败。安全拒绝集合在稳定对象下的限定证明，可以保留为有限静态事实；全观察等价仍open。

**Home.confine 原:192 parent!=base→true：有无需inode假设的真实API反例方向，不能静态关闭。** 唯一生产caller `WorkService::start` 已在私有pending下建立inputs_dir，再为合法start key调用confine。目录刚建立和持HomeLock只约束协作writer，不是base在接下来canonicalize/metadata期间永远存在的证明；base_canon只是已经捕获的路径值。

在base canonicalize成功后将base移至同一临时父目录的保留名称，rel取合法不存在叶（如key或new/leaf）。原循环见joined路径缺失，在probe.parent等于base时立即返回合法词法joined，不读base以上。变体继续将probe提升到已经消失的base、再到真实父目录；canonicalize该父目录后，它不以旧base_canon为前缀，误报「经符号链接逃出了根」InvalidPath，整个fixture没有符号链接。可在canonicalize成功、metadata循环前加必要feature rendezvous，保持原API输入与物理rename事实、全部原bytes，不造成功状态。

这只证明confine返回值/诊断的差异。实际work start原分支随后安全创建文件也可能因目录已移走返回NotFound；不主张Work start在base消失后成功、已提交或有最终目录。oracle分别核API词法返回与真实caller失败停止、无Work/request/audit/最终目录；序号已分配可留空号依原合同，不把它误报为新增业务行。

**traverse_root 原:198 dot/dotdot OR→AND：当前完整caller正规化闭包的静态候选成立。** 不能只凭AbsPath：其new只查absolute，直接构造含`.`/`..`字符串仍可能合格。真实闭包如下：

| caller | 进入traverse_root前的producer事实 |
| --- | --- |
| ManagedFs.open_existing→open_root | Home.root字段私有，只由Home.resolve产生；canonicalize_deepest先取真实存在祖先，返回尾部由file_name正常组件拼回，不返回未解析dot/dotdot。 |
| Home.acquire_lock_once→create_root | 同一个Home.root正规化来源；不从任意AbsPath直接造Home。 |
| ExternalReadFile.open_regular→open_root | 父目录已成功std::fs::canonicalize，再转UTF-8 AbsPath；失败不进入traverse_root。 |
| ExternalReadFile.open_regular_no_follow→open_root | canonical_external_root先lexical_abs逐component忽略CurDir、弹出ParentDir，只拼Normal；系统/tmp、/var别名只替为固定正常前缀。 |
| ExternalReadTree.open→open_root | 同上；其managed入口另经Home.root及严格ManagedRelPath，不引入根dot/dotdot。 |
| 当前T50 cfg消费者 | create_root(home.root)仍同来源；非目录占位的open_root(home.rel("occupied"))由Home.root加严格RelPath，依然无dot/dotdot。它测试kind，并非未正规化根caller。 |

open_root/create_root只有上述当前调用者；它们又是traverse_root唯一caller。因此当前位置每个非空segment都不是`.`/`..`，OR/AND两谓词恒假，producer自身的拒绝/规范化仍执行。该证明限定冻结当前caller闭包；未来新原始AbsPath入口或直接未正规化helper测试会使范围改变。不能据此删根guard、放宽AbsPath或称整个root边界全平台通过。

三项都保持原mutant ID/原执行结果。两个open项提供的是可执行方向，尚无本报告动态red；traverse-root只可记scoped_static_disposition候选及源码/consumer边界，不改为caught。
