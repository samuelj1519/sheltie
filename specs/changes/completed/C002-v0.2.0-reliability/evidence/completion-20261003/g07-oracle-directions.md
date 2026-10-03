G07 准确6个旧→当前ID的只读独立准备。未写/跑oracle、未改源码或plan、不授动态PASS；仅macOS aarch64。T51 commit hooks不受影响。精确ID与原/current diff SHA、当前源码SHA见同名JSON。

| 组 | 数量 | 真实consumer/独立期望/窗口 |
| --- | ---: | --- |
| U1 同原件同步来源 | 2 | 历史write_file显式重放及delete marker→sync_managed_regular_file_handle→sync_regular_file_handle_locked。合法原FD/同root epoch/同rel同步；错来源在fsync前拒，之后path换对象必须RecoveryRequired。 |
| U2 观察后绑定 | 1 | WorkService.replace输入验证；delete marker同步后验证→verify_managed_file_bound。合法handle/source相同；换path/inode必须在资格或mark前停，不用旧FD正确bytes代替current name。 |
| U3 封存长度 | 1 | submit正常观察FD与重启后重新open FD→verify_seal_reference→set_readonly。以真实提交ArtifactRef的sha/bytes为准，任何已观察长度/内容不符不得封存或published=1。 |
| U4 观察map/published | 1 | recovery::finish_checked_request的Some(observed)且complete组合。完整当前caller证明受影响的Some+published组合不可达，限定静态候选，不为它编造直接helper参数当当前producer。 |
| U5 输出恰好硬上限 | 1 | CLI attempt submit→observe_output→core decide。声明output.max_bytes=32MiB的真实Flow，恰好32MiB接受、32MiB+1拒，完整rows/ArtifactRef/mode/请求状态独立核。 |

**U1/U2不自动由同FD覆盖来源和名字。** file.managed的root/path只是首次捕获的来源；wrapper后来构造新的ManagedFs，same root字符串不证明same root inode。tmp same-self情形可有局部producer恒等，但历史效果/marker跨wrapper不能套它。错误epoch的真实SafeFile、合法新root/lock、同rel且实际同object的受限API对照可以核来源拒绝；要声称真实CLI已检测该root变化，先确保不是更早check_lock已经挡住，不能裸造ManagedOrigin。

同步helper在fsync前后各有verify_path_matches_handle；这是对live目录项重新读取，不是不可变前置事实。路径真正漂移必须保留原件及准确错误，不能只断言最终is_err。U1的两个||→&&还能因优先级吞root_ident条件，不仅影响“rel”字段。完整SQL行、original/pending_original、published旗标、原bytes/mode/dev/ino与parent sync阶段分别核。

**U2优先复用replace_after_input_open。** 用真实已运行Attempt和已绑定输入：捕获原A的FD后保留A，将输入name换成不同bytes的普通单链接B；原FD读到A，已绑定Ref也仍是A。原实现必须在core替换/创建新Attempt前拒，旧running、replacement额度、revision/request/audit/brief都不动；Fn→Ok可能让旧FD观察通过并把新任务绑定到已变成B的路径。只用同bytes换inode也可检观察资格，但different bytes更直接核worker将读什么，独立期望取原Ref/原bytes，不由renderer算。

delete_marker_after_validation_before_sync在**同步之前**。若在那里换marker，sync helper自身第一次path核验会先拒，不能声称删除verify_managed_file_bound已被检测。目标是sync helper前/后核验已经通过之后，到最终bound函数之前的live变化；仅靠同FD marker JSON正确或先前parent sync不能证明当前name仍是那个已同步对象。需要精准现有点/必要feature观察点，捕获真实删除请求与原completion marker，不在fake里直接写成功事实。预期EFFECT_PENDING、原committed身份/合法original保留、published=0，未知对象不清掉。

**U3不能静态说后SHA/length总兜底。** 持续增长/缩短通常会在read_bounded、actual_bytes或after fstat被后层拒，可能只换诊断/多读；不要把这种样本等同前guard检测。真实反例方向是before fstat已捕获长度N+1后、进入hash read前把同inode内容/长度恢复原N。原before比较仍必须拒；弱化变体可能读到正确N/sha并通过after fstat，继而封存。长度是live可变事实，dev/ino恒定不支配它。现有submit_after_commit_before_seal足以先制造growth，第二个“参考before fstat完成后”的准确窗口尚需核，不能用概率两次写。

此场景只用真实Runtime观察形成的ArtifactRef/FD，两个实际文件操作与reached/release；不手造sha系统事实。核before观察、original snapshot、mode、published与完整rows及拒绝cause；暂时恢复不等于未观察到坏长度。若只测持续growth得到后层检测，按该范围记录，不整项等价。

**U5必须给合法producer开放上限。** 普通样例声明上限64KiB等时，32MiB数据会被core自己的output上限拒，不能验证runtime >→>=。准备一节点/合法方法，明确max_bytes=33554432；worker真实普通单链接文件可用sparse创建恰好此size，合法submit应成功并返回Ref.bytes=33554432、独立SHA、0444/按合同封存、单一请求/revision。再只加1byte，预期OUTPUT_TOO_LARGE且actual=33554433/max=33554432，未登记该请求、不改running/revision/audit/业务原件。不写actor自报摘要，不用被测limit helper算expected；硬上限拒绝先于读全文件，声明上限仍由core守。

**U4完整当前caller静态候选。** whole-crates搜索finish_checked_request/recovery::finish_request全部callsite：before_write恢复直达传None；WorkService.replay直达传None。Work start与Workbook add/remove经finish_request传None/verify_published=false。唯一Some入口是finish_request_with_observed→recover_finish_request(Some,false)→finish_request；若该捕获request.row.published=true，finish_request先在同一局部值上early return，根本不进finish_checked_request；若false则complete=true，两guard都执行同一路。即使Store外部随后改变published，Rust已捕获RequestRow不变，不产生Some+complete=false。未来新增Some+verify_published=true或直接Some直达caller才会改变闭包。

因此此单mutant在冻结当前完整caller下不会改变成功/拒绝、I/O或错误JSON，可做scoped_static_disposition候选；这不是任意私有helper参数的全域等价或实际caught。保留原mutant标签和未执行状态，不为关闭义务发明假请求/假published状态。

每组先真实非零baseline及合法对照，再精确少量red；窗口没触到、前层拒、timeout、Source drift分别停止/保存原文。当前准备只给入口、规范事实、观测与静态范围，后续实际结果需作者执行并由未参与实现者复核。
