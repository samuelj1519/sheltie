# D-034: Length framing for `workbook-digest/v2`

English | [简体中文](../../../zh-CN/explanation/decisions/D-034-workbook-digest-v2.md)

Status: `accepted`
Date: 2026-09-27
Related change: [C002](../../history/changes/C002-v0.2.0-reliability/README.md)

## Context

v0.1.0 sorted paths and hashed concatenated `path\0contents` with SHA256 (O07). Two different legal trees, `za="zb\0X"` and `za="", zb="X"`, had identical digests. Missing boundary lengths caused an encoding collision without breaking SHA256. Review reproduced it with two real Workbook directories (`hash-framing.json`). Earlier C002 proposed freezing double SHA256 as `digest/v1`, which would freeze the defect.

## Decision

The new format is workbook-digest/v2. Its stream is domain prefix `"sheltie-workbook-digest/v2\0"`, BE64 file count, then each regular file sorted by path bytes as `BE64(path length) || path || BE64(content length) || content`, followed by **one** SHA256. Empty directories do not participate. Reject symlinks, hard links, and special files. Check per-file and total limits before streaming reads with exact counts. Independent tests hand-assemble expected bytes instead of calling production digest helpers. The old algorithm is not a production fallback: schema 2 stores only v2 digests ([D-033](D-033-store-schema-2.md)).

Additional decision: declared Workbook output paths use portable ASCII (`A-Za-z0-9._-`, segments ≤ 128 bytes). Case folding then fully detects filename aliases without a Unicode normalization dependency approximating filesystem behavior. Parsing rejects non-ASCII output paths.

## Rejected alternatives

- Double SHA256 (hash files, then concatenate hashes): no length framing, so the structural collision persists in another form.
- Keep v1 and distinguish by field prefix: two semantics in one column violates D-033's single field contract.

## Consequences

New installs and frozen copies use v2. Old v1 digests are rejected with their old database, without mixed use.

## Verification

Independent vectors hand-assemble streams and expected hashes. The two `za/zb` boundaries differ; reordering is stable; changing one path/content byte changes the digest; excess limits and nonregular files are rejected.
