#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../../.."
export RUSTC_WRAPPER=
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/private/tmp/sheltie-c005-target}"
nextest=(cargo nextest run)
if [[ ${SHELTIE_NEXTEST_VERSION_OVERRIDE:-0} == 1 ]]; then
  echo 'Supplemental execution: --override-version-check; the configured-version gate remains not_run.' >&2
  nextest+=(--override-version-check)
fi
task_filter() {
  scripts/test-owners.sh | awk -v task="$1" '$1 == task { printf "%stest(/(^|::)%s$/)", (n++ ? " | " : ""), $2 }'
}
case "${1:-}" in
  primitives|future-red|feature)
    task=C005-T02
    [[ $1 != primitives ]] || task=C005-T01
    filter=$(task_filter "$task")
    [[ -n $filter ]] || { echo "No registered tests for $task" >&2; exit 2; }
    if [[ ${SHELTIE_NEXTEST_VERSION_OVERRIDE:-0} != 1 ]]; then
      scripts/task.sh "$task"
    else
      "${nextest[@]}" --all-features --no-tests=fail --run-ignored all --no-fail-fast -E "$filter"
    fi
    ;;
  regression)
    "${nextest[@]}" --all-features --no-tests=fail --no-fail-fast
    ;;
  gates)
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
  *) echo 'Usage: commands.sh primitives|future-red|feature|regression|gates' >&2; exit 2 ;;
esac
