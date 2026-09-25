#!/usr/bin/env bash
# 只跑某个任务的测试，禁用的也跑（这样填空过程中不用先删标记就能看到红）。用法：scripts/task.sh T05
# 归属看测试上方的 `// Task: Tnn` 注释，不看名字；check-tests.sh 保证注释齐全且与任务卡一致。
# 零个测试匹配视为失败（任务号写错）。提交前仍须删掉 #[ignore]，check-task.sh 会核对。
set -euo pipefail

task="${1:?用法: scripts/task.sh Tnn}"

cd "$(dirname "$0")/.."
filter="$(scripts/test-owners.sh | awk -v t="$task" '$1 == t { printf "%stest(/(^|::)%s$/)", (n++ ? " | " : ""), $2 }')"
if [ -z "$filter" ]; then
	echo "task.sh: 没有归属 ${task} 的测试（// Task: ${task}）" >&2
	exit 1
fi
exec cargo nextest run --all-features --no-tests=fail --run-ignored all -E "$filter"
