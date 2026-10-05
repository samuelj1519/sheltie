Repair only report findings. Implementer rules still forbid signature/test/snapshot/dependency/new-file changes.

## Select report

| From | Findings |
| --- | --- |
| verify | verify_report |
| review | review_report |
| escalate | Human feedback/authorization, then identified verification/review report |

Ignore the stale other report for repair selection.

Read selected Findings and Repair round N; record N+1, reset to1 after escalation. Preserve task baseline verbatim; overall review repairs use original plan baseline. Consult relevant tasks/plan Gates/scaffold command only as needed, then enter project.

## Preserve cumulative handoff

Repair selection differs from cumulative history: inherit verified facts from verify_report or current plan even for review_report repairs. Record original baseline/actual source, validate prefixes, copy unchanged into change. No row deletion/self-certified repair append; later independent verify appends. Missing fields/sources stop.

If targeted change/verify report has correction, carry Approval correction source (actual Approval source in verification) plus Original approval source, irrespective of a different current escalation. Missing references stop.

## Execute

1. Git status clean.
2. Repair each finding, running relevant tests per repair.
3. If a finding should not be fixed, leave code unchanged and justify in Notes for reverification/review.
4. Requests to change tests/signatures block; only authorized scaffold/human may change them.
5. Run/fix every gate.
6. One fix commit under rules; Task is reported task or review for overall review without task number.

## Blockage

Missing human-only facts, high-risk authorization, broken environment, unclear/conflicting findings. Restore only task edits, preserve others, no commit, report Blocked.

## Output

First line exactly Repairs complete or Blocked, then:

```text
Target: Verification report | Review report
Task: Tnn | review
Baseline: <copied task baseline; none when blocked>
Commit: <candidate; none when blocked>
Original baseline: <unchanged plan baseline>
Inheritance source: <actual latest verification/current-plan path>
Repair round: <N+1>
Changed files:
- ...
Gates:
- <command>: PASS | FAIL
Notes: <each disposition: fixed or unfixed with reason; blockage cause>
```

Then unchanged six-column Verified tasks table matching plan/report. References only, total ≤32768 bytes.

## Reply

First line plus target report in one sentence so coordinator returns to verify/review or escalates.
