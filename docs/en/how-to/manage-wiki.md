# Publish the bilingual GitHub Wiki

English | [简体中文](../../zh-CN/how-to/manage-wiki.md)

Maintainers use this guide to publish the reader documentation at <https://github.com/samuelj1519/sheltie/wiki>. The main repository owns the authored documents; the independent Wiki Git checkout owns generated presentation. Keeping source beside code and specifications lets one review update them together, without a submodule or a second authoring repository.

## Author and review

- Write English pages under `docs/en/`; keep matching Chinese paths under `docs/zh-CN/`. Use the same filename and information structure in both languages.
- English is the default authority. Update both reader versions when meaning changes, preserving requirements, limits, evidence statuses, and version scope.
- Add reciprocal language links. Link to authoritative English specifications rather than duplicating contracts. Keep Chinese Workbook and skill instructions in their explicit method/instruction locations.
- Edit source pages, then review their source commit. Wiki pages are generated; manual changes to managed Wiki files must be reconciled before staging a replacement.

Run before committing:

```bash
scripts/check-docs.sh
scripts/check-specs.sh
python3 scripts/check-language.py
python3 scripts/wiki.py check
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s scripts/tests -p test_wiki.py
```

`check` permits a dirty source tree and validates page pairs, unique page names, relative targets, fragments, and export transformations. It preserves code blocks and inline code. Generation and verification require a clean, committed source tree. CI and pre-commit run the structural checks; they do not establish translation quality or agent-language efficacy.

## Initialize and clone the Wiki

Prerequisites: a reviewed source commit, Git and Python 3.9+, write access to the repository Wiki, and explicit publication authorization. On the first publication, generate and verify a fresh snapshot using the commands below, then create the Home page through GitHub's Wiki interface using the exact generated `Home.md` content. GitHub then exposes its separate `.wiki.git` repository. See [GitHub's instructions](https://docs.github.com/en/communities/documenting-your-project-with-wikis/adding-or-editing-wiki-pages).

Use an independent checkout inside ignored output or another explicitly chosen directory. This example creates a new checkout; on later runs use the existing clean checkout and `git pull --ff-only`.

```bash
git clone https://github.com/samuelj1519/sheltie.wiki.git output/github-wiki
```

Use existing Git authentication. This process needs no additional application, persistent credential, or host configuration change.

## Generate, inspect, and publish

Run from the main repository root. Choose a previously absent snapshot directory each time:

```bash
python3 scripts/wiki.py generate --output output/wiki-snapshot
python3 scripts/wiki.py verify --output output/wiki-snapshot
python3 scripts/wiki.py stage --output output/wiki-snapshot --checkout output/github-wiki
git -C output/github-wiki diff --cached --stat
git -C output/github-wiki diff --cached -- Home.md Zh-CN-Home.md _Sidebar.md
```

If you generated the snapshot to initialize Home, reuse that verified directory; do not generate over an existing path.

The exporter uses English Home, a separate Chinese Home, stable page names derived from language and full paths, and a shared sidebar. It rewrites reader links to Wiki pages, source links to the exact source commit, and historical data links to copied Wiki assets. The manifest records source and generated hashes. `verify` reconstructs the complete expected artifact from the same clean HEAD; a changed source commit or tampered output fails.

`stage` requires a clean, distinct Wiki checkout with this repository's Wiki origin. It prepares only generated paths, removes only obsolete manifest-managed files, preserves unrelated files, and refuses modified managed content or a new page name that conflicts with different unowned content. It does not commit or push.

After reviewing the prepared changes and confirming publication is authorized:

```bash
git -C output/github-wiki commit -m "docs(wiki): synchronize bilingual documentation"
git -C output/github-wiki push origin HEAD
```

Push uses normal fast-forward protection. If the remote advanced, preserve the local work, inspect the remote change, and reconcile it before publication. Do not use a force push for routine Wiki updates. Verify English Home, Chinese Home, a tutorial, language switching, and source/asset links in the published Wiki.

## History and recovery

The main source commit and Wiki commit are separate identities. Keep both when recording publication. Source archives remain in Git; the generated manifest does not replace original validation evidence. Obtain full history when a shallow clone lacks a recorded snapshot.

To restore a prior Wiki publication, inspect its Git commit, restore the required managed files, review, and publish a new normal commit. Never infer current product acceptance from a historical documentation snapshot.
