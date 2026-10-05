# Workbook editor

English | [简体中文](README.zh-CN.md)

An unreleased local web tool for creating Workbooks, opening directories, editing Flows/nodes, and downloading complete new ZIP copies. Requires Node.js 22 or later and a trusted `sheltie` binary from the current development source.

## Start

Install its dependencies from the repository root:

```bash
npm ci --prefix tools/workbook-editor
```

Start from the repository root, replacing the binary path with a verified absolute path:

```bash
node tools/workbook-editor/server.mjs --sheltie /absolute/path/sheltie --port 4311
```

Open the printed `http://127.0.0.1:4311` address exactly. `localhost` fails the Host check. The service listens only on `127.0.0.1`; stop it with Ctrl+C.

## Create and edit

1. Select **New Workbook**, then edit its ID, version, and name beneath the left-hand Workbook name.
2. Select **＋ Node** and edit its ID, title, executor, and instructions on the right. Instructions can be inline text or a file in the directory. Missing instruction files can be created within the draft.
3. Select **＋ Explicit edge** and choose source, target, and kind. Input bindings are independent: connecting nodes does not add inputs. Select an edge to edit or delete it.
4. Choose the Flow entry or select a node and **Set as entry**. **Choose or enter inputs** supports multiple inputs and fixed references; add/remove outputs individually. Advanced settings include tier, gate, visit/retry limits, and host-resource references. Declare host resources in Workbook advanced settings.
5. Select **Check structure**. Failures display the engine code, rule, field, reason, and next step. When no exact Flow file is identified, open a candidate definition and check its node/entry against the reported field. Expand **Technical diagnostics** for raw command/output. Follow the engine field path when correcting the draft; deleting nodes or changing IDs does not guess or repair references.
6. Select **Save new ZIP copy**. The tool rechecks the current file set and downloads a complete ZIP only after the real engine accepts it. Extract into a new directory and reopen with **Open directory**.

**Open directory** uses the browser directory picker to read the whole directory; this version does not import ZIPs. Fixed sample buttons open default English code-change and spec-dev. Chinese variants remain explicitly selectable as repository directories. Switch Flows in the left panel. Unedited Flows and binary resources retain original bytes; edited TOML may be reformatted. **View current TOML source** tracks draft bytes. Ordinary name/title/instruction edits preserve the properties-panel DOM. Complete objects and unknown fields remain present, and the real engine continues to reject unknown fields.

The main flow follows first-visit order, wrapping at roughly three nodes per row. The complete edge count stays visible. **Main flow / All edges** changes presentation only. A selected node shows adjacent edges; the complete left-hand list selects and locates hidden edges. Kind labels appear on hover/selection; highlights use the same route. Dragged positions persist until **Arrange layout** rearranges them.

Node bindings appear in **Inputs N / Outputs N** tabs. Compact source groups support name/source search. Select an item to edit it; selecting another replaces the expanded editor. Instructions start collapsed. Names, sources, and required declarations are written only on actual edits; switching, searching, and expanding do not modify the Workbook.

**＋ Add inputs** starts collapsed. Its title shows the pending count; uncommitted choices survive collapse and tab switches. Existing sources show read-only checkmarks and this step's used names. Remove an input individually from the selected list; other aliases sharing its source remain.

The unified picker selects direct predecessor outputs or reuses predecessor inputs' original source declarations. Reusing a source does not read a previous Attempt's frozen inputs. Rename/remove selections, then **Add selected inputs**. Names need be unique only within this node; Chinese and underscores are supported. Conflicts require renaming; identical name/source pairs are not duplicated. Optional outputs preserve their required declaration.

Enter a file location or URL in the same picker and press Enter to select a fixed external reference. The location is stored in the Workbook's own reference text file without reading host paths or accessing URLs; the eventual worker follows the location and method instructions. Existing start, ordinary resource, statistics, and custom sources retain their original declarations. Exact sources and required declarations appear in advanced settings. Editing a fixed location creates a new resource copy and preserves the old file. Renaming changes only the name; removing an input does not delete reference files. ZIPs contain every original resource and new reference text.

Dragging nodes changes only view layout. Drag empty space to pan, scroll or use buttons to zoom, and **Fit canvas** to adjust the view. Layout persists in browser localStorage; draft content exists only in page memory. Save a ZIP before closing/refreshing. On narrow screens, **Navigation** and **Properties** open the panels. Node lists and edges support keyboard selection, with visible form focus.

## Boundaries and failures

Input limits are 16 MiB, 1024 files, and 4096 UTF-8 bytes per relative path. Check the list before reading; stream-limit HTTP bodies to 24 MiB. Refuse the entire set for escaping paths, duplicates, ASCII-case/NFC aliases, directory aliases, file/directory prefix conflicts, or invalid base64.

Transmit original bytes as `[[path, base64], ...]`. Check/export accepts this set only, not host paths, arbitrary commands, or arbitrary binaries. Each check materializes/verifies bytes in a new owned temporary root (0700), then directly invokes the trusted startup-selected engine using `--json workbook add` and a new `SHELTIE_HOME`. Source directories, production Homes, and frozen Work copies are not written.

Each CLI call is limited to 30 seconds and separate 1 MiB stdout/stderr limits. Timeout, excess output, or a disconnected request triggers TERM, then KILL after one second if closure has not occurred. Clean the owned root only after observing direct-child termination and stream closure. Restore write permissions on engine-installed read-only directories only inside that root. If closure cannot be confirmed, fail with `residualRoot` and preserve it. The service does not promise whole-process-tree isolation or authenticate a person within the same OS account.

Host must equal the printed `127.0.0.1:port`; POST Origin must match exactly and carry the session token. Titles, instructions, paths, and errors are text. Markdown does not render HTML. Unparseable/unrepresentable documents retain original bytes and refuse the corresponding edits.

## Checks and interfaces

Run from the tool directory using this verified build; missing engine variables must fail rather than skip real tests:

```bash
cd tools/workbook-editor
SHELTIE_EDITOR_ENGINE=/absolute/path/sheltie npm test
SHELTIE_EDITOR_ENGINE=/absolute/path/sheltie node scripts/smoke.mjs
```

Tests cover independent file/model expectations, real examples, unknown-field CLI refusal, real HTTP rejection matrices, complete ZIP byte equality after extraction and add/show/verify, and direct-child timeout/output/cancellation boundaries. HTTP tests require local listening permission. Preserve sandbox denials verbatim and obtain scoped permission; denial is not product PASS.

Fixed interfaces: `GET /api/session` returns the page token; `GET /api/sample/code-change` and `/api/sample/spec-dev` read fixed samples; `POST /api/check` returns original CLI structural results; `POST /api/export` returns `application/zip` only after successful revalidation. POST uses `Content-Type: application/json`, `X-Editor-Token`, and an entry-array body. Static resources have a fixed allowlist, with no arbitrary host-path interface.

`smol-toml` 1.7.1 and `fflate` 0.8.2 are pinned in package-lock. The tool is outside Cargo/dist releases. Independent content review, browser operation, and human acceptance are separate results; npm test does not replace them.
