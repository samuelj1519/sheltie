G04 27 个精确旧→当前ID的只读独立准备；仅macOS aarch64。未编写oracle、未执行突变、不给PASS。逐ID原diff/current diff SHA、实际相邻flag、Darwin变体位值与分组见同名JSON；T50冻结源码/当前回归不由本报告改动。

先处理位代数再选consumer。已只读核Cargo.lock中的rustix1.1.4/libc0.2.189 primary源码：Darwin RDONLY=0、WRONLY=1、RDWR=2、CREATE=512、EXCL=2048、NOFOLLOW=256、NONBLOCK=4、DIRECTORY=0x100000、CLOEXEC=0x1000000。此处相邻非零flags都不重叠；Rust `&` 优先级高于 `|`，所以一个mutant常同时丢两个相邻flag。RDONLY为0不代表另一flag消失无影响；RDWR也不是RDONLY|WRONLY。JSON对每项解析原column与两个相邻token，不凭字段名猜变化。

| 组 | 数量 | 真实consumer及共享独立判据 |
| --- | ---: | --- |
| F1 独占创建 | 5 | ManagedDir.write_new（现行直接消费者主要为持句柄真实集成测试）、ManagedFs.write_new_observed及write_atomic；后两者经owner/start-input/Store staging、历史brief/card原子发布等真实CLI调用。新leaf正确bytes/mode；已存在普通/硬链接/链接占位不改bytes，不覆盖/越界。 |
| F2 根锁 | 4 | Home.acquire_lock / readonly schema retry的acquire_existing_lock；普通单链接合法，无root/lock不得由existing路径创建。锁类型/FD访问mode、NOFOLLOW、NONBLOCK/CLOEXEC按具体实际bit损失分别核；记录root/.lock dev/ino、原bytes与准确错误阶段。 |
| F3 目录打开 | 4 | ensure_dir新建分支3项、open_directory_at1项。mkdir后的实际open与已有目录的stat→open不是同一个callsite；错误类型或别名不得转成可写/可遍历句柄，捕获外部新增目录/权限等副作用，不只最终is_err。 |
| F4 外部树读取 | 2 | WorkbookRepo.add/copy/load→ExternalReadTree.open_file；核读取原字节、32MiB/256MiB边界、名称不跟随及父/叶身份。已有external_tree_after_stat是准确leaf窗口。 |
| F5 删除/权限树 | 12 | Workbook remove/pending cleanup/purge→remove_at/make_directories_writable；add/start冻结与发布→set_dir_tree_mode的目录和普通文件两分支。根外sentinel bytes/mode/条目完整不变，合法树才删除或置只读，失败保留对象/状态及准确committed响应。 |

**一个共享“同inode别名”场景比镜像每个flag更有辨别力。** 让真实consumer先stat一个合法普通文件或目录A；在现有after_stat屏障后将A移至保留位置，并把原leaf换成指向A的symlink。A仍是同dev/ino、同bytes，故仅fstat元数据或“同FD恒定”不能证明名称没有被跟随。原NOFOLLOW必须在open拒绝；丢NOFOLLOW的变体可能跟随回同A并通过全部dev/ino比较。读取组核原查询严格拒绝；目录/删除/权限组还核外部树没有新增child、删除或chmod，即使后层最终也拒绝，不让先发生副作用被最终错误掩盖。所有对象仅在自有临时fixture内，保留原件，不操作真实用户Home。

F4可直接复用 `external_tree_after_stat`。F2 existing可复用T50的 `existing_lock_after_stat`。F3已有目录用 `directory_open_after_stat`；新建目录则在mkdir/fsync后直接open，已有helper点不在该分支，需先核实际观察点是否触到。F5的remove_at/make_directories_writable/set_dir_tree_mode有各自直接open，不能拿generic helper点代替它们；必要时由原语作者在真实stat/预检后、实际open前加入仅feature观察点，不写第二套文件操作或fake成功。

**非阻塞与关闭继承分别观察，不靠测试超时。** 丢DIRECTORY的readonly新建/目录open或丢NONBLOCK的普通文件open，若leaf在检查后变成FIFO，可能阻塞而不是准确拒绝。用受控子进程/线程、有界握手与RAII回收；若用FIFO writer放行，记录真实kernel握手而非sleep概率。实际timeout仍是timeout，不能记caught。也可在真实已打开句柄上通过rustix安全 `fs::fcntl_getfl`观察access mode/NONBLOCK，通过 `io::fcntl_getfd`观察CLOEXEC；期望来自对应接口/采用机制，不能用被测OFlags表达式算同一个答案。创建类EXCL/CREATE等打开时选项不可假设F_GETFL会全部保留；NOFOLLOW是否拒别名应由真实open行为核。

CLOEXEC差异若只有FD配置观测，准确标为配置/机制检测，不扩大为真实跨exec隔离或产品整条链。只有当前consumer/采用合同能支持的义务才处置；不要为了杀mutant创造新宿主安全保证、公开FD getter、trait或泛化observer框架。短生命周期FD已关闭也不能声称已经证明所有并行spawn或继承行为。

**可以证明的producer覆盖只覆盖某一安全维度，未必覆盖整个ID。** F1中仍保留CREATE+EXCL的变体，在同一次原子open里对任何已存在leaf（包括dangling symlink）都失败；创建成功只会是新普通文件。因此这一producer保护其丢失的NOFOLLOW维度。可是同一变体往往还丢CLOEXEC，不能把整个ID记等价。若EXCL也被&消掉，则必须测已存在普通文件与外部别名sentinel，CREATE且无TRUNC也会从offset0改写已有内容，不能只断言文件长度没变。

write_atomic的随机tmp名不宜靠猜UUID或追逐竞态。若要检验tmp碰撞/别名，必须由真实生成后、open前的精确观察记录取得该次leaf；不修改随机算法、采用固定假ID或扫描到一个事后文件当事前oracle。无法证明必要窗口时保留open；CLOEXEC等可用精确实际FD事实另行有限检测。

所有stat与open前后都是活的外部事实；DIRECTORY/type或NOFOLLOW/name不由后一次同FD dev/ino恒定自动支配。合法producer、后层拒绝、错误payload与外部副作用要分开报告，不能把“不成功”都当同样可靠。先跑每个共享consumer的非零合法基线，再少量代表验证实际mutant red；未命中窗口、预算终止、源hash漂移或仅提前的其它guard拒绝就停止扩展，并保存原raw。27项每ID保留准确caller、观察点与检测/静态范围，不批量equivalent。
