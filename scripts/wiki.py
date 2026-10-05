#!/usr/bin/env python3
"""Validate bilingual docs and export a pinned, deterministic GitHub Wiki snapshot."""

import argparse
import hashlib
import html
import json
from pathlib import Path
import re
import subprocess
import sys
from urllib.parse import quote, unquote, urlsplit


MANIFEST = "wiki-manifest.json"
CHINESE = "\u7b80\u4f53\u4e2d\u6587"


class WikiError(ValueError):
    """A source or generated snapshot violates the publishing requirements."""


def git(repo, *args):
    return subprocess.check_output(["git", "-C", str(repo), *args], text=True).strip()


def clean_head(repo):
    if git(repo, "status", "--porcelain", "--untracked-files=all"):
        raise WikiError("Publication requires a clean source repository")
    return git(repo, "rev-parse", "HEAD")


def digest(data):
    return hashlib.sha256(data).hexdigest()


def blob_bytes(repo, commit, name):
    return subprocess.check_output(["git", "-C", str(repo), "cat-file", "blob", f"{commit}:{name}"])


def prose_lines(text):
    """Yield prose and fences separately, preserving every code block byte."""
    fence = None
    for line in text.splitlines(keepends=True):
        match = re.match(r"^ {0,3}(`{3,}|~{3,})(.*)$", line.rstrip("\r\n"))
        if fence:
            yield line, False
            if match and match[1][0] == fence[0] and len(match[1]) >= fence[1] and not match[2].strip():
                fence = None
        elif match:
            fence = (match[1][0], len(match[1]))
            yield line, False
        else:
            yield line, True
    if fence:
        raise WikiError("Close every fenced code block before publishing")


def anchors(text):
    result, counts = set(), {}
    previous = ""
    for line, prose in prose_lines(text):
        if not prose:
            previous = ""
            continue
        result.update(re.findall(r'<[^>]+\b(?:id|name)=["\']([^"\']+)["\']', line))
        heading = re.match(r"^ {0,3}#{1,6}\s+(.+?)\s*#*\s*$", line)
        value = heading[1] if heading else previous if re.match(r"^ {0,3}(?:=+|-+)\s*$", line) else None
        if value:
            value = re.sub(r"!?\[([^\]]+)\]\([^)]*\)", r"\1", value)
            value = re.sub(r"(`+)(.*?)\1", lambda match: html.escape(match[2], quote=False), value)
            value = html.unescape(re.sub(r"<[^>]*>", "", value)).replace("`", "").replace("*", "")
            value = re.sub(r"(?<!\w)_|_(?!\w)", "", value)
            slug = re.sub(r"[^\w\- ]", "", value.lower()).replace(" ", "-")
            count = counts.get(slug, 0)
            counts[slug] = count + 1
            result.add(slug if not count else f"{slug}-{count}")
        previous = line.strip()
    return result


def destination(line, start):
    """Locate a link destination, excluding its optional title and delimiters."""
    while start < len(line) and line[start].isspace():
        start += 1
    if start == len(line):
        return None
    if line[start] == "<":
        end = start + 1
        while end < len(line):
            if line[end] == "\\":
                end += 2
            elif line[end] == ">":
                return start + 1, end
            else:
                end += 1
        raise WikiError("Unterminated angle-bracket link destination")
    end, depth = start, 0
    while end < len(line):
        char = line[end]
        if char == "\\":
            end += 2
            continue
        if char == "(":
            depth += 1
        elif char == ")":
            if not depth:
                break
            depth -= 1
        elif char.isspace() and not depth:
            break
        end += 1
    return (start, end) if end > start else None


def rewrite_line(line, resolve):
    spans = []
    definition = re.match(r"^ {0,3}\[[^\]]+\]:\s*", line)
    if definition:
        target = destination(line, definition.end())
        if target:
            spans.append(target)
    index = 0
    while index < len(line):
        existing = next((end for start, end in spans if start <= index < end), None)
        if existing:
            index = existing
            continue
        char = line[index]
        if char == "\\":
            index += 2
            continue
        if char == "`":
            run = len(re.match(r"`+", line[index:])[0])
            close = line.find("`" * run, index + run)
            index = index + run if close < 0 else close + run
            continue
        if char == "[":
            close, depth = index + 1, 1
            while close < len(line) and depth:
                if line[close] == "\\":
                    close += 2
                    continue
                depth += (line[close] == "[") - (line[close] == "]")
                close += 1
            if not depth and close < len(line) and line[close] == "(":
                target = destination(line, close + 1)
                if target:
                    spans.append(target)
        index += 1
    for start, end in sorted(set(spans), reverse=True):
        line = line[:start] + resolve(line[start:end]) + line[end:]
    return line


