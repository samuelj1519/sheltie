# C012: English-default project migration

English | [简体中文](../../../../zh-CN/history/changes/C012-english-default/README.md)

Status: `completed`
Target version: `v0.3.0`
Compatibility: `minor; preserve product semantics, protocols, persistence and historical file trees`
Baseline: `206a944491acfa9fa4843ced06103427e13054aa`
Owner: Codex coordinator; worker agents; independent Reviewer
Record form: `reference`
Historical snapshot: `eec267b7b54b1e18e75578f7b5e269e7ddf73df8`

## Changes and rationale

Use English by default for documentation, developer and agent instructions, source comments, Rust API documentation, CLI messages, status cards, and the Workbook editor. Retain explicit Chinese document and method variants with language navigation. Preserve meaningful Unicode fixtures, user content, original evidence, and the persisted audit marker; the [language inventory](inventory.md) records the retained text and its purpose.

Translate 260 historical commit messages and map 43 local refs. Verify exact trees, non-parent metadata, identity, timestamps, and ordered mapped parents. Synchronize current references while preserving original published identities and remote-tracking refs. The [migration map](../../git-message-migration.md) and verified local recovery bundle retain traceability. Future messages use English. Keep the original MIT terms and ownership in the standard root LICENSE and add GitHub contribution, support, security, issue, and pull-request entry points.

## Validation and limits

The recorded engineering checks passed: 956/956 Rust tests, five doctests, formatting, all-target/all-feature compilation and Clippy, locked Rust 1.85 compilation, 72/72 editor tests, standalone editor/ZIP closure, ten bilingual CLI scenes, language and skill rejection controls, and independent review. A fresh dependency audit passed; a later advisory fetch failed with TLS, and the final commit hook passed using the previously audited database pinned at `ef6173cbc5c50ec8166f9a5b28f07834144373ee`. Preserve that failed fetch as FAIL.

Agent-language effectiveness, human acceptance, publication, remote history push, and other-platform validation remain `not_run`. CLI scenes used synthetic outputs and test approvals. Existing managed user data and installed/frozen methods were unchanged. The external bulk-translation route was authorized, but translations completed through parallel agents without executing that batch route. This reference records historical engineering outcomes; it does not qualify a later candidate or expand product acceptance.

## References

- [Language inventory and retained data](inventory.md).
- [Git message migration and recovery](../../git-message-migration.md).
- [Current language maintenance rules](../../../how-to/maintain-docs.md#english-defaults-and-chinese-variants).
- [Historical task mappings](../../../../../scripts/task-history/C012/ledger.md).

The complete plan, validation, independent review, object comparison, and original logs remain in the snapshot above at `specs/changes/completed/C012-english-default/`. Use the [historical lookup guide](../../../how-to/maintain-docs.md#read-original-historical-records), for example:

```bash
git show eec267b7b54b1e18e75578f7b5e269e7ddf73df8:specs/changes/completed/C012-english-default/validation.md
```
