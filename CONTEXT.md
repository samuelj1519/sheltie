# Sheltie 领域词汇

全仓库通用语言。一个东西只有一个名字；文档与代码都用这里的词。

## 命名

| 名称 | 含义 |
| --- | --- |
| Sheltie / `sheltie` | 产品名、仓库名、二进制名 |
| `sheltie-*` | 只用于 Cargo crate 名（`sheltie-core`、`sheltie-runtime`、`sheltie-cli`） |
| `workbook/v1`、`flow/v1`、`cli-result/v1` | 格式版本串，不加产品前缀 |
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
| Occurrence | 节点在一次 Work 里第 n 次被到达，写作 `node#n` |
| Attempt | 一个 Occurrence 内的一次执行尝试，写作 `node#n.retry`。只有执行事实：`running / succeeded / failed` |
| 任务书（brief） | 引擎为一次 Attempt 生成的文件：说明书原文加绑定好的输入路径与输出要求 |
| 状态卡（status card） | 引擎生成的紧凑进度视图；只有指针，没有历史正文 |
| 合法下一步（`next`） | 引擎算出的、协调者当前可以调用的操作集合 |
| 门槛（gate） | 节点属性。为真时 Attempt 成功后要真人批准才能离开 |
| 档位（tier） | 节点属性，`strong` 或 `standard`。给协调者选模型的标签，引擎不据此做任何事 |
| 产物（artifact） | Attempt 的输出文件，提交后按 sha256 冻结 |
| 参考文件（resource） | Workbook 内 `resources/` 下的文件，用 `resource.<path>` 绑成节点输入。随版本冻结，不装进宿主 |
| 宿主资源（host resource） | 必须装在宿主里才能用的 skill、命名 subagent、MCP。Workbook 在 `requires` 里按 `kind + name` 声明，引擎只列出不安装 |
| 冻结副本 | `work start` 时复制到 `works/<work_id>/workbook/` 的 Workbook 目录。本 Work 之后只读它 |
| 协调者（coordinator） | 读 Workbook、派活、理解回复、在 `next` 里选路的 agent 或人 |
| 工作 agent（worker） | 按一份任务书把输入变成输出的 agent 或人 |
| 引擎 | `sheltie` 二进制。记状态、发任务书、算 `next`、守门槛。不判断内容。`self` 命令组管它自己，`workbook` 命令组管方法，都只写 `~/.sheltie` |
| Package | 只指 Cargo 包或外部依赖。不是业务实体 |

## 仓库现状

代码是绿场脚手架。目标与进度见 [specs/README.md](specs/README.md)。文档描述目标，不表示已实现；进度只看 [specs/plan.md](specs/plan.md) 的状态列。
