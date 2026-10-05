# Support, compatibility, and trust boundaries

English | [简体中文](limitations.zh-CN.md)

This page describes current-source use. The [specification](../../specs/spec.md#current-product-environment) defines support; [acceptance](acceptance.md) and [releases](releases/README.md) establish historical/bounded conclusions.

## Versions and environments

The product targets local macOS aarch64/APFS. Other OS/architectures, non-APFS, and dedicated external-physical-device certification require future adoption. Physical non-UTF-8 names unconstructible on current APFS have no successful on-site traversal claim; byte-interface rejection tests do not replace actual media.

Released version is v0.2.0. Development source is 0.3.0-rc.1 with Store schema 4, cli-result/v4, work-result/v1, workbook-digest/v2. sheltie-export/authoring tools remain unreleased. Old schema 1/2/3 Stores are neither migrated/cleared nor interpreted as schema 4. Retain matching old binaries/roots for historical queries; use a new explicit development root.

## Operations and content

| Fact | Boundary |
| --- | --- |
| Attempt/Work succeeded | Execution/flow state only; content, code quality, user acceptance are separate |
| Gate approval/OS principal | Actual process-account call, without independent human authentication |
| resume/draft paths | Continuation pointers, without proof of existence/completeness/sealing |
| work result references | Bound to a successful terminal; ordinary queries do not reauthenticate all bytes |
| attempt replace | Revoke submission qualification without stopping processes, isolating workspaces, or undoing side effects |
| Read-only queries | No business advancement/effect recovery; [D-039](../explanation/decisions/D-039-sqlite-read-control-files.md) permits controls |
| Export complete | Required verification/non-overwrite/sync, without physical power-loss durability or same-permission isolation |

Before successors write shared workspaces, operators confirm old processes stopped or environments isolated. Editable copies are not result authorities; initial manifests do not guarantee later edits. See [export](../how-to/export-results.md) for errors/publication_unconfirmed and [resume](../how-to/resume-work.md) for pending effects.

## Scope of conclusions

Report implementation, bounded acceptance, actual user acceptance, release, and measurable benefit separately. New success does not close historical LEAK, cost, fair-comparison, or forensic gaps; see [acceptance](acceptance.md). Unknown costs are not zero. Attempt durations are not model usage.
