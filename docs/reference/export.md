# Final-result reads and export reference

English | [简体中文](export.zh-CN.md)

This page covers raw `work result` and external `sheltie-export` arguments, results, and limits. See the [guide](../how-to/export-results.md) and exact [protocol §8](../../specs/contracts/protocol.md#8-original-final-artifact-bytes-and-external-export). The tool remains unreleased; see [limits](limitations.md).

## Raw-byte reads

```text
sheltie --home <root> work result <work> --artifact <key> --revision <positive-integer>
```

artifact/revision must appear together. Use `complete` Work ID, `revision`, and literal `key` from the same result query. Keys need not satisfy ID rules. `--artifact=<key>` avoids treating a leading hyphen as an option.

Raw rejects --json/--request-id and emits original `bytes` without newline. Qualification requires exact `revision`, `final=true`, `complete` effects, and explicit `key` selection. Then verify source identity, actual `bytes`, and sha256. Mid-read errors may leave partial stdout; destination existence alone is insufficient.

## `sheltie-export` arguments

```text
sheltie-export --sheltie <binary> --home <root> --work <full-work-id> --to <parent> [--json]
```

| Argument | Requirement |
| --- | --- |
| `--sheltie` | Trusted executable absolute `path`, without PATH search |
| `--home` | Existing real absolute management root, without symlinks |
| `--work` | Complete Work ID, without prefixes |
| `--to` | Existing real explicitly authorized absolute parent, without mutual containment with Home |
| `--json` | One `work-export/v1` line; otherwise human-readable output |

Source must be succeeded, `final=true`, `effects_pending=false`, with nonempty selections. Queries/reads use CLI without Store access or newest-file reselection.

## Files and formats

New directories are `<work-id>-<random>`, without overwrite/merge. Directory permissions are 0700; `files` 0600. Sorted keys map to private indexes such as artifacts/0001/; keys are not concatenated into paths.

`manifest.json` uses `work-export-manifest/v1`, retaining the `complete` source result and `files` mappings. Entries contain `key`, relative copy `path`, `sha256`, and bytes. Copies are editable; original manifests no longer guarantee edited bytes. Source Works remain unchanged.

| Object | Limit |
| --- | --- |
| Result JSON | `1` MiB |
| Single `artifact` | 32 MiB |
| All `artifact` `bytes` | 256 MiB |

Verify source exits, actual sizes/digests, independently read all `files` back, and perform required OS sync before whole-directory publication. This is neither physical power-loss durability nor same-account isolation.

## Responses and exits

JSON includes format/status/work_id/revision/target_path/staging_path/error; unconfirmed/inapplicable values are null. It differs from engine ok/data/next.

| `status` | Exit | Confirmed boundary |
| --- | --- | --- |
| `complete` | `0` | Bytes verified, whole directory published without overwrite, OS sync completed; use `target_path` |
| `rejected` | `2` | Arguments or deterministic qualification `rejected` |
| `failed_before_publish` | `1` | Source/target/integrity/I/O failure before publication; staging is not a completed copy |
| `publication_unconfirmed` | `3` | Move occurred, but final identity/parent sync unconfirmed; retain state without automatic delete/rollback |

Termination may produce no response. Report only paths whose ownership is verified; names cannot establish ownership of other directories. Reruns always create new copies, without adopting old staging. See [failure handling](../how-to/export-results.md#failure-state).
