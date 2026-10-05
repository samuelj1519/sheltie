#!/usr/bin/env bash
# Generate and verify self-contained skill delivery (GF-18).
#
# The repository skill links to the authoritative specs/contracts without maintaining duplicates.
# Packaging collects those contracts under references/ and rewrites their links.
# Links outside the delivery become plain text, making the resulting directory self-contained
# without requiring a source checkout.
#
# Usage:
#   skill-delivery.sh pack <out-dir>   generate SKILL.md and references/
#   skill-delivery.sh tar <out.tar.gz> create the release asset rooted at sheltie/
#   skill-delivery.sh verify <dir>     verify:
#     1. Local links remain inside the delivery and point to existing files.
#     2. Every skill CLI group/verb appears in the delivered protocol section 2.
#     3. Every file equals a fresh generation from the same source, with no missing/extra files.
#        This comparison catches an unsynchronized reference.
#
# Boundaries:
#   verify compares against the current source tree, so use it for the delivery produced
#   from that source (release packaging or candidate installation). Comparing an old delivery
#   with a new tree is expected to fail.
#   External contract references become plain text rather than adding unrelated source files.
#   Self-contained delivery does not copy storage, roadmap, or decision documentation.
#   Consult repository specs for those details.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
skill_dir="$root/skills/sheltie"
sentinel="__SHELTIE_DEL__"

die() {
	echo "skill-delivery: $*" >&2
	exit 1
}

# Extract link targets as check-docs.sh does; skip URLs/pure anchors and retain target anchors.
targets_of() {
	grep -o '](\.\{0,2\}[^)#: ]*\(#[^)]*\)\{0,1\})' "$1" 2>/dev/null | sed 's/^](//; s/)$//' | sort -u || true
}

# Resolve an absolute path; fail when its parent directory is missing.
abspath() {
	local p="$1" dir base
	dir="$(dirname "$p")"
	base="$(basename "$p")"
	(cd "$dir" 2>/dev/null && printf '%s/%s\n' "$(pwd -P)" "$base") || return 1
}

