# 当前候选技术补验

源码候选：`100a6f845e619f95d2f112d2adc5637fdf00801f`。本次没有修改引擎、方法、测试或构建配置；只核对既有验收、执行最新候选的MSRV测试并准备真人验收材料。C010原采用任务和独审结论保持，不新增宿主、用户价值或发布通过声明。

## 原生结果的当前资格

[source-qualification.json](source-qualification.json) 重新核对C010原378项输入：369项内容/权限不变，9项为归档后的两份规格路由和7份旧active路径；编译/方法/测试输入不变。8份原记录的stdout/stderr解压字节数和SHA全部吻合。原生948测试引用原run `244c3674-e087-431d-a796-a63338ac7cd8`，归档治理另引用30测试run `97c47f41-e913-49ab-a5d0-87addeb49edb`，不称本次重新执行原生完整回归。

初次核对曾把许可证symlink文字同旧记录的跟随文件字节比较，得到三个伪漂移。读取旧算法后按其实际内容/stat规则核对均一致；新执行清单额外记录symlink literal。没有改动许可证或把初次诊断当产品失败。

## Rust 1.85实际执行

使用已安装的Rust/Cargo 1.85.0与临时合规nextest 0.9.145；PATH默认0.9.140未用于运行，没有override。空RUSTC_WRAPPER、明确可写target、all-features、locked、离线依赖。各命令有720秒超时，实际均未超时。

| Requirement / risk | Mode | Input closure | Command / raw run ID | Result | Evidence |
| --- | --- | --- | --- | --- | --- |
| 最新候选MSRV完整测试 | executed | 206项编译/方法/脚本/配置输入；source资格另核工作流/钩子，无漂移 | `cargo +1.85.0 nextest run --workspace --all-features --locked --no-tests=fail`；`ed11bd1a-42eb-4097-bb4d-c7bddd9ae864` | PASS：948/948，0skip，1slow，无LEAK；总墙钟158.5985秒 | [metadata及压缩原文](msrv-nextest.json) |
| MSRV公开类型约束 | executed | 同一206项输入，非默认空样本 | `cargo +1.85.0 test --workspace --doc --all-features --locked` | PASS：5compile-fail doctest；另外两个crate的0样本不增加数量 | [metadata及压缩原文](msrv-doctest.json) |
| 工具与外部消费者资格 | executed | 实际工具版本；工作流/钩子原摘要；原369项不变证据 | 实际rustc/cargo/nextest版本、文件读回 | PASS，仅实际输入资格 | [工具与消费者](tools-and-consumer-qualification.json)、[源资格](source-qualification.json) |

原C010只有MSRV编译检查，这次另补实际执行；不把后来测试倒填为原t02-msrv结果。原951测试来自较早候选，不作当前948测试的替代。费用、真人活动、真实关闭重开和历史LEAK因果未从测试耗时估算。

文档和规格的本次新增消费者另经实际治理检查，原件保存于C008本次交接目录；不是给原生运行改run ID。新真人试用环境见 [C007准备](../../../C007-pre-run-workbook-generation/evidence/human-acceptance-20261004/README.md)。
