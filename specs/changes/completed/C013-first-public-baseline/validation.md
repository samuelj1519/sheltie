# C013 validation

Candidate: `063029a9d9a41ef97ee8be48c12187312e599d1c`

The fixed candidate is the C013-T01 release source commit. [Input closure](evidence/t01/input-closure.json) pins tracked/new source, tests, contracts, methods, generated inputs, and documentation; package progress/evidence is excluded from executable consumer inputs. Runtime version is verified through Cargo JSON's actual executable. Rust gates used isolated target and RUSTC_WRAPPER=; Nextest 0.9.145 meets the repository requirement. Stable is Rust 1.98.1; locked MSRV is 1.85.0.

## Final T01 validation

| Requirement / risk | Mode | Input closure | Command / raw run ID | Result | Evidence |
| --- | --- | --- | --- | --- | --- |
| Formatting/check/lints | executed | T01 Rust/Cargo closure | cargo fmt/check/clippy | PASS | [fmt](evidence/t01/fmt.log), [check](evidence/t01/check.log), [clippy](evidence/t01/clippy.log) |
| Complete Rust consumer tests | executed | T01 executable closure | cargo nextest run --all-features --no-tests=pass --offline | PASS | [961 passed, 0 skipped, 1 slow](evidence/t01/nextest.log) |
| Locked MSRV | executed | T01 Cargo closure; Rust 1.85.0 | cargo +1.85.0 check --workspace --all-targets --all-features --locked --offline | PASS | [MSRV](evidence/t01/msrv.log) |
| Dependency licenses/advisories | executed | T01 Cargo.lock and fresh advisories | cargo deny check | PASS | [deny](evidence/t01/deny.log) |
| Docs/specs/language/test/skill governance | executed | T01 paired docs/specs/source | scripts/check-docs.sh; check-specs.sh; check-skill.sh; check-tests.sh; check-core-vocab.sh; check-language.py; typos | PASS | [docs](evidence/t01/docs.log), [specs](evidence/t01/specs.log), [skill](evidence/t01/skill.log), [tests](evidence/t01/test-governance.log), [vocabulary](evidence/t01/core-vocab.log), [language](evidence/t01/language.log), [typos](evidence/t01/typos.log) |
| Wiki source/exporter | executed | T01 reader docs and Wiki inputs | scripts/wiki.py check; unittest test_wiki.py | PASS | [Wiki](evidence/t01/wiki.log), [24 tests](evidence/t01/wiki-tests.log) |
| Authoring actual CLI/HTTP consumer | executed | T01 actual default-feature binary; pinned npm dependencies | SHELTIE_EDITOR_ENGINE=<Cargo executable> npm test; node scripts/smoke.mjs | PASS | [72 tests, 0 skips](evidence/t01/editor-tests-final.log), [smoke](evidence/t01/editor-smoke.log), [version](evidence/t01/source-version.json) |
| Actual distribution plan | executed | T01 Cargo/dist config and CHANGELOG | dist 0.32.0 plan --output-format=json | PASS | [single aarch64 target; exporter excluded](evidence/t01/dist-plan.json) |
| Independent Standards/Spec review | executed | T01 working diff from baseline | baseline_standards_review; baseline_spec_review | PASS | [review](review.md) |

## Diagnosis and input changes

[Red](evidence/t01/red.log) reproduced two governance failures before the withdrawal branch was implemented; [focused green](evidence/t01/governance-green.log) passed 39 tests before the additional accepted-authority rejection was added. Full Nextest includes all 40 governance tests and is the final Rust oracle.

The first editor run lacked pinned local dependencies ([output](evidence/t01/editor-tests.log)); the next lacked the mandatory real-engine environment ([output](evidence/t01/editor-tests-green.log)). Install dependencies and select the actual Cargo executable before the successful final run. Neither failed setup is recorded as product PASS.

Standards review required marking the CHANGELOG entry and assets as pending. Only CHANGELOG differed from the original input manifest during gates; executable Rust/tests/fixtures/contracts/methods were identical. Distribution and docs consumers were rerun after that wording correction. Candidate preparation does not establish publication.

