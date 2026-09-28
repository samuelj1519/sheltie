#!/usr/bin/env bash
# Specs 治理结构检查：change 生命周期、ADR、release 与 agent 入口。
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

status=0
fail() {
	echo "check-specs: $*" >&2
	status=1
}

# 取不到 tag/commit 时先分清「缺历史」与「没创建」：浅克隆或未拉全历史的检出
# （CI 的默认 checkout 就是）取不到老 tag，诊断必须点名缺历史，不能说成记录写错。
missing_history_diag() {
	if [ "$(git rev-parse --is-shallow-repository 2>/dev/null || echo false)" = "true" ]; then
		echo "缺历史：当前检出是浅克隆，取不到 $1；治理 job 要 fetch-depth: 0 拉完整历史与 tags"
	else
		echo "$1 在当前 Git 历史里找不到（未创建或未推送）"
	fi
}

for path in \
	specs/changes/README.md \
	specs/changes/proposed \
	specs/changes/active \
	specs/changes/completed \
	specs/changes/rejected \
	specs/decisions/README.md \
	specs/releases/README.md \
	specs/releases/v0.1.0/README.md \
	specs/releases/v0.1.0/plan.md \
	specs/releases/v0.1.0/decisions.md \
	specs/releases/v0.1.0/runbook.md \
	specs/releases/v0.1.0/tasks.toml; do
	[ -e "$path" ] || fail "缺 $path"
done

for legacy_root in specs/plan.md specs/decisions.md specs/t25-t26-runbook.md tasks.toml; do
	[ ! -e "$legacy_root" ] || fail "MVP 历史仍留在一级目录：$legacy_root"
done

outdated_release_refs="$(grep -RInE 'releases/(v[0-9]+\.[0-9]+\.[0-9]+|<version>)\.md' specs --include='*.md' || true)"
if [ -n "$outdated_release_refs" ]; then
	printf '%s\n' "$outdated_release_refs" >&2
	fail "文档仍引用旧式 release record 路径"
fi

packages="$(find specs/changes/proposed specs/changes/active specs/changes/completed specs/changes/rejected \
	-mindepth 1 -maxdepth 1 -type d -name 'C*' | sort)"

ids=""
while IFS= read -r package; do
	[ -n "$package" ] || continue
	name="$(basename "$package")"
	state="$(basename "$(dirname "$package")")"
	case "$name" in
	C[0-9][0-9][0-9]-*) ;;
	*) fail "$package 的目录名不是 Cnnn-<slug>" ;;
	esac
	[ -f "$package/README.md" ] || {
		fail "$package 缺 README.md"
		continue
	}
	grep -q "^状态：\`$state\`" "$package/README.md" ||
		fail "$package/README.md 的状态与目录 $state 不一致"
	for field in 目标版本 兼容性 基线 Owner; do
		head -15 "$package/README.md" | grep -q "^${field}：" || fail "$package/README.md 首屏缺 ${field}"
	done
	grep -qF "$package/README.md" specs/changes/README.md ||
		grep -qF "${package#specs/changes/}/README.md" specs/changes/README.md ||
		fail "$package 没登记在 specs/changes/README.md"
	ids="${ids}${name%%-*}
"
done <<<"$packages"

duplicates="$(printf '%s' "$ids" | sed '/^$/d' | sort | uniq -d)"
[ -z "$duplicates" ] || fail "change id 重复：$(printf '%s' "$duplicates" | tr '\n' ' ')"

active_count="$(find specs/changes/active -mindepth 1 -maxdepth 1 -type d -name 'C*' | wc -l | tr -d ' ')"
[ "$active_count" -le 1 ] || fail "active change 有 $active_count 个；默认上限是 1"

for package in specs/changes/active/C*; do
	[ -d "$package" ] || continue
	for file in README.md plan.md progress.md validation.md tasks.toml; do
		[ -f "$package/$file" ] || fail "$package 缺 active 必需文件 $file"
	done
	grep -q '^## 成功判据' "$package/README.md" || fail "$package/README.md 缺成功判据"
done

if [ "$active_count" -eq 0 ]; then
	grep -q '^Active change：无$' specs/changes/README.md || fail "change 索引没有声明 active 为空"
