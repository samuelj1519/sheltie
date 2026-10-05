# D-035: Management-root write locking with `fs4`

English | [简体中文](../../../zh-CN/explanation/decisions/D-035-root-write-lock-fs4.md)

Status: `accepted`
Date: 2026-09-27
Related change: [C002](../../history/changes/C002-v0.2.0-reliability/README.md)

## Context

`work start` / `workbook add` publish by committing, then renaming pending into its final directory. Interleaved local writers could let cleanup delete another writer's committed but unpublished source. External root replacement could leave a waiter writing through a stale inode. Root-level file-lifecycle serialization is needed, without creating files for reads (GF-30). C002 purge retains root and original lock; see D-038.

## Decision

Use an exclusive file lock on `<management-root>/.lock`, through `fs4` (MIT OR Apache-2.0) fs_std::FileExt. `lock_exclusive` waits blocking; OS process exit releases it, without polling/timeouts. Under the lock, recheck schema, detect replay, recover effects, prepare, transact, publish, and mark completion. `self uninstall --purge` removes data and binaries while retaining the empty root and same `.lock`, without releasing/deleting it and creating another lock.

Read-only operations do not acquire HomeLock. They perform read-only SQLite identification and managed-object checks under D-039. Writers recheck root and `.lock` identity after acquisition. Normal purge retains both; queued writers continue with the same lock. Authorized add/install may initialize under it. Old Work commands return NOT_FOUND after purge removes Store/Work rows, without recreating them. If external interference removes/replaces root or lock, waiters must release and retry against the managed root instead of writing through stale identity. API checks confirmed std `File` locking, `try_lock_exclusive` probing, and cross-platform support (unix fcntl/flock, Windows LockFileEx) for these operations. D-038 defines purge scope and normal same-lock waiting.

The lock serializes cooperating local processes only, without constraining manual changes by the same user. SQLite revision CAS remains a transaction check, without automatic business retry.

## Rejected alternatives

- SQLite `BEGIN IMMEDIATE` alone: covers transactions, not later rename/delete windows or paths outside the database such as `bin/` and pending/.
- `fd-lock`: borrowed-handle guards complicate long-lived lock files and inode rechecks; maintenance activity is comparable.
- Custom existence-marker locks: crashes leave deadlocks requiring manual recovery; OS does not release them.

## Consequences

Local writes are fully serialized; engine reads create no .lock. WAL reads may maintain SQLite shared-memory controls under D-039. Successful purge retains the empty root and original lock. `fs4` adds one direct dependency with few transitive dependencies.

## Verification

Start concurrent processes as a group, force interleaving with synchronization points, then join. Purge-waiter tests assert the same root/.lock is retained/reused and old Works do not revive. Separate external-replacement tests expose stale identities and prevent writing to a new root incorrectly.
