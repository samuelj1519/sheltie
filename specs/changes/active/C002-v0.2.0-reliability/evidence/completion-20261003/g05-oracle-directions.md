G05 16 精确旧→当前ID的只读独立准备，仅macOS aarch64；未写oracle、未执行、不给动态PASS。逐ID/原diff/current diff SHA及冻结生产源码SHA见同名JSON。五组按真实consumer与独立原件/源行判据组织，不镜像每条mutant。

| 组 | 数量 | 真实consumer/共享判据 |
| --- | ---: | --- |
| R1 verified EXCHANGE | 2 | self install覆盖现有binary→replace_verified_managed_file；候选A由真实SafeFile/expected bytes，旧target B有独立dev/ino/bytes。成功target=A、source=B；不明端点必须RecoveryRequired保留全部原件。 |
| R2 path EXCHANGE | 7 | self update在current与旧prev并存时→replace_managed_regular_file；恢复失败也可调用该原语。两端普通单链接A/B，成功交换完整bytes/dev/ino；源/目标后置类型/nlink/身份异常不能被同步成功掩盖。 |
| R3 按原件删除 | 2 | tmp过期regular cleanup与pending owner/deleted-marker cleanup→remove_regular_file_if_same。只有同root epoch、同rel、同观察对象才unlink；不明/替换对象保留，维护告警与业务响应分开。 |
| R4 set_mode | 2 | submit封存→set_readonly；self候选→verify_and_make_executable_confined→set_executable。根来源资格先于fchmod；持有原件FD与之后重新open的current name必须一致；旧FD正确不等于current正确。 |
| R5 rename源资格 | 3 | self update/rollback移binary与备份、ManagedFs/ManagedDir.rename_new。合法普通单链接及支持的目录才移动；硬链接/链接/特殊对象拒且from/to/外部原件不变。 |

**R1/R2共享四个真实边界案例足够覆盖多条关系。** 基线用真实A/B记录每端初始bytes/dev/ino/mode/nlink，再调用原语；expected取初始快照和手写EXCHANGE关系，不调用被测helper计算同一个答案。negative在两端pre-stat/候选路径验证均完成之后、EXCHANGE之前改变一个条件：

1. 将已观察的旧target B移至保留名，放另一普通单链接C到target。exchange后source=C、target=A，source不再是旧B。原语必须RecoveryRequired；同filesystem dev相同但inode不同。覆盖R1源端点及R2 source inode/邻接&&条件。
2. 将已观察的source A移至保留名，放C到source；verified版本继续持原A的真实FD和expected A bytes。exchange后source=B、target=C。原语必须RecoveryRequired，不能只verify原A FD仍有正确bytes就宣布target合格。覆盖R1 target关系及R2 target inode。
3. 在pre-stat后为B增加硬链接。exchange后source的nlink≠1，必须RecoveryRequired保留端点；type仍Regular、dev/ino仍旧B。覆盖R2 source nlink及邻接dev连接符。
4. 在pre-stat后为A增加硬链接。exchange后target的nlink≠1，覆盖R2 target nlink及邻接dev连接符。

原EXCHANGE可能已经生效，失败**不应自动反向交换、删除C或当原B清理**；独立比对source/target/保留A/B/C全部原件，不只检查target存在。verified FD的后层内容复核可再挡某些hardlink情况，需要记录准确失败层次，不能说第一关系guard已被检测。这里不需要跨设备、SHA碰撞、inode复用或新平台机制。

现有 `update_after_candidate_verify` 在helper自己的pre-stat之前，过早修改会被重新观察成新的pre-state，摸不到交换后关系。当前没有交换专用观察点；如必要，由原语作者在真实pre-stat完成/EXCHANGE前放feature限定握手，scope精确端点；不引入新状态、mock交换或泛化框架。仅after-EXCHANGE扰动也可测关系，但必须清楚区分捕获事实和实际已交换事实、双方sync与保留策略。不要让单端点stat失败替代目标关系反例。

**R4可先用不需要新observer的真实current-name对照。** 在同一root中打开合法A获得SafeFile，将A移至保留名，另放普通单链接B到原rel，再调用set_readonly/set_executable。fstat原held A正确且只改变A mode；后来的self.open_regular(rel)捕获B，原式必须拒绝，B bytes/mode不变。dev相同而inode不同足以区分OR→AND，不能拿“同FD恒定”覆盖这个比较，它比较的是原held与重新open的另一个对象。实际submit还可复用提交后封存观察点核EFFECT_PENDING/current original/准确cause；若前层已拒则只计前层检测，不当最终guard被跑到。

R4的origin root/ident，以及R3的origin root/ident/rel是另一条资格。相同root字符串并不证明同root inode；root被移走、同一路径重建形成新epoch时，旧SafeFile的managed origin仍是旧值。可用真实构造出的SafeFile、保留旧root、新root与合法新HomeLock做bounded API拒绝对照，绝不手造ManagedOrigin/private字段。set_mode必须在任何fchmod前拒错误epoch；最后current重读失败不能抹掉此前对错epoch原件的mode改变。

如要声称真实CLI/cleanup消费者已检测root epoch，必须证明到达此guard：这些caller持有原HomeLock，简单替换root通常先被check_lock挡住。需要精确root变化/捕获/恢复关系，或记录只检测了更早lock guard。SafeFile的open不自动带同一锁检查，pending wrapper之后会重新open ManagedFs；root变动后恢复原root的窗口不能被“HomeLock还有效”无条件支配。API反例也不能自动扩为全CLI的动态证明。

**R3只有一个有限producer覆盖事实。** cleanup_expired_tmp在同一个self上open_regular(&path)，随后把同一个path交remove_regular_file_if_same；origin.root/root_ident/rel由同一self/参数复制，三项在该consumer恒等，与后续name变化无关。pending owner/marker cleanup则先取得handle，后通过wrapper重新构造ManagedFs；不能套同self证明，root epoch需要独立核。即使rel由id/固定suffix保证相等，||→&&仍可能吞root_ident异常；没有所有caller闭包证明，不能整IDequivalent。

R3可补同rel的替换原件、错误epoch，以及path不同但同一个file经真实rename移过去的API拒绝对照；后者只是API资格场景，要与生产caller始终用同rel的事实分开。每次核当前path指向的实际对象、原held原件仍在、unlink是否发生与parent sync结果。不要只测试已经消失的路径（早期NotFound会掩盖origin谓词）。

**R5用一个hardlinked regular negative覆盖三个布尔变体。** 合法单链接regular移动成功；negative只为同一个源增加第二硬链接，rename_new必须拒且from仍在/to不存在，外部原件bytes/nlink不变。原条件为S（symlink）∨H（regular且nlink≠1）∨X（非regular/dir）。两个||→&&分别把H或特殊对象约束吞掉；kind==regular→!=regular也会放过hardlinked regular，并可能错误拒合法目录。目录正例与FIFO/链接拒绝作为共享补充，不靠顺序等待或读取FIFO产生超时。

所有观察点必须有reached/release、RAII/join、非零合法baseline与实际red；新root/保留树均在自有temp，有界输入/读取，不改真实Home。原变异标签、skips、timeout及源码闭包分别保存。该准备只提供规则、入口、窗口与独立oracle方向，不参与测试实现，不把保护某个consumer扩成所有producer或未来平台。
