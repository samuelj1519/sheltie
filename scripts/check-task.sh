#!/usr/bin/env bash
# 提交前核对一个任务没有越界。用法：scripts/check-task.sh <task> [基准提交] [--staged]
# MVP task 默认取最近一个 tNN-* tag；Cnnn-Tnn / Cnnn-Mn 默认取 change package README 的基线。
#
# 检查：
#   1. 改动范围是并集：基准以来提交说明含当前 `Task:` 的提交改动 ∪ 未提交改动。
#      每条都要在任务表的 files 或 test_files 里（package plan、tasks.toml 始终允许）。
#   2. files 里没有本任务的 todo!("<task>") 与不带标签的 todo!()。#[allow(unused_variables)]
#      只准留在还有 todo!() 的函数上；别的任务的占位按 plan.md 规则 8 原样保留。
#      files 条目可以是文件或目录（目录是检查 1 的白名单前缀）；目录只取其中的 .rs，
#      文档会引用 todo!() 字样。
#   3. 仓库里没有残留 #[ignore = "<task>"]。
#   4. test_files 相对基准的测试代码零改动，只允许删 #[ignore 行（allow_test_changes = true 的任务除外）。
#      与 files 重叠的是混合源文件：实现填充必须动文件，改查 #[cfg(test)] 起的测试模块。快照零改动。
#   5. plan.md 本任务状态为 done。
#   6. 最近一个属于当前任务的提交含 `Task:` 与 `Agent:` 两行（工作树干净时才检查，--staged 跳过）。
set -euo pipefail

task=""
# 默认基准：最近一个 tNN-* tag（t01-skeleton，或复核者补测试后打的 tNN-review）。
base="$(git describe --tags --abbrev=0 --match 't[0-9]*' 2>/dev/null || echo t01-skeleton)"
base_given=0
staged=0
for arg in "$@"; do
	case "$arg" in
		--staged) staged=1 ;;
		-*)
			echo "check-task: 未知参数 $arg" >&2
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
	echo "用法: scripts/check-task.sh <task> [base] [--staged]" >&2
	exit 2
fi
cd "$(dirname "$0")/.."

task_table="specs/releases/v0.1.0/tasks.toml"
plan_file="specs/releases/v0.1.0/plan.md"
case "$task" in
C[0-9][0-9][0-9]-T[0-9][0-9]|C[0-9][0-9][0-9]-M[0-9]*)
	change_id="${task%%-*}"
	change_dir="$(find specs/changes/active specs/changes/completed -mindepth 1 -maxdepth 1 -type d -name "${change_id}-*" -print)"
	if [ -z "$change_dir" ] || [ "$(printf '%s\n' "$change_dir" | grep -c .)" -ne 1 ]; then
		echo "check-task: active/completed change $change_id 不存在或不唯一" >&2
		exit 2
	fi
	task_table="$change_dir/tasks.toml"
	plan_file="$change_dir/plan.md"
	for required in "$task_table" "$plan_file"; do
		[ -f "$required" ] || {
			echo "check-task: 缺 $required" >&2
			exit 2
		}
	done
	if [ "$base_given" -eq 0 ]; then
		base="$(sed -n 's/^基线：`\([^`]*\)`.*/\1/p' "$change_dir/README.md")"
		[ -n "$base" ] || {
			echo "check-task: $change_dir/README.md 缺基线" >&2
			exit 2
		}
	fi
	;;
esac

status=0
fail() { echo "check-task: $*"; status=1; }

# 读任务表里本任务的数组字段（简单 TOML，一行一个数组）。
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

# macOS 自带 bash 3.2 没有 mapfile，用 while read 填数组。
files=()
while IFS= read -r line; do [ -n "$line" ] && files+=("$line"); done < <(field files)
test_files=()
while IFS= read -r line; do [ -n "$line" ] && test_files+=("$line"); done < <(field test_files)
allow_test_changes="$(scalar allow_test_changes)"

if ! awk -v t="[$task]" '$0 == t { found = 1; exit } END { exit !found }' "$task_table"; then
	fail "$task_table 里没有 $task"
	exit 1
fi
if [ "${#files[@]}" -eq 0 ] && [ "${#test_files[@]}" -eq 0 ]; then
	fail "$task_table 里的 $task 没有允许的文件或测试文件"
	exit 1
fi

