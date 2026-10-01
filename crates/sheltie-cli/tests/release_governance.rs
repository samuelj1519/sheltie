//! C002-T15：发布治理门禁——check-specs 的历史诊断与版本分流，release/build
//! 工作流的质量门禁接线。
//!
//! 治理夹具把仓库的治理树拷进临时目录并自建 Git 历史：release record 里的 40 位
//! commit 全部改写成夹具自己的 commit，tag 也由夹具创建，record、tag 与历史自洽。
//! 期望值（诊断措辞、门禁条件）由本文件写死，不调用 check-specs 的解析逻辑。
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn copy_tree(src: &Path, dst: &Path, relative: &Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let from = entry.path();
        let to = dst.join(entry.file_name());
        let relative = relative.join(entry.file_name());
        // 版本/历史夹具保留完整Markdown扫描输入，不复制与治理无关的原始运行产物。
        if relative.starts_with("specs")
            && relative
                .components()
                .any(|part| part.as_os_str() == "evidence")
            && from.is_file()
            && from.extension() != Some(std::ffi::OsStr::new("md"))
        {
            continue;
        }
        if from.is_dir() {
            copy_tree(&from, &to, &relative);
        } else {
            fs::copy(&from, &to).unwrap();
        }
    }
}

/// 治理树替身：check-specs 与它引用的仓库文件同构，所有改动都发生在替身里。
fn copy_governance_tree(base: &Path) -> PathBuf {
    let tree = base.join("gov");
    for name in ["scripts", "specs", ".github"] {
        copy_tree(&repo_root().join(name), &tree.join(name), Path::new(name));
    }
    fs::create_dir_all(&tree).unwrap();
    for name in [
        ".pre-commit-config.yaml",
        "AGENTS.md",
        "CHANGELOG.md",
        "Cargo.toml",
    ] {
        fs::copy(repo_root().join(name), tree.join(name)).unwrap();
    }
    tree
}

/// git 与 check-specs 都要和宿主仓库的 Git 状态隔离（T12 的教训：夹具 git 被
/// 指进宿主索引）；全局/系统配置也关掉，夹具自己带身份。
fn scrub_git_env(cmd: &mut Command) {
    for key in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_COMMON_DIR",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    ] {
        cmd.env_remove(key);
    }
    cmd.env("GIT_CONFIG_GLOBAL", "/dev/null");
    cmd.env("GIT_CONFIG_SYSTEM", "/dev/null");
}

fn git_cmd(root: &Path, args: &[&str]) -> Command {
    let mut cmd = Command::new("git");
    cmd.current_dir(root).args(args);
    scrub_git_env(&mut cmd);
    cmd
}

