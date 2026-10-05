"""Exercise Wiki publication boundaries with real temporary Git repositories."""

import importlib.util
import json
import os
from pathlib import Path
import shlex
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch


sys.dont_write_bytecode = True
SPEC = importlib.util.spec_from_file_location("wiki", Path(__file__).resolve().parents[1] / "wiki.py")
wiki = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(wiki)


class WikiTests(unittest.TestCase):
    def setUp(self):
        environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        environment.update({"GIT_CONFIG_GLOBAL": os.devnull, "GIT_CONFIG_NOSYSTEM": "1"})
        isolation = patch.dict(os.environ, environment, clear=True)
        isolation.start()
        self.addCleanup(isolation.stop)
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.repo = self.root / "repo"
        self.repo.mkdir()
        self.command("init", "-q")
        self.command("config", "user.name", "Wiki test")
        self.command("config", "user.email", "wiki@example.invalid")
        self.write(".gitignore", "/output\n")
        self.write("README.md", "# Project\n")
        self.write("docs/README.md", "# Router\n")
        self.pair("README.md", "# Docs\n\n[Run](how-to/run.md#start)\n")
        self.pair("how-to/run.md", "# Run\n\n## Start\n\n[Spec](../../../specs/spec.md#rules)\n")
        self.write("specs/spec.md", "# Specification\n\n## Rules\n")
        self.commit()
        self.output = self.root / "export"

    def command(self, *args):
        return subprocess.check_output(["git", "-C", str(self.repo), *args], text=True).strip()

    def write(self, path, text):
        file = self.repo / path
        file.parent.mkdir(parents=True, exist_ok=True)
        file.write_text(text)
        return file

    def pair(self, path, text):
        for language in ("en", "zh-CN"):
            self.write(f"docs/{language}/{path}", text)

    def commit(self):
        self.command("add", ".")
        self.command("commit", "-qm", "test: freeze source")

    def test_export_pins_source_and_maps_languages_assets_and_external_links(self):
        self.pair("how-to/README.md", "# Guides\n")
        self.pair("how-to/run.md", "# Run\n\n## Start\n\n[Spec](../../../specs/spec.md#rules)\n\n[Data](../data.json)\n")
        self.write("docs/en/data.json", '{"value":1}\n')
        self.write("docs/zh-CN/data.json", '{"value":2}\n')
        self.commit()
        wiki.generate(self.repo, "owner/project", self.output)
        head = self.command("rev-parse", "HEAD")
        home = (self.output / "Home.md").read_text()
        guide = (self.output / "How-to-run.md").read_text()
        self.assertIn("https://github.com/owner/project/wiki/How-to-run#start", home)
        self.assertIn("https://github.com/owner/project/wiki/Zh-CN-Home", home)
        self.assertIn(f"https://github.com/owner/project/blob/{head}/specs/spec.md#rules", guide)
        self.assertIn("https://raw.githubusercontent.com/wiki/owner/project/assets/docs/en/data.json", guide)
        self.assertEqual((self.output / "assets/docs/en/data.json").read_bytes(), b'{"value":1}\n')
        self.assertIn("wiki/How-to-README", (self.output / "_Sidebar.md").read_text())
        manifest = json.loads((self.output / wiki.MANIFEST).read_text())
        self.assertEqual(manifest["source_commit"], head)
        self.assertIn({"source": "docs/en/data.json", "output": "assets/docs/en/data.json", "sha256": "3a37782e8974c48eebf2a0517c866ad15641c53b3d31993188796b56aeb79624"}, manifest["sources"])
        wiki.verify(self.repo, "owner/project", self.output)

    def test_code_fences_inline_code_and_nested_image_labels(self):
        text = '# Run\n\n## Start\n\n```sh\n[x](missing.md)\n```\n\n~~~text\n[x](missing.md)\n~~~\n\n`[x](missing.md)`\n\n[![alt](../picture.png)](../README.md)\n'
        self.pair("how-to/run.md", text)
        for language in ("en", "zh-CN"):
            self.write(f"docs/{language}/picture.png", "image bytes")
        files = wiki.build(self.repo, "owner/project")
        result = files["How-to-run.md"].decode()
        self.assertIn('```sh\n[x](missing.md)\n```', result)
        self.assertIn('~~~text\n[x](missing.md)\n~~~', result)
        self.assertIn('`[x](missing.md)`', result)
        self.assertIn('[![alt](https://raw.githubusercontent.com/wiki/owner/project/assets/docs/en/picture.png)](https://github.com/owner/project/wiki/Home)', result)

    def test_closed_fences_preserve_trailing_bytes_and_unclosed_fences_fail(self):
        self.pair("how-to/run.md", "# Run\n\n## Start\n\n```sh\nprintf hello  \n```  \n\n\n")
        result = wiki.build(self.repo, "owner/project")["How-to-run.md"].decode()
        self.assertIn("```sh\nprintf hello  \n```  \n\n\n", result)
        self.pair("how-to/run.md", "# Run\n\n## Start\n\n```sh\nprintf hello\n")
        with self.assertRaisesRegex(wiki.WikiError, "Close every fenced"):
            wiki.build(self.repo, "owner/project")

    def test_reference_links_angles_titles_and_balanced_parentheses(self):
        self.pair("how-to/run.md", '# Run\n\n## Start\n\n[Ref][target]\n\n[target]: <../README.md> "Title"\n\n[External](https://example.invalid/a(b) "title")\n')
        result = wiki.build(self.repo, "owner/project")["How-to-run.md"].decode()
        self.assertIn('[target]: <https://github.com/owner/project/wiki/Home> "Title"', result)
        self.assertIn('[External](https://example.invalid/a(b) "title")', result)

    def test_dotted_release_paths_have_stable_page_ids(self):
        self.pair("reference/releases/v0.2.0/README.md", "# Release\n")
        self.assertIn("Reference-releases-v0.2.0-README.md", wiki.build(self.repo, "owner/project"))

    def test_heading_code_preserves_angle_bracket_argument_names(self):
        self.write("specs/spec.md", "# Specification\n\n## `work stats <work>`\n")
        self.pair("how-to/run.md", "# Run\n\n## Start\n\n[Stats](../../../specs/spec.md#work-stats-work)\n")
        self.assertIn("/specs/spec.md#work-stats-work", wiki.build(self.repo, "owner/project")["How-to-run.md"].decode())

    def test_dirty_source_is_checkable_but_cannot_generate_or_verify(self):
        wiki.generate(self.repo, "owner/project", self.output)
        self.write("README.md", "# Changed\n")
        wiki.build(self.repo, "owner/project")
        for operation in (lambda: wiki.generate(self.repo, "owner/project", self.root / "other"), lambda: wiki.verify(self.repo, "owner/project", self.output)):
            with self.assertRaisesRegex(wiki.WikiError, "clean source"):
                operation()
        self.assertFalse((self.root / "other").exists())

    def test_missing_partners_collisions_targets_and_fragments_fail(self):
        cases = (
            ("docs/zh-CN/how-to/run.md", None, "Missing language partner"),
            ("docs/en/how-to-run.md", "# Collision\n", "collision"),
            ("docs/en/README.md", "# Docs\n[x](missing.md)\n", "missing or escaping"),
            ("docs/en/README.md", "# Docs\n[x](how-to/run.md#absent)\n", "missing anchor"),
            ("docs/en/README.md", "# Docs\n[x](../../../../outside.md)\n", "escaping"),
        )
        for path, value, message in cases:
            with self.subTest(message=message):
                original = self.repo / path
                previous = original.read_bytes() if original.exists() else None
                if value is None:
                    original.unlink()
                else:
                    self.write(path, value)
                with self.assertRaisesRegex(wiki.WikiError, message):
                    wiki.build(self.repo, "owner/project")
                if previous is None:
                    original.unlink()
                else:
                    original.write_bytes(previous)

    def test_publication_rejects_symlinks_existing_output_and_authored_output(self):
        with self.assertRaisesRegex(wiki.WikiError, "outside authored"):
            wiki.generate(self.repo, "owner/project", self.repo / "docs/generated")
        with self.assertRaisesRegex(wiki.WikiError, "Git-ignored"):
            wiki.generate(self.repo, "owner/project", self.repo / "generated")
        self.output.mkdir()
        with self.assertRaisesRegex(wiki.WikiError, "must not exist"):
            wiki.generate(self.repo, "owner/project", self.output)
        link = self.repo / "docs/en/link.md"
        link.symlink_to(self.repo / "README.md")
        with self.assertRaisesRegex(wiki.WikiError, "Symlinks"):
            wiki.build(self.repo, "owner/project")

    def test_verify_detects_tampering_extra_files_and_new_source_commit(self):
        wiki.generate(self.repo, "owner/project", self.output)
        home = self.output / "Home.md"
        original = home.read_bytes()
        home.write_bytes(b"altered")
        with self.assertRaisesRegex(wiki.WikiError, "Home.md"):
            wiki.verify(self.repo, "owner/project", self.output)
        home.write_bytes(original)
        extra = self.output / "Unexpected.md"
        extra.write_text("extra")
        with self.assertRaisesRegex(wiki.WikiError, "Unexpected.md"):
            wiki.verify(self.repo, "owner/project", self.output)
        extra.unlink()
        self.write("README.md", "# New version\n")
        self.commit()
        with self.assertRaisesRegex(wiki.WikiError, "pinned source"):
            wiki.verify(self.repo, "owner/project", self.output)

    def test_ignored_output_remains_clean_and_is_deterministic(self):
        output = self.repo / "output/wiki"
        wiki.generate(self.repo, "owner/project", output)
        self.assertEqual(self.command("status", "--porcelain"), "")
        wiki.verify(self.repo, "owner/project", output)
        another = self.root / "second"
        wiki.generate(self.repo, "owner/project", another)
        self.assertEqual((output / wiki.MANIFEST).read_bytes(), (another / wiki.MANIFEST).read_bytes())

    def test_ignored_uncommitted_docs_cannot_enter_publication(self):
        self.write(".gitignore", "/output\n*.json\n")
        self.commit()
        self.write("docs/en/uncommitted.json", "{}\n")
        self.assertEqual(self.command("status", "--porcelain"), "")
        with self.assertRaisesRegex(wiki.WikiError, "uncommitted docs"):
            wiki.generate(self.repo, "owner/project", self.output)
        self.assertFalse(self.output.exists())

    def test_publication_rejects_hidden_doc_changes_despite_clean_git_status(self):
        wiki.generate(self.repo, "owner/project", self.output)
        self.command("update-index", "--assume-unchanged", "docs/en/README.md")
        wiki.verify(self.repo, "owner/project", self.output)
        page = self.repo / "docs/en/README.md"
        page.write_text(page.read_text() + "\nUncommitted prose\n")
        self.assertEqual(self.command("status", "--porcelain"), "")
        self.assertIn(b"Uncommitted prose", wiki.build(self.repo, "owner/project")["Home.md"])
        output = self.root / "hidden-doc-export"
        with self.assertRaisesRegex(wiki.WikiError, "pinned"):
            wiki.generate(self.repo, "owner/project", output)
        self.assertFalse(output.exists())

    def test_publication_rejects_hidden_asset_changes_and_missing_tracked_assets(self):
        asset = self.write("docs/en/data.json", '{"value":1}\n')
        self.commit()
        wiki.generate(self.repo, "owner/project", self.output)
        self.assertEqual((self.output / "assets/docs/en/data.json").read_bytes(), b'{"value":1}\n')
        self.command("update-index", "--assume-unchanged", "docs/en/data.json")
        wiki.verify(self.repo, "owner/project", self.output)
        asset.write_text('{"value":2}\n')
        self.assertEqual(self.command("status", "--porcelain"), "")
        output = self.root / "hidden-asset-export"
        with self.assertRaisesRegex(wiki.WikiError, "pinned"):
            wiki.generate(self.repo, "owner/project", output)
        self.assertFalse(output.exists())
        asset.unlink()
        self.assertEqual(self.command("status", "--porcelain"), "")
        with self.assertRaisesRegex(wiki.WikiError, "Missing pinned docs"):
            wiki.generate(self.repo, "owner/project", output)
        self.assertFalse(output.exists())

    def test_publication_validates_outside_markdown_anchors_from_pinned_blobs(self):
        self.command("update-index", "--assume-unchanged", "specs/spec.md")
        self.write("specs/spec.md", "# Specification\n\n## Hidden heading\n")
        self.assertEqual(self.command("status", "--porcelain"), "")
        with self.assertRaisesRegex(wiki.WikiError, "missing anchor"):
            wiki.build(self.repo, "owner/project")
        wiki.generate(self.repo, "owner/project", self.output)
        self.assertIn("#rules", (self.output / "How-to-run.md").read_text())
        self.pair("how-to/run.md", "# Run\n\n## Start\n\n[Spec](../../../specs/spec.md#hidden-heading)\n")
        self.commit()
        wiki.build(self.repo, "owner/project")
        output = self.root / "hidden-anchor-export"
        with self.assertRaisesRegex(wiki.WikiError, "missing anchor"):
            wiki.generate(self.repo, "owner/project", output)
        self.assertFalse(output.exists())

    def wiki_checkout(self, checkout=None, git_dir=None):
        checkout = checkout or self.root / "wiki"
        checkout.mkdir(parents=True)
        def command(*args):
            return subprocess.check_output(["git", "-C", str(checkout), *args], text=True).strip()
        init_args = ["init", "-q"]
        if git_dir is not None:
            init_args.extend(["--separate-git-dir", str(git_dir)])
        command(*init_args)
        command("config", "user.name", "Wiki test")
        command("config", "user.email", "wiki@example.invalid")
        command("remote", "add", "origin", "https://github.com/owner/project.wiki.git")
        (checkout / "Home.md").write_text("Initial Wiki home\n")
        (checkout / "Unrelated.md").write_text("Preserve this independent page\n")
        command("add", ".")
        command("commit", "-qm", "docs: initialize wiki")
        return checkout, command

    def seed_home(self, checkout, command, output):
        (checkout / "Home.md").write_bytes((output / "Home.md").read_bytes())
        command("add", "Home.md")
        command("commit", "-qm", "docs: seed exact generated home")

    def checkout_bytes(self, checkout):
        return {path.relative_to(checkout).as_posix(): path.read_bytes() for path in checkout.rglob("*") if path.is_file()}

    def test_fixture_git_isolation_preserves_enclosing_hook_repository(self):
        hooks = self.root / "host-hooks"
        hooks.mkdir()
        marker = self.root / "host-hook-ran"
        hook = hooks / "pre-commit"
        hook.write_text("#!/bin/sh\nprintf leaked > " + shlex.quote(str(marker)) + "\n")
        hook.chmod(0o755)
        global_config = self.root / "host-global-config"
        global_config.write_text('[user]\n\tname = Host user\n\temail = host@example.invalid\n')
        system_config = self.root / "host-system-config"
        system_config.write_text("[core]\n\thooksPath = " + str(hooks) + "\n")
        injected = dict(os.environ)
        injected.update({
            "GIT_DIR": str(self.repo / ".git"),
            "GIT_COMMON_DIR": str(self.repo / ".git"),
            "GIT_WORK_TREE": str(self.repo),
            "GIT_INDEX_FILE": str(self.repo / ".git/index"),
            "GIT_CONFIG_GLOBAL": str(global_config),
            "GIT_CONFIG_SYSTEM": str(system_config),
            "GIT_CONFIG_NOSYSTEM": "0",
            "GIT_CONFIG_COUNT": "2",
            "GIT_CONFIG_KEY_0": "core.hooksPath",
            "GIT_CONFIG_VALUE_0": str(hooks),
            "GIT_CONFIG_KEY_1": "user.name",
            "GIT_CONFIG_VALUE_1": "Injected host user",
        })
        before = self.checkout_bytes(self.repo)
        config_before = (global_config.read_bytes(), system_config.read_bytes())
        child = subprocess.run([sys.executable, str(Path(__file__).resolve()), "WikiTests.test_stage_updates_only_managed_files_and_removes_obsolete_pages", "-q"], env=injected, capture_output=True, text=True, check=False)
        self.assertEqual(child.returncode, 0, child.stdout + child.stderr)
        self.assertFalse(marker.exists())
        self.assertEqual(self.checkout_bytes(self.repo), before)
        self.assertEqual((global_config.read_bytes(), system_config.read_bytes()), config_before)

    def test_stage_allows_separate_wiki_in_ignored_source_output(self):
        checkout, command = self.wiki_checkout(self.repo / "output/github-wiki")
        output = self.repo / "output/export"
        self.assertEqual(self.command("status", "--porcelain"), "")
        wiki.generate(self.repo, "owner/project", output)
        self.seed_home(checkout, command, output)
        _, summary = wiki.stage(self.repo, "owner/project", output, checkout)
        self.assertIn("How-to-run.md", summary)
        self.assertIn("How-to-run.md", command("diff", "--cached", "--name-only"))
        self.assertEqual(self.command("status", "--porcelain"), "")

    def test_stage_rejects_unignored_nesting_authored_docs_and_git_metadata(self):
        wiki.generate(self.repo, "owner/project", self.output)
        unignored = self.repo / "nested-wiki"
        unignored.mkdir()
        with self.assertRaisesRegex(wiki.WikiError, "Git-ignored"):
            wiki.stage(self.repo, "owner/project", self.output, unignored)
        with self.assertRaisesRegex(wiki.WikiError, "authored docs"):
            wiki.stage(self.repo, "owner/project", self.output, self.repo / "docs")
        with self.assertRaisesRegex(wiki.WikiError, "Git metadata"):
            wiki.stage(self.repo, "owner/project", self.output, self.repo / ".git")
        unignored.rmdir()
        checkout, _ = self.wiki_checkout(unignored)
        with self.assertRaisesRegex(wiki.WikiError, "clean source"):
            wiki.stage(self.repo, "owner/project", self.output, checkout)

    def test_stage_rejects_separate_wiki_git_dir_inside_source_git_metadata(self):
        source_git = Path(self.command("rev-parse", "--path-format=absolute", "--git-common-dir"))
        checkout, command = self.wiki_checkout(git_dir=source_git / "wiki-cache")
        wiki.generate(self.repo, "owner/project", self.output)
        self.seed_home(checkout, command, self.output)
        before_source_git = self.checkout_bytes(source_git)
        before_wiki = self.checkout_bytes(checkout)
        with self.assertRaisesRegex(wiki.WikiError, "Git metadata"):
            wiki.stage(self.repo, "owner/project", self.output, checkout)
        self.assertEqual(self.checkout_bytes(source_git), before_source_git)
        self.assertEqual(self.checkout_bytes(checkout), before_wiki)
        self.assertEqual(command("status", "--porcelain"), "")

    def test_stage_updates_only_managed_files_and_removes_obsolete_pages(self):
        checkout, command = self.wiki_checkout()
        self.pair("obsolete.md", "# Obsolete\n")
        self.commit()
        wiki.generate(self.repo, "owner/project", self.output)
        self.seed_home(checkout, command, self.output)
        original_head = command("rev-parse", "HEAD")
        _, summary = wiki.stage(self.repo, "owner/project", self.output, checkout)
        self.assertIn("How-to-run.md", summary)
        self.assertEqual(command("rev-parse", "HEAD"), original_head)
        self.assertEqual((checkout / "Unrelated.md").read_text(), "Preserve this independent page\n")
        self.assertNotIn("Unrelated.md", command("diff", "--cached", "--name-only"))
        command("commit", "-qm", "docs: publish generated snapshot")
        for language in ("en", "zh-CN"):
            (self.repo / f"docs/{language}/obsolete.md").unlink()
        self.commit()
        new_output = self.root / "next-export"
        wiki.generate(self.repo, "owner/project", new_output)
        wiki.stage(self.repo, "owner/project", new_output, checkout)
        self.assertFalse((checkout / "Obsolete.md").exists())
        self.assertFalse((checkout / "Zh-CN-Obsolete.md").exists())
        self.assertEqual((checkout / "Unrelated.md").read_text(), "Preserve this independent page\n")
        self.assertIn("D\tObsolete.md", command("diff", "--cached", "--name-status"))
        self.assertTrue((checkout / ".git/config").is_file())

    def test_stage_rejects_colliding_unowned_page_before_any_mutation(self):
        checkout, command = self.wiki_checkout()
        wiki.generate(self.repo, "owner/project", self.output)
        self.seed_home(checkout, command, self.output)
        wiki.stage(self.repo, "owner/project", self.output, checkout)
        command("commit", "-qm", "docs: publish generated snapshot")
        (checkout / "Clash.md").write_text("Independent Wiki content\n")
        command("add", "Clash.md")
        command("commit", "-qm", "docs: preserve independent page")
        self.pair("clash.md", "# New generated page\n")
        self.commit()
        output = self.root / "next-export"
        wiki.generate(self.repo, "owner/project", output)
        before = self.checkout_bytes(checkout)
        with self.assertRaisesRegex(wiki.WikiError, "unowned"):
            wiki.stage(self.repo, "owner/project", output, checkout)
        self.assertEqual(self.checkout_bytes(checkout), before)
        self.assertEqual(command("status", "--porcelain"), "")

    def test_stage_uses_literal_asset_paths_and_keeps_ignored_private_files_unstaged(self):
        self.write("docs/en/data*.json", '{"public":true}\n')
        self.commit()
        wiki.generate(self.repo, "owner/project", self.output)
        checkout, command = self.wiki_checkout()
        self.seed_home(checkout, command, self.output)
        (checkout / ".gitignore").write_text("assets/docs/en/data-private.json\n")
        command("add", ".gitignore")
        command("commit", "-qm", "test: ignore private Wiki file")
        private = checkout / "assets/docs/en/data-private.json"
        private.parent.mkdir(parents=True)
        private.write_text("PRIVATE_SENTINEL\n")
        self.assertEqual(command("status", "--porcelain"), "")
        wiki.stage(self.repo, "owner/project", self.output, checkout)
        staged = set(command("diff", "--cached", "--name-only").splitlines())
        self.assertNotIn("assets/docs/en/data-private.json", staged)
        self.assertEqual(staged, {"How-to-run.md", "Zh-CN-Home.md", "Zh-CN-How-to-run.md", "assets/docs/en/data*.json", "_Sidebar.md", wiki.MANIFEST})
        self.assertEqual(private.read_text(), "PRIVATE_SENTINEL\n")
        self.assertNotIn("PRIVATE_SENTINEL", command("diff", "--cached"))

    def test_stage_rejects_hardlinked_owned_file_and_preserves_outside_bytes(self):
        checkout, command = self.wiki_checkout()
        wiki.generate(self.repo, "owner/project", self.output)
        self.seed_home(checkout, command, self.output)
        wiki.stage(self.repo, "owner/project", self.output, checkout)
        command("commit", "-qm", "docs: publish generated snapshot")
        home = checkout / "Home.md"
        sentinel = self.root / "outside.md"
        original = home.read_bytes()
        sentinel.write_bytes(original)
        home.unlink()
        os.link(sentinel, home)
        self.write("README.md", "# Advance pinned source\n")
        self.commit()
        output = self.root / "next-export"
        wiki.generate(self.repo, "owner/project", output)
        self.assertNotEqual((output / "Home.md").read_bytes(), original)
        self.assertEqual(command("status", "--porcelain"), "")
        before = self.checkout_bytes(checkout)
        with self.assertRaisesRegex(wiki.WikiError, "hardlink"):
            wiki.stage(self.repo, "owner/project", output, checkout)
        self.assertEqual(sentinel.read_bytes(), original)
        self.assertEqual(self.checkout_bytes(checkout), before)
        self.assertEqual(command("status", "--porcelain"), "")

    def test_stage_rejects_wrong_remote_dirty_clone_unsafe_manifest_and_overlap(self):
        checkout, command = self.wiki_checkout()
        wiki.generate(self.repo, "owner/project", self.output)
        command("remote", "set-url", "origin", "https://github.com/owner/different.wiki.git")
        with self.assertRaisesRegex(wiki.WikiError, "origin"):
            wiki.stage(self.repo, "owner/project", self.output, checkout)
        command("remote", "set-url", "origin", "git@github.com:owner/project.wiki.git")
        unrelated = checkout / "Unrelated.md"
        unrelated.write_text("Dirty\n")
        with self.assertRaisesRegex(wiki.WikiError, "clean source"):
            wiki.stage(self.repo, "owner/project", self.output, checkout)
        command("restore", "Unrelated.md")
        config_bytes = (checkout / ".git/config").read_bytes()
        for unsafe in ("../escape.md", ".git/config", ".GIT/config", "/absolute.md"):
            with self.subTest(path=unsafe):
                malicious = {"format": "sheltie-wiki/v1", "repository": "owner/project", "files": {unsafe: "0" * 64}}
                (checkout / wiki.MANIFEST).write_text(json.dumps(malicious))
                command("add", wiki.MANIFEST)
                command("commit", "-qm", "test: unsafe manifest")
                with self.assertRaisesRegex(wiki.WikiError, "Invalid managed path"):
                    wiki.stage(self.repo, "owner/project", self.output, checkout)
                self.assertEqual((checkout / ".git/config").read_bytes(), config_bytes)
        self.assertEqual(command("status", "--porcelain"), "")
        self.assertEqual((checkout / "Home.md").read_text(), "Initial Wiki home\n")
        with self.assertRaisesRegex(wiki.WikiError, "overlap"):
            wiki.stage(self.repo, "owner/project", self.output, self.repo)


if __name__ == "__main__":
    unittest.main()
