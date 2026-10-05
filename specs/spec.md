# Sheltie product specification

This is the sole source of product semantics; architecture, contracts, and plans derive from it. See [constitution](constitution.md), [architecture](architecture.md), and [roadmap](roadmap.md). Must states the adopted target, not implementation completion. Current release/change/progress entry points are in the [map](README.md); MVP history is in [v0.1.0 archive](../docs/en/reference/releases/v0.1.0/README.md).

## 1. Problem

Multistep work with agents such as Claude Code/Codex repeatedly needs mechanical bookkeeping: remember progress, hand outputs to successors, redeliver instructions, stop for approval, and prevent skipped checks. Agent-maintained bookkeeping is error-prone and token-expensive.

People express methods as **Workbooks** (TOML graphs plus natural-language instructions). **Coordinators** delegate; **workers** or people execute; the **engine** records state, generates briefs, and computes legal next actions.

Assess product value through total investment and final quality during method reuse: fewer repeated explanations, process checks, handoffs, and interruption rework while retaining provenance and approvals. Report preparation/first-task/reuse separately; completion or local time savings do not establish benefit. Initial scenarios/adoption conditions are in the roadmap.

### Current product environment

Current commitments/acceptance cover adopted local macOS aarch64/APFS scenarios, including representable names/paths, legal Han names, Workbooks, and exported copies. Other platforms/filesystems and external physical-device qualification require separately adopted demand.

Non-UTF-8 names APFS cannot create are not mandatory live inputs. Preserve accurate invalid argument/name/path rejection contracts/tests. External physical-drive tests do not block local APFS delivery. Scope restrictions neither convert unrun tests into passes nor introduce runtime filesystem admission. Adopt explicit scope/real environments before extending verification ([D-044](../docs/en/explanation/decisions/D-044-macos-apfs-product-scope.md)).

## 2. Success

A new user must be able to use documentation alone to:

1. Choose a Workbook; author/install a two- or three-step method when needed.
2. Invoke /sheltie in Claude Code, select a method, and start Work.
3. Observe coordinator brief acquisition/delegation/submission/advancement without unnecessary human intervention.
4. Stop at declared gates and continue after approval.
5. Locate explicitly selected terminal references through work result and originals in fixed directories, tracing each to its particular binding/sealing Attempt. Coordinators/users still judge content.
6. Run entirely different methods (writing, code review, research organization) without engine changes.

## 3. Roles and boundaries

| Role | Responsibilities | No authority to |
| --- | --- | --- |
| Author | Write step/input/output/check instructions, connect graph, declare gates | Change running graphs or grant host/network/file permissions |
| User | Select method/Flow, supply inputs, approve gates/installation | Turn historical failures into successes |
| Coordinator | Read method, acquire brief, delegate, interpret responses, choose legal next | Bypass next/gates or maintain another progress authority |
| Worker/person | Transform inputs, write conclusion documents, summarize | Change others' outputs/standards/records |
| Engine | Parse graph, create/store Work, bind briefs, enforce next/gates/limits, maintain output directories | Judge content, choose routes, infer natural-language facts, install resources |

**People own Workbooks.** AI may draft; human reading, editing, and adoption establish freezing. Unfrozen drafts establish no graph/gates.

Authors prepare graphs/instructions/stable rules/failure paths; users choose methods and provide current goals/inputs. One person may do both, with costs separate. Graphs express stable stages; agents may investigate/subdivide within them. Ordinary repairs do not automatically add approvals.

**Review is ordinary work.** Its process matches writing; only instructions specify review targets/standards/conclusion output. The engine does not identify Review nodes.

## 4. Step execution

```text
Input files → execution → output files + natural-language response → coordinator chooses legal next
                  ↑                                                       │
                  └──────────────── rework / repair branch ───────────────┘
```

For review:

1. Begin returns verbatim prewritten instructions plus bound paths/output requirements.
2. Worker reads/reviews, writes Accepted or Rejected with findings into declared output, then briefly reports.
3. Submit records bounded summary/file digests after existence/size checks and returns next.
4. Explicit edges provide main acceptance, back rework, branch repair, and re_review return. Coordinator reads outputs and chooses.
5. Beginning the target **is edge selection**; no separate advance operation.

