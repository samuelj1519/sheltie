# Finish your first Work

English | [简体中文](first-work.zh-CN.md)

Use two-step: outline then summary. Learn method versus run, claiming versus submitting, and flow success versus result selection. Only new temporary roots/declared outputs are written, without installation/repository/old-Store changes.

Open Bash at the root. [Build](../how-to/build-from-source.md) engine_binary/source_home/session_dir/project_repo and verify self version. Continue in one session. Stop on nonzero/ok=false/unverifiable outcomes and retain full output; see [resume](../how-to/resume-work.md).

## 1. Install two-step

```bash
"$engine_binary" --home "$source_home" --json workbook add examples/two-step
"$engine_binary" --home "$source_home" --json workbook verify two-step@1.0.1
"$engine_binary" --home "$source_home" --json workbook show two-step@1.0.1
```

show lists outline/summary in default with topic input. See [method directory](../../examples/two-step). Installation freezes a copy, without model calls.

## 2. Create a run

```bash
"$engine_binary" --home "$source_home" --json work start \
  --workbook two-step@1.0.1 --flow default --input 'topic=Introduce Sheltie' \
  > "$session_dir/start.json"
cat "$session_dir/start.json"
work_id=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["data"]["work_id"])' "$session_dir/start.json")
```

Retain actual Work ID. next permits begin outline/cancel, without skipping to summary.

## 3. Claim the brief and write an outline

```bash
"$engine_binary" --home "$source_home" --json attempt begin "$work_id" --node outline \
  > "$session_dir/outline-begin.json"
cat "$session_dir/outline-begin.json"
brief_path=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["data"]["brief_path"])' "$session_dir/outline-begin.json")
outline_path=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["data"]["outputs"]["outline"])' "$session_dir/outline-begin.json")
outline_attempt=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["data"]["attempt"])' "$session_dir/outline-begin.json")
cat "$brief_path"
```

The brief gives frozen topic paths/outline requirements. Real use delegates it to workers; here write a three-point outline manually:

```bash
cat > "$outline_path" <<'OUTLINE'
- Methods, runs, and responsibilities
  - Workbook holds a method; Work holds one run.
  - Coordinators judge content; the engine records facts and legal next actions.
- Inputs, outputs, and continuation
  - Inputs freeze by bytes; submission seals outputs.
  - After interruption, continue from current briefs and frozen inputs.
- Gates, state, and results
  - Methods may require approval gates; this exercise has none.
  - Flow success differs from explicit result selection.
OUTLINE
"$engine_binary" --home "$source_home" --json attempt submit "$work_id" \
  --attempt "$outline_attempt" --summary 'Three-point outline written'
"$engine_binary" --home "$source_home" --json work status "$work_id"
```

submit checks/seals files without judging outlines. New next permits summary; previous files are no longer editable drafts.

## 4. Write a summary from the frozen outline

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

Confirm begin-bound outline reads, then write three paragraphs:

```bash
cat > "$summary_path" <<'SUMMARY'
Sheltie holds reusable methods in Workbooks and individual runs in Works. Methods declare steps, inputs, outputs, and explicit edges. Coordinators interpret tasks/reports, choose legal next actions, and delegate content to workers through briefs. The engine records execution facts, generates briefs, and restricts actions without judging natural-language correctness.

Each claim creates an Attempt. Briefs list frozen inputs and output locations; submission validates file contracts and seals bytes for bound downstream use. After interruption, coordinators query current state and read current briefs/frozen inputs. Draft paths only locate potential content and prove neither existence nor sealing.

Methods may require approval gates; this two-step exercise has none. Terminal completion succeeds the Work, while readers judge content quality. Final results additionally need explicit terminal input/output selection. A successful flow without selections has an empty result set; directory files cannot be guessed as final results.
SUMMARY
"$engine_binary" --home "$source_home" --json attempt submit "$work_id" \
  --attempt "$summary_attempt" --summary 'Summary follows the three-point outline'
```

## 5. Observe success and selections

```bash
"$engine_binary" --home "$source_home" --json work status "$work_id"
"$engine_binary" --home "$source_home" --json work stats "$work_id"
"$engine_binary" --home "$source_home" --json work result "$work_id"
```

Expect succeeded/empty next and one execution per node. result may be final=true with empty artifacts because two-step declares none. Outputs still exist at sealed Attempt paths: absence of selection is not execution failure.

This completes one fixed path. See [model](../explanation/workflow-model.md), [authoring](../how-to/write-workbook.md), or real-task [code-change](../how-to/run-code-change.md). This tutorial does not alter examples to manufacture selections.
