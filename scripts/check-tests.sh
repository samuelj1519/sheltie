#!/usr/bin/env bash
# Test naming/ownership gate. Usage: scripts/check-tests.sh
# 1. Names describe behavior, without tNN_ task prefixes.
# 2. Each test has an immediately preceding Task comment; non-tests must not carry one.
# 3. An ignore task label must match its Task comment.
# 4. Test names are repository-wide unique because task.sh filters by name.
# 5. Each plan task's Tests line lists existing tests owned by it; ignore parenthetical notes.
set -euo pipefail
cd "$(dirname "$0")/.."

owners="$(scripts/test-owners.sh)"
status=0
fail() {
	echo "check-tests: $*" >&2
	status=1
}

while read -r task name loc ign; do
	case "$name" in t[0-9][0-9]_*) fail "$loc test name has a task prefix: $name" ;; esac
	[ "$task" = "-" ] && fail "$loc test $name is missing an adjacent // Task: Tnn"
	[ "$ign" != "-" ] && [ "$ign" != "$task" ] && fail "$loc #[ignore = \"$ign\"] differs from // Task: $task"
done <<<"$owners"

comments="$(grep -rE '^[[:space:]]*// Task: (T[0-9]{2}|C[0-9]{3}-T[0-9]{2})[[:space:]]*$' crates --include='*.rs' | wc -l | tr -d ' ')"
tests="$(echo "$owners" | grep -c . || true)"
[ "$comments" = "$tests" ] || fail "$comments Task comments and $tests tests must correspond one-to-one"

dups="$(echo "$owners" | awk '{print $2}' | sort | uniq -d)"
[ -z "$dups" ] || fail "duplicate test names: $(echo $dups)"

# Read test identifiers from task headings and Tests lines, excluding parenthetical notes.
plan_files=(scripts/task-history/MVP/ledger.md)
while IFS= read -r plan; do [ -n "$plan" ] && plan_files+=("$plan"); done < <(
	find specs/changes/active -mindepth 2 -maxdepth 2 -type f -name plan.md | sort
)
cards="$(perl -CSD -Mutf8 -ne '
	$t = $1 if /^### ((?:T\d\d)|(?:C\d{3}-T\d\d))/;
	if ($t && /^\*\*Tests[.:]\*\*/) { s/\([^)]*\)//g; print "$t $1\n" while /`([a-z][a-z0-9_]*)`/g }
' "${plan_files[@]}")"
while read -r task name; do
	[ -z "$name" ] && continue
	got="$(echo "$owners" | awk -v n="$name" '$2 == n { print $1 }')"
	if [ -z "$got" ]; then
		fail "plan.md $task lists nonexistent test $name"
	elif [ "$got" != "$task" ]; then
		fail "plan.md $task lists test $name owned by $got"
	fi
done <<<"$cards"

if [ "$status" -eq 0 ]; then
	echo "check-tests: OK ($tests tests, $(echo "$cards" | grep -c .) task-card entries)"
fi
exit "$status"
