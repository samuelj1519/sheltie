# Install, verify, upgrade, and remove Workbooks

English | [简体中文](manage-workbooks.zh-CN.md)

Manage author directories/installed versions using a trusted engine and explicit root. [Build](build-from-source.md) for new trials; existing Works retain original roots. engine_binary/management_root/author_dir are actual paths.

## 1. Install the full author directory

Verify all manifest-listed Flows/instructions/resources exist, then:

```bash
"$engine_binary" --home "$management_root" --json workbook add "$author_dir"
"$engine_binary" --home "$management_root" --json workbook list
```

Check every response. Installation validates/copies complete directories without scripts/models. Retain ID/version/digest. Existing versions return `WORKBOOK_EXISTS` without overwrite.

For safe retries, generate/save UUID before writes and pass --request-id. Retrying keeps ID/original directory arguments. See [recovery](resume-work.md#incomplete-file-effects-and-unknown-outcomes) for unfinished effects.

## 2. Inspect structure, inputs, and integrity

Set `workbook_spec` to actual `<id>@<version>`, e.g. two-step@1.0.1:

```bash
"$engine_binary" --home "$management_root" --json workbook show "$workbook_spec"
"$engine_binary" --home "$management_root" --json workbook verify "$workbook_spec"
```

Check selected Flow start_inputs/nodes/edges/requires in show. Provide every start key and verify actual host dependencies before start. Declarations are not installations. verify must be `ok`; retain detail.results/original directories for tampered/missing and investigate manually. Renaming versions does not replace corruption investigation.

Omitted verify checks every version. list selects highest by literal order; pin reproducible versions. Reads do not recover pending_publish=true. Pending originals may support queries, but recover registered writes before advancing.

## 3. Upgrade a method

Edit author copies with new versions/IDs. add/show/verify again, then start new Works with explicit versions. Existing Works retain frozen copies, without hot switching or Store edits to continue newer versions.

See [authoring](write-workbook.md) and [canvas editing](edit-workbook.md).

## 4. Remove an exact version

Confirm objects from list/Work status. After explicit authorization, use full ID/version:

```bash
"$engine_binary" --home "$management_root" --json workbook remove "$workbook_spec"
"$engine_binary" --home "$management_root" --json workbook list
```

`WORKBOOK_IN_USE` lists nonterminal references in detail.works. Complete those Works first. If cancellation is needed, authorize it separately; do not automatically cancel to remove methods. Terminal frozen copies neither block removal nor disappear with installed versions.

Stop further actions on nonzero exit/ok=false. Retain original command/request ID/stdout/stderr/exit. See [troubleshooting](troubleshoot.md).
