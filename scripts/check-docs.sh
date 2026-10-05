#!/usr/bin/env bash
# Documentation gate: resolve relative links and reject prohibited implementation wording.
# Usage: scripts/check-docs.sh [directories...] (all repository Markdown by default).
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

if [ "$#" -gt 0 ]; then
	files=$(find "$@" -name '*.md' -not -path '*/target/*')
else
	files=$(find . -name '*.md' -not -path './target/*' -not -path './.git/*')
fi

status=0

# 1. Resolve relative links.
while IFS= read -r file; do
	dir=$(dirname "$file")
	links=$(grep -o '](\.\{0,2\}[^)#: ]*\(#[^)]*\)\{0,1\})' "$file" | sed 's/^](//; s/)$//; s/#.*$//' || true)
	for target in $links; do
		if [ ! -e "$dir/$target" ]; then
			echo "Broken link  $file -> $target"
			status=1
		fi
	done
done <<<"$files"

# 2. Documentation describes the target; reject wording that makes inherited code the authority.
# Source identifiers are checked by check-core-vocab.sh; documents may cite them as counterexamples.
blacklist='沿现有|复用现有|保留现有|沿当前|旧代码|历史代码|双 reader|along existing|reuse existing|retain existing|along current|old code|historical code|dual reader'
if grep -nE "$blacklist" $files; then
	echo "Prohibited wording found (see above)"
	status=1
fi

# 3. Decision IDs must be unique so D-nn references are unambiguous.
dup=$(grep -oE '^## D-[0-9]+' docs/explanation/decisions/mvp.md | sort | uniq -d || true)
if [ -n "$dup" ]; then
	echo "Duplicate decision IDs: ${dup}"
	status=1
fi

if [ "$status" -eq 0 ]; then
	echo "check-docs: OK ($(echo "$files" | wc -l | tr -d ' ') files)"
fi
exit "$status"
