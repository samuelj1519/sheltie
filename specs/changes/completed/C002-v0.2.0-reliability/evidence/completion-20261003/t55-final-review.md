# C002-T55 / G08 独立最终审查

**PASS，限定当前macOS aarch64的候选源码/oracle及采用的G08验收路径：20旧ID动态、4限定静态、9结构迁移，共33准确旧项。** 原stage1标签不改；9结构项old native仍not_run/Caught=false，不称旧mutant等价。当前候选路径可登记已验收，不能把它写成9次历史native执行或C002-M2通过。Reviewer独立只读，未准备G08方向/实现/oracle，不改tracked或并发构建。全部sourceSHA/逐target panic/diff/log SHA与证据原文见JSON。

A11/11、B9/9旧target动态Caught，164.064/73.185秒，各22非零baseline；C新共享9/9、48.837秒，与旧ID计数分开。广C9很多失败是正常add/start setup错拒，并非每个pure表已执行；因此补同一冻结源的pure-only组：**2非零baseline、9/9、48.473秒**，逐日志只有实际pure Interface test红。Directory true/OR明确`kind=Directory actual=(12,7) expected=(11,7)`返回true/期望false，验证独立sameinum/differentdev语义；另3变体拒绝合法同身份。Sole4由完整手写布尔关系失败。原广C保持原始证据，不用后补覆盖。

20direct逐实际失败核：11项直接Ok或is_err=false（private digest/final check3、root epoch4、global登记list1、recursive namebinding3）；2晚marker检查未给预期StoreCorrupt，精确guard源码后直接Ok路径解释其机制，raw未打印result不假称已打印成功；1删树后错误先创建.deleted再晚拒，直接marker存在断言；2whole-root预检已改权限/原件before-return，full snapshot直接差异；1合法symlink purge被拒；3具体诊断（AlreadyExists cause Io、initial marker跳到晚诊断、临时Io误分NotFound）。这些不是全部最终业务接受，更不自动证明private helper完整original/五表；public Runtime collision/list另有真实请求证据。

独立旧33集合与e54dcd41原SK02 map精确相等，无漏重。24direct当前function/column/genre/replacement与63b05ca同源行块及current inventory/diff逐核；4静态亦按准确目标，不复用同类名称。9target=null已清楚保存previous_candidate_current_id/diffSHA，未给消失表达式造current target。5原215目录connector+4locator connector为9；原目录6语法位置里1595在215外，正式stage1原Missed后已有独立focusedCLI后补Caught/no missing，不造第34项，不改原formal标签。

结构验收按active plan采用路径完成：Directory8关系/三处正确named-stat→held-current-fstat接线/De Morgan同式/cast保等，Sole4关系/四role同式；不误用root_ident，不新增root-device策略、trait/public/state或beforeRetry保证。真实verify/rename/latestSourceStat/delete-last/sourceepoch控制保留；fresh Repo.add经过WriteSession空tmp清理、真实Begin经过PrepareAttempt sync也走第三helper；全部同源回归。Directory first connectors原native跨dev同ino仍无现场，但新纯module合同已实际受测；没有把较晚guard、普通sameDev新ino或数学静态假装成原native反例。

4限定静态单列：起始Ref shape的执行producer全被private CheckedEffects及Work路径/typedSha约束，两个predicate都false，普通输出/拒绝/I/O相同但clone/CPU/OOM不等价；marker同已读bytes经canonical比较仍拒，detail变化/额外pure序列化保留；syncSymlink与preflight非单链Regular fallback落wildcard同拒同停止，reason变化，均不称完整error JSON等价。后续真实sha/bytes属于新观察，不能套shape证明。完整当前caller与失效条件见t55-static-review。

F-T55-01 fixture清理修复及native控制齐：新目录/实际add/start fixture沿用既有FD持根/no-follow/只chmod目录OwnedTempDir，helperfile未改，scope worker先join，outer最后Drop。readonly owned root实际消失；外0755目录与444文件bytes/dev/ino/mode保留，自有hardlink期间nlink2，创建前/Drop后均1。两个新观察点均有真实carrier Io错误移动前/读取前原件保全控制。旧PlainTempDir source/samples、compile与2Miss阶段、修前docs exit1保留，不借新source覆写原失败。

四轮A/B/C/pure190 source/fixture/config完全相同且回读无漂移，gate134项包含其中并匹配；source_unchanged与独立SHA一致。两个Cargo JSON真实executable路径、原/frozenSHA与显式测试env均核，不能按target约定猜。完整工程**935/935、0skip、0LEAK**，run `b062be43-16ad-4137-8760-805efe524d21`；5compile-fail doctest、fmt/check/clippy、Rust1.85/default均exit0。Task22/22与修后docs/specs/tests/skill/core-vocab实际exit0及原文SHA核；原prepare docs1单列，文案修改没有Rust输入漂移。

Owner仍须把9结构current-candidate验收状态登记、Task done后check-task、归档回读及提交，本Reviewer未执行。20/4/9与newShared9+pure9分别计数，原历史native缺口仍如实存在；不批准其它G组、下一任务、C002-M2或C001–C008全完成。
