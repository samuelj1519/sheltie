# 跑完第一个 Work

[English](first-work.md) | 简体中文

这个练习使用仓库的 two-step 方法：先列提纲，再写摘要。完成后能区分方法与运行、领取与提交，以及流程成功与最终成果声明。练习只写新的临时管理根和声明输出，不安装引擎、不修改仓库或旧 Store。

先在仓库根打开 Bash，按[构建指南](../how-to/build-from-source.zh-CN.md)取得 `engine_binary`、`source_home`、`session_dir` 和 `project_repo`，核 `self version` 的版本与根。在同一会话继续。每条命令非零、`ok=false` 或结果无法核实时停止，保存完整输出；学习排障见[接续指南](../how-to/resume-work.zh-CN.md)。

## 1. 装入两步方法

```bash
"$engine_binary" --home "$source_home" --json workbook add examples/two-step-zh-CN
"$engine_binary" --home "$source_home" --json workbook verify two-step-zh-cn@1.0.0
"$engine_binary" --home "$source_home" --json workbook show two-step-zh-cn@1.0.0
```

show 展示 default Flow 的 outline、summary 两个 Node，入口需要 topic。方法说明在 [two-step 目录](../../examples/two-step)；这里只装入冻结副本，不调用模型。

## 2. 创建一次运行

```bash
"$engine_binary" --home "$source_home" --json work start \
  --workbook two-step-zh-cn@1.0.0 --flow default --input 'topic=介绍 Sheltie' \
  > "$session_dir/start.json"
cat "$session_dir/start.json"
work_id=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["data"]["work_id"])' "$session_dir/start.json")
```

保存真实 Work ID。响应 next 应允许 begin outline；它还允许取消，不意味着协调者可以跳到 summary。

## 3. 领取任务书并完成提纲

```bash
"$engine_binary" --home "$source_home" --json attempt begin "$work_id" --node outline \
  > "$session_dir/outline-begin.json"
cat "$session_dir/outline-begin.json"
brief_path=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["data"]["brief_path"])' "$session_dir/outline-begin.json")
outline_path=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["data"]["outputs"]["outline"])' "$session_dir/outline-begin.json")
outline_attempt=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["data"]["attempt"])' "$session_dir/outline-begin.json")
cat "$brief_path"
```

任务书给出冻结 topic 的输入路径和 outline 输出要求。在真实使用中将它交给工作 agent；练习中手工完成工作者动作，写三项提纲：

```bash
cat > "$outline_path" <<'OUTLINE'
- 方法、运行与分工
  - Workbook 保存方法，Work 保存一次运行。
  - 协调者判断内容，引擎记录事实与合法下一步。
- 输入、输出与接续
  - 输入按字节冻结，输出提交后封存。
  - 中断后从当前任务书与冻结输入继续。
- 门槛、状态与成果
  - 方法可以声明需要批准的门槛，本练习没有门槛。
  - 流程成功与明确成果选择分别表达。
OUTLINE
"$engine_binary" --home "$source_home" --json attempt submit "$work_id" \
  --attempt "$outline_attempt" --summary '已写三项提纲'
"$engine_binary" --home "$source_home" --json work status "$work_id"
```

submit 核文件合同并封存，不判断提纲好坏。新的 next 应允许 begin summary；上一步文件不再是可编辑草稿。

## 4. 按冻结提纲写摘要

```bash
"$engine_binary" --home "$source_home" --json attempt begin "$work_id" --node summary \
  > "$session_dir/summary-begin.json"
summary_brief=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["data"]["brief_path"])' "$session_dir/summary-begin.json")
frozen_outline=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["data"]["inputs"]["outline"])' "$session_dir/summary-begin.json")
summary_path=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["data"]["outputs"]["summary"])' "$session_dir/summary-begin.json")
summary_attempt=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["data"]["attempt"])' "$session_dir/summary-begin.json")
cat "$summary_brief"
cat "$frozen_outline"
```

确认读取的是 begin 绑定的提纲，再写三段摘要：

```bash
cat > "$summary_path" <<'SUMMARY'
Sheltie 用 Workbook 保存可复用的方法，用 Work 保存这份方法的一次运行。方法声明步骤、输入、输出和显式边；协调者理解任务与报告，选择合法下一步，工作者按任务书完成内容。引擎负责记录执行事实、生成任务书和限制操作，不从自然语言判断内容是否正确。

每次领取形成一个 Attempt。任务书列出本次冻结输入和声明输出位置，提交时引擎核文件合同并封存字节，下游继续使用已绑定的来源。会话中断后，协调者先查询当前状态，再读取当前任务书与冻结输入；草稿路径只是位置提示，不能证明内容存在或已经封存。

方法可以在需要批准的节点声明门槛，本练习的两步方法没有门槛。完成终点后，Work 成为 succeeded，但内容质量仍由读者判断。最终成果还需要终点明确选择输入或输出；没有声明选集时，工作流可以成功而成果集合为空，不能从目录里猜一份文件充当最终成果。
SUMMARY
"$engine_binary" --home "$source_home" --json attempt submit "$work_id" \
  --attempt "$summary_attempt" --summary '已按三项提纲写摘要'
```

## 5. 观察成功与成果选择

```bash
"$engine_binary" --home "$source_home" --json work status "$work_id"
"$engine_binary" --home "$source_home" --json work stats "$work_id"
"$engine_binary" --home "$source_home" --json work result "$work_id"
```

status 应为 succeeded，next 为空；统计中两步各执行一次。result 可以 final=true 而 artifacts 为空，因为 two-step 没有 result 声明。两份输出仍在具体 Attempt 的封存路径，这是方法未选择最终选集，不是运行失败。

至此完成一条固定路径。理解两种结果见[工作流模型](../explanation/workflow-model.zh-CN.md)；准备自己的方法见[编写 Workbook](../how-to/write-workbook.zh-CN.md)；实际代码任务使用[code-change 指南](../how-to/run-code-change.zh-CN.md)。本教程不修改既有样例来人为增加成果选择。
