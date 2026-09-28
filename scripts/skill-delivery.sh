#!/usr/bin/env bash
# skill 交付生成与校验（规格 GF-18）。
#
# 仓库里 skills/sheltie/SKILL.md 可以保留指向 specs/contracts 的链接：合同就是单一权威，
# 不复制第二份。交付是生成物——pack 把 SKILL.md 指向的合同收进 references/ 并改写链接，
# 再把交付内解析不到的本地链接去掉链接、只留文字。于是交付目录自包含：复制到任何目录后
# 全部引用仍可解析，不要求用户保留仓库路径。
#
# 用法：
#   scripts/skill-delivery.sh pack <out-dir>   生成交付目录（SKILL.md 与 references/）
#   scripts/skill-delivery.sh tar <out.tar.gz> 生成发布资产（tar 根目录为 sheltie/）
#   scripts/skill-delivery.sh verify <dir>     校验交付目录：
#     1. 每条本地链接在交付内解析得到（越出交付、指向不存在的文件都失败）；
#     2. SKILL.md 的每条 `sheltie <group> <verb>` 都在交付内 references/protocol.md §2；
#     3. 与从当前仓库重新生成的一份逐文件相同：缺文件、多余文件、字节不同都失败。
#        「由单一权威生成并校验」就落在这一步——漏同步一份 reference 必被发现。
#
# 两条边界，用之前要知道：
#   · verify 第 3 步拿的是「当前这棵树」重新生成的一份来比，所以只适合校验同一源码的
#     交付（发布时打包即校验、T16 核 rc 装出的副本）。拿旧版本的交付对着新树校验会
#     报「不一致」，那是预期，不是缺陷。
#   · 交付只含 SKILL.md 与 references/ 两份合同。合同里指向交付外的文件（storage.md、
#     roadmap、decisions 等）在交付里是去掉链接的文字，不随包发布——自包含与死链之间
#     只有这条路（GF-18）。要看那些内容请回仓库 specs/。
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
skill_dir="$root/skills/sheltie"
sentinel="__SHELTIE_DEL__"

die() {
	echo "skill-delivery: $*" >&2
	exit 1
}

# 与 check-docs.sh 同一取法：](目标)，跳过 URL 与纯锚点，目标含 #锚点时原样保留。
targets_of() {
	grep -o '](\.\{0,2\}[^)#: ]*\(#[^)]*\)\{0,1\})' "$1" 2>/dev/null | sed 's/^](//; s/)$//' | sort -u || true
}

# 目标解析成绝对路径；所在目录不存在时失败。
abspath() {
	local p="$1" dir base
	dir="$(dirname "$p")"
	base="$(basename "$p")"
	(cd "$dir" 2>/dev/null && printf '%s/%s\n' "$(pwd -P)" "$base") || return 1
}

