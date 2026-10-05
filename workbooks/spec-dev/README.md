# spec-dev: specification-driven software development

English | [简体中文](../spec-dev-zh-CN/README.md)

Turn a request into verified, committed, reviewed small changes. People approve specification/plan at the start and delivery/external actions at the end. Agents handle the middle, escalating only when blocked.

Default English: spec-dev@0.2.3. Maintained Chinese: spec-dev-zh-cn@0.2.2. Installable identities differ; existing Works keep their frozen methods. The Chinese variant preserves instruction language while following the current English commit policy; it is not byte-identical historical evidence.

## Flow

```text
spec → plan → plan-review(human) → scaffold → implement → verify → review → deliver → retro(gate)
  ↑       ↑       │                            ↑          │         │
  └───────┴─back──┘                            └── main ───┘         │
                                                         branch     │ branch
                                                            ↓       ↓
                                                            fix ──re_review→ verify/review
scaffold/implement/fix/verify ──branch→ escalate(human)
escalate: Continue → blocked stage; Skip → implement; Revise plan → plan; Stop losses → deliver
```

Eleven nodes, twenty-five explicit edges. Mandatory human participation: plan-review and retro gate (delivery plus reflection); escalate is conditional. Scaffold/implement/verify bind shared approval_rules frozen with the Work; ordinary approval checks still occur before execution.

| Step | Executor | Output | Responsibility |
| --- | --- | --- | --- |
| spec | Strong model | spec.md | Verifiable requirements, recommended answers |
| plan | Strong model | plan.md/tasks.md | Technical approach, gates, three to twenty bounded tasks |
| plan-review | Person | decision.md/byte-exact reviewed plan/tasks | Accepted / Revise specification / Revise plan; all three outputs required |
| scaffold | Strong model | scaffold.md/skeleton commit | Types/signatures/comments/placeholders/disabled tests; final design stage |
| implement | Standard model | change.md | One task: enable/fill/check/commit; Complete Tnn / Blocked Tnn |
| verify | Fresh standard model | report.md | Independently run focused tests/gates, check task baseline/candidate/scope/placeholders/approval |
| fix | Standard model | change.md | Repair findings only, one commit; Repairs complete / Blocked |
| escalate | Person | decision.md | Blockage/two unsuccessful repairs/authorization; Continue / Skip / Revise plan / Stop losses |
| review | Fresh strong model | report.md | Original baseline..HEAD overall review, installed mutation tools; Accepted / Rejected |
| deliver | Standard model | delivery.md | Results/checks/remaining/external commands |
| retro | Standard model/gate | lessons.md | Stats/report-backed method proposals; human approval completes Work |

Tier appears in brief/next. Strong uses the most capable model; standard supports economical models. Four strong nodes: spec/plan/scaffold/review. Escalate is human; other agent nodes standard.

## Coordinator routing

Report **first lines** govern coordinator choices, never engine inference:

| Current | First line | Next |
| --- | --- | --- |
| plan-review | Accepted | scaffold |
| plan-review | Revise specification / Revise plan | spec / plan |
| scaffold | Complete / Blocked | implement / escalate |
| implement | Complete Tnn / Blocked Tnn | verify / escalate |
| verify | Accepted, next task Tnn | implement |
| verify | Accepted, all tasks complete | review |
| verify | Rejected / Rejected, needs human | fix / escalate |
| fix | Repairs complete, verification target | verify |
| fix | Repairs complete, review target | review |
| fix | Blocked | escalate |
| escalate | Continue | Actual blocked scaffold/implement/fix/verify; identify predecessor via status done's second-last entry |
| escalate | Skip / Revise plan / Stop losses | implement / plan / deliver |
| review | Accepted / Rejected | deliver / fix |
| deliver | Delivery written | retro |
| retro | blocked(gate) after submit | Notify user to read delivery/lessons and approve |

Verify/review require **fresh workers**, not implementer sessions. At escalation report who blocked, concrete Notes/Findings, and decision output path; do not decide for the user. N tasks require N implement/verify arrivals; limits24/32 bound scope, so split larger requests.

## Replanning handoff

Plan-review always preserves reviewed plan/tasks bytes. Later plan briefs bind previous_plan/tasks plus available previous_verification/change/fix_change. A fresh worker recovers original baseline/verified prefix from these paths and project Git.

Implementation/repair copy verified rows unchanged. Independent verify checks Git, approved versions, gates, and raw evidence before appending current task. Planner recursively validates references; missing fields/rows or changed baseline stop. Old tasks/chat cannot establish later completion. Unverified current changes carry only the old verified prefix.

Replanning preserves original baseline/history. Changed acceptance requires explicit revalidation adopted by new plan-review. Tables store path/hash references rather than full reports. Behavior-named tests use separate task ownership, no mandatory tNN prefix.

Regress methods with separate roots. Same installed ID/version is never overwritten; existing Works continue frozen graphs/instructions.

## User procedure

