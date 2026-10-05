#!/usr/bin/env bash
# Verify task scope before committing. Usage: scripts/check-task.sh <task> [base] [--staged]
# MVP defaults to the latest tNN-* tag; Cnnn tasks default to the change README baseline.
#
# Checks:
# 1. Scope is the union of task-labelled commits since the baseline and uncommitted changes.
#    Each path must be allowed by files/test_files; the plan and tasks.toml are always allowed.
# 2. Remove this task's labelled and unlabelled todo! placeholders. #[allow(unused_variables)]
#    remains only on functions that still have todo!; preserve other tasks' placeholders.
#    File entries may be files/directories; directories are allowlist prefixes, but placeholder
#    checks inspect Rust files only because documentation may mention todo! literally.
# 3. No #[ignore = "<task>"] may remain.
# 4. Test code may only remove ignore attributes unless allow_test_changes is true.
#    For mixed implementation/test files, compare from #[cfg(test)]; snapshots must not change.
# 5. The task state in plan.md must be done.
# 6. The latest task commit must have Task/Agent trailers; skip for dirty worktrees or --staged.
set -euo pipefail

task=""
# Default baseline: latest tNN-* tag, including skeleton or reviewer-added test tags.
base="$(git describe --tags --abbrev=0 --match 't[0-9]*' 2>/dev/null || echo t01-skeleton)"
base_given=0
staged=0
for arg in "$@"; do
	case "$arg" in
		--staged) staged=1 ;;
		-*)
			echo "check-task: unknown argument $arg" >&2
			exit 2
			;;
		*)
			if [ -z "$task" ]; then
				task="$arg"
			else
				base="$arg"
				base_given=1
			fi
			;;
	esac
done
if [ -z "$task" ]; then
	echo "Usage: scripts/check-task.sh <task> [base] [--staged]" >&2
	exit 2
fi
cd "$(dirname "$0")/.."

task_table="scripts/task-history/MVP/tasks.toml"
plan_file="scripts/task-history/MVP/ledger.md"
original_task_table="specs/releases/v0.1.0/tasks.toml"
original_plan_file="specs/releases/v0.1.0/plan.md"
case "$task" in
C[0-9][0-9][0-9]-T[0-9][0-9]|C[0-9][0-9][0-9]-M[0-9]*)
	change_id="${task%%-*}"
	change_dir="$(find specs/changes/active specs/changes/completed docs/history/changes -mindepth 1 -maxdepth 1 -type d -name "${change_id}-*" -print)"
	if [ -z "$change_dir" ] || [ "$(printf '%s\n' "$change_dir" | grep -c .)" -ne 1 ]; then
		echo "check-task: active/completed change $change_id is missing or ambiguous" >&2
		exit 2
	fi
	task_table="$change_dir/tasks.toml"
	plan_file="$change_dir/plan.md"
	original_task_table="$task_table"
	original_plan_file="$plan_file"
	case "$change_dir" in
	docs/history/changes/*)
		original_task_table="specs/changes/completed/$(basename "$change_dir")/tasks.toml"
		original_plan_file="specs/changes/completed/$(basename "$change_dir")/plan.md"
		;;
	esac
	if grep -q '^Record form: `reference`$' "$change_dir/README.md"; then
		task_table="scripts/task-history/$change_id/tasks.toml"
		plan_file="scripts/task-history/$change_id/ledger.md"
	fi
	for required in "$task_table" "$plan_file"; do
		[ -f "$required" ] || {
			echo "check-task: missing $required" >&2
			exit 2
		}
	done
	if [ "$base_given" -eq 0 ]; then
		base="$(sed -n 's/^Baseline: `\([^`]*\)`.*/\1/p' "$change_dir/README.md")"
		[ -n "$base" ] || {
			echo "check-task: $change_dir/README.md is missing a baseline" >&2
			exit 2
		}
	fi
	;;
esac

status=0
fail() { echo "check-task: $*"; status=1; }

