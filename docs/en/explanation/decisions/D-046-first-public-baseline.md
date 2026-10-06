# D-046: First supported public baseline

English | [简体中文](../../../zh-CN/explanation/decisions/D-046-first-public-baseline.md)

Status: `accepted`
Adopted: 2026-10-06
Related change: [C013](../../../../specs/changes/active/C013-first-public-baseline/README.md)

The owner confirmed that the newly recreated repository has no external users and authorized withdrawing the restored v0.1.0/v0.2.0 publications. Establish v0.3.0 from the current implementation as the first supported public baseline after candidate and actual-artifact verification. Retain the version number rather than assigning new contents to previously published names. This changes publication/support policy, not historical validation outcomes.

Keep current schema 4, cli-result/v4, workbook-digest/v2, and independently versioned Workbooks. Do not add legacy parsers, migration, dual formats, or old-command aliases. Current data validation, byte preservation on rejection, request replay, and crash recovery remain required. Resetting format numbers or deleting rejection tests would add risk without supporting a real consumer.

Withdrawn release records preserve fixed source and acceptance commits, original checksums, limits, and original qualifications. They require explicit withdrawal authority and remain verifiable without their removed tags. Any tag still present must match its recorded source commit; active release records continue to require exact tag identity. Removing releases does not rewrite Git history or authorize deleting user data. [D-043](D-043-development-target-authority.md) still separates development target, candidate, release, and acceptance.
