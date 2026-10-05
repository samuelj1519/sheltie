# Sheltie

English | [简体中文](README.zh-CN.md)

A local workflow engine for coordinator agents. People describe a method as a **Workbook**: a TOML graph with natural-language instructions. The coordinator assigns work and interprets outputs; Sheltie records state, generates briefs, computes legal next actions, and enforces human approval gates. The engine does not judge content or call models.

The current source candidate is **`0.3.0-rc.1`**, with Store schema 4; it has not been released. The latest documented release is **v0.2.0**, for **macOS aarch64**. See the [implementation baseline](docs/reference/implementation.md), [limitations](docs/reference/limitations.md), and [release records](docs/reference/releases/README.md). Historical v0.1.0 Linux assets remain unchanged.

## Start with the current source

Read [Build from source](docs/how-to/build-from-source.md), then [Your first Work](docs/tutorials/first-work.md). Both use a new, explicit management root. For an authorized repository task, follow the [code-change guide](docs/how-to/run-code-change.md).

The development candidate provides [code-change](examples/code-change/README.md), a method with implementation, independent review, and delivery stages. It also supports explicit final results and continuation from the current Attempt. These features are not included in the v0.2.0 installation below.

## Install the released version

On macOS aarch64:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/samuelj1519/sheltie/releases/latest/download/sheltie-cli-installer.sh | sh
export PATH="$HOME/.sheltie/bin:$PATH"
sheltie self version
```

The installer also suggests sourcing `~/.sheltie/bin/env` to set the same path. Install a sample Workbook from the parent of the cloned repository:

```bash
git clone --depth 1 https://github.com/samuelj1519/sheltie.git
sheltie workbook add sheltie/examples/two-step
```

Start a Work. Substitute the returned `work_id`, or its unique prefix, for `<work>`:

```bash
sheltie work start --workbook two-step --flow default --input topic="Introduce Sheltie to a newcomer"
sheltie attempt begin <work> --node outline
```

Read the returned `brief_path` and write `outline.md` to the declared output directory. Then submit and begin the next step:

```bash
sheltie attempt submit <work> --attempt outline#1.0 --summary "Outline: purpose, usage, and boundaries"
sheltie attempt begin <work> --node summary
```

Read the new brief, write `summary.md`, and submit:

```bash
sheltie attempt submit <work> --attempt summary#1.0 --summary "Completed the summary from the outline"
sheltie work status <work>          # status: succeeded
```

Find the legal next step in each response or `sheltie work status <work>`. JSON mode uses the `next` array. Output language depends on the installed version; the current source defaults to English.

Update with `sheltie self update`; restore the previous binary with `sheltie self rollback`. Rollback does not downgrade the Store. v0.2.0 uses schema 2; read historical data with its matching binary and management root.

`sheltie self uninstall` removes `bin/` while preserving the Store, Workbooks, and Works. `sheltie self uninstall --purge --yes` removes managed data and binaries but retains the management-root directory and the same `.lock`. Failure responses identify completed top-level removals and the failure location; fix the cause before repeating the operation.

## Use the coordinator skill

In Claude Code, `/sheltie` invokes the coordinator. Each release includes a self-contained `sheltie-skill.tar.gz` asset; it does not require a source checkout:

```bash
mkdir -p ~/.claude/skills
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/samuelj1519/sheltie/releases/latest/download/sheltie-skill.tar.gz | tar xz -C ~/.claude/skills/
```

## Documentation and contributions

- [Documentation](docs/README.md): tutorials, guides, reference, and explanations.
- [Specifications](specs/README.md): adopted behavior, contracts, and acceptance requirements.
- [Domain vocabulary](CONTEXT.md): shared names and definitions.
- [Contributing](CONTRIBUTING.md): development and validation.
- [Agent instructions](AGENTS.md): repository rules for coding agents.
- [Security](SECURITY.md) and [support](SUPPORT.md): reporting and help.

English is the default project language. Chinese documentation is available through `简体中文` links; Chinese Workbook variants are explicitly named. Meaningful Unicode test inputs and original historical evidence are preserved.

## Development

```bash
cargo fmt --all -- --check
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run --all-features --no-tests=pass
cargo deny check
scripts/check-docs.sh
scripts/check-specs.sh
python3 scripts/check-language.py
```

`rust-toolchain.toml` selects stable Rust. The workspace uses edition 2024 and MSRV 1.85. See [engineering rules](specs/engineering.md) for checks required by each change.

## License

Sheltie is licensed under the [MIT License](LICENSE). Third-party files with separate license notices retain their respective terms.
