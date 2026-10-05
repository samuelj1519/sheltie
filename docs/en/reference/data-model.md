# State and result reference

English | [简体中文](../../zh-CN/reference/data-model.md)

[CONTEXT](../../../CONTEXT.md) defines terms. This page organizes facts by `running` objects, `status` queries, and results. The [storage contract](../../../specs/contracts/storage.md) defines exact persistent structures; the [protocol](../../../specs/contracts/protocol.md) defines responses.

## Object relationships

| Object | Relationship/identity | Implementation |
| --- | --- | --- |
| `WorkbookDef`, `FlowDef`, `NodeDef` | Validated method definitions compiled into `Graph` | [workbook](../../../crates/sheltie-core/src/workbook), [flow](../../../crates/sheltie-core/src/flow) |
| `WorkState` | One frozen-version run, with revision/current arrival/history | [work/state.rs](../../../crates/sheltie-core/src/work/state.rs) |
| `Occurrence` | Node's nth arrival in this Work, e.g. `review#2` | work/state.rs |
| `Attempt` | Execution within an arrival, e.g. `review#2.0` | work/state.rs |
| `Command`, `Decision`, `Reply`, `Effect` | Observed command/pure decision/reply/pending file effects | [command.rs](../../../crates/sheltie-core/src/work/command.rs), [decide.rs](../../../crates/sheltie-core/src/work/decide.rs) |
| `ArtifactRef` | Frozen file absolute `path`, `sha256`, `bytes` | work/state.rs |
| `WorkReadBundle`, `StatusReadView` | Single read snapshot/status projection | [store/read.rs](../../../crates/sheltie-runtime/src/store/read.rs), [runtime/result.rs](../../../crates/sheltie-runtime/src/result.rs) |
| `ResultView` | Frozen references explicitly selected by a successful terminal | [work/result.rs](../../../crates/sheltie-core/src/work/result.rs) |

`Occurrence` starts at 1; `Attempt` `number` increases consecutively from 0. Failure retries stay within the same arrival; returning along an edge creates another Occurrence. Replacement increments `number` without incrementing real failure counts. Responses supply actual IDs.

## Work states and legal actions

JSON states are objects, e.g. `{"kind":"active"}` / {"kind":"blocked","reason":"gate"}. Text `status` cards use `active` / blocked(gate).

| State | Meaning | Legal actions |
| --- | --- | --- |
| `active` | Run can continue | Claim from `next` without a `running` `Attempt`; otherwise submit, real fail, or permitted replace; cancel |
| `blocked(gate)` | Successful submission awaits approval | Current-node `gate approve` or cancel |
| `blocked(retries_exhausted)` | Actual execution failures exhausted for this arrival | Cancel only, without increasing frozen limits |
| `blocked(no_legal_edge)` | Arrival limits exhausted for every edge target | Cancel only, without adding edges/skipping nodes |
| `succeeded` | Terminal `succeeded` and required gates approved | Terminal; `next=[]`, queries remain |
| `cancelled` | Run `cancelled` | Terminal; `next=[]`, queries remain; host processes are not stopped |

This is a summary. Current `work status` `next` determines invocation qualification. executor/tier in `next` guide delegation; labels do not invoke models or install host resources.

## `Attempt` states

| State | Meaning/file boundary |
| --- | --- |
| `running` | Outputs are drafts and may not exist yet |
| `succeeded` | Output contracts passed and sealing recorded; coordinator judges quality |
| `failed` | Actual execution failure, counted toward `max_retries` |
| `superseded` | Formal submission qualification revoked; old draft facts remain, without business failure |

Each `Occurrence` allows one replacement. The new `Attempt` inherits frozen non-statistical inputs, without adopting old drafts; stats use new committed state. Work cancellation does not mark a `running` `Attempt` `failed` or permit later submission.

## `status`, `resume`, and statistics

`work status` `data` includes work_id/status/current/done/resume/revision/effects_pending/pending_publish. Top-level `next` comes from the same snapshot. See [protocol §6](../../../specs/contracts/protocol.md#6-status-card-status-cardmd) for the complete card.

`resume=null` means no `current` Attempt. Otherwise it includes `current` `Attempt` identity, brief, frozen inputs, and draft_outputs. `resume.inputs` maps names to ArtifactRef/null; read `path` from references. `draft_outputs` maps names to draft paths. These differ from begin `data.inputs/outputs`, which directly contain absolute paths. Non-null does not ensure running. Draft locations appear only while `running` and do not prove files exist. Check Work state, `Attempt` identity, and `current` qualification together before resuming.

`effects_pending` means related committed requests have unfinished effects. `pending_publish` means Work publication is incomplete. Reads do not recover them. Disk `status-card.md` is a write-generated projection, without replacing fresh queries.

`work stats` lists per-node visits/attempts/failed/superseded/avg_seconds/edge-source counts and Work totals. Durations derive from ended Attempts and include human/waiting time; they are not model usage/cost. See the [statistics contract](../../../specs/contracts/protocol.md#work-stats-work).

## Final results

`work result` data.format=work-result/v1. State, `revision`, effect facts, and selections come from one snapshot.

| Result | Meaning |
| --- | --- |
| `final=false`, `artifacts=[]` | No qualified successful terminal yet, or incomplete effects |
| `final=true`, `artifacts=[]` | Successful terminal declared no result selections |
| `final=true`, nonempty `artifacts` | Explicit terminal-selected frozen inputs/sealed outputs |

Each entry has key/path/sha256/bytes/source. `source.attempt` is the successful terminal that bound/sealed it. `source.kind` is input/output; `source.name` is the logical slot. An `input`'s producer may be earlier: the selecting `Attempt` is not necessarily its producer.

Reference queries do not reauthenticate all `source` `bytes` or verify code candidates/check results in reports. Reads/copies use the same query `revision` and literal `key`, verifying final exit, `bytes`, and `sha256`; see [export](export.md).
