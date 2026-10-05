# Task execution and validation budgets

English | [简体中文](../../zh-CN/how-to/validate-change.md)

For planning, implementation, review, and acceptance. Identify behavior/independent oracles before choosing scope/cost; add complexity for actual needs. Follow [engineering §2.3](../../../specs/engineering.md#23-commands) and active plans for mandatory gates. This guide explains scope/evidence reuse. Automatic hook reuse is unimplemented, without available-feature claims.

## 1. Freeze delivery and validation scope at start

Record these in task cards, without a separate progress system:

| Item | Required question |
| --- | --- |
| Delivery | What observable behavior is needed? What is outside this task? |
| Authority/entry | Which contracts/real callers/write-recovery-failure-stop paths? |
| Cases | Accepted/one-condition rejected cases? Independent expectation source? |
| Checks | Which obligation per command? Available reusable evidence? |
| Budget | Expected counts, same-scope baseline time, concurrency, remaining range? |
| Completion | Required items, explicit exceptions, candidate, Reviewer present? |

Wording/license metadata/docs organization do not automatically require full-product review. Boundary repairs need real chains, without substituting compile/unit passes. Assign new issues by impact/priority. Style preferences, abstractions without near-term consumers, and extra performance experiments do not expand acceptance automatically.

## 2. Confirm behavior with short feedback before long validation

1. Reproduce with the smallest real caller and exact failure signal. Use independent path/byte/identity/state expectations rather than implementation-generated answers.
2. Repair and run affected positive/negative cases including stops/recovery. Update all real consumers for interface changes.
3. Before batch mutation, review trust transitions, side-effect timing, and whether failures still release successful snapshots.
4. Freeze candidates after feedback converges. Do not change long-run inputs. Retain old outcomes for real defects, fix first, then derive invalidated scope from dependencies.
5. After affected chains, run milestone/release-wide gates. Finish once task cards are satisfied; expand only for new changes/failures/unresolved concerns.

Mutation/specialized checks remain useful for unknown defects. Avoid repeated full pipelines before known defects/oracles are resolved. Budgets cannot turn required incomplete checks into PASS.

## 3. Select gates by impact

These apply engineering layers; extensions alone do not determine risk. Assess Cargo.toml license strings separately from dependency/feature changes.

| Change | First checks | Wider-validation trigger |
| --- | --- | --- |
| Wording/docs | Links/governance/fidelity/references | Real consumers when docs are test/generation/business inputs |
| License metadata only | Cargo metadata/package files/text/license policy | Dependencies/build/releases also change |
| Local repair | Format/compile/lint/affected modules/real caller cases | Unknown impact, cross-crate interfaces, milestones |
| Persistence/protocol/files/recovery | Complete affected chains/independent state-byte oracles/fault windows | Core state/public constraints change |
| Milestone | Whole gates/contract closure on frozen inputs | Affected input changes invalidate results |
| Full mutation/platform/release | Separate task/scope/Owner/budget/raw output | Explicit plan requirement, without defaulting for every small fix |

Retain large-file exact/+1, native paths, and actual process-fault checks in related capabilities/whole candidates. Do not reread huge trees for unrelated mutants. Prefer existing real interfaces/small inputs for feedback; no test-only public traits or duplicate production configuration for speed.

## 4. Measure throughput before larger batches

Run same-configuration baselines and a few representative cost types first. Confirm nonzero actual tests, real artifacts, and successful build/execution. Record wall time/completions/failures/timeouts/concurrency. Do not combine means across different scopes.

For representative same-configuration samples, estimate remaining range as completed-sample wall time / sample count × remaining count. The mean already includes concurrency: do not divide again. Small samples/timeouts/load variation reduce confidence, without certain promises.

For repeated similar timeouts, zero tests, wrong artifacts, or time beyond upper estimates, stop dispatching similar batches and classify:

- Real failure: repair implementation/oracle, then assess impact.
- Hang: minimize trigger, retain timeout output; timeout is not caught.
- Congestion: reduce mutation jobs × Cargo jobs × Nextest threads and inspect I/O/memory instead of merely extending timeout.
- Tool/input error: verify package/filter/features/target/binary sources; baselines need actual cases.

Use one responsible long-run entry per input. Resume by checking processes/locks/inputs/progress/raw output, without starting another pipeline. Manage only self-started verified processes. Exceeded budgets trigger adjustment/reporting, not waivers/completion.

## 5. Verify mutations through real consumers

Generate full scoped inventories first, binding every ID to diff/capability/consumer/group. Crate-only tests miss CLI consumers; whole-workspace reruns for every missed item multiply unrelated costs.

Record real scopes of valid direct detections. Reuse cheap exact current producer/consumer equivalence or inapplicability proofs; otherwise select capability consumers/independent oracles. Escalate unclear impact to full workspace. Filters must execute nonzero tests. Baseline complex parameters first; zero tests are not PASS.

Group shared oracles/proofs while retaining each exact ID. Humans handle anomalies/unclassifiable differences. Prefer clear detection when proofs cost more. Do not write mirror tests for each survivor or shorten required inventories/omit IDs due cost.

Bind controlled observation copies separately from formal candidates. Supplemental detection does not rewrite original Missed/Timeout labels. Report pauses/not_run/unbuildable/static-equivalence/caught separately. Missing evidence stays missing; coverage modes/accounting closure are not verification success.

## 6. Do not rerun identical gates on identical inputs

Reusable records include input closure/actual command/scope/exit/test count/run ID/original location. Derive closure from real dependencies: source/generated files/fixtures/locks/config/toolchain/features/profile/filter/environment, plus history/docs/external inputs where consumed. Identical SHA is not automatically enough; docs-only changes do not invalidate every result automatically.

Separate executed/reused from PASS. Rerun changed closures or conservatively when equality is unproven. Hook reuse requires dedicated implementation/verification. CI validates its own candidate/environment, without inferred PASS from local records.

## 7. Review and preserve evidence once effectively

Scope reviews to actual risk. Full product/boundary/milestone reviews need both axes; simple metadata/fidelity changes may use one independent Reviewer. Incremental checks cover affected chains, without rereviewing unchanged parts. Separate defects/blockers/optional advice; finish required work.

Keep conclusions/candidates/commands/gaps in package review/validation with one structured index of shared originals. Compress/freeze originals instead of duplicating data/full source/huge JSON across stages. Before deletion, verify byte-level recoverability, especially ignored logs. Across machines, ensure referenced objects/archives are actually available: hashes in text are not backups.

Read historical diagnosis/timing originals from [fixed snapshots](maintain-docs.md#read-original-historical-records).
