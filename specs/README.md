# Current development specifications

Released: [`v0.3.0`](../docs/en/reference/releases/v0.3.0/README.md)
Development target: `v0.3.0`
Active change: none

The specifications contain current rules and change entry points. Learning, operations, source navigation, and historical references belong in [docs](../docs/en/README.md).

| Authority | Contents |
| --- | --- |
| [constitution.md](constitution.md) | Product boundaries and invariants |
| [spec.md](spec.md) | Adopted behavior, supported environment, and acceptance requirements |
| [architecture.md](architecture.md) | Layers, state, files, and recovery constraints |
| [contracts/](contracts/README.md) | Exact Workbook, CLI, and Store definitions |
| [engineering.md](engineering.md) | Coding, testing, validation, commits, and review |
| [changes/](changes/README.md) | Proposals, implementation, and templates; the active plan alone defines progress |
| [roadmap.md](roadmap.md) | Unadopted directions and demand conditions |

Use [CONTEXT](../CONTEXT.md) terminology. Resolve conflicts in this order: constitution → specification → architecture → contracts. Fix implementation defects or explicitly adopt an upstream change; defects do not automatically become requirements. Do not independently implement a proposed change when none is active.

Current source uses 0.3.0, the first supported public baseline under [D-046](../docs/en/explanation/decisions/D-046-first-public-baseline.md); the product environment is macOS aarch64/APFS. Assess implementation, validation, release, and demonstrated benefit separately. See [implementation entry points](../docs/en/reference/implementation.md), [acceptance boundaries](../docs/en/reference/acceptance.md), and [release records](../docs/en/reference/releases/README.md). The development target defines the source base version. Numeric active product targets must match it; it does not replace release tags or artifacts.

After changes, run `scripts/check-docs.sh` and `scripts/check-specs.sh`; add `scripts/check-tests.sh` when test declarations are affected.
