# Engineering rules

For people and agents writing code or documentation, or reviewing this repository. Readers know Rust but may be new to Sheltie. This document defines how to work; product semantics are in the [specification](spec.md), and mechanisms in the [architecture](architecture.md) and [contracts](contracts).

## 1. Specifications before code

1. Find the authority before starting. Product behavior: `spec.md`; types and modules: `architecture.md`; fields, commands, and tables: the relevant contract; current tasks and progress: the active change's `plan.md`. Start at the [change index](changes/README.md).
2. If authority is missing or conflicting, **update upstream documentation before code**. Product questions belong in `spec.md`; mechanisms in architecture/contracts; important cross-task decisions in an ADR; task order in the active plan. Include the document change in the commit. Before naming a third-party library in a contract or task, verify that its public API supports every required operation (M3 lesson D-30: `axoupdater` proved unsuitable after it had been written into the contract).
3. Specifications describe the target, not execution progress. Only the active plan's task states and Git history describe implementation progress. When no change is active, do not independently implement proposed packages. Do not present planned capabilities as supported.
4. Define each fact once and link to it elsewhere. `scripts/check-docs.sh` checks links and prohibited wording.

An unreleased Cargo candidate's base version comes only from the `Development target` field on the first screen of the [specification index](README.md). Numeric active product targets must match it. Completed changes, non-product experiments, and the absence of an active change do not turn a candidate into a release. Verify released versions against the release/tag/history and CHANGELOG; see [D-043](../docs/en/explanation/decisions/D-043-development-target-authority.md).

## 2. Rust conventions

### 2.1 Workspace

See [implementation entry points](../docs/en/reference/implementation.md) for directories and sources. Manage dependencies in `[workspace.dependencies]`; release configuration is in root Cargo.toml, and changes must verify actual distribution consumers.

Engine dependencies flow only downward: `cli → runtime → core`. The exporter may depend on pure core and shared safety libraries, not runtime. `sheltie-core`'s Cargo.toml must not contain `rusqlite`, `tokio`, `rand`, or filesystem/clock libraries.

### 2.2 Code style

- Every crate forbids `unsafe`; root `[workspace.lints.rust] unsafe_code = "forbid"` enforces this. Library crates also deny `clippy::unwrap_used` and `clippy::expect_used`; tests may unwrap. OS operations use the safe Rust APIs selected by D-036, without direct `libc` calls.
- Each crate has an `Error` enum using `thiserror`. Errors carry locating fields such as paths, field names, and rule names; CLI maps them to [protocol](contracts/protocol.md) §7 codes. Do not carry `anyhow` across crate boundaries; it is allowed within CLI.
- IDs, paths, digests, and bounded text use newtypes with validating constructors and private fields. Module boundaries do not pass bare `String` values.
- Complete contract payloads use `#[serde(deny_unknown_fields)]`. Enums use `rename_all = "snake_case"`. Private read-only identity projections may decode only required fields, but their success does not establish validity of the complete payload. Strictly decode complete Commands and successful snapshots, including data, before reading frozen business files. Validate and strictly decode the complete effect closure under storage contract §3.2 before any effect action. Verifying original-response qualification may read only frozen definitions; an effect error must not discard a verified legitimate original. Projections serve existing obligations rather than a new payload entry point.
- Model state machines with enums and exhaustive matches, without catch-all `_ =>` branches.
- Functions are private by default; expose `pub` only for actual external callers. Do not add traits solely for mocks.
- Comments explain reasons the code cannot express. Avoid phase narratives and descriptions of obvious operations. Comments and Rust API documentation use English.
- Business terms such as Review, Verdict, Pass, Fail, Spec, Plan, and Git must not appear in core/runtime type names, variant names, or branches. `scripts/check-core-vocab.sh` checks source in CI.

### 2.3 Commands

Run from the repository root. Choose required checks according to actual impact: source, fixtures, instructions, generated inputs, and build configuration can all be consumer inputs. File extensions alone do not determine the scope.