The engine never interprets document contents. It knows execution finished, files exist, summary is bounded, and legal edges. **Execution success is not acceptance.** Completed rejection is successful execution; crashes/timeouts/missing delivery are execution failures retryable within limits.

## 5. Capability requirements

GF identifiers connect architecture/contracts/tests. These requirements are adopted unless marked roadmap. See [implementation](../docs/en/reference/implementation.md) and [acceptance](../docs/en/reference/acceptance.md) for actual evidence/scope.

### 5.1 General engine

`GF-01` **Business independence.** One binary runs no-review two-step, review/repair, and human-gate Flows. Engine code contains no business vocabulary such as Spec/Plan/Git/code review.

`GF-02` **Separate language from machine contracts.** Instructions express work; TOML expresses dependencies/edges/limits/output contracts. The engine infers no acceptance, permissions, or next steps from language.

`GF-03` **Bounded graphs, explicit edges.** Declare nodes/edges (main/back/branch/re_review); loading checks references/entry/reachability/limits. Running graphs are immutable.

`GF-04` **Pull progression.** Coordinators acquire/execute/submit. Every write response includes next. Read queries change no state.

`GF-05` **Definitions and runs differ.** Workbook defines; Work runs one ID/version/digest. Arrivals are Occurrences; executions are Attempts. Retries/loops append rather than overwrite records.

### 5.2 Briefs, inputs, outputs

`GF-06` **Prewritten skeletons and path bindings.** Deliver original instructions once plus absolute input/output requirements; no generated/rewritten bodies. Coordinator rewording must preserve intent.

`GF-07` **Byte-frozen inputs.** Record path/sha256. Later bindings reject modifications rather than same-name replacements.

`GF-08` **Output contracts and immutable artifacts.** Declare name/path/required/size. Submit checks existence/size, records sha256, seals. Successors read identical bytes. Structural validation establishes compliance, not quality.

`GF-09` **Bounded responses.** Natural-language summary plus output refs, no conversation history; default 4 KiB. Reject excess, never truncate.

`GF-33` **Original result bytes and new copies.** [Protocol](contracts/protocol.md) governs reading/export. Engine reads originals; external tools write authorized parents only, without lifecycle changes.

### 5.3 State and progression

`GF-10` **External state and compact cards.** Regenerate current node/completed/pending/latest summary/output paths/block reasons/next after transitions. Resume via current brief/frozen inputs/running draft paths; live queries add same-snapshot revision/pending effects. Draft paths prove no existence/sealing. Text/JSON share facts and shapes, including failure reasons, complete path/digest/byte refs, and next; coordinators need not reread history.

`GF-11` **Completion differs from conclusion.** Attempt records running/succeeded/failed and administrative superseded. Revoke old qualification and start same-Occurrence replacement atomically, at most once per Occurrence, with caller-reported reason. No process stop/executor authentication/host write exclusion. Business conclusions remain coordinator-interpreted documents; no Review state machine.

`GF-12` **Gates cannot be bypassed.** Successful gated Attempts block Work with only approve/cancel. Approval records actual OS process account, never spoofable environment variables. No outgoing edge is legal before approval. Shared accounts provide no independent human authentication; report approval by that account only.

`GF-13` **Mechanical limits, free strategy.** Max_visits counts arrivals including loops; max_retries counts business-failure retries per Occurrence. Exhausted edges/retries disappear from next. Exhaustion/all-target limits block with reasons. Count failed directly: k permits k retries, failure k+1 blocks. Numbers do not determine allowance; superseded is not failed. Coordinator chooses retry/abandon within bounds.

`GF-14` **Completion and result selection.** Nodes without outgoing edges are terminal. Successful terminal Attempt without unapproved gate succeeds Work. Nongate blocking allows cancel only; terminal Work rejects writes. Terminal required inputs/outputs may select results. Result binds to the specific successful terminal Attempt. Valid terminal/gate facts plus all Work effects complete establish final=true; other states have empty selection. No selections is explicitly empty; corrupt refs reject rather than select latest files/parse reports.

