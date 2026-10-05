# Sheltie constitution

This is the shortest and most stable authority. It defines why Sheltie exists and which rules no version may break. It takes precedence over conflicting documents. Amend this document before changing rules elsewhere.

## 1. One sentence

Sheltie is a local workflow engine that **performs repetitive mechanical work for coordinator agents without making their judgments**. People write methods as Workbooks; coordinator agents read them and delegate; workers execute; the engine records state, generates briefs, and enforces rules.

## 2. Responsibilities

| Role | Responsibilities | Boundaries |
| --- | --- | --- |
| People (Workbook authors and users) | Write methods; approve gates and installation | Do not change a running graph |
| Coordinator agent | Read the Workbook, delegate each step, interpret worker responses, choose among legal next actions | Must follow engine-provided legal next actions |
| Worker agent or person | Transform inputs into outputs according to instructions; report conclusions in natural language | Must not modify others' outputs, standards, or records |
| Engine | Parse graphs, record state, bind inputs, generate briefs, compute legal next actions, enforce gates and limits | Does not judge quality or choose routes |

Ask: **Without the engine, would the coordinator have to repeat this mechanical operation each time?** If yes, it belongs in the engine; otherwise, leave it to the coordinator or Workbook. This question guides thinking; it is not an automatic adjudicator.

## 3. Invariants (INV)

Any violation is a design error and must be rejected in review.

| ID | Rule | Counterexample |
| --- | --- | --- |
| `INV-1` | The engine contains no business judgment. It neither determines acceptance nor chooses a legal edge for the coordinator | Automatically advancing after reading “accepted” in a review document |
| `INV-2` | The engine infers no system facts from natural language | Parsing “isolated” from a worker response into state |
| `INV-3` | The engine neither writes host configuration nor installs resources into agents. It only checks and informs. Copying a Workbook into its own management root is not host installation | Writing `~/.claude/` after discovering a missing skill |
| `INV-4` | Package means a Cargo package or external dependency, never a business entity. Workbook is the sole name for a business method | Types named `PackageId` or `PackageCatalog` |
| `INV-5` | Review is an ordinary Node with no dedicated state machine. The engine records execution facts and structural validation results | `Verdict`, `Pass`, or `Fail` enums in the engine |
| `INV-6` | Model outputs must not become system facts: identity, time, file digests, edge legality, gate approvals, limit counters, or artifact ownership | Accepting a worker's self-reported output sha256 |
| `INV-7` | There is one state authority. Work state resides only in SQLite; cards, directories, and logs are projections that cannot advance state | Recovering Work identity from a directory name |

## 4. Three hard rules

These constraints govern legal next actions and submission validation.

1. **Edges must be explicit.** Legal next actions come only from declared Flow edges. There is no implicit next step.
2. **Inputs freeze by bytes.** Each input bound in a brief has a path and digest. If modified after execution begins, the engine rejects it rather than reading a replacement with the same name.
3. **Gates cannot be bypassed.** Leaving a node with `gate = true` requires a human approval record. Edges, retries, and recovery cannot bypass it.

## 5. Human approval has two places

1. **Human execution nodes.** A human executor's submission is that node's conclusion.
2. **Gates and external authorization.** Leaving `gate = true` requires approval. Host, network, and workspace permissions are governed by the host and operator. The current engine validates legal operations and gates; it does not evaluate arbitrary host permissions or provide isolation.

Other operations within agent authority belong to the coordinator. This follows OpenAI and Anthropic guidance to intervene for high risk and exceeded failure limits while automating the rest.

Approval records truthfully identify which operating-system account invoked approval and when. They are accounting facts. In a shared OS-account environment, the engine neither provides nor claims independent human authentication. The host and user decide whether invocation authority is reserved for a person.

## 6. Cost discipline

Avoid token waste in the design.

| ID | Mechanism |
| --- | --- |
| `T-1` | People prewrite brief skeletons in the Workbook; the engine delivers them once and binds paths without generating bodies |
| `T-2` | Model-visible material travels through files and path references rather than large prompt excerpts |
| `T-3` | The engine stores state; coordinators read compact cards rather than history |
| `T-4` | Worker responses contain bounded summaries and output references; oversized responses are rejected, never truncated |
| `T-5` | Rework and repair loops deliver only findings documents and changed files |

Every cost metric requires a quality criterion. Savings without evidence that quality was preserved do not establish benefit.

## 7. Evolution

As models improve, the engine becomes smaller. Do not move tasks coordinators reliably handle into the engine. Retain state, legal edges, immutable artifacts, gates, and limits. Host readiness is an unadopted [roadmap](roadmap.md) direction; currently the engine passes requires declarations only. Describe the product as a reusable-method engine for coordinators, rather than an autonomous-company operating system.