| Change | Required checks for each commit | Complete engineering gate |
| --- | --- | --- |
| Wording, proposals, or experiment records only | docs/specs, TOML, and fact/reference review; check-tests when test declarations are affected | Unrelated Rust reruns are not required |
| Workbook, skill, fixture, or generated input | Above checks plus actual parsing, CLI scenarios, or generating consumers | Expand when the impact boundary cannot be established |
| Rust implementation | fmt/check/clippy, task tests, and affected real consumers; deny for new dependencies | Required for cross-crate public interfaces, protocol, state, or persistent-format changes |
| Stable product milestone or release candidate | Complete adopted closure and independent review | All four commands below, plus deny and documentation/specification/skill/test governance as applicable |

The complete engineering gate is:

```bash
cargo fmt --all -- --check
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run --all-features --no-tests=pass
```

Also available: `cargo deny check` (dependency licenses and advisories), `scripts/check-docs.sh`, `scripts/check-specs.sh`, `scripts/check-tests.sh`, `scripts/check-skill.sh`, and `scripts/check-core-vocab.sh`. CI runs its complete requirements on its own candidate and environment. Local pre-commit hooks filter files; they do not replace the task's assessment of indirect consumers. Store manual validation in the package, without assuming hook support for result reuse.

## 3. Tests: red before green

### 3.1 Cycle

Follow [complete behavior implementation](../docs/en/how-to/implement-change.md) to prepare interfaces, independent expectations, and real consumers. A capable model or equivalently experienced author prepares high-risk primitives and necessary tests; implementers complete bounded work on frozen interfaces; a Reviewer who did not prepare or implement them checks the resulting behavior.

Reproduce a defect first, or establish the expected failure for new behavior, then implement and verify. Compilation failure, environment denial, and zero tests do not establish a behavioral red result.

- Test names describe behavior, without task IDs. Task comments sit immediately above test attributes and are checked by ownership tools.
- Every capability has an accepted case and a rejected case changing only one condition. Every mandatory/prohibited rule has a rejection oracle.
- Expectations come from contracts, handwritten bytes, or independent calculation; they do not call the implementation under test to compute the same answer.
- Every size/count limit has exact and +1 cases. Every numeric field has at least one nonzero, nondefault assertion; construct time differences independently.
- Values required by interface documentation come from parameters, not guesses within the function.
- Tables, dates, carry, and correction terms cover their effective ranges; search reachable inputs when needed.
- Replay/recovery assertions verify exact bytes or digest equality, not mere file existence.

### 3.2 Layers

| Layer | Location | Behavior | Tools |
| --- | --- | --- | --- |
| Core unit | `crates/sheltie-core/src/**/tests.rs` or `tests/` | Parsing, compilation, `decide`, `legal_next`, status-card rendering; handwritten input, fixed time/IDs | `#[test]`, `insta` status snapshots |
| Runtime integration | `crates/sheltie-runtime/tests/` | Atomicity, replay, `REVISION_CONFLICT`, sealing, path boundaries, old-store refusal | `tempfile` with independent `SHELTIE_HOME` |
| CLI end-to-end | `crates/sheltie-cli/tests/` | Complete example Workbook scenarios, parseable JSON, exit codes | `assert_cmd` and `tempfile` |
| Crash | `crates/sheltie-runtime/tests/crash.rs` | Faults before/after COMMIT, restart consistency | Fail-points or killed subprocesses, temporary directories |

Locate built executables through `cargo build --message-format=json`'s `executable`, rather than guessing target paths. Fakes replace external boundaries (clock, IDs, file observation), never directly record success into state.

### 3.3 Insufficient verification

- Compilation alone.
- Internal counters or `is_ok()` alone.
- A test computing expectations with the implementation's helper.
- Deleting failed cases or changing expectations to match current output.
- Repeating a flaky failure until it passes. Diagnose the cause first.

## 4. Commits

- One active-plan task per commit. Applicable §2.3 checks and the plan's additional gates must pass first. Preserve MVP legacy task mappings.
- Do not leave uncompilable intermediate states or add empty implementations, fake success, or lasting compatibility layers merely to compile.
- Update all callers, fixtures, and documentation for an interface change in the same commit.
- Use this format in the repository and the default spec-dev Workbook:

```text
<type>(<scope>): <English summary, one line, at most 72 characters, no final period>

<Body: what changed, why, and actual validation. One idea per paragraph; do not repeat the diff.>

Change: C002
Task: C002-T05
Work: 2026-09-24-001-example
Agent: Claude
```

