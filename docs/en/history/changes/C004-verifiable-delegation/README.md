# C004: Explicit results and reliable resume

English | [简体中文](../../../../zh-CN/history/changes/C004-verifiable-delegation/README.md)

Status: `completed`
Target version: `v0.3.0 (development target, unreleased)`
Compatibility: `no compatibility layer or data migration; implement only the adopted single contract`
Baseline: `3692c4eed62e22f57a94573635890c8b61405cc4`
Owner: `Codex /root`
Record form: `reference`
Historical snapshot: `9d98bf8f15944b7bda4fbd4096e7771b09eab728`

## Changes and rationale

Add terminal result declarations, work result, and current-Attempt resume pointers. Final results bind inputs/sealed outputs of a specific successful terminal Attempt, without guessing from newest files, report bodies, or summaries. Resume the same running Attempt from current status after session interruption.

Adopt one Store switch (schema 3, cli-result/v3, work-result/v1); C005 later establishes current schema 4 / cli-result/v4.

## Validation and limits

Actual agent document delivery, same-Attempt cold resume, and new-consumer reading were verified within bounded scope. Queries list frozen references without proving quality or rechecking source bytes. Technical outcomes cannot establish the original human comparison, total cost, net benefit, or missing historical query evidence.

This page retains design/outcome summaries. Root specifications/contracts govern current behavior. Historical validation cannot directly qualify the current candidate as PASS.

## References

[Resume guide](../../../how-to/resume-work.md), [Protocol contract](../../../../../specs/contracts/protocol.md), [Format decision](../../../explanation/decisions/D-040-result-resume-format.md). Read full tasks, reviews, and original runs from the snapshot above using the [historical lookup guide](../../../how-to/maintain-docs.md#read-original-historical-records).
