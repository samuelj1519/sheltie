Create and commit a **compilable skeleton with placeholder bodies and complete disabled tests**. Subsequent tasks only enable tests/fill bodies. You make the final design decisions; implementers make none.

## Read

1. Approval_rules for corrections/inherited corrections; ordinary approvals follow local checks.
2. Plan Approach/Gates for files/commands.
3. Tasks' files/tests/observable outcomes: cover every outcome.
4. Spec acceptance: each criterion has a test, possibly final end-to-end.
5. Decision: before work compare Approved specification/plan sha256 against brief-bound files using shasum -a 256 (sha256sum on Linux). Mismatch means invalidated approval: report Blocked with both hashes, do not scaffold unapproved plans. Human escalate may correct approval; follow/record correction. Apply every accepted condition, such as extra tests.
6. Escalation only when From is escalate; follow feedback/authorization rather than repeat failed approaches. Otherwise stale, ignore.
7. Rules: all scaffold requirements.
8. Template: scaffold.md shape.
9. Project: enter root/read referenced relevant files/types/conventions, without repository-wide reading.

## Execute

1. Git status must be clean.
2. Add/modify approach-listed types/signatures/doc comments. Use language placeholders (Rust todo!(), Python raise NotImplementedError, TypeScript throw new Error("todo")) with one same-line task owner, e.g. todo!() // T03. Each body belongs to one task so verifier distinguishes current from later placeholders. Preserve existing bodies; add signatures only.
3. Public docs state behavior/failure in two lines, sufficient for implementation without rereading plan. Each function does one thing; subdivide large functions into private documented placeholders.
4. Write task tests with behavior names and separate ownership comments/markers. Disable all with task reasons (Rust ignore, pytest skip, Jest test.skip). Handwrite expectations; complete helper/fixture bodies without placeholders.
5. Provide a task-filtered command by ownership or explicit card test names that fails on zero tests, e.g. cargo nextest run --no-tests=fail -E 'test(/(^|::)(accepts_valid_input|rejects_missing_input)$/)' or pytest -k 'accepts_valid_input or rejects_missing_input' (zero exit5). Prefer project task scripts. No inferred tNN ownership. Record in scaffold.md.
6. Run every gate. Placeholders/disabled tests must not fail them. Existing tests/gate config change only if approach explicitly authorizes it.
7. One commit using Rules' commit format.
8. On replanned return, incrementally adapt prior scaffold without destroying it or changing already-passing tests.

## Blockage

Approach conflicts requiring existing-body changes, untestable outcomes, or unplanned dependencies require a clear Blocked report without partial commits.

## Approval correction

Read bound approval_rules before correcting/inheriting; follow qualification, references, conditions, and stop rules.

## Output/reply

Use template at scaffold path, first line Complete or Blocked. Reply with commit hash, task/test counts, and focused test command in three sentences.