`type` is `feat | fix | refactor | test | docs | chore | perf | revert`; `scope` names the crate or directory, such as `core`, `runtime`, `cli`, `specs`, or `workbook`. `git-cliff` groups by type/scope. New summaries and bodies use English. These trailers form one block: Change/Task identify the new iteration's active package; Work is included only when running within a Sheltie Work; Agent identifies the actual committer and is required. MVP T01–T26 keep the legacy Task-only format. Omit inapplicable trailers instead of placeholders. Other trailers, including Co-Authored-By, share the block without blank lines.

After committing, inspect `git show --stat` and confirm that only task files are included. Historical message translation preserves each commit's tree, identities, timestamps, and other metadata; its parent IDs follow the recorded mapping.

## 5. Review checklist

Reviewers do not write the code they review. Apply each row; explain inapplicable items.

| Item | Question |
| --- | --- |
| Authority | Does each behavior point to a specification, contract, or plan rule? Is any behavior undocumented? |
| Invariants | Are INV-1 through INV-7 affected? Does core perform I/O or runtime judge business content? |
| Cases | Does every rule have accepted/rejected cases, with one-condition rejection? Does every must/must-not have an oracle? |
| Real chain | Do end-to-end tests use the CLI and real temporary directories rather than private functions? |
| Crash | Do new write paths survive faults before/after COMMIT? Are effects idempotent? |
| Boundaries | Are external paths confined? Are limits tested at exact/+1? Are unknown fields rejected? |
| Documentation | Do contracts, examples, status snapshots, and implementation agree? Does check-docs pass? |
| Commits | One task per commit, valid messages, and no unrelated files? |
| Mutation | Does each surviving mutant have a disposition: new coverage or dead-code removal? |
| Evidence | Does each conclusion have a repeatable command and complete output? Were rebuttals verified individually? |

Conclusions: PASS, changes required (each finding and authority), or BLOCKED (missing input).

Wording cleanup must not silently change frozen tests. Necessary changes require independent verification against contracts and baselines, then refreezing under the task plan. MVP tNN-review applies only to historical tasks.

The plan defines task/milestone review scope. Store final review, candidate hash, and input closure in package review/validation. After completion, follow [documentation maintenance](../docs/en/how-to/maintain-docs.md); original qualification remains verifiable.

## 6. Documentation

- English is the default. Use short sentences, one idea per paragraph, and [CONTEXT.md](../CONTEXT.md) terminology. Maintain Chinese reader versions only in docs and the root README. Specifications, repository instructions, changelogs, and tool guides elsewhere have one English version. Retain explicit Chinese Workbook and skill instructions; keep method resources actually consumed by agents, including the Chinese spec-dev README revision history.
- Reader docs use matching `docs/en/<path>.md` and `docs/zh-CN/<path>.md` files; the root README uses `README.md` and `README.zh-CN.md`. Provide reciprocal language navigation. The English document is the default authority; translations must preserve every requirement, limit, exclusion, and evidence status. For semantic changes to a maintained reader pair, update both variants. Original historical evidence remains verbatim.
- Commands, paths, fields, and error codes use code formatting and actual symbols, without paraphrased identifiers.
- Tables compare parallel items; numbered lists describe sequential steps.
- Must and must not express requirements/prohibitions; may expresses an option. Avoid approximate requirements.
- Each document has one purpose: specifications define targets, contracts define fields, plans define steps, progress gives handoff, validation provides evidence, ADRs explain decisions, and this document defines working methods.
- Run `scripts/check-docs.sh` after documentation changes. Bilingual changes also check matching language links and actual method consumers.

## 7. Gaps

| Finding | Action |
| --- | --- |
| Product behavior/boundary question | Record a proposed change; after adoption update spec.md and an ADR for an important choice |
| Missing field, command, table, or state transition | Update the relevant contract |
| Incorrect task order/dependencies | Update the active plan |
| Implementation violates a defined rule | Fix it and add a rejection case |
| Local implementation choice | Decide within scope |

Do not invent a second state model, compatibility entry point, or temporary source of truth merely to pass a task.
