# D-032: Manage iterations with change packages

English | [简体中文](../../../zh-CN/explanation/decisions/D-032-use-change-packages.md)

Status: `accepted`
Date: 2026-09-27
Related change: C001

Partially supersedes D-22's current-stage content in `AGENTS.md` and Task-only trailers for later versions. D-22's MVP history remains unchanged.

## Context

After MVP completion, `plan.md` and `decisions.md` combined current entry points, task history, review logs, design reasons, and real runs. `AGENTS.md` also retained an obsolete next task. Further appends increased default context, duplicate facts, and misread state.

## Decision

Root `specs/` retains stable authority. Each iteration uses an independent change package with `proposed / active / completed / rejected` lifecycle states. Completed reference records and release history belong in docs under [D-045](D-045-specs-and-diataxis.md). `AGENTS.md` contains only durable rules and entry-point routing.

See [C001 design](../../history/changes/C001-specs-governance/README.md) for responsibilities, lifecycle, templates, and migration rules.

## Rejected alternatives

- **Keep extending root plan.md.** Closed MVP work and later tasks would share a status table.
- **Copy all specifications and contracts per version.** Multiple current rules drift; tags reconstruct release snapshots.
- **Use only issues or chat.** New sessions and local agents cannot reliably discover external context.
- **Put current state in AGENTS.md.** This always-loaded entry point quickly becomes stale as tasks progress.

## Consequences

- Unadopted proposals do not appear authorized for implementation.
- Current progress exists only in the active package plan.
- Decisions and releases retain durable explanations; fixed Git snapshots preserve completion qualification and original runs.
- On completion, extract reference summaries following [documentation maintenance](../../how-to/maintain-docs.md), instead of keeping execution logs in the default reading set forever.
- Change indexes and structure checks require maintenance.

## Verification

Reading only `AGENTS.md`, `specs/README.md`, and the change index, a new session should identify the current release, active and proposed changes, and next task. `scripts/check-specs.sh` checks directory states, indexes, ADRs, and release closure.
