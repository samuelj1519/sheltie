# C002-T50 / G03 最终独立审查

结论：**通过，限定当前T50/G03候选。** 25个精确动态ID实际Caught、1个完整当前caller闭包限定静态，无漏无重覆盖原G03的26项。默认feature行为边界、17个新增oracle和6个观察点审查通过；不是全产品/其他SK02组、原候选、跨平台或M2验收。

Reviewer：`Codex /root/oracle_review`，未编写G03方向、oracle或实现。只读核代码、实际失败原文、输入闭包、二进制与工程结果，没有修改源码或重新执行测试。基准 `44d6d1b4ed05c13637122047d552e1e4fa6781cd`。最终fsx SHA256 `1010aa21a78170d9b92a91048f8d4ee4d04798befd91e991c114af1da20807bf`，home `cf6cb4470000003c412f5b8523853ba4b3840e9d828a1f5831182900ed90529b`，store/mod `fe3654a0baec00afc8e67b638529714c27fbee0468cedf4a1692277588d07534`。

逐ID失败、分类、log/diff SHA和闭包细节在同名JSON；准备期的纠正、阶段和保全范围见 `t50-oracles-review.md`。

## 观察点、默认行为与接口

6点为 existing_lock_after_stat、ensure_directory_after_stat、new_file_before_identity_check、purge_after_control_rescan、store_before_metadata、confine_after_base_canonicalize。分别位于旧锁stat→安全open、首次目录stat→类型match、file写/sync→较晚metadata、purge控制重扫→最后unknown entry扫描、Store控制预检→path metadata、base canonicalize→ancestor probe的真实间隙。

默认无failpoint时沿已有rendezvous no-op，无新FS/SQL动作、注入环境读取或默认错误结果。stat Result只是从inline match提到局部后match，读取次数及默认顺序不改；不冒称默认AST逐字相同或零分配。3个改动src文件仅上述观察点、feature-only错误映射和cfg(test)支持；无Core修改、无生产公共接口扩展、无新依赖或宿主配置/安装。

写后marker真实非目录错误为RecoveryRequired，已同步新文件/当前端点保留；parent sync尚未执行，不证明断电持久。purge末marker真实错误沿partial_purge_error报告实际removed roots，原root/lock保留。旧schema RW对照已经移到无持锁状态；另持锁RO必须等待真实purge再重查，避免mode变体在自己持有的锁上超时。

scoped worker正常/异常都由Release与scope释放并join；早结束返回真实Result，不以未到点10秒预算当captured。PermissionRestore在chmod前安装，3处失败unwind仍尝试恢复；Drop为best effort，不保证永久OS错误下成功。unlink数据保全只到held FD生命周期，不宣称最后FD关闭后无名字inode仍持久。不存在Store证明无业务行，实际SQL/bytes与mode/dev/ino按各测试消费者核，不借纯classifier或privileged原语声称完整CLI/宿主安全。

## 25项实际执行

| 批次 | 秒数 | 实际完成 |
| --- | --- | --- |
| A | 123.401 | 12/12 Caught、exit0 |
| B | 75.213 | 12/12 Caught、exit0 |
| C | 16.764 | 1/1 Caught、exit0 |

三批各有17/17未变异真实baseline与完成end_time，0Missed/Timeout/Unviable，source_unchanged=true。独立逐log全部Build Success、Test Failure100；目标断言实际失败，不拿编译或未到观察点超时补数。

| 实际检测 | 数量 | 保留的边界 |
| --- | --- | --- |
| 非法对象/不完整操作被接受 | 10 | 非单链接/换对象锁、目录置换、新写FD nlink、late未知purge对象、锁后orphan sidecar；直接错误接受或错误成功 |
| 错误类别/cause或假absence | 5 | permission/NotFound误归、捕获非目录变晚NotFound、注册目录变InvalidRequest、外部missing parent误归Io；不能全写成“非法对象被接受” |
| 诊断路径/阶段差异 | 3 | traverse_root前后拒绝定位、FIFO前后guard、display_path为空；仍拒但定位不同 |
| 合法词法API输入被拒 | 2 | base原本缺失与base canonicalize后消失；仅公共confine返回值/边界，不证明之后Work.start提交成功 |
| 只读等待/重查机制 | 2 | schema模式变体在reached前返回真实旧Mismatch，违反RO等待义务；此时purge未执行，不说“purge后返回旧Mismatch” |
| 纯retry classifier合同 | 3 | 手写Io参数对应Root/NotFound/PermissionDenied分类；不充当真实锁恢复完整链 |