def page_id(language, relative):
    if relative == Path("README.md"):
        return "Home" if language == "en" else "Zh-CN-Home"
    name = "-".join(relative.with_suffix("").parts)
    if not re.fullmatch(r"[A-Za-z0-9_.-]+", name):
        raise WikiError(f"Page path requires an ASCII filename: {relative}")
    name = name[0].upper() + name[1:]
    return name if language == "en" else "Zh-CN-" + name


class Snapshot:
    def __init__(self, repo, repository, commit):
        self.repo = repo.resolve()
        self.docs = self.repo / "docs"
        if self.docs.is_symlink():
            raise WikiError("Symlinks are forbidden in docs")
        if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repository):
            raise WikiError("Repository must have the form OWNER/REPOSITORY")
        self.repository, self.commit = repository, commit
        self.committed_files = None
        self.pinned_bytes = {}
        self.wiki = f"https://github.com/{repository}/wiki/"
        self.raw = f"https://raw.githubusercontent.com/wiki/{repository}/"
        self.pages, self.assets, self.sources, self.headings = {}, {}, {}, {}
        identifiers = {"_sidebar", "wiki-manifest"}
        for language in ("en", "zh-CN"):
            directory = self.docs / language
            if not directory.is_dir() or directory.is_symlink():
                raise WikiError(f"Missing ordinary language directory: {directory}")
            for path in sorted(directory.rglob("*")):
                if path.is_symlink():
                    raise WikiError(f"Symlinks are forbidden in docs: {path}")
                if not path.is_file():
                    continue
                self.sources[path] = path.read_bytes()
                if path.suffix == ".md":
                    name = page_id(language, path.relative_to(directory))
                    if name.casefold() in identifiers:
                        raise WikiError(f"Wiki page ID collision: {name}")
                    identifiers.add(name.casefold())
                    self.pages[path] = name
                else:
                    self.assets[path] = "assets/" + path.relative_to(self.repo).as_posix()
        for path in self.pages:
            relative = path.relative_to(self.docs)
            partner_language = "zh-CN" if relative.parts[0] == "en" else "en"
            partner = self.docs / partner_language / Path(*relative.parts[1:])
            if partner not in self.pages:
                raise WikiError(f"Missing language partner: {partner.relative_to(self.repo)}")
        if self.docs / "en/README.md" not in self.pages:
            raise WikiError("Both language home pages are required")

    def source_url(self, path, fragment="", query=""):
        kind = "tree" if path.is_dir() else "blob"
        relative = "" if path == self.repo else path.relative_to(self.repo).as_posix()
        if self.committed_files is not None:
            if not relative:
                present = bool(self.committed_files)
                kind = "tree"
            elif relative in self.committed_files:
                present, kind = True, "blob"
            else:
                present = any(name.startswith(relative + "/") for name in self.committed_files)
                kind = "tree"
            if not present:
                raise WikiError(f"Source link is absent from the pinned commit: {relative}")
        url = f"https://github.com/{self.repository}/{kind}/{self.commit}/" + quote(relative, safe="/")
        return url + ("?" + query if query else "") + ("#" + quote(fragment, safe="-_") if fragment else "")

    def resolve(self, source, target):
        target = re.sub(r"\\(.)", r"\1", target)
        url = urlsplit(target)
        if url.scheme or url.netloc:
            if url.scheme not in ("", "http", "https", "mailto"):
                raise WikiError(f"{source.relative_to(self.repo)}: unsupported link scheme: {target}")
            return target
        path = source if not url.path else source.parent / unquote(url.path)
        if path.is_absolute() and url.path.startswith("/"):
            raise WikiError(f"{source.relative_to(self.repo)}: absolute local link: {target}")
        try:
            relative = path.absolute().relative_to(self.repo)
        except ValueError as error:
            raise WikiError(f"Link escapes repository: {target}") from error
        current = self.repo
        for part in relative.parts:
            current = current / part
            if current.is_symlink():
                raise WikiError(f"Symlink link target: {target}")
        path = path.resolve()
        if not path.is_relative_to(self.repo) or not path.exists():
            raise WikiError(f"{source.relative_to(self.repo)}: missing or escaping target: {target}")
        if path.is_dir() and path.is_relative_to(self.docs):
            path = path / "README.md"
        if path == self.docs / "README.md":
            path = self.docs / "en/README.md"
        fragment = unquote(url.fragment)
        if fragment and path.suffix == ".md":
            if path not in self.headings:
                if self.committed_files is not None:
                    relative = path.relative_to(self.repo).as_posix()
                    if relative not in self.committed_files:
                        raise WikiError(f"Source link is absent from the pinned commit: {relative}")
                    if path not in self.pinned_bytes:
                        self.pinned_bytes[path] = blob_bytes(self.repo, self.commit, relative)
                    content = self.pinned_bytes[path]
                else:
                    content = self.sources[path] if path in self.sources else path.read_bytes()
                self.headings[path] = anchors(content.decode())
            if fragment not in self.headings[path] and not (path not in self.pages and re.fullmatch(r"L\d+(?:-L\d+)?", fragment)):
                raise WikiError(f"{source.relative_to(self.repo)}: missing anchor: {target}")
        if path in self.pages:
            value = self.wiki + self.pages[path]
            return value + ("?" + url.query if url.query else "") + ("#" + quote(fragment, safe="-_") if fragment else "")
        if path in self.assets:
            if url.query or fragment:
                raise WikiError(f"Asset links cannot contain queries or fragments: {target}")
            return self.raw + quote(self.assets[path], safe="/")
        if path.is_relative_to(self.docs):
            raise WikiError(f"Docs target is outside the bilingual source trees: {target}")
        return self.source_url(path, fragment, url.query)

    def render(self, path):
        text = self.sources[path].decode()
        relative = path.relative_to(self.docs)
        chinese = relative.parts[0] == "zh-CN"
        partner = self.docs / ("en" if chinese else "zh-CN") / Path(*relative.parts[1:])
        selector = f"[English]({self.wiki}{self.pages[partner]}) | {CHINESE}" if chinese else f"English | [{CHINESE}]({self.wiki}{self.pages[partner]})"
        lines = []
        inserted = False
        for line, prose in prose_lines(text):
            if prose and re.fullmatch(r"(?:English|\[English\]\([^)]*\))\s*\|\s*(?:" + CHINESE + r"|\[" + CHINESE + r"\]\([^)]*\))", line.strip()):
                continue
            if prose and re.search(r"<(?:img|a)\b[^>]*\b(?:src|href)=", line, re.I):
                raise WikiError(f"Use Markdown links instead of HTML src/href in {path.relative_to(self.repo)}")
            lines.append(rewrite_line(line, lambda target: self.resolve(path, target)) if prose else line)
            if not inserted and prose and re.match(r"^#\s", line):
                lines.append("\n" + selector + "\n")
                inserted = True
        if not inserted:
            lines.insert(0, selector + "\n\n")
        body = "".join(lines)
        return (body + ("" if body.endswith("\n") else "\n") + f"\n---\n\n[Source]({self.source_url(path)})\n").encode()

    def files(self):
        files = {name + ".md": self.render(path) for path, name in self.pages.items()}
        files.update({name: self.sources[path] for path, name in self.assets.items()})
        sidebar = ["[English](" + self.wiki + "Home) | [" + CHINESE + "](" + self.wiki + "Zh-CN-Home)\n"]
        for language, label in (("en", "English"), ("zh-CN", CHINESE)):
            sidebar.append("\n### " + label + "\n\n")
            for path, name in self.pages.items():
                relative = path.relative_to(self.docs / language) if path.is_relative_to(self.docs / language) else None
                if relative and len(relative.parts) == 2 and relative.name == "README.md":
                    title = next((line.removeprefix("# ").strip() for line in self.sources[path].decode().splitlines() if line.startswith("# ")), relative.parts[0])
                    sidebar.append(f"- [{title}]({self.wiki}{name})\n")
        files["_Sidebar.md"] = "".join(sidebar).encode()
        mapping = [{"source": path.relative_to(self.repo).as_posix(), "sha256": digest(data), "output": self.pages[path] + ".md" if path in self.pages else self.assets[path]} for path, data in sorted(self.sources.items())]
        manifest = {"format": "sheltie-wiki/v1", "repository": self.repository, "source_commit": self.commit, "sources": mapping, "files": {name: digest(data) for name, data in sorted(files.items())}}
        files[MANIFEST] = (json.dumps(manifest, indent=2, ensure_ascii=True, sort_keys=True) + "\n").encode()
        return files


