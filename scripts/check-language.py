#!/usr/bin/env python3
"""Check English defaults, declared Unicode literals, and reciprocal language links."""

import json
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parent.parent
HAN = re.compile(r"[\u3400-\u9fff\uf900-\ufaff\U00020000-\U0002ffff\U00030000-\U000323af]")
SOURCE_SUFFIXES = {
    ".md", ".rs", ".toml", ".mjs", ".html", ".css", ".sh", ".py", ".yml", ".yaml", ".snap"
}


def main():
    entries = json.loads((ROOT / "scripts/language-exceptions.json").read_text())
    exceptions = {(entry["path"], entry["text"]): entry["purpose"] for entry in entries}
    paths = subprocess.check_output(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"], cwd=ROOT
    ).decode().split("\0")
    errors = []
    seen = set()
    for name in dict.fromkeys(paths):
        if not name:
            continue
        path = ROOT / name
        if not path.is_file() or path.is_symlink() or path.suffix not in SOURCE_SUFFIXES:
            continue
        if name.startswith("docs/zh-CN/"):
            counterpart = ROOT / "docs/en" / path.relative_to(ROOT / "docs/zh-CN")
            if path.suffix == ".md" and not counterpart.is_file():
                errors.append(f"{name}: missing English source document")
            continue
        relative_parts = Path(name).parts
        if (
            relative_parts[0] in {"examples", "workbooks"}
            and len(relative_parts) > 1
            and relative_parts[1].endswith("-zh-CN")
        ):
            continue
        if ".zh-CN." in path.name:
            if name not in {"README.zh-CN.md", "skills/sheltie/SKILL.zh-CN.md"}:
                errors.append(f"{name}: Chinese reader documents belong in docs/zh-CN")
            continue
        text = path.read_text()
        for number, line in enumerate(text.splitlines(), 1):
            if not HAN.search(line.replace("\u7b80\u4f53\u4e2d\u6587", "")):
                continue
            key = (name, line.strip())
            if key in exceptions:
                seen.add(key)
            else:
                errors.append(f"{name}:{number}: undeclared non-English source text")
        if name.startswith("docs/en/") and path.suffix == ".md":
            localized = ROOT / "docs/zh-CN" / path.relative_to(ROOT / "docs/en")
            if not localized.is_file():
                errors.append(f"{name}: missing Chinese reader document")
        else:
            localized = path.with_name(path.stem + ".zh-CN" + path.suffix)
        if path.suffix == ".md" and localized.exists() and path.name != "CLAUDE.md":
            counterpart = localized.read_text()
            english = re.findall(r"\[English\]\(([^)]+)\)", counterpart)
            if len(english) != 1 or (localized.parent / english[0]).resolve() != path:
                errors.append(f"{localized.relative_to(ROOT)}: invalid English selector")
            chinese = re.findall(r"\[\u7b80\u4f53\u4e2d\u6587\]\(([^)]+)\)", text)
            if len(chinese) != 1 or (path.parent / chinese[0]).resolve() != localized:
                errors.append(f"{name}: invalid Chinese selector")
    for key in exceptions.keys() - seen:
        errors.append(f"{key[0]}: stale language exception; review its purpose")
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print(f"check-language: OK ({len(seen)} declared literals; reciprocal links verified)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
