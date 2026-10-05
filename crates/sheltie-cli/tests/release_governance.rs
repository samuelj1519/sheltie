//! C002-T15/T17：发布治理门禁、完整历史与版本分流、最终验证表。
//!
//! 夹具自建 change 生命周期与 release records，不继承宿主的发布进度；record 里的
//! commit 与 tag 都指向夹具自己的 Git 历史。只有被测脚本和工作流取自仓库。
//! 期望值（诊断措辞、门禁条件）由本文件写死，不调用 check-specs 的解析逻辑。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::copy_dir;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn write_fixture_file(root: &Path, name: &str, text: &str) {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

#[derive(Clone, Copy)]
enum Lifecycle {
    ActiveC002,
    CompletedC002,
    CompletedUnreleasedC002,
    ActiveExperiment,
}

const FINAL_TABLE_HEADER: &str = "| Requirement / risk | Mode | Input closure | Command / raw run ID | Result | Evidence |\n\
     | --- | --- | --- | --- | --- | --- |\n";

fn final_validation(result: &str) -> String {
    format!(
        "# C002 验证\n\nCandidate: `SELF`\n\n{FINAL_TABLE_HEADER}\
         | 当前要求 | executed | 固定夹具 | `fixture-run` | {result} | raw.log |\n"
    )
}

fn setup_lifecycle(root: &Path, lifecycle: Lifecycle) {
    write_fixture_file(root, "docs/explanation/decisions/mvp.md", "# MVP\n");
    write_fixture_file(root, "docs/history/changes/README.md", "# 历史变更\n");
    for state in ["proposed", "active", "completed", "rejected"] {
        write_fixture_file(root, &format!("specs/changes/{state}/.gitkeep"), "");
    }
    let state = match lifecycle {
        Lifecycle::ActiveC002 => "active",
        Lifecycle::CompletedC002
        | Lifecycle::CompletedUnreleasedC002
        | Lifecycle::ActiveExperiment => "completed",
    };
    let package = format!("specs/changes/{state}/C002-v0.2.0-reliability");
    write_fixture_file(
        root,
        &format!("{package}/README.md"),
        &format!(
            "# C002 夹具\n\n状态：`{state}`\n目标版本：`v0.2.0`\n\
             兼容性：breaking\n基线：`fixture`\nOwner：fixture\n\n## 成功判据\n\n固定要求。\n"
        ),
    );
    write_fixture_file(
        root,
        &format!("{package}/plan.md"),
        "| Task | 状态 | 验收 |\n| --- | --- | --- |\n| C002-T01 | done | 固定要求 |\n",
    );
    write_fixture_file(root, &format!("{package}/progress.md"), "# 交接\n");
    write_fixture_file(root, &format!("{package}/tasks.toml"), "");
    write_fixture_file(
        root,
        &format!("{package}/validation.md"),
        &final_validation("PASS"),
    );
    write_fixture_file(
        root,
        &format!("{package}/review.md"),
        "# 审查\n\n结论：`PASS`\n",
    );
    let active = match lifecycle {
        Lifecycle::ActiveC002 => "Active change：[C002](active/C002-v0.2.0-reliability/README.md)",
        Lifecycle::CompletedC002 | Lifecycle::CompletedUnreleasedC002 => "Active change：无",
        Lifecycle::ActiveExperiment => {
            "Active change：[C007](active/C007-test-experiment/README.md)"
        }
    };
    write_fixture_file(
        root,
        "specs/changes/README.md",
        &format!(
            "# Change 索引\n\n{active}\n\n[C002]({state}/C002-v0.2.0-reliability/README.md)\n"
        ),
    );
    write_fixture_file(root, "docs/explanation/decisions/README.md", "# ADR 索引\n");
    write_fixture_file(
        root,
        "specs/README.md",
        "# 文档地图\n\n开发目标：`v0.2.0`\n",
    );
    if matches!(lifecycle, Lifecycle::ActiveExperiment) {
        let package = "specs/changes/active/C007-test-experiment";
        write_fixture_file(
            root,
            &format!("{package}/README.md"),
            "# C007 夹具\n\n状态：`active`\n目标版本：`不进入产品 release`\n兼容性：不改产品\n基线：`fixture`\nOwner：fixture\n\n## 成功判据\n\n实验记录。\n",
        );
        for file in ["plan.md", "progress.md", "validation.md", "tasks.toml"] {
            write_fixture_file(root, &format!("{package}/{file}"), "");
        }
        let index = root.join("specs/changes/README.md");
        let text = fs::read_to_string(&index).unwrap();
        fs::write(
            index,
            format!("{text}\n[C007](active/C007-test-experiment/README.md)\n"),
        )
        .unwrap();
    }
    let mut release_index = String::from("# Release 索引\n");
    let versions = match lifecycle {
        Lifecycle::ActiveC002
        | Lifecycle::CompletedUnreleasedC002
        | Lifecycle::ActiveExperiment => &["0.1.0"][..],
        Lifecycle::CompletedC002 => &["0.1.0", "0.2.0"][..],
    };
    for version in versions {
        let release = format!("docs/reference/releases/v{version}");
        write_fixture_file(
            root,
            &format!("{release}/README.md"),
            &format!(
                "# v{version}\n\nGit tag：`v{version}`\n\
                 Release commit：`0000000000000000000000000000000000000000`\n\
                 验收闭包：`0000000000000000000000000000000000000000`\n\n\
                 ## 验收\n\n[固定证据](plan.md)\n\n## 已知限制\n\n夹具不发布。\n"
            ),
        );
        for file in ["plan.md", "decisions.md", "runbook.md", "tasks.toml"] {
            write_fixture_file(root, &format!("{release}/{file}"), "");
        }
        release_index.push_str(&format!("\n[v{version}](v{version}/README.md)\n"));
    }
    write_fixture_file(root, "docs/reference/releases/README.md", &release_index);
}

