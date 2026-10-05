# Implementation architecture constraints

English | [简体中文](architecture.zh-CN.md)

This document defines mandatory layering, state, reading, and file-effect boundaries. The [specification](spec.md) defines semantics; [contracts](contracts/README.md) define exact fields and ordering. See [architecture explanation](../docs/explanation/architecture.md), [source tour](../docs/explanation/source-tour.md), and [data model](../docs/reference/data-model.md) for rationale and locations.

## 1. Layers and write boundaries

The engine is a single sheltie binary with downward dependencies: cli → runtime → core.

| Layer | Responsibilities and mandatory boundaries |
| --- | --- |
| core | Parsing, compilation, decide, legal_next, and pure rendering; no filesystem, database, clock, or random I/O; observed facts arrive as parameters |
| runtime | Real file and OS-principal observation, managed handles, Store transactions, effects, and recovery; no report interpretation or recomputation of core business decisions |
| cli | Arguments, dispatch, text/JSON, and error codes; no direct database access or alternative business-state construction |

The engine writes only its management root, never host configuration or host resources. Unreleased sheltie-export obtains sources solely through public CLI, without runtime dependency or direct Store access, and publishes a new copy only under an explicitly authorized parent. The author tool materializes drafts in its own temporary root and validates the same byte set through trusted CLI. It does not modify author originals, the production Home, frozen Work copies, or engine formats.

## 2. Definitions, identity, and state authority

Parsed and compiled Workbook/Flow values are validated definitions. Work start owns the actual frozen method copy; later operations do not load graphs from the installed-method directory. SQLite alone stores execution state; cards, directories, and logs are projections.

IDs, paths, digests, and bounded text use validating newtypes. Public WorkState fields do not guarantee valid cross-field combinations. Production transitions come from decide; persisted loading strictly validates state, frozen graphs, gates, and path ownership. Missing critical fields must not be interpreted through defaults.

An Occurrence is a node arrival; Attempt number is sequential creation within that arrival. Only failed consumes the business-failure allowance; superseded does not. Revocation and replacement occur in one write operation, at most once per Occurrence. Non-statistical inputs inherit old frozen bindings and verify bytes through the same handle; statistical inputs derive from state including the new Attempt. Old drafts do not become new frozen inputs.

## 3. Trusted reads and source closure

Each state, stats, or result query uses one trusted read context. Revision, pending facts, and next must not be assembled across connections or snapshots. Read Work and associated requests/audit/effects in one read-only transaction; strictly decode complete payloads and validate closure before consuming business files. Partial identity projections do not replace complete contract decoding.

Final results come only from the particular terminal Attempt that made the Work succeed. Unapproved gates, unfinished effects, or non-success states cannot confer final qualification. Reference queries do not claim to reverify all source bytes. When original bytes are needed, bind revision/key from that same read, verify source identity, and read/validate size and digest through one confined handle.

## 4. Requests, transactions, and recovery

Argument parsing determines only @file source paths. A historical request-id hit first verifies the complete operation, target, and original intent; it does not reread submitted request sources or business outputs to determine the historical response. The same intent returns the original response; different intent is rejected. Historical next is not current operation authority.

Deterministic work start rejection occurs before creating the management root/lock, allocating a sequence, or creating business directories. Workbook add follows the storage contract for coarse pre-lock checks, complete locked private-copy validation, and failure cleanup. Allocate start sequences in an independent short transaction; later failure may leave a gap that is never reclaimed.

Legal writes hold the management-root write lock, recheck prerequisites inside it, recover previous effects, then observe, decide, and commit. One transaction records request deduplication, revision CAS, state, original response, audit, and effects. Publish files after commit according to registration. Validate the complete effect closure before any action. If completion cannot be proved, stop new writes, distinguish precommit rejection from committed recovery errors, and retain any verified legitimate original response.

Restore historical business files using committed bytes; regenerate cards from latest state. Recover/clean pending files by ownership, identity, and Store references, never age or takeover of unknown directories. Read-only operations do not repair effects or create engine locks; SQLite control-file exceptions follow D-039.

## 5. Managed files and external publication

Derive managed paths from the management root; verify confinement, real ancestors, and handle identity. Observation, size, digests, sealing, and recovery use the same object. Illegal names, links, hard links, replacements, changed content, and unprovable objects stop with their accurate errors. A path name does not establish identity.

Workbook freezing, state/effect loading, and publication validation share trusted-snapshot obligations. Separate engine files and worker outputs. Seal artifacts in place; downstream reads verify frozen bytes. Purge requires explicit confirmation and retains the original root and lock; partial failure and unexpected late files must not disappear silently.

External export first strictly validates one final selection, receives each original byte stream within limits, reads back and creates a manifest, performs required synchronization, then publishes a new directory with same-parent NOREPLACE. Only confirmation of the final object and parent sync establishes complete. Insufficient confirmation after rename is publication_unconfirmed; do not delete the final directory as compensation. The protocol defines file/process limits, overlaps, and errors. OS sync does not promise power-loss durability or same-permission isolation.

## 6. Verification obligations

Engineering rules, core-vocab, and actual layer tests verify invariants and vocabulary boundaries. Identity, bytes, concurrent reads, pre/postcommit recovery, and successor consumers are separate obligations; simplification must retain their independent oracles. See [implementation baseline](../docs/reference/implementation.md) and [acceptance boundaries](../docs/reference/acceptance.md).
