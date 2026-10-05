//! C002-T13: self-contained skill installation artifacts (GF-18).
//!
//! Handwritten expectations: independently parse or define delivered files, archive members, links, and command allowlists,
//! without calling skill-delivery.sh parsing logic; run the script only as the subject under test.
//! Install/validate only in temporary directories, without touching host configuration.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use assert_cmd::Command as Bin;
use common::copy_dir;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Run scripts in an isolated source-tree substitute mirroring scripts, skills, and contracts.
fn temp_source_tree(base: &Path) -> PathBuf {
    let tree = base.join("src");
    copy_dir(&repo_root().join("scripts"), &tree.join("scripts"));
    copy_dir(&repo_root().join("skills"), &tree.join("skills"));
    fs::create_dir_all(tree.join("specs")).unwrap();
    copy_dir(
        &repo_root().join("specs/contracts"),
        &tree.join("specs/contracts"),
    );
    tree
}

fn bash(script: &Path, args: &[&str]) -> Output {
    Command::new("bash")
        .arg(script)
        .args(args)
        .output()
        .unwrap()
}

fn out_text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// Handwritten link extraction matching check-docs: skip URLs/pure anchors and strip target anchors.
fn link_targets(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(p) = rest.find("](") {
        let after = &rest[p + 2..];
        let Some(end) = after.find(')') else {
            break;
        };
        let raw = &after[..end];
        rest = &after[end + 1..];
        if raw.is_empty() || raw.starts_with('#') || raw.contains("://") || raw.contains(' ') {
            continue;
        }
        out.push(raw.split('#').next().unwrap().to_string());
    }
    out
}

fn md_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in fs::read_dir(&d).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().map(|e| e == "md").unwrap_or(false) {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

fn file_set(dir: &Path) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut stack = vec![(dir.to_path_buf(), String::new())];
    while let Some((d, prefix)) = stack.pop() {
        for entry in fs::read_dir(&d).unwrap() {
            let entry = entry.unwrap();
            let name = entry.file_name().to_string_lossy().to_string();
            let rel = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{prefix}/{name}")
            };
            if entry.path().is_dir() {
                stack.push((entry.path(), rel));
            } else {
                out.insert(rel);
            }
        }
    }
    out
}

/// Resolve every local relative delivery link beneath the delivery root.
fn delivery_links(dir: &Path) -> Vec<(String, String)> {
    let root = dir.canonicalize().unwrap();
    let mut out = Vec::new();
    for md in md_files(dir) {
        let rel = md
            .strip_prefix(dir)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let text = fs::read_to_string(&md).unwrap();
        for t in link_targets(&text) {
            let joined = md.parent().unwrap().join(&t);
            assert!(joined.exists(), "Cannot resolve link: {rel} -> {t}");
            let canon = joined.canonicalize().unwrap();
            assert!(
                canon.starts_with(&root),
                "Link escapes delivery root: {rel} -> {t}"
            );
            out.push((rel.clone(), t));
        }
    }
    out.sort();
    out
}

/// Commands referenced by SKILL.md: sheltie <group> <verb>.
fn skill_commands(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for line in text.lines() {
        let Some(rest) = line.trim().strip_prefix("sheltie ") else {
            continue;
        };
        let mut words = rest.split_whitespace();
        let group = words.next().unwrap().to_string();
        let verb = words.next().unwrap().to_string();
        out.push((group, verb));
    }
    out
}

/// Protocol §2 group/verb pairs, scanning only that section until the next ## heading.
fn protocol_pairs(text: &str) -> BTreeSet<(String, String)> {
    let mut out = BTreeSet::new();
    let mut inside = false;
    for line in text.lines() {
        if line.starts_with("## 2.") {
            inside = true;
            continue;
        }
        if line.starts_with("## ") {
            inside = false;
        }
        if !inside {
            continue;
        }
        let Some(rest) = line.strip_prefix("| `") else {
            continue;
        };
        let Some(end) = rest.find('`') else {
            continue;
        };
        // Match check-skill.sh: take the next two words in the cell as group/verb
        let mut words = rest[..end].split_whitespace();
        let (Some(a), Some(b)) = (words.next(), words.next()) else {
            continue;
        };
        out.insert((a.to_string(), b.to_string()));
    }
    out
}

