# Explanation and source

English | [简体中文](README.zh-CN.md)

Explanations describe why the system works this way. See [guides](../how-to/README.md) for operations and [reference](../reference/README.md) for fields and commands.

- [Architecture and recovery](architecture.md): three layers, pure decisions, and the separation of transactions from file publication.
- [Source tour](source-tour.md): real call chains and tests starting at public commands.
- [Methods, state, and content judgment](workflow-model.md): Workbook, Work, Attempt, and gate relationships.
- [Authoring tool](workbook-editor.md): complete drafts and public CLI validation; see [reference](../reference/workbook-editor.md) for interfaces and limits.
- [Design sources](design-sources.md) and [product/Rust reasoning](product-rust-design.md): external reading, verification dates, and applicability.

Formal reasons for adopted choices are organized by topic in [architecture decisions](decisions/README.md). For a particular change or historical validation, use the [historical archive](../history/README.md). Current explanations do not require reading every iteration in order. External advice does not establish a product commitment.
