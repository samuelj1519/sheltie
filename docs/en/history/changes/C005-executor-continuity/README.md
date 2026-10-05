# C005: Attempt qualification revocation

English | [简体中文](../../../../zh-CN/history/changes/C005-executor-continuity/README.md)

Status: `completed`
Target version: `v0.3.0 (development target, unreleased)`
Compatibility: `no format compatibility; preserve old Stores and reject automatic migration`
Baseline: `3f2387f0daf8fb4e6e5329044babff9aa47654dc`
Owner: `Codex /root`
Record form: `reference`
Historical snapshot: `9d98bf8f15944b7bda4fbd4096e7771b09eab728`

## Changes and rationale

attempt replace marks the old running Attempt superseded and starts a new Attempt in one write. number is the creation sequence; only failed counts as business failure. Each Occurrence permits at most one replacement. Non-statistical inputs inherit frozen bindings; statistics are regenerated from state including the new Attempt.

Switch schema 4 / cli-result/v4 together. Preserve old Stores without automatic migration.

## Validation and limits

Native/MSRV technical verification and real-demand negative preflight are complete. Without actual revocation events, use ordinary resume rather than manufacturing incidents. The engine does not stop old processes, authenticate successors, isolate hosts, or undo external side effects. Real revocation value, human cost, and original LEAK causality lack sufficient evidence.

This page retains design/outcome summaries. Root specifications/contracts govern current behavior. Historical validation cannot directly qualify the current candidate as PASS.

## References

[Resume and revocation](../../../how-to/resume-work.md), [D-041](../../../explanation/decisions/D-041-attempt-number-and-replacement.md), [Current limits](../../../reference/limitations.md). Read full tasks, reviews, and original runs from the snapshot above using the [historical lookup guide](../../../how-to/maintain-docs.md#read-original-historical-records).
