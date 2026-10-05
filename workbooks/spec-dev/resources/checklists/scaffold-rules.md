# Scaffold rules

A newcomer with limited implementation ability must be able to fill correct bodies from signatures/comments/failing tests.

## Structure

1. Declare every new type/function now; implementers add no files/public items/dependencies. Same-line task labels identify each placeholder (todo!() // T03); one owner each, separate test ownership markers. Verifiers reject unlabeled placeholders.
2. One responsibility per function, small enough for one junior implementation. Split into private documented/testable placeholders.
3. Make invalid states unrepresentable: enums over annotated strings, newtypes over primitives, construction checks over deferred validation.
4. Use exhaustive matching without catch-all where supported; compiler finds omissions.
5. Two doc lines: behavior/failure; reference exact spec/plan sections rather than repeat.

## Tests

6. Each task has legal and one-condition rejection cases.
7. Handwritten/spec-derived/independently calculated expectations, never tested-code calculations.
8. Author goldens/snapshots now; implementers cannot update them.
9. Behavior-named tests with separate ownership; all disabled with task reasons.
10. Complete runnable helpers/fixtures/fake clocks/IDs yourself.
11. Complete task allowlists; tests/config/gates are not implementation allowlisted files.

## Order

12. Skeleton gates must pass: compiling placeholders, disabled tests, clean lint. Permit necessary placeholder lint (e.g. do not enable clippy::todo) rather than remove placeholders.
13. Lowest types, dependent functions, tests; every referenced symbol exists.
14. Before committing positively trial one smallest task: temporarily fill it, run focused tests/all gates, then revert trial bodies. Negative-only checks miss tool defects.
15. Implementers independently commit tool fixes under rules; review every tool change.

## Commit

Same implementer format: chore(scaffold): <English summary>; body names types/functions/test count/focused command; Task: scaffold, Work: <actual-id>, Agent: <actual-model>. Follow stricter repository trailers when applicable.
