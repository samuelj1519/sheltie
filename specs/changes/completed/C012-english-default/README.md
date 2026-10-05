# C012: English-default project migration

Status: `completed`
Target version: `v0.3.0`
Compatibility: `minor`
Baseline: `206a944491acfa9fa4843ced06103427e13054aa`
Owner: `Codex`
Authority: adopted by the user's 2026-10-05 instruction; language and repository migration only

## Problem

The project is primarily documented and presented in Chinese, which prevents English-speaking users and contributors from using its existing behavior and workflow instructions.

## Success criteria

- English is the default for current documentation, developer instructions, source comments, Rust API documentation, CLI output, status cards, and the Workbook editor.
- Chinese documentation, Workbooks, examples, and coordinator instructions remain explicitly discoverable; authoritative rules and machine identities remain consistent.
- Existing commit messages are translated to English. Their trees, author/committer identities and timestamps, header metadata, ordering, and merge topology are preserved; parent object IDs change only through the recorded mapping. Source references are synchronized without altering original evidence.
- New commit messages use English. MIT license text and ownership remain intact in GitHub's conventional root LICENSE file, with valid crate packaging links.
- Unicode fixtures, user data, and original evidence are inventoried with an explicit retain/translate/remove decision.
- Documentation, governance, delivery, editor, Rust, and affected real CLI scenarios pass; model-effectiveness claims require an actual comparison and otherwise remain not_run.

## Scope boundaries

No product capability, persistent schema, supported-platform expansion, host installation, model invocation, or external publication is implied. History changes are local; remote refs and published releases are not force-pushed. Existing Chinese historical file trees remain unchanged because this is a commit-message-only rewrite.

## Entry points

- [Plan](plan.md): migration stages and acceptance.
- [Inventory](inventory.md): language scope and retained data.
- [Validation](validation.md): actual checks and evidence.
- [Progress](progress.md): handoff.
- [Review](review.md): independent findings.
