# Plan and implement complete behaviors

English | [简体中文](../../zh-CN/how-to/implement-change.md)

For later changes. Preparation, adoption, implementation, experiment, and release are distinct. Proposed packages may be rewritten/reviewed; execute actual tasks only within adopted scope. Active plans alone define progress.

## 1. Scope from user outcomes

1. Describe real users/triggers/current practices/pain/acceptable results. Separate observations from hypotheses.
2. Separate one-time method preparation from per-task inputs and author/user/executor/reviewer costs. Authors may also be users; no distribution platform is required first.
3. Identify real consumers/nonoptional obligations for every mechanism. Current contracts, verifiable near-term needs, or real friction justify work without waiting for incidents or filling capability numbers.
4. Freeze quality/maximum investment/stops before choosing minimal experiments. Outcomes may change instructions only, defer capability, or stop.
5. Keep candidate behavior within its package; after adoption update root specifications/architecture/contracts. Proposed fields are not current interfaces.

Without compatibility needs, design one payload/call path, without migration/dual formats/permanent compatibility. Do not automatically delete/rewrite real data/artifacts. New formats accurately reject old versions and preserve originals.

## 2. Reading and ownership

New readers start with AGENTS/CONTEXT, specs index, active README/spec/design, plan table/task card, then engineering/source entries. Confirm workspace/scope/candidate/dependencies/allowed files/independent oracles.

Implementation assumes junior developers new to Sheltie but familiar with its stack. Capable models/equally experienced authors establish contracts/architecture/formats/trust/oracles. Simpler models/juniors complete bounded bodies/wiring/experiments against frozen interfaces. Capable authors fully implement high-risk process/handle/same-snapshot primitives, without leaving names only.

Each task has one actual Owner. Model descriptions express needed ability, without product/host requirements. Independent capable Reviewers did not author reviewed designs/primitives/code/oracles. For small experiments, capable authors prepare complete methods/tools; juniors run them, without empty scripts handed off for filling.

Capable owners may combine design/implementation with adopted-plan reasons, independent expectations, and stage review. Do not split real behavior into excessive handoffs to lower model tiers or leave architecture to juniors to reduce task counts.

## 3. Stage preparation and handoff

Product work usually separates architecture/primitives/test readiness from complete behavior/real use. Independent capable models review each M task within planned scope. M1 foundations do not qualify public behavior/benefit. Experimental M1 checks protocols/methods/tools/first-reader usability; M2 checks real execution/quality/cost/conclusions without unrelated Rust gates.

Before stages, capable authors:

1. Freeze actual interfaces/data flow/all callers/fields/errors and complete experience-dependent primitives. Fully handle coupling instead of half-compiling with fake success/compatibility.
2. Create narrow scaffolds, all real tests/fixtures/independent expectations. Task-owned unfinished placeholders must stay off normal commands. Preserve correct regressions without forcing all tests red.
3. Assign primitive tests to preparation and pass them. Assign new behavior tests to implementation, temporarily ignore, then task.sh must produce at least one expected behavioral failure and required real-caller failures. Compile/environment/zero tests/placeholder panic alone are not behavioral red.
4. Replace task-card acceptance cases with actual names. Register mixed source in both files/test_files. Create new files/check scope before handoff. Preparation may edit tests; simple implementation uses allow_test_changes=false.
5. stage-1.md/runbook.md/plan-named manuals state commands/parameter sources/cwd/test counts/exits/expected outputs/allowed bodies/stop owner/full scaffold-test commits/raw evidence. Juniors must not guess protocols/approvals/syscalls.
6. Independent M1 verifies preparation closure/usability. Later interface/test changes refreeze revisions and review impact, without silently changing frozen oracles.

Experiments need no Rust placeholders/ignore. Capable authors complete scripts/necessary mechanism tests; juniors use verified tools without changing protocol/fixtures/oracles/source. M1 passes before formal runs; fixtures do not prove value.

## 4. Task cards must support implementation

| Item | Content |
| --- | --- |
| Outcome | Complete behavior observable by users/real callers |
| Prerequisites | Adoption reasons/dependencies/frozen inputs/boundaries |
| Entry | Existing files/symbols/new locations through every real consumer |
| Decisions | Fields/transitions/failure/recovery/provenance/side-effect ordering |
| Oracles | Accepted/single-condition rejected cases independent of implementation |
| Scope | File/test Owners, explicit mixed sources, every interface consumer |
| Validation | Actual commands/obligations/nonzero tests/budget/stops |
| Handoff | Candidate/output/remaining obligations/next executable action |

