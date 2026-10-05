# Review checklist

Report correctness/security/maintainability impact only; mark each Blocking or Suggestion.

## Correctness

1. Every criterion has a test or executable evidence.
2. New branches/errors/boundaries covered.
3. Behavior assertions, not internal counters/is_ok alone.
4. No commented/ignored tests.

## Security

5. Validate external arguments/files/network inputs before internal logic.
6. No hardcoded secrets/passwords/personal paths.
7. Controlled command/file paths; inputs cannot escape boundaries.

## Maintainability

8. Reuse existing logic rather than duplicate.
9. Public interfaces have actual callers.
10. Errors explain next actions.
11. No speculative empty implementations/switches/compatibility layers.
12. Comments explain why, not obvious operations.

## Delivery

13. Each commit independently compiles with Task trailer.
14. Behavior changes update documentation.
15. No expansion beyond specification Scope. Extra ideas are suggestions for humans.
