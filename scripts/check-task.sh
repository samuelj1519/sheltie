#!/usr/bin/env bash
# 提交前核对一个填空任务没有越界。用法：scripts/check-task.sh T05 [基准提交，默认 t01-skeleton]
#
# 检查：
#   1. 相对基准的改动文件都在 tasks.toml 该任务的 files 或 test_files 里（plan.md、tasks.toml 始终允许）。
#   2. files 里没有残留 todo!()、#[allow(unused_variables)]。
#   3. 仓库里没有残留 #[ignore = "Tnn"]。
#   4. test_files 相对基准的 diff 只有删除 #[ignore 行（allow_test_changes = true 的任务除外）；快照零改动。
#   5. plan.md 该任务状态为 done。
#   6. 最近一次提交信息含 `Task: Tnn` 与 `Agent:` 两行（只在 HEAD 已提交时检查，用 --staged 跳过）。
set -euo pipefail

task="${1:?用法: scripts/check-task.sh Tnn [base]}"
base="${2:-t01-skeleton}"
cd "$(dirname "$0")/.."

status=0
fail() { echo "check-task: $*"; status=1; }

# 读 tasks.toml 里该任务的数组字段（简单 TOML，一行一个数组）。
field() {
	awk -v t="[$task]" -v f="$1" '
		$0 == t { inside = 1; next }
		/^\[/ { inside = 0 }
		inside && index($0, f " =") == 1 { print; exit }
	' tasks.toml | sed -E 's/^[a-z_]+ = \[(.*)\]$/\1/; s/"//g; s/, */\n/g'
}
scalar() {
	awk -v t="[$task]" -v f="$1" '
		$0 == t { inside = 1; next }
		/^\[/ { inside = 0 }
		inside && index($0, f " =") == 1 { print $3; exit }
	' tasks.toml | tr -d '"'
}

# macOS 自带 bash 3.2 没有 mapfile，用 while read 填数组。
files=()
while IFS= read -r line; do [ -n "$line" ] && files+=("$line"); done < <(field files)
test_files=()
while IFS= read -r line; do [ -n "$line" ] && test_files+=("$line"); done < <(field test_files)
allow_test_changes="$(scalar allow_test_changes)"

if [ "${#files[@]}" -eq 0 ]; then
	fail "tasks.toml 里没有 $task"
	exit 1
fi

in_list() {
	local path="$1"; shift
	for allowed in "$@" specs/plan.md tasks.toml; do
		case "$path" in
			"$allowed" | "$allowed"/*) return 0 ;;
		esac
	done
	return 1
}

# 1. 改动文件白名单
while IFS= read -r changed; do
	[ -z "$changed" ] && continue
	if ! in_list "$changed" "${files[@]}" "${test_files[@]}"; then
		fail "越界改动：$changed"
	fi
done < <(git diff --name-only "$base" HEAD; git diff --name-only --cached; git diff --name-only)

# 2. 残留占位
for f in "${files[@]}"; do
	[ -e "$f" ] || continue
	if grep -rnE 'todo!\(|#\[allow\(unused_variables\)\]' "$f" >/dev/null; then
		grep -rnE 'todo!\(|#\[allow\(unused_variables\)\]' "$f" | head -5
		fail "$f 里还有占位"
	fi
done

# 3. 残留禁用
if grep -rn "#\[ignore = \"$task\"\]" crates >/dev/null; then
	grep -rn "#\[ignore = \"$task\"\]" crates | head -5
	fail "还有 $task 的测试没解开"
fi

# 4. 测试文件只删了禁用标记
if [ "$allow_test_changes" != "true" ]; then
	for f in "${test_files[@]}"; do
		[ -e "$f" ] || continue
		if git diff "$base" -- "$f" | grep -E '^[+-]' | grep -vE '^(\+\+\+|---)' | grep -vE '^-\s*#\[ignore' | grep -q .; then
			fail "$f 有除删禁用标记之外的改动"
		fi
	done
	if git diff --name-only "$base" HEAD -- '**/snapshots/*.snap' | grep -q .; then
		fail "快照被改动"
	fi
fi

# 5. 计划状态
if ! grep -qE "^\| $task \| done \|" specs/plan.md; then
	fail "specs/plan.md 里 $task 的状态不是 done"
fi

# 6. 提交信息
if [ "${3:-}" != "--staged" ]; then
	msg="$(git log -1 --format=%B)"
	echo "$msg" | grep -q "^Task: $task$" || fail "提交信息缺 'Task: $task'"
	echo "$msg" | grep -q "^Agent: " || fail "提交信息缺 'Agent:'"
fi

[ "$status" -eq 0 ] && echo "check-task: $task OK"
exit "$status"
