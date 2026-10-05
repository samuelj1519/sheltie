# Sheltie documentation

English | [简体中文](../zh-CN/README.md)

Sheltie is a local workflow engine for coordinator agents. A Workbook holds a method; a Work holds one run. The engine records state, generates briefs, computes legal next actions, and enforces gates. The coordinator judges content and delegates tasks.

These documents serve method authors, coordinators, operators, and maintainers. They follow [Diátaxis](https://diataxis.fr/): tutorials, how-to guides, reference, and explanation.

## Start with your current goal

| Goal | Entry point |
| --- | --- |
| Learn to run your first method | [Build from source](how-to/build-from-source.md) → [First Work](tutorials/first-work.md) → [Gates and results](tutorials/gate-and-result.md) |
| Make an authorized repository change | [code-change](how-to/run-code-change.md); use [spec-dev](how-to/run-spec-dev.md) to plan first and implement task by task |
| Write or edit a method | [Write a Workbook](how-to/write-workbook.md) / [Canvas editing](how-to/edit-workbook.md) |
| Install, verify, or remove a method version | [Manage Workbooks](how-to/manage-workbooks.md) |
| Resume a session, rework, or replace execution qualification | [Resume a Work](how-to/resume-work.md) |
| Read and copy final results | [Export results](how-to/export-results.md) |
| Install, update, or uninstall the engine | [Manage installation](how-to/manage-installation.md) |
| Resolve a command error or blocked flow | [Troubleshoot](how-to/troubleshoot.md) |
| Look up arguments, states, files, and tool interfaces | [Technical reference](reference/README.md) |
| Understand concepts, modules, transactions, and recovery | [Explanation and source](explanation/README.md) |
| Change code or maintain documentation | [Implement a change](how-to/implement-change.md) / [Validate a change](how-to/validate-change.md) / [Maintain documentation](how-to/maintain-docs.md) |

## Version and reading scope

Current source is on the `0.3.0-rc.1` development line; the released version remains v0.2.0. Instructions and formats here target current source. See [release records](reference/releases/README.md) for released binaries and artifacts. Store formats do not migrate automatically. Try the development line with a new explicit management root; see [support and compatibility](reference/limitations.md).

The engine does not install host resources, invoke models, judge report quality, or publish results automatically. The authoring tool and exporter are unreleased external tools, each with separate file and permission boundaries.

## Four document types and specification authority

- [Tutorials](tutorials/README.md): fixed exercises with observable learning outcomes.
- [How-to guides](how-to/README.md): prerequisites, steps, results, and failure handling for actual goals.
- [Reference](reference/README.md): commands, data, files, tools, and support scope organized by product structure.
- [Explanation](explanation/README.md): concept relationships, design reasons, and source call chains.

[specs](../../specs/README.md) defines required product behavior. Exact fields, limits, errors, and persistent formats are maintained only in specs/contracts. docs provides summaries and usage material, without establishing a second specification. Use [CONTEXT](../../CONTEXT.md) terminology.

Implementation, verification, user acceptance, release, and measurable benefit are separate conclusions. See [implementation entry points](reference/implementation.md) and [acceptance boundaries](reference/acceptance.md). Use the [historical archive](history/README.md) for traceability; historical records are not current operating instructions.
