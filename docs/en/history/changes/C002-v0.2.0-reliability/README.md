# C002: Reliability repairs

English | [简体中文](../../../../zh-CN/history/changes/C002-v0.2.0-reliability/README.md)

Status: `completed`
Target version: `outside the product release`
Compatibility: `breaking (schema 2 rejects old databases; old roots/data are retained intact)`
Baseline: `de2b8832e87357571a0199076192020baf9e9fa8`
Owner: Claude (original implementation), Codex (later repairs and convergence); adopted by the user (2026-09-27)
Record form: `reference`
Historical snapshot: `9d98bf8f15944b7bda4fbd4096e7771b09eab728`

## Changes and rationale

Repair request identity and historical replay, managed-file identity, Workbook digest framing, transactional publication/recovery, deletion proofs, statistical snapshots, and coordinator resume. Verify complete identity/input closure before file effects. Historical responses retain commit-time facts; current queries use one read snapshot.

Switch schema 1 to schema 2, explicitly rejecting old databases without dual interpretations or automatic migration. Current storage contracts govern later source formats.

## Validation and limits

v0.2.0 was released for macOS aarch64; release records retain artifacts, checksums, and authorized exceptions. Later recovery acceptance covers only inputs constructible on macOS aarch64/APFS. The 215 dispositions include dynamic, bounded static, and structural checks, without execution of every historical native mutant. Original SK01/SK02, historical LEAK, and old forensic limits retain original outcomes; new runs cannot fill missing past evidence.

This page retains design/outcome summaries. Root specifications/contracts govern current behavior. Historical validation cannot directly qualify the current candidate as PASS.

## References

[v0.2.0](../../../reference/releases/v0.2.0/README.md), [Storage contract](../../../../../specs/contracts/storage.md), [File handles](../../../explanation/decisions/D-037-managed-file-handles.md), [Read-only SQLite boundary](../../../explanation/decisions/D-039-sqlite-read-control-files.md). Read full tasks, reviews, and original runs from the snapshot above using the [historical lookup guide](../../../how-to/maintain-docs.md#read-original-historical-records).
