# CLI 参考

[English](../../en/reference/cli.md) | 简体中文

本页按命令组查询当前源码的调用形式、主要结果和输入规则。完整返回字段、拒绝优先级与错误码由[协议合同](../../../specs/contracts/protocol.md)定义。实际操作见[操作指南](../how-to/README.md)，故障处理见[排查错误](../how-to/troubleshoot.md)。

## 调用形式与全局选项

```text
sheltie [--home <dir>] [--json] [--request-id <uuid>] <group> <verb> [args]
```

尖括号表示必须替换的参数，方括号表示可选部分，`...` 表示可重复。前三项是全局选项，也可在子命令后给出。

| 选项 | 行为 |
| --- | --- |
| `--home <dir>` | 指定管理根；优先于 `SHELTIE_HOME`，未设置时使用 `~/.sheltie` |
| `--json` | 普通命令向 stdout 输出一行 JSON；告警与诊断可独立写入 stderr |
| `--request-id <uuid>` | 仅用于 Workbook、Work、Attempt、gate 的业务写操作；省略时生成。重试保持原 ID 与原意图 |
| `--help`、`-h` | 查看命令帮助；每个命令组和子命令都有帮助 |
| `--version`、`-V` | 查看二进制版本；管理根、平台与 Store 格式用 `self version` 查询 |

只读命令和整个 `self` 组不接受 `--request-id`。引擎不读取 `USER` 或 `USERNAME` 来认证操作者；审计与批准中的主体来自实际 OS 进程账户。

## `self`：管理引擎

| 调用 | 主要行为或结果 |
| --- | --- |
| `self version` | 查询 `version`、`platform`、`home`、`schema_version`；不创建管理根 |
| `self install` | 将正在运行的二进制复制到管理根的 `bin/sheltie`，建立 Store；相同字节返回 `already_installed=true` |
| `self update [--version <version>]` | 下载并核验正式发布；省略版本取 latest，指定版本固定 tag `v<version>` |
| `self rollback` | 恢复 `bin/sheltie.prev`；只保留一级回滚，不降级 Store |
| `self uninstall [--purge] [--yes]` | 默认删除 `bin/` 并保留运行数据；确认 purge 后清理数据与二进制，保留根目录及原 `.lock` |

`--version` 的值写 `0.2.0`，不带 `v`。purge 的文本确认输入 `yes`；JSON 模式必须给 `--yes`。`SHELTIE_RELEASE_BASE` 可覆盖更新来源，默认是本项目 GitHub Releases；使用前核对实际来源。安装与更新不会修改 shell 配置或安装协调者 skill。步骤见[管理安装](../how-to/manage-installation.md)。

## `workbook`：管理方法版本

| 调用 | 主要行为或结果 |
| --- | --- |
| `workbook add <dir>` | 校验并装入整个目录；同 ID、同版本不覆盖 |
| `workbook list` | 列出版本与名称，标记各 ID 的最高版本及待发布状态 |
| `workbook show <id>[@<version>]` | 查询各 Flow 的节点、边、`start_inputs` 与宿主资源声明 |
| `workbook verify [<id>@<version>]` | 重算目录摘要；省略时核所有版本，结果为 `ok`、`tampered` 或 `missing` |
| `workbook remove <id>@<version>` | 删除准确版本；被非终态 Work 引用时拒绝 |

省略方法版本时按已装版本字符串的字面排序取最高值，不进行语义化版本范围计算。可复现运行应明确指定版本。方法管理见[管理 Workbook](../how-to/manage-workbooks.md)。

## `work`：运行与查询

