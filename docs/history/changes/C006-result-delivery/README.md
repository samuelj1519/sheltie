# C006: Editable result copies

English | [简体中文](README.zh-CN.md)

Status: `completed`
Target version: `v0.3.0 (unreleased; external tool excluded from dist)`
Compatibility: `work-result/v1 and adopted source-read contract only; release versions identify sources`
Baseline: `eaba7100905229f6f908949b3d24c093d6cd4a09`
Owner: `Codex /root`
Record form: `reference`
Historical snapshot: `9d98bf8f15944b7bda4fbd4096e7771b09eab728`

## Changes and rationale

The engine exposes confined raw-byte reading bound to one snapshot. Unreleased external sheltie-export obtains final selections only through public CLI, without Store access. Export stages, verifies bytes/manifest, synchronizes, then publishes a new directory with NOREPLACE. Editable `completed` copies do not become a second result authority.

## Validation and limits

Actual agent consumption, retained old edits, repeated new copies, and cross-APFS-media chains received bounded acceptance. Virtual APFS cross-device tests do not certify external physical devices. complete means contracted verification, publication, and OS synchronization, without physical power-loss durability or same-permission isolation. The tool remains unreleased.

This page retains design/outcome summaries. Root specifications/contracts govern current behavior. Historical validation cannot directly qualify the current candidate as PASS.

## References

[Export guide](../../../how-to/export-results.md), [D-042](../../../explanation/decisions/D-042-final-artifact-copy.md), [Source entry](../../../../crates/sheltie-export). Read full tasks, reviews, and original runs from the snapshot above using the [historical lookup guide](../../../how-to/maintain-docs.md#read-original-historical-records).
