#!/usr/bin/env bash
# 测试命名与归属的机械门禁。用法：scripts/check-tests.sh
#   1. 测试名说行为，不带任务前缀（不许 fn tNN_）。
#   2. 每个 #[test] 紧贴上方有 `// Task: Tnn` 或 `// Task: Cnnn-Tnn`；`// Task:` 注释不许挂在非测试上。
#   3. #[ignore = "<task>"] 的标签与 `// Task:` 相同。
#   4. 测试名全仓唯一（task.sh 按名字过滤）。
#   5. plan.md 每张任务卡「测试」行列出的名字都存在，且归属本任务（括号里的注解不算）。
set -euo pipefail
cd "$(dirname "$0")/.."

owners="$(scripts/test-owners.sh)"
status=0
fail() {
	echo "check-tests: $*" >&2
	status=1
}

while read -r task name loc ign; do
	case "$name" in t[0-9][0-9]_*) fail "$loc 测试名带任务前缀：$name" ;; esac
	[ "$task" = "-" ] && fail "$loc 测试 $name 上方缺 // Task: Tnn"
	[ "$ign" != "-" ] && [ "$ign" != "$task" ] && fail "$loc #[ignore = \"$ign\"] 与 // Task: $task 不一致"
done <<<"$owners"

comments="$(grep -rE '^[[:space:]]*// Task: (T[0-9]{2}|C[0-9]{3}-T[0-9]{2})[[:space:]]*$' crates --include='*.rs' | wc -l | tr -d ' ')"
tests="$(echo "$owners" | grep -c . || true)"
[ "$comments" = "$tests" ] || fail "// Task: 注释 $comments 条，测试 $tests 个，两者必须一一对应"

dups="$(echo "$owners" | awk '{print $2}' | sort | uniq -d)"
[ -z "$dups" ] || fail "测试名重复：$(echo $dups)"

# 任务卡：`### Tnn` 标题下以「**测试。**」开头的行，去掉全角括号注解后取反引号里的小写标识符。
plan_files=(specs/plan.md)
while IFS= read -r plan; do [ -n "$plan" ] && plan_files+=("$plan"); done < <(
	find specs/changes/active -mindepth 2 -maxdepth 2 -type f -name plan.md | sort
)
cards="$(perl -CSD -Mutf8 -ne '
	$t = $1 if /^### ((?:T\d\d)|(?:C\d{3}-T\d\d))/;
	if ($t && /^\*\*测试。\*\*/) { s/（[^）]*）//g; print "$t $1\n" while /`([a-z][a-z0-9_]*)`/g }
' "${plan_files[@]}")"
while read -r task name; do
	[ -z "$name" ] && continue
	got="$(echo "$owners" | awk -v n="$name" '$2 == n { print $1 }')"
	if [ -z "$got" ]; then
		fail "plan.md $task 列的测试 $name 不存在"
	elif [ "$got" != "$task" ]; then
		fail "plan.md $task 列的测试 $name 归属写的是 $got"
	fi
done <<<"$cards"

if [ "$status" -eq 0 ]; then
	echo "check-tests: OK ($tests 个测试，任务卡 $(echo "$cards" | grep -c .) 条)"
fi
exit "$status"