/// Independently strip Markdown links to their labels, verifying generated references preserve authority prose
/// verbatim except for link markup; link stripping must not swallow body text.
fn strip_links(text: &str) -> String {
    let mut out = String::new();
    let mut i = 0;
    while let Some(rel) = text[i..].find('[') {
        let open = i + rel;
        let after_open = &text[open + 1..];
        let Some(close) = after_open.find("](") else {
            out.push_str(&text[i..]);
            return out;
        };
        let text_end = open + 1 + close;
        if text[open + 1..text_end].contains('[') {
            out.push_str(&text[i..open + 1]);
            i = open + 1;
            continue;
        }
        let after_br = &text[text_end + 2..];
        let Some(paren) = after_br.find(')') else {
            out.push_str(&text[i..]);
            return out;
        };
        out.push_str(&text[i..open]);
        out.push_str(&text[open + 1..text_end]);
        i = text_end + 2 + paren + 1;
    }
    out.push_str(&text[i..]);
    out
}

/// Validate isolated installations: links, paths, and commands, also testing each command with real-binary --help.
fn assert_delivery_resolves(dir: &Path) {
    let links = delivery_links(dir);
    assert!(
        links
            .iter()
            .all(|(_, t)| !t.contains("specs/") && !t.contains("..")),
        "Delivery must not reference repository paths: {links:?}"
    );
    let expected: BTreeSet<(String, String)> = [
        ("SKILL.md", "SKILL.zh-CN.md"),
        ("SKILL.md", "references/protocol.md"),
        ("SKILL.md", "references/workbook.md"),
        ("SKILL.zh-CN.md", "SKILL.md"),
        ("SKILL.zh-CN.md", "references/protocol.zh-CN.md"),
        ("SKILL.zh-CN.md", "references/workbook.zh-CN.md"),
        ("references/protocol.md", "protocol.zh-CN.md"),
        ("references/protocol.md", "workbook.md"),
        ("references/protocol.zh-CN.md", "protocol.md"),
        ("references/protocol.zh-CN.md", "workbook.zh-CN.md"),
        ("references/workbook.md", "workbook.zh-CN.md"),
        ("references/workbook.md", "protocol.md"),
        ("references/workbook.zh-CN.md", "workbook.md"),
        ("references/workbook.zh-CN.md", "protocol.zh-CN.md"),
    ]
    .into_iter()
    .map(|(from, to)| (from.to_string(), to.to_string()))
    .collect();
    assert_eq!(links.iter().cloned().collect::<BTreeSet<_>>(), expected);

    for (skill_file, protocol_file) in [
        ("SKILL.md", "protocol.md"),
        ("SKILL.zh-CN.md", "protocol.zh-CN.md"),
    ] {
        let skill = fs::read_to_string(dir.join(skill_file)).unwrap();
        let protocol = fs::read_to_string(dir.join("references").join(protocol_file)).unwrap();
        let allowed = protocol_pairs(&protocol);
        let cmds = skill_commands(&skill);
        assert!(
            cmds.len() >= 8,
            "{skill_file} must demonstrate at least eight commands; actual {}",
            cmds.len()
        );
        for (g, v) in &cmds {
            assert!(
                allowed.contains(&(g.clone(), v.clone())),
                "Command absent from {protocol_file} section 2: sheltie {g} {v}"
            );
            Bin::cargo_bin("sheltie")
                .unwrap()
                .args([g.as_str(), v.as_str(), "--help"])
                .assert()
                .success();
        }
    }
}

fn install_copy(from: &Path, host: &Path) -> PathBuf {
    let installed = host.join("skills").join("sheltie");
    copy_dir(from, &installed);
    installed
}

