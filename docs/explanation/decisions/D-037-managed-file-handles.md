# D-037: Managed files use directory handles and safe APIs

English | [简体中文](D-037-managed-file-handles.zh-CN.md)

Status: `accepted`
Date: `2026-09-28`
Related change: [C002](../../history/changes/C002-v0.2.0-reliability/README.md)

## Context

M1 found lexical `..` escapes, sealing via path chmod, reopening digest files by path after enumeration, and self/recovery callers bypassing fsx. Operations could affect another object. Engineering prohibits unsafe and requires constructor-validated path types.

## Decision

runtime anchors paths in opened management-root/subdirectory handles. `ManagedRelPath` rejects empty, absolute, `.`, `..`, empty segments, and NUL. Open directory segments individually with NOFOLLOW. First fstatat the leaf to reject known special objects, then open NOFOLLOW/NONBLOCK/CLOEXEC and fstat identity, type, and nlink. T19 fixes module interfaces without extending core, introducing generic filesystem traits, or adding persistent identity fields.

Use direct rustix `=1.1.4` with `fs` enabled: safe `openat`, Dir traversal, directory-relative rename/unlink, fchmod, fsync. NOREPLACE mappings were checked in rustix source and tested by T18 on macOS arm64. Linux execution was waived by the user and remains not_run. Sheltie retains `#![forbid(unsafe_code)]` without direct libc calls.

Output observation, digesting, and normal submit sealing reuse one opened SafeFile. After COMMIT, before sealing, recheck bytes/nlink against ArtifactRef on that handle, then chmod/sync the same object and verify the original path still points to it. Recovery reopens recorded references and verifies bytes. Explicit external `@file` is a read-only entry point, without managed write/delete/chmod operations.

This does not prevent arbitrary whole-directory moves by the same OS account or make readonly bits user isolation. Cross-mount, FIFO, link, and sync failures are accurately reported, without path reopening fallbacks, ignored errors, or unsafe code.

## Consequences

Home/fsx and real callers jointly enforce managed paths. CLI parses paths only; Store writes use HomeLock. Dependencies/interfaces are unverified until T18 API probes and T19–T23 caller gates pass.

## Verification

Check rustix 1.1.4 docs/vendor source. T18 macOS API/FIFO/sync probes ran. User-waived Linux runs remain `not_run`, without claiming Linux verification. T19–T23 use temporary Homes, single-condition link/object replacements, and sentinels against real macOS callers.
