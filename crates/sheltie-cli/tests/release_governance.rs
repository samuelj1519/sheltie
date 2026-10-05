//! C002-T15/T17: release gates, complete history, version routing, and final validation tables.
//!
//! Fixtures construct change lifecycles and release records without inheriting host release progress;
//! commits/tags refer to fixture-owned history. Only scripts and workflows come from the repository.
//! Handwritten diagnostics and gate expectations do not call check-specs parsing logic.
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
        "# C002 validation\n\nCandidate: `SELF`\n\n{FINAL_TABLE_HEADER}\
         | Current requirement | executed | Fixed fixture | `fixture-run` | {result} | raw.log |\n"
    )
}

fn setup_lifecycle(root: &Path, lifecycle: Lifecycle) {
    write_fixture_file(root, "docs/en/explanation/decisions/mvp.md", "# MVP\n");
    write_fixture_file(
        root,
        "docs/en/history/changes/README.md",
        "# Historical changes\n",
    );
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
            "# C002 fixture\n\nStatus: `{state}`\nTarget version: `v0.2.0`\n\
             Compatibility: breaking\nBaseline: `fixture`\nOwner: fixture\n\n## Success criteria\n\nFixed requirement.\n"
        ),
    );
    write_fixture_file(
        root,
        &format!("{package}/plan.md"),
        "| Task | State | Acceptance |\n| --- | --- | --- |\n| C002-T01 | done | Fixed requirement |\n",
    );
    write_fixture_file(root, &format!("{package}/progress.md"), "# Handoff\n");
    write_fixture_file(root, &format!("{package}/tasks.toml"), "");
    write_fixture_file(
        root,
        &format!("{package}/validation.md"),
        &final_validation("PASS"),
    );
    write_fixture_file(
        root,
        &format!("{package}/review.md"),
        "# Review\n\nConclusion: `PASS`\n",
    );
    let active = match lifecycle {
        Lifecycle::ActiveC002 => "Active change: [C002](active/C002-v0.2.0-reliability/README.md)",
        Lifecycle::CompletedC002 | Lifecycle::CompletedUnreleasedC002 => "Active change: none",
        Lifecycle::ActiveExperiment => {
            "Active change: [C007](active/C007-test-experiment/README.md)"
        }
    };
    write_fixture_file(
        root,
        "specs/changes/README.md",
        &format!(
            "# Change index\n\n{active}\n\n[C002]({state}/C002-v0.2.0-reliability/README.md)\n"
        ),
    );
    write_fixture_file(
        root,
        "docs/en/explanation/decisions/README.md",
        "# ADR index\n",
    );
    write_fixture_file(
        root,
        "specs/README.md",
        "# Documentation map\n\nDevelopment target: `v0.2.0`\n",
    );
    if matches!(lifecycle, Lifecycle::ActiveExperiment) {
        let package = "specs/changes/active/C007-test-experiment";
        write_fixture_file(
            root,
            &format!("{package}/README.md"),
            "# C007 fixture\n\nStatus: `active`\nTarget version: `not included in a product release`\nCompatibility: No product changes\nBaseline: `fixture`\nOwner: fixture\n\n## Success criteria\n\nExperiment record.\n",
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
    let mut release_index = String::from("# Release index\n");
    let versions = match lifecycle {
        Lifecycle::ActiveC002
        | Lifecycle::CompletedUnreleasedC002
        | Lifecycle::ActiveExperiment => &["0.1.0"][..],
        Lifecycle::CompletedC002 => &["0.1.0", "0.2.0"][..],
    };
    for version in versions {
        let release = format!("docs/en/reference/releases/v{version}");
        write_fixture_file(
            root,
            &format!("{release}/README.md"),
            &format!(
                "# v{version}\n\nGit tag: `v{version}`\n\
                 Release commit: `0000000000000000000000000000000000000000`\n\
                 Acceptance closure: `0000000000000000000000000000000000000000`\n\n\
                 ## Acceptance\n\n[Fixed evidence](plan.md)\n\n## Known limits\n\nFixture does not publish.\n"
            ),
        );
        for file in ["plan.md", "decisions.md", "runbook.md", "tasks.toml"] {
            write_fixture_file(root, &format!("{release}/{file}"), "");
        }
        release_index.push_str(&format!("\n[v{version}](v{version}/README.md)\n"));
    }
    write_fixture_file(root, "docs/en/reference/releases/README.md", &release_index);
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

/// Isolate git/check-specs from host Git state (T12 lesson: fixture Git accidentally used
/// the host index); disable global/system configuration and use fixture-owned identity.
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
        "git {args:?} failed: {}",
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

/// Change only workspace.package version, preserving dependency versions.
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
    assert!(replaced, "Cargo.toml workspace.package has no version line");
    fs::write(&path, out).unwrap();
}

/// Insert a version entry after CHANGELOG's --- marker.
fn insert_changelog_heading(root: &Path, heading: &str) {
    let path = root.join("CHANGELOG.md");
    let text = fs::read_to_string(&path).unwrap();
    let marker = "\n---\n";
    let idx = text.find(marker).expect("CHANGELOG.md has no --- marker");
    let at = idx + marker.len();
    fs::write(
        &path,
        format!("{}{heading}\n\n{}", &text[..at], &text[at..]),
    )
    .unwrap();
}

fn push_run(out: &mut String, run: &str, to: &str) {
    // Only exact 40-digit hexadecimal commits are rewritten; preserve short hashes such as 7196697.
    if run.len() == 40 {
        out.push_str(to);
    } else {
        out.push_str(run);
    }
}

/// Rewrite every 40-digit release-record commit to to.
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
    for entry in fs::read_dir(root.join("docs/en/reference/releases")).unwrap() {
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

/// Governance fixture: copy governance tree, optionally change version/CHANGELOG, and create consistent Git history.
///
/// History shape: commit1 is the recorded release commit; commit2 aligns records
/// to fixture history, with v0.1.0 tagging commit1. A shallow commit2 checkout lacks both tag and commit1,
/// exercising check-specs' incomplete-history branch.
fn make_fixture(version: &str, changelog_heading: Option<&str>, lifecycle: Lifecycle) -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let root = copy_governance_tree(dir.path());
    setup_lifecycle(&root, lifecycle);
    set_workspace_version(&root, version);
    if let Some(heading) = changelog_heading {
        insert_changelog_heading(&root, heading);
    }
    git_ok(&root, &["init", "-q"]);
    let head1 = git_commit(&root, "fixture: release fixture baseline");
    rewrite_release_commits(&root, &head1);
    git_commit(&root, "fixture: align release records with fixture history");
    git_ok(&root, &["tag", "v0.1.0", &head1]);
    if matches!(lifecycle, Lifecycle::CompletedC002) {
        git_ok(&root, &["tag", "v0.2.0", &head1]);
    }
    Fixture { _dir: dir, root }
}

/// Shallow-checkout substitute: tip only, without old tags or recorded commits.
fn shallow_clone(fx: &Fixture) -> PathBuf {
    let dst = fx.root.parent().unwrap().join("shallow");
    let url = format!("file://{}", fx.root.canonicalize().unwrap().display());
    let mut cmd = Command::new("git");
    cmd.args(["clone", "-q", "--depth", "1", &url]).arg(&dst);
    scrub_git_env(&mut cmd);
    let out = cmd.output().unwrap();
    assert!(
        out.status.success(),
        "Shallow clone failed: {}",
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

/// Extract complete job blocks from two-space-indented job headers.
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
    assert!(inside, "Workflow has no job {job}");
    out
}

/// Check announce's quality gate: quality in needs and successful quality in if.
/// Omitting either condition would allow Release creation after quality failure.
fn announce_needs_quality(announce: &str) -> bool {
    announce.contains("      - quality") && announce.contains("needs.quality.result == 'success'")
}

// Task: C002-T15
#[test]
fn check_specs_full_history_resolves_release_commits() {
    // Complete history and fixture-aligned records/tags/commits pass governance.
    let fx = make_fixture("0.1.0", None, Lifecycle::ActiveC002);
    let (ok, text) = run_check_specs(&fx.root);
    assert!(ok, "{text}");
}

// Task: C002-T15
#[test]
fn check_specs_accepts_active_target_without_tag() {
    // Development versions need match the active target, without a tag before release.
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
    // Active-target RCs also pass as development versions without RC tags.
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
    assert!(
        ok,
        "Explicit development-target RCs must not be rejected by nonproduct experiments: {text}"
    );
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
    assert!(
        ok,
        "Explicit development-target RCs must not be rejected when no change is active: {text}"
    );
}

// Task: C006-T05
#[test]
fn check_specs_rejects_missing_duplicate_or_invalid_development_authority() {
    let mut accepted = Vec::new();
    for declaration in [
        "# Documentation map\n",
        "# Documentation map\n\nDevelopment target: `v0.2.0`\nDevelopment target: `v0.2.0`\n",
        "# Documentation map\n\nDevelopment target: `v0.2`\n",
        "# Documentation map\n\nDevelopment target: `v00.2.0`\n",
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
        assert!(failures[0].contains("Development target"), "{text}");
    }
    assert!(
        accepted.is_empty(),
        "Invalid development-target authorities passed: {accepted:?}"
    );
}

// Task: C006-T05
#[test]
fn check_specs_requires_development_authority_in_first_screen() {
    let fx = make_fixture("0.1.0", None, Lifecycle::ActiveC002);
    write_fixture_file(
        &fx.root,
        "specs/README.md",
        &format!("{}Development target: `v0.2.0`\n", "\n".repeat(15)),
    );
    let (ok, text) = run_check_specs(&fx.root);
    assert!(
        !ok,
        "A body field must not replace first-screen authority: {text}"
    );
    assert_eq!(failure_lines(&text).len(), 1, "{text}");
    assert!(text.contains("Development target"), "{text}");
}

// Task: C006-T05
#[test]
fn check_specs_rejects_numeric_active_conflict_even_when_cargo_version_is_released() {
    let fx = make_fixture("0.1.0", None, Lifecycle::ActiveC002);
    write_fixture_file(
        &fx.root,
        "specs/README.md",
        "# Documentation map\n\nDevelopment target: `v0.3.0`\n",
    );
    let (ok, text) = run_check_specs(&fx.root);
    assert!(
        !ok,
        "Released Cargo versions must not hide active-target conflicts: {text}"
    );
    assert_eq!(failure_lines(&text).len(), 1, "{text}");
    assert!(
        text.contains("active target") && text.contains("Development target"),
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
    fs::write(
        path,
        text.replace(
            "`not included in a product release`",
            "`experiment undecided`",
        ),
    )
    .unwrap();
    let (ok, text) = run_check_specs(&fx.root);
    assert!(
        !ok,
        "Unknown experiment targets must not pass through fallback: {text}"
    );
    assert_eq!(failure_lines(&text).len(), 1, "{text}");
    assert!(text.contains("active target"), "{text}");
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
    assert!(
        !ok,
        "Nonproduct experiments must not admit another development line: {text}"
    );
    assert_eq!(failure_lines(&text).len(), 1, "{text}");
    assert!(
        text.contains("Development target") && text.contains("active target"),
        "{text}"
    );
}

// Task: C006-T05
#[test]
fn check_specs_authorized_rc_without_active_still_requires_changelog() {
    let fx = make_fixture("0.2.0-rc.1", None, Lifecycle::CompletedUnreleasedC002);
    let (ok, text) = run_check_specs(&fx.root);
    assert!(
        !ok,
        "No active change must not waive the candidate CHANGELOG: {text}"
    );
    assert_eq!(failure_lines(&text).len(), 1, "{text}");
    assert!(text.contains("CHANGELOG"), "{text}");
}

// Task: C002-T15
#[test]
fn check_specs_rejects_version_outside_active_target() {
    // Change one rejected-case condition: active-target version 0.2.0 becomes 9.9.9, still with its own
    // CHANGELOG entry, matching neither target nor RC.
    let fx = make_fixture(
        "9.9.9",
        Some("## [9.9.9] - 2026-09-28"),
        Lifecycle::ActiveC002,
    );
    let (ok, text) = run_check_specs(&fx.root);
    assert!(!ok, "Version 9.9.9 must not pass: {text}");
    let fails = failure_lines(&text);
    assert_eq!(fails.len(), 1, "{text}");
    assert!(fails[0].contains("active target"), "{text}");
}

// Task: C002-T15
#[test]
fn check_specs_dev_version_requires_changelog_section() {
    // One changed condition: remove the accepted active target's CHANGELOG entry.
    let fx = make_fixture("0.2.0", None, Lifecycle::ActiveC002);
    let (ok, text) = run_check_specs(&fx.root);
    assert!(
        !ok,
        "Missing CHANGELOG version entries must not pass: {text}"
    );
    let fails = failure_lines(&text);
    assert_eq!(fails.len(), 1, "{text}");
    assert!(fails[0].contains("CHANGELOG"), "{text}");
}

// Task: C002-T15
#[test]
fn check_specs_shallow_clone_reports_missing_history() {
    // Shallow checkout retains only tip, lacking old tags/recorded commits; diagnostics must identify
    // incomplete history rather than incorrect records.
    let fx = make_fixture("0.1.0", None, Lifecycle::ActiveC002);
    let shallow = shallow_clone(&fx);
    let (ok, text) = run_check_specs(&shallow);
    assert!(!ok, "{text}");
    assert!(text.contains("incomplete history"), "{text}");
    assert!(text.contains("shallow"), "{text}");
    let fails = failure_lines(&text);
    assert!(!fails.is_empty(), "{text}");
    for line in fails {
        assert!(
            line.contains("incomplete history"),
            "Reported more than incomplete history: {text}"
        );
    }
}

// Task: C002-T15
#[test]
fn check_specs_missing_tag_without_shallow_reports_not_created() {
    // Delete only the tag while retaining full history; report absent/unpushed tags, not incomplete history.
    let fx = make_fixture("0.1.0", None, Lifecycle::ActiveC002);
    git_ok(&fx.root, &["tag", "-d", "v0.1.0"]);
    let (ok, text) = run_check_specs(&fx.root);
    assert!(!ok, "{text}");
    assert!(!text.contains("shallow"), "{text}");
    let fails = failure_lines(&text);
    assert!(!fails.is_empty(), "{text}");
    for line in fails {
        assert!(
            line.contains("not created"),
            "Diagnostic must identify an uncreated tag: {text}"
        );
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
    // Two single-condition rejected cases: omit quality from needs, or omit its successful if condition;
    // either omission would permit Release creation after quality failure.
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
    // Governance fetches full history (shallow checkouts miss old tags); the MSRV 1.85 locked gate actually runs,
    // stable cannot replace it, and announce creates the Release from the same SHA.
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
            "{original}\nRecord form: `reference`\nHistorical snapshot: `{snapshot}`\n\n\
             ## Changes and rationale\n\nFixed reference.\n\n## Validation and limits\n\nFixed scope.\n\n\
             ## References\n\nFixed entry point.\n"
        ),
    )
    .unwrap();
    for file in ["plan.md", "validation.md", "review.md", "tasks.toml"] {
        fs::remove_file(package.join(file)).unwrap();
    }
    let archive = fx
        .root
        .join("docs/en/history/changes/C002-v0.2.0-reliability");
    fs::rename(package, archive).unwrap();
    write_fixture_file(
        &fx.root,
        "docs/en/history/changes/README.md",
        "# Historical changes\n\n[C002](C002-v0.2.0-reliability/README.md)\n",
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
    assert!(
        !ok,
        "Incomplete history must not default to passing: {text}"
    );
    assert!(
        text.contains("cannot verify original completion records"),
        "{text}"
    );
}

// Task: C002-T17
#[test]
fn check_specs_reference_rejects_failed_qualification_in_snapshot() {
    let fx = completed_fixture();
    set_completed_validation(&fx, &final_validation("FAIL"));
    let snapshot = git_commit(&fx.root, "fixture: final validation failure");
    compact_completed(&fx, &snapshot);
    let (ok, text) = run_check_specs(&fx.root);
    assert!(!ok, "Summaries must not hide final FAIL results: {text}");
    assert!(text.contains("non-PASS"), "{text}");
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
    let snapshot = git_commit(&fx.root, "fixture: missing independent review");
    fs::write(
        fx.root
            .join("specs/changes/completed/C002-v0.2.0-reliability/review.md"),
        "placeholder",
    )
    .unwrap();
    compact_completed(&fx, &snapshot);
    let (ok, text) = run_check_specs(&fx.root);
    assert!(!ok, "Missing original review must not pass: {text}");
    assert!(text.contains("snapshot is missing review.md"), "{text}");
}

// Task: C002-T17
#[test]
fn check_specs_reference_rejects_snapshot_of_another_reference() {
    let fx = completed_fixture();
    let package = fx
        .root
        .join("specs/changes/completed/C002-v0.2.0-reliability/README.md");
    let original = fs::read_to_string(&package).unwrap();
    fs::write(&package, format!("{original}\nRecord form: `reference`\n")).unwrap();
    let snapshot = git_commit(&fx.root, "fixture: another summary");
    fs::write(&package, original).unwrap();
    compact_completed(&fx, &snapshot);
    let (ok, text) = run_check_specs(&fx.root);
    assert!(!ok, "Summaries must not replace complete originals: {text}");
    assert!(text.contains("not another summary"), "{text}");
}

// Task: C002-T17
#[test]
fn check_specs_completed_accepts_final_pass_without_active_change() {
    let fx = completed_fixture();
    let (ok, text) = run_check_specs(&fx.root);
    assert!(ok, "{text}");
    assert!(text.contains("0 active"), "{text}");
}

// Task: C002-T17
#[test]
fn check_specs_completed_accepts_final_pass_and_preserves_historical_failures() {
    let fx = completed_fixture();
    let history = "\n## Historical facts\n\n\
        | Identity | Fixed value |\n| --- | --- |\n| Old candidate | abcdef0 |\n\n\
        | Requirement / risk | Mode | Input closure | Command / raw run ID | Historical result | Evidence |\n\
        | --- | --- | --- | --- | --- | --- |\n\
        | Early failure | executed | Old candidate | `old-run` | FAIL | old.log |\n\
        | Actually skipped | not_run | Old candidate | none | SKIP | skip.json |\n";
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
        "{}\n| Historical requirement | Mode | Input | Command | Old result | Evidence |\n\
         | --- | --- | --- | --- | --- | --- |\n\
         | Old run | executed | old | old-run | PASS | old.log |\n",
        final_validation("FAIL")
    );
    set_completed_validation(&fx, &validation);
    let (ok, text) = run_check_specs(&fx.root);
    assert!(!ok, "Current FAIL must not pass: {text}");
    assert_eq!(failure_lines(&text).len(), 1, "{text}");
    assert!(text.contains("non-PASS"), "{text}");
}

// Task: C002-T17
#[test]
fn check_specs_completed_rejects_empty_final_table() {
    let fx = completed_fixture();
    set_completed_validation(&fx, &format!("Candidate: `SELF`\n\n{FINAL_TABLE_HEADER}"));
    let (ok, text) = run_check_specs(&fx.root);
    assert!(!ok, "Final tables without data must not pass: {text}");
    assert_eq!(failure_lines(&text).len(), 1, "{text}");
    assert!(text.contains("has no data rows"), "{text}");
}

// Task: C002-T17
#[test]
fn check_specs_completed_requires_exact_final_table_header() {
    let fx = completed_fixture();
    for (from, to) in [
        ("Input closure", "Old input"),
        ("| Result | Evidence |", "| Result | Evidence | extra"),
    ] {
        let validation = final_validation("PASS").replace(from, to);
        set_completed_validation(&fx, &validation);
        let (ok, text) = run_check_specs(&fx.root);
        assert!(
            !ok,
            "Similar historical tables must not replace the final table: {to}\n{text}"
        );
        assert_eq!(failure_lines(&text).len(), 1, "{text}");
        assert!(text.contains("missing final validation table"), "{text}");
    }
}

// Task: C002-T17
#[test]
fn check_specs_completed_rejects_validation_without_final_table() {
    let fx = completed_fixture();
    set_completed_validation(&fx, "# Validation\n\nCandidate: `SELF`\n\nNo execution.\n");
    let (ok, text) = run_check_specs(&fx.root);
    assert!(!ok, "Missing final tables must not pass: {text}");
    assert_eq!(failure_lines(&text).len(), 1, "{text}");
    assert!(text.contains("missing final validation table"), "{text}");
}

// Task: C002-T17
#[test]
fn check_specs_completed_checks_every_final_table() {
    let fx = completed_fixture();
    let validation = format!(
        "{}\n## Second item\n\n{}",
        final_validation("PASS"),
        final_validation("FAIL")
    );
    set_completed_validation(&fx, &validation);
    let (ok, text) = run_check_specs(&fx.root);
    assert!(!ok, "A second final table's FAIL must not pass: {text}");
    assert_eq!(failure_lines(&text).len(), 1, "{text}");
    assert!(text.contains("non-PASS"), "{text}");
}

// Task: C002-T17
#[test]
fn check_specs_completed_rejects_empty_final_table_beside_nonempty_table() {
    let fx = completed_fixture();
    set_completed_validation(
        &fx,
        &format!(
            "{}\n## Second item\n\n{FINAL_TABLE_HEADER}",
            final_validation("PASS")
        ),
    );
    let (ok, text) = run_check_specs(&fx.root);
    assert!(!ok, "An empty second final table must not pass: {text}");
    assert_eq!(failure_lines(&text).len(), 1, "{text}");
    assert!(text.contains("has no data rows"), "{text}");
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
        assert!(!ok, "Empty final-row fields must not pass: {row}\n{text}");
        assert_eq!(failure_lines(&text).len(), 1, "{text}");
        assert!(text.contains("is empty"), "{row}\n{text}");
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
        assert!(!ok, "Extra final-row columns must not pass: {row}\n{text}");
        assert_eq!(failure_lines(&text).len(), 1, "{text}");
        assert!(text.contains("six columns"), "{text}");
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
    assert!(!ok, "Missing final-row columns must not pass: {text}");
    assert_eq!(failure_lines(&text).len(), 1, "{text}");
    assert!(text.contains("six columns"), "{text}");
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
        assert!(
            !ok,
            "Extra separator-row columns must not pass: {separator}\n{text}"
        );
        assert_eq!(failure_lines(&text).len(), 1, "{text}");
        assert!(text.contains("six columns"), "{text}");
    }
}
