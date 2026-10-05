# D-045: Separate specification authority from Diátaxis documentation

English | [简体中文](D-045-specs-and-diataxis.zh-CN.md)

Status: `accepted`
Date: 2026-10-05
Related: user request to restructure documentation around completed capabilities; refines [D-032](D-032-use-change-packages.md) document responsibilities

## Context

After process-artifact cleanup, specs still combined behavioral constraints, operating instructions, reference indexes, source explanations, and external reading. Maintainers need authority and adopted scope; users and source readers need goal-oriented material. A shared entry point blurred these needs.

## Decision

specs retains constitution, product specification, architectural constraints, exact contracts, engineering rules, roadmap, and unfinished changes. Implementation, acceptance, and releases belong in docs/reference; ADRs in docs/explanation/decisions; closed changes in docs/history/changes. Governance tools continue checking original qualification without requiring historical material to remain in specs. docs follows [Diátaxis](https://diataxis.fr/) tutorials/how-to/reference/explanation for lasting technical documents; mixed content is split by purpose. Historical archives sit outside those four current-reader categories. Topic explanations do not require reconstructing design iteration by iteration.

Current code verifies implementation facts; independent acceptance/releases limit verified/released claims. Broad constitution language on host permissions/readiness is clarified using existing GF-20/GF-22 boundaries, without permission evaluation, isolation, or probes. Code does not automatically supersede specifications: record differences and repair implementation or explicitly adopt upstream change. Future directions are not current capabilities.

Only specs/contracts defines fields, commands, errors, limits, and persistent formats. docs/reference supplies implemented relationships and entry points linked to exact contracts, without manually duplicating complete definitions. Architectural constraints remain in specs; causal explanation/source navigation belongs in docs. ADRs retain formal reasons for important tradeoffs.

## Alternatives and consequences

- Moving files alone leaves mixed responsibilities such as quickstart/real-task manuals and architecture/type signatures/explanation, failing distinct reading needs.
- Copying contracts into docs creates drifting precise sources; retain one normative source and reader entry points.
- Describing implementation as accepted/released loses environment, cost, and fair-comparison boundaries; maintain implementation baselines and acceptance records separately.

Root README routes learning to docs. AGENTS routes behavior changes to specs, then task-specific docs. Migration repairs references, generated inputs, and checking consumers together. Paths inside historical snapshots remain.

## Verification

New readers should find tutorials, goal-specific operations, product reference, and explanations. Maintainers should find current capabilities, exact rules, unadopted directions, and acceptance boundaries. Mechanical checks validate links, versions, ADRs, lifecycle, and contract consumers; real CLI runs validate new operational paths. Structural/command success does not replace first-reader experience.
