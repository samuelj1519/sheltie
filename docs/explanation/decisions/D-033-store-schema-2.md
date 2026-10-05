# D-033: Store schema 2 explicitly rejects old formats

English | [简体中文](D-033-store-schema-2.zh-CN.md)

Status: `accepted`
Date: 2026-09-27
Related change: [C002](../../history/changes/C002-v0.2.0-reliability/README.md)

Current applicability: one strict format, without automatic migration or deletion of old data, remains the rule. schema 2 / `cli-result/v2` was superseded by schema 3 / cli-result/v3 in [D-040](D-040-result-resume-format.md), then current schema 4 / cli-result/v4 in [D-041](D-041-attempt-number-and-replacement.md). [D-039](D-039-sqlite-read-control-files.md) limits permitted SQLite control-file changes during reads. Current fields are in the [storage contract](../../../specs/contracts/storage.md).

## Context

v0.1.0 schema 1 had persistent-format gaps (C002 findings O02–O05, N03): request fingerprints omitted targets; historical responses were assembled from current state; `requests` had no effect registration; creation DDL and `user_version` were not transactional. Repairs changed `requests` fields and semantics, WorkState structure (accumulated blocked facts), and directory digests. Earlier C002 considered two interpretations in one schema (LegacyV1, defaults for missing fields). Review found that this retained two meanings for one field and could not fix digest format.

## Decision

`SCHEMA_VERSION` increases from 1 to 2. All tables, `workbook-digest/v2`, new Work layouts, and `cli-result/v2` responses belong to **one format switch**, integrated together by C002-T07. Old databases (`user_version = 1`) are wholly rejected after read-only identification (`STORE_SCHEMA_MISMATCH`), with zero writes to database files before rejection. No automatic migration, clearing, or downgrade interpretation; old binaries also reject schema 2. v0.2.0 starts under a new explicit management root. Keep old roots and binaries for historical queries. Design evidence-backed migration only when a real need to continue old Works arises.

## Rejected alternatives

- Automatic schema 1 → 2 migration: old `requests` lack `intent_hash` and effect records, so snapshots cannot be reconstructed. Forced migration would invent history.
- Two interpretations in one schema plus `serde(default)`: one field has two meanings and replay semantics become indeterminate.
- New and old response shapes inside `reply_json`: callers cannot distinguish them, violating one contract per field.

## Consequences

After upgrading to v0.2.0, both writes and reads against old roots are rejected; use the old binary for old records. This is an explicit breaking change in 0.x, which README and update notes must explain. No tool may automatically delete old data.

## Verification

Use a schema 1 database as a rejection fixture. Assert `STORE_SCHEMA_MISMATCH`, unchanged main database and existing WAL bytes, and no creation of Home, engine lock, or business files. Check SQLite control files under D-039.
