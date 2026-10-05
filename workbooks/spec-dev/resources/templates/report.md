<First line: exact conclusion from instructions>

Verify and overall review use these location fields. Overall uses Task=overall, plan's overall Baseline/Original baseline, and current HEAD Commit.

Task: <Tnn or overall>
Repair round: <N>
Baseline: <task baseline>
Commit: <current candidate>
Original baseline: <overall baseline>

The following handoff fields/table are verify-only. Overall review omits them and appends no task rows.

Current change: <actually checked change/fix path>
Inheritance source: <current change/fix prefix source>
Verified specification: <spec input path>
Verified plan: <plan input path>
Verified tasks file: <tasks input path>
Approval source: <decision or explicitly qualified correction escalation>
Original approval source: <original decision, correction only>

## Verified tasks

Preserve all old rows verbatim. Append current row only after independent success; none on failure. Paths/hashes only; report ≤32768 bytes.

| Task | Task baseline | Candidate commit | Approval source | Verification report | Raw evidence |
| --- | --- | --- | --- | --- | --- |

## Checks

| Item | Result |
| --- | --- |
| `<command or observation>` | PASS / FAIL |

## Findings

<None or items in this shape.>

1. `<file>:<location>`. Observed: <fact>. Expected: <requirement>. <Blocking / Suggestion>