def build(repo, repository, publication=False):
    commit = clean_head(repo) if publication else git(repo, "rev-parse", "HEAD")
    snapshot = Snapshot(repo, repository, commit)
    if publication:
        tracked = set(git(repo, "ls-tree", "-r", "--name-only", "-z", commit).split("\0"))
        snapshot.committed_files = tracked
        pinned_docs = {name for name in tracked if name.startswith(("docs/en/", "docs/zh-CN/"))}
        source_names = {path.relative_to(snapshot.repo).as_posix() for path in snapshot.sources}
        missing = sorted(pinned_docs - source_names)
        if missing:
            raise WikiError("Missing pinned docs: " + ", ".join(missing))
        unpublished = sorted(source_names - pinned_docs)
        if unpublished:
            raise WikiError("Publication includes uncommitted docs: " + ", ".join(unpublished))
        for path, content in snapshot.sources.items():
            name = path.relative_to(snapshot.repo).as_posix()
            pinned = blob_bytes(repo, commit, name)
            if content != pinned:
                raise WikiError(f"Docs bytes differ from the pinned commit: {name}")
            snapshot.pinned_bytes[path] = pinned
    files = snapshot.files()
    if publication and clean_head(repo) != commit:
        raise WikiError("Source commit changed while generating the snapshot")
    return files


