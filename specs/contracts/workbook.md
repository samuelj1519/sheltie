# Workbook and Flow formats

This contract defines Workbook directories, `workbook.toml`, Flow files, and loading validation. Initial format versions are `workbook/v1` and `flow/v1`. **Every object rejects unknown fields.**

## 1. Directory

A Workbook is a directory authored in any repository:

```text
my-workbook/
  workbook.toml
  flows/
    default.toml
  instructions/
    draft.md
    review.md
  resources/                 # Optional worker references, bound as resource.<path> inputs
    review-checklist.md
```

`sheltie workbook add <dir>` copies the directory into this operation's `pending/`, then parses the manifest, compiles every Flow, and checks `instruction.file`, `resource.<path>`, and `requires` references. Validation and registered identity derive only from the final copy ([storage §5.2](storage.md)); directory digests use `workbook-digest/v2` ([storage §5.1](storage.md)). Precommit failure creates no Workbook row or final directory; uncommitted private staging is cleaned under the storage contract. Postcommit publication failure returns `EFFECT_PENDING`, retaining the Store-protected original for recovery. Validation executes no scripts, models, or network requests.

An existing `<id>/<version>` is rejected (`WORKBOOK_EXISTS`). Changed content requires a new version.

### Visual author tool

An independent local web tool may read a user-selected source directory, edit existing fields/instructions, and download a new complete ZIP copy that can be reopened after extraction. Unedited files retain original bytes. Edited TOML retains the complete original object, including unknown fields that must still be rejected; form projections must not silently sanitize it. Graph coordinates must not become unknown fields. The initial tool input limit is 16 MiB/1024 files; reject excess without truncation. This does not change engine loading limits. Final structural judgment comes from trusted public CLI loading the same file set into the tool's temporary Home, not UI preflight. Revalidate current bytes before saving rather than relying on previous preflight. The tool does not modify installed versions or frozen Work copies.

Default layout and “Arrange layout” change views only. Arrows, line styles, and text distinguish main, rework, and branch directions. Selecting a path, label, or edge-list entry selects the same actual edge shown by its endpoints/type in the properties panel and highlighted on canvas; redraw must not turn a click into another edge or node. Input-source labels “Provided at start / Another step's output / Workbook reference / Run statistics” map exactly to existing from syntax. Unknown or illegal values display their original custom values without automatic repair. Fold required, result, and max_bytes into advanced controls. Preserve untouched fields, including omitted defaults and unknown objects; rendering a friendly form must not write defaults, change semantics, or add edges.

Navigation and properties panels have visible in-panel close buttons, accessible even when narrow windows cover the canvas. Esc closes the side panel and returns to canvas. Visibility changes neither method bytes nor selected definitions.

When adding edges, determine duplicates by `(from,to)` regardless of `kind`. Select and explain an existing edge without adding or rewriting it. Property edits must reject self-loops and duplicate endpoint pairs, retaining the original value and restoring the form. Imported illegal definitions remain unchanged for complete public CLI validation.

## 2. `workbook.toml`

```toml
schema = "workbook/v1"
id = "article-review"          # Lowercase letters, digits, single hyphens; ≤ 64 bytes; globally unique
version = "1.0.0"              # Semantic version string; literal comparison, no range evaluation
name = "Article writing and review"
description = "Write an article and have an independent agent review it; return rejected drafts for revision."
flows = ["flows/default.toml"] # At least one path relative to the Workbook root

[[requires]]                   # Optional host resources: declared, not bundled
kind = "skill"                 # skill | agent | mcp
name = "company-api"
version = "^1"                 # Optional range, for people and installation tools
 digest = "sha256:…"           # Optional; when supplied, bytes must match
source = "https://github.com/acme/skills/tree/main/company-api" # Optional installation source
```

| Field | Type | Rule |
| --- | --- | --- |
| `schema` | string | Must equal `workbook/v1` |
| `id` | string | `^[a-z0-9]+(-[a-z0-9]+)*$`; ≤ 64 bytes |
| `version` | string | Nonempty, ≤ 32 bytes, only `[0-9A-Za-z.+-]`; reject `.`, `..`, and reserved `.staging` because it is one safe directory segment ([storage §5.3](storage.md)) |
| `name` | string | Nonempty; ≤ 128 bytes |
| `description` | string | Optional; ≤ 2 KiB |
| `flows` | string array | Nonempty; Workbook-relative paths without `..`, absolute paths, or symlinks |
| `requires` | table array | Optional; ≤ 32 entries; unique `(kind, name)` |