特别核 fsx display_path→empty 的原log：首个marker/new-file失败来自hook scope被清空，未发生预期change，**不采用**为产品检测。采用同log `a_captured_non_directory...` 的真实诊断路径为空与独立手写绝对path不符；原对象没有被移走，不冒称after-change身份检测。

17条最终oracle基线包含新增missing-base控制。第一轮A12Caught/B11Caught+1Missed/C未派发保存在mutation-g03-first-final；其Missed是base canonicalize NotFound guard→false。补正后新运行Caught是新事实，不回改原Missed。preclippy101及其它初次fixture结果各保留原文。

## 1项静态与26唯一覆盖

精确 `fsx.rs:198:31 ||→&& in ManagedFs::traverse_root` 限定成立。不能以AbsPath类型推断规范化，AbsPath只保证absolute。独立核当前全部入口：Home.root私有，唯一resolve/canonicalize_deepest产生规范现存前缀及普通file_name尾段；acquire/open-existing用该Home；ExternalReadFile.open_regular先canonicalize parent；no-follow与ExternalReadTree走canonical_external_root/lexical_abs；当前直接测试同样用规范Home加受限RelPath。traverse仅由open_root/create_root调用，当前完整caller不提供literal `.`/`..` 段，所以两个谓词恒false，OR/AND在此闭包观察相同。

此证明不覆盖未来raw internal AbsPath caller、任意直接私有调用或新增未规范test，不记动态caught。不能套到晚stat、同FD nlink、schema等待等其它26条件。

映射26唯一，动态25唯一与静态1无交集、union完全相同，无extra/missing；全部old_id与归档e54dcd41的原G03缺项26完全一致。三批190份source/fixture/skill/config映射相同且当前SHA无漂移；最终工程共同输入逐项同SHA。静态关键producer源码已实际只读核，不把当前范例值当通用type保证。

## 固定二进制的最终工程

原unfrozen run 875/880、5 exporter setup Add失败、stdout/stderr空完整保留，根因**unknown**。读取现有export fixture可确认其默认每进程Cargo build同shared binary target，但不是该5失败的已证明因果；12并发Add probe与6 exporter focused PASS也不改原失败结论。

使用既有SHELTIE_TEST_ENGINE_BINARY/EXPORT_BINARY入口，单次真实all-features/locked build后固定2份copy，不改repo fixture/生产：engine SHA256 `bbe300ff76afef2ba849bdfd760345949f0f52bb4417ca4fb2009e4ae08502ae`，exporter `5eae70f2a9c4ba145907ad736fd10a61511cd23ce842af864952fe821efe4967`。独立当前hash与build copy/gate记录一致，最终env明确选择这两份；export fixture已有入口因此不在测试中再并发构建它们。恢复证明限定新稳定执行环境。

新final fmt/check/clippy/nextest/doctest全部exit0，五份原输出SHA与记录相符，source_unchanged=true。Nextest run `0fa1bc4a-c025-4ad2-82f9-3944a09d58fe`：880/880、0skip/LEAK、2slow，test144.115s（整命令144.608s）；core5 compile-fail通过，其它crate0 doc不补覆盖。治理原文docs/specs/core-vocab OK、check-tests880/204；后续文档/范围/done/提交检查由执行者收尾。

可以在当前范围关闭T50/G03证据义务；不批准旧失败、旧candidate、其它SK02组或M2，不将纯分类器、诊断检测及当前caller静态范围宣传为全产品安全/平台资格。
