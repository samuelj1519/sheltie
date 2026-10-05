# Task decomposition

Every task satisfies all conditions; otherwise split/merge.

1. One observable result via command/file, not merely refactor/cleanup.
2. All gates pass at completion; never defer disabled failing tests.
3. Independently committable. Interface changes update all callers in one task without uncompilable intermediates.
4. Skeleton owns tests; cards list behavior names. Pure documentation specifies its checks.
5. Only allowlisted files. Missing needed files mean bad decomposition; stop rather than silently expand.
6. Dependency order; later tasks use previously delivered interfaces.
7. One to six bodies, three to fifteen tests, achievable by a junior in an hour; split larger tasks.
8. First task fills lowest types/helpers.
9. Last task completes docs/changelog/cleanup.
10. Three to twenty tasks; beyond twenty return to requirements for splitting.
