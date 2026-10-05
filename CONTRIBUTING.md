# Contributing to Sheltie

Start with the [domain vocabulary](CONTEXT.md), [current specifications](specs/README.md), and [engineering rules](specs/engineering.md). Coding agents must also follow [AGENTS.md](AGENTS.md).

## Propose a change

Use [GitHub issues](https://github.com/samuelj1519/sheltie/issues) for a reproducible bug or a concrete proposal. Include the version, platform, relevant command, expected and actual behavior, and a minimal example. Remove credentials and private input/output content.

An issue is not an adopted specification. Product changes follow the [change lifecycle](specs/changes/README.md); a person adopts the scope before implementation. For an active change, work from its plan rather than choosing unrelated proposed work.

## Develop and validate

Use the [build guide](docs/how-to/build-from-source.md) and a separate, explicit `SHELTIE_HOME`. The adopted product environment is macOS aarch64/APFS; checks on other platforms do not establish product acceptance.

Follow [implementing a change](docs/how-to/implement-change.md) and [validation](docs/how-to/validate-change.md). Run the checks required by affected consumers and the package plan. Keep unrelated working-tree changes intact.

## Language and commits

English is the default. Documentation translations use `.zh-CN.md`; Chinese Workbook variants are explicitly named. Keep command syntax, fields, IDs, format versions, authorization boundaries, and validation outcomes exact. Update paired current documentation when its meaning changes.

Write `type(scope): English summary` commits, one task per commit. Keep the required `Change:`, `Task:`, and `Agent:` trailers; preserve MVP legacy mappings. Explain the reason and actual validation in the body.

Follow the [code of conduct](CODE_OF_CONDUCT.md) in project discussions.

## Pull requests

Use the pull-request template. State the concrete problem and resulting behavior, link the adopted task or issue, and report executed checks and remaining `not_run` limits. Review must be independent of implementation. A passing engineering gate does not establish user acceptance, release approval, or deployment authorization.

Contributions are covered by the [MIT License](LICENSE). See [security reporting](SECURITY.md) for sensitive findings.
