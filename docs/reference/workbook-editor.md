# Workbook authoring reference

English | [简体中文](workbook-editor.zh-CN.md)

The unreleased local web tool in `tools/workbook-editor` edits drafts and downloads complete ZIPs. See [canvas editing](../how-to/edit-workbook.md), [design](../explanation/workbook-editor.md), and the authoritative [Workbook contract](../../specs/contracts/workbook.md)`.`

## Startup and environment

Requires Node.js ≥ 22, locked tool dependencies, and a trusted absolute `sheltie` path.

```text
node tools/workbook-editor/server.mjs --sheltie <absolute-binary> --port <port>
```

Pass startup arguments in this order. Port is an integer `0`–65535; `0` lets the OS choose. Listens only on 127.0.0.1. Use the printed actual URL without substituting localhost. Ctrl+C stops it.

## Interface and saving

| Feature | Behavior/boundary |
| --- | --- |
| New method, open directory, fixed sample | Load browser drafts; first version does not import ZIPs; extract downloads before opening |
| Node/Edge editing | Edit explicit definitions; new/edited self-loops or duplicate endpoints reject; CLI judges existing invalid imports |
| Clear flow, all edges, node focus | Display only; clear mode retains all definitions and hidden edges remain selectable in lists |
| Input/output tabs and search | Browsing only; opening/editing an item writes declarations |
| Select/enter material | Multiple predecessor outputs, inherited original input sources, or file locations/URLs |
| Freeze external reference | Store location as Workbook reference text, without reading host files/fetching URLs |
| Check structure | Public CLI on complete current bytes; not a content-quality verdict |
| Save new ZIP copy | Recheck draft before download, without overwriting author directories |
| Drag/pan/zoom/arrange | View only; layout in localStorage, draft bytes in page memory |

Edit input bindings separately from edges. Inherited predecessor inputs use original from declarations, not previous Attempt frozen files. Checked existing sources indicate use; remove explicitly from the selected list, without implicitly deleting multiple aliases. See the [tool README](../../tools/workbook-editor/README.md) for forms.

## Limits and file boundaries

| Object | Limit/rule |
| --- | --- |
| File set | ≤ 16 MiB, ≤ 1024 files; reject whole oversized set |
| HTTP body | Streamed total ≤ 24 MiB |
| Relative path | UTF-8 ≤ 4096 bytes; reject absolute paths, drive prefixes, backslash, NUL, colon, empty/./.. segments |
| Path set | Reject duplicates, ASCII case/NFC aliases, directory-spelling aliases, file/directory prefix conflicts |
| CLI invocation | 30 seconds; stdout/stderr each ≤ 1 MiB |
| Termination | Timeout/output excess/disconnect sends TERM, then KILL after 1 second without closure |

Check inventories before reading, then verify the same complete materialized set. Cleanup occurs only after the direct child exits and streams close. Otherwise fail and report `residualRoot`, without claiming the process tree stopped.

## HTTP interfaces

| Method/path | Result |
| --- | --- |
| `GET /api/session` | This page session token |
| `GET /api/sample/code-change` | Fixed code-change file set |
| `GET /api/sample/spec-dev` | Fixed spec-dev file set |
| `POST /api/check` | Current-set structure result, including engine result |
| `POST /api/export` | Recheck then `application/zip` |

POST requires `Content-Type: application/json`, `X-Editor-Token`, and Origin exactly matching the printed HTTP origin. Bodies are entry arrays `[[relative_path, base64], ...]`, without host paths/arbitrary commands. Every Host must equal actual `127.0.0.1`:port.

Structural success is HTTP 200; failed checks 422; Host/Origin/token mismatch 403; Content-Type mismatch 415; oversized bodies 413; unknown routes 404. Other rejections retain diagnostics; HTTP status alone does not determine cause.

## Fidelity and trust

Untouched files retain original bytes. TOML edits may reformat but retain complete objects, unknown/unmodified fields. Engines still reject unknown fields; forms must not silently sanitize them. Unparseable/unrepresentable documents retain bytes and reject relevant edits.

Titles/descriptions/errors render as text; Markdown does not render HTML. There are no arbitrary host-path, shell, binary-selection, or Work interfaces. Temporary structural success proves neither user acceptance, report quality, nor execution success. See [implementation](implementation.md)`.`
