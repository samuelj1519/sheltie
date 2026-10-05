# Workbook reference

English | [简体中文](workbook.zh-CN.md)

A Workbook is one complete method directory. This summarizes directory/declaration/examples; the [Workbook contract](../../specs/contracts/workbook.md) defines types, limits, and installation rejection order. See [authoring](../how-to/write-workbook.md).

## Directories and formats

```text
my-workbook/
  workbook.toml
  flows/default.toml
  instructions/draft.md
  resources/checklist.md
```

| Location/identifier | Purpose |
| --- | --- |
| `workbook.toml` | `workbook/v1` manifest: ID/version/name/Flow files/optional host requirements |
| Manifest `flows` | Explicitly lists every Flow; no directory scan |
| Flow file | `flow/v1`: id/entry/Nodes/explicit Edges |
| `instructions/` | Node instructions; `instruction.text` may inline content |
| `resources/` | References bound by `resource.<path>`, frozen with versions |
| `workbook-digest/v2` | Full-directory digest; [storage §5.1](../../specs/contracts/storage.md#51-workbook-digestv2) |

All definition objects reject unknown fields. Graph coordinates are authoring views, outside manifests/Flows. Install uses the full directory; execution uses the Work's frozen copy. Later author edits do not change existing runs.

## Nodes, Edges, and defaults

| Declaration | Summary |
| --- | --- |
| `instruction` | Exactly `file` or `text`; files relative to Workbook root |
| `executor` | Required agent/human, without automatic `gate` |
| `tier` | `agent` defaults `standard`; `strong` allowed; `human` must omit |
| `gate` | Default `false`; `true` blocks after successful submission until approval |
| `max_visits` | Default `1`; limits arrivals, including back-edge arrivals |
| `max_retries` | Default `1`; limits failures within an Occurrence |
| `inputs`, `outputs` | Empty arrays by default; `required` defaults `true` |
| `outputs[].max_bytes` | Default `1` MiB, maximum 32 MiB; safe regular output files |
| `result` | Default `false` for inputs/outputs; only `required` terminal items may be final results |
| `edges[].kind` | main/back/branch/re_review; same engine legality, purpose labels for coordinators |

Edge `(from,to)` pairs are unique; self-loops are prohibited. All Nodes must be reachable from `entry`, with at least one terminal lacking outgoing edges. Cycles are allowed with arrival limits. Edges/input sources are declared separately: edges do not transfer files automatically.

Output `path` is relative to Attempt `outputs/`, without escape, duplicate/ancestor paths, or ASCII case aliases. Use contracted ASCII `path` characters, without applying Unicode display-name rules to filenames. See [Node fields](../../specs/contracts/workbook.md#32-node-fields) and [compilation](../../specs/contracts/workbook.md#4-compilation-validation).

## Input sources

| from | Bound content | Missing-source rule |
| --- | --- | --- |
| `start.<key>` | Text/`@file` materialized at start | Required; `required=false` prohibited |
| `resource.<path>` | Reference in frozen Workbook | Must exist; `required=false` prohibited |
| `engine.stats` | Stats generated/frozen at begin | Engine-provided; `required=false` prohibited |
| `<node>.<output>` | Declared output from latest successful Attempt of that node | Missing `required` sources reject; optional sources may be `null` |

Upstream references must satisfy graph reachability and cannot reference self. Optional `outputs` cannot supply `required` inputs. Without new feedback, optional feedback may still bind an earlier successful output. Read brief provenance: bound does not mean newly produced.

Terminal `inputs` freeze references at its begin; selected `outputs` freeze at its submit. Success without `result=true` has an empty final set. See [final results](data-model.md#final-results) for exact provenance.

## Host requirements

Manifest `requires` identifies skill/agent/mcp by `kind + name`; Nodes reference kind:name. Version/digest/source are declarations passed through responses/briefs. The engine does not probe hosts, evaluate compatibility ranges, or install resources. Coordinators verify availability before delegation.

Use `resources/` for file-readable material; `requires` for host mechanisms. References, host installation, and actual permissions differ; see [limits](limitations.md).

## Repository methods

| Method | Start `inputs` | Learning/use |
| --- | --- | --- |
| [two-step](../../examples/two-step) | `topic` | Outline → summary; no gates/back edges/final selections |
| [article-review](../../examples/article-review) | `topic` | Optional feedback/resources/rework/human `executor` |
| [gated-release](../../examples/gated-release) | `version` | Post-submit blocking/approval, without actual software publication |
| [code-change](../../examples/code-change/README.md) | `task`, `project` | Repository edits/independent review/rework/change-review-delivery results |
| [spec-dev](../../workbooks/spec-dev/README.md) | `request`, `project` | Specification/planning/task implementation and verification/replanning/delivery/reflection |

Before running, verify actual Flow/start keys/declarations with `workbook show <id>@<version>`. Default English spec-dev is 0.2.3; Chinese spec-dev-zh-cn is 0.2.2. Method versions differ from engine versions. Examples retain English default directories and explicit *-zh-CN method variants. See [code-change](../how-to/run-code-change.md) and [spec-dev](../how-to/run-spec-dev.md).