## Compatibility audit

Store::check_schema accepts only SCHEMA_VERSION=4 and exact table DDL. Public protocol is cli-result/v4; no legacy parser or serde aliases were found on the CLI/Store boundary. Existing defaults implement current optional Workbook fields rather than historical format migration. Old-format rejection and unchanged-byte tests remain intact. No runtime compatibility code was removed or introduced.

## Publication and excluded coverage

C013-T02 publication and public-asset checks are recorded below; they do not extend the excluded coverage. Linux/x86_64/non-APFS/external-device certification, independent security completeness, and measurable human/model cost benefit are not established by these gates.

## Final T02 validation

| Requirement / risk | Mode | Input closure | Command / raw run ID | Result | Evidence |
| --- | --- | --- | --- | --- | --- |
| Exact release candidate and remote quality | executed | Tag v0.3.0 = 063029a; same-SHA macOS CI | Release run 37417410662 | PASS | [all six jobs](evidence/t02/release-run.json), [full raw output; 961 passed, 0 skipped](evidence/t02/release-complete.log) |
| Public artifact identity and platform | executed | Actual public Release 404343380 assets and dist manifest | verify-public.py SHA256/size/manifest/file checks | PASS | [summary](evidence/t02/public-verification.json), [metadata](evidence/t02/new-release.json), [script](evidence/t02/verify-public.py) |
| Public shell installer/fresh-root Work | executed | Downloaded v0.3.0 installer/package; current frozen examples | unmanaged installer; install/add/start/begin/submit/status | PASS | [installer](evidence/t02/public-installer.log), [complete CLI responses](evidence/t02/public-cli-evidence.json) |
| Actual pinned remote update and rollback | executed | Default GitHub source; unpublished schema-4 candidate 6d0f37a to public 0.3.0 | self update --version 0.3.0; self rollback | PASS | [binary/Store hashes](evidence/t02/public-verification.json), [responses](evidence/t02/public-cli-evidence.json), [updater build](evidence/t02/updater-build.json) |
| Actual release skill matches contracts | executed | Public sheltie-skill.tar.gz from 063029a | scripts/skill-delivery.sh verify | PASS | [output](evidence/t02/public-skill.log); asset SHA in public verification |
| Recoverable exact-scope withdrawal | executed | Backed-up API asset sizes/digests and Git bundle; expected-old refs | DELETE exact Release IDs; atomic leased tag deletes; guarded local update-ref | PASS | [original metadata](evidence/t02/old-releases.json), [inventory](evidence/t02/recovery-inventory.json), [readback](evidence/t02/withdrawal-readback.json) |

The downloaded installer was executed with SHELTIE_CLI_UNMANAGED_INSTALL pointing under /private/tmp; it did not change shell files, receipts, credentials, or existing management roots. Its PATH-shadow warning is retained; all checks use absolute verified binary paths. The updater is a source-built 0.3.0-rc.1/schema-4 binary, not an old release: this validates binary transport/rollback without adopting legacy-format migration. Both update and rollback preserve the original Store hash exactly.

The source-release commit includes pre-publication status, which remains immutable. Current publication/withdrawal records are later documentation maintenance; runtime/Cargo/fixtures/contracts are unchanged. Original v0.1.0/v0.2.0 source/acceptance commits remain available after tag withdrawal. Recovery bundle and original asset bytes are retained at /private/tmp/sheltie-first-baseline-20261006; the repository records metadata/digests, not duplicate binary backups.

## Final publication-document checks

Docs/specs/language/test ownership/Wiki source/typos and both independent T02 review axes passed on the final publication record set. The original raw CI log includes trailing whitespace emitted by the runner; retain its bytes and exclude evidence from the authored-file diff whitespace check, consistent with the repository pre-commit evidence exclusions. These are documentation consumers; runtime, Cargo, tests, contracts, and method inputs remain the immutable 063029a release candidate, so unrelated Rust reruns are not required.
