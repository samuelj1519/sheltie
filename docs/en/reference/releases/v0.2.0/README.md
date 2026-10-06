# `v0.2.0` release record

English | [简体中文](../../../../zh-CN/reference/releases/v0.2.0/README.md)

Status: `released` (within user-authorized scope)
Version: `0.2.0`
Git tag: `v0.2.0`
Release commit: `5c7183a3644dd250069ed362dc1adeeb7fb71cc6`
Original published commit: `8455aed2bcd9ac1739852be3987091a0975e7ac3`
Real-host and resume acceptance closure: `ed73febf31bf039bf1e09b5e83ef22fd56a5b5a2`
Original publication: `2026-10-02T17:03:04Z` (UTC)
Release target: `aarch64-apple-darwin`

## Scope

The release tag points to the corresponding commit on `main`, with the same source tree as Original published commit. After repository recreation, original release assets are uploaded again with identical bytes and checksums. GitHub release records have new identities, and old workflow runs are unavailable. Recorded checks were not rerun, and no new binaries were built for this restoration.

Includes [C002 reliability](../../../history/changes/C002-v0.2.0-reliability/README.md): request identity/replay, file ownership, Workbook digest/freezing, transactional publication/recovery, fact views, coordinator resume, Workbooks/skill, and release governance. Remains three crates, one `sheltie` binary, and a local SQLite workflow engine.

Only macOS aarch64 packages were released. Linux/x86_64 were excluded by user instruction, with no packages in this version. Earlier failures/incomplete checks remain in C002 evidence, without cross-platform PASS. See [CHANGELOG](../../../../../CHANGELOG.md).

## Acceptance

- The [official release](https://github.com/samuelj1519/sheltie/releases/tag/v0.2.0) restores assets built from Original published commit above. Original workflow `37037558064` completed quality/build/host/announce with success; its GitHub run was removed when the repository was recreated. Committed evidence remains in the [original records](../../../how-to/maintain-docs.md#read-original-historical-records).
- The same source SHA passed engineering, docs/specs/skill/test governance, dependencies, and locked MSRV 1.85 gates. Nextest: 700/700, 0 skipped, 1 slow, 156.541 seconds in the test phase. See [original scope/output](../../../how-to/maintain-docs.md#read-original-historical-records).
- T16 locally completed examples, four `spec-dev` tasks, independent review, Work006 continuation after genuinely closing/reopening the session, and Work003 final retro gate after specific approval. The T16 commit above fixes closure; see [historical review](../../../how-to/maintain-docs.md#read-original-historical-records).
- After release, actual packages/manifests/binaries were downloaded and reverified, without reusing PR bytes. Default GitHub-source pinned `0.2.0` remote update/rollback ran in an independent root. Rolled-back bytes were identical, Store unchanged, prev removed. The source was T16's C002/schema 2 build showing `0.1.0`; this does not prove legacy schema 1 migration.

| Artifact | SHA256 |
| --- | --- |
| [macOS aarch64 package](https://github.com/samuelj1519/sheltie/releases/download/v0.2.0/sheltie-cli-aarch64-apple-darwin.tar.xz), 1779600 bytes | `fc63c9d0c7132796cdacd050be102ed5501f637f2db0f19b279da6b4744899e4` |
| Packaged `sheltie` binary | `a220309ab437a5de52202acb8b8f933370a45969e2027c55964930818e315130` |
| [dist-manifest.json](https://github.com/samuelj1519/sheltie/releases/download/v0.2.0/dist-manifest.json) | `8540728b9db73a31a74e7f762128bebed90210ecfb2ec24845786db02a02354b` |

## Compatibility

`v0.2.0` uses schema 2, `workbook-digest/v2`, new Work layout, cli-result/v2. Old schema 1 roots reject with `STORE_SCHEMA_MISMATCH`, without migration/clearing/new-format interpretation. Use a new explicit root, preserving old roots/binaries for history.

## Known limits

- M1 completed under authorized exceptions: SK01 lacked final Spec approval; SK02 lacked additional execution of 215 exact mutant IDs. Full mutation/security validation and final Spec approval remain unpassed; original FAIL, pauses, and missing records remain.
- This release does not turn Linux-native/cross-platform verification into PASS; Linux/x86_64 packages are excluded.
- T16 used a manually supplied skill; local Host-window access limits remain. Model usage is null. Work/Attempt durations include humans, closed sessions, and waiting, without conversion to usage/cost.
- OS principals/local gates do not authenticate independent humans. See [C002 originals](../../../how-to/maintain-docs.md#read-original-historical-records) and [review entry](../../../how-to/maintain-docs.md#read-original-historical-records) for limits, evidence, and completion scope.