/// Generate delivery beneath the substitute source tree at out/sheltie; repository-relative escaped links
/// still resolve in the tree, isolating the delivery-root escape rejection.
fn pack_in_tree(tree: &Path) -> PathBuf {
    let dist = tree.join("out").join("sheltie");
    let out = bash(
        &tree.join("scripts/skill-delivery.sh"),
        &["pack", dist.to_str().unwrap()],
    );
    assert!(out.status.success(), "{}", out_text(&out));
    dist
}

// Task: C002-T13
#[test]
fn generated_reference_keeps_authority_prose_verbatim() {
    let tmp = tempfile::tempdir().unwrap();
    let tree = temp_source_tree(tmp.path());
    let dist = pack_in_tree(&tree);
    for name in [
        "protocol.md",
        "protocol.zh-CN.md",
        "workbook.md",
        "workbook.zh-CN.md",
    ] {
        let authority = fs::read_to_string(tree.join("specs/contracts").join(name)).unwrap();
        let generated = fs::read_to_string(dist.join("references").join(name)).unwrap();
        assert_eq!(
            strip_links(&authority),
            strip_links(&generated),
            "{name} must match authority prose verbatim except for link markup"
        );
    }
}

// Task: C002-T13
#[test]
fn installed_delivery_stays_self_contained_after_source_tree_is_removed() {
    let tmp = tempfile::tempdir().unwrap();
    let tree = temp_source_tree(tmp.path());
    let dist = tmp.path().join("dist").join("sheltie");
    let out = bash(
        &tree.join("scripts/skill-delivery.sh"),
        &["pack", dist.to_str().unwrap()],
    );
    assert!(out.status.success(), "{}", out_text(&out));

    // Handwritten closure: both skill languages and four generated contract references
    let files = file_set(&dist);
    let expected: BTreeSet<String> = [
        "SKILL.md".to_string(),
        "SKILL.zh-CN.md".to_string(),
        "references/protocol.md".to_string(),
        "references/protocol.zh-CN.md".to_string(),
        "references/workbook.md".to_string(),
        "references/workbook.zh-CN.md".to_string(),
    ]
    .into_iter()
    .collect();
    assert_eq!(files, expected);

    // Isolated installation into a temporary host skill directory
    let host = tmp.path().join("host-home");
    let installed = install_copy(&dist, &host);
    assert_delivery_resolves(&installed);

    // Remove the source tree, relocate the installation, and repeat the same checks
    fs::remove_dir_all(&tree).unwrap();
    let moved_host = tmp.path().join("moved-home");
    fs::rename(&host, &moved_host).unwrap();
    assert_delivery_resolves(&moved_host.join("skills").join("sheltie"));
}

