//! C002-T13：skill 安装产物自包含（GF-18）。
//!
//! 期望值全部手写：交付文件集合、发布资产成员表、链接清单、命令白名单都由本文件独立
//! 解析或写死，不调用 scripts/skill-delivery.sh 的解析逻辑——脚本只作为被测对象运行。
//! 安装与校验都发生在临时目录，绝不触碰真实宿主配置。
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use assert_cmd::Command as Bin;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn copy_tree(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_tree(&from, &to);
        } else {
            fs::copy(&from, &to).unwrap();
        }
    }
}

/// 在独立源码树替身里跑脚本：scripts + skills + specs/contracts 与仓库同构。
fn temp_source_tree(base: &Path) -> PathBuf {
    let tree = base.join("src");
    copy_tree(&repo_root().join("scripts"), &tree.join("scripts"));
    copy_tree(&repo_root().join("skills"), &tree.join("skills"));
    fs::create_dir_all(tree.join("specs")).unwrap();
    copy_tree(
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

/// 与 check-docs.sh 同一取法的手写版：](目标)，跳过 URL 与纯锚点，去掉 #锚点。
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

/// 逐个解析交付内的本地链接（相对路径），必须全部在交付目录内解析得到。
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
            assert!(joined.exists(), "链接解析不到：{rel} → {t}");
            let canon = joined.canonicalize().unwrap();
            assert!(canon.starts_with(&root), "链接越出交付目录：{rel} → {t}");
            out.push((rel.clone(), t));
        }
    }
    out.sort();
    out
}

/// SKILL.md 里引用的命令（sheltie <group> <verb>）。
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

/// 协议 §2 操作一览里的 `group verb` 对（只扫 §2 标题到下一个 ## 之间）。
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
        // 与 check-skill.sh 的取法一致：单元格里取随后两个词作 group verb
        let mut words = rest[..end].split_whitespace();
        let (Some(a), Some(b)) = (words.next(), words.next()) else {
            continue;
        };
        out.insert((a.to_string(), b.to_string()));
    }
    out
}

/// 把 markdown 链接 `[文字](目标)` 收成 `文字`。独立手写实现，用来核生成 reference
/// 除链接外与单一权威逐字相同——去链接变换不得吞正文。
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

/// 隔离安装副本逐项可解析：链接、相对路径、引用命令（命令还用真实二进制过一遍 --help）。
fn assert_delivery_resolves(dir: &Path) {
    let links = delivery_links(dir);
    assert!(
        links
            .iter()
            .all(|(_, t)| !t.contains("specs/") && !t.contains("..")),
        "交付里不应再指向仓库路径：{links:?}"
    );
    let expected: BTreeSet<(String, String)> = [
        ("SKILL.md".to_string(), "references/protocol.md".to_string()),
        ("SKILL.md".to_string(), "references/workbook.md".to_string()),
        (
            "references/protocol.md".to_string(),
            "workbook.md".to_string(),
        ),
        (
            "references/workbook.md".to_string(),
            "protocol.md".to_string(),
        ),
    ]
    .into_iter()
    .collect();
    assert_eq!(links.iter().cloned().collect::<BTreeSet<_>>(), expected);

    let skill = fs::read_to_string(dir.join("SKILL.md")).unwrap();
    let protocol = fs::read_to_string(dir.join("references/protocol.md")).unwrap();
    let allowed = protocol_pairs(&protocol);
    let cmds = skill_commands(&skill);
    assert!(
        cmds.len() >= 8,
        "skill 里至少要演示八条命令，实际 {}",
        cmds.len()
    );
    for (g, v) in &cmds {
        assert!(
            allowed.contains(&(g.clone(), v.clone())),
            "命令不在交付协议 §2：sheltie {g} {v}"
        );
        Bin::cargo_bin("sheltie")
            .unwrap()
            .args([g.as_str(), v.as_str(), "--help"])
            .assert()
            .success();
    }
}

fn install_copy(from: &Path, host: &Path) -> PathBuf {
    let installed = host.join("skills").join("sheltie");
    copy_tree(from, &installed);
    installed
}

/// 在源码树替身里生成一份交付。放在树内 `out/sheltie`，退回仓库相对路径的链接
/// 在树内仍解析得到，才能单独验出「越出交付目录」这一个条件。
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
    for name in ["protocol.md", "workbook.md"] {
        let authority = fs::read_to_string(tree.join("specs/contracts").join(name)).unwrap();
        let generated = fs::read_to_string(dist.join("references").join(name)).unwrap();
        assert_eq!(
            strip_links(&authority),
            strip_links(&generated),
            "{name} 除链接外应与单一权威逐字相同"
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

    // 手写期望：交付文件恰为 SKILL.md 与两份生成 reference
    let files = file_set(&dist);
    let expected: BTreeSet<String> = [
        "SKILL.md".to_string(),
        "references/protocol.md".to_string(),
        "references/workbook.md".to_string(),
    ]
    .into_iter()
    .collect();
    assert_eq!(files, expected);

    // 隔离安装：复制进临时宿主 skill 目录（不是真实 ~/.claude）
    let host = tmp.path().join("host-home");
    let installed = install_copy(&dist, &host);
    assert_delivery_resolves(&installed);

    // 移除源码树、再移动安装位置，重复同一套检查
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

    // 成员表手写期望（目录项不算文件）
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
            "sheltie/references/protocol.md",
            "sheltie/references/workbook.md"
        ]
    );

    // README 的安装方式：解压进 skills 目录后得到一个自包含 skill
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

    // README 的下载地址与发布工作流挂出的资产是同一个名字
    let asset = "sheltie-skill.tar.gz";
    let readme = fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(
        readme.contains(&format!("releases/latest/download/{asset}")),
        "README 应给出该发布资产的下载地址"
    );
    let workflow = fs::read_to_string(repo_root().join(".github/workflows/release.yml")).unwrap();
    assert!(
        workflow.contains(&format!("artifacts/{asset}")),
        "发布工作流应挂出同名资产"
    );
    assert!(
        workflow.contains("scripts/skill-delivery.sh tar"),
        "发布工作流的打包入口应是 skill-delivery.sh tar"
    );
}

// Task: C002-T13
#[test]
fn check_skill_fails_when_delivery_misses_one_reference() {
    let tmp = tempfile::tempdir().unwrap();
    let tree = temp_source_tree(tmp.path());
    let dist = pack_in_tree(&tree);

    // 唯一改变的条件：漏同步 workbook 这一份 reference
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

    // 唯一改变的条件：protocol 这份 reference 内容过期（追加一行）
    let protocol = dist.join("references/protocol.md");
    let mut text = fs::read_to_string(&protocol).unwrap();
    text.push_str("（过期）\n");
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

    // 唯一改变的条件：交付里链接退回仓库相对路径（O11 的缺陷形态）。
    // 交付放在树内 out/sheltie，该路径在树内解析得到，失败点只剩「越出交付目录」。
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
        out_text(&out).contains("越出交付目录"),
        "{}",
        out_text(&out)
    );
}

// Task: C002-T13
#[test]
fn check_skill_fails_when_reference_source_is_gone() {
    let tmp = tempfile::tempdir().unwrap();
    let tree = temp_source_tree(tmp.path());

    // 唯一改变的条件：单一权威缺了 workbook 合同，生成不出那份 reference
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
