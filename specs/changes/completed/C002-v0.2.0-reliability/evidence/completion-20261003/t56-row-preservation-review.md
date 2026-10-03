# C002-T56 F-T56-01 修后增量复核

**源码修补通过，F-T56-01已解决。** 原需修改报告保留；Reviewer只读未写代码/oracle或跑cargo。修后SHA及当前baseline状态见JSON；编译Result alias fixture错误原文保留，不当产品红。

original_tree独立使用真实symlink_metadata、relative完整keys/dev/ino/mode（含type）/nlink及filebytes或raw link目标，只有真实dir递归，不跟随symlink。缺pending root的真实保存树在cleanup之前捕获，之后全等；不再只靠saved存在。timestamps/uid不是此snapshot的断言字段，不扩大。

persistent_rows逐workbooks/works/work_sequence/requests/audit读取全部columns，以rowid排序保存原始rusqlite Value，未借Store getter补做connect/cleanup或调用生产helper生成答案。实际pending移出后、cleanup前捕获，cleanup后直接查询比较五表；表内JSON/Text字节值保持，不称整个SQLite/WAL/SHM/隐藏控制表字节不写。before快照跨多独立SQL读取，但当前test没有并发writer，所以该scope有稳定值。

warning全部归实际unpublished-start且不含qualified-add关系保留，root内pending仍absent；对该missing-root分支证明原件与行零变化，不给其它合法cleanup放开权限路径发明mode零改。源码没有生产变化，此增量只加强独立保全oracle。

修后focused11仍按actual终态登记，未完成时不称PASS；最终15项mutants/工程尚未运行，不关闭T56/M2。

修后focused真实11/11通过，run `d5797885-b6f3-44dd-810f-e56524ff1aaf`，正确原文SHA见JSON。当前两个静态proof的current inventory/function/span/genre/replacement/diff及新sourceSHA已增量绑定；最终mutation/gates尚未审。
