# CLI reference

English | [简体中文](../../zh-CN/reference/cli.md)

Look up invocation forms, main results, and input rules by command group for current source. The [protocol contract](../../../specs/contracts/protocol.md) defines complete fields, rejection precedence, and error codes. See [guides](../how-to/README.md) for operations and [troubleshooting](../how-to/troubleshoot.md) for failures.

## Invocation and global options

```text
sheltie [--home <dir>] [--json] [--request-id <uuid>] <group> <verb> [args]
```

Angle brackets mark arguments to `replace`; brackets mark optional parts; `...` means repeatable. The first three options are global and may also follow subcommands.

| Option | Behavior |
| --- | --- |
| `--home <dir>` | Management root; precedes `SHELTIE_HOME`, default `~/.sheltie` |
| `--json` | Ordinary commands emit one JSON line to stdout; warnings/diagnostics may independently use stderr |
| `--request-id <uuid>` | Business writes for Workbook/Work/Attempt/gate only; generated if omitted. Retrying preserves ID and intent |
| `--help`, `-h` | Help at each command group/subcommand |
| `--version`, `-V` | Binary `version`; `self version` reports root/platform/Store `format` |

Read-only commands and all `self` commands reject --request-id. USER/USERNAME do not authenticate operators. Audit/approval principals come from the actual OS process account.

## `self`: manage the engine

| Invocation | Main behavior/result |
| --- | --- |
| `self version` | version/platform/home/schema_version without creating the root |
| `self install` | Copy running binary to `bin/sheltie` and initialize Store; identical bytes return `already_installed=true` |
| `self update [--version <version>]` | Download/verify a release; omitted `version` uses latest, explicit `version` pins tag `v<version>` |
| `self rollback` | Restore `bin/sheltie.prev`; one rollback level, without Store downgrade |
| `self uninstall [--purge] [--yes]` | Default removes `bin/` and retains `data`; confirmed purge removes data/binaries but retains root/original `.lock` |

Use `0.2.0` without `v` for --version. Text purge confirmation `requires` `yes`; JSON `requires` --yes. `SHELTIE_RELEASE_BASE` overrides update source, defaulting to this project's GitHub Releases; check actual sources first. Install/update do not modify shell configuration or install coordinator skills. See [installation](../how-to/manage-installation.md).

## `workbook`: manage method versions

| Invocation | Main behavior/result |
| --- | --- |
| `workbook add <dir>` | Validate/install the whole directory without overwriting the same ID/version |
| `workbook list` | Versions/names, highest `version` per ID, pending-publication state |
| `workbook show <id>[@<version>]` | Flow nodes/edges/start_inputs/host declarations |
| `workbook verify [<id>@<version>]` | Recompute directory digests; omitted selection checks all, returning ok/tampered/missing |
| `workbook remove <id>@<version>` | Remove exact `version`; reject nonterminal Work references |

Omitted versions select the highest installed `version` by literal string order, without semantic-version range evaluation. Pin versions for reproducible runs. See [Workbook management](../how-to/manage-workbooks.md).

## `work`: run and query

| Invocation | Main behavior/result |
| --- | --- |
| `work start --workbook <id>[@<version>] --flow <flow> [--name <name>] [--input <key=value>]...` | Freeze method/start `inputs`, create Work, return complete work_id/work_dir/next |
| `work list` | IDs/names/state/current node/update time |
| `work status <work>` | Current state/revision/resume/pending flags/current `next` |
| `work stats <work>` | Facts about visits/attempts/failures/revocations/durations/edge sources |
| `work result <work>` | Terminal-selected frozen references, without file contents |
| `work result <work> --artifact <key> --revision <revision>` | Raw bytes; both parameters required together, no --json/--request-id |
| `work cancel <work>` | Cancel a nonterminal Work without stopping host processes or undoing external effects |

`<work>` accepts complete IDs or unique prefixes. Ambiguous prefixes reject with candidates. Coordinating/exporting uses complete IDs. Work dates/daily sequences use UTC, without guessing local dates.

Start `inputs` must exactly match the selected Flow's `start_inputs` in `workbook` show. `--input 'topic=Introduce Sheltie'` passes literal text; `--input "topic=@/absolute/path/topic.md"` reads file contents. `--input project=/absolute/path/repo` freezes path text, without copying the repository. File sources must be UTF-8 single-link regular files without symlinks, subject to [storage-read limits](../../../specs/contracts/storage.md#53-source-directories-and-host-metadata).

## `attempt` and `gate`: advance a run

| Invocation | Main behavior/result |
| --- | --- |
| `attempt begin <work> --node <node>` | Claim a node permitted by current `next`; return attempt/brief_path/inputs/outputs/requires |
| `attempt` `submit` `<work>` --attempt <attempt> --summary <text-or-@file> | Verify/seal declared `outputs`; return references/Work state/next |
| `attempt fail <work> --attempt <attempt> --reason <text>` | Record actual failure; remaining limits allow retry or block |
| `attempt` `replace` `<work>` --attempt <attempt> --reason <text-or-@file> | Revoke a running Attempt and claim a new brief in the same Occurrence |
| `gate approve <work> --node <node>` | Record approval at current `blocked(gate)`, then recompute `next` |

Summary/reason texts are at most 4096 bytes. submit/replace support `@file`; `fail` treats reason literally. Use output paths, Attempt IDs, and nodes from actual responses, without guessing directories/numbers. See [resume](../how-to/resume-work.md) to distinguish continuation, failure, content rework, and revocation.

## Responses, replay, and exit codes

Ordinary success has `ok=true`, `data`, and next. Successful business writes include `request_id`; Work writes include top-level revision. Query revisions are inside corresponding data. Inapplicable fields are omitted. Failure has `ok=false` and `error.code/message/detail`; do not parse natural-language messages alone.

Same-ID/same-intent replay returns the commit-time snapshot with data.replayed=true. `next` is historical too; query fresh `work status` before continuing. Same-ID/different-intent returns REQUEST_CONFLICT. `EFFECT_PENDING` `committed` distinguishes this request's commit from blocking by old effects; see [recovery](../how-to/resume-work.md#incomplete-file-effects-and-unknown-outcomes).

| Exit | Meaning |
| --- | --- |
| `0` | Command succeeded; still check business outcomes such as final/pending flags |
| `1` | Business/file/recovery failure; determine `committed` effects from structured responses |
| `2` | Parsing or specific input rejection, e.g. invalid `@file`, request ID on queries, invalid raw parameter combinations |

`cli-result/v4` is the persistent business-response snapshot `format`; ordinary outer JSON need not include format. `work result` `data.format` is work-result/v1. Raw stdout contains original bytes without an added newline. Partial output may precede failure; verify exit and integrity before consumption.

External `sheltie-export` has independent `work-export/v1` statuses/exits; see [export reference](export.md). See [compatibility](limitations.md) and [releases](releases/README.md) for current-source/release relationships.