```bash
sheltie workbook add workbooks/spec-dev
sheltie work start --workbook spec-dev --flow default \
  --name "Export CSV" \
  --input request="Add a report-page CSV export button for current filtered results" \
  --input project=/abs/path/to/repo
```

Invoke /sheltie in Claude Code and select this Work. Replanning repeats plan approval; escalation adds conditional participation:

1. Plan-review: read spec/plan/tasks, byte-copy reviewed plan/tasks and compare digests, then decision. Accept/reject both submit three outputs.
2. Escalate: only blockage/two failed repairs/authorization. Read report, write one exact choice with comments and bounded authority.
3. After retro: read delivery; perform authorized external push/PR actions, then `sheltie gate approve <work> --node retro`, or cancel if unsatisfied. Manual edits may precede approval. Read lessons and adopt into a new version, recording accepted/rejected Ln and reasons.

The flow itself never pushes, opens PRs, or releases. Project CI runs after push outside this method. Copy real CI commands into plan Gates to align checks.

## Design rationale

- Human direction at start, acceptance/authorization at end. Intermediate gates/criteria/checklists support agent decisions; high-risk/irreversible choices stay human, consistent with OpenAI/Anthropic guidance.
- Reflection proposes only. Humans choose lessons, create new versions/add them; engine never changes limits/tiers/skips nodes. A version behaves consistently across runs.
- CI is external fresh-environment confirmation, not acceptance. In-flow gates are run at implementation, verification, and review; plan uses actual CI commands.
- Strong-model scaffolding concentrates types/interfaces/comments/tests. Implementers fill defined bodies; prewritten expectations and exhaustive typing reduce junior-model uncertainty.
- One task keeps context bounded (roughly3k–8k tokens, a few files/tests), limiting failures to one task.
- Compiler/tests/gates check each task; verify mechanically checks per-task baseline/candidate, allowlists, unchanged expectations, and current placeholders. Overall review examines original baseline..HEAD once, with optional mutations.
- Tier guidance reserves strongest models for design/overall review; economical models handle repeated bounded tasks.
- Independent verification runs commands rather than trusts implementation reports. Final review checks the whole rather than repeats every task's model review.
- First-line conclusions let coordinators route quickly; repair workers act from Findings.
- Missing facts/authorization/environment/two failed repairs escalate, without forced work/bypass. Human decisions are files passed as inputs. Repair round travels through reports.
- From selects current process inputs; optional values can retain older content. Decision is the exception: every relevant node checks approved digests regardless of From. Escalation is current only when From=escalate; qualified correction refs remain separately authoritative.
- Plan fixes gate commands instead of each implementer guessing them.
- Files carry specs/plans/tasks/reports; briefs bind paths without pasting full documents into conversations.

## Explicit results

After final approval `sheltie work result <work> --json` yields selected delivery input and lessons output bound/sealed by the particular terminal retro Attempt. Before approval selection is empty. Reading results does not execute external commands.

## English instruction effect evaluation

1. Use equivalent fresh authorized code tasks and independent temporary Homes; install default English and explicit Chinese variants. Predeclare acceptance, executor/model allocation, repair/approval limits, observable usage/cost collection, and independent final quality checks.
2. Run English end-to-end including plan approval, one implementation/verification/repair path as applicable, overall review, reflection, and explicit final gate. Capture generated brief language, exact first-line routing, approved digest checks, preserved verified prefixes, and final result provenance.
3. Run matched Chinese tasks independently, including Chinese prose with English future commits. Do not rerun already-solved tasks to estimate learning-adjusted benefit.
4. Compare first-use comprehension, human questions/interventions, wrong routes, omitted requirements, repair rounds, duration, observable usage/actual charges, and independent final quality. Missing usage is null; failed/unacceptable tasks still count costs.
5. Record actual command/run IDs and raw evidence in the adopted change package. Parsing/CLI tests establish mechanical compatibility; unexecuted model/host evaluation remains not_run. No comparative language-effect claim is established by this checklist.

## Revision history

- 0.2.3: English default instructions/resources and English commit policy; maintained Chinese ID spec-dev-zh-cn@0.2.2 installs separately. Existing frozen Works stay unchanged. Effect-evaluation steps added; actual outcomes recorded separately.
- 0.2.2: Three real nodes bind shared correction rules; terminal delivery/lessons available after approval. Existing0.2.1 copies unchanged.
- 0.2.1: C002 host regression corrected retrospective history paths to occurrence-<NNN>/attempt-<NNN>/outputs/ with padded occurrence/then-retry labels. Historical Work copies retain original bytes.
- 0.2.0: C002 O09/O10/N08 closure repairs: per-task baseline/candidate verification, task-tagged placeholders, digest-bound approvals checked by scaffold/implement/verify, unchanged Work original baseline across replans. Adopted fixes, not retro proposals.
- 0.1.0: Initial version.

Language links select repository source variants. Installed/frozen instructions and resources use the chosen language; sibling language directories are not included in that copy.
