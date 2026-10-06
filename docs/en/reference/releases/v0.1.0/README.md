# `v0.1.0` release record

English | [简体中文](../../../../zh-CN/reference/releases/v0.1.0/README.md)

Status: `withdrawn`
Withdrawal date: `2026-10-06`
Withdrawal authority: [D-046](../../../explanation/decisions/D-046-first-public-baseline.md)
Version: `0.1.0`
Git tag: `v0.1.0`
Release commit: `6af9501b7bc12e2b18ac6017b6c894dadfc43626`
Original published commit: `9f87188f3b0ad2523abc7e975ae98e96db6f492b`
Installation, publication, and quickstart closure: `4bf86f7be09c27bbee98a11825eb57704e4403b9`
Real-host and MVP acceptance closure: `7196697ea40c9c303d059279c02f9e3d2218efd9`

## Withdrawal

The owner authorized removal of the GitHub Release, its assets, and the local/remote tag because there are no external users. v0.3.0 is the first supported public baseline. This record retains original source identities, checksums, qualifications, and limits; it is not an installation or upgrade source. Read release-time files by the fixed Release commit above instead of the removed tag. Withdrawal does not revise original PASS/FAIL/not_run conclusions.

## Scope

The former release tag pointed to the corresponding commit on `main`, with the same source tree as Original published commit. After repository recreation, original release assets were uploaded again with identical bytes and checksums before withdrawal. GitHub release records have new identities, and old workflow runs are unavailable. Recorded checks were not rerun, and no new binaries were built for this restoration.

MVP implemented three crates, one `sheltie` binary, Workbook/Flow, Work state machine, SQLite Store, CLI, self management, three examples, `spec-dev`, and coordinator skill.

See [MVP reasons](../../../explanation/decisions/mvp.md). Complete tasks, milestones, and historical operations are in [fixed snapshots](../../../how-to/maintain-docs.md#read-original-historical-records). [CHANGELOG](../../../../../CHANGELOG.md) records user-visible changes.

## Acceptance

- MVP T01–T26 and M1–M3 completed; full plans are in [original records](../../../how-to/maintain-docs.md#read-original-historical-records).
- Historical T25/T26 manuals record `v0.1.0` four-platform publication, install/update/rollback, and real-host steps. They do not expand later support scope.
- First real use ran three Works in Claude Code, including an article-review back loop, recorded in those snapshots.
- CHANGELOG fixes `v0.1.0` changes; the project accepted MVP at 7196697.

## Known limits

Post-T26 review found reliability, directory readability, skill, and Workbook problems adopted into [C002](../../../history/changes/C002-v0.2.0-reliability/README.md). Later repairs do not rewrite `v0.1.0` release/MVP completion facts. See [current limits](../../limitations.md).
