# AGENTS.md

Instructions for agents and developers working in this repository. Claude Code reaches this file through `CLAUDE.md`; Codex and Cursor read it directly. [简体中文](AGENTS.zh-CN.md)

## Project

Sheltie is a local workflow engine for coordinator agents. People write methods as Workbooks; the engine records state, generates briefs, computes legal next actions, and enforces gates without judging content. Rust: three engine crates and one `sheltie` binary. The unreleased external tool `sheltie-export` obtains results only through the CLI and creates a new copy under an explicitly authorized parent directory.

## Entry points

1. Read `CONTEXT.md` and use the project vocabulary.
2. Read `specs/README.md` for the current release, active change, and document authority.
3. When an active change exists, read the package linked by `specs/changes/README.md`; use its plan for tasks and validation.
4. Before changing code or tests, or committing, read `specs/engineering.md`.
5. For usage documentation or source explanations, use `docs/README.md` to choose tutorials, guides, reference, or explanations. Fields and behavior remain defined in `specs/`; implementation entry points are in `docs/reference/implementation.md`.

When no change is active, do not independently adopt a proposal from `proposed/`. People decide proposal adoption, task initiation, and external publication.

## Hard boundaries

- Product semantics: `specs/spec.md`. Invariants: `specs/constitution.md`. Fields and commands: `specs/contracts/`. Resolve conflicts in upstream documents before changing code.
- Implementation progress is defined only by the active package's `plan.md`. A proposal, review, commit, or passing test does not establish product completion.
- `sheltie-core` performs no I/O; `sheltie-runtime` makes no business judgments. The engine does not interpret natural language to reach conclusions (`INV-1`, `INV-2`).
- The engine writes only to `~/.sheltie`; it does not write host configuration or install anything into agents (`INV-3`).
- Package means a Cargo package. A business method is always a Workbook (`INV-4`).
- Record findings, design, plans, progress, validation, and review within their own change package. Root specifications and decision records are not execution logs.

## Language

English is the default for documentation, comments, Rust API documentation, product messages, developer instructions, and new commits. Preserve Chinese documentation and method variants through explicit language links. Keep meaningful Unicode fixtures and original evidence; follow `docs/how-to/maintain-docs.md`.

## Commands

```bash
cargo fmt --all -- --check
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run --all-features --no-tests=pass
cargo deny check
scripts/check-docs.sh
scripts/check-specs.sh
python3 scripts/check-language.py
scripts/task.sh <task>          # when required by the package plan
scripts/check-task.sh <task>    # when required by the package plan
```

## Commits

One task per commit; run the package plan's required gates first. Follow `specs/engineering.md` §4: `type(scope): English summary`, with a body explaining why and how it was verified. New changes use `Change:`, `Task:`, and `Agent:` trailers; MVP T01–T26 retain their legacy trailer format.

## Gaps

Product questions belong in `specs/spec.md`; mechanisms belong in architecture or contracts. Add `docs/explanation/decisions/D-*.md` for important cross-task choices. Change task order in the active plan. Keep unadopted questions in `changes/proposed/`, without presenting them as current behavior. Do not invent a second state model, compatibility layer, or temporary source of truth merely to make a task compile.
