#!/usr/bin/env bash
# 列出每个测试的归属：一行一个「任务 测试名 文件:行 ignore标签」，缺的字段写 -。
# 归属只认紧贴在 #[test] 上方的 `// Task: Tnn` 或 `// Task: Cnnn-Tnn`；task.sh 与 check-tests.sh 共用。
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
