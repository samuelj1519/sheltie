# Install, update, roll back, and uninstall the engine

English | [简体中文](../../zh-CN/how-to/manage-installation.md)

Manage an already obtained trusted `sheltie` binary. Verify source/version/root before installing; see [limits](../reference/limitations.md). Learning from source needs only the [build guide](build-from-source.md), without installation.

`engine_binary` is a verified absolute binary path; `management_root` is an explicitly selected absolute root. Stop on nonzero exit/ok=false, retaining complete responses/errors before further actions.

## 1. Match versions and management roots

```bash
"$engine_binary" --home "$management_root" --json self version
```

Check data.version/platform/home/schema_version. See [releases](../reference/releases/README.md) for binaries. The v0.3.0 baseline uses schema 4. No old-version compatibility or migration is supported; mismatched Stores are rejected without modification.

Choose a new root for new installs. `self install` installs the executing binary rather than fetching releases; a development build installs its candidate. After confirming paths:

```bash
"$engine_binary" --home "$management_root" --json self install
installed_binary="$management_root/bin/sheltie"
"$installed_binary" --home "$management_root" --json self version
```

Success returns `installed_to`; identical bytes return already_installed=true. Installation writes only the root, without shell configuration. Temporarily set the current shell for short commands:

```bash
export PATH="$management_root/bin:$PATH"
```

Continue using explicit `--home` so PATH versions and business roots match. Coordinator skills are separate deliverables. Review [instructions](../../../skills/sheltie/SKILL.md) and configure hosts under actual authorization; the engine does not install them.

## 2. Update released versions

Updates download release-channel packages. Verify target/platform assets/Store format/actual `SHELTIE_RELEASE_BASE` (default GitHub Releases). Updates replace only the binary; they do not migrate Store data.

Use a release-matching installed root and choose the version explicitly. This example applies only when the actual target is `0.3.0`:

```bash
"$installed_binary" --home "$management_root" --json self update --version 0.3.0
"$installed_binary" --home "$management_root" --json self version
```

Omit `v` from versions. Missing `--version` selects latest, unsuitable for frozen-candidate reproduction. Verify from/to/up_to_date/actual updated version. Manifest digests verify new bytes; stop accurately for network/platform/digest failures without skipping checks.

## 3. Roll back one binary level

After confirming root/bin/sheltie.prev belong `to` this installation and rollback is authorized:

```bash
"$installed_binary" --home "$management_root" --json self rollback
"$installed_binary" --home "$management_root" --json self version
```

Success restores the binary by moving prev back. Rollback neither restores Works nor downgrades Store, and does not prove database compatibility. If update interruption leaves only prev, invoke the same `self rollback` using that verified prev. Do not substitute a guessed download for the original rollback artifact.

## 4. Uninstall while retaining data

To remove only the engine installation:

```bash
"$installed_binary" --home "$management_root" --json self uninstall
```

Check data.kept. Default removes `bin/` and retains Store/Workbooks/Works/other data. Later queries need a matching binary. Host configuration/independently installed skills remain.

## 5. Explicitly purge managed data

Purge deletes this root's runs/methods/pending/staging/binaries, preventing later queries. Record root/full deletion scope, preserve required originals, and obtain explicit root-purge authorization. Purge is not troubleshooting or format upgrading.

Text `self uninstall --purge` requires yes. Automated JSON must explicitly confirm:

```bash
"$installed_binary" --home "$management_root" --json self uninstall --purge --yes
```

Success retains the empty root/original .lock. Failure may mean partial deletion: preserve accurate errors and inspect remaining objects/original intent, without claiming nothing happened. All self commands reject --request-id.
