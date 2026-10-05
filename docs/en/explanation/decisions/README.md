# Architecture decisions

English | [简体中文](../../../zh-CN/explanation/decisions/README.md)

These records preserve context, alternatives, and consequences of important choices, explaining why the current system works this way. [Specifications/contracts](../../../../specs/README.md) define current behavior and fields. Use the [historical archive](../../history/README.md) for outcomes by iteration.

Choose a topic below rather than reading every ID. `accepted` means the decision was adopted; current applicability is stated per record and in the table. It does not imply all historical formats remain valid.

## Storage, formats, and recovery

| Decision | Status | Current applicability |
| --- | --- | --- |
| [D-033: Store schema 2 rejection](D-033-store-schema-2.md) | `accepted` | One format, preserved/rejected old data; D-040/D-041 supersede schema 2 / cli-result/v2 |
| [D-034: Length-framed digest](D-034-workbook-digest-v2.md) | `accepted` | workbook-digest/v2 and output-path aliases |
| [D-035: Root write lock](D-035-root-write-lock-fs4.md) | `accepted` | Serialized writes, lock identity, revision checks |
| [D-037: Managed files and safe APIs](D-037-managed-file-handles.md) | `accepted` | Directory handles, object/byte checks; D-044 limits platform scope |
| [D-038: Retain root lock during purge](D-038-purge-lock-lifecycle.md) | `accepted` | Stable root/lock, accurate partial cleanup |
| [D-039: Read-only SQLite controls](D-039-sqlite-read-control-files.md) | `accepted` | Business reads versus permitted control-file changes |
| [D-040: One result/resume format](D-040-result-resume-format.md) | `accepted` | Frozen results/same-snapshot resume; D-041 supersedes schema 3 / cli-result/v3 |
| [D-041: Creation number and replacement](D-041-attempt-number-and-replacement.md) | `accepted` | schema 4 / cli-result/v4, number, one replacement |

## Identity, authorization, and delivery

| Decision | Status | Current applicability |
| --- | --- | --- |
| [D-036: Actual OS principal](D-036-os-principal-from-uid.md) | `accepted` | Audit/approval account accounting, without independent human authentication |
| [D-042: Raw bytes and external copies](D-042-final-artifact-copy.md) | `accepted` | Public results, independent export, non-overwrite, failure boundaries |
| [D-044: macOS/APFS scope](D-044-macos-apfs-product-scope.md) | `accepted` | Current support/acceptance scope, without retrospective evidence of coverage/benefit |

## Engineering and documentation governance

| Decision | Status | Current applicability |
| --- | --- | --- |
| [D-032: Change packages](D-032-use-change-packages.md) | `accepted` | Adopted scope, active plans, commit ownership, closure qualification; D-045 refines document types |
| [D-043: Development target](D-043-development-target-authority.md) | `accepted` | Separate candidate base version, implementation progress, and release identity |
| [D-045: Specifications and documentation](D-045-specs-and-diataxis.md) | `accepted` | specs authority, reader-oriented docs, closed-change archives |

## Reading and maintaining records

[MVP summary](mvp.md) preserves D-01–D-31 reasons and partial successor relationships. Full milestone reviews, first runs, and repeated reviews are in [fixed snapshots](../../how-to/maintain-docs.md#read-original-historical-records).

One important decision per Markdown file: status, adoption date, related change, context, decision, alternatives, consequences, and verification. Retain IDs/original reasons and state current applicability on the first screen.

When wholly superseded, use `superseded by D-nnn` and bidirectional links. For partial supersession, retain adopted status and identify invalidated choices, remaining principles, and successors. Historical/current differences must be explicit; do not silently rewrite adoption facts.
