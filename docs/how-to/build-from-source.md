# Build current source

English | [简体中文](build-from-source.zh-CN.md)

Use this for the 0.3.0-rc.1 development line. Builds yield a trusted binary without installing/replacing user versions. Requires a source checkout on macOS aarch64/APFS, repository toolchain, Git, Python 3, and Bash.

Run from the source root in one Bash session. Stop immediately on any failure, retaining stdout/stderr/exit. First use must not target an old Store or clear old data.

```bash
project_repo=$(pwd -P)
session_dir=$(mktemp -d /private/tmp/sheltie-source.XXXXXX)
source_home="$session_dir/home"
RUSTC_WRAPPER= CARGO_TARGET_DIR="$session_dir/target" \
  cargo build -p sheltie-cli --bin sheltie --locked --message-format=json \
  > "$session_dir/build.json"
```

After exit 0, extract actual `executable` from Cargo JSON instead of guessing `target/debug`:

```bash
engine_binary=$(python3 - "$session_dir/build.json" <<'PY'
import json, os, sys
from pathlib import Path
paths = set()
for line in Path(sys.argv[1]).read_text().splitlines():
    item = json.loads(line)
    if (item.get('reason') == 'compiler-artifact'
            and item.get('target', {}).get('name') == 'sheltie'
            and item.get('executable')):
        paths.add(item['executable'])
assert len(paths) == 1, paths
binary = Path(paths.pop())
assert binary.is_absolute() and binary.is_file() and os.access(binary, os.X_OK)
print(binary)
PY
)
"$engine_binary" --home "$source_home" --json self version
```

Stop on parsing failure. Verify `data.version=0.3.0-rc.1`, `data.schema_version=4`, and `data.home` matching this source_home. Record actual absolute engine_binary/source_home/session_dir paths. Every later call uses the same `--home`; restore those values after reopening rather than falling back to SHELTIE_HOME/~/.sheltie. The new root does not yet exist; `self version` does not create it.

## Use the build

Continue with the [tutorial](../tutorials/first-work.md) for basics or [code-change](run-code-change.md) for real repository tasks. Builds create an independent session directory; `self version` does not create business Home. Method installation/Work startup initialize it through their writes. Builds replace neither installed binaries nor old Stores.
