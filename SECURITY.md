# Security

Sheltie's file confinement, frozen artifacts, request replay, persistence, and recovery boundaries are defined in the [constitution](specs/constitution.md) and [contracts](specs/contracts/README.md).

## Versions and environment

The latest documented release is v0.2.0 for macOS aarch64. The current `0.3.0-rc.1` source is an unreleased candidate. See [release records](docs/en/reference/releases/README.md) and [limitations](docs/en/reference/limitations.md) for precise scope; the project does not promise a maintenance period or response SLA.

## Report a vulnerability

For a sensitive finding, check the repository's [Security page](https://github.com/samuelj1519/sheltie/security) for an available private reporting channel. If one is unavailable, open an issue requesting a private contact without publishing exploit details or sensitive data.

Include the version or commit, platform, minimal reproduction, affected boundary, and observed impact. Keep credentials, private Work data, and unrelated host files out of the report. Do not exercise a vulnerability against another person's system.

Public bug reports belong in [GitHub issues](https://github.com/samuelj1519/sheltie/issues). Publication of an advisory or release requires the maintainer's decision.
