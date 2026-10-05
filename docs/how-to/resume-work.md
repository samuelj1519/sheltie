# Choose `resume`, execution failure, content rework, or revocation

English | [简体中文](resume-work.zh-CN.md)

For coordinators/operators reopening sessions, continuing after waits, or replacing executors. Query current Work first, then choose continuation, actual failure, declared back-edge rework, or revocation of formal submission qualification. Actual facts determine choices; the engine does not judge review content.

Replace angle-bracket placeholders with actual values. New trials use unused independent roots; existing Works always use their `original` explicit `--home` after reopening too. Reads omit --request-id. Save UUIDs before writes and retain ID/intent for retries. See [protocol](../../specs/contracts/protocol.md), [status](../reference/data-model.md), and [D-041](../explanation/decisions/D-041-attempt-number-and-replacement.md).

## Query current status first

```bash
sheltie --json --home "<absolute-management-root-for-this-work>" work status "<full-work-id>"
```

Save JSON and read data.status/current/resume/revision/effects_pending/pending_publish and top-level next. Queries neither advance Works, refresh disk cards, nor recover effects. Replay preserves commit-time `next`; fresh `work status` determines current qualification.

`resume`=`null` means no current Attempt; otherwise it supplies attempt/brief_path/frozen inputs/draft_outputs. Only running includes draft locations, which prove neither existence/completeness/sealing. Non-null `resume.inputs` are references with path/sha256/bytes; read path. Unbound optional `inputs` are null. Read brief/inputs before continuing; newest-modified files do not establish versions.

## Choose from actual triggers

| Situation | Choice | Observations/stops |
| --- | --- | --- |
| Interruption/wait, Attempt still running, operator confirmed old executor/workspace handling | Resume the same Attempt, without new Work/begin | Check identity/brief/frozen inputs/drafts; stop delegation for unconfirmed identity/handling, without fabricated fail/replace |
| Actual crash, timeout, or inability to deliver declared `outputs` | Call `attempt fail` if current `next` permits, with truthful reason | Check data.attempt/work_status/next; retry only if allowed. Stop at `blocked(retries_exhausted)`, without changing limits |
| Review completed and content `requires` rework | Submit completed review, then select declared edge=back from `next` | Negative content verdict still succeeds operationally; target must be legal. Stop without a legal back edge, without execution fail |
| Formal qualification of old running Attempt must be revoked | Select `attempt replace` from `next` with actual operator reason | Revocation differs from ordinary resume. Before successor writes, confirm old process stopped/environment isolated; engine does neither, nor authenticates successors |

`node#occurrence.number` uses consecutive creation numbers from 0, not failure counts. Only `failed` counts; `superseded` does not consume max_retries. `max_retries`=k permits k retries in one Occurrence; real failure k+1 blocks with retries_exhausted.

## Record real execution failure

Requires current lawful running qualification and actual failure. Reasons ≤ 4096 bytes. Do not fabricate failures to test commands or switch actors.

```bash
sheltie --json --home "<absolute-management-root-for-this-work>" --request-id "<saved-uuid>" attempt fail "<full-work-id>" --attempt "<current-attempt-id>" --reason "<actual-execution-failure>"
```

Check ok/structured codes, then Attempt/Work state. Natural-language errors cannot determine commit status; retain complete responses. Retry only from successful legal `next`; after replay query status again.

## Rework after a content review

The reviewer completes required review/output, writes the conclusion to declared output, then submits. Content conclusions and execution states remain separate.

```bash
sheltie --json --home "<absolute-management-root-for-this-work>" --request-id "<saved-submit-uuid>" attempt submit "<full-work-id>" --attempt "<review-attempt-id>" --summary "<actual-review-summary>"
```

Select a back edge only if current post-submit `next` includes its target with edge=back. Use args.work/args.node/executor. Returning creates a new Occurrence under `max_visits`, without failure retry within the old Attempt.

```bash
sheltie --json --home "<absolute-management-root-for-this-work>" --request-id "<different-saved-begin-uuid>" attempt begin "<next.args.work>" --node "<back-item-next.args.node>"
```

Handoff new brief_path/inputs/outputs/requires. Missing required host resources stop delegation; engine/coordinator must not install independently. Human executors go to people. If submission blocks on gate, stop for explicit user approval before following `next`; do not bypass gates to rework.

## Replace only when revocation is needed

Use replacement when operators must revoke old running qualification. Reopening/waiting/content rework does not automatically qualify. Revocation may happen first, but successor workspace writes still require confirmed old-process stop/isolation. Stop delegation if unconfirmed. Replacement does not stop/authenticate/isolate or undo external effects.

1. Query status. New replacement requires active Work, current Occurrence's latest running Attempt, and matching `attempt replace` in next. One per Occurrence; new request IDs do not reset limits.
2. Save UUID/full command and call with the actual reason:

   ```bash
   sheltie --json --home "<absolute-management-root-for-this-work>" --request-id "<saved-replace-uuid>" attempt replace "<next.args.work>" --attempt "<replace-item-next.args.attempt>" --reason "<actual-revocation-reason>"
   ```

   Reasons ≤ 4096 `bytes`; --reason "@<absolute-reason-file>" is allowed. Replay retains original target/literal reason/source `path` and does not reread committed files. Changed targets/reasons return `REQUEST_CONFLICT`, without hiding original outcomes.