Proposed paths are not callable interfaces. Describe unimplemented tests as acceptance cases; preparation creates real tests, then implementation cards list actual names on **Tests.** lines and manuals link that list. Product headings use `### Cnnn-Tnn`; check-tests derives ownership from them, not manual names. Python experiments do not invent Rust ownership. Narrow allowlists against real consumers at adoption/start, without deferring necessary callers.

Tasks deliver complete behaviors rather than core/runtime/CLI layers. Contract design may be separate, but partially wired paths/fake success/placeholders/disabled tests must not reach users.

## 5. Implement and check

1. Check dependencies/start candidates and prepare minimal real oracles distinguishing correct behavior. Reproduce repairs first; establish expected feature failures; correct existing cases need no artificial red.
2. Simple implementers complete handed-off bodies/wiring and run/enable tests, without changing signatures/assertions/snapshots/formats/dependencies/policy. Explicit capable repair tasks may add tests, independently review/refreeze. Rust Task comments sit above attributes, without milestone ownership.
3. Select [engineering](../../../specs/engineering.md#23-commands) checks: docs/governance for records, actual consumers for methods/fixtures, full gates for cross-crate/state/protocol/persistence changes.
4. Run `scripts/task.sh <task>` for owned Rust tests and verify nonzero execution. External experiments run card-specified CLI/script oracles without invented Rust tests.
5. Before done, uninvolved Reviewers check affected semantics/consumers/oracles with short feedback then long validation. Review simple records for fidelity/calculation. Record conclusions in package review, without extra milestones/repeated full gates. Retain candidate/closure/commands/exits/counts/raw paths; mark done only after obligations. One task per commit; completion does not authorize push/merge/install/publication.

Pass full stage scaffold/latest independent test-revision commits for simple implementation. Documentation/preparation/review use task-start candidates:

```bash
scripts/check-task.sh "$task_id" "$baseline_commit" --staged
scripts/check-task.sh "$task_id" "$baseline_commit"
```

Set `task_id` to actual active-plan ID and `baseline_commit` to full scaffold/latest independent revision. Without active changes, do not run implementation gates or reuse historical IDs for current tasks.

Preparation/repair tasks use allow_test_changes=true; frozen implementation false. Do not tailor tests to implementation. Contract/expectation changes require explanation/independent review. Reviewers write review/evidence, not repairs. plan/tasks are automatically allowed; other evidence needs explicit allowlists. Tools inspect active packages only; proposal checks do not establish task acceptance.

## 6. Stops, repairs, and workspaces

Stop affected actions for contract conflict, unknown provenance, missing permissions, inadequate APIs, disconnected real callers, or sustained budget overruns. Narrow/repair first, without silent compatibility/second state/unauthorized effects. Difficulty alone is not a stop reason.

Preserve others' edits/originals. Verify or isolate unowned changes. Whole-workspace checkout/reset/clean is not a generic exit. Clean only temporary resources provably owned by this task within established scope.

Juniors retain actual failures for capable authors when interfaces/tests/permissions are inadequate, without whole-workspace rollback. Add necessary repair tasks in the same active package with Owner/scope/oracles and updated plan/tasks. Reviewers assess affected chains without reviewing their own subsequent repairs or asking users to approve ordinary rework individually.

## 7. Stage and complete independent review

Each M task uses a capable reviewer uninvolved in relevant preparation/primitives/implementation. One independent Reviewer may cover stages. Reference same-closure completed task/stage reviews and add only new obligations, without full reruns.

Stable-candidate milestones cover product promises/INV/provenance/state/files/all consumers/failure-recovery/text-JSON/method use/delivery quality. Short semantic feedback precedes long runs. Unchanged inputs require no unaffected rereview.

Mutations target adopted requirements/actual risk. Freeze inventories/consumers/groups/throughput/stops first. Expand unclear impact; not_run/timeout/pause are not caught. Reuse cites original run IDs, not SHA alone. See [budgets](validate-change.md).

plan holds progress; progress handoff; validation shared raw indexes; review conclusions/blockers. Save originals once, without parallel stage ledgers.

## 8. Completion boundaries

Reviewed/structurally valid proposals remain proposed. Experiments conclude only for actual samples/observations. Synthetic fixtures prove mechanisms; agent rehearsals do not prove human first use. Combined trials do not isolate each capability's benefits automatically.

Implementation completion needs every adopted obligation, real candidate, and independent conclusion. Unqualified optional proposals remain proposed. Releases need their own candidates/validation/authorization.
