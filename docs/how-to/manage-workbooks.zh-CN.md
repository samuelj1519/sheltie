# 装入、核验、升级与移除 Workbook

[English](manage-workbooks.md) | 简体中文

用于管理作者目录和已装方法版本。需要可信引擎与明确的管理根；新试用先[构建](build-from-source.zh-CN.md)，已有 Work 始终使用它原来的根。命令中的 `engine_binary`、`management_root` 和 `author_dir` 均为实际路径。

## 1. 装入完整作者目录

确认目录含 manifest 列出的全部 Flow、说明和参考文件，再执行：

```bash
"$engine_binary" --home "$management_root" --json workbook add "$author_dir"
"$engine_binary" --home "$management_root" --json workbook list
```

逐条核成功响应。装入会校验并复制完整目录，不执行脚本或调用模型。保存返回的 ID、版本和 digest。同版本已存在时返回 `WORKBOOK_EXISTS`，不会用新作者内容覆盖它。

业务写操作需安全重试时，调用前生成并保存 UUID，以 `--request-id` 传入；重试保持同一 ID 和原目录参数。效果未完成按[接续指南](resume-work.zh-CN.md#文件效果未完成与未知结果)恢复。

## 2. 查结构、输入与完整性

把 `workbook_spec` 设为实际 `<id>@<version>`，例如 `two-step-zh-cn@1.0.0`：

```bash
"$engine_binary" --home "$management_root" --json workbook show "$workbook_spec"
"$engine_binary" --home "$management_root" --json workbook verify "$workbook_spec"
```

show 中核所选 Flow 的 `start_inputs`、节点、边与 requires。运行 start 前提供全部起始键，确认实际宿主依赖；声明不等于已安装。verify 的结果必须为 `ok`；`tampered` 或 `missing` 时保留 `detail.results` 和原目录，人工核查，不同版本名不能替代损坏调查。

省略 verify 参数核所有版本。list 的最高版本按字面排序决定，运行可复现任务时给出明确版本。只读操作不会恢复 `pending_publish=true`；待发布原件可用于查询，但推进前应先恢复已登记写请求。

## 3. 升级方法

在作者副本中改内容，同时使用新的版本或新的 ID；重新 add、show、verify 后，用明确新版本启动新 Work。已有 Work 保留自己的冻结副本，不热切换到新方法，也不靠编辑 Store 延续新版本。

需要编写方法见[编写 Workbook](write-workbook.zh-CN.md)；画布修改见[作者工具](edit-workbook.zh-CN.md)。

## 4. 移除准确版本

先从 list 和已有 Work 状态确认对象。明确授权移除后，以完整 ID 与版本执行：

```bash
"$engine_binary" --home "$management_root" --json workbook remove "$workbook_spec"
"$engine_binary" --home "$management_root" --json workbook list
```

有非终态 Work 引用时返回 `WORKBOOK_IN_USE`，`detail.works` 列出对象。先完成这些 Work；确需取消时单独按授权取消，不能为了删除方法自动取消运行。终态 Work 的冻结副本不阻断 remove，也不随已装版本移除而删除。

任何非零退出或 `ok=false` 都停止后续动作，保存原命令、request ID、stdout、stderr 和退出码。错误分流见[排查错误](troubleshoot.zh-CN.md)。
