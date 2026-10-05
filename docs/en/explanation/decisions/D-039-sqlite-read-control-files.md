# D-039: Control-file boundaries for read-only SQLite

English | [简体中文](../../../zh-CN/explanation/decisions/D-039-sqlite-read-control-files.md)

Status: `accepted`
Date: `2026-09-28`
Related change: [C002](../../history/changes/C002-v0.2.0-reliability/README.md)

## Context

M1 found that WAL read-only connections may create a wal-index in an existing writable directory when `-shm` is missing. Error side effects or `immutable=1` cannot evade this; neither silent checkpoint/migration nor changes to old data before schema 1 rejection are permitted.

## Decision

Read-only SQLite may create/maintain `store.db-shm` in an existing root whose Home identity is verified. For an existing WAL-mode Store with missing WAL, it may create a zero-byte `store.db-wal` control carrier, without header/frame writes or changing existing WAL bytes. It must not create Home, `.lock`, Store, WAL data records, or business files. Before schema 1 rejection, main/existing WAL bytes remain identical, without checkpointing. Do not reject solely by main-file length: valid WAL may contain the full committed logical database despite a short main file. If a corrupt WAL header accompanies a valid main database, bundled SQLite may ignore WAL and return the old main snapshot. Accept its queryable view without guessing an ignored state or parsing WAL formats; main/WAL bytes must still stay unchanged. Query errors reject accurately, without repairing/deleting old files.

Before the first identification query, call safe rusqlite Connection::set_db_config(DbConfig::SQLITE_DBCONFIG_NO_CKPT_ON_CLOSE, true). Reject leaf/sidecar links and special objects before opening SQLite. No automatic RW fallback. `SQLITE_OPEN_NOFOLLOW` is not equivalent to rustix directory-fd anchoring. Do not use `immutable=1`, automatic RW fallback, or a custom SQLite VFS.

## Consequences

Reads do not change Work/Workbook/audit/request/main/WAL business state, acquire HomeLock, or create Home/.lock. Record permitted `-shm` changes and creation of missing zero-byte WAL separately, without a second state source. Schema/formats remain unchanged.

## Verification

Using the bundled SQLite version pinned by Cargo.lock, test existing WAL, missing shm, active writers, schema 1 main/WAL bytes before/after, sidecar links, and bytes after connection close. macOS probes ran. Linux was explicitly waived and remains `not_run`, without Linux/cross-platform PASS. API docs do not replace actual platform tests. Added checks verify a WAL created by legal read-only preflight is zero bytes and main bytes are unchanged. Post-Store-deletion purge rescans accept only legal single-link SHM or zero-byte WAL, retain nonempty late WAL/other anomalous objects, and report partial cleanup. Original runs/reviews are in fixed snapshots from [historical lookup](../../how-to/maintain-docs.md#read-original-historical-records).
