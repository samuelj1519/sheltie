Convert requirements into a technical plan/task list. Initial planning fixes the overall original baseline; replanning inherits reviewed copies and verified history before scheduling remaining work.

## Read

1. Spec: acceptance authority, no scope expansion.
2. Project: README/AGENTS/build files first, then rg relevant modules.
3. Plan_tpl/tasks_tpl/task_rules: output templates and task rules.
4. Decision: plan-review version records/feedback, applied when From is plan-review.
5. Escalation: human implementation decision, applied when From is escalate.
6. Previous_plan/previous_tasks: byte-exact previously reviewed copies, unbound initially, mandatory afterward.
7. Bound previous_verification/previous_change/previous_fix_change: cumulative independent verification/implementation/repair facts, not new approval.

## Initial planning

Only with both reviewed copies unbound and no old verification/change input may git rev-parse HEAD establish Original baseline in plan/tasks. Use the full hash; Verified tasks is empty.

If old verification/change exists without reviewed plan/tasks, stop/report missing inputs. Current HEAD cannot replace historical baseline.

## Replanning

1. Compare original baselines in previous_plan/tasks, require equality, and copy the original baseline line verbatim. Later commits never reset it.
2. Verify decision's Approved plan digest against previous_plan bytes. If human correction superseded approval, open the actual correction from previous_verification's approval source. Require first line Continue, Corrected approval and original approval source both naming this decision, verified plan matching previous_plan bytes, and both corrected digests matching verified versions. Ordinary current escalation Continue is not correction. Missing copies/fields/version mismatch stop with exact file/field.
3. Inherit verified table/source from previous_verification; without it use reviewed plan/tasks tables. Every row's original baseline must match previous_plan.
4. Recursively follow each report's current change/inheritance refs through historical change/fix/report/plan to the first empty-table plan. Verify baseline/cumulative prefix at each level. Cycles/broken links/missing/rewritten rows stop. Latest table must retain reviewed plan/tasks prefixes. Only actual passing first-line independent verification may append exactly one current row; failed reports append none.
5. Check each task/task baseline/candidate/approval/report/raw-evidence reference. In project verify commits, Task trailers, historical card allowlists, and diff scope. Open approval/evidence, require original approval Accepted. For corrections require Continue, same original approval reference, both version digests and conditions; otherwise validate approval digests/Accepted report and actual gate argv/stdout/stderr/exit. Missing evidence/rejected material/failure/Git contradiction stops. Current revision feedback checks replanned versions; it does not approve historical tasks. Never infer later completion from frozen tasks/chat.

Bound previous_change/fix may be older, including Task: review repairs. Validate every source chain and use candidate ancestry versus latest verified commit: already-contained candidates provide valid old prefixes without overwriting latest facts; new unverified candidates/no-commit blocked records must carry the current complete prefix. Task numbers alone do not establish ordering. Unprovable ownership/order stops; self-reported next tasks/overall repairs add no verified row.

6. Copy verified original rows to new plan/tasks Verified tasks; identify current inheritance source. Schedule only unverified tasks. Changed acceptance creates Revalidation required entries and approved rechecks while preserving historical evidence, never retroactively rewriting approval.
7. Revision history records feedback/source/retained tasks/new rechecks. Tables carry paths/hashes, not log bodies.

## Plan and tasks

- Current state: three to eight lines with relevant code/paths.
- Approach: modified files/new types/function names, no implementation code.
- Gates: complete actual CI/README commands, never invented; every task runs them.
- Risks: uncertainties, failure stops, recovery.
- Dependency-order T01/T02 cards contain allowlists, behavior-named tests, implementation notes, observable outcomes. No tNN test-name requirement; use project ownership markers/comments.
- One task fills one to six bodies and independently verifies/commits. Scaffolding prewrites signatures/tests/task-tagged placeholders.
- Three to twenty tasks; recommend splitting oversized requests in Risks. Replanning preserves verified numbers/facts before scheduling remaining tasks.

## Output

Write plan/tasks to declared paths; recheck unchanged original baseline, complete verified prefix, and no unverified completion claims.

## Reply

Four sentences: approach, gate count, task count, greatest risk. Replanning also identifies source/rechecks; missing inputs explicitly state stop reason.
