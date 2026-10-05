# Change entry points

English | [简体中文](README.zh-CN.md)

Active change: none

[C012](completed/C012-english-default/README.md) completed the authorized English-default migration, retained Chinese documentation and method variants, and translated local historical messages. There are no proposed or rejected changes. Execute a plan only after human adoption; only one change may be active at a time.

```text
proposed ──human adoption──▶ active ──implementation, validation, independent review──▶ completed
    └──human rejection──▶ rejected
```

See [templates](templates/README.md) and the [implementation guide](../../docs/how-to/implement-change.md). Only the active plan defines current progress; completed does not mean released or that every experiment passed.

Preserve completion qualification in the complete package first. After freezing a Git snapshot, move lasting references into [historical changes](../../docs/history/changes/README.md) and clean up the implementation directory according to the [maintenance guide](../../docs/how-to/maintain-docs.md). Historical snapshots remain subject to completion-gate verification. Specifications do not duplicate closed-task summaries.
