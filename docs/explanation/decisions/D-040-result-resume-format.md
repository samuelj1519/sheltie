# D-040: One Store format for results and resume

English | [简体中文](D-040-result-resume-format.zh-CN.md)

Status: `accepted`
Date: 2026-10-03
Related change: [C004](../../history/changes/C004-verifiable-delegation/README.md)

Current applicability: explicit terminal selection, frozen references, same-snapshot reads, and one strict format remain. This extends [D-033](D-033-store-schema-2.md) old-format rejection while superseding schema 2 / cli-result/v2 with schema 3 / cli-result/v3. [D-041](D-041-attempt-number-and-replacement.md) later supersedes only the version choice with current schema 4 / cli-result/v4. work-result/v1 and workbook-digest/v2 remain; fields are in the [storage contract](../../../specs/contracts/storage.md).

## Context

C004 adds terminal result declarations, current-Attempt resume pointers, and strict same-snapshot reads. The user stated there were no actual users at that time. One new format without migration is an engineering decision; it does not imply original records do not exist. Existing records must be preserved. Strict Store/frozen definitions cannot silently gain multiple response semantics in one format.

## Decision

Store becomes schema 3; the public envelope is cli-result/v3; result DTOs use work-result/v1. Workbook/Flow stay v1 with optional result defaulting false. Reject old Stores wholly, without migration/clearing. Keep workbook-digest/v2. Maintain one strict read/write closure.

Preparation and implementation use equally capable owners, preserving T01/M1/T02 boundaries, independent oracles, and independent reviewers who did not author the work. Complete coupled pure types and read primitives before exposing public commands, without placeholders or fake success in normal entry points.

## Rejected alternatives

- Two interpretations in one schema or defaults as migration: ambiguous frozen definition/response contracts.
- Live effect readiness in status cards: refresh precedes mark_published, leaving stale readiness.
- Assemble state/revision/effects from separate connections: concurrent publication can mix versions.

## Consequences and verification

Accurately reject and preserve old schema 1/2. status/result strictly verify related requests/audit/effects in one read snapshot. Other Works' unfinished effects do not affect results. Real CLI, SQLite, fault-window tests, and independent review verify common-source projections and read-only boundaries.
