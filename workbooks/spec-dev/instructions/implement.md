Complete **one** task: enable designed tests, fill placeholder bodies, make tests/gates pass, commit once. Types/signatures/comments/tests already exist. Read rules first; they are mandatory.

## Select task

Use the brief's From node:

| From | Action |
| --- | --- |
| scaffold | Derive verified prefix from current plan/tasks; choose first unverified task, T01 for initial empty prefix |
| verify | Follow report's first-line next task and previous-task reminders |
| escalate | Read escalation: Continue retries blocked task with human notes/authority; Skip follows second-line task |

Stale escalation/report does not select current work or replace human decisions. Cumulative history still follows handoff rules; never lose prefixes because a report came from another predecessor.

## Read

1. Approval_rules for correction/inheritance; ordinary approval checks below.
2. Decision's Approved specification/plan digests against bound files (shasum -a 256, Linux sha256sum). Mismatch invalidates approval: Blocked with hashes, no unapproved work. Human escalation correction may authorize; record it. Apply Accepted conditions. Read spec only to hash it.
3. Your task card only; note allowlisted files and tests.
4. Scaffold's focused command/current task reminders.
5. Allowlisted source; doc comments define behavior.
6. Named tests as expectations.
7. Relevant plan Approach lines only if tests/comments unclear, plus Gates.
8. Project root.

No other implementation reading.

## Inherit verified tasks

Before work read plan/tasks original baseline/tables. Any bound report must validate baseline/evidence and supplies latest independently verified cumulative facts, including return through Continue/Skip. Plan/tasks may lag only as unchanged prefixes. Replanned scaffold checks retained history. Without report use only provenance-proved plan/tasks. Missing fields/rows/trust or unequal baselines block; never fall back to older empty tables.

Change records original baseline, actual inheritance path, and full old table even when blocked. Current commit remains unverified; only later independent verify appends after Git/approval/raw-evidence checks.

## Execute

1. Git status clean; otherwise Blocked without edits.
2. Remove current test disable markers and run focused tests; establish all red. Zero tests/compilation failure are not red; first make tests runnable.
3. One red test at a time: read assertion, fill body, run, then next.
4. Run/fix every gate after green.
5. Record precommit git rev-parse HEAD as task Baseline. Stage allowlisted files plus tests with removed disable markers; commit under rules.
6. Record resulting HEAD as Commit candidate.

## Blockage

Rules item9 plus human-only missing facts, out-of-task high-risk actions (dependencies/CI/directory deletion/network/database/other-module interfaces), or broken environment. Restore only your task edits, preserve others' work, do not commit, report Blocked. Difficulty alone is not blockage.

## Approval correction

Read bound approval_rules and enforce qualification/references/conditions/stops.

## Output

First line exactly Complete Tnn or Blocked Tnn, followed by:

```text
Baseline: <task baseline, none when blocked>
Commit: <candidate, none when blocked>
Original baseline: <unchanged plan baseline>
Inheritance source: <latest report or current plan; plan on first arrival>
Repair round: 0
Enabled tests: <count>
Changed files:
- path/a.rs
Gates:
- <command>: PASS | FAIL (one-line reason)
Notes: <blockage/required facts/authorization; optional when complete>
```

Then Verified tasks columns Task/Task baseline/Candidate commit/Approval source/Verification report/Raw evidence. Copy old rows verbatim; no current task append; initial table empty. References only, no logs; total change ≤32768 bytes.

## Reply

Repeat first line in one sentence, with reason when blocked. No “mostly complete.”
