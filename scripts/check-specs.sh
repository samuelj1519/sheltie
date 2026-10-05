#!/usr/bin/env bash
# Specification governance: change lifecycle, ADRs, releases, and agent entry points.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

status=0
qualification_root="$(mktemp -d "${TMPDIR:-/tmp}/sheltie-specs.XXXXXX")"
trap 'rm -rf "$qualification_root"' EXIT
fail() {
	echo "check-specs: $*" >&2
	status=1
}

# Distinguish missing history from missing objects: shallow or incomplete checkouts
# may omit old tags. Diagnose incomplete history explicitly instead of blaming the record.
missing_history_diag() {
	if [ "$(git rev-parse --is-shallow-repository 2>/dev/null || echo false)" = "true" ]; then
		echo "incomplete history: shallow checkout cannot resolve $1; governance requires fetch-depth: 0 and complete history/tags"
	else
		echo "$1 is absent from this Git history (not created or not pushed)"
	fi
}

for path in \
	specs/changes/README.md \
	specs/changes/proposed \
	specs/changes/active \
	specs/changes/completed \
	specs/changes/rejected \
	docs/en/explanation/decisions/README.md \
	docs/en/reference/releases/README.md \
	docs/en/reference/releases/v0.1.0/README.md \
	docs/en/explanation/decisions/mvp.md \
	docs/en/history/changes/README.md; do
	[ -e "$path" ] || fail "missing $path"
done

for legacy_root in specs/plan.md specs/decisions.md specs/t25-t26-runbook.md tasks.toml; do
	[ ! -e "$legacy_root" ] || fail "MVP history remains at the top level: $legacy_root"
done

outdated_release_refs="$(grep -RInE 'releases/(v[0-9]+\.[0-9]+\.[0-9]+|<version>)\.md' specs docs --include='*.md' || true)"
if [ -n "$outdated_release_refs" ]; then
	printf '%s\n' "$outdated_release_refs" >&2
	fail "documentation still references obsolete release-record paths"
fi

packages="$(find specs/changes/proposed specs/changes/active specs/changes/completed specs/changes/rejected docs/en/history/changes \
	-mindepth 1 -maxdepth 1 -type d -name 'C*' | sort)"

