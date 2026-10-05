#!/usr/bin/env bash
# List one test per line: task, name, file:line, ignore label; missing fields use a dash.
# Only immediately adjacent Task comments define ownership; task.sh/check-tests.sh share this output.
set -euo pipefail
cd "$(dirname "$0")/.."
find crates -name '*.rs' -not -path '*/target/*' -print0 | sort -z | xargs -0 awk '
FNR == 1 { pend = ""; test = 0; ign = "-" }
/^[[:space:]]*\/\/ Task: (T[0-9][0-9]|C[0-9][0-9][0-9]-T[0-9][0-9])[[:space:]]*$/ {
	pend = $0; sub(/^[[:space:]]*\/\/ Task: /, "", pend); sub(/[[:space:]]*$/, "", pend)
	test = 0; ign = "-"; next
}
/^[[:space:]]*#\[test\]/ { test = 1; next }
/^[[:space:]]*#\[ignore = "(T[0-9][0-9]|C[0-9][0-9][0-9]-T[0-9][0-9])"\]/ {
	ign = $0; sub(/^[^"]*"/, "", ign); sub(/".*$/, "", ign); next
}
/^[[:space:]]*#\[/ { next }
/^[[:space:]]*(pub )?fn [a-z0-9_]+/ {
	if (test) {
		n = $0; sub(/^[[:space:]]*(pub )?fn /, "", n); sub(/[^a-z0-9_].*$/, "", n)
		printf "%s %s %s:%d %s\n", (pend == "" ? "-" : pend), n, FILENAME, FNR, ign
	}
	pend = ""; test = 0; ign = "-"; next
}
{ pend = ""; test = 0; ign = "-" }
'
