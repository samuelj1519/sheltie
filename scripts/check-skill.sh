#!/usr/bin/env bash
# skill 门禁（规格 GF-18）：SKILL.md 只是说明书，不含状态存储与推进判定。
#   1. 文中每条行首（去空白后）为 `sheltie ` 的命令，其 <group> <verb> 必须在
#      specs/contracts/protocol.md §2 的操作一览里。取法与 crates/sheltie-cli/tests/skill.rs 一致：
#      trim 后以 "sheltie " 开头的行，取随后两个词。
#   2. 不得提到状态存储与投影文件：store.db、state_json、sqlite、status-card.md。
#      「写入指令」无法与「提及」机械区分，提到即失败；协调者只需要 CLI 与它的输出。
set -euo pipefail
cd "$(dirname "$0")/.."

skill="skills/sheltie/SKILL.md"
protocol="specs/contracts/protocol.md"

for f in "$skill" "$protocol"; do
	if [ ! -f "$f" ]; then
		echo "check-skill: 缺 $f" >&2
		exit 1
	fi
done

# 白名单的单一事实源是协议 §2 的表：取行首反引号里的 `group verb` 两个词。
whitelist="$(awk '
	/^## 2\. 操作一览/ { inside = 1; next }
	/^## / { inside = 0 }
	inside
' "$protocol" | grep -oE '^\| `[a-z]+ [a-z-]+' | sed 's/^| `//' | sort -u)"
if [ -z "$whitelist" ]; then
	echo "check-skill: 从 $protocol §2 提取不到命令白名单" >&2
	exit 1
fi

status=0

# 1. 命令白名单
while IFS= read -r hit; do
	lineno="${hit%%:*}"
	line="${hit#*:}"
	pair="$(printf '%s' "$line" | sed 's/^[[:space:]]*sheltie[[:space:]]\{1,\}//' | awk '{ print $1, $2 }')"
	if ! printf '%s\n' "$whitelist" | grep -qxF "$pair"; then
		echo "check-skill: $skill:$lineno 的命令不在协议 §2：sheltie $pair"
		status=1
	fi
done < <(grep -nE '^[[:space:]]*sheltie ' "$skill" || true)

# 2. 状态存储与投影文件
if grep -niE 'store\.db|state_json|sqlite|status-card\.md' "$skill"; then
	echo "check-skill: $skill 提到状态存储或投影文件（见上），skill 只准教 CLI 用法（GF-18）"
	status=1
fi

[ "$status" -eq 0 ] && echo "check-skill: OK"
exit "$status"
