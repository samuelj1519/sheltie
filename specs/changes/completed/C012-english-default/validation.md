# C012 validation

Candidate: `SELF`

Engineering source: mapped baseline `206a944491acfa9fa4843ced06103427e13054aa` plus this task's frozen working tree. Exact production/document/tool inputs and bytes are in [source-manifest.json](evidence/source-manifest.json). Rust source remained unchanged after the successful final run.

| Requirement / risk | Mode | Input closure | Command / raw run ID | Result | Evidence |
| --- | --- | --- | --- | --- | --- |
| Commit-message-only rewrite | executed | All260 original/replacement objects and43 local refs | Raw object/header/ordered-parent verification and guarded transaction | PASS | [object proof](history-object-verification.json), [application](history-application.json) |
| English defaults and declared Unicode | executed | Final default source and declared literals | `python3 scripts/check-language.py` | PASS |56 distinct declarations, reciprocal language links; [inventory](inventory.md) |
| Language/Chinese-skill rejection controls | executed | Isolated exact source copy | Default prose, self-selector, unknown command, and storage-mention mutations | PASS | [positive/rejection controls](evidence/language-skill-negative.json) |
| Rust formatting | executed | Final Rust source | `cargo fmt --all -- --check` | PASS | [fmt](evidence/fmt.log) |
| All-target/all-feature compilation | executed | Final Cargo/Rust inputs | `cargo check --all-targets --all-features` | PASS | [check](evidence/check-final.log) |
| Clippy | executed | Same final source | `cargo clippy --all-targets --all-features -- -D warnings` | PASS | [clippy](evidence/clippy-final.log) |
| Complete Rust regression | executed | Final source, nextest0.9.145, isolated target | `cargo nextest run --all-features --no-tests=pass --no-fail-fast` | PASS | [956/956,0skipped,1slow](evidence/nextest-final-closure.log) |
| Rust API examples | executed | Final source | `cargo test --doc --all-features` | PASS | [5doctests](evidence/doctests.log) |
| MSRV1.85 | executed | Locked all-target/all-feature closure | `cargo +1.85.0 check --workspace --all-targets --all-features --locked` | PASS | [MSRV](evidence/msrv.log) |
| Commit-time cached dependency policy | executed | Pinned previously fetched database and unchanged graph/policy | Supported offline cargo deny; same four categories | PASS | [cache audit](evidence/deny-pinned-cache.log), exact database SHA below |
| Dependency policy/fresh advisories | executed | Unchanged dependency policy, refreshed database | Process-only Git HTTPS configuration; `cargo deny check` | PASS | [fresh audit](evidence/deny-https.log) |
| Editor consumers and HTTP boundaries | executed | Final editor source and actual discovered binary | `SHELTIE_EDITOR_ENGINE=<verified path> npm test` | PASS | [72/72](evidence/editor-tests-final-ready.log) |
| Standalone editor and complete ZIP | executed | Actual local server and verified binary | `node scripts/smoke.mjs` | PASS | [startup/check/export/closure](evidence/editor-smoke-final-localhost.log) |
| Browser English presentation | executed | Final local editor, native Chrome viewport | cua_repl read-only DOM/screenshot inspection | PASS | [visual record](evidence/browser-visual.txt) |
| Bilingual methods through actual CLI | executed |5English/5Chinese Workbook versions, isolated Home |10add/show/verify/start/step/result scenes | PASS | [mechanical scenes](evidence/bilingual-cli.json) |
| MIT license/packaging pointers | executed | Original MIT text and3crate symlinks | Byte comparison to baseline; symlink resolution | PASS | Independent review; unchanged ownership/terms |

## Failures and repairs

Original failure logs remain. The first Rust run found translated field synonyms and an accidentally translated expected Unicode payload. Complete second run exposed the rest of those expectation families, English instruction/diagnostic mismatches, and a test-only table parser treating the new `Task` header as a Tnn row. Canonical field keys, unchanged UTF-8 expectations, operation-specific diagnostic fragments, and exact Tnn recognition were independently reviewed; typed codes, limits, bytes, identity, provenance, fault windows, and rejection cases remain intact. Final956/956, rerun after removing only excess EOF blanks from two embedded templates, supersedes those engineering failures without deleting their evidence.

The sandbox could not acquire the advisory lock; the authorized cache retry hit a global Git HTTPS-to-SSH rewrite. A process-only global/system-config override fetched the official RustSec database through HTTPS; host configuration/policy stayed unchanged. Editor HTTP tests initially failed listenEPERM; the scoped local-listener run passed72/72. These initial environment failures were not product PASS. A later fake-child run exited on TERM before the Node fixture registered its handler; the fixture now emits an explicit READY marker and uses a startup-tolerant5s test deadline, retaining the SIGKILL/closure/root-cleanup/sentinel assertions and the production30s limit. Its71/72failure remains recorded separately. The first repair filter had a syntax error; corrected selection executed actual cases.

Nextest's cached official0.9.145 archive SHA256 was reverified against its saved official release asset digest: `52ecaedb4f5af9267ef7ed02bc937d2a15a94ff96cb663080e81311f798c9905`. Default installed0.9.140 was not used to bypass the required version.

## Explicit limits

- Agent-language effectiveness/quality/cost comparison: `not_run`. READMEs specify actual preregistered trial steps; structural/CLI tests do not establish model efficacy.
- Human acceptance, release/deployment, remote history push, and other platforms: `not_run`; this is an authorized local English-default migration on macOSaarch64/APFS.
- Bilingual CLI scenes used synthetic output files and test approvals. They prove engine behavior, not real content approval or user benefit.
- Original published remote identities, historical file trees/logs, frozen installed Works/Workbooks, and user data remain unchanged.
- External bulk translation route was authorized after automatic review initially rejected it; all bulk translations completed through the current parallel agents, and no such external batch was executed.

Independent final review is recorded separately in [review.md](review.md).

## Commit-hook dependency environment

The initial existing-hook run passed all Rust/dependency gates but reported formatting/spelling fixes. The next normal hook run passed code/document/test gates, while a new HTTPS advisory fetch failed with TLS; [raw failure](evidence/precommit-tls-failure.log) remains FAIL. The dependency graph/policy is unchanged. Final hook verification uses supported offline mode against the previously fetched/audited RustSec database pinned at `ef6173cbc5c50ec8166f9a5b28f07834144373ee`, with all four categories passing in [cache check](evidence/deny-pinned-cache.log). This cache check is not a claim that the failed later fetch succeeded. A temporary tool PATH handles this run; installed tools, host Git configuration, and advisory policy are unchanged.

Source whitespace checks exclude only verbatim raw evidence: `git diff --cached --check -- . ':!specs/changes/completed/C012-english-default/evidence/**'`. Original SSH/renderer/test outputs contain intentional CR/trailing whitespace and are preserved rather than rewritten.

Final commit closure after reviewed index/link/spelling/EOF/regex changes is recorded separately in [commit-source-manifest.json](evidence/commit-source-manifest.json); the original reviewed closure remains untouched.