`requires[]` fields:

| Field | Rule |
| --- | --- |
| `kind` | One of `skill`, `agent`, `mcp` |
| `name` | ID character rules. Identity is `kind + name`: matching declarations in different Workbooks or existing host resources name the same resource |
| `version` | Optional; ≤ 32 bytes. MVP passes it through without comparison |
| `digest` | Optional; `sha256:` followed by 64 lowercase hexadecimal digits; supplied values require byte equality |
| `source` | Optional URL; ≤ 512 bytes |

**Resources have two layers.** Worker-readable files belong in `resources/`, bound as inputs and frozen with the version, without installation or name collisions. Use `requires` only for host mechanisms: automatic skill invocation/tool authorization, named subagents with model/tool limits, or MCP services. Declaration is not bundling. The engine lists declarations in briefs without checking or installing host resources; see [roadmap GF-20](../roadmap.md).

## 3. Flow file

```toml
schema = "flow/v1"
id = "default"                 # Unique within the Workbook
entry = "draft"                # Entry node

[[nodes]]
id = "draft"
title = "Write a draft"
executor = "agent"
instruction = { file = "instructions/draft.md" }
inputs = [
  { name = "topic", from = "start.topic" },
  { name = "review", from = "review.verdict", required = false },
]
outputs = [{ name = "article", path = "article.md", max_bytes = 262144 }]
max_visits = 3

[[nodes]]
id = "review"
title = "Review the draft"
executor = "agent"
instruction = { file = "instructions/review.md" }
inputs = [
  { name = "article", from = "draft.article" },
  { name = "checklist", from = "resource.resources/review-checklist.md" },
]
outputs = [{ name = "verdict", path = "review.md", max_bytes = 65536 }]
max_visits = 3

[[nodes]]
id = "publish"
title = "Finalize"
executor = "human"
instruction = { text = "Read the accepted article and confirm it may be published. Copy the final version to final.md." }
inputs = [{ name = "article", from = "draft.article" }, { name = "review", from = "review.verdict" }]
outputs = [{ name = "final", path = "final.md" }]

[[edges]]
from = "draft"
to = "review"
kind = "main"

[[edges]]
from = "review"
to = "publish"
kind = "main"

[[edges]]
from = "review"
to = "draft"
kind = "back"
```

### 3.1 Flow fields

| Field | Type | Rule |
| --- | --- | --- |
| `schema` | string | Must equal `flow/v1` |
| `id` | string | Workbook ID character rules |
| `entry` | string | Must identify a node |
| `nodes` | table array | 1–64; unique `id` |
| `edges` | table array | 0–256; unique `(from, to)` |

### 3.2 Node fields

| Field | Type | Default | Rule |
| --- | --- | --- | --- |
| `id` | string | Required | ID character rules |
| `title` | string | Required | ≤ 128 bytes; human-readable |
| `executor` | `"agent"` or `"human"` | Required | Executor only; does not imply a gate |
| `tier` | `"strong"` or `"standard"` | `"standard"` | Coordinator model-selection label: strong for design judgment, standard for filling templates, commands, and checklist comparison. Passed to next/briefs without engine action; forbidden for human executors |
| `instruction` | `{ file = ... }` or `{ text = ... }` | Required | Exactly one; file is Workbook-relative, UTF-8, ≤ 64 KiB; text nonempty, ≤ 8 KiB |
| `inputs` | array | `[]` | `{ name, from, required?, result? }`; names unique within the node; required defaults true |
| `outputs` | array | `[]` | `{ name, path, required?, max_bytes?, result? }`; names and paths unique within the node |
| `requires` | string array | `[]` | `"<kind>:<name>"` must match a manifest declaration; listed in briefs |
| `gate` | bool | `false` | true requires human approval to leave after Attempt success |
| `max_visits` | integer | `1` | 1–32 node arrivals, including loops |
| `max_retries` | integer | `1` | 0–8 business-failure retries within one arrival |

