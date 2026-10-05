A person decides whether requirements/plan may proceed and preserves the actually reviewed plan/task list. Agents implement/verify/review/repair approved versions; escalation/final approval remain human.

## Read

1. Spec: Goal/Acceptance and every Open question, answering each.
2. Plan: approach/direction and actual gate commands.
3. Tasks: sample two or three for small body-filling scope and tests whose names reveal behavior.

Review direction rather than every word; agents investigate detailed errors later.

## Decision

First line of decision.md must be exactly one:

- Accepted: begin strong-model scaffolding, then task implementation.
- Revise specification: return to spec.
- Revise plan: return to plan.

Every decision then records these exact fields:

```text
Approved specification: <sha256>
Approved plan: <sha256>
```

Hash brief-bound spec/plan with shasum -a 256 (Linux sha256sum). Accepted binds approval; scaffold/implement/verify recheck, and later edits invalidate old approval. Revision decisions identify returned versions for reconciliation at next review.

Then one located comment per line, plus open-question answers. Accepted may include conditions enforced during implementation without asking again.

## Preserve reviewed copies and submit

1. Byte-copy plan to reviewed-plan and tasks to reviewed-tasks for every decision, including rejection. No reformatting/annotations/regeneration.
2. Compare each input/output sha256. Each copy ≤65536 bytes; comments belong only in decision.
3. Write decision with version records from actual brief-bound spec/plan for every outcome.
4. After all three outputs exist/validate, run the brief's submit. Planner inherits reviewed copies, not chat-only old plans.
