# Why the authoring tool is separate from the engine

English | [简体中文](../../zh-CN/explanation/workbook-editor.md)

Workbook authors need to see the graph, edit instructions and inputs/outputs, and obtain a complete runnable method directory. The authoring tool provides a canvas and uses a trusted `sheltie` public CLI to judge structure. It does not run Works or maintain another execution state.

See [edit a Workbook](../how-to/edit-workbook.md) for operations and [authoring reference](../reference/workbook-editor.md) for interfaces, limits, and file rules.

## Complete files are safer than form projections

`workbook show` is useful for a method summary, but cannot reconstruct the complete directory. Resources may be binary; instructions have original bytes; TOML may contain unknown fields that should be rejected.

The browser therefore retains original bytes in a file Map. Forms modify only fields actually edited. Untouched files retain original bytes; edited TOML retains the complete object. Unparsable or unrepresentable files refuse the relevant edit instead of being replaced by templates. Structural checks can still find existing problems, without form rendering turning an invalid definition into a different valid one.

Layout and definitions are also separate. Position, search, folding, and connection display modes do not change Nodes, Edges, or input bindings. Lines represent explicit flow only. Sources for predecessor inputs are selected separately, without inferring file transfer from adjacent nodes.

## Structure must be judged by the real engine

Browser checks provide early feedback but cannot replace engine compilation rules. The service materializes the current complete set in a new temporary root it owns, then calls public `workbook add` with the trusted engine fixed at startup. It neither reads Store directly nor bypasses public validation through internal modules.

Saving a ZIP revalidates the exact set received. A previous green indicator cannot qualify later edits. The download is an author draft, not sealed results from a formal Work. Installation and actual execution still use their respective public checks.

## Why arrays of entries and whole-set preflight

Receiving same-named files as a JSON object or extracting a ZIP first can overwrite duplicates before validation. An entry array preserves every path and its bytes, allowing duplicate, alias, directory-conflict, and limit rejection before materializing the whole set.

Filesystems can treat different spellings as one path, so preflight and post-materialization verification of the complete byte set have separate obligations. Normalizing paths and retaining the last entry silently loses data. The first version opens directories only: extract downloaded ZIPs before opening, avoiding a second ZIP input parser.

## Why a narrow local interface

The service listens locally and verifies the actual Host, Origin, and session token. Interfaces accept draft bytes, without arbitrary host paths or commands. Fixed routes and text rendering keep imported instructions as data rather than executable page content.

The tool can verify exit of its direct engine child, but cannot prove that the whole host process tree has stopped. Cleanup waits until that child and its streams have closed. If this cannot be confirmed, it retains the root and reports it. This scope limits both tool permissions and success guarantees.

Implementation entry points are [server.mjs](../../../tools/workbook-editor/server.mjs), [file boundary](../../../tools/workbook-editor/lib/files.mjs), [CLI boundary](../../../tools/workbook-editor/lib/engine.mjs), and [browser model](../../../tools/workbook-editor/public/model.mjs). See the [tool README](../../../tools/workbook-editor/README.md) for tests and execution, and the [Workbook contract](../../../specs/contracts/workbook.md) for method rules.
