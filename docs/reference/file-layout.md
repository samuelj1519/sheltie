# Management root and file layout

English | [简体中文](file-layout.zh-CN.md)

Select roots by `--home` → `SHELTIE_HOME` → ~/.sheltie. Relative arguments resolve against invocation cwd. Managed ancestors/directories/files must satisfy identity/path constraints. See the [storage contract](../../specs/contracts/storage.md) for exact permissions, ownership, and recovery.

## Directory tree

```text
<management-root>/
  .lock
  store.db
  store.db-wal
  store.db-shm
  bin/
    sheltie
    sheltie.prev
  workbooks/<id>/<version>/
  works/<work_id>/
    workbook/
    start-inputs/<key>
    status-card.md
    attempts/<node>/
      occurrence-<NNN>/attempt-<NNN>/
        brief.md
        engine/stats.json
        outputs/<declared-path>
  pending/<internal-id>/payload/
  tmp/
```

This illustrates roles, without guaranteeing every entry exists. `NNN` has at least three zero-padded digits; Attempt IDs remain node#occurrence.number. Read actual paths from responses/resume/ArtifactRef, without guessing from labels.

## State, projections, and artifacts

| Object | Purpose/maintenance boundary |
| --- | --- |
| `store.db` | Sole state authority for Works/requests/audit/effects; never manually advance flow |
| `store.db-wal`, `store.db-shm` | SQLite log/shared-memory controls; do not independently delete for lock/corruption problems |
| `.lock` | Root lock for legal writes; queries neither create nor acquire it; purge retains it |
| `workbooks/<id>/<version>/` | Read-only installed methods, without same-version overwrite |
| Work `workbook/` | Run's own frozen method; later operations do not switch to installed copies |
| `start-inputs/` | Materialized frozen text/file contents |
| `brief.md` | Attempt instructions, bound inputs, output requirements |
| `engine/stats.json` | Engine-generated frozen stats, bound only when declared |
| `outputs/` | Executor writes declared drafts while running; submit seals by digest |
| `status-card.md` | Latest-write projection; queries do not refresh; existence grants no qualification |
| `pending/` and .owner/.deleted sidecars | Publication/deletion originals and ownership proofs; use database registration/object identity |
| `tmp/` | Owned download/database staging, distinct from protected pending originals |
| `bin/sheltie.prev` | One-level rollback binary, without Store backup |

Reads do not create Home, refresh cards, or recover effects. [D-039](../explanation/decisions/D-039-sqlite-read-control-files.md) permits specific SQLite control maintenance; business reads do not mean zero control-file changes.

## Write and copy boundaries

Worker agents write reports only to declared brief outputs. Business-repository changes require task authorization. Do not edit frozen inputs, installed methods, Work method copies, sealed outputs, or Store metadata.

`self uninstall` defaults to binary removal. Confirmed purge makes data unqueryable and is not troubleshooting. For anomalous pending/unknown directories, inspect responses, original commands, and ownership first; never claim/delete/overwrite by name. See [resume](../how-to/resume-work.md) and [troubleshooting](../how-to/troubleshoot.md).

External tools have different boundaries: the authoring tool creates owned temporary roots/downloads ZIP; exporter reads via CLI and creates a new copy under an authorized parent. Copies neither become a second state authority nor write back into Works.
