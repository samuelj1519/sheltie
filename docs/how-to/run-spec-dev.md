# Run specification-driven development with spec-dev

English | [简体中文](run-spec-dev.zh-CN.md)

For authorized repositories needing specifications/plans before task implementation, independent verification, full `review`, and delivery. For one bounded change use smaller [code-change](run-code-change.md). See [method README](../../workbooks/spec-dev/README.md) for exact instructions/routing; the engine does not interpret reports automatically.

## 1. Prepare method, `request`, and repository

[Build](build-from-source.md) engine_binary/source_home/session_dir. Prepare `request_file` with actual requirements/quality/scope/budget and `task_repo` as authorized absolute repository. Check HEAD/existing edits/allowed actions, preserving others' work.

Install/check from the source root:

```bash
"$engine_binary" --home "$source_home" --json workbook add workbooks/spec-dev
"$engine_binary" --home "$source_home" --json workbook show spec-dev@0.2.3
"$engine_binary" --home "$source_home" --json workbook verify spec-dev@0.2.3
```

Check success, default request/project start keys, and requires. Missing required resources stop; engine does not install. Default English method 0.2.3 differs from engine versions; Chinese spec-dev-zh-cn uses `0.2.2` in workbooks/spec-dev-zh-CN.

## 2. Create Work

Save UUID/full command first:

```bash
start_request_id=$(python3 -c 'import uuid; print(uuid.uuid4())')
"$engine_binary" --home "$source_home" --json --request-id "$start_request_id"   work start --workbook spec-dev@0.2.3 --flow default --name 'Specification development'   --input "request=@$request_file" --input "project=$task_repo"   > "$session_dir/spec-dev-start.json"
cat "$session_dir/spec-dev-start.json"
```

`request` freezes file content; `project` freezes path text, without repository copying. Save actual work_id. Query current status before progression; retries preserve ID/intent rather than creating Works for unknown responses.

## 3. Organize implementation and verification from briefs

Select legal `next` nodes for begin. Executors read actual brief_path/all bound inputs and write declared outputs. Coordinators `verify` reports/candidates/required files then submit actual Attempt IDs.

| Stage | Coordination |
| --- | --- |
| `spec` → `plan` | Verifiable specification/plans/tasks; hand back missing questions/constraints rather than guessing |
| `plan-review` | Human `review`, decision plus exact reviewed plan/tasks copies; submit all three |
| `scaffold` | Approved-version scaffold/independent expectations/tests; read frozen `approval_rules` |
| `implement` → `verify` | One task at a time; fresh independent `verify` executor runs gates and checks candidate/scope/approved version |
| `fix` | Repair actual verification/review findings; select legal return to the corresponding verifier |
| `review` | Uninvolved executor reviews original baseline to candidate, without chat summaries substituting diffs/evidence |
| `deliver` → `retro` | Results/remaining duties/reflection; `retro` reads frozen stats and waits at post-submit gate |

First-line report conventions guide coordinators, who read necessary full evidence and check next. Completed negative reviews submit then back; actual inability to `deliver` fails. See [resume](resume-work.md).

## 4. Escalation and replanning

For authority gaps, exhausted repair budgets, or required escalation, enter `escalate` only if legal next. Give actual reasons/reports to people. Continue/skip/replan/stop decisions must satisfy declared edges/current legality/project authorization; skip cannot fabricate done.

Replanning uses bound previous_plan/previous_tasks/verified prefix. Preserve original overall baseline/real verification rows/report references. Changed acceptance lists rechecks and renewed `plan` review. Old checks do not qualify new candidates; unverified current edits do not enter verified prefixes.

`implement` permits 24 arrivals, `verify` 32; frozen definitions set other limits. Also obey external time/cost/call budgets. Stop on exhausted limits/cancel-only `next`, without increasing max_visits/edges. Resume interruptions without automatic revocation.

## 5. Approve the gate and read results

After `retro` submit expect blocked(gate). Present bound delivery/generated `lessons` to users. Only explicit current-gate approval permits the legal work/node call:

```bash
"$engine_binary" --home "$source_home" --json gate approve "$work_id" --node retro
"$engine_binary" --home "$source_home" --json work result "$work_id"
```

Verify terminal success/final=true/effects_pending=false and delivery/lessons selections. `delivery` is `retro`'s begin-bound input; `lessons` is sealed output. Before approval, results are empty.

Result reading, code quality, acceptance, and external actions remain separate. Delivery push/PR/merge/release commands are not engine-executed; gates do not authorize them. See [editable export](export-results.md).
