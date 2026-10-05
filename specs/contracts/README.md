# Exact contracts

English | [简体中文](README.zh-CN.md)

Method authors, CLI consumers, and source maintainers use this directory for fields, formats, commands, and rejection conditions. See the [specification](../spec.md) for product targets, [architecture](../architecture.md) for layers, and [CONTEXT](../../CONTEXT.md) for terminology.

| Contract | Lookup | Main consumers |
| --- | --- | --- |
| [Workbook](workbook.md) | Manifest, Flow, nodes, explicit edges, input sources, outputs/results, compilation | Method authors, core parser/compiler, author tool |
| [Protocol](protocol.md) | Commands, arguments, JSON/text responses, next, errors, raw results, export | Coordinators, CLI, skill, external tools |
| [Storage](storage.md) | Schema, request identity, transactions, frozen files, effect publication/recovery, read-only and self management | Runtime, persistent-data and recovery tests |

Current source uses workbook/v1, flow/v1, cli-result/v4, work-result/v1, workbook-digest/v2, and Store schema 4. See the [document map](../README.md) for development/release differences; read older contracts at their release tags.

Before changing a contract, locate its product authority and assess every real entry point and consumer. Update fields, errors, persistent formats, and recovery observation points together. Defaults, silent compatibility, or changed test expectations must not conceal contract gaps.
