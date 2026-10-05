# Troubleshoot command errors and blocked runs

English | [简体中文](troubleshoot.zh-CN.md)

For nonzero exits/ok=false/pending effects/blocked flows. Identify real binaries/roots/targets first. [Protocol §7](../../specs/contracts/protocol.md#7-closed-error-code-set) defines complete codes; this guide supplies order, without replacing contracts.

## 1. Retain failure state

Retain command/cwd/binary/version/explicit Home/stdout/stderr/exit/request ID/Work ID/Attempt ID. Record only relevant environment variables, avoiding credentials in reports.

Complete responses provide error.code/detail/committed/request identity. Incomplete responses remain unknown; timeouts/stderr do not prove no commit. Original UUID/intent permits same-ID verification. Missing records stop, without guessed new IDs.

## 2. Read-only check versions, roots, and state

Query trusted engine_binary/actual `management_root`:

```bash
"$engine_binary" --home "$management_root" --json self version
"$engine_binary" --home "$management_root" --json work list
```

For specific Works, confirm full `work_id`:

```bash
"$engine_binary" --home "$management_root" --json work status "$work_id"
```

Check versions/schema/Home/revision/pending/resume/next. `NOT_FOUND` on new-root list does not prove old deletion; verify root selection. Reads omit request IDs and do not recover effects.

## 3. Handle errors

| Error | Check/action |
| --- | --- |
| `INVALID_REQUEST`, `INPUT_MISSING` | Help/start_inputs/extra keys/UUID/parameter combinations; `@file` UTF-8/size/safe single-link regular file/path |
| `NOT_FOUND` | Full ID/version/Flow/root; restarting Works does not locate existing objects |
| `WORKBOOK_INVALID`, `FLOW_INVALID` | Repair author copies using `detail.path/rule` and reinstall; never edit frozen/installed directories |
| `WORKBOOK_EXISTS` | Verify identity; changed content needs a new version, without deletion to bypass freezing |
| `WORKBOOK_IN_USE` | Query `detail.works`; complete or explicitly cancel before removal |
| `WORKBOOK_TAMPERED`, `ARTIFACT_MODIFIED` | Preserve objects/digests/originals for human investigation; do not rewrite bytes to manufacture expected results |
| `OUTPUT_MISSING`, `OUTPUT_TOO_LARGE` | Attempt remains running; `original` executor fixes declared outputs and rechecks before submit |
| `SUMMARY_TOO_LONG` | ≤ 4096 bytes; full report in declared outputs, without oversized `@file` bypass |
| `ILLEGAL_NEXT`, `ATTEMPT_NOT_RUNNING` | Fresh status/identity/next, without stale snapshots |
| `REPLACEMENTS_EXHAUSTED` | One replacement used; current qualification remains; no increased limits |
| `WORK_TERMINAL` | Stop writes; queries remain. New goals need new Works, without resurrection |
| `REQUEST_CONFLICT` | Compare original targets/parameters; recovery retains intent. New UUID only for independent new operations |
| `REVISION_CONFLICT` | Query/check concurrency/choose legal `next`; no unconditional automatic retry |
| `EFFECT_PENDING` | Recover registered requests using committed/pending_request_id below |
| `STORE_SCHEMA_MISMATCH` | Development rejects 1/2/3; retain old roots/binaries, new roots for new formats |
| `STORE_CORRUPT` | Preserve database/files/requests/errors, stop for human investigation; no state edits/fabricated originals |
| `UPDATE_UNAVAILABLE`, `UPDATE_CHECKSUM_MISMATCH` | Version/platform/source/manifest; no bypass or treating mismatched packages as installed |
| `IO` | Permissions/disk/types from `detail.path`; establish commit/ownership before handling, without lock/WAL/pending deletion |

## 4. Recover incomplete effects

`committed=true` retries `original` ID/target/user parameters. false with `pending_request_id=A` means B uncommitted: recover A from records, then retry B. original/pending_original belong to different requests.

See [recovery](resume-work.md#incomplete-file-effects-and-unknown-outcomes). Queries do not recover; no pending deletion, conflict overwrite, or fabricated failure. Query status after recovery because replay `next` remains historical.

## 5. Inspect blocked flows

| State | Boundary |
| --- | --- |
| `blocked(gate)` | Present outputs; explicit approval before legal gate approve |
| retries_exhausted/no_legal_edge | Stop/report; cancellation alone remains and needs authorization |
| Running after reopening | Confirm executor/workspace, resume old Attempt without automatic fail/replace |
| Success with empty results | Check terminal `result=true`; valid success may select nothing |

See [export failures](export-results.md#failure-state). Authoring check errors locate draft problems; retain material before refresh. See [tool reference](../reference/workbook-editor.md) for processes/HTTP. Unconfirmed identity/originals/permissions remain unknown; stop affected writes for human investigation.