# $1 是否为 $2 目录下的路径。
inside() {
	case "$1" in
	"$2"/*) return 0 ;;
	*) return 1 ;;
	esac
}

# 把文件里 ](old) 字面替换成 ](new)；new 为 DEL 时去掉链接、只留 [文字] 里的文字。
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
			die "skill-delivery: 去链接失败（残留标记）\n" if $text =~ /\x01DEL\x01/;
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
	# 输出目录只准是尚不存在或已空的目录：绝不 rm -rf 整棵树，防止误传仓库根或 skill 源目录。
	case "$out" in
	"" | "/" | "." | ".." | "$root" | "$root/" | "$skill_dir" | "$skill_dir/")
		die "拒绝的交付输出路径：$out"
		;;
	esac
	if [ -e "$out" ] && { [ ! -d "$out" ] || [ -n "$(ls -A "$out")" ]; }; then
		die "交付输出路径已存在且非空，拒绝覆盖：$out"
	fi
	mkdir -p "$out"
	cp -R "$skill_dir"/. "$out"/

	local out_abs skill_abs
	out_abs="$(cd "$out" && pwd -P)"
	skill_abs="$(cd "$skill_dir" && pwd -P)"
	if grep -q "$sentinel" "$out/SKILL.md" 2>/dev/null; then
		die "SKILL.md 含保留标记 $sentinel"
	fi

	# 阶段 1：SKILL.md 指向 skill 目录之外的本地链接 → 生成 references/<文件名> 并改写链接。
	local t filepart anchor src base new
	while IFS= read -r t; do
		[ -n "$t" ] || continue
		filepart="${t%%#*}"
		anchor=""
		case "$t" in
		*#*) anchor="#${t#*#}" ;;
		esac
		src="$(abspath "$skill_abs/$filepart")" || die "SKILL.md 的链接解析不到：$t"
		[ -f "$src" ] || die "SKILL.md 的链接指向不存在的文件：$t"
		if inside "$src" "$skill_abs"; then
			continue
		fi
		base="$(basename "$filepart")"
		mkdir -p "$out/references"
		if [ -f "$out/references/$base" ]; then
			cmp -s "$src" "$out/references/$base" || die "references 文件名冲突：$base"
		else
			cp "$src" "$out/references/$base"
		fi
		new="references/$base$anchor"
		apply_link_edit "$out/SKILL.md" "$t" "$new"
	done < <(targets_of "$skill_abs/SKILL.md")

	# 阶段 2：交付内每个 md 的本地链接——在交付内解析得到就留着，否则去链接只留文字。
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
	# 先校验再打包：挂出去的资产一定是过了 verify 的那份。
	verify "$tmp/sheltie"
	tar -czf "$out_tar" -C "$tmp" sheltie
	rm -rf "$tmp"
}

verify() {
	local dir="$1" dir_abs
	dir_abs="$(cd "$dir" 2>/dev/null && pwd -P)" || die "交付目录不存在：$dir"
	[ -f "$dir_abs/SKILL.md" ] || die "交付缺 SKILL.md：$dir_abs"

	# 1. 链接闭合。
	local f t filepart src rel
	while IFS= read -r f; do
		while IFS= read -r t; do
			[ -n "$t" ] || continue
			filepart="${t%%#*}"
			[ -n "$filepart" ] || continue
			src="$(abspath "$(dirname "$f")/$filepart")" || die "交付里的链接解析不到：$f → $t"
			if [ ! -f "$src" ]; then
				rel="${src#"$dir_abs"/}"
				die "交付里的链接指向不存在的文件：$f → ${t}（交付内 ${rel}）"
			fi
			inside "$src" "$dir_abs" || die "交付里的链接越出交付目录：$f → $t"
		done < <(targets_of "$f")
	done < <(find "$dir_abs" -name '*.md')

	# 2. 引用命令：SKILL.md 里的每条 sheltie <group> <verb> 都能在交付内协议 §2 对上。
	local whitelist pair
	if grep -qE '^[[:space:]]*sheltie ' "$dir_abs/SKILL.md"; then
		[ -f "$dir_abs/references/protocol.md" ] || die "交付缺 references/protocol.md"
		whitelist="$(awk '
			/^## 2\. 操作一览/ { inside = 1; next }
			/^## / { inside = 0 }
			inside
		' "$dir_abs/references/protocol.md" | grep -oE '^\| `[a-z]+ [a-z-]+' | sed 's/^| `//' | sort -u)"
		while IFS= read -r hit; do
			pair="$(printf '%s' "${hit#*:}" | sed 's/^[[:space:]]*sheltie[[:space:]]\{1,\}//' | awk '{ print $1, $2 }')"
			printf '%s\n' "$whitelist" | grep -qxF "$pair" || die "交付 SKILL.md 的命令不在交付协议 §2：sheltie $pair"
		done < <(grep -nE '^[[:space:]]*sheltie ' "$dir_abs/SKILL.md" || true)
	fi

	# 3. 与从当前仓库重新生成的一份逐文件比对：漏同步的 reference 在这里现形。
	local fresh rel
	fresh="$(mktemp -d)"
	pack "$fresh/sheltie"
	while IFS= read -r rel; do
		[ -f "$dir_abs/$rel" ] || die "交付缺文件：$rel"
		cmp -s "$fresh/sheltie/$rel" "$dir_abs/$rel" || die "交付文件与生成结果不一致：$rel"
	done < <(cd "$fresh/sheltie" && find . -type f | sed 's|^\./||' | sort)
	while IFS= read -r rel; do
		[ -f "$fresh/sheltie/$rel" ] || die "交付多余文件：$rel"
	done < <(cd "$dir_abs" && find . -type f | sed 's|^\./||' | sort)
	rm -rf "$fresh"
}

case "${1:-}" in
pack)
	[ "$#" -eq 2 ] || die "用法：skill-delivery.sh pack <out-dir>"
	pack "$2"
	;;
tar)
	[ "$#" -eq 2 ] || die "用法：skill-delivery.sh tar <out.tar.gz>"
	tar_pack "$2"
	;;
verify)
	[ "$#" -eq 2 ] || die "用法：skill-delivery.sh verify <dir>"
	verify "$2"
	;;
*)
	die "用法：skill-delivery.sh {pack|tar|verify} <目标>"
	;;
esac