// Task: C002-T13
#[test]
fn release_tarball_matches_readme_install_shape() {
    let tmp = tempfile::tempdir().unwrap();
    let tarball = tmp.path().join("sheltie-skill.tar.gz");
    let out = bash(
        &repo_root().join("scripts/skill-delivery.sh"),
        &["tar", tarball.to_str().unwrap()],
    );
    assert!(out.status.success(), "{}", out_text(&out));

    // Handwritten archive member expectations, excluding directory entries
    let list = Command::new("tar")
        .args(["tzf", tarball.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(list.status.success());
    let mut members: Vec<String> = String::from_utf8_lossy(&list.stdout)
        .lines()
        .filter(|l| !l.ends_with('/'))
        .map(|l| l.trim_end_matches('/').to_string())
        .collect();
    members.sort();
    assert_eq!(
        members,
        vec![
            "sheltie/SKILL.md",
            "sheltie/SKILL.zh-CN.md",
            "sheltie/references/protocol.md",
            "sheltie/references/protocol.zh-CN.md",
            "sheltie/references/workbook.md",
            "sheltie/references/workbook.zh-CN.md"
        ]
    );

    // README installation: extract into skills to obtain a self-contained skill
    let skills_dir = tmp.path().join("claude-skills");
    fs::create_dir_all(&skills_dir).unwrap();
    let x = Command::new("tar")
        .args([
            "xzf",
            tarball.to_str().unwrap(),
            "-C",
            skills_dir.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(x.status.success());
    assert_delivery_resolves(&skills_dir.join("sheltie"));

    // README download and release workflow refer to the same asset name
    let asset = "sheltie-skill.tar.gz";
    let readme = fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(
        readme.contains(&format!("releases/latest/download/{asset}")),
        "README must give this release asset's download URL"
    );
    let workflow = fs::read_to_string(repo_root().join(".github/workflows/release.yml")).unwrap();
    assert!(
        workflow.contains(&format!("artifacts/{asset}")),
        "Release workflow must attach the same-named asset"
    );
    assert!(
        workflow.contains("scripts/skill-delivery.sh tar"),
        "Release workflow packaging entry must be skill-delivery.sh tar"
    );
}

// Task: C002-T13
#[test]
fn check_skill_fails_when_delivery_misses_one_reference() {
    let tmp = tempfile::tempdir().unwrap();
    let tree = temp_source_tree(tmp.path());
    let dist = pack_in_tree(&tree);

    // One changed condition: omit the workbook reference
    fs::remove_file(dist.join("references/workbook.md")).unwrap();
    let out = bash(
        &tree.join("scripts/check-skill.sh"),
        &["--delivery", dist.to_str().unwrap()],
    );
    assert!(!out.status.success(), "{}", out_text(&out));
    assert!(
        out_text(&out).contains("references/workbook.md"),
        "{}",
        out_text(&out)
    );
}

// Task: C002-T13
#[test]
fn check_skill_fails_when_delivery_reference_is_stale() {
    let tmp = tempfile::tempdir().unwrap();
    let tree = temp_source_tree(tmp.path());
    let dist = pack_in_tree(&tree);

    // One changed condition: stale protocol reference, by appending one line
    let protocol = dist.join("references/protocol.md");
    let mut text = fs::read_to_string(&protocol).unwrap();
    text.push_str("(stale)\n");
    fs::write(&protocol, text).unwrap();
    let out = bash(
        &tree.join("scripts/check-skill.sh"),
        &["--delivery", dist.to_str().unwrap()],
    );
    assert!(!out.status.success(), "{}", out_text(&out));
    assert!(
        out_text(&out).contains("references/protocol.md"),
        "{}",
        out_text(&out)
    );
}

// Task: C002-T13
#[test]
fn check_skill_fails_when_delivery_link_escapes() {
    let tmp = tempfile::tempdir().unwrap();
    let tree = temp_source_tree(tmp.path());
    let dist = pack_in_tree(&tree);

    // One changed condition: delivery links fall back to repository-relative paths (O11).
    // Delivery at out/sheltie leaves the path resolvable, isolating the delivery-root escape rejection.
    let skill = dist.join("SKILL.md");
    let text = fs::read_to_string(&skill).unwrap();
    fs::write(
        &skill,
        text.replace(
            "](references/protocol.md)",
            "](../../specs/contracts/protocol.md)",
        ),
    )
    .unwrap();
    let out = bash(
        &tree.join("scripts/check-skill.sh"),
        &["--delivery", dist.to_str().unwrap()],
    );
    assert!(!out.status.success(), "{}", out_text(&out));
    assert!(
        out_text(&out).contains("escapes the delivery directory"),
        "{}",
        out_text(&out)
    );
}

// Task: C002-T13
#[test]
fn check_skill_fails_when_reference_source_is_gone() {
    let tmp = tempfile::tempdir().unwrap();
    let tree = temp_source_tree(tmp.path());

    // One changed condition: missing workbook authority prevents reference generation
    fs::remove_file(tree.join("specs/contracts/workbook.md")).unwrap();
    let out = bash(&tree.join("scripts/check-skill.sh"), &[]);
    assert!(!out.status.success(), "{}", out_text(&out));
    assert!(out_text(&out).contains("workbook.md"), "{}", out_text(&out));
}

// Task: C002-T13
#[test]
fn check_skill_passes_on_repository_skill() {
    let out = bash(&repo_root().join("scripts/check-skill.sh"), &[]);
    assert!(out.status.success(), "{}", out_text(&out));
    assert!(
        out_text(&out).contains("check-skill: OK"),
        "{}",
        out_text(&out)
    );
}
