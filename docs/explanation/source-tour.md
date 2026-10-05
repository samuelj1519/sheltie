# Source tour

English | [简体中文](source-tour.zh-CN.md)

For developers who know Rust and are reading Sheltie for the first time. Read the [domain vocabulary](../../CONTEXT.md), then trace one public command. [Contracts](../../specs/contracts) define exact interfaces; [architecture](../../specs/architecture.md) defines the full structure.

## Three engine layers and two external tools

| Location | Responsibility | Entry points |
| --- | --- | --- |
| sheltie-cli | Arguments, public commands, error codes, text/JSON output | [main.rs](../../crates/sheltie-cli/src/main.rs), [commands/](../../crates/sheltie-cli/src/commands), [output.rs](../../crates/sheltie-cli/src/output.rs) |
| sheltie-runtime | Files and OS observations, Store transactions, effects, recovery | [service.rs](../../crates/sheltie-runtime/src/service.rs), [load.rs](../../crates/sheltie-runtime/src/load.rs), [recovery.rs](../../crates/sheltie-runtime/src/recovery.rs) |
| sheltie-core | Definition validation, state decisions, next actions, pure rendering | [flow/compile.rs](../../crates/sheltie-core/src/flow/compile.rs), [work/decide.rs](../../crates/sheltie-core/src/work/decide.rs), [work/next.rs](../../crates/sheltie-core/src/work/next.rs) |
| sheltie-export | Verify final results through CLI and publish a new copy under an authorized parent | [crate](../../crates/sheltie-export), [export guide](../how-to/export-results.md) |
| workbook-editor | Edit Workbook drafts, validate through a trusted CLI, export ZIP | [tool README](../../tools/workbook-editor/README.md), [design](workbook-editor.md) |

Engine dependencies flow cli → runtime → core. core receives observed facts without I/O; runtime executes decisions without judging report content. Business methods live in [examples](../../examples) and [workbooks](../../workbooks), rather than engine branches.

## Trace a write command

For attempt submit:

1. [commands/attempt.rs](../../crates/sheltie-cli/src/commands/attempt.rs) passes arguments to WorkService. Request ID and raw intent distinguish new writes from historical replay.
2. `submit` / `run_command` in [service.rs](../../crates/sheltie-runtime/src/service.rs) load a trusted Work and graph, observe declared outputs, and build facts for core. File existence is only a starting observation: identity, ownership, size, and bytes must also be verified.
3. `decide` / `decide_submit` in [decide.rs](../../crates/sheltie-core/src/work/decide.rs) check Attempt qualification, state, and output contracts, returning new state, reply, and file effects. A report's pass/fail content does not affect the engine's decision.
4. [store/commit.rs](../../crates/sheltie-runtime/src/store/commit.rs) records state, request response, and pending effects, using revision to prevent concurrent overwrite.
5. [effects.rs](../../crates/sheltie-runtime/src/effects.rs) executes file effects; [recovery.rs](../../crates/sheltie-runtime/src/recovery.rs) handles incomplete post-commit effects. Database commit and completed file publication may occur at different times; errors must preserve that distinction.
6. CLI maps accurate errors or returns the frozen reply and next. A replay reply belongs to its historical snapshot; query fresh status before a later action.

Diagnose from the public command and error, locate the failure before or after commit, then read the relevant recovery-window tests. Do not merely change the last failing helper or directly modify Store or managed metadata.

## Trace a query

`read_work_bundle` in [store/read.rs](../../crates/sheltie-runtime/src/store/read.rs) reads Work, related requests, and effects in one read-only transaction. load validates complete payloads, state, and file identity. status_read, result, and stats project state and next from that same context, avoiding mixed revisions.

[core result_view](../../crates/sheltie-core/src/work/result.rs) selects results only from declarations on a successful terminal node. For actual bytes, trace [runtime/result.rs](../../crates/sheltie-runtime/src/result.rs) and CLI result-artifact through same-snapshot, same-handle validation. An ordinary result query does not re-prove all file contents.

Read-only operations neither advance business state nor complete file effects. See [D-039](decisions/D-039-sqlite-read-control-files.md) for SQLite control-file exceptions.

## Find tests by question

| Behavior | Independent expectations and real consumers |
| --- | --- |
| Graph and state rules | core flow/work module tests and rendering snapshots |
| Request identity and original replies | [CLI replay](../../crates/sheltie-cli/tests/replay.rs), [runtime schema2_replay](../../crates/sheltie-runtime/tests/schema2_replay.rs) |
| Publication, effects, crash recovery | [effect_contracts](../../crates/sheltie-runtime/tests/effect_contracts.rs), [reliability_crash](../../crates/sheltie-cli/tests/reliability_crash.rs) |
| File identity and path boundaries | [fs_boundary](../../crates/sheltie-runtime/tests/fs_boundary.rs), [snapshot_qualification](../../crates/sheltie-cli/tests/snapshot_qualification.rs) |
| Final results, raw bytes, revocation | [work_result](../../crates/sheltie-cli/tests/work_result.rs), [result_artifact](../../crates/sheltie-cli/tests/result_artifact.rs), [attempt_replace](../../crates/sheltie-cli/tests/attempt_replace.rs) |
| Real methods | [scenario_code_change](../../crates/sheltie-cli/tests/scenario_code_change.rs), [scenario_spec_dev](../../crates/sheltie-cli/tests/scenario_spec_dev.rs) |
| External tools and release governance | [export tests](../../crates/sheltie-export/tests), [editor tests](../../tools/workbook-editor/test), [release_governance](../../crates/sheltie-cli/tests/release_governance.rs) |

Test names describe behavior; Task comments support tool grouping. Historical task IDs do not imply that implementation is still active. Before changes, connect contract obligations, real entry points, positive/negative expectations, and recovery timing. Test simplification must preserve distinct identity, bytes, observation timing, and successor-call evidence. See [engineering](../../specs/engineering.md) and the [validation budget guide](../how-to/validate-change.md).