### 5.4 Reliability

`GF-15` **Idempotence and recovery.** Work/Workbook writes support request_id; reads/self reject it as argument error. IDs bind operation/full target/user parameters. Same intent returns complete commit response with replayed=true; different target/parameters conflict. Precommit failures leave no successful record and may retry same ID. Postcommit unfinished effects recover from registration on replay/next write; unprovable completion reports committed recovery failure and stops new writes. Historical next is historical; resume requires current status. Any kill/restart yields consistent state/files or explicit unprovable effects; cards regenerate from state.

`GF-16` **Accurate structural rejection.** Validate DB/Workbook structures, rejecting whole mismatches without migration/clearing. Structure changes increment initial format versions.

### 5.5 Workbook and host integration

`GF-34` **Visual author tool.** Unreleased local web tool creates/edits Workbook source copies after user directory selection. Canvas edits explicit Flows/instructions/inputs/outputs while retaining advanced declarations. Duplicate endpoint additions select/explain existing edges; endpoint edits reject self/duplicate edges preserving originals. Compact reading-order main flow, focused adjacency/full-edge mode, and arrange layout retain all definitions. Directional ports and rework/branch distinction do not force right-out/left-in. Labels appear on hover/selection; highlight reuses exact path. Selected edge/panel/highlight/endpoints identify one actual edge. Customize input names, multi-select predecessor outputs or reuse original input sources. Multiple external URLs/locations become fixed internal reference text without access. Renaming preserves source declarations. Input/output count tabs, grouped summaries, item editing, collapsed instructions, and actual-source checked states preserve aliases and require explicit removal. Friendly source labels/advanced collapse retain untouched/unknown values. Panels have visible accessible close controls in narrow overlays; Esc returns to canvas without draft change. Layout is view-only, absent from format/state. Trusted CLI validates same bytes in own temporary Home. Saving produces a complete reopenable ZIP, never overwrites source/installed/frozen copies. Local listening/user-selected reads; clear size/unknown-format/incomplete-validation rejection. CLI/Store/workbook/v1/flow/v1 unchanged.

`GF-17` **Independent Workbook release and local lifecycle.** A directory contains manifest/Flows/instructions/optional resources and may live in any repo. Add privately copies then validates/registers only final bytes, with no scripts/models/network. Concurrent source changes reject invalid copies; valid copies register actual bytes without claims to detect restored transient changes. Remove rejects nonterminal references; verify detects installed tampering. Upgrade installs new versions; old versions remain until manual removal. Start copies/reverifies methods then uses that copy only, unaffected by upgrade/removal. Add/remove support request replay.

`GF-27` **Self installation/upgrades.** Self writes only ~/.sheltie, never shell config. Pin release tags/assets, verify, replace, retain rollback. Default uninstall preserves data; confirmed purge removes data/binary but retains empty root/original lock. No request_id.

`GF-28` **Host declarations, not bundling.** Worker references live in resources and arrive as frozen inputs. Host skills/named subagents/MCP use kind+name manifest declarations referenced by nodes. Engine lists needs without host checks/install. Matching kind/name identifies one resource across methods/hosts for future installation deduplication.

`GF-18` **Sheltie skill is instructions only.** Deliver SKILL.md teaching existing CLI operations, no state store/progression judgment; CI checks mechanically. Installed artifact is self-contained after copying anywhere. Referenced contracts generate/check from one authority, no manually maintained duplicate.

`GF-19` **Honest delivery.** Report source, offline tests, actual Claude Code interaction, and external installation separately. Tests alone are not value evidence.

### 5.6 Facts and reflection