ids=""
while IFS= read -r package; do
	[ -n "$package" ] || continue
	name="$(basename "$package")"
	state="$(basename "$(dirname "$package")")"
	index="specs/changes/README.md"
	case "$package" in
	docs/en/history/changes/*) state=completed; index="docs/en/history/changes/README.md" ;;
	esac
	case "$name" in
	C[0-9][0-9][0-9]-*) ;;
	*) fail "$package directory must be named Cnnn-<slug>" ;;
	esac
	[ -f "$package/README.md" ] || {
		fail "$package is missing README.md"
		continue
	}
	grep -q "^Status: \`$state\`" "$package/README.md" ||
		fail "$package/README.md status does not match directory $state"
	for field in "Target version" Compatibility Baseline Owner; do
		head -15 "$package/README.md" | grep -q "^${field}: " || fail "$package/README.md first screen is missing ${field}"
	done
	grep -qF "$package/README.md" "$index" ||
		grep -qF "$(basename "$package")/README.md" "$index" ||
		grep -qF "${package#specs/changes/}/README.md" "$index" ||
		fail "$package is not listed in $index"
	ids="${ids}${name%%-*}
"
done <<<"$packages"

duplicates="$(printf '%s' "$ids" | sed '/^$/d' | sort | uniq -d)"
[ -z "$duplicates" ] || fail "duplicate change IDs: $(printf '%s' "$duplicates" | tr '\n' ' ')"

active_count="$(find specs/changes/active -mindepth 1 -maxdepth 1 -type d -name 'C*' | wc -l | tr -d ' ')"
[ "$active_count" -le 1 ] || fail "$active_count active changes; the default limit is 1"

for package in specs/changes/active/C*; do
	[ -d "$package" ] || continue
	for file in README.md plan.md progress.md validation.md tasks.toml; do
		[ -f "$package/$file" ] || fail "$package is missing required active file $file"
	done
	grep -q '^## Success criteria' "$package/README.md" || fail "$package/README.md is missing success criteria"
done

if [ "$active_count" -eq 0 ]; then
	grep -q '^Active change: none$' specs/changes/README.md || fail "change index must declare that no change is active"
else
	active_name="$(find specs/changes/active -mindepth 1 -maxdepth 1 -type d -name 'C*' -exec basename {} \;)"
	grep -q "$active_name" specs/changes/README.md || fail "change index does not link active package $active_name"
fi

for package in specs/changes/completed/C* docs/en/history/changes/C*; do
	[ -d "$package" ] || continue
	qualification="$package"
	source_package="$package"
	case "$package" in
	docs/en/history/changes/*) source_package="specs/changes/completed/$(basename "$package")" ;;
	esac
	if grep -q '^Record form: `reference`$' "$package/README.md"; then
		for section in "Changes and rationale" "Validation and limits" References; do
			grep -q "^## $section$" "$package/README.md" || fail "$package/README.md is missing reference section $section"
		done
		snapshot="$(sed -n 's/^Historical snapshot: `\([0-9a-f]\{40\}\)`$/\1/p' "$package/README.md")"
		if [ -z "$snapshot" ] || ! git cat-file -e "${snapshot}^{commit}" 2>/dev/null; then
			fail "$(missing_history_diag "reference snapshot ${snapshot:-unspecified}"); $package cannot verify original completion records"
			continue
		fi
		qualification="$qualification_root/$(basename "$package")"
		mkdir -p "$qualification"
		for file in README.md plan.md validation.md review.md tasks.toml; do
			git show "$snapshot:$source_package/$file" > "$qualification/$file" 2>/dev/null || fail "$package snapshot is missing $file"
		done
		grep -Eq '^(Status: |状态：)`completed`' "$qualification/README.md" || fail "$package snapshot is not completed"
		if grep -Eq '^(Record form: |记录形式：)`reference`$' "$qualification/README.md"; then
			fail "$package snapshot must point to complete qualification records, not another summary"
			continue
		fi
	fi
	for file in README.md plan.md validation.md review.md tasks.toml; do
		[ -f "$qualification/$file" ] || fail "$package is missing required completed file $file"
	done
	if [ -f "$qualification/plan.md" ]; then
		unfinished="$(awk -F'|' '
			/^\| C[0-9][0-9][0-9]-(T[0-9][0-9]|M[0-9]+) / {
				state = $3; gsub(/^[[:space:]]+|[[:space:]]+$/, "", state)
				if (state != "done") print $0
			}
		' "$qualification/plan.md")"
		[ -z "$unfinished" ] || fail "$package/plan.md has unfinished tasks"
	fi
	if [ -f "$qualification/validation.md" ]; then
		grep -Eq '^Candidate: `(SELF|[0-9a-f]{7,40})`' "$qualification/validation.md" ||
			fail "$package/validation.md is missing a fixed candidate"
		# Preserve original historical results; check only the final validation table (C002-T17).
		validation_errors="$(awk -F'|' '
			function trim(value) {
				gsub(/^[[:space:]]+|[[:space:]]+$/, "", value)
				return value
			}
			function six_columns() {
				return NF == 8 && trim($1) == "" && trim($8) == ""
			}
			function final_header() {
				return six_columns() && trim($2) == "Requirement / risk" &&
					trim($3) == "Mode" && trim($4) == "Input closure" &&
					trim($5) == "Command / raw run ID" && trim($6) == "Result" &&
					trim($7) == "Evidence"
			}
			function separator( column) {
				if (!six_columns()) return 0
				for (column = 2; column <= 7; column++)
					if (trim($column) !~ /^:?-+:?$/) return 0
				return 1
			}
			function finish_table() {
				if (inside && rows == 0) print "final validation table has no data rows"
				inside = 0
			}
			/^\|/ {
				if (final_header()) {
					finish_table()
					inside = 1; tables++; rows = 0
					next
				}
				if (!inside || separator()) next
				rows++
				if (!six_columns()) {
					print "final validation row must have six columns: " $0
					next
				}
				empty_column = 0
				for (column = 2; column <= 7; column++) {
					if (trim($column) == "") { empty_column = column - 1; break }
				}
				if (empty_column) {
					print "final validation row column " empty_column " is empty: " $0
					next
				}
				if (trim($6) != "PASS") print "non-PASS validation entry: " $0
				next
			}
			{ finish_table() }
			END {
				finish_table()
				if (tables == 0) print "missing final validation table"
			}
		' "$qualification/validation.md")"
		[ -z "$validation_errors" ] || fail "$package/validation.md final validation table is invalid: $validation_errors"
	fi
	if [ -f "$qualification/review.md" ]; then
		grep -Eq '^(Conclusion: |结论：)`PASS`' "$qualification/review.md" || fail "$package/review.md has no final PASS"
	fi
done

for package in specs/changes/rejected/C*; do
	[ -d "$package" ] || continue
	grep -q '^## Rejection reasons' "$package/README.md" || fail "$package/README.md is missing rejection reasons"
	grep -q '^## Alternatives' "$package/README.md" || fail "$package/README.md is missing alternatives"
	awk '
		/^## Alternatives/ { inside = 1; next }
		/^## / { inside = 0 }
		inside { print }
	' "$package/README.md" | grep -Eq '\]\(|^none[.]?$' || fail "$package/README.md alternatives must link a replacement or state none"
done

adr_dups="$(find docs/en/explanation/decisions -maxdepth 1 -type f -name 'D-[0-9][0-9][0-9]-*.md' ! -name '*.zh-CN.md' \
	-exec basename {} \; | cut -d- -f1-2 | sort | uniq -d)"
[ -z "$adr_dups" ] || fail "duplicate ADR IDs: $(printf '%s' "$adr_dups" | tr '\n' ' ')"

for adr in docs/en/explanation/decisions/D-[0-9][0-9][0-9]-*.md; do
	case "$adr" in *.zh-CN.md) continue ;; esac
	[ -f "$adr" ] || continue
	grep -Eq '^Status: `(proposed|accepted|rejected|deprecated|superseded by D-[0-9]{3})`' "$adr" ||
		fail "$adr has an invalid status"
	grep -qF "$(basename "$adr")" docs/en/explanation/decisions/README.md || fail "$adr is not listed in decisions/README.md"
	superseded="$(sed -n 's/^Status: `superseded by \(D-[0-9][0-9][0-9]\)`.*/\1/p' "$adr")"
	if [ -n "$superseded" ] && ! find docs/en/explanation/decisions -maxdepth 1 -type f -name "${superseded}-*.md" ! -name "*.zh-CN.md" | grep -q .; then
		fail "$adr references nonexistent $superseded"
	elif [ -n "$superseded" ]; then
		target="$(find docs/en/explanation/decisions -maxdepth 1 -type f -name "${superseded}-*.md" ! -name "*.zh-CN.md" | head -1)"
		current="$(basename "$adr" | cut -d- -f1-2)"
		grep -q "$current" "$target" || fail "$target does not link back to $current"
	fi
