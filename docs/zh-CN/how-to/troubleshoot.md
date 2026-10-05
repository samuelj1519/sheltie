# 排查命令错误与运行受阻

[English](../../en/how-to/troubleshoot.md) | 简体中文

用于已出现非零退出、`ok=false`、待恢复效果或流程受阻的情况。先确定真实二进制、管理根与目标，再按错误分流。完整错误码见[协议 §7](../../../specs/contracts/protocol.md#7-closed-error-code-set)；本页给出检查顺序，不代替合同。

## 1. 保存失败现场

保存实际命令、工作目录、二进制路径和版本、显式 Home、stdout、stderr、退出码，以及已有 request ID、Work ID、Attempt ID。只记录与本次调用有关的环境变量，避免把凭据写入报告。

响应完整时读取 `error.code`、`error.detail`、`committed` 和请求身份；响应不完整时记 unknown，不能由超时或 stderr 文案判断“尚未提交”。业务写调用有原 UUID 与原意图时，可以同 ID 重发核实；缺原记录时停止，不能新建 ID 猜测补做。

## 2. 只读核版本、根与当前状态

以已有的可信 `engine_binary` 和实际 `management_root` 查询：

```bash
"$engine_binary" --home "$management_root" --json self version
"$engine_binary" --home "$management_root" --json work list
```

若失败的是具体 Work，确认完整 `work_id` 后：

```bash
"$engine_binary" --home "$management_root" --json work status "$work_id"
```

核版本、Store 格式、Home、revision、pending 标志、resume 与当前 `next`。新根没有 Store 时 list 的 `NOT_FOUND` 不代表旧 Work 被删除；先核调用是否选错根。只读命令不带 request ID，也不恢复文件效果。

## 3. 按错误处理

| 观察到的错误 | 检查与处理 |
| --- | --- |
| `INVALID_REQUEST`、`INPUT_MISSING` | 核命令帮助、Flow 的 `start_inputs`、多给的键、UUID 和参数组合。`@file` 核 UTF-8、大小、安全普通单链接文件及实际路径 |
| `NOT_FOUND` | 核完整 ID、方法版本、Flow 和实际管理根；不以重开 Work 代替寻找已有对象 |
| `WORKBOOK_INVALID`、`FLOW_INVALID` | 按 `detail.path/rule` 修作者副本，重新装入；不修改已装或冻结目录 |
| `WORKBOOK_EXISTS` | 核已装身份；内容要改就用新版本，不删除同版本来绕过冻结 |
| `WORKBOOK_IN_USE` | 查询 `detail.works` 中的非终态运行；完成或经授权取消后再移除 |
| `WORKBOOK_TAMPERED`、`ARTIFACT_MODIFIED` | 保存准确对象、摘要与原件，人工核查；不把当前 bytes 改写成记录中的期望来取得通过 |
| `OUTPUT_MISSING`、`OUTPUT_TOO_LARGE` | Attempt 仍 running；由原执行者在声明路径补齐或缩小输出，再核合同后提交 |
| `SUMMARY_TOO_LONG` | 缩短至 4096 字节内，完整报告放声明输出；不能用超大 `@file` 绕过文本限额 |
| `ILLEGAL_NEXT`、`ATTEMPT_NOT_RUNNING` | 重新查询 status，按当前身份与 `next` 判断；不要重用旧写快照推进 |
| `REPLACEMENTS_EXHAUSTED` | 每 Occurrence 的一次撤销资格已使用；当前 Attempt 资格仍保留，按当前 next 处理，不增加额度 |
| `WORK_TERMINAL` | 停止写入该 Work，仍可查询；新目标另开 Work，不复活终态 |
| `REQUEST_CONFLICT` | 对照原 ID 对应的目标与参数。恢复原请求保持原意图；确为独立新操作才生成新 UUID |
| `REVISION_CONFLICT` | 重新查询当前状态，核并发操作，再决定合法下一步；不是无条件自动重试 |
| `EFFECT_PENDING` | 按 `committed` 与 `pending_request_id` 恢复登记的请求，见下一节 |
| `STORE_SCHEMA_MISMATCH` | 当前开发线拒绝 schema 1/2/3；保留旧根，旧二进制查旧记录，新格式使用新根 |
| `STORE_CORRUPT` | 保留数据库、业务文件、请求及准确错误，停止推进并人工核查；不手改状态或补造原件 |
| `UPDATE_UNAVAILABLE`、`UPDATE_CHECKSUM_MISMATCH` | 核版本、平台、发布来源和清单；不跳过摘要校验或把不匹配包当成功安装 |
| `IO` | 依 `detail.path` 核权限、磁盘与对象类型；先确定是否提交和归属，再处理，不直接删锁、WAL 或 pending |

## 4. 恢复未完成文件效果

`committed=true` 表示本次请求已提交，用原 ID、原目标和原用户参数重发同一命令。`committed=false` 且 `pending_request_id=A` 表示本次 B 未提交，先以真实记录恢复 A，再重试 B。`original` 与 `pending_original` 分属对应请求，不可混用。

准确步骤见[接续指南](resume-work.md#文件效果未完成与未知结果)。查询不会恢复效果，不能删除 pending、覆盖冲突对象或制造一次 Attempt 失败来消除提示。恢复后先查新的 status，重放里的 next 仍是历史值。

## 5. 判断流程受阻

| 状态 | 操作边界 |
| --- | --- |
| `blocked(gate)` | 将当前产出交用户；明确批准后才调用当前 next 的 gate approve |
| `blocked(retries_exhausted)`、`blocked(no_legal_edge)` | 停止派活，报告额度用尽；当前只剩取消选择，取消仍需实际授权 |
| running 且会话重开 | 先核旧执行者和共享工作区，按 resume 接续原 Attempt，不自动 fail 或 replace |
| 成功但结果集合为空 | 核终点是否声明 `result=true`；可能是合法成功，没有最终选集 |

导出异常另见[导出指南](export-results.md#失败现场)。作者工具检查失败时按错误定位草稿；刷新前先保存可保留资料。工具进程或 HTTP 错误见[作者工具参考](../reference/workbook-editor.md)。无法确认身份、原件或必要权限时保留 unknown，停止相关写操作并交人工核查。
