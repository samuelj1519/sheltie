# Write a Workbook

English | [简体中文](../../zh-CN/how-to/write-workbook.md)

Express stable stages, inputs/outputs, and allowed rework as runnable methods. Specify goals/executors/delivery/stops first. Executors investigate/subdivide within stages; temporary actions need not each become Nodes.

For a complete minimal definition, finish [gates/results](../tutorials/gate-and-result.md). This guide writes your own method. See [reference](../reference/workbook.md) and [contract](../../../specs/contracts/workbook.md).

## 1. Create an independent author directory

Create workbook.toml/Flows/instructions/resources under an authorized directory. Copy [two-step](../../../examples/two-step) for basics, [code-change](../../../examples/code-change) for independent review/back edges, or [spec-dev](../../../workbooks/spec-dev) for fuller development.

Use new IDs/versions, without editing installed/frozen copies. `flows` lists every Flow; each declares entry/Nodes/Edges. References are relative to Workbook root, without inferred auto-loading by filename.

## 2. Specify inputs and deliveries per step

Instructions state reads/actions/completion/output slots and handback owners for missing authority/information/budget. Omit runtime absolute paths; generated briefs append actual bindings.

Choose sources by purpose:

- Per-task text/files: start.<key>.
- Frozen method references: resource.<path>.
- Upstream outputs: `<node>.<output>`; `required=false` only when missing initial review feedback is allowed.
- Reflection facts: engine.stats.

Output `path` is within Attempt `outputs/`, with legal ASCII/reasonable max_bytes. Summaries are bounded; full reports use declared files. Optional upstream outputs need optional downstream inputs; missing-now does not redefine requirements.

## 3. Connect legal routes and limits

Declare main/back/branch/re_review edges explicitly. Verify reachability/terminal existence/no self-loops/duplicate endpoints. Check edges and input sources separately; edges alone do not bind all inputs.

Set `max_visits` for arrivals and `max_retries` for actual failure retries within arrivals. Completed negative content reviews submit; coordinators choose back edges, without execution fail.

Use `gate=true` where approval is required before leaving. `executor=human` is separate from gate and omits tier. Report pass does not choose edges/approve gates.

## 4. Select results and host dependencies

On terminals without outgoing edges, select required inputs/outputs with `result=true` and unique names across both. Inputs deliver begin-bound originals; outputs deliver submit-sealed originals. Success without declarations has empty results.

Use `resources/` for file material. Only actual host skill/named agent/MCP needs use manifest `requires` and Node kind:name. Engine neither checks nor installs; authors/coordinators verify availability.

## 5. Verify and trial the version

[Build](build-from-source.md) trusted engine/new explicit root. Set actual author_dir/workbook_spec:

```bash
"$engine_binary" --home "$source_home" --json workbook add "$author_dir"
"$engine_binary" --home "$source_home" --json workbook show "$workbook_spec"
"$engine_binary" --home "$source_home" --json workbook verify "$workbook_spec"
```

Stop on nonzero/ok=false and repair author copies by codes/fields/rules. After installation, changes require new versions, without managed edits/deletion to bypass freezing.

After structure passes, new Works test normal/back/gate/failure-stop/final selections. Write actual declared files and verify identity/digests/content. Structure does not prove quality. See [canvas](edit-workbook.md) and [management](manage-workbooks.md).
