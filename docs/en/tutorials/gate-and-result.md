# Observe gates and explicit results

English | [简体中文](../../zh-CN/tutorials/gate-and-result.md)

Create a single-terminal method, write a `note`, and observe successful submission, waiting approval, and readable results. Write only temporary author directories/new roots, without models/publication.

Complete [first Work](first-work.md), retaining engine_binary/session_dir in the same Bash session. Use a new `gate_home` here. Except expected `final=false`, stop on nonzero/ok=false/mismatched outcomes.

## 1. Create the method directory

```bash
author_dir=$(mktemp -d /private/tmp/sheltie-gate-method.XXXXXX)
gate_home="${author_dir}-home"
mkdir "$author_dir/flows"
cat > "$author_dir/workbook.toml" <<'TOML'
schema = "workbook/v1"
id = "gate-demo"
version = "1.0.0"
name = "Gate and result exercise"
flows = ["flows/default.toml"]
TOML
cat > "$author_dir/flows/default.toml" <<'TOML'
schema = "flow/v1"
id = "default"
entry = "finish"

[[nodes]]
id = "finish"
title = "Write a note"
executor = "agent"
instruction = { text = "Read topic and write a paragraph to the note output." }
inputs = [{ name = "topic", from = "start.topic", result = true }]
outputs = [{ name = "note", path = "note.md", max_bytes = 65536, result = true }]
gate = true
TOML
"$engine_binary" --home "$gate_home" --json workbook add "$author_dir"
"$engine_binary" --home "$gate_home" --json workbook verify gate-demo@1.0.0
```

This terminal has no outgoing edges, explicitly selects one input/output, and requires approval after successful submission.

## 2. Start Work and claim output locations

```bash
"$engine_binary" --home "$gate_home" --json work start   --workbook gate-demo@1.0.0 --flow default --input 'topic=Why freeze files'   > "$session_dir/gate-start.json"
gate_work=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["data"]["work_id"])' "$session_dir/gate-start.json")
"$engine_binary" --home "$gate_home" --json attempt begin "$gate_work" --node finish   > "$session_dir/gate-begin.json"
gate_attempt=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["data"]["attempt"])' "$session_dir/gate-begin.json")
note_path=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["data"]["outputs"]["note"])' "$session_dir/gate-begin.json")
gate_brief=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["data"]["brief_path"])' "$session_dir/gate-begin.json")
cat "$gate_brief"
```

The brief lists frozen topic/note destinations. Act as the worker manually, write a paragraph, then submit:

```bash
printf '%s\n' 'Frozen files let later steps read identical inputs; author edits do not silently change already bound content.' > "$note_path"
"$engine_binary" --home "$gate_home" --json attempt submit "$gate_work"   --attempt "$gate_attempt" --summary 'Note written'
```

## 3. Inspect unapproved status

```bash
"$engine_binary" --home "$gate_home" --json work status "$gate_work"
"$engine_binary" --home "$gate_home" --json work result "$gate_work"
```

Expect `{"kind":"blocked","reason":"gate"}`, with next permitting finish approval/cancellation. Attempt succeeded, Work not yet; final=false/empty selections.

## 4. Approve this exercise gate

After reading `note` and confirming this exercise can finish, approve:

```bash
"$engine_binary" --home "$gate_home" --json gate approve "$gate_work" --node finish
"$engine_binary" --home "$gate_home" --json work result "$gate_work"   > "$session_dir/gate-result.json"
cat "$session_dir/gate-result.json"
```

Expect succeeded/final=true/effects_pending=false, sorted note/topic. Both `source.attempt` refer to this finish Attempt; `note` kind=output, `topic` kind=input. `topic` bytes come from start rather than terminal-generated text.

## 5. Read original `note` bytes

Get revision from the same query and write a new temporary file:

```bash
result_revision=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["data"]["revision"])' "$session_dir/gate-result.json")
raw_note=$(mktemp /private/tmp/sheltie-gate-note.XXXXXX)
"$engine_binary" --home "$gate_home" work result "$gate_work"   --artifact note --revision "$result_revision" > "$raw_note"
cmp "$note_path" "$raw_note"
```

Both commands succeed; cmp prints no difference. Raw adds no JSON/newline. Verify final exits rather than existence alone.

You can now distinguish sealed outputs, approval, and explicit results. Real-task gate authorization comes from users; process accounts do not authenticate independent humans. See [authoring](../how-to/write-workbook.md) or [editable copies](../how-to/export-results.md).