`GF-29` **Engine facts, Workbook reflection, human evolution.** Read-only stats count arrivals/Attempts/failures/mean duration/source node+edge/blocking/approvals. Blocking is cumulative transition history, unchanged by cancellation/progression. Ordinary reflection nodes bind engine.stats and read reports to produce evidenced, located method improvements. Engine changes no parameters/methods; human review/new version add adopts improvements. A given Workbook version behaves consistently across runs.

`GF-30` **Input preflight, no business side effects.** Show exposes all ordered start keys. Deterministic start rejection precedes sequences/business directories without Home/lock creation. Existing WAL Store identification permits D-039 control-file exceptions only. Correct inputs may reuse request IDs.

`GF-31` **Closed persistent recovery.** No business rows/finals before commit; preparation failures after sequence allocation may leave unreclaimed gaps. Clean pending by ownership/Store references, never age. Unprovable committed effects stop accurately. Restore historical bytes, regenerate latest cards. No automatic old-root/user-data cleanup; purge requires confirmation/retains lock.

`GF-32` **Trusted confined file identity.** Derive paths from root and verify managed handles for writes/recovery/deletion. Observation/limits/digest/sealing refer to one object. Reject Workbook digest/frozen identity mismatch; separate engine/worker files. D-039 shared memory is control data, not business state.

### 5.7 Roadmap clauses

Unadopted directions retain IDs for questions, with [demand conditions](roadmap.md): GF-20 host installation/readiness; GF-21 version reminders/continuation; GF-22 isolation/permission evaluation; GF-23 MCP/multiple hosts; GF-24 cost measurement/gates; GF-25 parallel drafts/single-writer selection; GF-26 dynamic expansion templates.

## 6. Non-goals

Do not introduce these for competitive parity:

- Natural-language acceptance/edge inference or running-graph modification.
- Coding-CLI adapter farms/resident session pools.
- Generic buses/second execution databases/second advancing Work state.
- Unauthorized installation or automatic isolation downgrade.
- Agent/adapter counts as proof of efficiency.

## 7. Acceptance scenarios

Every row requires at least one end-to-end test.

| Scenario | Required observation | Must not occur |
| --- | --- | --- |
| No-review two-step | One successful Attempt each; succeeded Work/two files | Global-review blocking |
| Accepted review | Worker writes acceptance; coordinator selects main | Engine acceptance judgment |
| Rejected review | Back/branch then re_review; second review Occurrence | Success conflated with acceptance/old outputs overwritten |
| Loop limit | Target at max_visits absent from next | Back treated as error/unbounded loop |
| Human gate | Successful Attempt blocks; approve legalizes edges | Begin successor before approval |
| Modified input | Downstream begin ARTIFACT_MODIFIED | Reading replacement |
| Missing output | Submit OUTPUT_MISSING; remains running | Success with empty directory |
| Replay | Same-ID original response; different payload conflict | Duplicate Work/Attempt |
| Killed write | Restart status agrees with disk | Half state/card-DB disagreement |
| Referenced removal | Nonterminal WORKBOOK_IN_USE; terminal removal/status remains complete | Lost running instructions |
| Resource input | Brief points at Work frozen copy; installed changes irrelevant | Inline bodies/new downstream bytes |
| Upgrade/rollback | Bad checksum rejected with intact old binary; successful rollback restores | Half replacement/identical current and prev after rollback |
| Facts | After rework stats show draft2/3 and review via draft×1; stats digest matches recorded state | Engine recommendations/inconsistent stats |
| Cross-target replay | Cancel A then same-ID B conflicts; B unchanged | Cancel B/return A as B |
| Postcommit crash | Before publication kill; next write publishes originals; replay returns original plus publication outcome | Original discarded/history rewritten |
| New-root rejected start | Missing input creates no files/directories or consumed sequence | Half root/sequence gap |
| New user | Documentation alone enables install/start/complete/output retrieval | Oral background required |

## 8. External claims

May promise: identical inputs follow identical rules within declared/validated boundaries; missing conditions are never claimed satisfied.

Must not promise: models never misjudge, every task finishes, or content is always correct.

Report distinct facts: output structure validated; independent agent review completed; user authorized the gate. Generic “high security” does not replace concrete scope.