Four input source forms:

| Form | Meaning | Validation |
| --- | --- | --- |
| `"start.<key>"` | Initial input key | key follows ID rules; missing key rejects work start |
| `"resource.<path>"` | Workbook file frozen with the version | Workbook-relative existing regular file, ≤ 32 MiB; resolves to this Work's frozen copy |
| `"engine.stats"` | Facts generated at begin ([work stats JSON](protocol.md)), written as `engine/stats.json` under the Attempt and byte-frozen | Literal stats only after engine.; intended for reflection; numbers without engine conclusions |
| `"<node>.<output>"` | Output of that node's latest successful Attempt | Node exists, differs from self, and is not named start/resource/engine; declared output exists; see compilation rule 5 |

`required = false` is meaningful only for node outputs. Without an upstream successful Attempt, leave the input unbound, mark it unavailable in the brief, and do not reject begin with `INPUT_UNAVAILABLE`. This supports feedback loops: optional review feedback is absent on first arrival. Compilation rejects optional start/resource/stats inputs because these always exist.

| Output field | Default | Rule |
| --- | --- | --- |
| `name` | Required | ID character rules |
| `path` | Required | Relative to Attempt `outputs/`; rules below |
| `required` | `true` | Missing optional files are allowed; downstream must not require them |
| `max_bytes` | `1048576` | 1–33554432 (32 MiB) |

Output-path rejections name `nodes[i].outputs[j].path`:

1. No `..`, absolute path, or empty segment (general RelPath rules).
2. **Portable characters:** each nonempty segment contains only `A-Z a-z 0-9 . _ -`, ≤ 128 bytes. Reject non-ASCII, including Han and Unicode variants; ASCII case folding then fully determines aliases without guessing Unicode normalization.
3. Paths in one node must neither duplicate nor be ancestors of one another (`out` versus `out/x.md`).
4. Fold each segment's ASCII case before checking equality/ancestry (`OUT.md`/`out.md`, `Out`/`out/x.md`), because the target default filesystem is case-insensitive.

Engine files (`brief.md`, `engine/stats.json`) reside at Attempt root; worker files under `outputs/`. These namespaces are not compared; `outputs/brief.md` and `outputs/stats.json` are legal.

### 3.3 Final result declarations

Inputs/outputs may declare `result = true` (default false) only on terminal Nodes without outgoing edges, and selected entries must be required. Selected logical names are unique across both categories. Rule 10 rejects nonterminal, optional, or duplicate selections with field paths. Input source syntax remains §3.2.

Selections bind to the terminal Attempt that caused success: inputs use begin-time ArtifactRefs; outputs use that Attempt's sealed submit-time ArtifactRefs. Selection belongs to the terminal node; producers may be earlier Attempts or start/resource/engine files. Without declarations, the result is empty, never inferred from history or directories. See [work result](protocol.md#work-result) for reading/readiness.

### 3.4 Edge fields

| Field | Rule |
| --- | --- |
| `from`, `to` | Existing node IDs; distinct endpoints |
| `kind` | `main`, `back`, `branch`, `re_review` |

Kinds are coordinator labels; engine legality is identical for all four. Main advances, back returns upstream, branch enters repair, and re_review returns from repair to review.

## 4. Compilation validation

Check in this order. Any failure returns `FLOW_INVALID` with field path and reason:

1. All IDs valid/unique; node IDs exclude start/resource/engine; entry exists.
2. Every edge has existing distinct endpoints and a unique endpoint pair.
3. Every node is reachable from entry; unreachable nodes are errors, not warnings.
4. At least one terminal node has no outgoing edge.
5. Referenced nodes/outputs exist and a directed path leads from producer to consumer. Optional outputs require optional consuming inputs. Start/resource/stats must not be optional. These exhaust static checks; unavailable successful upstream Attempts cause runtime `INPUT_UNAVAILABLE`.
6. Gate nodes must not have empty text instructions.
7. Referenced files exist within size limits; instruction files are UTF-8; resources may use any encoding.
8. Every node requires entry matches the manifest, without node-local duplicates.
9. Human nodes must not declare tier.
10. Result selections are required terminal inputs/outputs with unique names across categories.