| 调用 | 主要行为或结果 |
| --- | --- |
| `work start --workbook <id>[@<version>] --flow <flow> [--name <name>] [--input <key=value>]...` | 冻结方法和起始输入，创建 Work；返回完整 `work_id`、`work_dir` 与 `next` |
| `work list` | 列出 Work 的 ID、名称、状态、当前节点与更新时间 |
| `work status <work>` | 当前状态、`revision`、`resume`、pending 标志和当前 `next` |
| `work stats <work>` | 节点到达、尝试、失败、撤销、耗时与边来源的事实统计 |
| `work result <work>` | 查询终点明确选择的冻结引用集合；不直接返回文件内容 |
| `work result <work> --artifact <key> --revision <revision>` | 原字节读取；两参数必须一起出现，不接受 `--json` 或 `--request-id` |
| `work cancel <work>` | 取消非终态 Work；不停止宿主进程或撤销外部副作用 |

`<work>` 接受完整 ID 或唯一前缀；多匹配时拒绝并列出候选。协调与导出使用完整 ID。Work ID 的日期和当日序号按 UTC 分配，不能按本地日期猜测。

起始输入必须与 `workbook show` 对应 Flow 的 `start_inputs` 完全匹配。`--input 'topic=介绍 Sheltie'` 传字面文本；`--input "topic=@/absolute/path/topic.md"` 读取文件内容。`--input project=/absolute/path/repo` 冻结的是路径文本，不复制整个仓库。文件源要求 UTF-8、普通文件、单链接、无符号链接，并遵守[存储读取限额](../../../specs/contracts/storage.md#53-source-directories-and-host-metadata)。

## `attempt` 与 `gate`：推进当前运行

| 调用 | 主要行为或结果 |
| --- | --- |
| `attempt begin <work> --node <node>` | 领取当前 `next` 允许的节点；返回 `attempt`、`brief_path`、`inputs`、`outputs` 与 `requires` |
| `attempt submit <work> --attempt <attempt> --summary <text或@file>` | 核声明输出并封存；返回输出引用、Work 状态与下一步 |
| `attempt fail <work> --attempt <attempt> --reason <text>` | 如实登记执行失败，按剩余额度开放重试或受阻 |
| `attempt replace <work> --attempt <attempt> --reason <text或@file>` | 撤销当前 running Attempt 的提交资格并领取同一 Occurrence 的新任务书 |
| `gate approve <work> --node <node>` | 在当前 `blocked(gate)` 处记录批准；批准后重算下一步 |

摘要与原因文本至多 4096 字节。`submit` 和 `replace` 支持 `@file`；`fail` 的原因按字面文本处理。输出路径、Attempt ID 和节点来自实际响应，不从目录或编号推测。接续、失败、内容返工与资格撤销的选择见[接续 Work](../how-to/resume-work.md)。

## 响应、重放与退出码

普通成功响应含 `ok=true`、`data` 和 `next`。业务写成功还含 `request_id`，Work 写成功含顶层 `revision`。查询的 revision 在相应 `data` 中；不适用字段省略。失败含 `ok=false` 和 `error.code/message/detail`，不能只解析自然语言消息。

同 ID、同意图的业务写重放返回提交时快照，并置 `data.replayed=true`。`next` 也属于历史快照，继续前重新查询 `work status`。同 ID、不同意图返回 `REQUEST_CONFLICT`。`EFFECT_PENDING` 的 `committed` 用于区分本次已提交与被旧效果阻断，处理见[恢复指南](../how-to/resume-work.md#文件效果未完成与未知结果)。

| 退出码 | 解释 |
| --- | --- |
| `0` | 命令成功；仍需检查业务结果，如 `final` 和 pending 标志 |
| `1` | 普通业务、文件或恢复失败；已发生的效果按结构化响应判断 |
| `2` | 参数解析或特定输入拒绝，如非法 `@file`、查询携带 request ID、raw 参数组合错误 |

`cli-result/v4` 是持久业务响应快照的格式，普通外层 JSON 不一定带 `format`。`work result` 的 `data.format` 为 `work-result/v1`。raw 模式 stdout 是原 bytes，不补换行；即使已输出部分 bytes，最终仍可能失败，消费前必须核退出码和完整性。

外围 `sheltie-export` 的 `work-export/v1`、状态和退出码独立于引擎，见[导出参考](export.md)。当前源码与正式发布的对应关系见[支持与兼容性](limitations.md)和[发布记录](releases/README.md)。
