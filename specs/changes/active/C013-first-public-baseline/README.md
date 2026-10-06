# C013: First supported public baseline

Status: `active`
Target version: `v0.3.0`
Compatibility: none; no legacy-format support or migration
Baseline: `6d0f37a7c27c8cc265b0d799365e6a48dd66102c`
Owner: Codex
Authority: adopted by the user on 2026-10-06; withdraw v0.1.0/v0.2.0 and publish the current implementation after validation

## Problem

The recreated repository has no external users according to the owner. Restored historical releases expose obsolete binaries and installation instructions. Establish one current support baseline while retaining original development history and evidence.

## Success criteria

- v0.3.0 is the first supported public baseline on macOS aarch64/APFS.
- Remote/local v0.1.0/v0.2.0 tags and their GitHub Releases/assets are removed after backups and new-candidate verification.
- Source, lockfile, installation routes, and bilingual documentation agree. Withdrawn versions remain historical records with fixed commits and original qualifications.
- Current schema 4, cli-result/v4, workbook-digest/v2, and Workbook versions remain unchanged; incompatible Stores are rejected without modification.
- Complete engineering, dependency, MSRV, governance, independent review, and actual public-asset/fresh-install/update/rollback verification are recorded.

## Exclusions

No Git history rewrite, runtime format reset, data purge/migration, host installation, new platform support, or inferred cost/security/value acceptance. sheltie-export remains source-only; the local authoring tool remains available from the source tree.

## Document entry points

- [Plan](plan.md), [handoff](progress.md), [validation](validation.md), [task ownership](tasks.toml).
- [Release baseline decision](../../../../docs/en/explanation/decisions/D-046-first-public-baseline.md).
