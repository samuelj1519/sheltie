# Architecture and recovery

English | [简体中文](../../zh-CN/explanation/architecture.md)

Sheltie assigns repeatable, mechanically verifiable work to the engine: state, explicit edges, frozen sources, gates, and limits. Workbooks describe business methods; coordinators judge content and choose routes. This boundary lets one binary run writing, code-change, and other methods.

## Why three layers

CLI turns user arguments into requests and presents accurate results. runtime handles files, process identity, the database, and real observations. core receives observed facts and uses pure functions to decide state, legal next actions, and required effects. Dependencies flow cli → runtime → core.

Pure core rules can be tested with handwritten states and independent expectations, without temporary databases or mock I/O. Real runtime tests check transactions, file identity, recovery, and errors. CLI tests check public consumers instead of proving only internal helpers. Follow the complete chain in the [source tour](source-tour.md).

## Why database commits and file publication are separate

SQLite can atomically record state, request responses, and effect obligations. It cannot include filesystem rename or permission changes in that transaction. The engine first persists what was committed and what still needs publication, then executes the recorded file effects.

An error can occur before commit or after state has committed. The latter cannot simply report that nothing happened: a coordinator might repeat business work already performed. Recovery errors retain accurate commit identity and original responses. Later writes prove that old effects completed before making a new decision.

Replay returns the response frozen when the original request committed; current queries return a new read snapshot. Historical `next` can already be stale, which is why resuming starts with a status query. Read-only queries do not publish or recover effects.

## Why objects matter as well as paths

A path can be replaced between system calls; the contents of the same inode can also change. Path strings, file objects, and actual bytes answer different questions. Managed directory handles and reads through the same file handle bind observation, digest, and action to the same object as far as possible. Stop when ownership or integrity cannot be proved.

A Work's method copy and input bindings are frozen by bytes so that method upgrades or changes to a same-named file cannot silently affect execution. An editable export is a separate delivery object. It may change without becoming a source of Work state or results.

## Why queries use one snapshot

If state and next come from different revisions, a consumer can combine new state with old qualification. Store `read_work_bundle` and trusted loading read state, related requests, and effects within one context. status, stats, and result project from it, with references bound to a specific Attempt.

A final-result query only lists explicit selections. Consuming actual bytes additionally requires matching revision, key, and confined handles. See [data model](../reference/data-model.md) and [limits](../reference/limitations.md).

## Why external tools are independent

The exporter needs only public results and raw bytes, without Store access. The authoring tool needs complete drafts, actual structural decisions, and new ZIP files, without advancing a Work. Small interfaces confine permissions and recovery obligations to actual needs, preventing browser drafts or external copies from becoming a second source of truth.

Required implementation rules are in the [architecture](../../../specs/architecture.md); formal design reasons are in [ADRs](decisions/README.md).