# Read a task array from simple TOML with one array per line.
field() {
	awk -v t="[$task]" -v f="$1" '
		$0 == t { inside = 1; next }
		/^\[/ { inside = 0 }
		inside && index($0, f " =") == 1 { print; exit }
	' "$task_table" | sed -E 's/^[a-z_]+ = \[(.*)\]$/\1/; s/"//g; s/, */\n/g'
}
scalar() {
	awk -v t="[$task]" -v f="$1" '
		$0 == t { inside = 1; next }
		/^\[/ { inside = 0 }
		inside && index($0, f " =") == 1 { print $3; exit }
	' "$task_table" | tr -d '"'
}

# macOS bash 3.2 has no mapfile; fill arrays with while/read.
files=()
while IFS= read -r line; do [ -n "$line" ] && files+=("$line"); done < <(field files)
test_files=()
while IFS= read -r line; do [ -n "$line" ] && test_files+=("$line"); done < <(field test_files)
allow_test_changes="$(scalar allow_test_changes)"

if ! awk -v t="[$task]" '$0 == t { found = 1; exit } END { exit !found }' "$task_table"; then
	fail "$task_table has no task $task"
	exit 1
fi
if [ "${#files[@]}" -eq 0 ] && [ "${#test_files[@]}" -eq 0 ]; then
	fail "$task_table task $task has no allowed production/test files"
	exit 1
fi

in_list() {
	local path="$1"; shift
	for allowed in "$@" "$plan_file" "$task_table" "$original_plan_file" "$original_task_table"; do
		case "$path" in
			"$allowed" | "$allowed"/*) return 0 ;;
		esac
	done
	return 1
}

# Scope is the union of uncommitted changes and task-labelled commits since the baseline.
# Dirty files must not hide out-of-scope committed changes.
changed_paths() {
	git diff --name-only --cached
	git diff --name-only
	while IFS= read -r c; do
		[ -z "$c" ] && continue
		# Avoid grep -q: early pipe closure gives git log SIGPIPE and a false mismatch under pipefail.
		if git log -1 --format=%B "$c" | grep "^Task: $task$" >/dev/null; then
			git diff-tree --no-commit-id --name-only -r "$c"
		fi
	done < <(git rev-list "$base"..HEAD 2>/dev/null || true)
}

# 1. File allowlist; guard empty array expansion for bash 3.2 with set -u.
while IFS= read -r changed; do
	[ -z "$changed" ] && continue
	if ! in_list "$changed" ${files[@]+"${files[@]}"} ${test_files[@]+"${test_files[@]}"}; then
		fail "out-of-scope change: $changed"
	fi
done < <(changed_paths)

# 2. Remove this task's placeholders and stale allow(unused_variables) on completed functions.
# Directory entries are allowlist prefixes; inspect their Rust files only.
# Documentation references to todo! are not implementation placeholders.
for entry in ${files[@]+"${files[@]}"}; do
	[ -e "$entry" ] || continue
	targets=()
	if [ -d "$entry" ]; then
		while IFS= read -r g; do
			[ -n "$g" ] && targets+=("$g")
		done < <(find "$entry" -type f -name '*.rs' | sort)
	elif [ -f "$entry" ] && [ "${entry##*.}" = "rs" ]; then
		targets+=("$entry")
	fi
	for f in ${targets[@]+"${targets[@]}"}; do
		if grep -nE "todo!\(\"$task\"\)|todo!\(\)" "$f" >/dev/null; then
			grep -nE "todo!\(\"$task\"\)|todo!\(\)" "$f" | head -5
			fail "$f retains placeholders for this task"
		fi
		# An allow spans to the next allow/test module; remove it once its function has no todo!.
		residue="$(awk '
			/#\[allow\(unused_variables\)\]/ {
				if (seen && !has_todo) print start
				seen = 1; start = NR; has_todo = 0
				next
			}
			/#\[cfg\(test\)\]/ {
				if (seen && !has_todo) print start
				seen = 0
				next
			}
			seen && /todo!\(/ { has_todo = 1 }
			END { if (seen && !has_todo) print start }
		' "$f")"
		if [ -n "$residue" ]; then
			echo "$f lines ${residue}: completed functions retain #[allow(unused_variables)]"
			fail "$f retains stale #[allow(unused_variables)]"
		fi
	done
done

# 3. Remaining disabled tests.
if grep -rn "#\[ignore = \"$task\"\]" crates >/dev/null; then
	grep -rn "#\[ignore = \"$task\"\]" crates | head -5
	fail "some $task tests remain disabled"
fi

# 4. Test changes may only remove ignore attributes; snapshots must not change.
if [ "$allow_test_changes" != "true" ]; then
	for f in ${test_files[@]+"${test_files[@]}"}; do
		[ -e "$f" ] || continue
		mixed=0
		for src in ${files[@]+"${files[@]}"}; do [ "$f" = "$src" ] && mixed=1; done
		if [ "$mixed" -eq 1 ]; then
			before="$(git show "$base:$f" 2>/dev/null | sed -n '/#\[cfg(test)\]/,$p' | grep -vE '^[[:space:]]*#\[ignore' || true)"
			after="$(sed -n '/#\[cfg(test)\]/,$p' "$f" | grep -vE '^[[:space:]]*#\[ignore' || true)"
			if [ "$before" != "$after" ]; then
				fail "$f test code changed (only removing ignore attributes is allowed)"
				diff -u <(printf '%s\n' "$before") <(printf '%s\n' "$after") | head -20 || true
			fi
		else
			if git diff "$base" -- "$f" | grep -E '^[+-]' | grep -vE '^(\+\+\+|---)' | grep -vE '^-[[:space:]]*#\[ignore' | grep -q .; then
				fail "$f changes more than removal of ignore attributes"
			fi
		fi
	done
	if git diff --name-only "$base" -- '**/snapshots/*.snap' | grep -q .; then
		fail "snapshots changed"
	fi
fi

# 5. Plan state.
if ! grep -qE "^\| $task \| done \|" "$plan_file"; then
	fail "$plan_file task $task is not done"
fi

# 6. Commit trailers: skip for dirty worktrees or --staged.
# Inspect the latest task-labelled commit, not HEAD, so closed tasks remain verifiable.
if [ "$staged" -eq 0 ] && [ -z "$(git status --porcelain)" ]; then
	task_commit=""
	while IFS= read -r c; do
		if git log -1 --format=%B "$c" | grep "^Task: $task$" >/dev/null; then
			task_commit="$c"
			break
		fi
	done < <(git rev-list HEAD)
	if [ -z "$task_commit" ]; then
		fail "history has no commit with 'Task: $task'"
		exit 1
	fi
	msg="$(git log -1 --format=%B "$task_commit")"
	echo "$msg" | grep -q "^Task: $task$" || fail "commit message is missing 'Task: $task'"
	echo "$msg" | grep -q "^Agent: " || fail "commit message is missing 'Agent:'"
fi

[ "$status" -eq 0 ] && echo "check-task: $task OK"
exit "$status"
