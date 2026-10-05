# D-037 受管文件使用目录句柄和安全 API

状态：`accepted`
日期：`2026-09-28`
关联 change：[C002](../../history/changes/C002-v0.2.0-reliability/README.md)

## 背景

M1发现`..`词法路径逃逸、封存按路径chmod、摘要枚举后重新按路径打开文件；self与恢复caller也有绕过fsx的操作。它们可让受管操作作用到另一对象。工程规范禁止unsafe，要求路径由构造时校验的类型约束。

## 选择

runtime以已打开管理根/子目录句柄作为根内路径锚点。`ManagedRelPath`拒绝空、绝对、`.`、`..`、空段、NUL。目录段以NOFOLLOW逐段打开；叶文件先fstatat拒绝已知特殊对象，再以NOFOLLOW/NONBLOCK/CLOEXEC打开，之后fstat核身份、类型和nlink。T19固定具体模块接口，不扩展core，不加通用filesystem trait，不增加持久身份字段。

直接依赖rustix `=1.1.4`，启用`fs`，使用安全`openat`、Dir遍历、目录相对rename/unlink、fchmod和fsync。NOREPLACE平台映射经rustix源码核查，macOS arm64由T18探针实测；Linux运行经用户豁免，仍为`not_run`。Sheltie源码仍`#![forbid(unsafe_code)]`，不直接调用libc。

输出观察、摘要和正常submit封存复用同一个打开的SafeFile；COMMIT后封存前重验该句柄的bytes/nlink等于ArtifactRef，再对同对象chmod/sync，封存后核原路径仍指同一对象。恢复从登记引用重开并核字节。外部显式`@file`是只读入口，不提供受管写/删/chmod操作。

本机制不声称阻止同OS账户任意移动整个目录，也不把readonly位描述为用户隔离。跨挂载点、FIFO、链接及sync失败都准确报错；不能退回按路径重开、忽略错误或使用unsafe。

## 后果

Home/fsx与真实caller共同落实受管路径。CLI只解析路径，写Store操作受HomeLock约束。T18 API探针和T19–T23 caller门禁未通过前，依赖和接口不算验证通过。

## 确认方式

核对rustix 1.1.4文档/vendor源码；T18 macOS API/FIFO/sync探针已运行。Linux运行由用户豁免，状态`not_run`，不代表Linux验证通过。T19–T23用临时Home、单条件链接/对象替换与哨兵验证macOS真实caller。
