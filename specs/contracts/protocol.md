# Public operations, status cards, and errors

English | [简体中文](protocol.zh-CN.md)

This contract defines every engine operation available to coordinators and people. MVP exposes CLI only; a future MCP interface is a thin wrapper over the same operations ([roadmap](../roadmap.md)). Response format is `cli-result/v4`: every Work/Workbook write-response field comes from its commit-time snapshot; CLI must not reread postcommit state to assemble data.

## 1. Global conventions

```text
sheltie [--json] [--home <dir>] <group> <verb> [args]
```

| Flag | Meaning |
| --- | --- |
| `--json` | One JSON line on stdout (§5). Maintenance warnings go separately to stderr without changing business JSON. Otherwise render human-readable text |
| `--home <dir>` | Override management root; otherwise SHELTIE_HOME, then ~/.sheltie |
| `--request-id <uuid>` | Optional only for Work/Workbook writes. Generated and returned when omitted. Coordinators must record an ID before calling when safe retry is needed. Read-only operations and all self commands reject it with INVALID_REQUEST/exit 2; queries must not invent IDs |

**Principal.** Use the invoking process's actual OS account (Unix effective uid's account name; `uid:<number>` if unavailable or non-UTF-8) in audit/approval. Do not trust USER/USERNAME environment variables. A shared OS account does not provide independent human authentication (constitution §5).

**Read-only operations** (list, show, status, result, stats, verify, self version) do not change business state, create Home/engine .lock, or refresh cards. WAL queries may maintain an existing Store's shared-memory control file or create a missing zero-byte WAL carrier under D-039. Deterministic start rejection creates no Home (GF-30). Add source structure/type/limit coarse-check failure creates no Home. After coarse checks, validate content only in the locked private copy. Failure must not register business rows, requests, audit, or final Workbook; an empty schema-4 control Store, lock, and owned uncommitted pending may remain.

## 2. Operations

| Command | Writes | Purpose |
| --- | --- | --- |
| `self install` | Yes | Install running binary at bin/sheltie; create root; no shell configuration |
| `self update [--version <v>]` | Yes | Download, verify, atomically replace |
| `self rollback` | Yes | Restore previous version |
| `self uninstall [--purge]` | Yes | Default removes bin/; confirmed purge removes user data/binary, retaining empty root and the same .lock |
| `self version` | No | Version, platform, root, SCHEMA_VERSION |
| `workbook add <dir>` | Yes | Validate/copy Workbook into root |
| `workbook list` | No | Installed IDs, versions, names, highest version per ID; unpublished entries have pending_publish |
| `workbook show <id>[@<version>]` | No | Manifest, host declarations, Flow nodes/edges/ordered start keys |
| `workbook remove <id>@<version>` | Yes | Remove version; reject nonterminal references |
| `workbook verify [<id>@<version>]` | No | Compare directory digests with records |
| `work start --workbook <id>[@<version>] --flow <flow> [--name <n>] [--input k=v]...` | Yes | Create Work and frozen Workbook copy |
| `work list` | No | Work ID/name/status/current node/update time |
| `work status <work>` | No | Compact state and current Attempt resume pointers, text or JSON |
| `work result <work>` | No | Explicit terminal result references and delivery readiness |
| `work stats <work>` | No | Arrival/attempt/failure counts, mean duration, arrival sources |
| `work cancel <work>` | Yes | Cancel |
| `attempt begin <work> --node <node>` | Yes | Enter node/start Attempt; return brief |
| `attempt submit <work> --attempt <id> --summary <text\|@file>` | Yes | Validate/seal outputs and submit |
| `attempt fail <work> --attempt <id> --reason <text>` | Yes | Record real execution failure |
| `attempt replace <work> --attempt <id> --reason <text\|@file>` | Yes | Atomically revoke old qualification/start replacement; at most once per Occurrence |
| `gate approve <work> --node <node>` | Yes | Human gate approval |

Work accepts a full ID (2026-09-24-001-article) or unique prefix (2026-09-24-001). Ambiguous new-request prefixes return INVALID_REQUEST with candidates. Historical request replay first checks the original requests.work_id; later same-prefix Works do not alter it. Input values beginning @ read files. Omitted Workbook versions select the highest installed version by literal ordering.

