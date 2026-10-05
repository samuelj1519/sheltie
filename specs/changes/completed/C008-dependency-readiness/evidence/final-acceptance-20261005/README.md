# 当前范围最终实跑验收

范围为用户确认的macOS aarch64／APFS本地场景及已指定完成的C011、翻译、DSH三项样本，依据[产品规格](../../../../../spec.md#当前产品环境)与[D-044](../../../../../decisions/D-044-macos-apfs-product-scope.md)。本次在冻结的232个产品／测试／构建／工具输入上实际运行验收，执行后无漂移。完整标准与身份见[计划](plan.json)、[源输入](source-inputs.json)和[验收矩阵](acceptance-matrix.json)。

## 本次实际执行

| 范围 | 结果 | 资格原件 |
| --- | --- | --- |
| 格式、全目标编译、Clippy严格检查 | PASS | fmt01/check01/clippy01的JSON及stdout/stderr |
| Rust全量nextest | 948/948，0失败、0跳过、无LEAK；1慢例72.138秒正常通过 | [实跑资格](executed-test-qualification.json)，run32976d61-a53c-4a04-8c79-a90da816cb26 |
| doctest | 5/5通过，包含compile-fail检查 | doctest01.json及原流 |
| Rust1.85兼容编译 | 全targets/features通过 | msrvcheck01.json；此行是编译，不冒本轮MSRV全测试 |
| 漏洞／依赖许可／来源／重复策略 | 四类检查通过，策略允许的warning保留 | deny02.json；deny01沙盒缓存锁失败原件保留 |
| Workbook编辑器真实CLI／HTTP／ZIP与模型保护 | 72/72，0失败／跳过／取消 | editor01.json；使用Cargo JSON取到并固定的新引擎字节 |
| 文档／规格／测试声明／skill／核心词汇／diff | 全部适用治理门禁PASS | 各命令JSON与原流；测试声明计数不冒Rust执行 |

[固定nextest资格](nextest-qualification.json)证明0.9.145官方资产digest及解包binary一致；PATH默认0.9.140不足，未采用，也未放宽最低版本。Node两依赖按锁文件离线ci，禁生命周期脚本。真实Cargo JSON可执行路径与固定binary SHA见[fresh-binaries.json](fresh-binaries.json)和[fixed-fresh-binaries.json](fixed-fresh-binaries.json)。两辅助记录脚本原字节保存为.txt，不作为产品代码或测试答案。

## 实物及实际接受

[独立实物报告](artifact-independent-readback.md)与[完整读回](artifact-independent-readback.json)核三个终点15原件、C006三副本和第四clean副本/旧审计注记、翻译73目标及145原件/5备份、DSH29源/tree与App582/target619；均按实际字节和摘要读取。原C011真人接受与同Attempt宿主重开、DSH接受、H01三指南接受仍对应相同字节；不由命令退出0替人接受，不重复已完成的模型调用。译文原独立语义审查按同内容资格引用，本次placement字节复核不冒重新全73语义审阅。

首实物检查器误拒声明中的3个LICENSE symlink，已按冻结的link目标/目标bytes/mode修正，其首draft完整保留于check_history；未改任何源。nextest run ID首解析假设也保留原草稿，按同一原流正确提取，没有为解析问题重跑测试。

当前必需验收矩阵全PASS；最终独立总判据审阅在本目录另记，输出后再确认本轮收尾。E01/E02依用户范围不要求，不改历史未执行为PASS；真撤销/依赖条件保留未来触发入口。历史Native、公平、盲审、费用和15%收益未知原件保持。没有发布、推送、合并或替换用户安装。

## 最终判定

[独立总验收](final-review.md)已给PASS，当前范围没有待修或阻断项。948/948 Rust、5/5 doctest与72/72编辑器测试为本次实际执行，源与交付物闭包保持。最终状态和原件摘要见 [最终验收封存](final-acceptance-closure.json)；报告加入后的文档门禁另存，旧计数不改写。
