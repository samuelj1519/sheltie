#!/usr/bin/env bash
(
set -e
RUSTC_WRAPPER= CARGO_TARGET_DIR=/private/tmp/sheltie-export-build \
  cargo build -p sheltie-cli -p sheltie-export --locked --message-format=json \
  > /private/tmp/sheltie-export-build.json
python3 - <<'PY'
import json, hashlib
from pathlib import Path
paths = {}
for line in Path('/private/tmp/sheltie-export-build.json').read_text().splitlines():
    item = json.loads(line)
    if item.get('reason') == 'compiler-artifact' and item.get('executable'):
        name = item['target']['name']
        if name in ('sheltie', 'sheltie-export'):
            paths[name] = Path(item['executable'])
assert set(paths) == {'sheltie', 'sheltie-export'}, paths
for name, p in paths.items():
    print(name, p, hashlib.sha256(p.read_bytes()).hexdigest())
PY
)