## 3. Operation details

### `self` group

Install copies std::env::current_exe() to bin/sheltie, creating root/parents before store.db. Identical existing bytes return data.already_installed=true ([storage §9](storage.md); [MVP D-31](../../docs/explanation/decisions/mvp.md)). Print one hint to add ~/.sheltie/bin to PATH; never write shell configuration (INV-3).

Update fixes release identity first: omitted version selects latest; explicit version pins tag v<v>; manifest/assets come from that same tag. Follow storage §9: download to tmp/, verify sha256, move sheltie to sheltie.prev, rename new file into place. Missing platform/version release returns UPDATE_UNAVAILABLE. Digest mismatch returns UPDATE_CHECKSUM_MISMATCH and deletes download. Success returns {from,to}; already latest returns data.up_to_date=true.

Rollback requires sheltie.prev or returns NOT_FOUND. Retain only one prior version.

Uninstall defaults to bin/ only and reports retained Store/workbooks/works. Explicitly confirmed purge deletes all user data/binary, retaining the empty root and original .lock. List the deletion scope first; text mode requires yes, JSON mode --yes. Failures report partial cleanup; repeated purge may continue.

Version is read-only and does not require an existing root.

### `workbook add <dir>`

Construct intent from the source argument and look up request-id before reading the current source. Only new requests validate/copy under [Workbook §1](workbook.md#1-directory). Register final-copy identity/digest. Return {id,version,digest,flows:[...],requires:[...]}; digest uses workbook-digest/v2 ([storage §5.1](storage.md)). Request replay follows GF-15.

Committed unpublished add remains readable through list/show/verify from the same protected pending original ([storage §3.3](storage.md)). List entry, show data, and verify result carry pending_publish=true; text says pending publication. A matching original digest yields verify status=ok, not missing merely because final directory is absent. The next write recovers pending remove before continuing. Read-only queries do not redisplay versions whose rows were removed.

### `workbook remove <id>@<version>`

1. Require an explicit version; no highest-version default.
2. Check references, delete row, audit, and register request in one transaction (storage §5.2). Validate every works.state_json and redundant column, then find nonterminal references from validated state. References yield WORKBOOK_IN_USE/detail.works. Corrupt rows yield STORE_CORRUPT; do not prefilter by redundant status and skip validation. Terminal Works retain frozen copies and do not block removal.
3. After commit, move the ownership-verified directory to this operation's pending and delete it. Recover failures through effects rather than silence. Return {id,version,replayed}; request-id replay is supported.

### `workbook show <id>[@<version>]`

Include manifest, host declarations, Flow nodes/edges, and ordered start_inputs: all start.<key> references in first-appearance node-declaration order. Coordinators get every required start key without probing through failed starts. Text/JSON both include them; flows[].start_inputs is a string array.

### `workbook verify [<id>@<version>]`

Omission verifies every installed version. Recompute each digest and compare workbooks.digest; report ok/tampered/missing. Any non-ok yields ok=false, WORKBOOK_TAMPERED, and the complete table in detail.results.

### `work start`

Perform deterministic **preflight without business side effects** (GF-30). For committed request-id hits, compare intent under storage §2.1 before current Workbook or @file reads. Equal intent recovers necessary existing effects and returns the original response without resolving latest again. Only misses continue deterministic validation. Failure allocates no daily sequence, creates no final directory/request, and changes no existing main database/WAL/business bytes. Under D-039 read-only SQLite may maintain existing store.db-shm or create missing zero-byte store.db-wal; this never creates root, .lock, or database.

1. Parse input keys and literal values or lexical @file paths. Construct RequestIntent from raw name value and omission state, without WorkName normalization, file reads, or Workbook loading. Omission differs from explicit flow name (storage §2.1).
2. Read-only identify existing Store/request-id. Same intent replays original snapshot; different intent returns REQUEST_CONFLICT. Recovery failures return EFFECT_PENDING (§5). Continue only for new requests.
3. Normalize new WorkName only: default to flow ID, trim, collapse consecutive whitespace into one hyphen, lowercase. Require lowercase letters, digits, accepted Han, and single hyphens, no leading/trailing/consecutive hyphens, ≤48 bytes; otherwise INVALID_REQUEST. Accepted Han is exactly: 3400–4DBF (A), 4E00–9FFF (basic), F900–FAFF (compatibility), 20000–2A6DF (B), 2A700–2B73F (C), 2B740–2B81F (D), 2B820–2CEAF (E), 2CEB0–2EBEF (F), 2EBF0–2EE5F (I), 2F800–2FA1F (compatibility supplement), 30000–3134F (G), 31350–323AF (H). Reject other Han-script points, including radicals, Kangxi radicals, and U+3007.
4. Resolve installed Workbook, load/compile, confirm Flow. Protected pending add originals are readable under storage §3.3, then recovered/rechecked under lock. Return NOT_FOUND only for actual missing Workbook/Flow.
5. Require all and only Flow start keys. Core start_requirements provides the same set/order as show.start_inputs. Read @file contents only after key validation (read failure exit 2). In the write path reload definitions/recheck keys under lock and reuse exactly the preflight-read contents.

Then enter [storage §2.3](storage.md):

6. Allocate <UTC YYYY-MM-DD>-<daily 001..999>-<name> in a SQLite transaction (storage §7). The 1000th daily Work returns INVALID_REQUEST. Later failures may leave gaps; the first five steps do not consume numbers.
7. Prepare pending/<internal-id>/payload/: copy/reverify Workbook digest (storage §5.4), make it read-only, create start-inputs/<key> value files and ArtifactRefs. Later operations use this frozen definition, never workbooks/.
8. Set current=entry#1/status=active; after COMMIT rename payload to works/<work_id>/ and refresh card.
9. Return {work_id,name,workbook:{id,version,digest},flow,work_dir,requires:[...]} and next entirely from commit snapshot. Requires includes every host declaration in manifest order, {kind,name,version,digest,source}; omitted fields are null; digest is bare 64-digit hex without sha256:. Coordinators check readiness; MVP does not inspect hosts.

English example: `sheltie work start --workbook article-review --flow default --name "Article Draft"` produces `2026-09-24-003-article-draft`. Han names remain valid under the exact ranges above.

### `attempt begin <work> --node <node>`

1. Node must be in current next; otherwise ILLEGAL_NEXT includes current next.
2. For a different current node, enter along the edge, increment visits, set node#n, and record source Occurrence/edge kind. Retries retain the previous source.
3. Bind input sources, recomputing sha256 against recorded values; mismatch yields ARTIFACT_MODIFIED. Resources come from the frozen Work copy and are covered by its aggregate digest rather than an independent prior digest: missing/mismatched copies yield STORE_CORRUPT (storage §5.4). Missing successful upstream output yields INPUT_UNAVAILABLE for required inputs; optional inputs stay unbound and show unavailable.
4. Determine exact brief.md (§4), engine/stats.json bytes and targets before commit. After COMMIT prepare attempts/<node>/occurrence-<NNN>/attempt-<NNN>/ (three-digit padded labels; AttemptId remains node#n.number), engine/, outputs/, and output parents, then write registered historical bytes. Effect failure returns EFFECT_PENDING.
5. Return {attempt,node,occurrence,number,brief_path,output_dir,inputs:{name:path},outputs:{name:path},requires:[...]}. Output_dir is Attempt outputs/; each output path joins its declared path.

Paths are absolute; resources point to works/<work_id>/workbook/<path>. Unbound optional inputs remain present as null and unavailable in briefs. Requires follows node declaration order, resolving full manifest declarations in start's shape. Give brief_path to the worker.

### `attempt submit <work> --attempt <id> --summary <text|@file>`

1. Require running or ATTEMPT_NOT_RUNNING.
2. Summary ≤4096 bytes or SUMMARY_TOO_LONG.
3. Each output is a regular nonsymlink single-link file at output_dir/<path> (storage §4). Missing required yields OUTPUT_MISSING; excess size yields OUTPUT_TOO_LARGE. Any rejection leaves Attempt running and all state unchanged.
4. Record ArtifactRefs at submit; after COMMIT seal the same registered file objects. Effect failure returns EFFECT_PENDING.
5. Mark Attempt succeeded, then in order: gate → blocked(gate); no outgoing edge → succeeded; every target at max_visits → blocked(no_legal_edge); otherwise active.
6. Return {attempt,outputs:{name:ArtifactRef},work_status} and next.

### `attempt fail <work> --attempt <id> --reason <text>`

Mark failed and record reason (≤4096 bytes). max_retries=k allows k business-failure retries per Occurrence; the (k+1)th actual failed blocks with retries_exhausted. Number is creation order; superseded is neither failure nor business retry. Validate historical fail responses against the failed prefix through that Attempt, not current total failures. Return {attempt,work_status} and next.

### `attempt replace <work> --attempt <id> --reason <text|@file>`

For new requests check in order: nonterminal Work (WORK_TERMINAL), old identity exists (NOT_FOUND), running (ATTEMPT_NOT_RUNNING), latest active qualification of current Occurrence (ILLEGAL_NEXT), unused fixed allowance (REPLACEMENTS_EXHAUSTED), bounded reason and frozen inputs. Request replay precedes these checks. Reason ≤4096 bytes. @file intent records only lexical absolute source path; committed replay does not reread it.

In one Decision/transaction/revision, supersede old Attempt with ended_at/replacement_reason; append same-Occurrence running Attempt with checked number+1, inherited entered_from and complete non-stats inputs including optional null. Runtime validates old ref path/sha/bytes through the same handle; do not rebind latest upstream or promote old drafts. Generate new stats from poststate and brief from frozen instructions. Register exact historical bytes and publish through prepare/write/refresh effects. Postcommit failure is EFFECT_PENDING/committed=true; same-ID recovery returns the same replacement and bytes.

Persist {replaced_attempt,attempt,node,occurrence,number,brief_path,output_dir,inputs,outputs,requires}, following begin structure/sources. Top-level revision/request_id/next come from commit snapshot. Suffixes are contiguous from zero, not failure counts. At most one superseded per Occurrence. Exhausted replacement allowance adds no blocked state; current Attempt may submit/fail/cancel. Qualified next includes attempt replace with {work,attempt}; caller supplies reason.

New submit/fail for old superseded returns ATTEMPT_NOT_RUNNING in nonterminal Work; terminal Work returns WORK_TERMINAL first. Old successful requests still replay historical next, not current next. Replacement changes no standards, skips no node/gate, stops no old process, authenticates no successor, and provides no host isolation. Operators check old processes/shared workspace.

### `gate approve <work> --node <node>`

Require blocked(gate) and current node, otherwise ILLEGAL_NEXT. Record {node,occurrence,by,at}, with actual invoking OS account. An agent executing human-authorized approval records the agent process's account truthfully, never independently authenticated human (constitution §5). Apply submit's rules excluding the gate: terminal → succeeded; no legal edge → blocked(no_legal_edge); otherwise active. Return {node,occurrence,by,at,work_status} and next.

### `work result`

Read-only, with §2 prefix resolution; rejects request-id; no recovery/card refresh. Data is work-result/v1:

| Field | Contract |
| --- | --- |
| `format` | Exactly work-result/v1 |
| `work_id`, `revision` | Full ID and integer revision from one SQLite snapshot |
| `workbook` | This Work's frozen {id,version,digest} |
| `flow`, `status` | Frozen Flow ID and WorkStatus object |
| `effects_pending` | Associated committed requests have incomplete file effects; strictly validate complete requests/audit/effects without prefiltering published |
| `final` | True only for valid succeeded terminal/gate facts and no pending effects |
| `artifacts` | Empty unless final; otherwise sorted by key, {key,path,sha256,bytes,source:{attempt,kind,name}} |

Source kind is input/output; source Attempt is the particular successful terminal binding/sealing Attempt; name identifies the selected slot. Preserve its complete frozen ArtifactRef, never latest upstream. No declarations yields final=true/artifacts=[] and “No final results declared.” Contradictory terminal/required-ref/ownership success facts yield STORE_CORRUPT, never false empty selection. Next derives from that same state/graph.

This lists sealed references without claiming to reverify all current bytes, reported commits, exit codes, or business conclusions. Actual readers/exporters validate bytes under their own reading contract.

### `work stats <work>`

Read-only counting of WorkState without judgment:

```text
# Stats 2026-09-24-001-t

status: active   total: 3120s   blocked: 1   approvals: 0

| node | visits | attempts | failed | superseded | avg | entered_via |
| --- | --- | --- | --- | --- | --- | --- |
| draft | 2/3 | 3 | 1 | 0 | 640s | entry×1, review(back)×1 |
| review | 1/3 | 1 | 0 | 0 | 150s | draft(main)×1 |
| publish | 0/1 | 0 | 0 | 0 | 0s |  |
```

Rows follow graph declaration order. Superseded counts administrative revocations, not failures, while ended Attempt durations include them. Total is created_at to updated_at; approvals counts approve calls; avg averages ended Attempts per node. Entered_via preserves first-appearance order and edge kinds, source(edge)×count or entry×count. JSON node fields: node/visits/max_visits/attempts/failed/superseded/avg_seconds/entered_via. Entries are {from:string|null,edge:string|null,count:n}; null source denotes entry; edge kinds remain main/back/branch/re_review.

Blocked is cumulative (GF-29): +1 for successful gated submit, exhausted retries, or no_legal_edge. Transitions record it at occurrence; cancellation never decrements it.

Engine.stats binds this JSON at begin as attempts/<node>/occurrence-*/attempt-*/engine/stats.json with sha256 and a normal brief input row. It includes the new Attempt's visits/attempts; total_seconds reaches current commit time. Finalize bytes before commit and register them in requests.effects_json. Crash replay restores those exact bytes rather than recomputing latest state (D-29 semantics unchanged; recovery follows v2).

### `work cancel <work>`

Allowed for nonterminal Work. Mark cancelled; running Attempts remain running without fabricated completion. Return {work_id,work_status:{kind:"cancelled"}} and empty next.

## 4. Brief `brief.md`

Generated under the Attempt directory. Wrap verbatim instructions in fixed engine headers/footer:

```markdown
# Brief: <node.title>

Work: <work_id> (<name>)
Node: <node>#<n>, Attempt <number>
From: <upstream>#<m> (<edge.kind> edge)       # Entry nodes say entry
Executor: agent (standard)                  # Human nodes say human only

## Inputs

| Name | Path | sha256 |
| --- | --- | --- |
| topic | /Users/me/.sheltie/works/<id>/start-inputs/topic | 3f2a… |
| checklist | /Users/me/.sheltie/works/<id>/workbook/resources/review-checklist.md | 8b40… |
| decision | Not available (upstream plan-review has not produced output) | |

## Required host resources

| Kind | Name | Version | Instructions |
| --- | --- | --- | --- |
| skill | company-api | ^1 | Confirm your host has this skill installed; otherwise stop and inform the user |

## Instructions

<verbatim instruction; trim trailing newlines and let the template insert them>

## Output requirements

| Name | Write to | Required | Limit |
| --- | --- | --- | --- |
| article | /Users/me/.sheltie/works/<id>/attempts/draft/occurrence-001/attempt-000/outputs/article.md | yes | 256 KiB |

Do not modify input files after finishing. Summarize your conclusion for the coordinator and list the output files you wrote.
```

Omit host-resource section when undeclared. Substitute agent/mcp for skill by kind; missing version displays -. For human executors replace the final paragraph with:

```markdown
After writing the output files, run in your terminal:

    sheltie attempt submit <work_id> --attempt <attempt_id> --summary "<one-sentence conclusion>"
```

Outputs live under outputs/; stats under engine/, outside the worker namespace. Coordinators may append context or reword before delivery but must preserve direction, standards, and output requirements. They retain their modified version; the engine does not receive it.

## 5. Response envelope (`--json`, `cli-result/v4`)

Success:

```json
{
  "ok": true,
  "request_id": "0192…",
  "revision": 7,
  "data": {},
  "next": [
    { "op": "attempt begin", "args": { "work": "…", "node": "review" }, "edge": "main", "executor": "agent", "tier": "strong" },
    { "op": "attempt begin", "args": { "work": "…", "node": "draft" }, "edge": "back", "executor": "agent", "tier": "standard" },
    { "op": "work cancel", "args": { "work": "…" } }
  ]
}
```

Work/Workbook writes have data.replayed=false initially/true on replay. Other fields come from commit-time ResponseSnapshot (storage §1.2), including unchanged historical next; resume requires current status. Request_id appears on successful/committed EFFECT_PENDING writes; revision only on committed Work writes. Read-only/self omit inapplicable fields; self has no request snapshot/replayed. Every response has next, empty when none.

Write data.work_status and card data.status are WorkStatus objects: {kind:"active"}, {kind:"succeeded"}, {kind:"cancelled"}, or {kind:"blocked",reason:"gate"}; other reasons retries_exhausted/no_legal_edge. Text renders active/blocked(gate).

Failure:

```json
{
  "ok": false,
  "error": { "code": "OUTPUT_MISSING", "message": "Required output article is missing: /…/article.md", "detail": { "output": "article" } },
  "next": []
}
```

When this Work/Workbook request committed but its own effect failed, include:

```json
{
  "ok": false,
  "error": { "code": "EFFECT_PENDING", "message": "…", "detail": { "cause": "IO" } },
  "next": [],
  "committed": true,
  "revision": 7,
  "request_id": "0192…",
  "original": { "ok": true, "data": {} }
}
```

Original represents the **complete** commit-time success response, abbreviated in the example. Retry the same ID to recover; do not mistake it for uncommitted failure and issue a new ID. Workbook EFFECT_PENDING omits revision but keeps request_id/committed/original.

Original/pending_original contain only verified snapshots. Valid field types and mutually consistent Reply/data do not establish business binding: verify request, audit, target, and corresponding frozen-definition facts. Corrupt metadata/bindings omit unverifiable original/revision, retain confirmed committed identity/accurate cause, and fabricate no success or historical repair. Verified legitimate originals remain available for effect payload/path/sync/mark errors. If Start/add publication registration prevents independent snapshot-identity verification, omit original rather than substitute current installed definitions or guess payloads.

If new request B is blocked under lock by old A's effects, B has not committed. Return EFFECT_PENDING/committed=false/request_id=B with detail.pending_request_id=A and verified detail.pending_original=A's commit response, no top-level B original/revision. Repair A then retry B with its same ID; A's snapshot is not B's success:

```json
{
  "ok": false,
  "error": { "code": "EFFECT_PENDING", "message": "An earlier request has unfinished file effects", "detail": { "pending_request_id": "A", "pending_original": { "ok": true }, "cause": "IO" } },
  "next": [],
  "committed": false,
  "request_id": "B"
}
```

Next lists legal operations and target identity arguments. Callers supply summaries/failure/replacement reasons and replace text placeholders before executing. One next shape applies everywhere, including card data.next: op/args, plus edge/executor/tier only for attempt begin. Read-only responses also include next; executor/tier support delegation before execution.

New-request @file failures (missing, unreadable, non-UTF-8, nonregular, symlink, hardlink nlink>1, >32 MiB source limit from storage §5.3) yield INVALID_REQUEST/exit2 with path/reason. Only new requests open sources; same-path replay does not. Materialized summary/reason >4096 bytes still yields SUMMARY_TOO_LONG/exit1, separate from source-read limits.

Exit: success 0; ok=false 1; argument parsing 2.

## 6. Status card `status-card.md`

After each affecting write commit, refresh_status_card rewrites works/<work_id>/status-card.md from latest state; failures return EFFECT_PENDING. Status reads state/revision, associated requests/audit/effects, and Start publication location in one read-only transaction, strictly validates, then projects the same compact pointer-only card. Unpublished Start uses protected pending original (storage §3.3).

```markdown
# Work 2026-09-24-001-t (t)

workbook: article-review@1.0.1   flow: default   status: active
current: draft#2
done: draft#1, review#1
pending: publish
visits: draft 2/3, review 1/3, publish 0/1

## Current task

attempt: draft#2.0
brief_path: /tmp/sheltie-test/works/2026-09-24-001-t/attempts/draft/occurrence-002/attempt-000/brief.md
inputs:
  review → /tmp/sheltie-test/works/2026-09-24-001-t/attempts/review/occurrence-001/attempt-000/outputs/review.md (sha256 fc11d6f28e59d3cc33c0b14ceb644bf0902ebd63d61218dffe9e7dac7c254542, 10 B)
  topic → /tmp/sheltie-test/works/2026-09-24-001-t/start-inputs/topic (sha256 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824, 5 B)
draft_outputs:
  article → /tmp/sheltie-test/works/2026-09-24-001-t/attempts/draft/occurrence-002/attempt-000/outputs/article.md

## Latest Attempt

draft#2.0 running

## Legal next actions

- sheltie attempt submit 2026-09-24-001-t --attempt draft#2.0 --summary "<one-sentence conclusion>"
- sheltie attempt fail 2026-09-24-001-t --attempt draft#2.0 --reason "<reason>"
- sheltie attempt replace 2026-09-24-001-t --attempt draft#2.0 --reason "<reason>"
- sheltie work cancel 2026-09-24-001-t
```

JSON matches text: work_id/name/workbook/flow/status/current/done/pending/visits/blocked/last_attempt/next. Last_attempt contains attempt/status/summary/reason/outputs; reason is failure text; outputs maps names to complete {path,sha256,bytes}. Blocked is matching explanation text (for example gate: review#2 requires gate approve), or null. Next matches §5 exactly. Live queries always include pending_publish boolean, true for incomplete Start publication, false otherwise.

Resume is null without current Attempt; otherwise {attempt,brief_path,inputs,draft_outputs}. Attempt is current Occurrence's latest; inputs preserve its frozen complete ArtifactRefs/null. Draft_outputs maps names to absolute declared draft paths only while running, otherwise {}. Draft locations prove neither existence, completeness, nor sealing. Do not expand historical bodies by default.

Live data also includes revision/effects_pending/pending_publish from the same Store snapshot. Effects_pending checks only this Work; other Works' effects do not hide results. Disk cards store shared state/graph/resume only, not live revision/readiness. Refresh occurs before mark_published and must not permanently freeze pending=true. Queries acquire no HomeLock, change no business state, and recover no effects.

Done lists successful Occurrences; pending lists never-arrived nodes. Blocked explanations: gate: review#2 requires gate approve; retries_exhausted: draft#2; no_legal_edge: review#3 has all outgoing targets at max_visits.

## 7. Closed error-code set

Each error belongs to exactly one row. New codes require this table and a test.

| Code | Meaning | What happened | Next action |
| --- | --- | --- | --- |
| `INVALID_REQUEST` | Invalid argument syntax/value | No change | Correct arguments |
| `NOT_FOUND` | Missing Workbook/Work/node/Attempt | No change | Verify ID |
| `WORKBOOK_INVALID` | Invalid manifest/file refs; detail.path | No Workbook row/final directory; owned private pending may remain | Fix; next locked write verifies ownership and cleans |
| `FLOW_INVALID` | Compilation failure; detail.path/rule | Same as above | Fix Flow; next locked write cleans owned pending |
| `WORKBOOK_EXISTS` | Same ID/version installed | No change | Bump version |
| `WORKBOOK_IN_USE` | Nonterminal references; detail.works | No change | Finish/cancel them |
| `WORKBOOK_TAMPERED` | Digest mismatch/missing installed directory; detail.results | No change | Restore registered original bytes manually; same-version add does not overwrite; existing Works retain copies |
| `UPDATE_UNAVAILABLE` | Missing platform release/unreachable network; detail.reason | No change | Retry/manual install |
| `UPDATE_CHECKSUM_MISMATCH` | Download digest differs | Download removed | Retry/report repeated failure |
| `INPUT_MISSING` | Missing start key | No change | Supply --input |
| `WORK_TERMINAL` | Terminal Work | No change | None |
| `ILLEGAL_NEXT` | Operation absent from next; detail.next | No change | Choose legal next |
| `INPUT_UNAVAILABLE` | No successful upstream output; detail.input/node | Node not entered | Finish upstream |
| `ARTIFACT_MODIFIED` | Input digest mismatch; detail.path | Node not entered | Inspect manually |
| `ATTEMPT_NOT_RUNNING` | Submit/fail/replace nonrunning Attempt | No change | Read status |
| `REPLACEMENTS_EXHAUSTED` | Fixed one-per-Occurrence replacement used | No state change; current qualification remains | Submit/fail/cancel or established business rework |
| `SUMMARY_TOO_LONG` | Summary/reason >4096 bytes | No change | Shorten; put detail in files |
| `OUTPUT_MISSING` | Required output absent; detail.output | No change; Attempt running | Worker supplies file |
| `OUTPUT_TOO_LARGE` | Exceeds max_bytes | Same | Reduce size |
| `REQUEST_CONFLICT` | Same ID, different target/payload | No change | New ID |
| `REVISION_CONFLICT` | Concurrent write/expected_revision mismatch | No change | Reread status/retry |
| `EFFECT_PENDING` | Incomplete effects; own committed request carries original; old blocking effects use committed=false/pending_request_id | Current request committed or not as indicated; old effects pending | Recover old ID when present, retry current ID |
| `STORE_SCHEMA_MISMATCH` | Structure differs from SCHEMA_VERSION=4, including schema1/2/3 | Refuse business open; main/existing WAL bytes unchanged; D-039 control-file exception | New root; old binary/root for old records |
| `STORE_CORRUPT` | Database, persisted identity, or precommit managed files violate contract | No new business commit | Inspect manually; committed integrity errors appear in EFFECT_PENDING.detail.cause |
| `IO` | Filesystem error; detail.path/system text | Depends on operation; response explains | Check permissions/disk |

Replay is not an error: same ID/intent returns original success with replayed=true. Observed file changes do not affect intent fingerprints. Replacement intent preserves literal reason or source path; audit preserves full materialized bounded reason and compares it verbatim with replaced Attempt reason. Committed replay does not reread reason files.

## 8. Original final Artifact bytes and external export

```text
sheltie --home <management-root> work result <full-work-id> --artifact <key> --revision <positive-integer>
sheltie-export --sheltie <absolute-trusted-binary> --home <real-absolute-management-root> --work <full-work-id> --to <existing-real-absolute-parent> [--json]
```

Raw mode requires artifact/revision together and rejects json/request-id. Argument rejection writes no stdout, diagnoses stderr, exit2. One validated result read verifies exact revision, final=true, effects_pending=false, and key. Ordinary mode still lists refs only. Read the verified Ref through one confined regular single-link FD; require limits, actual size/sha/identity and final success for exit0. Missing/modified/unsafe originals or qualification failures use existing codes/exits, diagnostics only on stderr. Partial stdout is possible; consumers stage it and check final exit rather than treat it as complete. No cleanup/recovery.

External tools strictly accept the C004 payload: final/succeeded/no effects/nonempty selection, without recomputing selection. Metadata ≤1 MiB, file ≤32 MiB, aggregate ≤256 MiB with checked addition. Reject unknown fields/formats, unsorted/duplicate keys, or unrepresentable NUL keys before staging. Empty/Unicode keys are valid, not IDs; leaf names are safe segments only within indexed private directories. Close stdin, invoke direct argv without shell/PATH. Independently verify stdout/digest/bytes and child exit. On failure terminate/wait direct child; stderr natural language is never success authority.

Work-export-manifest/v1 retains complete result and key-sorted files {key,path,sha256,bytes}; paths are copy-relative, without timestamps/staging names. Verify real parent/ancestors do not overlap Home; reject links/identity replacement. Exclusively create directories0700/files0600, regular single-link. After complete reception, independent readback, and required OS sync, atomically publish the whole directory with NOREPLACE as <work-id>-<random>. No overwrite, merge, old-staging takeover, competitor deletion, or Store writeback.

One-line work-export/v1 JSON contains status/work_id/revision/target_path/staging_path/error (stable code/message/source exit status); unknown/unconfirmed values are null. Complete exit0; argument/deterministic qualification rejected exit2; source/target/integrity/IO failed_before_publish exit1; postrename parent-sync/final-identity uncertainty publication_unconfirmed exit3. Never compensate by deleting unconfirmed publication; report only ownership-verifiable paths. After kill/no response, inspect the scene; reruns create new copies. No promise of process-tree termination, physical power-loss durability, host isolation, or continued equality after user edits.
