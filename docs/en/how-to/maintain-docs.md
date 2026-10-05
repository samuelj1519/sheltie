# Maintain documentation and read history

English | [简体中文](../../zh-CN/how-to/maintain-docs.md)

Users learn/operate/look up/understand through docs. Maintainers find behavioral authority, rules, acceptance, and adopted scope in specs. Verify current source capabilities and exact contracts before choosing affected reader material.

## 1. Place content by purpose

| Content | Location | Boundary |
| --- | --- | --- |
| Invariants, adopted behavior, architectural constraints | specs/constitution, spec, architecture | One specification; defects do not become promises |
| Fields, errors, formats, persistence | specs/contracts | Define exactly once; docs/en/reference summarizes/links |
| Implementation and bounded acceptance | docs/en/reference/implementation, acceptance | Separate implementation, verification, release, user benefit |
| Future directions/unfinished changes | specs/roadmap, changes | Needs/triggers/adoption/current progress |
| Design reasons | docs/en/explanation/decisions | Topic index, full/partial successors, original adoption reasons |
| Closed changes/releases | docs/en/history/changes; docs/en/reference/releases | Historical scope/limits/fixed snapshots, outside current explanations |
| Fixed learning paths | docs/en/tutorials | Start/expected actions/learning outcomes, without all real-task branches |
| One actual goal | docs/en/how-to | Prerequisites/actions/stops/recovery; link explanations |
| Commands/state/files/limits | docs/en/reference | Product-structured facts, without full contract duplication |
| Concepts/source/tradeoffs | docs/en/explanation | Relationships/reasons, ADRs, dated external sources |