def generate(repo, repository, output):
    output = output.absolute()
    if output.exists() or output.is_symlink():
        raise WikiError("Output directory must not exist; use a fresh staging path")
    repo, output = repo.resolve(), output.resolve()
    if output.is_relative_to(repo / "docs"):
        raise WikiError("Output must be outside authored docs")
    if output.is_relative_to(repo):
        probe = output.relative_to(repo).as_posix() + "/" + MANIFEST
        ignored = subprocess.run(["git", "-C", str(repo), "check-ignore", "-q", probe], check=False)
        if ignored.returncode != 0:
            raise WikiError("Output inside the source repository must be Git-ignored")
    files = build(repo, repository, publication=True)
    output.mkdir(parents=True, exist_ok=False)
    for name, data in files.items():
        path = output / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
    return len(files)


def verify(repo, repository, output):
    expected = build(repo, repository, publication=True)
    actual = {}
    if not output.is_dir() or output.is_symlink():
        raise WikiError("Snapshot must be an ordinary directory")
    for path in output.rglob("*"):
        if path.is_symlink():
            raise WikiError(f"Snapshot contains a symlink: {path}")
        if path.is_file():
            actual[path.relative_to(output).as_posix()] = path.read_bytes()
    if actual != expected:
        names = sorted(name for name in actual.keys() | expected.keys() if actual.get(name) != expected.get(name))
        raise WikiError("Snapshot differs from pinned source: " + ", ".join(names))
    return len(actual)


def managed_path(checkout, name):
    if not isinstance(name, str) or not name or "\\" in name or any(ord(char) < 32 for char in name):
        raise WikiError(f"Invalid managed path: {name!r}")
    parts = name.split("/")
    if any(part in ("", ".", "..") or part.casefold() == ".git" for part in parts):
        raise WikiError(f"Invalid managed path: {name!r}")
    path = checkout
    for index, part in enumerate(parts):
        path = path / part
        if path.is_symlink():
            raise WikiError(f"Managed path contains a symlink: {name}")
        if path.exists() and (index < len(parts) - 1 and not path.is_dir() or index == len(parts) - 1 and not path.is_file()):
            raise WikiError(f"Managed path has the wrong file type: {name}")
        if path.is_file() and path.stat().st_nlink > 1:
            raise WikiError(f"Managed path contains a hardlink: {name}")
    return path