else
	active_name="$(find specs/changes/active -mindepth 1 -maxdepth 1 -type d -name 'C*' -exec basename {} \;)"
	grep -q "$active_name" specs/changes/README.md || fail "change 索引没有指向 active package $active_name"
fi

for package in specs/changes/completed/C*; do
	[ -d "$package" ] || continue
	for file in README.md plan.md validation.md review.md tasks.toml; do
		[ -f "$package/$file" ] || fail "$package 缺 completed 必需文件 $file"
	done
	if [ -f "$package/plan.md" ]; then
		unfinished="$(awk -F'|' '
			/^\| C[0-9][0-9][0-9]-(T[0-9][0-9]|M[0-9]+) / {
				state = $3; gsub(/^[[:space:]]+|[[:space:]]+$/, "", state)
				if (state != "done") print $0
			}
		' "$package/plan.md")"
		[ -z "$unfinished" ] || fail "$package/plan.md 还有非 done 任务"
	fi
	if [ -f "$package/validation.md" ]; then
		grep -Eq '^Candidate: `(SELF|[0-9a-f]{7,40})`' "$package/validation.md" ||
			fail "$package/validation.md 缺固定候选"
		invalid_results="$(awk -F'|' '
			/^\|/ {
				name = $2; result = $6
				gsub(/^[[:space:]]+|[[:space:]]+$/, "", name)
				gsub(/^[[:space:]]+|[[:space:]]+$/, "", result)
				if (name != "" && name != "Requirement / risk" && name !~ /^-+$/ && result != "PASS") print $0
			}
		' "$package/validation.md")"
		[ -z "$invalid_results" ] || fail "$package/validation.md 有非 PASS 验证项"
	fi
	if [ -f "$package/review.md" ]; then
		grep -q '^结论：`PASS`' "$package/review.md" || fail "$package/review.md 没有 final PASS"
	fi
done