in_list() {
	local path="$1"; shift
	for allowed in "$@" "$plan_file" "$task_table"; do
		case "$path" in
			"$allowed" | "$allowed"/*) return 0 ;;
		esac
	done
	return 1
}

# 本次提交的范围：未提交改动 ∪ 基准以来提交说明含当前 `Task:` 的提交改动。
# 两条路径是并集：工作树有脏文件也不能掩盖已提交的越界改动。
changed_paths() {
	git diff --name-only --cached
	git diff --name-only
	while IFS= read -r c; do
		[ -z "$c" ] && continue
		# 不用 grep -q：提前关管道会让 git log 吃 SIGPIPE，pipefail 下误判为不匹配。
		if git log -1 --format=%B "$c" | grep "^Task: $task$" >/dev/null; then
			git diff-tree --no-commit-id --name-only -r "$c"
		fi
	done < <(git rev-list "$base"..HEAD 2>/dev/null || true)
}

# 1. 改动文件白名单（bash 3.2 + set -u 下空数组不能直接展开，用 ${arr[@]+"${arr[@]}"}）
while IFS= read -r changed; do
	[ -z "$changed" ] && continue
	if ! in_list "$changed" ${files[@]+"${files[@]}"} ${test_files[@]+"${test_files[@]}"}; then
		fail "越界改动：$changed"
	fi
done < <(changed_paths)

# 2. 本任务占位清零；填完的函数不许留 #[allow(unused_variables)]
# files 条目可以是文件或目录（目录是检查 1 的白名单前缀）；只检查其中的 .rs，
# 文档（.md 等）会引用 todo!() 字样，不是占位。
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
			fail "$f 里还有本任务的占位"
		fi
		# 每个 allow 管到下一个 allow 或测试模块为止；这段里已无 todo!() 说明函数填完了，allow 必须删掉。
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
			echo "$f 行 ${residue}：函数已填完却留着 #[allow(unused_variables)]"
			fail "$f 里有填完未删的 #[allow(unused_variables)]"
		fi
	done
done

# 3. 残留禁用
if grep -rn "#\[ignore = \"$task\"\]" crates >/dev/null; then
	grep -rn "#\[ignore = \"$task\"\]" crates | head -5
	fail "还有 $task 的测试没启用"
fi

# 4. 测试代码只删了禁用标记；快照零改动
if [ "$allow_test_changes" != "true" ]; then
	for f in ${test_files[@]+"${test_files[@]}"}; do
		[ -e "$f" ] || continue
		mixed=0
		for src in ${files[@]+"${files[@]}"}; do [ "$f" = "$src" ] && mixed=1; done
		if [ "$mixed" -eq 1 ]; then
			before="$(git show "$base:$f" 2>/dev/null | sed -n '/#\[cfg(test)\]/,$p' | grep -vE '^[[:space:]]*#\[ignore' || true)"
			after="$(sed -n '/#\[cfg(test)\]/,$p' "$f" | grep -vE '^[[:space:]]*#\[ignore' || true)"
			if [ "$before" != "$after" ]; then
				fail "$f 的测试代码被改动（只允许删 #[ignore 行）"
				diff -u <(printf '%s\n' "$before") <(printf '%s\n' "$after") | head -20 || true
			fi
		else
			if git diff "$base" -- "$f" | grep -E '^[+-]' | grep -vE '^(\+\+\+|---)' | grep -vE '^-[[:space:]]*#\[ignore' | grep -q .; then
				fail "$f 有除删禁用标记之外的改动"
			fi
		fi
	done
	if git diff --name-only "$base" -- '**/snapshots/*.snap' | grep -q .; then
		fail "快照被改动"
	fi
fi

# 5. 计划状态
if ! grep -qE "^\| $task \| done \|" "$plan_file"; then
	fail "$plan_file 里 $task 的状态不是 done"
fi

# 6. 提交信息（提交前工作树不干净，跳过；--staged 也跳过）。
# 检查最近一个属于本任务的提交，而不是 HEAD；这样任务关闭后仍可复查。
if [ "$staged" -eq 0 ] && [ -z "$(git status --porcelain)" ]; then
	task_commit=""
	while IFS= read -r c; do
		if git log -1 --format=%B "$c" | grep "^Task: $task$" >/dev/null; then
			task_commit="$c"
			break
		fi
	done < <(git rev-list HEAD)
	if [ -z "$task_commit" ]; then
		fail "历史中没有含 'Task: $task' 的提交"
		exit 1
	fi
	msg="$(git log -1 --format=%B "$task_commit")"
	echo "$msg" | grep -q "^Task: $task$" || fail "提交信息缺 'Task: $task'"
	echo "$msg" | grep -q "^Agent: " || fail "提交信息缺 'Agent:'"
fi

[ "$status" -eq 0 ] && echo "check-task: $task OK"
exit "$status"
