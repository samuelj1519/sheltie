# C013 validation

Candidate: `SELF`

SELF identifies the C013-T01 commit containing this source and validation record. [Input closure](evidence/t01/input-closure.json) pins tracked/new source, tests, contracts, methods, generated inputs, and documentation; package progress/evidence is excluded from executable consumer inputs. Runtime version is verified through Cargo JSON's actual executable. Rust gates used isolated target and RUSTC_WRAPPER=; Nextest 0.9.145 meets the repository requirement. Stable is Rust 1.98.1; locked MSRV is 1.85.0.

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

C013-T02 actual remote publication, withdrawal, public-asset installation/update/rollback are pending, not_run. Linux/x86_64/non-APFS/external-device certification, independent security completeness, and measurable human/model cost benefit are not established by these gates.