fn git_ok(root: &Path, args: &[&str]) -> String {
    let out = git_cmd(root, args).output().unwrap();
    assert!(
        out.status.success(),
        "git {args:?} 失败：{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn git_commit(root: &Path, message: &str) -> String {
    git_ok(root, &["add", "-A"]);
    git_ok(
        root,
        &[
            "-c",
            "user.name=fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "-q",
            "-m",
            message,
        ],
    );
    git_ok(root, &["rev-parse", "HEAD"])
}

/// 只改 `[workspace.package]` 段的 `version`，依赖版本一个不碰。
fn set_workspace_version(root: &Path, version: &str) {
    let path = root.join("Cargo.toml");
    let text = fs::read_to_string(&path).unwrap();
    let mut out = String::new();
    let mut inside = false;
    let mut replaced = false;
    for line in text.lines() {
        if line.starts_with('[') {
            inside = line.trim() == "[workspace.package]";
        }
        if inside && !replaced && line.starts_with("version = ") {
            out.push_str(&format!("version = \"{version}\"\n"));
            replaced = true;
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    assert!(
        replaced,
        "Cargo.toml 的 [workspace.package] 没有 version 行"
    );
    fs::write(&path, out).unwrap();
}

/// 在 CHANGELOG 的 `---` 标记后插一段版本记录。
fn insert_changelog_heading(root: &Path, heading: &str) {
    let path = root.join("CHANGELOG.md");
    let text = fs::read_to_string(&path).unwrap();
    let marker = "\n---\n";
    let idx = text.find(marker).expect("CHANGELOG.md 没有 --- 标记");
    let at = idx + marker.len();
    fs::write(&path, format!("{}{heading}\n\n", &text[..at])).unwrap();
}

fn push_run(out: &mut String, run: &str, to: &str) {
    // 恰好 40 位的十六进制串才是 record 记的 commit；短 hash（如 `31d7dde`）不动。
    if run.len() == 40 {
        out.push_str(to);
    } else {
        out.push_str(run);
    }
}

/// 把 release record 里的 40 位 commit 全改写成 `to`。
fn rewrite_commits(text: &str, to: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut run = String::new();
    for ch in text.chars() {
        if ch.is_ascii_hexdigit() {
            run.push(ch);
            continue;
        }
        push_run(&mut out, &run, to);
        run.clear();
        out.push(ch);
    }
    push_run(&mut out, &run, to);
    out
}

fn rewrite_release_commits(root: &Path, to: &str) {
    for entry in fs::read_dir(root.join("specs/releases")).unwrap() {
        let readme = entry.unwrap().path().join("README.md");
        if !readme.is_file() {
            continue;
        }
        fs::write(
            &readme,
            rewrite_commits(&fs::read_to_string(&readme).unwrap(), to),
        )
        .unwrap();
    }
}

struct Fixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
}

/// 治理夹具：拷治理树 →（可选）版本/CHANGELOG 改动 → 建自洽 Git 历史。
///
/// 历史形状：commit1 是 record 记的「发布 commit」，commit2 装着 record 对齐
/// 改写，`v0.1.0` tag 指 commit1。浅克隆 tip（commit2）时 tag 与 commit1 都
/// 取不到，正好落在 check-specs 的缺历史分支上。
fn make_fixture(version: &str, changelog_heading: Option<&str>) -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let root = copy_governance_tree(dir.path());
    set_workspace_version(&root, version);
    if let Some(heading) = changelog_heading {
        insert_changelog_heading(&root, heading);
    }
    git_ok(&root, &["init", "-q"]);
    let head1 = git_commit(&root, "fixture: 发布夹具基线");
    rewrite_release_commits(&root, &head1);
    git_commit(&root, "fixture: release record 对齐夹具历史");
    git_ok(&root, &["tag", "v0.1.0", &head1]);
    Fixture { _dir: dir, root }
}

/// 浅克隆替身：只剩 tip commit，老 tag 与 record 记的老 commit 都不在。
fn shallow_clone(fx: &Fixture) -> PathBuf {
    let dst = fx.root.parent().unwrap().join("shallow");
    let url = format!("file://{}", fx.root.canonicalize().unwrap().display());
    let mut cmd = Command::new("git");
    cmd.args(["clone", "-q", "--depth", "1", &url]).arg(&dst);
    scrub_git_env(&mut cmd);
    let out = cmd.output().unwrap();
    assert!(
        out.status.success(),
        "浅克隆失败：{}",
        String::from_utf8_lossy(&out.stderr)
    );
    dst
}

fn run_check_specs(root: &Path) -> (bool, String) {
    let mut cmd = Command::new("bash");
    cmd.arg(root.join("scripts/check-specs.sh"));
    scrub_git_env(&mut cmd);
    let out = cmd.output().unwrap();
    (
        out.status.success(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
    )
}

fn failure_lines(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|line| line.starts_with("check-specs: "))
        .collect()
}

fn workflow(name: &str) -> String {
    fs::read_to_string(repo_root().join(".github/workflows").join(name)).unwrap()
}

/// 按 2 空格缩进的 job 头切出整个 job 块。
fn job_block(workflow: &str, job: &str) -> String {
    let header = format!("  {job}:");
    let mut out = String::new();
    let mut inside = false;
    for line in workflow.lines() {
        if !inside {
            if line == header {
                inside = true;
                out.push_str(line);
                out.push('\n');
            }
            continue;
        }
        if line.starts_with("  ") && !line.starts_with("   ") && line.ends_with(':') {
            break;
        }
        out.push_str(line);
        out.push('\n');
    }
    assert!(inside, "workflow 里没有 job {job}");
    out
}

/// announce 是否真被质量门禁挡住：needs 里有 quality，if 里还要 quality 成功。
/// 少任一条件，质量失败都还能建 Release。
fn announce_needs_quality(announce: &str) -> bool {
    announce.contains("      - quality") && announce.contains("needs.quality.result == 'success'")
}

// Task: C002-T15
#[test]
fn check_specs_full_history_resolves_release_commits() {
    // 全历史、record 对齐夹具的 tag 与 commit：治理检查整体放行。
    let fx = make_fixture("0.1.0", None);
    let (ok, text) = run_check_specs(&fx.root);
    assert!(ok, "{text}");
}

// Task: C002-T15
#[test]
fn check_specs_accepts_active_target_without_tag() {
    // 开发中的版本等于 active 目标版本就行，不要求已有 tag（发布还没做）。
    let fx = make_fixture("0.2.0", Some("## [0.2.0] - 2026-09-28"));
    assert_eq!(git_ok(&fx.root, &["tag", "-l", "v0.2.0"]), "");
    let (ok, text) = run_check_specs(&fx.root);
    assert!(ok, "{text}");
}

// Task: C002-T15
#[test]
fn check_specs_accepts_rc_of_active_target() {
    // active 目标版本的 RC 同样按开发中放行，不要求 RC 的 tag。
    let fx = make_fixture("0.2.0-rc.1", Some("## [0.2.0-rc.1] - 2026-09-28"));
    assert_eq!(git_ok(&fx.root, &["tag", "-l", "v0.2.0-rc.1"]), "");
    let (ok, text) = run_check_specs(&fx.root);
    assert!(ok, "{text}");
}

// Task: C002-T15
#[test]
fn check_specs_rejects_version_outside_active_target() {
    // 反例只改一个条件：版本从 active 目标 0.2.0 换成 9.9.9（照样有自己的
    // CHANGELOG 段）——既不是目标版本也不是它的 RC。
    let fx = make_fixture("9.9.9", Some("## [9.9.9] - 2026-09-28"));
    let (ok, text) = run_check_specs(&fx.root);
    assert!(!ok, "版本 9.9.9 不该被放行：{text}");
    let fails = failure_lines(&text);
    assert_eq!(fails.len(), 1, "{text}");
    assert!(fails[0].contains("active 目标版本"), "{text}");
}

// Task: C002-T15
#[test]
fn check_specs_dev_version_requires_changelog_section() {
    // 反例只改一个条件：相对「接受 active 目标」去掉 CHANGELOG 的版本段。
    let fx = make_fixture("0.2.0", None);
    let (ok, text) = run_check_specs(&fx.root);
    assert!(!ok, "没写 CHANGELOG 版本段不该被放行：{text}");
    let fails = failure_lines(&text);
    assert_eq!(fails.len(), 1, "{text}");
    assert!(fails[0].contains("CHANGELOG"), "{text}");
}

// Task: C002-T15
#[test]
fn check_specs_shallow_clone_reports_missing_history() {
    // 浅克隆只剩 tip：老 tag 与 record 记的老 commit 都取不到，诊断必须点名
    // 缺历史，不能说成 record 写错。
    let fx = make_fixture("0.1.0", None);
    let shallow = shallow_clone(&fx);
    let (ok, text) = run_check_specs(&shallow);
    assert!(!ok, "{text}");
    assert!(text.contains("缺历史"), "{text}");
    assert!(text.contains("浅克隆"), "{text}");
    let fails = failure_lines(&text);
    assert!(!fails.is_empty(), "{text}");
    for line in fails {
        assert!(line.contains("缺历史"), "缺历史之外还报了别的：{text}");
    }
}

// Task: C002-T15
#[test]
fn check_specs_missing_tag_without_shallow_reports_not_created() {
    // 反例只删 tag（历史完整）：诊断是「未创建或未推送」，不能说成缺历史。
    let fx = make_fixture("0.1.0", None);
    git_ok(&fx.root, &["tag", "-d", "v0.1.0"]);
    let (ok, text) = run_check_specs(&fx.root);
    assert!(!ok, "{text}");
    assert!(!text.contains("浅克隆"), "{text}");
    let fails = failure_lines(&text);
    assert!(!fails.is_empty(), "{text}");
    for line in fails {
        assert!(line.contains("未创建"), "诊断没说清是 tag 没建：{text}");
    }
}

// Task: C002-T15
#[test]
fn release_workflow_gates_announce_on_quality_job() {
    // 发布挂在同一 SHA 的质量 job：quality 验 GITHUB_SHA、拉全历史、跑全套
    // 门禁与 MSRV，announce 只在 quality 成功时建 Release。
    let release = workflow("release.yml");
    let quality = job_block(&release, "quality");
    assert!(quality.contains("fetch-depth: 0"), "{quality}");
    assert!(
        quality.contains(r#"test "$(git rev-parse HEAD)" = "$GITHUB_SHA""#),
        "{quality}"
    );
    assert!(
        quality.contains("cargo +1.85.0 check --workspace --all-targets --all-features --locked"),
        "{quality}"
    );
    let announce = job_block(&release, "announce");
    assert!(announce_needs_quality(&announce), "{announce}");
}

// Task: C002-T15
#[test]
fn release_workflow_quality_failure_blocks_announce() {
    // 两个单条件反例：needs 少 quality、或 if 少 quality 成功条件，
    // 挡住发布的谓词就翻——少任一条件质量失败都能建 Release。
    let announce = job_block(&workflow("release.yml"), "announce");
    assert!(announce_needs_quality(&announce), "{announce}");

    let no_needs = announce.replace("      - quality\n", "");
    assert_ne!(no_needs, announce);
    assert!(!announce_needs_quality(&no_needs), "{no_needs}");

    let no_if = announce.replace(" && needs.quality.result == 'success'", "");
    assert_ne!(no_if, announce);
    assert!(!announce_needs_quality(&no_if), "{no_if}");
}

// Task: C002-T15
#[test]
fn build_workflow_fetches_history_and_runs_msrv_gate() {
    // 治理 job 拉全历史（浅检出取不到老 tag）；MSRV 1.85 locked 门禁真的跑，
    // stable 通过不能代替它；announce 建 Release 用的是同一个 SHA。
    let build = workflow("build.yml");
    let docs = job_block(&build, "docs");
    assert!(docs.contains("fetch-depth: 0"), "{docs}");
    let msrv = job_block(&build, "msrv");
    assert!(
        msrv.contains("cargo +1.85.0 check --workspace --all-targets --all-features --locked"),
        "{msrv}"
    );

    let release = workflow("release.yml");
    let quality = job_block(&release, "quality");
    assert!(
        quality.contains("cargo +1.85.0 check --workspace --all-targets --all-features --locked"),
        "{quality}"
    );
    let announce = job_block(&release, "announce");
    assert!(
        announce.contains(r#"RELEASE_COMMIT: "${{ github.sha }}""#),
        "{announce}"
    );
}
