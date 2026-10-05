#!/usr/bin/env bash
# Skill gate (GF-18): instructions contain no state storage or advancement logic; delivery is self-contained.
# 1. For every trimmed line beginning with `sheltie `, the group/verb pair must appear in
#    protocol.md section 2. Use the same extraction as crates/sheltie-cli/tests/skill.rs:
#    trim, match the prefix, then read the next two words.
# 2. Reject mentions of storage/projection files: store.db, state_json, sqlite, status-card.md.
#    Mentions and write instructions cannot be mechanically distinguished; teach CLI operations only.
# 3. Resolve local skill links. Repository instructions may link to specs/contracts,
#    their single authority; skill-delivery.sh collects those references when packaging.
# 4. Generate and verify a self-contained delivery, or use --delivery for an existing directory.
#    A missing synchronized reference fails this check. Compare against the same source tree
#    used to package/install the candidate; an old delivery against a new tree must differ.
# Usage: scripts/check-skill.sh [--delivery <dir>]
set -euo pipefail
cd "$(dirname "$0")/.."

skill="skills/sheltie/SKILL.md"
protocol="specs/contracts/protocol.md"
delivery=""

while [ "$#" -gt 0 ]; do
	case "$1" in
	--delivery)
		[ "$#" -ge 2 ] || { echo "check-skill: --delivery requires a directory" >&2; exit 2; }
		delivery="$2"
		shift 2
		;;
	*)
		echo "check-skill: unknown argument $1" >&2
		exit 2
		;;
	esac
done

for f in "$skill" "$protocol"; do
	if [ ! -f "$f" ]; then
		echo "check-skill: missing $f" >&2
		exit 1
	fi
done

# The protocol section 2 table is the command authority: extract its leading group/verb pair.
whitelist="$(awk '
	/^## 2\. Operations/ { inside = 1; next }
	/^## / { inside = 0 }
	inside
' "$protocol" | grep -oE '^\| `[a-z]+ [a-z-]+' | sed 's/^| `//' | sort -u)"
if [ -z "$whitelist" ]; then
	echo "check-skill: cannot extract the command allowlist from $protocol section 2" >&2
	exit 1
fi

status=0

# 1. Command allowlist and storage boundaries apply to both maintained languages.
while IFS= read -r skill; do
while IFS= read -r hit; do
	lineno="${hit%%:*}"
	line="${hit#*:}"
	pair="$(printf '%s' "$line" | sed 's/^[[:space:]]*sheltie[[:space:]]\{1,\}//' | awk '{ print $1, $2 }')"
	if ! printf '%s\n' "$whitelist" | grep -qxF "$pair"; then
		echo "check-skill: $skill:$lineno command is absent from protocol section 2: sheltie $pair"
		status=1
	fi
done < <(grep -nE '^[[:space:]]*sheltie ' "$skill" || true)

# 2. Storage/projection mentions.
if grep -niE 'store\.db|state_json|sqlite|status-card\.md' "$skill"; then
	echo "check-skill: $skill mentions state storage/projection files (see above); teach CLI usage only (GF-18)"
	status=1
fi
done < <(find skills/sheltie -maxdepth 1 -type f -name 'SKILL*.md' | sort)

# 3. Resolve repository links using the same extraction as check-docs.sh.
while IFS= read -r file; do
	dir="$(dirname "$file")"
	while IFS= read -r target; do
		[ -n "$target" ] || continue
		if [ ! -e "$dir/$target" ]; then
			echo "check-skill: cannot resolve $file link: $target"
			status=1
		fi
	done < <(grep -o '](\.\{0,2\}[^)#: ]*\(#[^)]*\)\{0,1\})' "$file" | sed 's/^](//; s/)$//; s/#.*$//' || true)
done < <(find skills -name '*.md')

# 4. Generate and verify delivery, or verify the supplied directory.
if [ -n "$delivery" ]; then
	if ! scripts/skill-delivery.sh verify "$delivery"; then
		echo "check-skill: delivery verification failed: $delivery" >&2
		status=1
	fi
else
	tmp="$(mktemp -d)"
	if ! scripts/skill-delivery.sh pack "$tmp/sheltie" || ! scripts/skill-delivery.sh verify "$tmp/sheltie"; then
		echo "check-skill: delivery generation or verification failed" >&2
		status=1
	fi
	rm -rf "$tmp"
fi

[ "$status" -eq 0 ] && echo "check-skill: OK"
exit "$status"
