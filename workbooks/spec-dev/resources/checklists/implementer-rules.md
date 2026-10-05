# Implementer rules

Types/signatures/comments/tests are already designed; fill bodies. Every rule is mandatory.

1. Read only your task card, its allowlisted sources, and corresponding tests. Decision/spec/plan are for approval/condition checks only, not implementation context. No whole-repository reading.
2. Enable current tests first and run scaffold's command. Establish behavioral red, not compilation failure; make compilation succeed first.
3. Fill only placeholders and remove their task labels. No signatures/types/public items/dependencies/new files. Comments define behavior. Leave other tasks' placeholders untouched.
4. Never change tests/snapshots. Fix implementation, not assertions/goldens.
5. Turn one red test green at a time.
6. Run every gate once green; all must pass.
7. One task commit, allowlisted files only.
8. No incidental changes to other placeholders/tests/possible optimizations.
9. Stop for conflicting tests, impossible unchanged interfaces/tests, new dependencies, five failed attempts on one test, or card/plan contradictions. Restore worktree and report Blocked. Sole exception: defective focused command/scaffold script prevents compliant work. Fix tool in a separate Task: scaffold commit, explaining each defect without weaker checks or test/signature/plan changes.
10. Do not invent behavior absent from comments/tests/plan; use rule9.

## Commit message

```text
<type>(<scope>): <English summary, one line, at most 72 characters>

<English body: what changed, why, actual verification.>

Task: T05
Work: <work_id from brief>
Agent: <actual model name>
```

Type: feat/fix/refactor/test/docs/chore. Scope: module/directory. English summary/body. Preserve the trailer block and actual brief Work ID; follow stricter repository Change/Task trailer requirements when applicable.
