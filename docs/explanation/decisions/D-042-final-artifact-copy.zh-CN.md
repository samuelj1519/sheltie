# D-042：最终成果原字节与外围副本

[English](D-042-final-artifact-copy.md) | 简体中文

状态：`accepted`
日期：2026-10-03
关联 change：[C006](../../history/changes/C006-result-delivery/README.zh-CN.md)

## 选择与理由

`work result` 是唯一成果选择来源。引擎以同一已核结果快照绑定 revision 与 key，用受限的同一文件句柄读取。stdout 为原 bytes，诊断在 stderr，不写宿主或回执。

外围 `sheltie-export` 只通过公开 CLI，不依赖 runtime，不打开 Store。它的 Cargo 元数据为 `publish=false`、`dist=false`，外部分发另定；副本不成为第二结果来源。

Home 与目标父目录必须为显式、真实、无符号链接的绝对目录，其对象和祖先不能相互包含。结果 metadata 限 1 MiB，单文件限 32 MiB，全部成果限 256 MiB。全部文件接收后，独立核大小、摘要、源 child 退出和读回内容；私有目录与清单完成规定 OS 同步后，以 NOREPLACE 整目录发布。

key 使用原槽字符串，可以为空或 Unicode，不重新作为 ID 校验；argv 无法表达 NUL 时准确拒绝。目标路径只使用源的安全文件名和排序索引，在受限父目录句柄内创建，不接受 metadata 指定任意目标路径。

## 失败与保证边界

已有对象不覆盖。移动后确认失败报告 `publication_unconfirmed`，不删除或回滚；工具保留能证明属于本次的现场。重跑建立独立新副本，不接管旧暂存，不增加第二生命周期、续传或缓存。

`complete` 不承诺物理断电持久、同权限隔离或用户编辑后的持续一致。参数、清单与退出状态见[导出参考](../../reference/export.zh-CN.md)，处理步骤见[导出指南](../../how-to/export-results.zh-CN.md)。

## 确认方式

高级原语与独立判定依据在 T01 完成，并执行实际平台测试，由 M1 独立核验；T02 才开放 raw/export 入口。耦合基础可以由代码作者完成，但不能省略独立审查、有效的 feature 行为失败验证或准确失败边界。历史原件见关联 change 的固定快照。
