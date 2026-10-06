# C013 implementation plan

| ID | Status | Owner | Dependencies | Result |
| --- | --- | --- | --- | --- |
| C013-T01 | done | Codex | none | Validated and independently reviewed v0.3.0 source baseline and withdrawal governance |
| C013-T02 | done | Codex | C013-T01 | Old releases withdrawn, v0.3.0 published and actual artifacts verified |

## C013-T01 Prepare the current baseline

**Files.** Cargo.toml/Cargo.lock, CHANGELOG.md, specs, bilingual README/docs, scripts/check-specs.sh, release_governance.rs, build workflow comments.

**Steps.** Record upstream scope; retain current runtime formats and integrity checks; support withdrawn historical release records without tags while retaining commit qualification; align candidate documentation and installation commands; run complete gates and independent Standards/Spec review.

**Accepted case.** A withdrawn release with fixed source/acceptance commits and explicit withdrawal authority passes without its tag. A normal released record still resolves its exact tag.
**Rejected case.** Missing source/acceptance commits or withdrawal authority, and moving an existing historical tag, fail. Normal released records without tags still fail.
**Stop conditions.** Runtime compatibility layers are discovered, candidate validation fails, or remote refs drift; diagnose before publication.
**Validation.** Governance positive/negative fixtures; full engineering/deny/MSRV/docs/specs/language/skill/tests/Wiki/tool gates; independent review of diff from baseline.
**Commit.** `chore(release): establish the first supported v0.3.0 baseline`

## C013-T02 Publish and verify

**Files.** Package evidence, release records/indexes, first-screen release status, acceptance boundary wording, CHANGELOG withdrawal notes.

**Steps.** Verify backed-up release metadata/assets and Git bundle; push only reviewed main/tag; wait for same-SHA release quality/build/announce; verify real public bytes, manifest, architecture, fresh install, pinned update and rollback in independent roots; remove only v0.1.0/v0.2.0 Release IDs and guarded tag refs; mark withdrawal and actual publication truthfully; complete and commit this package.

**Accepted case.** Only v0.3.0 remains published; its public assets match the recorded SHA/version/platform and a fresh-root workflow succeeds.
**Rejected case.** A checksum/version/architecture mismatch or missing release quality stops acceptance; no unrelated tags/data are removed.
**Stop conditions.** Backup verification, new-publication gates, or public-asset validation fail; keep accurate partial state and repair within scope.
**Validation.** GitHub release/run readback, remote/local tag readback, public downloads and SHA256, Mach-O architecture, fresh installation/demo, isolated pinned update/rollback with exact binary/Store hashes, post-publication documentation governance.
**Commit.** `docs(release): record v0.3.0 publication and withdrawn previews`