# Whether path $1 is inside directory $2.
inside() {
	case "$1" in
	"$2"/*) return 0 ;;
	*) return 1 ;;
	esac
}

# Replace ](old) with ](new); DEL removes the link and retains its label.
apply_link_edit() {
	local f="$1" old="$2" new="$3"
	OLD="$old" NEW="$new" perl -e '
		use strict;
		my ($old, $new) = ($ENV{OLD}, $ENV{NEW});
		local $/;
		my $text = <STDIN>;
		my $pat = "]($old)";
		if ($new eq "DEL") {
			$text =~ s/\Q$pat\E/]\x01DEL\x01/g;
			$text =~ s/\[([^\[\]]*)\]\x01DEL\x01/$1/g;
			die "skill-delivery: link removal failed (residual marker)\n" if $text =~ /\x01DEL\x01/;
		} else {
			my $rep = "]($new)";
			$text =~ s/\Q$pat\E/$rep/g;
		}
		print $text;
	' <"$f" >"$f.tmp"
	mv "$f.tmp" "$f"
}

pack() {
	local out="$1"
	# The output must be absent or empty; never recursively remove a mistakenly supplied source/root.
	case "$out" in
	"" | "/" | "." | ".." | "$root" | "$root/" | "$skill_dir" | "$skill_dir/")
		die "refused delivery output path: $out"
		;;
	esac
	if [ -e "$out" ] && { [ ! -d "$out" ] || [ -n "$(ls -A "$out")" ]; }; then
		die "delivery output is not an empty directory; refusing overwrite: $out"
	fi
	mkdir -p "$out"
	cp -R "$skill_dir"/. "$out"/

	local out_abs skill_abs
	out_abs="$(cd "$out" && pwd -P)"
	skill_abs="$(cd "$skill_dir" && pwd -P)"
	if grep -q "$sentinel" "$out/SKILL.md" 2>/dev/null; then
		die "SKILL.md contains reserved marker $sentinel"
	fi

	# 1. Collect external skill references by basename and rewrite links into references/.
	local t filepart anchor src base new skill_source skill_relative
	while IFS= read -r skill_source; do
		skill_relative="${skill_source#"$skill_abs"/}"
	while IFS= read -r t; do
		[ -n "$t" ] || continue
		filepart="${t%%#*}"
		anchor=""
		case "$t" in
		*#*) anchor="#${t#*#}" ;;
		esac
		src="$(abspath "$(dirname "$skill_source")/$filepart")" || die "cannot resolve $skill_relative link: $t"
		[ -f "$src" ] || die "$skill_relative links to a nonexistent file: $t"
		if inside "$src" "$skill_abs"; then
			continue
		fi
		base="$(basename "$filepart")"
		mkdir -p "$out/references"
		if [ -f "$out/references/$base" ]; then
			cmp -s "$src" "$out/references/$base" || die "reference filename collision: $base"
		else
			cp "$src" "$out/references/$base"
		fi
		new="references/$base$anchor"
		apply_link_edit "$out/$skill_relative" "$t" "$new"
	done < <(targets_of "$skill_source")
	done < <(find "$skill_abs" -maxdepth 1 -type f -name 'SKILL*.md' | sort)

	# 2. Keep resolvable delivery-local links; turn external links into plain text.
	local f
	while IFS= read -r f; do
		while IFS= read -r t; do
			[ -n "$t" ] || continue
			filepart="${t%%#*}"
			[ -n "$filepart" ] || continue
			src="$(abspath "$(dirname "$f")/$filepart")" || {
				apply_link_edit "$f" "$t" "DEL"
				continue
			}
			if [ -f "$src" ] && inside "$src" "$out_abs"; then
				continue
			fi
			apply_link_edit "$f" "$t" "DEL"
		done < <(targets_of "$f")
	done < <(find "$out" -name '*.md')
}

tar_pack() {
	local out_tar="$1" tmp
	tmp="$(mktemp -d)"
	pack "$tmp/sheltie"
	# Verify before packaging: release assets contain the verified directory.
	verify "$tmp/sheltie"
	tar -czf "$out_tar" -C "$tmp" sheltie
	rm -rf "$tmp"
}

verify() {
	local dir="$1" dir_abs
	dir_abs="$(cd "$dir" 2>/dev/null && pwd -P)" || die "delivery directory does not exist: $dir"
	[ -f "$dir_abs/SKILL.md" ] || die "delivery is missing SKILL.md: $dir_abs"

	# 1. Resolve links within the delivery.
	local f t filepart src rel
	while IFS= read -r f; do
		while IFS= read -r t; do
			[ -n "$t" ] || continue
			filepart="${t%%#*}"
			[ -n "$filepart" ] || continue
			src="$(abspath "$(dirname "$f")/$filepart")" || die "cannot resolve delivery link: $f → $t"
			if [ ! -f "$src" ]; then
				rel="${src#"$dir_abs"/}"
				die "delivery link targets a nonexistent file: $f → ${t} (relative path ${rel})"
			fi
			inside "$src" "$dir_abs" || die "link escapes the delivery directory: $f → $t"
		done < <(targets_of "$f")
	done < <(find "$dir_abs" -name '*.md')

	# 2. Verify skill command pairs against the delivered protocol section 2.
	local whitelist pair skill_entry
	while IFS= read -r skill_entry; do
	if grep -niE 'store\.db|state_json|sqlite|status-card\.md' "$skill_entry"; then
		die "$skill_entry mentions storage/projection internals; teach CLI usage only"
	fi
	if grep -qE '^[[:space:]]*sheltie ' "$skill_entry"; then
		[ -f "$dir_abs/references/protocol.md" ] || die "delivery is missing references/protocol.md"
		whitelist="$(awk '
			/^## 2\. Operations/ { inside = 1; next }
			/^## / { inside = 0 }
			inside
		' "$dir_abs/references/protocol.md" | grep -oE '^\| `[a-z]+ [a-z-]+' | sed 's/^| `//' | sort -u)"
		while IFS= read -r hit; do
			pair="$(printf '%s' "${hit#*:}" | sed 's/^[[:space:]]*sheltie[[:space:]]\{1,\}//' | awk '{ print $1, $2 }')"
			printf '%s\n' "$whitelist" | grep -qxF "$pair" || die "delivery skill command is absent from protocol section 2: sheltie $pair"
		done < <(grep -nE '^[[:space:]]*sheltie ' "$skill_entry" || true)
	fi
	done < <(find "$dir_abs" -maxdepth 1 -type f -name 'SKILL*.md' | sort)

	# 3. Compare every file with a fresh generation to detect unsynchronized references.
	local fresh rel
	fresh="$(mktemp -d)"
	pack "$fresh/sheltie"
	while IFS= read -r rel; do
		[ -f "$dir_abs/$rel" ] || die "delivery is missing file: $rel"
		cmp -s "$fresh/sheltie/$rel" "$dir_abs/$rel" || die "delivery differs from fresh generation: $rel"
	done < <(cd "$fresh/sheltie" && find . -type f | sed 's|^\./||' | sort)
	while IFS= read -r rel; do
		[ -f "$fresh/sheltie/$rel" ] || die "extra delivery file: $rel"
	done < <(cd "$dir_abs" && find . -type f | sed 's|^\./||' | sort)
	rm -rf "$fresh"
}

case "${1:-}" in
pack)
	[ "$#" -eq 2 ] || die "Usage: skill-delivery.sh pack <out-dir>"
	pack "$2"
	;;
tar)
	[ "$#" -eq 2 ] || die "Usage: skill-delivery.sh tar <out.tar.gz>"
	tar_pack "$2"
	;;
verify)
	[ "$#" -eq 2 ] || die "Usage: skill-delivery.sh verify <dir>"
	verify "$2"
	;;
*)
	die "Usage: skill-delivery.sh {pack|tar|verify} <target>"
	;;
esac
