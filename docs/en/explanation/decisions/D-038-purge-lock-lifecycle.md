# D-038: Purge data while retaining the root lock

English | [简体中文](../../../zh-CN/explanation/decisions/D-038-purge-lock-lifecycle.md)

Status: `accepted`
Date: `2026-09-28`
Related change: [C002](../../history/changes/C002-v0.2.0-reliability/README.md)

## Context

M1 confirmed normal purge of a frozen 0555 Workbook failed with EACCES. The original implementation deleted `store.db` and `.lock` first, leaving other trees. Removing the lock before the root let waiters create new locks and write while purge still held the old inode lock.

## Decision

`self uninstall --purge --yes` removes user data, SQLite, workbooks, works, pending, tmp, and bin, retaining the empty root and original .lock. Within the same HomeLock, verify objects/permissions and delete data trees → pending/tmp/bin → database. SQLite is last; `.lock` is never deleted. Failure retains root/lock and accurately reports partial cleanup, without recreating Store/Works. Repeating purge may continue cleanup.

Waiters continue on the same root lock. Authorized workbook add/self install may initialize an empty Store. A waiting old Work write returns `NOT_FOUND` if Store/Work rows were removed, without creating a database or reviving old Works. The uninstall protocol explicitly retains root/.lock.

## Rejected alternatives

- Delete `.lock` before the root: a concurrency window permits lock recreation.
- Move the lock outside the root: violates INV-3 and adds a second managed location.
- Copy data and compensate by deletion: increases writes and cannot reliably recover partial deletion of the authoritative SQLite state.

## Consequences

Purge removes all user Work data and binaries; missing directories no longer define success. Root path and lock object remain stable.

## Verification

Use temporary Homes with frozen trees/Store to test normal cleanup, retries after partial failure, waiting install/add, and waiting old Work commands. Synchronization points confirm waiters actually reached lock waiting.