for package in specs/changes/rejected/C*; do
	[ -d "$package" ] || continue
	grep -q '^## 否决原因' "$package/README.md" || fail "$package/README.md 缺否决原因"
	grep -q '^## 替代' "$package/README.md" || fail "$package/README.md 缺替代说明"
	awk '
		/^## 替代/ { inside = 1; next }
		/^## / { inside = 0 }
		inside { print }
	' "$package/README.md" | grep -Eq '\]\(|^无[。]?$' || fail "$package/README.md 的替代说明既无链接也未写无"
done

adr_dups="$(find specs/decisions -maxdepth 1 -type f -name 'D-[0-9][0-9][0-9]-*.md' \
	-exec basename {} \; | cut -d- -f1-2 | sort | uniq -d)"
[ -z "$adr_dups" ] || fail "ADR 编号重复：$(printf '%s' "$adr_dups" | tr '\n' ' ')"

for adr in specs/decisions/D-[0-9][0-9][0-9]-*.md; do
	[ -f "$adr" ] || continue
	grep -Eq '^状态：`(proposed|accepted|rejected|deprecated|superseded by D-[0-9]{3})`' "$adr" ||
		fail "$adr 的状态不合法"
	grep -qF "$(basename "$adr")" specs/decisions/README.md || fail "$adr 没登记在 decisions/README.md"
	superseded="$(sed -n 's/^状态：`superseded by \(D-[0-9][0-9][0-9]\)`.*/\1/p' "$adr")"
	if [ -n "$superseded" ] && ! find specs/decisions -maxdepth 1 -type f -name "${superseded}-*.md" | grep -q .; then
		fail "$adr 指向不存在的 $superseded"
	elif [ -n "$superseded" ]; then
		target="$(find specs/decisions -maxdepth 1 -type f -name "${superseded}-*.md" | head -1)"
		current="$(basename "$adr" | cut -d- -f1-2)"
		grep -q "$current" "$target" || fail "$target 没有反向链接 $current"
	fi
done

for release in specs/releases/v*/README.md; do
	[ -f "$release" ] || continue
	release_name="$(basename "$(dirname "$release")")"
	tag="$(sed -n 's/^Git tag：`\([^`]*\)`.*/\1/p' "$release")"
	release_commit="$(sed -n 's/^Release commit：`\([0-9a-f]*\)`.*/\1/p' "$release")"
	if [ -z "$tag" ]; then
		fail "$release 缺 Git tag"
	elif ! git rev-parse --verify --quiet "refs/tags/$tag" >/dev/null; then
		fail "$(missing_history_diag "tag $tag")；$release 记着它"
	elif [ -n "$release_commit" ] && [ "$(git rev-list -n 1 "$tag")" != "$release_commit" ]; then
		fail "$release 的 Release commit 与 tag $tag 不一致"
	fi
	grep -qF "${release_name}/README.md" specs/releases/README.md || fail "$release 没登记在 releases/README.md"
	grep -Eq '^Release commit：`[0-9a-f]{40}`' "$release" || fail "$release 缺 Release commit"
	grep -Eq '^.+闭包：`[0-9a-f]{40}`' "$release" || fail "$release 缺验收闭包 commit"
	grep -q '^## 验收' "$release" || fail "$release 缺验收证据段"
	grep -q '^## 已知限制' "$release" || fail "$release 缺已知限制段"
	awk '
		/^## 验收/ { inside = 1; next }
		/^## / { inside = 0 }
		inside { print }
	' "$release" | grep -q '](' || fail "$release 的验收段没有证据链接"
	while IFS= read -r commit; do
		[ -n "$commit" ] || continue
		git cat-file -e "${commit}^{commit}" 2>/dev/null ||
			fail "$(missing_history_diag "commit $commit")；$release 提到它"
	done < <(grep -E 'commit|闭包' "$release" | grep -oE '[0-9a-f]{40}' || true)
done

if grep -nE '当前阶段|下一步(是|为).*(T[0-9]|C[0-9])|\b(todo|doing|blocked|done)\b' AGENTS.md; then
	fail "AGENTS.md 含易过期的当前阶段、任务编号或任务状态"
fi

grep -qF 'scripts/check-specs\.sh' .pre-commit-config.yaml ||
	fail "pre-commit 的 check-specs 不会在检查器自身变化时触发"
grep -q 'run: scripts/check-specs.sh' .github/workflows/build.yml || fail "CI 没运行 check-specs.sh"

version="$(awk '
	/^\[workspace\.package\]/ { inside = 1; next }
	/^\[/ { inside = 0 }
	inside && /^version = / { gsub(/"/, "", $3); print $3; exit }
' Cargo.toml)"
if [ -z "$version" ]; then
	fail "Cargo.toml 取不到 workspace version"
fi

# 已发布 release 与开发中的 active target/RC 分开验证（C002-T15）：有 release record
# 的版本按已发布核 tag 与 commit；开发中的版本等于 active 目标版本或其 RC 就行，
# 不要求已有 tag（发布还没做，不能逼着造 tag），CHANGELOG 允许先写 [Unreleased]。
current_release="specs/releases/v${version}/README.md"
base_version="${version%%-*}"
active_target="$(sed -n 's/^目标版本：`v\([^`]*\)`.*/\1/p' specs/changes/active/C*/README.md 2>/dev/null | head -1)"
if [ -f "$current_release" ]; then
	if ! grep -q "^## \[$version\]" CHANGELOG.md; then
		fail "CHANGELOG.md 没有版本 $version"
	fi
	if ! grep -q "^Git tag：\`v${version}\`" "$current_release"; then
		fail "$current_release 的 tag 与 Cargo 版本 $version 不一致"
	elif ! git rev-parse --verify --quiet "refs/tags/v${version}" >/dev/null; then
		fail "$(missing_history_diag "tag v${version}")；$current_release 记着它"
	fi
else
	dev_ok=""
	if [ -n "$active_target" ] && [ "$base_version" = "$active_target" ]; then
		case "$version" in
		"$active_target" | "$active_target"-rc.*) dev_ok=1 ;;
		esac
	fi
	if [ -z "$dev_ok" ]; then
		fail "当前 Cargo 版本 $version 既没有 release record，也不是 active 目标版本/RC（${active_target:-无 active}）"
	elif ! grep -q "^## \[$version\]" CHANGELOG.md && ! grep -q '^## \[Unreleased\]' CHANGELOG.md; then
		fail "CHANGELOG.md 既没有版本 $version 也没有 [Unreleased] 段"
	fi
fi

if [ "$status" -eq 0 ]; then
	echo "check-specs: OK ($(printf '%s' "$ids" | sed '/^$/d' | wc -l | tr -d ' ') 个 change，$active_count 个 active)"
fi
exit "$status"
