# 精确合同

简体中文 | [English](README.md)

本目录供方法作者、CLI 消费者和源码维护者查阅字段、格式、命令及拒绝条件。产品目标见[规格](../spec.zh-CN.md)，实现分层见[架构](../architecture.zh-CN.md)，统一术语见[CONTEXT](../../CONTEXT.zh-CN.md)。

| 合同 | 查什么 | 主要消费者 |
| --- | --- | --- |
| [Workbook](workbook.zh-CN.md) | manifest、Flow、节点、显式边、输入来源、输出与成果声明、编译规则 | 方法作者、core 解析／编译、作者工具 |
| [协议](protocol.zh-CN.md) | 命令、参数、JSON／文本响应、next、错误、raw 成果与导出 | 协调者、cli、skill、外围工具 |
| [存储](storage.zh-CN.md) | schema、请求身份、事务、冻结文件、效果发布／恢复、只读与 self 管理 | runtime、持久数据和恢复测试 |

当前源码使用 workbook/v1、flow/v1、cli-result/v4、work-result/v1、workbook-digest/v2 与 Store schema 4。开发线与已发布版本的区别见[文档地图](../README.zh-CN.md)；旧版本合同按发布 tag 读取。

修改合同前先定位产品依据，再沿全部真实入口和消费者核影响。字段、错误、持久格式与恢复时点一起更新；不可用默认值、静默兼容或修改测试期望掩盖合同缺口。
