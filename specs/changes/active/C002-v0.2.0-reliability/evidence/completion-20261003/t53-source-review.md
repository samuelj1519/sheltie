# C002-T53 源码/oracle 修后增量审查

**PASS，限定当前7项oracle和1个观察点的准备。** F-T53-01已解决；首次需修改原文保留在 `t53-first-source-review.md/json`，不反写。Reviewer `/root/oracle_review` 只读且未准备G06方向/写测试。当前source SHA及原文baseline见同名JSON；`t53-copy-control-baseline.txt`实际7/7 PASS、391过滤。

同fixture增长前先真实copy到独立legal-copy，返回33554432；目标完整len、同dev新inode、nlink1、0600均核，全部字节经流式SHA比较手写固定32MiB zero常量。Reviewer以Python hashlib逐块独立核常量`83ee47245398adee79bd9c0a8bc57b821e92aba10f5f9ade8a5d1fae4d8c4302`。随后另dst在最终opened核验后增长1字节，拒绝前data不创建。正负例现在覆盖完整同API cap接受，而非仅到观察点。

2个stream测试已实际读：declared来自真实File旧metadata4，随后append+；外部Read wrapper首次只委托真实File，第二次仅注入外部Io错误。合法首读→下一Read保持准确Io path/source，已知成长首读→InvalidRequest/frame不符。没有伪造成功、复制生产OR条件、改private状态或用最终is_err替代错误来源，确实能区分晚拒绝被外部Io覆盖。依据仍限定为真实长度frame的具体拒绝/停止机制；高层规格不因此新增一般IO错误优先序或性能预算。变体继续到下一Read并报Io属于诊断/优先序检测，不是最终Workbook接受或成功hash差异。

合法stream正控制以实际完整bytes的标准SHA和std UTF8给答案，覆盖普通ASCII、完整/残尾中文、跨64KiB中文及invalid前缀。已有hasher加任意独立prefix再比较标准SHA(prefix||bytes)，只证明stream追加字节及既有hash前缀完整性，不声称此测试独立证明BE64整Workbook帧编码。全文件源bytes保留；没有公开API或生产更改于workbook_digest。

原5个FS oracle/观察窗口/清理结论保留：opened最终fstat通过后、正文Read前；scope/release实际事件后释放join；marker同步错误先disarm再assert；copy可留下自有dst目录但不得创建被截断data，不扩为业务提交。UTF8当前caller分类限定静态代数仍成立，保留pending内存/累计复制差异，不称资源或全行为等价。

后续simplifier若改source，需绑定新SHA增量复核。最终8项准确集合、7动态/1静态实际账本、完整工程/MSRV/治理均未在本准备报告批准，保持未审/not_run；不关闭G06或C002-M2。

整理后增量复核通过：新module前fsx全bytes不变，workbook_digest整文件不变；4处身份tuple和SHA循环只提取直接stdlib/helper，File仍在原testscope存活。更新SHA见JSON，未改变断言或期望。
