#!/usr/bin/env bash
# 引擎源码里不得出现业务词汇作为标识符（INV-1、INV-4、INV-5）；core 不得直接做文件 I/O。
set -euo pipefail
cd "$(dirname "$0")/.."

status=0
# 业务词汇：只查类型名、变体名、函数名位置（大写开头或 fn/enum/struct 后），不查注释与字符串。
banned='\b(struct|enum|fn|type|trait|mod)\s+(Verdict|Pass|Fail|Review|PackageId|PackageCatalog|Package)\b|\b(Verdict|PackageId|PackageCatalog)\s*[({]'
if grep -rnE "$banned" crates/sheltie-core/src crates/sheltie-runtime/src --include='*.rs'; then
	echo "check-core-vocab: 引擎源码出现业务词汇"
	status=1
fi

# core 不做 I/O
if grep -rnE '\bstd::fs\b|\bstd::process\b|\bstd::net\b|\bSystemTime::now\b' crates/sheltie-core/src --include='*.rs'; then
	echo "check-core-vocab: sheltie-core 不得做 I/O 或读时钟"
	status=1
fi

# core 的依赖树不含 I/O crate
if cargo tree -p sheltie-core -e normal 2>/dev/null | grep -qE '\b(rusqlite|tokio|reqwest|libsqlite3-sys)\b'; then
	echo "check-core-vocab: sheltie-core 依赖树里有 I/O crate"
	status=1
fi

[ "$status" -eq 0 ] && echo "check-core-vocab: OK"
exit "$status"
