#!/usr/bin/env bash
# Run one task's tests, including ignored tests so placeholders can show red before activation.
# Ownership comes from adjacent Task comments; check-tests.sh verifies them against task cards.
# Zero matching tests is a failure. Remove ignore attributes before committing; check-task verifies.
set -euo pipefail

task="${1:?Usage: scripts/task.sh Tnn}"

cd "$(dirname "$0")/.."
filter="$(scripts/test-owners.sh | awk -v t="$task" '$1 == t { printf "%stest(/(^|::)%s$/)", (n++ ? " | " : ""), $2 }')"
if [ -z "$filter" ]; then
	echo "task.sh: no tests owned by ${task} (// Task: ${task})" >&2
	exit 1
fi
exec cargo nextest run --all-features --no-tests=fail --run-ignored all -E "$filter"