Classify by reader need, not title/filename. Split mixed content by purpose; four directories need not have equal counts. See [Diátaxis](https://diataxis.fr/) and [D-045](../explanation/decisions/D-045-specs-and-diataxis.md).

## 2. Align with current implementation

1. Use [implementation](../reference/implementation.md) to locate public entries/contracts/regressions and verify fields/errors. completed, function names, or existing tests alone do not prove complete coverage.
2. Preserve support/version/trust/unknown boundaries. Mark implemented-but-unreleased capabilities as development; leave unadopted directions in roadmap.
3. Update specifications before affected tutorials/guides/reference/explanations when behavior changes. Record discrepancies and repair implementation or adopt upstream changes explicitly, without silently removing requirements.
4. Validate new command paths in real new roots. Success does not replace first-reader experience, independent quality, or human acceptance.
5. Check every inbound link, README/AGENTS route, skill-generation input, tool, and snapshot. Historical fixed paths do not become current directories.

Use [CONTEXT](../../../CONTEXT.md) terminology. Preserve commands/fields/versions/rejections/original outcomes. specs/docs must not each independently define the same complete field contract.

## 3. Consolidate after completion

1. Complete adopted obligations/gates/independent review within the complete package, retaining frozen candidates/input closure/raw outcomes/unknowns.
2. Commit the complete package and verify snapshots contain tasks/validation/reviews/originals. Do not cite SHAs missing material or substitute summaries for originals.
3. Merge valid behavior into upstream specifications/contracts, reasons into ADRs, reusable operations/evaluation into appropriate docs categories. Explain current design by topic, index ADRs with applicability, and state partial successors while retaining original reasons. Keep historical release scope/limits in release records.
4. Place English README summaries in docs/en/history/changes and their Chinese counterparts in docs/zh-CN/history/changes with Record form: reference, full Historical snapshot SHA, and baseline; update indexes. Remove closed-package copies/plans/handoffs/evidence from specs. Summaries cannot replace original qualification.
5. Extract only needed task allowlists/completion/test mappings into [scripts/task-history](../../../scripts/task-history/README.md). Do not move full plans/evidence or use mappings as new plans.
6. Repair inbound links/reading maps/checks. check-specs still verifies completed tasks/final validation/independent conclusions from fixed snapshots; summaries cannot bypass original gates.

These are maintenance procedures, without adopting proposals, approving gates, or authorizing publication. Satisfy active-plan original-preservation/review requirements first.

## Read original historical records

The complete committed pre-cleanup documentation snapshot is `9d98bf8f15944b7bda4fbd4096e7771b09eab728`, containing full C001–C011 completed packages and MVP history. Earlier removed C002 evidence is at `9ca6714e17c12dbe3f0e81056a03a781dbbfc8fd` under specs/changes/active/C002-v0.2.0-reliability/. That is an archive commit, not a release tag.

The complete C012 language-migration package and its original evidence are at `eec267b7b54b1e18e75578f7b5e269e7ddf73df8` under `specs/changes/completed/C012-english-default/`. Its [historical reference](../history/changes/C012-english-default/README.md) and bilingual inventory retain the long-term reading path.

Read one historical file from the root:

```bash
git show 9d98bf8f15944b7bda4fbd4096e7771b09eab728:specs/changes/completed/C011-workbook-visual-editor/design.md
git show 9d98bf8f15944b7bda4fbd4096e7771b09eab728:specs/releases/v0.1.0/plan.md
git show eec267b7b54b1e18e75578f7b5e269e7ddf73df8:specs/changes/completed/C012-english-default/validation.md
```

Extract complete directories into new temporary directories:

```bash
history_dir=$(mktemp -d /private/tmp/sheltie-history.XXXXXX)
git archive 9d98bf8f15944b7bda4fbd4096e7771b09eab728 specs/changes/completed/C011-workbook-visual-editor | tar -x -C "$history_dir"
```

Shallow clones may lack history. Check `git cat-file -e <SHA>^{commit}`; fetch history containing missing commits, without defaulting unavailable originals to PASS. Governance CI uses fetch-depth: 0. Documentation consolidation does not rewrite history or reduce .git size. Explicitly authorized message-only history migration is separate: use its verified old/new mapping for references while preserving original record bytes.

Read release specifications from tags, e.g. git show v0.2.0:specs/contracts/storage.md. Historical task gates require matching historical source/plans/tools/input closure. Current mappings retain ownership/scope only, without reconstructing old execution environments.

## English defaults and Chinese variants

English is the default authority for documentation, developer instructions, comments, Rust API documentation, product text, and new commit summaries/bodies. Maintain reader documents at matching `docs/en/<path>.md` and `docs/zh-CN/<path>.md` paths, with reciprocal language navigation. The root README retains its `.zh-CN.md` counterpart. English is the default entry language; docs/README.md routes readers to either tree. Specifications, repository instructions, changelogs, and tool guides elsewhere have one English version. Chinese Workbook and skill instructions remain explicit agent inputs; the Chinese spec-dev README remains because its retrospective reads the revision history. Preserve requirements, limits, exclusions, and evidence statuses in both maintained reader variants. Update current semantic changes in both; never silently translate original historical logs.

Chinese links select available Chinese counterparts. Commands, code paths, protocol fields, IDs, bytes, and original results retain their exact meaning. Default methods use English instructions; explicit `*-zh-CN` directories use separate localized Workbook IDs. Install/check/run each variant through the real CLI, then compare graph semantics, gates, results, and recovery behavior. Engine scenarios prove operational behavior; separate actual agent trials with preregistered quality/cost criteria evaluate instruction effectiveness. Do not claim model efficacy from structural tests alone.

Retain meaningful Unicode fixtures for text, path, argument, and normalization coverage. Inventory user text/original evidence before removal; remove only redundant material after verifying purpose and recoverability. Future commits use English under engineering §4. Historical message migration needs its own verified mapping and metadata-preservation checks.

Wiki source layout, generation, staging, and publication are described in [Manage the Wiki](manage-wiki.md). Keep authored documents in this repository and use the separate Wiki checkout only for generated presentation.

## Checks after changes

Run from the repository root:

```bash
scripts/check-docs.sh
scripts/check-specs.sh
scripts/check-tests.sh
git diff --check
```

Changes to governance scripts/task mappings additionally verify real consumers/invalid-input rejection. Wording alone needs no unrelated Rust reruns. Manually verify real commands/source entries; mechanical links do not prove operations correct.
