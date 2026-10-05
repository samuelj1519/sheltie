#!/usr/bin/env bash
# Engine identifiers must not use business vocabulary (INV-1, INV-4, INV-5); core performs no file I/O.
set -euo pipefail
cd "$(dirname "$0")/.."

status=0
# Check identifier positions, not comments or string literals.
banned='\b(struct|enum|fn|type|trait|mod)\s+(Verdict|Pass|Fail|Review|PackageId|PackageCatalog|Package)\b|\b(Verdict|PackageId|PackageCatalog)\s*[({]'
if grep -rnE "$banned" crates/sheltie-core/src crates/sheltie-runtime/src --include='*.rs'; then
	echo "check-core-vocab: business vocabulary in engine source"
	status=1
fi

# Core performs no I/O.
if grep -rnE '\bstd::fs\b|\bstd::process\b|\bstd::net\b|\bSystemTime::now\b' crates/sheltie-core/src --include='*.rs'; then
	echo "check-core-vocab: sheltie-core must not perform I/O or read the clock"
	status=1
fi

# Core dependencies must not include I/O crates.
if cargo tree -p sheltie-core -e normal 2>/dev/null | grep -qE '\b(rusqlite|tokio|reqwest|libsqlite3-sys)\b'; then
	echo "check-core-vocab: sheltie-core dependency tree contains an I/O crate"
	status=1
fi

[ "$status" -eq 0 ] && echo "check-core-vocab: OK"
exit "$status"
