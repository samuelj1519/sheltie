# Edit a Workbook on the canvas and obtain a new copy

English | [简体中文](../../zh-CN/how-to/edit-workbook.md)

Use this to create/edit methods visually. The source-distributed local web tool requires Node.js ≥ 22 and a trusted current-source engine. It edits browser drafts and downloads new ZIPs, without overwriting author directories or running Works.

## 1. Start the local tool

Obtain `engine_binary` through the [build guide](build-from-source.md). Install tool dependencies from the root, then start only after success:

```bash
npm --prefix tools/workbook-editor ci --ignore-scripts --no-audit --no-fund
node tools/workbook-editor/server.mjs --sheltie "$engine_binary" --port 4311
```

Open the printed actual http://127.0.0.1:port. Substituting `localhost` fails Host checks. If occupied, stop this invocation and restart with an available port. Ctrl+C stops your service.

## 2. Open and edit a draft

Select New method or Open directory, or expand left-side samples for Code change / Specification development. Extract ZIPs into independent directories before opening; direct ZIP import is unsupported. To edit an installed same-ID/version method, create a new draft version.

Edit ID/version/Flow; add Nodes with instructions/inputs/outputs, set entry, then explicit Edges. Selecting existing endpoints locates that edge; self-loop/duplicate edits reject. Select input sources separately; lines do not bind files automatically.

Select a Node, browse Inputs N / Outputs N tabs, search, and open items to edit names/sources/output paths. Advanced settings cover required/result/sizes/limits/host requirements. Untouched fields retain declarations; human executors do not set tier.

## 3. Select input material

Expand + Add material to select multiple predecessor outputs or inherit original predecessor-input sources. Custom names need uniqueness within this step only. Resolve name conflicts as prompted, without renaming to replace old sources with different material.

Checked existing sources indicate use. Remove items explicitly from the selected list; other aliases of the same source remain. Pending selections survive folding/tab changes but require Add selected material to enter the draft.

Enter file locations/URLs and press Enter to create fixed reference text within the method. The tool neither reads those host paths nor fetches URLs; executors obtain actual material following instructions. Inheritance retains from declarations rather than reading prior Attempt frozen inputs.

## 4. Inspect the graph and locate problems

Use main-flow view for the trunk, all-connections view for every edge, and Node focus for neighbors. Hidden edges remain selectable in All connections; from/to/kind match properties. Drag/pan/zoom/Arrange layout change views only, without method bytes.

Click Check structure and fix drafts using returned files/Nodes/fields/rules. Green results qualify only that complete file set. Unknown fields remain engine-rejected, without form sanitization. Unparseable/unrepresentable documents retain original bytes and reject relevant edits.

## 5. Save, reopen, and install

Before closing/refreshing/switching drafts, click Save new ZIP copy. Saving rechecks the current set; extract successful ZIPs into a new author directory. Reopen with Open directory and verify contents. After choosing the delivery version, use [management](manage-workbooks.md) CLI add/show/verify, then exercise necessary paths.

Drafts live only in page memory; localStorage stores layout. Structural failure prevents successful ZIP generation. Refreshing does not recover unsaved content. On narrow screens, Close in navigation/properties or Esc returns to canvas.

See [tool reference](../reference/workbook-editor.md) for limits/HTTP rejection/cleanup. Structure does not prove quality, real completion, or user acceptance; see [design](../explanation/workbook-editor.md).
