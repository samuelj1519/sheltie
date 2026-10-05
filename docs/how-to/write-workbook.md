# 编写一份 Workbook

用于把稳定的阶段、输入输出与允许返工写成可运行方法。先明确目标、实际执行者、交付文件和停止条件；阶段内的调查与细分由执行者完成，不把每次临时动作都变成 Node。

需要一个完整的最小定义时，先完成[门槛与成果教程](../tutorials/gate-and-result.md)；本指南用于将自己的方法落到文件。字段与拒绝规则见[Workbook 参考](../reference/workbook.md)及[精确合同](../../specs/contracts/workbook.md)。

## 1. 建立独立作者目录

在已授权目录创建 `workbook.toml`、Flow、说明和参考文件。可以复制 [two-step](../../examples/two-step) 作为两步起点；有独立审查和回边时参考 [code-change](../../examples/code-change)，较完整开发方法参考 [spec-dev](../../workbooks/spec-dev)。

使用新的 ID 或版本，不编辑已装版本或 Work 冻结副本。manifest 的 `flows` 明确列出所有 Flow 文件；每个 Flow 指定 entry，并逐项声明 Node 和 Edge。文件引用相对 Workbook 根，不按文件名猜自动加载范围。

## 2. 写清每步的输入与交付

为每个 Node 写说明：读取什么、做什么、完成标准、结果写到哪个输出槽，以及权限、信息或预算不足时交回谁。说明不用写运行时绝对路径，引擎生成任务书时附实际绑定路径。

按用途选择输入：

- 本次任务提供的文本或文件用 `start.<key>`。
- 随方法冻结的参考文件用 `resource.<path>`。
- 上游产物用 `<node>.<output>`；首次允许没有审查反馈时才设 `required=false`。
- 反思所需运行事实用 `engine.stats`。

输出 `path` 位于本 Attempt 的 `outputs/`，使用合法 ASCII 路径和合理 `max_bytes`。回复摘要有界，完整报告写声明文件。上游输出为可选时，下游输入也必须可选；不将“当前没有文件”临时解释为可选。

## 3. 连接合法路径并设置额度

显式声明主干、返工、旁支和复审 Edge。核所有节点从 entry 可达、至少有一个终点、没有自环或重复端点。连线与输入来源分别核对；有边不意味着所有输入已经绑定。

根据实际任务设置 `max_visits` 与 `max_retries`，前者控制到达次数，后者控制同一次到达的真实执行失败重试。内容审查完成但不通过时，提交报告并由协调者选回边，不记作执行 fail。

在确需批准才能离开的节点设 `gate=true`。`executor=human` 表示谁执行，与 gate 不同；human Node 不声明 tier。报告中的“通过”不自动选边或批准门槛。

## 4. 选择明确成果与宿主依赖

在没有出边的终点上，对必需 input/output 设 `result=true`，确保跨两类的选中名称唯一。选择输入时交付终点开工已绑定的原件，选择输出时交付终点提交封存的原件。没有声明时可以成功，但 `work result` 集合为空。

普通文件资料放 `resources/` 并绑定输入。确需宿主 skill、命名 agent 或 MCP 时才在 manifest 写 `requires`，Node 再以 `kind:name` 引用。引擎不检查或安装宿主资源，作者与协调者先确认实际可用性。

## 5. 核验并试运行新版本

按[构建指南](build-from-source.md)取得可信引擎和新的显式管理根。把 `author_dir` 设为实际作者目录，`workbook_spec` 设为定义的 `<id>@<version>`：

```bash
"$engine_binary" --home "$source_home" --json workbook add "$author_dir"
"$engine_binary" --home "$source_home" --json workbook show "$workbook_spec"
"$engine_binary" --home "$source_home" --json workbook verify "$workbook_spec"
```

每步非零或 `ok=false` 时停止，按错误码、字段和规则修作者副本。内容已装入后要改动就使用新版本，不手改管理文件或删除旧版本来绕过冻结。

结构通过后，用新 Work 验证正常路径、必要回边、门槛、失败停止及最终选集。实际文件必须写声明位置，核身份、摘要和内容；结构通过不证明业务质量。可视化操作见[编辑 Workbook](edit-workbook.md)，升级和移除见[方法管理](manage-workbooks.md)。
