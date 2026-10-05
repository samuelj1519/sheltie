---
name: sheltie
description: Coordinate a Work through the sheltie workflow engine when the user asks to follow a Workbook, manage installed Workbooks or Works, resume from current pointers, explicitly revoke an Attempt qualification, query selected final results, or invokes /sheltie. Acquire briefs, delegate workers, and interpret outputs to choose edges; the engine owns state and legal next actions.
---

# Sheltie coordinator

English | [简体中文](SKILL.zh-CN.md)

Sheltie records state, generates briefs, computes legal next actions, and enforces gates without judging content. You coordinate: delegate, read outputs, and choose among engine-provided next actions.

## Rules

1. Follow next for **this Work's progression**. Write responses provide legal operations/target identities; callers supply summaries/failure/replacement reasons. Rejected actions return you to next rather than bypassing it. Discovery/management (workbook list/show/verify/add/remove, work start/list) has separate entry points independent of a particular Work's next. Empty next means terminal Work, not absence of other work.
2. Conclusions live in output documents, usually first lines. Successful execution means the step finished, not accepted content. Read conclusions and choose edges; the engine does not determine acceptance.
3. Access state only through CLI. Do not directly modify management-root files (default ~/.sheltie). Executors write declared outputs to brief paths; sealed outputs are immutable for everyone.
4. Use --json and response next. **For safe retries generate/record --request-id <uuid> before calling and tell the user.** IDs apply only to Work/Workbook writes; reads/self reject them. Uncertain execution may safely retry the same intent/ID, returning commit-time response/replayed=true without duplicate execution. Replay next is historical; always query current status before resume.

## Choose the entry point

- New work requiring method/inputs starts at step1; use already-specified method/inputs directly.
- Resume existing Work at step9, then choose current next. Resume running Attempts through resume, without new Work creation.
- Status/stats/results requests run only corresponding read commands (9/10), never create/submit/advance. If Work ID is unknown, discover through work list and identify target; do not require Workbook/Flow selection for reads.

## Procedure

1. **Choose Workbook.** Inspect installed methods, nodes/edges/host declarations, and ordered start keys:

   ```bash
   sheltie workbook list --json
   sheltie workbook show <id> --json
   ```

   Flow start_inputs lists every required key. Ask only when the user has not chosen Workbook/Flow. A specified but uninstalled method requires stop/report NOT_FOUND, without silent substitution. Default source methods are English; Chinese variants have IDs ending -zh-cn and are selected explicitly.

2. **Start Work.** Obtain keys from show, not failed-start probing. Reuse information already present in conversation/task/files and ask only for missing keys. Supply exactly the full key set:

   ```bash
   sheltie work start --workbook <id> --flow <flow> --name <name> --input <key>=<value> --json
   ```

   Omitted name defaults to Flow ID; @values read files. Response includes work_id/all manifest requires/first next. Later calls may use the unique Work prefix.

3. **Check host resources.** Start/begin/replace and briefs list required skills/named agents/MCP. Confirm installation; missing resources require stop/notify. Neither engine nor coordinator installs them implicitly.

4. **Acquire brief.** Choose a legal begin:

   ```bash
   sheltie attempt begin <work> --node <node> --json
   ```

   Brief_path provides original instructions, bound absolute inputs, and requirements. Next executor/tier directs delegation: agent gets a worker with suitable model; human receives the brief.

5. **Delegate.** Give brief_path to executor. They read/execute/write declared conclusions and reply briefly. Do not write outputs for them or modify files outside instructions.

6. **Submit or fail.** After delivery:

   ```bash
   sheltie attempt submit <work> --attempt <attempt_id> --summary "<brief conclusion>" --json
   ```

   Summary ≤4096 bytes; detail belongs in outputs. Engine checks files/compliance only. Crashed/timed-out/unable-to-deliver executors use:

   ```bash
   sheltie attempt fail <work> --attempt <attempt_id> --reason "<reason>" --json
   ```

   Retry only while next allows begin. A completed rejected review submits successfully; it is not fail.

7. **Choose edge.** Submit may provide multiple begins with main/back/branch/re_review labels. Read conclusions and select one declared target.

8. **Seek gate approval.** Gated blocking permits only approve/cancel. Show status/relevant outputs; execute only after explicit user approval:

   ```bash
   sheltie gate approve <work> --node <node> --json
   ```

   Retry exhaustion/no-legal-edge allows cancel only: report and let the user decide.

9. **Inspect/resume.**

   ```bash
   sheltie work status <work> --json
   sheltie work stats <work> --json
   ```

   Reopened sessions first read current status, then resume current brief/frozen inputs. Draft_outputs are declared locations, not proof of existing/sealed files. Reads do not recover effects_pending; use registered write-request recovery. Continue original running Attempt only after old executor/shared workspace are appropriately handled. Session reopening changes no state and does not itself justify fail/begin.

   Explicitly needed revocation may select replace from current next:

   ```bash
   sheltie attempt replace <work> --attempt <old_attempt_id> --reason "<reason>" --json
   ```

   Before new executor writes, operator confirms the old one stopped or the new environment is isolated. Qualification may be revoked first; unresolved disposal prevents delegation into a shared workspace. These are operator facts; engine stops/isolates/authenticates nothing. Replacement atomically ends old/starts new running, retaining non-statistical frozen inputs/source/instructions/gate. Old drafts become no new results; continue with new paths/brief.

   At most one replacement per Occurrence; REPLACEMENTS_EXHAUSTED leaves current submit/fail available. Replacement consumes no business retries. Number suffix is creation order from zero, not failure count. Reason ≤4096 bytes or @file; same-ID replay rereads no reason source. Old superseded new submit/fail yields ATTEMPT_NOT_RUNNING, or WORK_TERMINAL when terminal. Historical successes still replay; query status before continuation.

10. **Get explicit results.**

    ```bash
    sheltie work result <work> --json
    ```

    Only final=true exposes terminal-selected artifacts, with paths/digests/sizes/source binding/sealing slots. Bound inputs may originate upstream. No selection is explicitly empty; never infer latest files. Success/refs do not establish quality/code candidate/tool checks. Queries list frozen refs; validate originals under actual read contracts. Acceptance/copy/merge/publication follow existing authorization.

## Minimal code method

Code-change fixes implement → review → deliver with review back edges. Authors prepare stages/rules; users supply task goals/acceptance and project location/scope/check prerequisites. Agents investigate/subdivide within stages rather than add nodes per subtask. Default has no gate; adopted new versions declare real authorization boundaries.

Maintain the same management root throughout, including explicit --home after reopening. Development/released formats use separate roots. Preserve original graph/inputs/sealed reports; changed goals require new Work. Content rework submits and chooses explicit edges; execution failures use fail.

## References

See [protocol](../../specs/contracts/protocol.md) for arguments/briefs/errors and [Workbook contract](../../specs/contracts/workbook.md) for authoring. Packaged skill delivery rewrites these references to generated self-contained contracts in both languages.