**Cycles are intentional.** Back/re_review edges support repair loops. Hard max_visits limits make every path finite. Compilation does not detect cycles; runtime counts arrivals.

## 5. Instructions

Instructions are worker-facing natural language, like skills: direction, standards, and output requirements with room for judgment.

- State what to read, produce, and where conclusions go: “Read article, apply these three criteria, and write review.md with Accepted or Rejected on its first line.”
- Do not hardcode runtime paths. The engine appends absolute input and target output paths after instructions.
- Leave edge selection to the coordinator after reading outputs.
- For multiple optional inputs, use the brief's source line to identify the relevant input. Once bound, optional inputs retain their previous content; source identity mechanically distinguishes old and new.
- Put conclusions on each output's first line. Coordinators use it to choose edges; downstream workers use it to decide whether to read further.

## 6. Example Workbooks

Examples are also end-to-end fixtures:

| Directory | Demonstrates |
| --- | --- |
| `examples/two-step/` | Two agent nodes, one main edge, no review/gate; minimal business-independent flow |
| `examples/article-review/` | §3 graph; back loop, optional feedback, max_visits, human executor, resources |
| `examples/gated-release/` | Gated agent node followed by terminal node; blocking and approval |
| `examples/code-change/` | implement → review → deliver with explicit rework; frozen task/project; terminal change/review/delivery |

The complete business Workbook `workbooks/spec-dev/` has eleven nodes and twenty-five edges, optional inputs, engine.stats, tiers, two human-review nodes, independent verification/review loops, human escalation, and final reflection. The default English version is 0.2.3; the maintained Chinese variant is `spec-dev-zh-cn` 0.2.2. Both explicitly bind shared approval rules to scaffold/implement/verify; terminal retro selects delivery input and lessons output, available through work result after approval. Existing Works retain their original frozen versions. This is not a fixture, but compilation and actual CLI scenarios cover it to prevent contract changes from breaking it.

Default English examples retain their IDs at 1.0.1. Chinese sibling directories use IDs ending `-zh-cn` at 1.0.0 so both variants install concurrently. Chinese instructions remain maintained translations, including English future commit rules; they are not byte-identical historical evidence. Tests freeze fixture bytes. If implementation and examples differ, revise authority/examples before implementation rather than accommodating implementation through expectations.

The author tool's clear-flow view follows first arrivals along main edges from entry while preserving all cycles/branches. Full edge counts/modes are visible; node focus shows adjacent edges; selecting hidden edges in lists still shows the actual edge. Automatic ports and compact wrapped/side-column layouts change only views, never from/to/kind, bindings, or engine next. Labels appear on hover/selection; highlights reuse identical rendering geometry. Renaming an input changes only its name, preserving from and other declarations. Names obey existing node-local uniqueness, not node-ID syntax.

The tool may multi-select direct predecessor outputs to create node.output inputs, or predecessor inputs to explicitly reuse their original from declarations; this does not claim to read a previous Attempt's frozen input copy. Preserve required's three states; optional outputs must not become required inputs. Do not copy context-dependent result/max_bytes attributes. Do not add unrepresentable self-references, missing sources, or prohibited optional combinations for the current owner; preserve imported files for CLI judgment. External URLs and absolute/relative locations become actual Workbook reference-text resources bound by existing resource syntax. They are reference strings; the editor neither reads host contents nor accesses networks. No new from form, node.input source, or undeclared manifest/Flow field is introduced. Verify ownership of generated dedicated reference text; preserve unrecognized original resources without silent overwrite.

Use one editable multi-select “Select or enter material” control for predecessor candidates and custom external references, with customizable names rather than separate path/URL fields. Exact syntax appears in advanced/raw views only; untouched inputs are not converted or migrated.

Input/output tabs and compact grouped summaries support browsing. Search, grouping, collapse, and tabs affect display only, without declaration reordering/rewriting. Edit names/sources/filenames only after opening an item; instructions start collapsed. Candidate usage derives from exact current from declarations, displaying checked state and this step's names; aliases or differing required declarations are not merged/overwritten. A checked candidate means used material only. Remove explicitly from selected entries; unchecking must not implicitly delete multiple same-source inputs. Refresh checked state after edits/removals/additions, preserve unknown/illegal originals, and defer judgment to CLI.
