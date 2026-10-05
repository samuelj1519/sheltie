# D-041: Creation sequence and administrative replacement

English | [简体中文](D-041-attempt-number-and-replacement.zh-CN.md)

Status: `accepted`
Date: 2026-10-03
Related change: [C005](../../history/changes/C005-executor-continuity/README.md)

Current applicability: schema 4 / `cli-result/v4` and administrative replacement. This extends the single-format and frozen-read principles of [D-033](D-033-store-schema-2.md) and [D-040](D-040-result-resume-format.md), superseding their schema 2/3 and cli-result/v2/v3 versions without invalidating their remaining design reasons.

## Context

Revoking formal submission qualification must end the old Attempt and start the new one in a single transaction, rather than separate fail and begin calls. Business failure limits describe actual execution failures and must not be consumed by administrative revocation.

## Decision

Attempt suffixes use creation sequence `number`, with only this field set. Counts of `failed` and `superseded` derive from the fact sequence. Each Occurrence permits at most one replacement.

Store becomes schema 4 and public response snapshots cli-result/v4. `replacement_reason` must be present, though its value may be null; serde must not default a missing field to None. Preserve and wholly reject old schema 1/2/3 without migration or parallel retry/number fields. Workbook, result, and directory digest formats remain.

The new Attempt inherits old frozen non-statistical input references and entry source. runtime validates paths, sha256, and bytes with the same file handle. New stats use post-commit state including the new Attempt; requests record exact historical bytes.

Historical fail responses validate the original Failed Attempt and failure prefix through it, rather than current total failures or number. Replace audit records preserve the complete materialized bounded reason and compare it byte-for-byte with the old replacement_reason. Intent still records literal parameters or source file paths. This adds no state store; other commands retain their audit-summary policies.

## Consequences and verification

Replacement revokes formal interface qualification only. It does not stop processes, isolate hosts, or authenticate successors. Operators confirm treatment of old executors and shared workspaces; the engine does not turn external confirmation into its own verified fact.

Real consumers verify accepted/rejected cases, the one-replacement limit, `max_retries=0/1`, late submission, same-intent replay, one concurrent winner, and raw-byte recovery around COMMIT. Independent reviewers do not author reviewed repairs. See the [resume guide](../../how-to/resume-work.md) and [storage contract](../../../specs/contracts/storage.md).
