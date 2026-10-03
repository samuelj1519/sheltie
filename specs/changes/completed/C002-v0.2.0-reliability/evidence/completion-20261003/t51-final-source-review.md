# C002-T51 最终源码与 oracle 准备审查

结论：**通过，限定当前14条oracle、观察点及私有payload入口。** 无剩余必须修改的源码finding；27个最终动态ID、完整门禁、MSRV和最终提交范围仍需后续实际闭包。Reviewer `/root/oracle_review` 只读，未编写代码/oracle，不重跑构建或修改plan。

基准 `b5226d07a8abca1c2a1b7eaee4de71eb19cf770d`；fsx SHA256 `241b16b73ec2f8288966b25cf9e2d236b99ee436b8914d2ee4bfb4242ae06989`，failpoint `5a9268044be37aa05441aa81b66bc2ce6d4b1c708b19dd48b954615922e6889d`。完整属性与边界在同名JSON。

生产改动只是在实际fd读取/生成后open前/成功open后/写synced仍持FD的间隙加feature-scoped rendezvous，既有守卫/访问mode/随机tmp算法不改。默认无feature时沿原no-op，无新增FS/SQL或注入环境读取；额外path字符串构造不说零成本/AST逐字相同。Core和生产public接口不变，没有公开FD getter、通用observer trait、新业务状态或另一个随机名来源。

私有rendezvous_observed_path只在atomic生成实际tmp_path后/open前调用；old rendezvous继续把原name写reached。payload复用原name/scope匹配、临时reached→rename发布、release与配置。test-only observe_recorded_change只是向当前actor提供已存在sync目录，原observe_change委托它但仍只调用原change；不是泛化调度框架。T51白名单已包含fsx/failpoint和本package，测试变化获准；未发现范围外源码变化。

14条oracle期望来自真实内核fcntl、手写access2/0/mask3、原bytes/mode/dev/ino、真实Root公开/私有消费者；不复制生产flags表达式。real exec只enumerate/fstat /dev/fd比目标devino，不打开target或重写继承flags。本机stable std过程源实际没有CLOEXEC_DEFAULT/closefrom blanket clearing；仍需去CLOEXEC对应mutant让child出现匹配fd验证辨别力，empty list只证明当前spawn路径，不证明所有并发spawn或平台。

短生命周期FD在post-sync/after-open marker时仍live，actor真实child执行完才release/join，字段早返回导致未到点/未probe不算机制PASS。primitive因丢CREATE/WRONLY提前拒绝合法setup或write是另一实际维度，不能把它叫fcntl/exec结果。sameinode symlink反例合法：rename保存A完整对象，leaf→symlink回A，少NOFOLLOW会跟随A且fstat相同，正好排除“同inode说明名字未变”的假证明。

atomic before-open碰撞使用记录的该次真实tmp，不猜UUID；post-sync唯一temp扫描只为已持FD的CLOEXEC。普通collider、alias及外sentinel按实际bytes/readlink/mode/devino保留；不发布final，已有managed目标亦不截断。directory/file类型反例和sameinode tree delete/chmod反例位于准确open间隙；若错误仍在晚fstat/read_dir发生，只有诊断/cause/机制差异，不泛称非法对象被接受。

FIFO初版writer线程和Drop join unwrap已删除。现在release前建立O_RDWR|NONBLOCK rescue OwnedFd并持到operation完成；若建立失败，在放行前移除FIFO/还原原目录再panic。after-open marker有预置release；正确DIRECTORY必须NOTDIR且无after-open reached，DIRECTORY丢失会实际打开FIFO并写reached，晚read_dir的NOTDIR不能掩盖错误open。此边界不靠timeout、没有可挂起probe writer，也未操纵CLOEXEC probe继承。异常assert时rescue FD与scope自动回收；永久OS恢复错误不在本TempDir可回收证明内。

新增1 test的3个真实marker失败分支已核：managed-sync/atomic-sync在file.sync后报RecoveryRequired，保留精确bytes、普通file/nlink1/0600/dev和实际retained path；atomic-before在open前报Io、没有temp创建/没有final。carrier真实非目录错误不被写坏，Store仍不存在。函数名的committed仅描述文件bytes，**没有SQLite COMMIT**；file sync也不证明parent sync/断电持久。其余目录after-open错误只证明该阶段的访问/副作用边界，不自动转五表/完整CLI资格。

`t51-marker-errors-baseline.txt` 当前14/14 PASS；first7、12A10/2、next10的8/2及初次compile101各自保留。剩余两个DIRECTORY-only最后变体仍待实际验证，不把准备Source PASS、FIFO合法基线或语义分析当27捕获/完整T51验收。
