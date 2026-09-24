#!/usr/bin/env bash
# 只跑某个任务的测试，禁用的也跑（这样填空过程中不用先删标记就能看到红）。用法：scripts/task.sh T05
# 零个测试匹配视为失败（前缀写错）。提交前仍须删掉 #[ignore]，check-task.sh 会核对。
set -euo pipefail

task="${1:?用法: scripts/task.sh Tnn}"
prefix="$(echo "$task" | tr '[:upper:]' '[:lower:]')_"

cd "$(dirname "$0")/.."
exec cargo nextest run --all-features --no-tests=fail --run-ignored all -E "test(/^${prefix}/) | test(/::${prefix}/)"
