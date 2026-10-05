# Design and maintenance reading

English | [简体中文](../../zh-CN/explanation/design-sources.md)

These links come from existing project source notes. Sources were checked on 2026-09-27 / 2026-10-03 and may have changed. External advice is neither an adoption decision nor evidence that Sheltie performed a validation.

| Topic | Primary sources | Continue in this project |
| --- | --- | --- |
| Simple workflows, agent autonomy, and tool boundaries | [Building effective agents](https://www.anthropic.com/engineering/building-effective-agents), [Writing effective tools for agents](https://www.anthropic.com/engineering/writing-tools-for-agents) | Constitution, architecture, source tour |
| Completion, continuation, and independent evaluation for long tasks | [Effective harnesses for long-running agents](https://www.anthropic.com/engineering/effective-harnesses-for-long-running-agents), [Harness design for long-running apps](https://www.anthropic.com/engineering/harness-design-long-running-apps) | Implementation guide, workflow evaluation |
| Tutorial, how-to, reference, and explanation responsibilities | [Diátaxis](https://diataxis.fr/) | Documentation map, maintenance guide |
| Decision context, alternatives, and consequences | [Documenting Architecture Decisions](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions), [ADR templates](https://adr.github.io/adr-templates/) | decisions |
| Release compatibility and user-facing changes | [Semantic Versioning](https://semver.org/spec/v2.0.0.html), [Keep a Changelog](https://keepachangelog.com/en/2.0.0/) | Release records, CHANGELOG |
| Rust interfaces, modules, and validation | [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/), [Cargo Book](https://doc.rust-lang.org/cargo/) | Engineering, architecture |
| SQLite WAL control files and transactions | [SQLite WAL](https://sqlite.org/wal.html) | Storage contract, D-039 |

These materials offer design options. Adoption depends on the project's needs, contracts, real consumers, quality, and cost evidence. Limited experiments cannot prove general benefit or support for every platform. Read historical proposal derivations and source-coverage matrices through [fixed historical snapshots](../how-to/maintain-docs.md#read-original-historical-records).
