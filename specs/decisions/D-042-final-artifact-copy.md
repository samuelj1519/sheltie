# D-042 最终成果原字节与外围副本

状态：`accepted`
日期：2026-10-03
关联 change：[C006](../changes/active/C006-result-delivery/README.md)

C004结果是唯一选择来源。引擎同一已核结果快照绑定revision/key，受限同FD读取，stdout原bytes、诊断stderr，不写宿主或回执。外围sheltie-export只通过公开CLI，不依赖runtime，不开Store；publish=false/dist=false，外部分发另定。

显式真实无链接绝对home/to，父目录与管理根真实对象/祖先不重叠。metadata1MiB、单file32MiB、total256MiB；全部接收、独立摘要/大小/childexit与读回，私有目录及清单按规定OS sync后整目录NOREPLACE。已有对象不覆盖；移动后确认失败报告unconfirmed且不删除回滚。complete不承诺断电物理持久、同权限隔离或用户编辑后的持续一致。

key沿原槽字符串（含空/Unicode），不重新当ID；argv无法表达NUL时准确拒绝。路径仅取源安全leaf+排序索引，在受限父FD内创建；不让metadata给任意目标路径。工具失败保留自有可证明现场，重跑独立新副本，不建第二生命周期/续传/缓存。

高级原语和严格oracle在T01实现并实际平台测试，M1独立核；T02才开放raw/export入口。代码作者可完成必要耦合基础，但不跳过独立review、有效feature red或精确失败边界。
