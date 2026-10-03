# C002-T53 UTF8 准确ID限定静态复核

**PASS，仅当前返回UTF8分类/元数据的限定静态证明。** Reviewer `/root/oracle_review` 未准备G06方向、实现或oracle，只读。准确旧/当前ID均为 `crates/sheltie-runtime/src/workbook_digest.rs:163:27: replace match guard error.error_len().is_none() with true in Utf8State::push`；`MatchArmGuard`、function/column/true替换已核current inventory。原与当前diff SHA均为`c81bed10917e6f73590d76df1877538ee3e37bc8835242bcf8de2c5d9a9d59d8`，原G06归属及原始生产prefix与current逐字一致。完整sourceSHA见JSON。

静态证明：在出现明确invalid之前，Ok与不完整尾部路径完全一致。标准Utf8Error的None只代表末尾未完成1–3字节；Some代表从valid_up_to开始已存在错误编码。此时原实现valid=false并停止classifier后续解析；变体保留从该错误起的非空invalid suffix。确定错误编码的前缀不能通过追加suffix修复，所以以后变体每次仍留非空invalid pending；原的valid=false与变体的pending非空都让is_valid=false。合法/EOF残尾/错误编码及每次push后的分类都相同。这里比较的是同一真实Read/chunk轨迹下正常完成的值，不把资源异常或不同调度假定相同。

完整当前生产caller已核：私有Utf8State只在stream_file_into中构建/push并以is_valid返回；inspect_tree_v2把该bool放入ResourceMeta。public digest_dir_v2/managed digest经digest_tree/inspect仅全成功返回hash；observe公开build_resource_index→from_tree使用空captured，返回完整资源分类；WorkbookRepo load_tree先捕获manifest/Flow/instruction，captured分支另用stdUTF8，未捕获文件才流式，之后compile消费ResourceIndex。core规则7只对instruction文件要求UTF8，resource编码不限。没有生产caller直接消费.valid或.pending，也没有classifier状态成为第二事实源。hash/actual length与UTF8标志独立更新，错误短路不返回合格index/digest。

**保留实质差异：** 原pending最多3个未完成尾字节，明确invalid后不再解析；变体可保留到file实际输入上限的invalid suffix，每chunk反复搬移/复制增长prefix，累计工作可近二次。stream在push前核实际file大小，能限制输入字节，不证明RAM峰值、耗时或交错相同。Read控制结构未改，但成本/实际并发时序、OOM/分配失败/timeout结果不在分类值证明里。没有采用具体资源预算，不发明阈值或断言私有.valid来人为杀该变体。

处分为 **static_scoped_classification_proved**，动态执行`not_run`，Caught=false，全行为/资源等价=false。原stage1 MissedMutant保留。这份证明可支持第8项按准确限定静态范围登记，不能记动态捕获或全部产品等价；其余7项最终动态与工程闭包另审。
