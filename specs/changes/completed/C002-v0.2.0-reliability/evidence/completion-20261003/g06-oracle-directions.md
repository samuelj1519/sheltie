G06 准确8个旧→当前ID的只读独立准备。未写oracle/源码、未执行、不授动态PASS；仅macOS aarch64。精确ID与原/current diff SHA、冻结源SHA见同名JSON。五组优先复用真实public消费者和现有握手，不引入新平台/泛化framework。

| 组 | 数量 | 真实consumer与独立判据 |
| --- | ---: | --- |
| T1 最终树清单 | 2 | WorkbookRepo add/copy/load/digest→inspect_tree_v2/copy_tree_confined→ExternalReadTree.validate_unchanged。第一次捕获完整文件/目录集合，最后独立枚举必须仍相同；变化不得返回合格摘要/副本。 |
| T2 opened身份/长度 | 2 | ExternalReadTree.open_file供read_file、digest stream和copy读取。条目由真实树枚举产生；stat和open后的FD必须各自核device/inode/length，不让同长度换inode或同inode变长度放行。 |
| T3 extra-byte限额 | 2 | load_tree的manifest/flow/instruction read_file；add/start的copy_tree_confined。在最终fstat通过后，同FD源变大越过cap，必须发现额外字节，不能Take(cap)后正好等于旧declared就静默接受prefix。 |
| T4 流式frame长度 | 1 | inspect_tree_v2→stream_file_into；declared来自tree.files并先validate_sizes。源增长/缩短对frame/全局限额、早期停止、错误定位分开核；成功摘要须独立手写帧/标准SHA。 |
| T5 UTF8事实 | 1 | stream_file_into→Utf8State→ResourceMeta.is_utf8；captured文件另用std::str::from_utf8。合法跨chunk、明确错误续字节和EOF残尾，期望来自独立标准UTF8解码及原bytes。 |

T1的两个集合不是逐leaf open的重复guard。共享两个negative即可：捕获树后仅在root添加一个普通文件（directory identity map不变，files不同）；另一个仅添加空目录（directories不同、files相同）。稳定树为合法对照。AND变体会漏单侧差异，Ok变体会漏两者。适合在最后一个实际leaf已读取后、validate_unchanged前的边界改变，或真实tree API捕获后改变再验证。现有external_tree_after_stat处过早/只针对一个leaf，不能自动当末扫描窗口；若早改被其它guard挡住，只计那个早期检测。原件与所有业务SQL行保持，复制失败允许自己的私有控制暂存，不能新增最终Workbook/request/audit。

T2可直接用existing external_tree_after_stat。一例同inode只增加一个字节，一例移走原A保留、用同长度普通单链接B替换，要求同filesystem、dev相同而inode不同；不需要inode复用或跨设备。捕获entry、前stat与opened fstat是不同观察，不由旧值支配。原open_file应在任何正文读取前拒绝；如果变体仍在read-length或最后rescan拒绝，记录准确后层检测，不称opened guard被测。后扫描可能看到另一次事实，源被真实恢复时不能假设它永远补救第一次错误打开。root、原A、替代B bytes/dev/ino/nlink和是否向业务copy写入均独立核。

T3重点是**最后fstat通过之后**的增长。若在external_tree_after_stat就grow，前层opened长度guard先拒，摸不到+1→*1。当前open_file没有该“完整opened检查之后/Read之前”点；必要时只补准确feature观察点或直接真实API读窗口。cap生产为32MiB，可用sparse/独立手写文件避免重复写巨量数据；恰好cap合法、cap+1拒绝，源在读前后增长单列。read_file本体不会立即validate_unchanged；copy结束才validate。一种prefix误接受可能被最后清单拒绝，仍非限额检查同值证明；要声称整个copy检测，须分清是否真检测额外字节，还是最终长度漂移。Source后恢复的行为不能凭方法名字推断捕获。适用独立期望取原bytes/size、额外1byte与caller准确停止，不把被测take表达式放进expected。

T4有一个**有限静态成功集合/全局上限证明**，不能称全观察等价：唯一实际caller先tree.validate_sizes，故declared<=MAX_FILE_BYTES。actual用checked_add且单调；actual>MAX时也必>declared，所以OR/AND都会在全局越界拒绝；成功必须EOF且actual==declared，故此前从未actual>declared，成功hash/UTF8路径不变。若actual先>declared但未>MAX，变体继续更多读取/hasher更新，最后EOF mismatched或全局越界才拒；诊断、I/O错误优先序和成本不同。caller对任何Err立即退出，不finalize/返回污染的hasher或ResourceIndex。只能给这个确切consumer边界的静态候选，不能扩大到“增长当场停止”或资源成本等价。

动态若核T4早期停止，应使用真实增长文件/有界读取边界，或Read外部边界实际第二次失败的独立输入；不要在fake里直接写成功。仅断言最终is_err会让该变体存活，错误消息/读取次数的期望必须指回已采用的frame/停止义务，不能为mutant私有分支镜像一组断言。

T5有一个**最终UTF8分类的完整代数证明**。error_len=None表示合法prefix但尾部codepoint未完整；两实现同样保存尾部，后续续字节可正确完成。error_len=Some表示已有确定的无效字节/续字节，增加suffix不能使该prefix变合法。原实现valid=false后不再解析；变体把无效suffix留在pending，之后每次仍无效且pending非空，最终is_valid=`valid&&pending.empty`依旧false。合法/截断/错误编码及每个chunk后可消费的is_valid判断相同，完整资源UTF8事实不变。

但T5**不是内存/时延等价**：原pending只存不完整UTF8尾部（至多3bytes），明确invalid后停止；变体从首次invalid后可能保留直到file上限的大suffix，并在每次push搬移/复制累计prefix，产生额外内存与近似二次工作。流式读取仍有32MiB输入上限，但不意味着成本相同。不要只用原`.valid`私有字段或镜像branch断言人为杀它；若要资源oracle，先明确实际采用预算/consumer，再用真实binary资源与独立资源观测。无该预算时，可以准确记录分类限定静态证明，同时保留资源差异，不补造全behavior PASS。

已捕获源码/manifest/flow/说明文件的UTF8使用std直接判定，流式classifier只影响未captured数据；不得把两个producer混合成一个自算expected。摘要期望沿已有独立disk vectors、手写域前缀/BE64文件数/路径与内容帧；不调用digest_tree_v2得到自己的答案。

每组先非零baseline、合法对照和少量实际red，再按8个精确ID扩大；记录原run/source/feature/观察点。未到窗口、过早guard、Source hash漂移、timeout/截断或仅有限静态证明都按原范围单列，不改caught、等价或完整产品验收。
