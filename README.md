# Sheltie

English | [简体中文](README.zh-CN.md)

**Local, resumable workflows for coordinator agents.**

Write a method as a **Workbook**: a TOML graph with natural-language instructions. Run it as a **Work**. Sheltie keeps the state, briefs, inputs, outputs, and legal next actions explicit, while the coordinator delegates tasks and judges their results.

[Documentation](docs/en/README.md) · [Quick start](#quick-start) · [Contributing](CONTRIBUTING.md) · [Releases](https://github.com/samuelj1519/sheltie/releases)

## What Sheltie provides

- **Reusable methods:** declare steps, workers, inputs, outputs, review loops, and approval gates in a Workbook.
- **Concrete task briefs:** each Attempt receives frozen input paths and declared output requirements.
- **Durable local progress:** inspect the status card and resume from recorded state after an interruption.
- **Explicit control flow:** use the legal next actions returned by the CLI, in text or JSON.

The engine is a single Rust binary backed by local SQLite. It does not call models, judge natural-language content, install host resources, or publish work automatically. The coordinator retains those responsibilities. See [scope and limitations](docs/en/reference/limitations.md).

## Choose a version

The current product environment is **macOS on Apple Silicon (`aarch64`), using APFS**.

| Track | Version | Start here |
| --- | --- | --- |
| Latest documented release | **v0.2.0** | [Release record](docs/en/reference/releases/v0.2.0/README.md) or the installation below |
| Current source candidate | **0.3.0-rc.1**, unreleased | [Build from source](docs/en/how-to/build-from-source.md), then [your first Work](docs/en/tutorials/first-work.md) |

Current source defaults to English and includes newer capabilities such as [code-change](docs/en/how-to/run-code-change.md) and [explicit result export](docs/en/how-to/export-results.md). These are not included in the v0.2.0 binary. Repository documentation describes current source; released versions retain their own CLI wording and behavior.

v0.2.0 uses Store schema 2; current source uses schema 4. Use separate management roots: Store formats do not migrate automatically. Full installation, update, rollback, and removal instructions are in the [installation guide](docs/en/how-to/manage-installation.md).

## Quick start

### Install the released engine

On macOS Apple Silicon:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/samuelj1519/sheltie/releases/download/v0.2.0/sheltie-cli-installer.sh | sh
export PATH="$HOME/.sheltie/bin:$PATH"
```

Use a fresh temporary management root for this demo. Keep the same shell open; `self version` shows the selected root and binary version.

```bash
export SHELTIE_HOME="$(mktemp -d /private/tmp/sheltie-demo.XXXXXX)"
sheltie self version
git clone --depth 1 https://github.com/samuelj1519/sheltie.git
cd sheltie
sheltie workbook add examples/two-step
sheltie work start --workbook two-step --flow default --input 'topic=Introduce Sheltie to a newcomer'
```

### Run the two-step method

Replace `<work>` with the returned `work_id` or its unique prefix. Claim the first task:

```bash
sheltie attempt begin <work> --node outline
```

Read the returned `brief_path`. Write `outline.md` at the declared output location, then submit and claim the next task:

```bash
sheltie attempt submit <work> --attempt outline#1.0 --summary 'Outline written'
sheltie attempt begin <work> --node summary
```

Read the new brief and its bound outline. Write `summary.md` at the declared output location, then finish:

```bash
sheltie attempt submit <work> --attempt summary#1.0 --summary 'Summary follows the outline'
sheltie work status <work>
```

Expect `status: succeeded` and no legal next actions. Submission checks file contracts; the coordinator still checks the writing. For the current source version, the [complete tutorial](docs/en/tutorials/first-work.md) provides commands, sample outputs, and result-selection details.

## Use Sheltie from an agent

In Claude Code, the coordinator skill is invoked with `/sheltie`. The release includes a self-contained skill asset:

```bash
mkdir -p ~/.claude/skills
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/samuelj1519/sheltie/releases/latest/download/sheltie-skill.tar.gz | tar xz -C ~/.claude/skills/
```

See the current [coordinator instructions](skills/sheltie/SKILL.md) for the CLI workflow. Chinese instructions and Workbook variants are available through explicit language links and method IDs. The engine does not install or configure the host skill.

## Documentation and contributions

- **Learn:** [first Work](docs/en/tutorials/first-work.md), [gates and results](docs/en/tutorials/gate-and-result.md).
- **Run real tasks:** [code-change](docs/en/how-to/run-code-change.md), [spec-dev](docs/en/how-to/run-spec-dev.md), [resume a Work](docs/en/how-to/resume-work.md).
- **Author methods:** [write a Workbook](docs/en/how-to/write-workbook.md), [edit a Workbook](docs/en/how-to/edit-workbook.md).
- **Look up behavior:** [CLI reference](docs/en/reference/cli.md), [specifications](specs/README.md), [domain vocabulary](CONTEXT.md).
- **Contribute or get help:** [contributing](CONTRIBUTING.md), [agent instructions](AGENTS.md), [support](SUPPORT.md), [security reporting](SECURITY.md), [code of conduct](CODE_OF_CONDUCT.md).

Read the bilingual [GitHub Wiki](https://github.com/samuelj1519/sheltie/wiki) or browse the repository documentation linked above. English is the default project language. Chinese user documentation and method instructions remain available; meaningful Unicode fixtures and original historical evidence are preserved.

## License

[MIT](LICENSE). Third-party files with separate license notices retain their respective terms.
