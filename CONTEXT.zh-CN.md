# Sheltie 领域词汇

[English](CONTEXT.md) | 简体中文

全仓库通用语言。一个东西只有一个名字；文档与代码都用这里的词。

## 命名

| 名称 | 含义 |
| --- | --- |
| Sheltie / `sheltie` | 产品名、仓库名、二进制名 |
| `sheltie-*` | 只用于 Cargo crate 名（`sheltie-core`、`sheltie-runtime`、`sheltie-cli`、外围 `sheltie-export`） |
| `workbook/v1`、`flow/v1`、`cli-result/v4`、`work-result/v1` | 当前目标格式版本串，不加产品前缀；`cli-result/v1` / `cli-result/v2` 分别属于 v0.1.0 / v0.2.0 历史格式 |
| `workbook.toml` | Workbook manifest 文件名 |
| `SHELTIE_HOME` | 管理根环境变量，默认 `~/.sheltie` |

## 核心词

| 词 | 定义 |
| --- | --- |
| Workbook | 人写好并冻结的做事方法：一个目录，含 manifest、Flow 与说明书。业务方法只用这个词 |
| Flow | Workbook 内一张可运行的图：节点加显式边 |
| Node | 图上的一步。有说明书、输入、输出、执行者，可带门槛 |
| Edge | 两个节点间的显式有向边，类型 `main / back / branch / re_review` |
| Work | 某个 Workbook 版本的一次运行。`work_id` 形如 `2026-09-24-001-文章-初稿`：UTC 日期、当日序号、名字 |
| Occurrence | 节点在一次 Work 里第 n 次到达，写作 `node#n` |
| Attempt | 一个 Occurrence 内的一次执行尝试，写作 `node#n.number`。记录执行与行政撤销事实：`running / succeeded / failed / superseded` |
| 任务书（brief） | 引擎为一次 Attempt 生成的文件：说明书原文加绑定好的输入路径与输出要求 |
| 状态卡（status card） | 引擎生成的紧凑进度视图；只有指针，没有历史正文 |
| 合法下一步（`next`） | 引擎算出的、协调者当前可以调用的操作集合 |
| 门槛（gate） | 节点属性。为真时 Attempt 成功后要真人批准才能离开 |
| 档位（tier） | 节点属性，`strong` 或 `standard`。给协调者选模型的标签，引擎不据此做任何事 |
| 产物（artifact） | Attempt 的输出文件，提交后按 sha256 冻结 |
| 草稿（draft output） | running Attempt 的声明输出位置；路径不证明文件已存在或封存 |
| 最终成果（result） | 成功终点明确选择的冻结输入与封存输出；引用绑定到该具体终点 Attempt |
| 参考文件（resource） | Workbook 内 `resources/` 下的文件，用 `resource.<path>` 绑成节点输入。随版本冻结，不装进宿主 |
| 宿主资源（host resource） | 必须装在宿主里才能用的 skill、命名 subagent、MCP。Workbook 在 `requires` 里按 `kind + name` 声明，引擎只列出不安装 |
| 冻结副本 | `work start` 时复制到 `works/<work_id>/workbook/` 的 Workbook 目录。本 Work 之后只读它 |
| 协调者（coordinator） | 读 Workbook、派活、理解回复、在 `next` 里选路的 agent 或人 |
| 工作 agent（worker） | 按一份任务书把输入变成输出的 agent 或人 |
| 引擎 | `sheltie` 二进制。记状态、发任务书、算 `next`、守门槛。不判断内容。`self` 命令组管它自己，`workbook` 命令组管方法，都只写 `~/.sheltie` |
| 可编辑副本（export） | 外围工具核验明确最终成果后，在授权父目录发布的一份新目录；不写回引擎或成为第二结果来源 |
| Package | 只指 Cargo 包或外部依赖。不是业务实体 |

## 仓库现状

当前 release、active change、proposed change 与实施入口只在 [specs/README.md](specs/README.zh-CN.md) 定义。

Attempt 后缀 number 是同一 Occurrence 从 0 连续递增的创建顺序号，失败数只统计 failed，superseded 不消耗 max_retries。替换在一次写操作中撤销旧资格并开始新 Attempt，每个 Occurrence 固定最多一次；不停止旧进程或证明宿主隔离。
