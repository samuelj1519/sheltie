# Product design and Rust engineering sources

English | [简体中文](product-rust-design.zh-CN.md)

Checked: 2026-10-03. This document records primary sources and their applicability. It has no product or implementation authority; project goals, actual needs, and invariants determine choices.

## User problems and complete outcomes

[GOV.UK: Learning about users and their needs](https://www.gov.uk/service-manual/user-research/start-by-learning-user-needs) calls for understanding possible users, existing practices, difficulties, and required outcomes, and treating unverified suggestions as hypotheses. Its service domain differs from this project. Transferable methods include problems before solutions, observation of real users, and validating needs through outcomes.

The project applies this approach to distinguish method authors from task users, observing first use, reuse, and interruption costs across complete tasks before deciding general increments. Historical experiment sample sizes reflect the project budget, not a statistically required size from that source. Limited samples cannot prove general benefit; see [workflow evaluation](../how-to/evaluate-workflows.md).

[GOV.UK: Measuring success](https://www.gov.uk/service-manual/measuring-success) provides entry points for service performance and benefit measurement. This project reports total investment separately from delivery quality, including preparation, checking, failures, and incomplete work. Workflow state or local time savings do not establish user value. These are project-specific deductions, not universally mandatory metrics.

## Concrete Rust types and real consumers

The [Rust API Guidelines checklist](https://rust-lang.github.io/api-guidelines/checklist.html) covers type safety, documentation, reliability, and future evolution. Useful practices here include validating types, private fields, locating errors, explicit contracts, and actual callers. The existence of guidelines does not justify a trait, public constructor, or platform abstraction in every module.

[The Rust Programming Language: Test Organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html) distinguishes internal unit tests from integration tests using public interfaces. The project separately verifies pure core rules, real runtime/SQLite/files, and CLI consumers. Handwritten expectations, source/recovery oracles, reuse with identical input closure, and validation budgets follow actual project failure obligations.

## Design decisions

[cargo-dist configuration](https://axodotdev.github.io/cargo-dist/book/reference/config.html) allows package `dist=false` to exclude distribution; a workspace packages list can override per-package selection. Local config/v0.rs and announce implementations in the pinned 0.32.0 version also support Cargo metadata placement. External `sheltie-export` is excluded through its Cargo metadata. Adding a binary requires checking the actual dist plan so it does not silently enter the configured release set. This does not authorize a release run.

Synchronous core/runtime separation, a single SQLite authority, one target format, idempotent recovery through normal interfaces, independent review, and data preservation follow the project's constitution and contracts. The four sources do not prescribe those specific mechanisms. Best practices are not a mechanism checklist: only mechanisms supporting current guarantees or actual needs enter the product.
