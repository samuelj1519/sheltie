# T43 当前候选源码变化资格

本文件为独立只读的源码/caller复核，不把旧运行升级成当前执行，不单独授T43 PASS。绑定HEAD `4487ac1eb20672b8136aba186e8392fafa38ebca`；精确SHA、当前21项静态条目及原stage1核对见 `t43-static-current-delta-review.json`，各冻结190差异见 `t43-later-frozen-closure-audit.json`。

## 冻结分组到当前的实际变化

从早期06e5a91到当前，service/workbook_repo/recovery/snapshot/store.commit/schema生产文件逐字未变；store.read、load、effects、workbook_digest仅追加测试。后续T53没有修复workbook_digest生产算法，UTF8/frame两例只补真实Read的独立判据。

FSx/Home/Store的新观察点仍走既有feature控制函数；没有匹配arm/env时不增加文件操作。enable feature但无对应name/scope的调用保持原I/O、锁、stat/open/check、EXCHANGE、fsync/rename/unlink顺序；新观察失败支路只在真实配置命中时保留对象/返回既定类别。rendezvous原name包装以原name bytes调用私有payload；仅新generated-tmp观察点传实际路径，既有name/scope选择与release loop不变。默认feature不新增观察I/O；新增tmp-path字符串构造的分配成本不称全资源等价。

T55两个私有pure函数分别定义kind+dev+ino身份与candidate&&!other。当前三个目录调用仍先真实statat再fstat，expected只来自held对象；Darwin字段转u64保留相等关系。四locator调用的candidate/other角色及before/after读序正确，published=false路径与最终重试/NotFound/Io分类未改。没有采用“首次both后必须再重试”额外保证。

T56仅在write_all实际成功后、原file.sync_all前加入root-scoped模型sync失败；创建FD/metadata/回收同A或保全B逻辑原样，失败观察点在既有补偿前；directory sync仍check_lock/open_dir后才注入。模型故障不是原生fsync、填盘、partial-write或断电。T57只追加测试，不改生产或观察点。

上述新增name与旧各fixture的name/scope不同，单次arm匹配未改变旧pause所在stat/open/commit窗口；T49两测试原件/五表SQL、owner双观察/audit/index/copy停点保持，T50文件kind/mode/missing-base、schema/purge窗口保持，T51 fcntl/exec/collision/FIFO目标FD持有阶段保持，T52 EXCHANGE双端stat与旧epoch/candidate绑定保持，T53 Read之前最终fstat/manifest/copy边界保持，T54 Sync/Seal/max-limit保持。新增测试未改变先前断言或helper；selected/parent的资源释放、RAII、PermissionRestore、scoped join仍按各独立报告的准确边界。全部后来分组190共同的根Cargo/lock/config/外部fixtures字节未变；文件差异是所列观察/pure接线或cfg(test)增量，不能把这份复核叫作185次最新重跑。

## 限定证明

当前21项静态：T49十二项保持其完整immutable producer/caller、SQL work/revision1、strict资格及后层纯guard，八项观察相同、四项诊断文本差异保留；T50 root正规化完整当前caller限定不扩大到任意AbsPath；T53只证明同Read轨迹正常完成的UTF8分类，invalid pending可能增长及近二次成本保留；T54 Some observed仍仅verify_published=false并被同owned-row早退支配；T55四项shape/canonical/分支拒绝限定保留诊断/分配差异；T56两项纯索引最终len拒绝或immutable None/None continue仍成立。没有新增绕过producer的生产caller/字段访问。

当前9项结构：5目录connector+4locator connector原语法已移除，current target为null。T55模块源、3+4接线、完整纯关系表及真实verify/rename/delete/symlink/preflight/candidate/cfgpoint failure控制到当前未变；T55真实OwnedTempDir Drop后根消失、外部sentinel/root metadata不变的native cleanup用例仍逐字存在。原broad新shared9及独立pure-only9实际红仍各自同冻结190证据，后者两pure baseline非零；新9不是旧9native执行，旧native not_run/Caught=false保留。

旧与当前206可表达target中204项删除/替换行逐字相同；例外为verify_tree_at全函数删除（目录条件pure替换）和sync_dir_locked全函数删除（新增模型sync设施），分别已有T55/T56真实当前consumer红。九个函数名变化仅为已审T48command data helper/T49load_row_at/T57rendezvous_payload，同一表达式/column/genre/replacement对应，不复用旧行号猜测。

185动态的每项准确分类延续原独审：合法控制误拒、坏事实实际接受、准确首错诊断、FD fcntl/exec/FIFO opened机制、已写私有copy目标后晚拒、已观察frame矛盾后来源唯一的Read错误等分开。T56 cache原红先出healthyA publishedfalse、UUID先出warning缺失；T57 env匹配原红仅证明before_commit未reached/child已结束，未打印child status/stdout为unknown。不能把这些都称最终非法产品成功。
