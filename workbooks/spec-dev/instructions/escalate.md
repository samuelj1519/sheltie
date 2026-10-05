A person decides when an agent blocks, two repairs fail, or authorization is required.

## Read

From identifies blocked scaffold/implement/fix/verify; use corresponding report, not stale others.

1. Scaffold_report/implement_change/fix_change: Blocked first line, Notes state missing facts/authority. Scaffold conflicts/untestable acceptance usually require Revise plan.
2. Verify_report: Rejected, needs human points to persistent Findings.
3. Tasks/plan as needed.

Usually five minutes. You need not repair code; if you do, commit it and describe it in the decision.

## Decision

First line exactly one:

- Continue: return to blocked stage with hints; missing facts, corrected approach, human code edits, or corrected approval.
- Skip: abandon current task; second line Next task Tnn; low-priority/deferred work.
- Revise plan: return to plan/tasks for faulty decomposition/unworkable approach.
- Stop losses: stop implementation and deliver completed portion when further cost is unwarranted.

Then concrete comments, one per line: facts, authorized actions (install xyz/delete legacy), human edits. These pass verbatim to the next brief.

## Correct approval

Choose Continue, record Corrected approval: <original-decision-path>, Approved specification: <current-spec-sha256>, Approved plan: <current-plan-sha256>. Original path comes from approval input; hash bound spec/plan. Original first line must be Accepted. Preserve conditions; explicitly identify changes. Ordinary Continue/verbal agreement is not version correction. Downstream reports reference both files for replanning.

## Authorization

State exact scope, e.g. allow modifying .github/workflows/ci.yml to add one step. “Anything you want” is not scope. Without authorization choose Skip/Stop losses.

## Output

Write decision path and run brief's submit command.