3. Verify `data.replaced_attempt` is old, `data.attempt` is new in the same Occurrence, `data.number`=old+1. Check node/occurrence/brief_path/output_dir/inputs/outputs/requires and top-level revision/request_id/next. One transaction supersedes old and starts running new, increasing `revision` once. Retain structured failures and determine commit status below, without inventing Attempts.
4. Handoff new brief/output locations. Preserve entered_from/frozen non-engine.stats `inputs`, including `null` optionals, without reselecting latest upstream. Engine verifies old reference sizes/digests before replacement commit. Old unsubmitted drafts become neither new `inputs` nor outputs. Frozen Work instructions/gates/nodes/result criteria remain.

New `engine.stats` uses committed state including the new Attempt, with exact recorded `bytes`, without copying old stats or replay recomputation. attempts/superseded increase by 1; failed/visits do not. Query current statistics read-only:

```bash
sheltie --json --home "<absolute-management-root-for-this-work>" work stats "<full-work-id>"
```

`REPLACEMENTS_EXHAUSTED` adds no blocked state; current Attempt retains qualification for submit/real fail/cancel under next. [Protocol §3](../../specs/contracts/protocol.md#3-operation-details) defines precedence: nonterminal Work, old identity, running/latest active qualification, then limit/reason/inputs. New submit/fail for `superseded` Attempts returns `ATTEMPT_NOT_RUNNING` in nonterminal Works; succeeded/cancelled first returns WORK_TERMINAL. Same-ID `original` success replays before new-request checks, retaining history even after successors end. Query fresh status after replay.

## Incomplete file effects and unknown outcomes

`EFFECT_PENDING` means incomplete effects, without `executor` content failure. Retain `original` command/root/full stdout JSON/stderr/exit/request ID. Use structured [protocol §5](../../specs/contracts/protocol.md) and [recovery contract](../../specs/contracts/storage.md#3-crash-semantics-and-recovery):

| Response | Confirmed fact | Recovery/stops |
| --- | --- | --- |
| `EFFECT_PENDING`, committed=true | This request committed; own effects unfinished. Verified snapshot may appear as `original` and Work `revision` | Reissue same ID/target/user arguments. Committed replace recovers the same new Attempt/brief/stats bytes. `ok`=false does not justify new IDs/replacement |
| `EFFECT_PENDING`, committed=false, this request B, pending_request_id=A | B uncommitted, blocked by A; `pending_original` belongs to A | Recover A from actual `original` command/ID after fixing effects; then retry B's `original` ID/intent. A's snapshot is not B success; no fabricated B execution failure |
| Missing trusted complete JSON or unmatched commit/command identity | Outcome unknown; timeout/disconnect/text do not prove no commit | Preserve state/records, query status. With `original` UUID/full command, same-intent replay obtains facts. Stop/report for missing recovery records, contradictory fields, or unverifiable identity; no guessed new-ID rerun |

original/pending_original/revision may be omitted if snapshots cannot be independently verified; absence does not negate explicit commit identity. Inspect `error.detail.cause` and actual paths/system errors, without inferring business state from message/stderr. Missing/modified/unowned originals or unresolved integrity stop recovery for human inspection. No Store edits, pending deletion, fabricated originals, or overwrites to obtain success.

Status pending flags are same-snapshot hints. Queries neither recover nor advance. Registered writes recover through normal CLI. Query-only requests report pending facts and stop, without incidental writes. Successful replay still has historical `next`; query fresh status before action.

## Query, human, and terminal stops

- For status/stats/results only, query and finish. Known root/unknown ID permits `work list` to find full IDs, without new Works or reselecting methods.

  ```bash
  sheltie --json --home "<absolute-root-for-existing-work>" work list
  sheltie --json --home "<absolute-management-root-for-this-work>" work result "<full-work-id>"
  ```

- Non-null `resume` does not prove running. Check Work state/next identity before drafts. Unverifiable state stays unknown and stops progression, without directory/suffix/history guesses.
- begin `next` includes executor/tier. human goes to people; `tier` only labels model selection. begin/replace/brief `requires` are host declarations. Missing required resources stop/report without installation.
- `blocked(gate)` permits current approve/cancel only. Show outputs/state to users and obtain explicit approval before writing. Replacement/retry/rework cannot bypass it. by records the OS process account, without independent human authentication.

  ```bash
  sheltie --json --home "<absolute-management-root-for-this-work>" --request-id "<saved-approve-uuid>" gate approve "<next.args.work>" --node "<approve-item-next.args.node>"
  ```

  Check data.node/occurrence/by/at/work_status/next; after replay query status.
- retries_exhausted/no_legal_edge prohibit begin/replace/approve. Stop/report; only cancellation remains for user decision. Do not expand limits.
- succeeded/cancelled are terminal with `next`=[]. Stop progression; queries remain. final=true lists explicit `artifacts`; unqualified terminal/gates/effects give final=false/empty. Success without declarations may be final=true/empty. Newest files are not results; flow success/sealed references do not prove quality/candidate checks/acceptance/release authorization.