fn copy_governance_tree(base: &Path) -> PathBuf {
    let tree = base.join("gov");
    for name in ["scripts", ".github"] {
        copy_dir(&repo_root().join(name), &tree.join(name));
    }
    fs::create_dir_all(&tree).unwrap();
    for name in [".pre-commit-config.yaml", "AGENTS.md", "Cargo.toml"] {
        fs::copy(repo_root().join(name), tree.join(name)).unwrap();
    }
    write_fixture_file(
        &tree,
        "CHANGELOG.md",
        "# Changelog\n\n---\n\n## [0.1.0] - 2026-09-27\n",
    );
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
    fs::write(
        &path,
        format!("{}{heading}\n\n{}", &text[..at], &text[at..]),
    )
    .unwrap();
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
    for entry in fs::read_dir(root.join("docs/reference/releases")).unwrap() {
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
fn make_fixture(version: &str, changelog_heading: Option<&str>, lifecycle: Lifecycle) -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let root = copy_governance_tree(dir.path());
    setup_lifecycle(&root, lifecycle);
    set_workspace_version(&root, version);
    if let Some(heading) = changelog_heading {
        insert_changelog_heading(&root, heading);
    }
    git_ok(&root, &["init", "-q"]);
    let head1 = git_commit(&root, "fixture: 发布夹具基线");
    rewrite_release_commits(&root, &head1);
    git_commit(&root, "fixture: release record 对齐夹具历史");
    git_ok(&root, &["tag", "v0.1.0", &head1]);
    if matches!(lifecycle, Lifecycle::CompletedC002) {
        git_ok(&root, &["tag", "v0.2.0", &head1]);
    }
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
    let fx = make_fixture("0.1.0", None, Lifecycle::ActiveC002);
    let (ok, text) = run_check_specs(&fx.root);
    assert!(ok, "{text}");
}

// Task: C002-T15
#[test]
fn check_specs_accepts_active_target_without_tag() {
    // 开发中的版本等于 active 目标版本就行，不要求已有 tag（发布还没做）。
    let fx = make_fixture(
        "0.2.0",
        Some("## [0.2.0] - 2026-09-28"),
        Lifecycle::ActiveC002,
    );
    assert_eq!(git_ok(&fx.root, &["tag", "-l", "v0.2.0"]), "");
    let (ok, text) = run_check_specs(&fx.root);
    assert!(ok, "{text}");
}

// Task: C002-T15
#[test]
fn check_specs_accepts_rc_of_active_target() {
    // active 目标版本的 RC 同样按开发中放行，不要求 RC 的 tag。
    let fx = make_fixture(
        "0.2.0-rc.1",
        Some("## [0.2.0-rc.1] - 2026-09-28"),
        Lifecycle::ActiveC002,
    );
    assert_eq!(git_ok(&fx.root, &["tag", "-l", "v0.2.0-rc.1"]), "");
    let (ok, text) = run_check_specs(&fx.root);
    assert!(ok, "{text}");
}

// Task: C006-T05
#[test]
fn check_specs_accepts_authorized_rc_during_nonproduct_experiment() {
    let fx = make_fixture(
        "0.2.0-rc.1",
        Some("## [Unreleased]"),
        Lifecycle::ActiveExperiment,
    );
    assert_eq!(git_ok(&fx.root, &["tag", "-l", "v0.2.0-rc.1"]), "");
    let (ok, text) = run_check_specs(&fx.root);
    assert!(ok, "明确开发目标的RC不因非产品实验被拒绝：{text}");
}

// Task: C006-T05
#[test]
fn check_specs_accepts_authorized_rc_after_product_package_is_completed() {
    let fx = make_fixture(
        "0.2.0-rc.1",
        Some("## [Unreleased]"),
        Lifecycle::CompletedUnreleasedC002,
    );
    assert_eq!(git_ok(&fx.root, &["tag", "-l", "v0.2.0-rc.1"]), "");
    let (ok, text) = run_check_specs(&fx.root);
    assert!(ok, "明确开发目标的RC不因active为空被拒绝：{text}");
}

// Task: C006-T05
#[test]
fn check_specs_rejects_missing_duplicate_or_invalid_development_authority() {
    let mut accepted = Vec::new();
    for declaration in [
        "# 文档地图\n",
        "# 文档地图\n\n开发目标：`v0.2.0`\n开发目标：`v0.2.0`\n",
        "# 文档地图\n\n开发目标：`v0.2`\n",
        "# 文档地图\n\n开发目标：`v00.2.0`\n",
    ] {
        let fx = make_fixture("0.1.0", None, Lifecycle::ActiveC002);
        write_fixture_file(&fx.root, "specs/README.md", declaration);
        let (ok, text) = run_check_specs(&fx.root);
        if ok {
            accepted.push(declaration.to_string());
            continue;
        }
        let failures = failure_lines(&text);
        assert_eq!(failures.len(), 1, "{text}");
        assert!(failures[0].contains("开发目标"), "{text}");
    }
    assert!(accepted.is_empty(), "非法开发目标权威被放行：{accepted:?}");
}

// Task: C006-T05
#[test]
fn check_specs_requires_development_authority_in_first_screen() {
    let fx = make_fixture("0.1.0", None, Lifecycle::ActiveC002);
    write_fixture_file(
        &fx.root,
        "specs/README.md",
        &format!("{}开发目标：`v0.2.0`\n", "\n".repeat(15)),
    );
    let (ok, text) = run_check_specs(&fx.root);
    assert!(!ok, "正文深处字段不能替代首屏权威：{text}");
    assert_eq!(failure_lines(&text).len(), 1, "{text}");
    assert!(text.contains("开发目标"), "{text}");
}

// Task: C006-T05
#[test]
fn check_specs_rejects_numeric_active_conflict_even_when_cargo_version_is_released() {
    let fx = make_fixture("0.1.0", None, Lifecycle::ActiveC002);
    write_fixture_file(
        &fx.root,
        "specs/README.md",
        "# 文档地图\n\n开发目标：`v0.3.0`\n",
    );
    let (ok, text) = run_check_specs(&fx.root);
    assert!(!ok, "已发布Cargo版本不能掩盖active目标冲突：{text}");
    assert_eq!(failure_lines(&text).len(), 1, "{text}");
    assert!(
        text.contains("active 目标版本") && text.contains("开发目标"),
        "{text}"
    );
}

// Task: C006-T05
#[test]
fn check_specs_rejects_unknown_nonproduct_target_even_when_cargo_version_is_released() {
    let fx = make_fixture("0.1.0", None, Lifecycle::ActiveExperiment);
    let path = fx
        .root
        .join("specs/changes/active/C007-test-experiment/README.md");
    let text = fs::read_to_string(&path).unwrap();
    fs::write(path, text.replace("`不进入产品 release`", "`实验待定`")).unwrap();
    let (ok, text) = run_check_specs(&fx.root);
    assert!(!ok, "未知实验目标不能回退放行：{text}");
    assert_eq!(failure_lines(&text).len(), 1, "{text}");
    assert!(text.contains("active 目标版本"), "{text}");
}

// Task: C006-T05
#[test]
fn check_specs_rejects_rc_outside_authority_during_nonproduct_experiment() {
    let fx = make_fixture(
        "9.9.9-rc.1",
        Some("## [Unreleased]"),
        Lifecycle::ActiveExperiment,
    );
    let (ok, text) = run_check_specs(&fx.root);
    assert!(!ok, "非产品实验不能放行其他开发线：{text}");
    assert_eq!(failure_lines(&text).len(), 1, "{text}");
    assert!(
        text.contains("开发目标") && text.contains("active 目标版本"),
        "{text}"
    );
}

// Task: C006-T05
#[test]
fn check_specs_authorized_rc_without_active_still_requires_changelog() {
    let fx = make_fixture("0.2.0-rc.1", None, Lifecycle::CompletedUnreleasedC002);
    let (ok, text) = run_check_specs(&fx.root);
    assert!(!ok, "无active仍不能省略开发候选CHANGELOG：{text}");
    assert_eq!(failure_lines(&text).len(), 1, "{text}");
    assert!(text.contains("CHANGELOG"), "{text}");
}

// Task: C002-T15
#[test]
fn check_specs_rejects_version_outside_active_target() {
    // 反例只改一个条件：版本从 active 目标 0.2.0 换成 9.9.9（照样有自己的
    // CHANGELOG 段）——既不是目标版本也不是它的 RC。
    let fx = make_fixture(
        "9.9.9",
        Some("## [9.9.9] - 2026-09-28"),
        Lifecycle::ActiveC002,
    );
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
    let fx = make_fixture("0.2.0", None, Lifecycle::ActiveC002);
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
    let fx = make_fixture("0.1.0", None, Lifecycle::ActiveC002);
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
    let fx = make_fixture("0.1.0", None, Lifecycle::ActiveC002);
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

// Task: C002-T17
#[test]
fn release_workflow_gates_announce_on_quality_job() {
    let release = workflow("release.yml");
    let plan = job_block(&release, "plan");
    let quality = job_block(&release, "quality");
    let candidate = "${{ github.event.pull_request.head.sha || github.sha }}";
    assert!(
        plan.contains(&format!("candidate-sha: {candidate}")),
        "{plan}"
    );
    assert!(plan.contains(&format!("ref: {candidate}")), "{plan}");
    assert!(quality.contains(&format!("ref: {candidate}")), "{quality}");
    assert!(
        quality.contains(&format!("CANDIDATE_SHA: {candidate}")),
        "{quality}"
    );
    assert!(quality.contains("fetch-depth: 0"), "{quality}");
    assert!(
        quality.contains(r#"test "$(git rev-parse HEAD)" = "$CANDIDATE_SHA""#),
        "{quality}"
    );
    assert!(
        quality.contains("cargo +1.85.0 check --workspace --all-targets --all-features --locked"),
        "{quality}"
    );
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

// Task: C002-T17
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
    let announce = job_block(&release, "announce");
    assert!(
        announce.contains(r#"RELEASE_COMMIT: "${{ needs.plan.outputs.candidate-sha }}""#),
        "{announce}"
    );
}

fn completed_fixture() -> Fixture {
    make_fixture(
        "0.2.0",
        Some("## [0.2.0] - 2026-10-02"),
        Lifecycle::CompletedC002,
    )
}

fn set_completed_validation(fx: &Fixture, text: &str) -> PathBuf {
    let path = fx
        .root
        .join("specs/changes/completed/C002-v0.2.0-reliability/validation.md");
    fs::write(&path, text).unwrap();
    path
}

fn compact_completed(fx: &Fixture, snapshot: &str) {
    let package = fx
        .root
        .join("specs/changes/completed/C002-v0.2.0-reliability");
    let readme = package.join("README.md");
    let original = fs::read_to_string(&readme).unwrap();
    fs::write(
        &readme,
        format!(
            "{original}\n记录形式：`reference`\n历史快照：`{snapshot}`\n\n\
             ## 变化与理由\n\n固定参考。\n\n## 验证与限制\n\n固定范围。\n\n\
             ## 参考\n\n固定入口。\n"
        ),
    )
    .unwrap();
    for file in ["plan.md", "validation.md", "review.md", "tasks.toml"] {
        fs::remove_file(package.join(file)).unwrap();
    }
    let archive = fx.root.join("docs/history/changes/C002-v0.2.0-reliability");
    fs::rename(package, archive).unwrap();
    write_fixture_file(
        &fx.root,
        "docs/history/changes/README.md",
        "# 历史变更\n\n[C002](C002-v0.2.0-reliability/README.md)\n",
    );
}

// Task: C002-T17
#[test]
fn check_specs_reference_rechecks_completed_snapshot_without_loose_records() {
    let fx = completed_fixture();
    let snapshot = git_ok(&fx.root, &["rev-parse", "HEAD"]);
    compact_completed(&fx, &snapshot);
    let (ok, text) = run_check_specs(&fx.root);
    assert!(ok, "{text}");
    assert!(
        !fx.root
            .join("specs/changes/completed/C002-v0.2.0-reliability/validation.md")
            .exists()
    );
}

// Task: C002-T17
#[test]
fn check_specs_reference_rejects_missing_snapshot_history() {
    let fx = completed_fixture();
    compact_completed(&fx, "1111111111111111111111111111111111111111");
    let (ok, text) = run_check_specs(&fx.root);
    assert!(!ok, "缺历史不能默认为通过：{text}");
    assert!(text.contains("不能核原完成记录"), "{text}");
}

// Task: C002-T17
#[test]
fn check_specs_reference_rejects_failed_qualification_in_snapshot() {
    let fx = completed_fixture();
    set_completed_validation(&fx, &final_validation("FAIL"));
    let snapshot = git_commit(&fx.root, "fixture: 最终验证失败");
    compact_completed(&fx, &snapshot);
    let (ok, text) = run_check_specs(&fx.root);
    assert!(!ok, "摘要不能掩盖最终 FAIL：{text}");
    assert!(text.contains("非 PASS"), "{text}");
}

// Task: C002-T17
#[test]
fn check_specs_reference_rejects_snapshot_missing_original_records() {
    let fx = completed_fixture();
    fs::remove_file(
        fx.root
            .join("specs/changes/completed/C002-v0.2.0-reliability/review.md"),
    )
    .unwrap();
    let snapshot = git_commit(&fx.root, "fixture: 缺独立审查");
    fs::write(
        fx.root
            .join("specs/changes/completed/C002-v0.2.0-reliability/review.md"),
        "placeholder",
    )
    .unwrap();
    compact_completed(&fx, &snapshot);
    let (ok, text) = run_check_specs(&fx.root);
    assert!(!ok, "缺原审查不能通过：{text}");
    assert!(text.contains("快照缺 review.md"), "{text}");
}

// Task: C002-T17
#[test]
fn check_specs_reference_rejects_snapshot_of_another_reference() {
    let fx = completed_fixture();
    let package = fx
        .root
        .join("specs/changes/completed/C002-v0.2.0-reliability/README.md");
    let original = fs::read_to_string(&package).unwrap();
    fs::write(&package, format!("{original}\n记录形式：`reference`\n")).unwrap();
    let snapshot = git_commit(&fx.root, "fixture: 另一份摘要");
    fs::write(&package, original).unwrap();
    compact_completed(&fx, &snapshot);
    let (ok, text) = run_check_specs(&fx.root);
    assert!(!ok, "摘要不能代替完整原件：{text}");
    assert!(text.contains("不能再指向摘要"), "{text}");
}

// Task: C002-T17
#[test]
fn check_specs_completed_accepts_final_pass_without_active_change() {
    let fx = completed_fixture();
    let (ok, text) = run_check_specs(&fx.root);
    assert!(ok, "{text}");
    assert!(text.contains("0 个 active"), "{text}");
}

// Task: C002-T17
#[test]
fn check_specs_completed_accepts_final_pass_and_preserves_historical_failures() {
    let fx = completed_fixture();
    let history = "\n## 历史事实\n\n\
        | 身份 | 固定值 |\n| --- | --- |\n| 旧候选 | abcdef0 |\n\n\
        | Requirement / risk | Mode | Input closure | Command / raw run ID | 历史结果 | Evidence |\n\
        | --- | --- | --- | --- | --- | --- |\n\
        | 早期失败 | executed | 旧候选 | `old-run` | FAIL | old.log |\n\
        | 实际跳过 | not_run | 旧候选 | none | SKIP | skip.json |\n";
    let validation = format!("{}{}", final_validation("PASS"), history);
    let path = set_completed_validation(&fx, &validation);
    let (ok, text) = run_check_specs(&fx.root);
    assert!(ok, "{text}");
    assert_eq!(fs::read_to_string(path).unwrap(), validation);
}

// Task: C002-T17
#[test]
fn check_specs_completed_rejects_current_fail_even_with_historical_pass() {
    let fx = completed_fixture();
    let validation = format!(
        "{}\n| 历史要求 | 模式 | 输入 | 命令 | 旧结果 | 证据 |\n\
         | --- | --- | --- | --- | --- | --- |\n\
         | 旧运行 | executed | old | old-run | PASS | old.log |\n",
        final_validation("FAIL")
    );
    set_completed_validation(&fx, &validation);
    let (ok, text) = run_check_specs(&fx.root);
    assert!(!ok, "当前 FAIL 不该放行：{text}");
    assert_eq!(failure_lines(&text).len(), 1, "{text}");
    assert!(text.contains("非 PASS"), "{text}");
}

// Task: C002-T17
#[test]
fn check_specs_completed_rejects_empty_final_table() {
    let fx = completed_fixture();
    set_completed_validation(&fx, &format!("Candidate: `SELF`\n\n{FINAL_TABLE_HEADER}"));
    let (ok, text) = run_check_specs(&fx.root);
    assert!(!ok, "最终表没有数据不该放行：{text}");
    assert_eq!(failure_lines(&text).len(), 1, "{text}");
    assert!(text.contains("没有数据行"), "{text}");
}

// Task: C002-T17
#[test]
fn check_specs_completed_requires_exact_final_table_header() {
    let fx = completed_fixture();
    for (from, to) in [
        ("Input closure", "旧输入"),
        ("| Result | Evidence |", "| Result | Evidence | extra"),
    ] {
        let validation = final_validation("PASS").replace(from, to);
        set_completed_validation(&fx, &validation);
        let (ok, text) = run_check_specs(&fx.root);
        assert!(!ok, "相似历史表不能充当最终表：{to}\n{text}");
        assert_eq!(failure_lines(&text).len(), 1, "{text}");
        assert!(text.contains("缺最终验证表"), "{text}");
    }
}

// Task: C002-T17
#[test]
fn check_specs_completed_rejects_validation_without_final_table() {
    let fx = completed_fixture();
    set_completed_validation(&fx, "# 验证\n\nCandidate: `SELF`\n\n没有运行。\n");
    let (ok, text) = run_check_specs(&fx.root);
    assert!(!ok, "没有最终表不该放行：{text}");
    assert_eq!(failure_lines(&text).len(), 1, "{text}");
    assert!(text.contains("缺最终验证表"), "{text}");
}

// Task: C002-T17
#[test]
fn check_specs_completed_checks_every_final_table() {
    let fx = completed_fixture();
    let validation = format!(
        "{}\n## 第二项\n\n{}",
        final_validation("PASS"),
        final_validation("FAIL")
    );
    set_completed_validation(&fx, &validation);
    let (ok, text) = run_check_specs(&fx.root);
    assert!(!ok, "另一最终表 FAIL 不该放行：{text}");
    assert_eq!(failure_lines(&text).len(), 1, "{text}");
    assert!(text.contains("非 PASS"), "{text}");
}

// Task: C002-T17
#[test]
fn check_specs_completed_rejects_empty_final_table_beside_nonempty_table() {
    let fx = completed_fixture();
    set_completed_validation(
        &fx,
        &format!(
            "{}\n## 第二项\n\n{FINAL_TABLE_HEADER}",
            final_validation("PASS")
        ),
    );
    let (ok, text) = run_check_specs(&fx.root);
    assert!(!ok, "空的第二张最终表不该放行：{text}");
    assert_eq!(failure_lines(&text).len(), 1, "{text}");
    assert!(text.contains("没有数据行"), "{text}");
}

// Task: C002-T17
#[test]
fn check_specs_completed_rejects_empty_fields_in_final_rows() {
    let fx = completed_fixture();
    for row in [
        "|   | executed | closure | run | PASS | raw |",
        "| current |   | closure | run | PASS | raw |",
        "| current | executed |   | run | PASS | raw |",
        "| current | executed | closure |   | PASS | raw |",
        "| current | executed | closure | run |   | raw |",
        "| current | executed | closure | run | PASS |   |",
        "|   |   |   |   | PASS |   |",
    ] {
        set_completed_validation(
            &fx,
            &format!("Candidate: `SELF`\n\n{FINAL_TABLE_HEADER}{row}\n"),
        );
        let (ok, text) = run_check_specs(&fx.root);
        assert!(!ok, "最终行空字段不该放行：{row}\n{text}");
        assert_eq!(failure_lines(&text).len(), 1, "{text}");
        assert!(text.contains("空字段"), "{row}\n{text}");
    }
}

// Task: C002-T17
#[test]
fn check_specs_completed_rejects_more_than_six_columns_in_final_rows() {
    let fx = completed_fixture();
    for row in [
        "| current | executed | closure | run | PASS | raw | FAIL |",
        "| current | executed | closure | run | PASS | raw | FAIL",
    ] {
        set_completed_validation(
            &fx,
            &format!("Candidate: `SELF`\n\n{FINAL_TABLE_HEADER}{row}\n"),
        );
        let (ok, text) = run_check_specs(&fx.root);
        assert!(!ok, "多栏最终行不该放行：{row}\n{text}");
        assert_eq!(failure_lines(&text).len(), 1, "{text}");
        assert!(text.contains("六列"), "{text}");
    }
}

// Task: C002-T17
#[test]
fn check_specs_completed_rejects_fewer_than_six_columns_in_final_rows() {
    let fx = completed_fixture();
    set_completed_validation(
        &fx,
        &format!(
            "Candidate: `SELF`\n\n{FINAL_TABLE_HEADER}\
             | current | executed | closure | run | PASS |\n"
        ),
    );
    let (ok, text) = run_check_specs(&fx.root);
    assert!(!ok, "少栏最终行不该放行：{text}");
    assert_eq!(failure_lines(&text).len(), 1, "{text}");
    assert!(text.contains("六列"), "{text}");
}

// Task: C002-T17
#[test]
fn check_specs_completed_rejects_extra_columns_in_final_separator() {
    let fx = completed_fixture();
    for separator in [
        "| --- | --- | --- | --- | --- | --- | extra |",
        "| --- | --- | --- | --- | --- | --- | extra",
    ] {
        let validation =
            final_validation("PASS").replace("| --- | --- | --- | --- | --- | --- |", separator);
        set_completed_validation(&fx, &validation);
        let (ok, text) = run_check_specs(&fx.root);
        assert!(!ok, "分隔行多栏不该放行：{separator}\n{text}");
        assert_eq!(failure_lines(&text).len(), 1, "{text}");
        assert!(text.contains("六列"), "{text}");
    }
}