done

for release in docs/en/reference/releases/v*/README.md; do
	[ -f "$release" ] || continue
	release_name="$(basename "$(dirname "$release")")"
	tag="$(sed -n 's/^Git tag: `\([^`]*\)`.*/\1/p' "$release")"
	release_commit="$(sed -n 's/^Release commit: `\([0-9a-f]*\)`.*/\1/p' "$release")"
	if [ -z "$tag" ]; then
		fail "$release is missing Git tag"
	elif ! git rev-parse --verify --quiet "refs/tags/$tag" >/dev/null; then
		fail "$(missing_history_diag "tag $tag"); it is recorded in $release"
	elif [ -n "$release_commit" ] && [ "$(git rev-list -n 1 "$tag")" != "$release_commit" ]; then
		fail "$release Release commit does not match tag $tag"
	fi
	grep -qF "${release_name}/README.md" docs/en/reference/releases/README.md || fail "$release is not listed in releases/README.md"
	grep -Eq '^Release commit: `[0-9a-f]{40}`' "$release" || fail "$release is missing Release commit"
	grep -Eq '^.+closure: `[0-9a-f]{40}`' "$release" || fail "$release is missing an acceptance-closure commit"
	grep -q '^## Acceptance' "$release" || fail "$release is missing acceptance evidence"
	grep -q '^## Known limits' "$release" || fail "$release is missing known limits"
	awk '
		/^## Acceptance/ { inside = 1; next }
		/^## / { inside = 0 }
		inside { print }
	' "$release" | grep -q '](' || fail "$release acceptance section has no evidence links"
	# Original published IDs describe upstream evidence, not mapped local object availability.
	while IFS= read -r commit; do
		[ -n "$commit" ] || continue
		git cat-file -e "${commit}^{commit}" 2>/dev/null ||
			fail "$(missing_history_diag "commit $commit"); it is referenced in $release"
	done < <(grep -E 'commit|closure' "$release" | grep -v '^Original published commit:' | grep -oE '[0-9a-f]{40}' || true)
done

if grep -nE 'Current phase|Next.*(T[0-9]|C[0-9])|\b(todo|doing|blocked|done)\b' AGENTS.md; then
	fail "AGENTS.md contains transient phases, task IDs, or task states"
fi

grep -qF 'scripts/check-specs\.sh' .pre-commit-config.yaml ||
	fail "pre-commit does not trigger check-specs when the checker itself changes"
grep -q 'run: scripts/check-specs.sh' .github/workflows/build.yml || fail "CI does not run check-specs.sh"

version="$(awk '
	/^\[workspace\.package\]/ { inside = 1; next }
	/^\[/ { inside = 0 }
	inside && /^version = / { gsub(/"/, "", $3); print $3; exit }
' Cargo.toml)"
if [ -z "$version" ]; then
	fail "cannot read the workspace version from Cargo.toml"
fi

# The index defines the development target; active plans define progress, not version authority.
current_release="docs/en/reference/releases/v${version}/README.md"
development_target=""
authority_valid=""
base_pattern='(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)'
if [ ! -f specs/README.md ]; then
	fail "missing development-target authority specs/README.md"
else
	field_count="$(grep -c '^Development target: ' specs/README.md || true)"
	if [ "$field_count" -ne 1 ]; then
		fail "specs/README.md must have exactly one Development target field; found $field_count"
	else
		development_target="$(head -15 specs/README.md | sed -n 's/^Development target: `v\([^`]*\)`$/\1/p')"
		if [[ "$development_target" =~ ^${base_pattern}$ ]]; then
			authority_valid=1
		else
			fail "specs/README.md first screen requires a canonical base version: Development target: \`vX.Y.Z\`"
		fi
	fi
fi
active_target=""
active_declaration_valid=1
if [ "$active_count" -gt 0 ]; then
	active_value="$(sed -n 's/^Target version: `\([^`]*\)`.*/\1/p' specs/changes/active/C*/README.md)"
	if [[ "$active_value" =~ ^v${base_pattern}$ ]]; then
		active_target="${active_value#v}"
		if [ -n "$authority_valid" ] && [ "$active_target" != "$development_target" ]; then
			fail "active target $active_target differs from Development target $development_target"
			active_declaration_valid=""
		fi
	elif [ "$active_value" != "not included in a product release" ]; then
		fail "invalid active target: use a numeric version or explicitly exclude product release"
		active_declaration_valid=""
	fi
fi
# Released versions still verify release/tag/CHANGELOG; candidates do not require release records/tags.
if [ -f "$current_release" ]; then
	if ! grep -q "^## \[$version\]" CHANGELOG.md; then
		fail "CHANGELOG.md is missing version $version"
	fi
	if ! grep -q "^Git tag: \`v${version}\`" "$current_release"; then
		fail "$current_release tag does not match Cargo version $version"
	elif ! git rev-parse --verify --quiet "refs/tags/v${version}" >/dev/null; then
		fail "$(missing_history_diag "tag v${version}"); it is recorded in $current_release"
	fi
else
	if [ -n "$authority_valid" ] && [ -n "$active_declaration_valid" ]; then
		dev_ok=""
		case "$version" in
		"$development_target" | "$development_target"-rc.*) dev_ok=1 ;;
		esac
		if [ -z "$dev_ok" ]; then
			fail "Cargo version $version has no release record and matches neither Development target $development_target nor active target/RC (${active_target:-non-product or no active change})"
		elif ! grep -q "^## \[$version\]" CHANGELOG.md && ! grep -q '^## \[Unreleased\]' CHANGELOG.md; then
			fail "CHANGELOG.md has neither version $version nor an [Unreleased] section"
		fi
	fi
fi

if [ "$status" -eq 0 ]; then
	echo "check-specs: OK ($(printf '%s' "$ids" | sed '/^$/d' | wc -l | tr -d ' ') changes, $active_count active)"
fi
exit "$status"