def stage(repo, repository, output, checkout):
    verify(repo, repository, output)
    repo, output = repo.resolve(), output.resolve()
    if checkout.is_symlink() or not checkout.is_dir():
        raise WikiError("Wiki checkout must be an ordinary directory")
    checkout = checkout.resolve()
    if checkout == repo or repo.is_relative_to(checkout) or checkout.is_relative_to(output) or output.is_relative_to(checkout):
        raise WikiError("Wiki checkout must not overlap source or export staging")
    source_git = Path(git(repo, "rev-parse", "--path-format=absolute", "--git-common-dir")).resolve()
    if checkout.is_relative_to(source_git) or source_git.is_relative_to(checkout):
        raise WikiError("Wiki checkout must not overlap source Git metadata")
    docs = repo / "docs"
    if checkout.is_relative_to(docs) or docs.is_relative_to(checkout):
        raise WikiError("Wiki checkout must not overlap authored docs")
    if checkout.is_relative_to(repo):
        probe = checkout.relative_to(repo).as_posix() + "/" + MANIFEST
        ignored = subprocess.run(["git", "-C", str(repo), "check-ignore", "-q", probe], check=False)
        if ignored.returncode != 0:
            raise WikiError("Wiki checkout inside the source repository must be Git-ignored")
    if Path(git(checkout, "rev-parse", "--show-toplevel")).resolve() != checkout:
        raise WikiError("Wiki checkout must be its actual Git root")
    wiki_git = Path(git(checkout, "rev-parse", "--path-format=absolute", "--git-common-dir")).resolve()
    if wiki_git.is_relative_to(source_git) or source_git.is_relative_to(wiki_git):
        raise WikiError("Wiki and source Git metadata directories must not overlap")
    expected_origins = {f"https://github.com/{repository}.wiki.git", f"git@github.com:{repository}.wiki.git", f"ssh://git@github.com/{repository}.wiki.git"}
    if git(checkout, "remote", "get-url", "origin") not in expected_origins:
        raise WikiError("Wiki origin must match the source repository's .wiki.git")
    clean_head(checkout)
    expected = build(repo, repository, publication=True)
    manifest_path = managed_path(checkout, MANIFEST)
    previous = {}
    if manifest_path.exists():
        try:
            manifest = json.loads(manifest_path.read_bytes())
            if manifest["format"] != "sheltie-wiki/v1" or manifest["repository"] != repository or not isinstance(manifest["files"], dict):
                raise WikiError("Previous Wiki manifest has an incompatible format or repository")
            previous = manifest["files"]
        except (ValueError, KeyError, TypeError) as error:
            raise WikiError(f"Invalid previous Wiki manifest: {error}") from error
        for name, checksum in previous.items():
            path = managed_path(checkout, name)
            if not isinstance(checksum, str) or not re.fullmatch(r"[0-9a-f]{64}", checksum):
                raise WikiError(f"Invalid previous manifest checksum: {name}")
            if not path.is_file() or digest(path.read_bytes()) != checksum:
                raise WikiError(f"Previously managed Wiki file changed without its manifest: {name}")
    managed = sorted(expected.keys() | previous.keys())
    paths = {name: managed_path(checkout, name) for name in managed}
    writes = {}
    for name, data in expected.items():
        path = paths[name]
        if path.exists():
            if path.read_bytes() == data:
                continue
            if name not in previous and name != MANIFEST:
                raise WikiError(f"Generated page would overwrite an unowned Wiki file: {name}")
        writes[name] = data
    for name, data in writes.items():
        path = paths[name]
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
    for name in previous.keys() - expected.keys():
        paths[name].unlink()
    subprocess.run(["git", "-C", str(checkout), "--literal-pathspecs", "add", "--force", "-A", "--", *managed], check=True)
    summary = git(checkout, "diff", "--cached", "--stat")
    return len(expected), summary


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("check", "generate", "verify", "stage"))
    parser.add_argument("--repo", type=Path, default=Path(__file__).resolve().parent.parent)
    parser.add_argument("--repository", default="samuelj1519/sheltie", help="GitHub OWNER/REPOSITORY")
    parser.add_argument("--output", type=Path, help="Fresh directory for generate; existing snapshot for verify")
    parser.add_argument("--checkout", type=Path, help="Distinct clean GitHub Wiki checkout for stage")
    args = parser.parse_args(argv)
    try:
        repo = Path(git(args.repo, "rev-parse", "--show-toplevel"))
        if args.command == "check":
            count = len(build(repo, args.repository))
        elif args.output is None:
            parser.error("generate, verify, and stage require --output")
        elif args.command == "generate":
            count = generate(repo, args.repository, args.output)
        elif args.command == "verify":
            count = verify(repo, args.repository, args.output)
        elif args.checkout is None:
            parser.error("stage requires --checkout")
        else:
            count, summary = stage(repo, args.repository, args.output, args.checkout)
            print(summary or "No managed Wiki changes")
    except (WikiError, OSError, UnicodeError, subprocess.CalledProcessError) as error:
        print(f"wiki: FAIL: {error}", file=sys.stderr)
        return 1
    print(f"wiki: {args.command}: OK ({count} generated files)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
