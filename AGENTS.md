# Repository instructions

Sheltie is a local workflow engine for coordinator agents. People define methods as Workbooks; the engine records state, generates briefs, computes legal next actions, and enforces gates. The coordinator interprets content and delegates work. Read `CONTEXT.md` for the domain vocabulary.

## Start here

1. Check Git status and preserve unrelated worktree changes.
2. Read `specs/README.md` for document authority, release scope, and the active change.
3. For an active change, follow the linked package plan for task scope, dependencies, progress, and gates. Human instructions authorize maintenance work; the absence of an active change does not authorize adopting a proposal.
4. Before editing source, tests, specifications, or committing, read `specs/engineering.md`. Choose checks by affected consumers, including fixtures, instructions, and generated inputs.
5. For usage documentation, start at `docs/README.md`. For source navigation, read `docs/en/reference/implementation.md`. For documentation cleanup or language changes, read `docs/en/how-to/maintain-docs.md`.

## Authority and boundaries

- Resolve product behavior in `specs/spec.md`, invariants in `specs/constitution.md`, and exact fields, commands, and formats in `specs/contracts/`. Repair conflicting upstream documents before implementation; define each fact once.
- The active plan alone records implementation progress. Keep implementation, validation, human acceptance, release, and demonstrated benefit as separate conclusions.
- Dependencies flow `cli → runtime → core`. Core performs no I/O; runtime makes no business judgments. The engine does not interpret natural language, judge content, or invoke models (`INV-1`, `INV-2`).
- The engine writes only under its management root and does not modify host configuration or install agent resources (`INV-3`). External tools have their own explicit filesystem boundaries; the exporter reads results through the CLI and creates a new copy under an authorized parent.
- A Package is a Cargo package; a business method is a Workbook (`INV-4`).
- Keep task findings, plans, progress, validation, and reviews in their change package. Consolidate completed work through the documentation maintenance procedure, preserving fixed Git snapshots and original evidence.

## Language and delivery

Use English for repository instructions, specifications, comments, Rust API documentation, product messages, and new commit messages. Maintain Chinese reader versions only in `docs/` and the root README. Preserve explicit Chinese Workbook and skill instructions, meaningful Unicode fixtures, user data, and original evidence.

Run the engineering checks required by the affected consumers and active plan. Documentation-only wording does not require unrelated Rust reruns. At minimum, verify changed document links and specification governance; run the language gate when language routes or source text change. Report actual checks and retain FAIL, `not_run`, unknown, and excluded-platform boundaries.

Commit one active-plan task at a time using `type(scope): English summary`; follow engineering §4 for bodies and trailers. A commit or a passing test does not authorize publication. Perform repository or external actions within the user's explicit scope.

For missing authority, update the product specification, architecture, or contract; use an ADR for important cross-task decisions and the active plan for task ordering. Keep unadopted directions in `specs/changes/proposed/` until human adoption.
