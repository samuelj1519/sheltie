# C012 implementation plan

Task states: `todo | doing | blocked | done`. Completion requires validation of the actual final input closure and independent review.

| ID | State | Owner | Dependencies | Result |
| --- | --- | --- | --- | --- |
| C012-T01 | done | Codex | none | A verified English-default repository, bilingual reading and method paths, preserved Git metadata, and mapped history references |

## C012-T01 Migrate the project language and repository entry points

**Files.** Root documentation and configuration, docs/, specs/, crates/, scripts/, tools/workbook-editor/, examples/, workbooks/, skills/, and .github/. Do not edit unrelated external worktree files or host configuration.

**Stages.**

1. Preserve original Chinese documents, original refs, commit objects, and a recoverable Git bundle. Inventory meaningful Unicode data and original evidence.
2. Translate default documentation and instructions; add language navigation. Preserve normative strength, limits, status boundaries, and terminology. Update English authoring and commit rules in both current language variants.
3. Translate source/API comments and product messages. Preserve machine fields, error codes, format versions, data constraints, and test oracles; translate matching human-text expectations deliberately.
4. Preserve Chinese method trees and add distinct language selection. Use a new version when default Workbook bytes change. Parse and compile both versions and exercise real CLI consumers and skill delivery.
5. Update governance consumers for the English source while continuing to verify original historical snapshots. Normalize LICENSE and add focused GitHub contribution, support, security, issue, and PR entry points.
6. Translate historical commit messages in an isolated object store. Verify each old/new tree, non-parent headers, timestamp, identity, and mapped parent list before atomically updating authorized local refs. Preserve remote tracking state and original evidence; synchronize current references through an explicit old/new mapping. Do not push.
7. Run complete engineering and document gates, editor tests, bilingual method and CLI scenes, and independent review. Fix findings and record any model-effectiveness experiment as not_run unless it actually ran. Commit one coherent task after required gates pass.

**Positive oracles.** English CLI/help/status/editor text, working language links, compilable English/Chinese Workbooks, self-contained skill delivery, unchanged machine protocol, and commit metadata equality.

**Negative oracles.** Existing invalid input, confinement, replay, crash recovery, gate, and integrity refusals retain their error codes and behavior. Language checks reject untranslated default prose but permit declared Unicode fixtures and historical evidence.

**Stop conditions.** Unrelated worktree changes, changed commit trees or non-parent headers, signed-object incompatibility, missing source translation, lost contract meaning, or insufficient authority for a remote operation.

**Validation.** fmt/check/clippy/nextest with all features, cargo deny, docs/specs/skill/core-vocabulary/test governance, editor tests and smoke where available, real bilingual CLI scenes, commit object comparison, and independent review.

**Commit.** `chore(project): make English the default project language`, with Change: C012, Task: C012-T01, and Agent: Codex.
