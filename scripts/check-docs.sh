#!/usr/bin/env bash
# 文档机械门禁：相对链接必须能解析；描述不存在代码的措辞与被禁概念零命中。
# 用法：scripts/check-docs.sh [目录...]   默认检查仓库内全部 Markdown。
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

if [ "$#" -gt 0 ]; then
	files=$(find "$@" -name '*.md' -not -path '*/target/*')
else
	files=$(find . -name '*.md' -not -path './target/*' -not -path './.git/*')
fi

status=0

# 1. 相对链接可解析
while IFS= read -r file; do
	dir=$(dirname "$file")
	links=$(grep -o '](\.\{0,2\}[^)#: ]*\(#[^)]*\)\{0,1\})' "$file" | sed 's/^](//; s/)$//; s/#.*$//' || true)
	for target in $links; do
		if [ ! -e "$dir/$target" ]; then
			echo "断链  $file -> $target"
			status=1
		fi
	done
done <<<"$files"

# 2. 禁用措辞。文档只描述目标形状，不得以「现有代码」为主语；命中即失败。
# 代码标识符（Verdict、PackageId 等）由 scripts/check-core-vocab.sh 查源码，文档可以把它们当反例引用。
blacklist='沿现有|复用现有|保留现有|沿当前|旧代码|历史代码|双 reader'
if grep -nE "$blacklist" $files; then
	echo "禁用措辞命中（见上）"
	status=1
fi

if [ "$status" -eq 0 ]; then
	echo "check-docs: OK ($(echo "$files" | wc -l | tr -d ' ') 个文件)"
fi
exit "$status"
