# Run a code-change repository `task`

English | [简体中文](../../zh-CN/how-to/run-code-change.md)

For coordinators using the v0.3.0/schema 4/cli-result/v4 baseline for the first time. [README remote installation](../../../README.md#install-the-released-engine) targets this baseline; [release records](../reference/releases/README.md) establish availability. This guide runs source-build artifacts directly.

[code-change](../../../examples/code-change/README.md) handles authorized local work: implement → review → deliver with review back to implement. Workers execute content; coordinators check reports/candidates/checks and choose legal actions. Engine records/freezes/enforces outputs. Default has no gate and grants no merge/deploy/publish authority.

## 1. Build and freeze paths

Get engine_binary/source_home/session_dir/project_repo through [building](build-from-source.md), verify version/root, and continue in one Bash session. Existing Works use original binaries/roots, without new-root continuation.

## 2. Prepare actual `task` and `project`

`task` specifies goals/constraints/acceptance/stopping budget; `project` specifies absolute repository/initial candidate/allowed files/workspace/checks. start freezes both byte-for-byte. Stop for missing standards; changed goals require another Work.

This executable docs example applies only to authorized `README.md` changes in this `source` checkout. Replace both `inputs` for other tasks before start; do not alter methods to fit. Read-only-check clean state first; otherwise preserve existing edits and confirm prerequisites with the authorizing party, without cleanup:

```bash
git status --short --untracked-files=all
project_head=$(git rev-parse HEAD)
cat > "$session_dir/task.md" <<'TASK'
# Task
Add cargo fmt --all -- --check to README.md's Development command block. If already accurate, verify without duplication.
Preserve every original command and other text. Edit README.md only.
Acceptance: exact command appears once; links and diff whitespace pass.
Stop for failed checks/missing permissions/standards, retaining raw outputs. No install/merge/publication.
Budget: obey method visit/failure limits; never delete checks or expand scope for PASS.
TASK
cat > "$session_dir/project.md" <<PROJECT
# Project
Absolute repository: $project_repo
Initial HEAD: $project_head
Workspace: read-only verified clean; preserve others' files without cleanup/rollback.
Allowed edits: README.md. External task inputs are not product files to commit.
Required checks (repository root):
- scripts/check-docs.sh README.md
- git diff --check
Reports bind actual candidate/changed files/full commands/cwd/environment/stdout/stderr/exits.
Candidate changes invalidate old checks; retain unknown/not_run accurately.
PROJECT
```

## 3. Install the method and start Work

```bash
"$engine_binary" --home "$source_home" --json workbook add examples/code-change
"$engine_binary" --home "$source_home" --json workbook verify code-change@1.0.1
"$engine_binary" --home "$source_home" --json workbook show code-change@1.0.1
```

Check each success. verify must be ok. `default` Flow `data.flows[].start_inputs` must be task/project. Inspect `requires` and stop for missing resources; engine neither probes nor installs. This example declares none.

For safe Work/Workbook retries, save UUID before calling `--request-id`, preserving ID/parameters. Reads/all `self` reject it. Example start:

```bash
start_request_id=$(python3 -c 'import uuid; print(uuid.uuid4())')
"$engine_binary" --home "$source_home" --json --request-id "$start_request_id" \
  work start --workbook code-change@1.0.1 --flow default \
  --input "task=@$session_dir/task.md" --input "project=@$session_dir/project.md"
read -r -p 'Paste data.work_id from successful start: ' work_id
"$engine_binary" --home "$source_home" --json work status "$work_id"
```

Retain full ID/original response. Current `next` must permit implement, with `effects_pending`=false/pending_publish=false before progression.

## 4. Claim briefs, delegate, and submit

Check each node in current `next` before begin. These show normal first-round order, without unconditional script execution:

```bash
"$engine_binary" --home "$source_home" --json attempt begin "$work_id" --node implement
```

Read data.attempt/brief_path/inputs/outputs and give actual `brief_path` to the worker. It reads full brief/frozen task/project, performs authorized edits/checks, and writes `change.md` at outputs.change. First line gives actual conclusion; bind candidate/raw checks/failures/unknown/not_run. Initial `previous-review`=null is not feedback. Inputs/sealed outputs are read-only; never write Store/managed metadata.

Coordinator checks report/actual candidate/checks/required files before submit. Use actual begin Attempt ID without suffix guesses:

```bash
read -r -p 'Paste data.attempt from implement begin: ' attempt_id
"$engine_binary" --home "$source_home" --json attempt submit "$work_id" \
  --attempt "$attempt_id" --summary "Implementation and checks completed; see change.md"
"$engine_binary" --home "$source_home" --json work status "$work_id"
```

Engine submit validates file properties/limits/sealing, without report code/check-verdict evaluation. Missing/oversized outputs leave running; handle exact errors without calling rejection success.

## 5. Independent review, rework, and delivery

When current `next` permits review:

```bash
"$engine_binary" --home "$source_home" --json attempt begin "$work_id" --node review
```

Use an uninvolved reviewer reading new brief/frozen inputs/actual candidate against `task` criteria, writing `outputs.review` review.md. Coordinator reads and submits with actual review begin ID:

```bash
read -r -p 'Paste data.attempt from review begin: ' attempt_id
"$engine_binary" --home "$source_home" --json attempt submit "$work_id" \
  --attempt "$attempt_id" --summary "Independent review completed; see review.md"
"$engine_binary" --home "$source_home" --json work status "$work_id"
```

Content rework still submits; select `edge=back` implement from current next. New briefs bind specific previous-review. Repair then independent review again. When deliverable, choose `edge=main` deliver. Report pass does not automatically choose edges/approve gates.

```bash
"$engine_binary" --home "$source_home" --json attempt begin "$work_id" --node deliver
```

Delivery workers read actual briefs, verify change/review point to one candidate, and write `outputs.delivery` with locations/raw checks/review/instructions/remaining obligations. Coordinator verifies then submits actual ID:

```bash
read -r -p 'Paste data.attempt from deliver begin: ' attempt_id
"$engine_binary" --home "$source_home" --json attempt submit "$work_id" \
  --attempt "$attempt_id" --summary "Delivery prepared; see delivery.md for remaining obligations"
"$engine_binary" --home "$source_home" --json work result "$work_id"
```

Use selections only with final=true/effects_pending=false. This method selects change/review/delivery. Each path/sha256/bytes/source binds specific deliver inputs/sealed outputs. `final=false` is empty; never guess latest reports. Queries do not re-prove `source` bytes/quality/acceptance. Verify actual `bytes` for consumption. See [export](export-results.md); copying/merging/publication each require actual authorization.

## 6. Interruption, errors, and stops

Restore original binary/explicit Home after reopening, then read-only discover/query:

```bash
"$engine_binary" --home "$source_home" --json work list
"$engine_binary" --home "$source_home" --json work status "$work_id"
"$engine_binary" --home "$source_home" --json work stats "$work_id"
```

Check revision/status/pending/next/resume. null means no current Attempt; otherwise use brief_path/frozen inputs/drafts, whose paths prove no existence/completeness/sealing. Confirm old-executor/workspace handling before resuming running. Ordinary reopening does not manufacture fail/begin/replace. Replay `next` is historical, without current qualification.

| Observation | Boundary |
| --- | --- |
| Nonzero, `ok=false`, unknown commit | Stop; retain command/cwd/environment/stdout/stderr/exit/request ID/candidate. No unsupported success or changed standards. Verify unknown writes by saved ID/parameters, without guessed new Work |
| EFFECT_PENDING/pending flags | Queries do not recover. `committed=true` retries same ID; false recovers `pending_request_id` then original B. Retain `cause`; persistent errors/unknown intent or identity stop without managed edits |
| Actual crash/timeout/no delivery | Only legal `next` permits truthful fail. Content failures submit/back. implement/review allow 3 visits, 1 failure retry each; deliver 1 visit/1 retry. No increased limits/nodes |
| retries_exhausted/no_legal_edge | Stop/report, choose only authorized next. Cancellation is a separately authorized write. Also obey external time/call/cost budgets; unknown cost remains unknown |
| succeeded/cancelled | Terminal, no begin/submit/fail. Read explicit results after success; cancellation has no successful selections. State is not quality/acceptance |
| `STORE_SCHEMA_MISMATCH` | Reject schema 1/2/3 as a whole. Preserve old roots/binaries for history, use new Home without migration/clearing/database rollback |
| ARTIFACT_MODIFIED/STORE_CORRUPT/missing standards or authority | Preserve originals/exact errors; stop or hand back, without altered frozen inputs/sealed reports/acceptance |

For actual failure, get current `status` Attempt ID and record its concrete `cause`:

```bash
"$engine_binary" --home "$source_home" --json attempt fail "$work_id" \
  --attempt "$attempt_id" --reason "Actual execution failure reason"
```

See [protocol](../../../specs/contracts/protocol.md) and [coordinator skill](../../../skills/sheltie/SKILL.md).
