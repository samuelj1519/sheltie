#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../../.."
export RUSTC_WRAPPER=
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/private/tmp/sheltie-c006-target}"
nextest=(cargo nextest run)
if [[ ${SHELTIE_NEXTEST_VERSION_OVERRIDE:-0} == 1 ]]; then
  echo 'Supplemental execution: --override-version-check; the configured-version gate remains not_run.' >&2
  nextest+=(--override-version-check)
fi
task_filter() {
  scripts/test-owners.sh | awk -v task="$1" '$1 == task { printf "%stest(/(^|::)%s$/)", (n++ ? " | " : ""), $2 }'
}
build_binaries() {
  binary_json=$(mktemp /private/tmp/sheltie-c006-build.XXXXXX)
  cargo build -p sheltie-cli -p sheltie-export --all-features --message-format=json > "$binary_json"
  paths_json=$(mktemp /private/tmp/sheltie-c006-paths.XXXXXX)
  python3 - "$binary_json" "$paths_json" <<'PYDATA'
import sys,json,hashlib
from pathlib import Path
paths={}
for line in Path(sys.argv[1]).read_text().splitlines():
    item=json.loads(line)
    if item.get('reason')=='compiler-artifact' and item.get('executable'):
        name=item['target']['name']
        if name in ('sheltie','sheltie-export'):
            path=Path(item['executable'])
            paths[name]={'path':str(path),'sha256':hashlib.sha256(path.read_bytes()).hexdigest()}
assert set(paths)=={'sheltie','sheltie-export'},paths
Path(sys.argv[2]).write_text(json.dumps(paths,indent=2)+'\n')
print(json.dumps(paths))
PYDATA
  SHELTIE_TEST_ENGINE_BINARY=$(python3 -c 'import json,sys;print(json.load(open(sys.argv[1]))["sheltie"]["path"])' "$paths_json")
  SHELTIE_TEST_EXPORT_BINARY=$(python3 -c 'import json,sys;print(json.load(open(sys.argv[1]))["sheltie-export"]["path"])' "$paths_json")
  export SHELTIE_TEST_ENGINE_BINARY SHELTIE_TEST_EXPORT_BINARY
}

case "${1:-}" in
  primitives|future-red|feature)
    build_binaries
    task=C006-T02
    [[ $1 != primitives ]] || task=C006-T01
    filter=$(task_filter "$task")
    [[ -n $filter ]] || { echo "No registered tests for $task" >&2; exit 2; }
    if [[ ${SHELTIE_NEXTEST_VERSION_OVERRIDE:-0} != 1 ]]; then
      scripts/task.sh "$task"
    else
      "${nextest[@]}" --all-features --no-tests=fail --run-ignored all --no-fail-fast -E "$filter"
    fi
    ;;
  regression)
    build_binaries
    "${nextest[@]}" --all-features --no-tests=fail --no-fail-fast
    ;;
  gates)
    build_binaries
    cargo fmt --all -- --check
    cargo check --all-targets --all-features
    cargo clippy --all-targets --all-features -- -D warnings
    "${nextest[@]}" --all-features --no-tests=fail --no-fail-fast
    deny=(cargo deny --all-features --locked)
    [[ ${SHELTIE_DENY_OFFLINE:-0} != 1 ]] || deny+=(--offline)
    [[ -z ${SHELTIE_DENY_CONFIG:-} ]] || deny+=(--config "$SHELTIE_DENY_CONFIG")
    deny+=(check)
    "${deny[@]}"
    scripts/check-tests.sh
    scripts/check-docs.sh
    scripts/check-specs.sh
    git diff --check
    ;;
  dist-plan)
    cargo dist plan --output-format=json
    ;;
  *) echo 'Usage: commands.sh primitives|future-red|feature|regression|gates|dist-plan' >&2; exit 2 ;;
esac
