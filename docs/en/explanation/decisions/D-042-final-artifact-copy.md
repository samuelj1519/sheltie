# D-042: Raw final-result bytes and external copies

English | [简体中文](../../../zh-CN/explanation/decisions/D-042-final-artifact-copy.md)

Status: `accepted`
Date: 2026-10-03
Related change: [C006](../../history/changes/C006-result-delivery/README.md)

## Decision and rationale

`work result` is the sole source of result selection. The engine binds revision/key to one verified snapshot and reads through the same confined file handle. stdout contains original bytes, stderr diagnostics, without host writes or receipts.

External `sheltie-export` uses only public CLI, without runtime dependencies or Store access. Its Cargo metadata is `publish=false` and `dist=false`; separate distribution remains undecided. Copies do not become a second result source.

Home and destination parent must be explicit real absolute directories without symlinks. Their objects/ancestors must not contain one another. Result metadata is limited to 1 MiB, each file to 32 MiB, and all results to 256 MiB. After receiving all files, independently verify sizes, digests, source-child exit, and read-back content. Complete required OS synchronization of private directories and manifest before publishing the whole directory with NOREPLACE.

Keys retain original slot strings, including empty/Unicode values, without ID revalidation. Reject NUL accurately when argv cannot represent it. Destination paths use safe source filenames and sorted indexes, created within confined parent handles; metadata cannot choose arbitrary destinations.

## Failure and guarantee boundaries

Existing objects are never overwritten. Post-move confirmation failure reports `publication_unconfirmed` without deletion/rollback. Retain state provably owned by this invocation. Reruns create independent copies without adopting old staging, adding another lifecycle, resumable transfers, or caches.

`complete` promises neither physical power-loss durability, same-permission isolation, nor continued integrity after user edits. See [export reference](../../reference/export.md) for arguments/manifests/exits and the [guide](../../how-to/export-results.md) for handling.

## Verification

T01 completed advanced primitives and independent oracles and ran actual-platform tests, independently checked at M1; T02 then exposed raw/export entry points. Code authors may `complete` coupled foundations but cannot omit independent review, valid feature behavior red evidence, or accurate failure boundaries. Related change snapshots retain original evidence.
