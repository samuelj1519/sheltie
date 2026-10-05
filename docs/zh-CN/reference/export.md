# 最终成果读取与导出参考

[English](../../en/reference/export.md) | 简体中文

本页查询 `work result` 的原字节模式和外围 `sheltie-export` 的参数、结果及限额。操作步骤见[导出成果](../how-to/export-results.md)，精确语义见[协议 §8](../../../specs/contracts/protocol.md#8-original-final-artifact-bytes-and-external-export)。工具未正式发布，支持范围见[限制](limitations.md)。

## 原字节读取

```text
sheltie --home <root> work result <work> --artifact <key> --revision <positive-integer>
```

`artifact` 与 `revision` 必须一起给出。使用同一次成果查询的完整 Work ID、revision 和 literal key；key 不必符合 ID 规则。以 `--artifact=<key>` 传值可避免以连字符开头的 key 被当作选项。

raw 模式不接受 `--json` 或 `--request-id`，stdout 只写原 bytes，不补换行。资格包括 revision 精确相同、`final=true`、效果完成、key 在明确选集中；随后核源文件的对象身份、实际 bytes 和 sha256。读取中途失败可能已有部分 stdout，不能只依据目标文件存在认定成功。

## `sheltie-export` 参数

```text
sheltie-export --sheltie <binary> --home <root> --work <full-work-id> --to <parent> [--json]
```

| 参数 | 要求 |
| --- | --- |
| `--sheltie` | 可信、可执行的绝对二进制路径；不从 PATH 搜索 |
| `--home` | 已有的真实绝对管理根，无符号链接 |
| `--work` | 完整 Work ID，不接受前缀 |
| `--to` | 已有、真实、明确授权的绝对父目录；与 Home 不能相互包含 |
| `--json` | 输出一行 `work-export/v1`；省略时输出人读文本 |

源结果必须为 succeeded、`final=true`、`effects_pending=false` 且选集非空。工具通过 CLI 查询与读取，不打开 Store，也不重新挑选“最新文件”。

## 文件与格式

新副本目录为 `<work-id>-<random>`，不覆盖或合并已有对象。目录权限 0700，文件权限 0600；成果按 key 顺序映射到 `artifacts/0001/...` 等私有索引目录，key 不直接拼成路径。

`manifest.json` 使用 `work-export-manifest/v1`，保存完整源结果和 `files` 映射。每项包含 `key`、副本相对 `path`、`sha256`、`bytes`。副本可以编辑；编辑后的字节不再由原清单保证，源 Work 不变。

| 对象 | 上限 |
| --- | --- |
| 结果 JSON | 1 MiB |
| 单个成果文件 | 32 MiB |
| 全部成果字节 | 256 MiB |

工具核源退出码、实际大小与摘要，独立读回全体文件，完成规定的 OS 同步后整目录发布。该保证不等于物理断电持久或同一 OS 账户下的隔离。

## 响应与退出状态

JSON 含 `format`、`status`、`work_id`、`revision`、`target_path`、`staging_path`、`error`；不能确认或不适用的值为 `null`。它不是引擎的 `ok/data/next` 响应。

| `status` | 退出码 | 已确认边界 |
| --- | --- | --- |
| `complete` | `0` | 字节核验、整目录不覆盖发布和规定 OS 同步完成；副本位置取 `target_path` |
| `rejected` | `2` | 参数或确定性资格拒绝 |
| `failed_before_publish` | `1` | 发布前的源、目标、完整性或 I/O 失败；暂存不能当完成副本 |
| `publication_unconfirmed` | `3` | 移动已发生，但最终身份或父目录同步未确认；保留现场，不自动删除或回滚 |

进程被终止时可能没有响应。报告中的路径只指能核所属的对象；不能按名字认领其他目录。重跑始终建立新副本，不接管旧暂存，异常处理见[导出指南](../how-to/export-results.md#失败现场)。
