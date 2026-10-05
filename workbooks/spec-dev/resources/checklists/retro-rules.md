# Reflection rules

## Six categories (closed set)

| Category | Typical evidence | Location |
| --- | --- | --- |
| Instruction ambiguity | Repeated returns cite one sentence; change notes ask its meaning | instructions/<node>.md |
| Checklist gaps | Review findings absent from all checklists | resources/checklists/*.md |
| Task decomposition | At least two fix rounds for one task, or unusually short duration | task-rules.md/plan instructions |
| Tier mismatch | Standard repeatedly blocked/returned; strong fast first-pass success | Flow tier |
| Tool defects | Change notes identify command/script defects; Task: scaffold repair commits | scaffold-rules.md |
| Gate placement | Human says unnecessary intervention or missing approval | Flow gate/executor |

## Mandatory

1. Every proposal has openable evidence: <node>#<occurrence>.<number>/<file> and line, or stats number.
2. Every proposal identifies a Workbook file/paragraph. Project-code issues go in No change recommended rather than method reflection.

## Boundaries

- Code quality belongs to review.
- Write proposals only; humans adopt improvements in new versions.
- Facts, not feelings: fix zero visits and implement six first-pass tasks, not “went smoothly.”
