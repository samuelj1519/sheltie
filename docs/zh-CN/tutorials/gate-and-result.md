# 观察门槛与明确成果

[English](../../en/tutorials/gate-and-result.md) | 简体中文

本练习创建一个只有终点 Node 的方法，写一份说明，观察“输出提交成功”“等待批准”和“成果可读取”三个时点。只写临时作者目录和新的管理根，不运行模型、不发布任何东西。

先完成[第一个 Work](first-work.md)，并在同一 Bash 会话保留构建得到的 `engine_binary` 与 `session_dir`。本练习另用新的 `gate_home`。除明确观察的 `final=false` 外，任何非零退出、`ok=false` 或结果不符都停止。

## 1. 创建方法目录

```bash
author_dir=$(mktemp -d /private/tmp/sheltie-gate-method.XXXXXX)
gate_home="${author_dir}-home"
mkdir "$author_dir/flows"
cat > "$author_dir/workbook.toml" <<'TOML'
schema = "workbook/v1"
id = "gate-demo"
version = "1.0.0"
name = "门槛与成果练习"
flows = ["flows/default.toml"]
TOML
cat > "$author_dir/flows/default.toml" <<'TOML'
schema = "flow/v1"
id = "default"
entry = "finish"

[[nodes]]
id = "finish"
title = "写一份说明"
executor = "agent"
instruction = { text = "读取 topic，写一段说明到 note 输出。" }
inputs = [{ name = "topic", from = "start.topic", result = true }]
outputs = [{ name = "note", path = "note.md", max_bytes = 65536, result = true }]
gate = true
TOML
"$engine_binary" --home "$gate_home" --json workbook add "$author_dir"
"$engine_binary" --home "$gate_home" --json workbook verify gate-demo@1.0.0
```

这是无出边终点。它明确选择一项输入和一项输出，成功提交后需要门槛批准。

## 2. 开 Work 并领取输出位置

```bash
"$engine_binary" --home "$gate_home" --json work start   --workbook gate-demo@1.0.0 --flow default --input 'topic=冻结文件的用途'   > "$session_dir/gate-start.json"
gate_work=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["data"]["work_id"])' "$session_dir/gate-start.json")
"$engine_binary" --home "$gate_home" --json attempt begin "$gate_work" --node finish   > "$session_dir/gate-begin.json"
gate_attempt=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["data"]["attempt"])' "$session_dir/gate-begin.json")
note_path=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["data"]["outputs"]["note"])' "$session_dir/gate-begin.json")
gate_brief=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["data"]["brief_path"])' "$session_dir/gate-begin.json")
cat "$gate_brief"
```

任务书列出冻结 topic 和 note 的目标位置。本练习手工扮演工作者，写一段说明后提交：

```bash
printf '%s\n' '冻结文件使后续步骤读取同一份输入；原作者的修改不会静默改变已经绑定的内容。' > "$note_path"
"$engine_binary" --home "$gate_home" --json attempt submit "$gate_work"   --attempt "$gate_attempt" --summary '说明已写入'
```

## 3. 查看尚未批准的状态

```bash
"$engine_binary" --home "$gate_home" --json work status "$gate_work"
"$engine_binary" --home "$gate_home" --json work result "$gate_work"
```

状态应为 `{"kind":"blocked","reason":"gate"}`，当前 next 允许批准 finish 或取消。Attempt 已 succeeded，但 Work 尚未成功，`final=false` 且成果集合为空。

## 4. 批准本次练习门槛

阅读 note 并确认本练习可以完成后，由练习者批准这个门槛：

```bash
"$engine_binary" --home "$gate_home" --json gate approve "$gate_work" --node finish
"$engine_binary" --home "$gate_home" --json work result "$gate_work"   > "$session_dir/gate-result.json"
cat "$session_dir/gate-result.json"
```

结果应为 succeeded、`final=true`、`effects_pending=false`，选集按 key 排序为 `note`、`topic`。两项的 `source.attempt` 都是本次 finish Attempt；note 的 kind 为 output，topic 的 kind 为 input。topic 的 bytes 来自起始输入，不是终点重新生成的文本。

## 5. 读取 note 的原字节

从同一次查询取得 revision，再写入本次新暂存文件：

```bash
result_revision=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["data"]["revision"])' "$session_dir/gate-result.json")
raw_note=$(mktemp /private/tmp/sheltie-gate-note.XXXXXX)
"$engine_binary" --home "$gate_home" work result "$gate_work"   --artifact note --revision "$result_revision" > "$raw_note"
cmp "$note_path" "$raw_note"
```

两条命令均应成功，cmp 不输出差异。raw 不加 JSON 包装或换行；消费者要核最终退出码，不能只看文件存在。

现在可以区分封存输出、门槛批准和明确成果。真实任务中的门槛授权来自用户，进程账户记录不提供独立真人认证。继续编写方法见[编写 Workbook](../how-to/write-workbook.md)；复制可编辑成果见[导出指南](../how-to/export-results.md)。
