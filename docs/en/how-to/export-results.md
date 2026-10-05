# Obtain editable final-result copies

English | [简体中文](../../zh-CN/how-to/export-results.md)

Deliver explicit final results to later consumers or create editable copies. Current source provides raw reads and unreleased external sheltie-export. See [reference](../reference/export.md) for arguments/formats/limits/exits.

Requires a trusted current engine, accurate root for the existing Work, and explicitly authorized destination parent. Home/parent must be real absolute directories without symlinks or mutual containment. Set management_root/work_id/destination_parent to actual values; use full Work IDs.

## 1. Build and record actual binaries

Run from the source root in one Bash session. Stop on failed builds without parsing/executing old artifacts:

```bash
export_session=$(mktemp -d /private/tmp/sheltie-export.XXXXXX)
RUSTC_WRAPPER= CARGO_TARGET_DIR="$export_session/target" \
  cargo build -p sheltie-cli -p sheltie-export --locked --message-format=json \
  > "$export_session/build.json"
```

After success, extract both actual executables from Cargo JSON. Do not guess target paths or use fixed shared logs:

```bash
python3 - "$export_session/build.json" <<'PYTHON'
import hashlib, os, sys
from pathlib import Path
import json
paths = {}
for line in Path(sys.argv[1]).read_text().splitlines():
    item = json.loads(line)
    if item.get('reason') != 'compiler-artifact' or not item.get('executable'):
        continue
    name = item['target']['name']
    if name in ('sheltie', 'sheltie-export'):
        paths[name] = Path(item['executable'])
assert set(paths) == {'sheltie', 'sheltie-export'}, paths
for name, path in sorted(paths.items()):
    assert path.is_absolute() and path.is_file() and os.access(path, os.X_OK)
    print(name, path, hashlib.sha256(path.read_bytes()).hexdigest())
PYTHON
```

Assign actual paths to engine_binary/export_binary and retain printed sha256. `self version` verifies matching engine/Home/Store. Existing trusted current builds may be reused; current exporter does not interpret released old formats.

## 2. Query one result snapshot

```bash
"$engine_binary" --home "$management_root" --json work result "$work_id" \
  > "$export_session/result.json"
cat "$export_session/result.json"
```

After success verify `data.final=true`, `data.status.kind=succeeded`, `data.effects_pending=false`, and nonempty data.artifacts. For `final=false`, finish legal execution/recovery first. `final=true` with an empty set means no selections, without guessing historical files.

Ordinary queries list references; originals still require byte checks. Stop here if references suffice.

## 3. Publish a new copy

After confirming destination parent and copying authorization:

```bash
"$export_binary" --sheltie "$engine_binary" --home "$management_root" \
  --work "$work_id" --to "$destination_parent" --json \
  > "$export_session/export.json"
cat "$export_session/export.json"
```

Use `target_path` only with exit 0 and status=complete. The tool re-queries/verifies sources and raw bytes, without replacing source reads with saved JSON.

`manifest.json` retains source results/file mappings. Private indexes such as artifacts/0001/ follow key order. Use manifest relative paths, not keys as paths. Copies may be edited; initial manifests no longer guarantee edited bytes. Source Works remain unchanged.

Reruns create new copies, retaining prior edits without overwrite/merge. See [limits](../reference/export.md#files-and-formats).

## 4. Read one raw artifact

Get positive revision from the saved same-snapshot result and set `artifact_key` to its exact key. mktemp creates a new file without shell redirection truncating existing user files:

```bash
result_revision=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["data"]["revision"])' "$export_session/result.json")
local_file=$(mktemp /private/tmp/sheltie-artifact.XXXXXX)
"$engine_binary" --home "$management_root" work result "$work_id" \
  "--artifact=$artifact_key" --revision "$result_revision" > "$local_file"
```

Raw rejects --json/--request-id and adds no newline. Verify final exit, actual bytes, and sha256 against the entry. Failed reads may leave partial bytes; do not deliver that staging file.

## Failure state

| Status/exit | Handling |
| --- | --- |
| `rejected` / 2 | Check `error.code`, paths, qualification, arguments; unsuccessful targets cannot be used |
| `failed_before_publish` / 1 | Retain source/integrity/I/O errors; `staging_path` is only verified owned staging, not a complete copy |
| `publication_unconfirmed` / 3 | Move occurred but identity/parent sync unconfirmed; retain objects/manifest/bytes without automatic deletion/rollback |

Termination may provide no response. Retain stdout/stderr/exit/actual paths/permissions/source identity and inspect actual objects/manifests. Do not claim ownership by names, adopt old staging, or delete unknown directories. Report uncertainty rather than equating directory existence with completion. Reruns still create new copies.

Scope is macOS aarch64/APFS. OS synchronization is not physical power-loss durability; cross-APFS acceptance does not certify external physical devices. The tool cannot guarantee whole process-tree termination, same-permission isolation, or continued integrity after edits. See [limits](../reference/limitations.md).
