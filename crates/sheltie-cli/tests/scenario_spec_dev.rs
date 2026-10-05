//! C002-T12: five spec-dev single-task and complete-delivery regression groups.
//!
//! The real CLI runs the Workbook; each test builds an independent temporary Git project. Simulated workers follow
//! workbooks/spec-dev artifact rules: task baseline/candidate commit, approval digests, and placeholder ownership.
//! Handwritten oracles implement verify.md checks; expectations do not call production helpers.
//! Actual agent delivery quality belongs to C002-T16.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::path::{Path, PathBuf};
use std::process::Command as Sh;

use common::*;
use serde_json::Value;

#[path = "common/spec_dev_replan.rs"]
mod replan;
use replan::verified_table;

// Task: C010-T02
#[test]
fn spec_dev_final_results_are_frozen_and_wait_for_gate_approval() {
    let env = Env::new();
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../workbooks/spec-dev");
    env.ok(&["workbook", "add", source.to_str().unwrap()]);
    let work = env.start(
        "spec-dev",
        &[
            ("request", "Check method delivery"),
            ("project", &env.home()),
        ],
    );
    let mut next = env.status(&work);
    let mut rule_bindings = Vec::new();
    for node in [
        "spec",
        "plan",
        "plan-review",
        "scaffold",
        "implement",
        "verify",
        "review",
    ] {
        let begun = env.follow_begin(&next, node);
        if matches!(node, "scaffold" | "implement" | "verify") {
            rule_bindings.push(begun.clone());
        }
        next = env.submit_all(&work, &begun, "Complete the current stage");
    }
    let delivery = env.follow_begin(&next, "deliver");
    let delivery_path = delivery["data"]["outputs"]["delivery"].as_str().unwrap();
    std::fs::write(delivery_path, b"delivered result\n").unwrap();
    next = env.ok(&[
        "attempt",
        "submit",
        &work,
        "--attempt",
        "deliver#1.0",
        "--summary",
        "Delivery description complete",
    ]);
    let retro = env.follow_begin(&next, "retro");
    let lessons_path = retro["data"]["outputs"]["lessons"].as_str().unwrap();
    std::fs::write(lessons_path, b"observed method\n").unwrap();
    env.ok(&[
        "attempt",
        "submit",
        &work,
        "--attempt",
        "retro#1.0",
        "--summary",
        "Reflection complete; awaiting approval",
    ]);
    let blocked = env.ok(&["work", "result", &work]);
    assert_eq!(blocked["data"]["status"]["kind"], "blocked");
    assert_eq!(blocked["data"]["status"]["reason"], "gate");
    assert_eq!(blocked["data"]["final"], false);
    assert!(blocked["data"]["artifacts"].as_array().unwrap().is_empty());
    env.ok(&["gate", "approve", &work, "--node", "retro"]);
    let result = env.ok(&["work", "result", &work]);
    assert_eq!(result["data"]["final"], true);
    assert_eq!(result["data"]["effects_pending"], false);
    let artifacts = result["data"]["artifacts"].as_array().unwrap();
    assert_eq!(
        artifacts.len(),
        2,
        "The method must explicitly select two final artifacts"
    );
    assert_eq!(result["data"]["workbook"]["version"], "0.2.3");
    let revision = result["data"]["revision"].as_u64().unwrap().to_string();
    let before = env.status(&work);
    for (artifact, key, kind, path, bytes, digest) in [
        (
            &artifacts[0],
            "delivery",
            "input",
            delivery_path,
            b"delivered result\n".as_slice(),
            "500eb64777c6d890644470f0f028b9e0b4bb815ec24574f7951fb6925f0b8e2f",
        ),
        (
            &artifacts[1],
            "lessons",
            "output",
            lessons_path,
            b"observed method\n".as_slice(),
            "a1f7f94b33bff77f023b089fc115b0d2321fab18b6587dc5e904d916919639d1",
        ),
    ] {
        assert_eq!(artifact["key"], key);
        assert_eq!(artifact["path"], path);
        assert_eq!(artifact["sha256"], digest);
        assert_eq!(artifact["bytes"], bytes.len());
        assert_eq!(artifact["source"]["attempt"], "retro#1.0");
        assert_eq!(artifact["source"]["kind"], kind);
        assert_eq!(artifact["source"]["name"], key);
        let raw = env
            .cmd_text(&[
                "work",
                "result",
                &work,
                "--artifact",
                key,
                "--revision",
                &revision,
            ])
            .output()
            .unwrap();
        assert!(raw.status.success(), "{:?}", raw.stderr);
        assert_eq!(raw.stdout, bytes);
        assert!(raw.stderr.is_empty());
    }
    assert_eq!(env.status(&work), before);
    let rules = env
        .work_dir(&work)
        .canonicalize()
        .unwrap()
        .join("workbook/resources/checklists/approval-rules.md");
    for begun in rule_bindings {
        assert_eq!(
            begun["data"]["inputs"]["approval_rules"],
            rules.to_str().unwrap()
        );
        assert!(brief_text(&begun).contains(rules.to_str().unwrap()));
    }
    assert_eq!(
        std::fs::read(&rules).unwrap(),
        std::fs::read(source.join("resources/checklists/approval-rules.md")).unwrap()
    );
}

// ── Independent temporary Git projects ─────────────────────────────────────────────────────

struct Proj {
    root: PathBuf,
}

impl Proj {
    fn init(env: &Env, name: &str) -> Proj {
        let root = env.dir.path().join(name);
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::create_dir_all(root.join("tests")).unwrap();
        std::fs::write(
            root.join("gate.sh"),
            "#!/bin/sh\necho 'fixture gate passed'\nexit 0\n",
        )
        .unwrap();
        let proj = Proj { root };
        proj.git(&["init", "-q"]);
        proj.git(&["config", "user.email", "sim@example.com"]);
        proj.git(&["config", "user.name", "sim"]);
        proj.git(&["config", "commit.gpgsign", "false"]);
        // Use an empty hooksPath so host hooks cannot affect fixture commits.
        std::fs::create_dir_all(proj.root.join("hooks-empty")).unwrap();
        proj.git(&["config", "core.hooksPath", "hooks-empty"]);
        proj.git(&["add", "-A"]);
        proj.git(&["commit", "-q", "-m", "chore: starting point"]);
        proj
    }

    fn root(&self) -> &Path {
        &self.root
    }

    fn git(&self, args: &[&str]) -> String {
        // Discover only the temporary repository at cwd. Host hooks may pass GIT_DIR/GIT_INDEX_FILE
        // and related variables to descendants; Git could then ignore cwd and modify the host
        // index. Remove all such variables here.
        let out = Sh::new("git")
            .args(args)
            .current_dir(&self.root)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_INDEX_FILE")
            .env_remove("GIT_OBJECT_DIRECTORY")
            .env_remove("GIT_ALTERNATE_OBJECT_DIRECTORIES")
            .env_remove("GIT_COMMON_DIR")
            .env_remove("GIT_NAMESPACE")
            .env_remove("GIT_CEILING_DIRECTORIES")
            .env_remove("GIT_DISCOVERY_ACROSS_FILESYSTEM")
            .env_remove("GIT_CONFIG_GLOBAL")
            .env_remove("GIT_CONFIG_SYSTEM")
            .env_remove("GIT_CONFIG_COUNT")
            .env_remove("GIT_CONFIG_PARAMETERS")
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    fn head(&self) -> String {
        self.git(&["rev-parse", "HEAD"])
    }

    /// Run the project command declared by the plan Gates section.
    fn gate(&self) {
        let out = Sh::new("sh")
            .arg("gate.sh")
            .current_dir(&self.root)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "Gate failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    fn capture_gate(&self, base: &str, candidate: &str, evidence: &Path) {
        let output = Sh::new("sh")
            .arg("gate.sh")
            .current_dir(&self.root)
            .output()
            .unwrap();
        let stdout = evidence.with_extension("stdout");
        let stderr = evidence.with_extension("stderr");
        std::fs::write(&stdout, &output.stdout).unwrap();
        std::fs::write(&stderr, &output.stderr).unwrap();
        std::fs::write(evidence, format!(
            "Command: sh gate.sh\nDirectory: {}\nExit code: {}\nCommit: {candidate}\nBaseline: {base}\nFiles: {}\nstdout: {}\nstdout_sha256: {}\nstderr: {}\nstderr_sha256: {}\n",
            self.root.display(), output.status.code().unwrap(), self.diff_names(base, candidate).join(","),
            stdout.display(), sha256_file(&stdout), stderr.display(), sha256_file(&stderr)
        )).unwrap();
        println!(
            "SPEC_DEV_RAW_GATE {}",
            serde_json::json!({
                "argv": ["sh", "gate.sh"], "cwd": self.root,
                "exit": output.status.code(), "baseline": base, "candidate": candidate,
                "files": self.diff_names(base, candidate),
                "stdout_bytes": output.stdout, "stderr_bytes": output.stderr,
                "manifest": evidence,
            })
        );
        assert!(
            output.status.success(),
            "fixture gate failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    /// Commit all changes and return the candidate commit hash.
    fn commit(&self, msg: &str) -> String {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "-m", msg]);
        self.head()
    }

    /// `git diff --name-only <Baseline>..<Commit>`.
    fn diff_names(&self, base: &str, head: &str) -> Vec<String> {
        let range = format!("{base}..{head}");
        self.git(&["diff", "--name-only", &range])
            .lines()
            .map(|l| l.to_string())
            .collect()
    }

    /// Return actual added/removed lines from the scoped Git diff, excluding diff headers.
    fn diff_lines(&self, base: &str, head: &str, file: &str) -> Vec<String> {
        let range = format!("{base}..{head}");
        self.git(&["diff", &range, "--", file])
            .lines()
            .filter(|l| {
                (l.starts_with('+') || l.starts_with('-'))
                    && !l.starts_with("+++")
                    && !l.starts_with("---")
            })
            .map(|l| l.to_string())
            .collect()
    }
}

// Independent oracles: digests, artifact-text parsing, and verification rules.

/// Match the instructions: shasum -a 256, or sha256sum on Linux, without production digest code.
fn sha256_file(path: &Path) -> String {
    let p = path.to_str().unwrap();
    if let Ok(out) = Sh::new("shasum").args(["-a", "256", p]).output() {
        if out.status.success() {
            return first_field(&String::from_utf8_lossy(&out.stdout));
        }
    }
    let out = Sh::new("sha256sum").arg(p).output().unwrap();
    assert!(out.status.success(), "sha256sum failed: {p}");
    first_field(&String::from_utf8_lossy(&out.stdout))
}

fn first_field(text: &str) -> String {
    text.split_whitespace().next().unwrap().to_string()
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap()
}

/// Parse key: value metadata in change/report/decision artifacts.
fn field<'a>(text: &'a str, key: &str) -> &'a str {
    let prefix = format!("{key}:");
    text.lines()
        .find_map(|l| l.strip_prefix(prefix.as_str()))
        .map(|v| v.trim())
        .unwrap_or_else(|| panic!("Missing {key}: line"))
}

/// Extract the full original hash from the plan Baseline section.
fn plan_baseline(plan_text: &str) -> String {
    let line = plan_text
        .lines()
        .find(|l| l.starts_with("Original baseline ("))
        .expect("plan.md Missing Baseline original baseline line");
    let hash = line
        .rsplit('`')
        .nth(1)
        .expect("Original baseline line lacks a backtick-quoted hash");
    assert_eq!(
        hash.len(),
        40,
        "Original baseline must be a full hash: {hash}"
    );
    hash.to_string()
}

/// A trailing // Tnn or # Tnn identifies placeholder ownership.
fn line_task(line: &str) -> Option<String> {
    for sep in ["//", "#"] {
        if let Some(idx) = line.rfind(sep) {
            let tag = line[idx + sep.len()..].trim();
            if let Some(rest) = tag.strip_prefix('T') {
                if rest.len() == 2 && rest.bytes().all(|b| b.is_ascii_digit()) {
                    return Some(tag.to_string());
                }
            }
        }
    }
    None
}

/// Verify only current-task placeholders; return owned remnants and untagged lines.
/// Recognize Rust placeholders with // Tnn and Python placeholders with # Tnn under scaffold rules.
fn placeholder_left(text: &str, own: &str) -> (Vec<String>, Vec<String>) {
    let rust_stub = concat!("todo", "!()");
    let mut own_left = Vec::new();
    let mut unmarked = Vec::new();
    for line in text.lines() {
        if !(line.contains(rust_stub) || line.contains("raise NotImplementedError")) {
            continue;
        }
        match line_task(line) {
            Some(task) if task == own => own_left.push(line.to_string()),
            Some(_) => {}
            None => unmarked.push(line.to_string()),
        }
    }
    (own_left, unmarked)
}

/// Both approval digests must equal hashes of brief-bound inputs.
/// On mismatch, return actual file digests for the findings.
fn check_approval(
    decision_text: &str,
    spec_path: &Path,
    plan_path: &Path,
) -> Result<(), (String, String)> {
    let got_spec = sha256_file(spec_path);
    let got_plan = sha256_file(plan_path);
    if field(decision_text, "Approved specification") == got_spec
        && field(decision_text, "Approved plan") == got_plan
    {
        Ok(())
    } else {
        Err((got_spec, got_plan))
    }
}

/// Task changes, including repairs, must stay in Allowed files; return violations.
fn out_of_scope(diff: &[String], whitelist: &[&str]) -> Vec<String> {
    diff.iter()
        .filter(|f| !whitelist.contains(&f.as_str()))
        .cloned()
        .collect()
}

/// Implementer removes only the disable-marker line, preserving all other bytes.
fn unskip(text: &str) -> String {
    let mut out = String::new();
    for line in text.lines() {
        if !line.contains("@pytest.mark.skip") {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

// ── Artifact text produced by simulated workers under the instructions ──────────────────────────

const SPEC_MD: &str = "# Specification: export the report page as CSV\n\n## Goal\n\nExport the filtered results to a CSV file.\n\n## Acceptance criteria\n\n- The exported CSV contains only the current filtered results.\n";

/// The plan baseline comes only from the baseline parameter; reuse it unchanged on replanning.
fn plan_md(baseline: &str, revision: &str) -> String {
    format!(
        "# Technical plan: export the report page as CSV\n\n\
         ## Current state\n\nThe report module currently outputs JSON only.\n\n\
         ## Approach\n\n- src/export.py: Export entry point (T01)\n- src/encode.py: CSV Encoding (T02)\n\n\
         ## Gates\n\nEvery completed task must pass all commands, run at the project root.\n\n```bash\nsh gate.sh\n```\n\n\
         ## Baseline\n\nOriginal baseline (Work-start `git rev-parse HEAD`; copy this line verbatim from the prior plan without resetting): `{baseline}`\n\n\
         ## Risks\n\n- Encoding details may require rework.\n\n\
         ## Revision history\n\n{revision}"
    )
}

fn tasks_md(items: &[(&str, &str, &str)]) -> String {
    let mut s = String::from("# Tasks\n\n");
    for (id, files, tests) in items {
        s.push_str(&format!(
            "## {id}\n\n- Allowed files: {files}\n- Tests to enable: {tests}\n- Observable outcome: the corresponding tests pass\n\n"
        ));
    }
    s
}

/// Plan-review decisions bind approved versions and may carry conditions.
fn approval_md(spec_sha: &str, plan_sha: &str, conditions: &str) -> String {
    format!(
        "Accepted\nApproved specification: {spec_sha}\nApproved plan: {plan_sha}\n\n{conditions}\n"
    )
}

/// Escalation decisions contain one of four rulings and human comments.
fn escalation_md(first: &str, opinions: &str) -> String {
    format!("{first}\n{opinions}\n")
}

/// Implementation changes start with Complete Tnn or Blocked Tnn.
fn change_done(
    task: &str,
    baseline: &str,
    commit: &str,
    fix_round: u32,
    files: &[&str],
    note: &str,
) -> String {
    let mut s = format!(
        "Complete {task}\nBaseline: {baseline}\nCommit: {commit}\nRepair round: {fix_round}\nTests enabled: 1\nChanged files:\n"
    );
    for f in files {
        s.push_str(&format!("- {f}\n"));
    }
    s.push_str("Gates:\n- sh gate.sh: PASS\n");
    s.push_str(&format!("Notes: {note}\n"));
    s
}

fn change_stuck(task: &str, note: &str) -> String {
    format!(
        "Blocked {task}\nBaseline: none\nCommit: none\nRepair round: 0\nTests enabled: 0\nChanged files:\nGates:\nNotes: {note}\n"
    )
}

/// Repair changes start with Repairs complete or Blocked.
fn fix_done(task: &str, baseline: &str, commit: &str, fix_round: u32, note: &str) -> String {
    format!(
        "Repairs complete\nTarget: Verification report\nTask: {task}\nBaseline: {baseline}\nCommit: {commit}\nRepair round: {fix_round}\nChanged files:\nGates:\n- sh gate.sh: PASS\nNotes: {note}\n"
    )
}

/// Verification/review first lines select routes; copy metadata from the change.
fn verify_report(first: &str, task: &str, fix_round: u32, baseline: &str, checks: &str) -> String {
    format!(
        "{first}\nTask: {task}\nRepair round: {fix_round}\nBaseline: {baseline}\n\n## Checks\n\n{checks}\n"
    )
}

// ── Real CLI scenarios ────────────────────────────────────────────────────────

fn spec_dev_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../workbooks/spec-dev")
        .canonicalize()
        .unwrap()
}

fn start_spec_dev(env: &Env, proj: &Proj) -> String {
    env.ok(&["workbook", "add", spec_dev_dir().to_str().unwrap()]);
    env.start(
        "spec-dev",
        &[
            ("request", "Add CSV export to the report page"),
            ("project", proj.root().to_str().unwrap()),
        ],
    )
}

/// Write declared outputs by name and submit; unspecified contents use fixture text.
fn submit_outputs(
    env: &Env,
    wid: &str,
    begun: &Value,
    outputs: &[(&str, &str)],
    summary: &str,
) -> Value {
    let attempt = begun["data"]["attempt"].as_str().unwrap().to_string();
    let declared = begun["data"]["outputs"].as_object().unwrap();
    for (output, input) in [("reviewed-plan", "plan"), ("reviewed-tasks", "tasks")] {
        if let Some(destination) = declared.get(output).and_then(Value::as_str) {
            let destination = Path::new(destination);
            std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
            std::fs::copy(input_path(begun, input), destination).unwrap();
        }
    }
    for (name, content) in outputs {
        let path = declared
            .get(*name)
            .unwrap_or_else(|| panic!("{attempt} No declared output {name}"))
            .as_str()
            .unwrap();
        let p = Path::new(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, content).unwrap();
    }
    env.ok(&[
        "attempt",
        "submit",
        wid,
        "--attempt",
        &attempt,
        "--summary",
        summary,
    ])
}

/// The file bound by the brief input table.
fn input_path(begun: &Value, name: &str) -> PathBuf {
    let v = begun["data"]["inputs"]
        .as_object()
        .unwrap()
        .get(name)
        .unwrap_or_else(|| panic!("No input {name}"));
    assert!(!v.is_null(), "Input {name} not bound");
    PathBuf::from(v.as_str().unwrap())
}

fn brief_text(begun: &Value) -> String {
    read(Path::new(begun["data"]["brief_path"].as_str().unwrap()))
}

/// An output target in the brief, read back from disk after submission.
fn output_file(begun: &Value, name: &str) -> PathBuf {
    PathBuf::from(
        begun["data"]["outputs"]
            .as_object()
            .unwrap()
            .get(name)
            .unwrap()
            .as_str()
            .unwrap(),
    )
}

/// Plan in three steps: spec, plan with original baseline, then accepted plan-review.
/// Return the approval text.
fn plan_and_approve(env: &Env, wid: &str, proj: &Proj, baseline: &str, conditions: &str) -> String {
    plan_and_approve_with_hash(env, wid, proj, baseline, conditions, None)
}

fn plan_and_approve_with_hash(
    env: &Env,
    wid: &str,
    proj: &Proj,
    baseline: &str,
    conditions: &str,
    plan_hash: Option<&str>,
) -> String {
    // Before implementation commits, the plan baseline is the Work-start HEAD.
    assert_eq!(proj.head(), baseline);
    let s0 = env.begin(wid, "spec");
    submit_outputs(
        env,
        wid,
        &s0,
        &[("spec", SPEC_MD)],
        "Specification complete",
    );
    let b1 = env.begin(wid, "plan");
    assert_eq!(b1["data"]["attempt"], "plan#1.0");
    let plan = format!(
        "{}\nInheritance source: none\n\n{}",
        plan_md(baseline, "- 2026-09-28 Initial version\n"),
        verified_table(&[])
    );
    let tasks = tasks_md(&[
        (
            "T01 Export entry point",
            "src/export.py、tests/test_export.py",
            "test_export",
        ),
        (
            "T02 Encoding",
            "src/encode.py、tests/test_encode.py",
            "test_encode",
        ),
    ]);
    let tasks = format!(
        "{tasks}\nOriginal baseline: {baseline}\nInheritance source: none\n\n{}",
        verified_table(&[])
    );
    submit_outputs(
        env,
        wid,
        &b1,
        &[("plan", &plan), ("tasks", &tasks)],
        "Plan complete",
    );
    let pr = env.begin(wid, "plan-review");
    let decision = approval_md(
        &sha256_file(&input_path(&pr, "spec")),
        &plan_hash
            .map(str::to_string)
            .unwrap_or_else(|| sha256_file(&input_path(&pr, "plan"))),
        conditions,
    );
    submit_outputs(env, wid, &pr, &[("decision", &decision)], "Approve");
    decision
}

/// Check approval versions, write placeholders, commit the scaffold, and submit scaffold.md.
fn do_scaffold(env: &Env, wid: &str, proj: &Proj, files: &[(&str, &str)]) -> Value {
    let b = env.begin(wid, "scaffold");
    let approval_path = b["data"]["inputs"]["escalation"]
        .as_str()
        .map(PathBuf::from)
        .filter(|path| read(path).contains("Corrected approval:"))
        .unwrap_or_else(|| input_path(&b, "decision"));
    assert_eq!(
        check_approval(
            &read(&approval_path),
            &input_path(&b, "spec"),
            &input_path(&b, "plan")
        ),
        Ok(())
    );
    for (path, content) in files {
        std::fs::write(proj.root().join(path), content).unwrap();
    }
    proj.gate();
    let head = proj.commit("feat: scaffold\n\nTask: scaffold\nAgent: sim");
    let md = format!("Complete\nCommit: {head}\nFocused test command: pytest tests/ -k <Task>\n");
    submit_outputs(env, wid, &b, &[("scaffold", &md)], "Scaffold ready")
}

/// Two tasks own separate scaffold files (G1/G4a).
fn scaffold_two_files(env: &Env, wid: &str, proj: &Proj) -> Value {
    do_scaffold(
        env,
        wid,
        proj,
        &[
            ("src/export.py", EXPORT_PY_STUB),
            ("tests/test_export.py", TEST_EXPORT_PY_SKIPPED),
            ("src/encode.py", ENCODE_PY_STUB),
            ("tests/test_encode.py", TEST_ENCODE_PY_SKIPPED),
        ],
    )
}

const EXPORT_PY_STUB: &str = "def export(rows):\n    raise NotImplementedError  # T01\n";
const EXPORT_PY_DONE: &str = "def export(rows):\n    return \",\".join(rows)\n";
const ENCODE_PY_STUB: &str = "def encode(rows):\n    raise NotImplementedError  # T02\n";
const TEST_EXPORT_PY_SKIPPED: &str = "import pytest\n\nfrom src.export import export\n\n\n@pytest.mark.skip(reason=\"T01\")\ndef test_export():\n    assert export([\"a\"]) == \"a\"\n";
const TEST_ENCODE_PY_SKIPPED: &str = "import pytest\n\nfrom src.encode import encode\n\n\n@pytest.mark.skip(reason=\"T02\")\ndef test_encode():\n    assert encode([\"a\"]) == \"a\"\n";

/// Prepare real T01 writes/gates/commit only; callers own approvals, change metadata, and expectations.
fn commit_export_task(proj: &Proj, agent: &str) -> (String, String) {
    let base = proj.head();
    std::fs::write(proj.root().join("src/export.py"), EXPORT_PY_DONE).unwrap();
    std::fs::write(
        proj.root().join("tests/test_export.py"),
        unskip(TEST_EXPORT_PY_SKIPPED),
    )
    .unwrap();
    proj.gate();
    let candidate = proj.commit(&format!(
        "feat(export): Export entry point\n\nTask: T01\nAgent: {agent}"
    ));
    (base, candidate)
}

/// Handwritten verification oracle for a cleanly completed T01.
fn assert_t01_clean(proj: &Proj, base: &str, commit: &str, v: &Value) {
    let diff = proj.diff_names(base, commit);
    assert_eq!(diff, ["src/export.py", "tests/test_export.py"]);
    assert!(out_of_scope(&diff, &["src/export.py", "tests/test_export.py"]).is_empty());
    // The test diff removes only the disable-marker line.
    assert_eq!(
        proj.diff_lines(base, commit, "tests/test_export.py"),
        ["-@pytest.mark.skip(reason=\"T01\")"]
    );
    // No owned or untagged placeholders may remain.
    let (own_left, unmarked) = placeholder_left(&read(&proj.root().join("src/export.py")), "T01");
    assert!(
        own_left.is_empty(),
        "This task retains placeholders: {own_left:?}"
    );
    assert!(
        unmarked.is_empty(),
        "Placeholder without a task ID: {unmarked:?}"
    );
    // Approval matches the version bound by plan-review.
    assert_eq!(
        check_approval(
            &read(&input_path(v, "decision")),
            &input_path(v, "spec"),
            &input_path(v, "plan"),
        ),
        Ok(())
    );
}

// ── Five complete workflow groups ─────────────────────────────────────────────────────────────

// Task: C002-T12
#[test]
fn two_task_loops_scope_each_task_diff_to_its_own_files() {
    let env = Env::new();
    let proj = Proj::init(&env, "g1");
    let baseline0 = proj.head();
    let wid = start_spec_dev(&env, &proj);
    plan_and_approve(
        &env,
        &wid,
        &proj,
        &baseline0,
        "Conditions: export filename is export.csv.",
    );
    let sc = scaffold_two_files(&env, &wid, &proj);
    let scaffold_commit = proj.head();

    // T01 changes only its two owned files.
    let im1 = env.follow_begin(&sc, "implement");
    assert_eq!(im1["data"]["attempt"], "implement#1.0");
    assert!(brief_text(&im1).contains("From: scaffold#1 (main edge)"));
    assert!(
        input_path(&im1, "decision")
            .to_str()
            .unwrap()
            .ends_with("/attempts/plan-review/occurrence-001/attempt-000/outputs/decision.md")
    );
    let (t01_base, t01_commit) = commit_export_task(&proj, "sim");
    let change1 = change_done(
        "T01",
        &t01_base,
        &t01_commit,
        0,
        &["src/export.py", "tests/test_export.py"],
        "",
    );
    let s1 = submit_outputs(&env, &wid, &im1, &[("change", &change1)], "Complete T01");

    let v1 = env.follow_begin(&s1, "verify");
    assert_eq!(v1["data"]["attempt"], "verify#1.0");
    assert!(brief_text(&v1).contains("From: implement#1 (main edge)"));
    assert!(
        input_path(&v1, "change")
            .to_str()
            .unwrap()
            .ends_with("/attempts/implement/occurrence-001/attempt-000/outputs/change.md")
    );
    assert_t01_clean(&proj, &t01_base, &t01_commit, &v1);
    let r1 = verify_report(
        "Accepted, next task T02",
        "T01",
        0,
        &t01_base,
        "- Approval version: matches\n- Changed scope: Entirely allowlisted\n- Placeholders: No placeholders remain for this task\n- Conditions: Export filename is export.csv, holds",
    );
    let s2 = submit_outputs(
        &env,
        &wid,
        &v1,
        &[("report", &r1)],
        "Accepted, next task T02",
    );

    // T02 leaves a placeholder and needs one repair; scope remains T02 files only.
    let im2 = env.follow_begin(&s2, "implement");
    assert_eq!(im2["data"]["attempt"], "implement#2.0");
    assert!(brief_text(&im2).contains("From: verify#1 (main edge)"));
    // Bind the prior report path to implement without embedding its body in the brief.
    let report_in = input_path(&im2, "report");
    assert!(
        report_in
            .to_str()
            .unwrap()
            .ends_with("/attempts/verify/occurrence-001/attempt-000/outputs/report.md")
    );
    assert!(brief_text(&im2).contains(&format!("| report | {} | ", report_in.display())));
    let t02_base = proj.head();
    assert_eq!(t02_base, t01_commit);
    std::fs::write(
        proj.root().join("tests/test_encode.py"),
        unskip(TEST_ENCODE_PY_SKIPPED),
    )
    .unwrap();
    proj.gate();
    let t02_commit = proj.commit("feat(encode): Encoding\n\nTask: T02\nAgent: sim");
    let change2 = change_done(
        "T02",
        &t02_base,
        &t02_commit,
        0,
        &["src/encode.py", "tests/test_encode.py"],
        "The placeholder is still unimplemented.",
    );
    let s3 = submit_outputs(&env, &wid, &im2, &[("change", &change2)], "Complete T02");

    let v2 = env.follow_begin(&s3, "verify");
    let diff2 = proj.diff_names(&t02_base, &t02_commit);
    assert_eq!(diff2, ["tests/test_encode.py"]);
    let (own_left, _) = placeholder_left(&read(&proj.root().join("src/encode.py")), "T02");
    assert_eq!(own_left, ["    raise NotImplementedError  # T02"]);
    let r2 = verify_report(
        "Rejected",
        "T02",
        0,
        &t02_base,
        "- Changed scope: Entirely allowlisted\n- Placeholders: One placeholder remains for this task",
    );
    let s4 = submit_outputs(&env, &wid, &v2, &[("report", &r2)], "Rejected");

    let fx = env.follow_begin(&s4, "fix");
    assert!(brief_text(&fx).contains("From: verify#2 (branch edge)"));
    std::fs::write(
        proj.root().join("src/encode.py"),
        "def encode(rows):\n    return \"|\".join(rows)\n",
    )
    .unwrap();
    proj.gate();
    let fix_commit =
        proj.commit("fix(encode): Complete the implementation\n\nTask: T02\nAgent: sim");
    let fix_md = fix_done(
        "T02",
        &t02_base,
        &fix_commit,
        1,
        "The placeholder is implemented.",
    );
    let s5 = submit_outputs(&env, &wid, &fx, &[("change", &fix_md)], "Repairs complete");

    // Include repairs from the same task baseline: two files, excluding T01.
    let v3 = env.follow_begin(&s5, "verify");
    assert_eq!(v3["data"]["attempt"], "verify#3.0");
    assert!(brief_text(&v3).contains("From: fix#1 (re_review edge)"));
    assert!(
        input_path(&v3, "fix_change")
            .to_str()
            .unwrap()
            .ends_with("/attempts/fix/occurrence-001/attempt-000/outputs/change.md")
    );
    let fix_change_text = read(&input_path(&v3, "fix_change"));
    assert_eq!(field(&fix_change_text, "Repair round"), "1");
    assert_eq!(field(&fix_change_text, "Baseline"), t02_base);
    assert_eq!(field(&fix_change_text, "Commit"), fix_commit);
    let diff3 = proj.diff_names(&t02_base, &fix_commit);
    assert_eq!(diff3, ["src/encode.py", "tests/test_encode.py"]);
    assert!(out_of_scope(&diff3, &["src/encode.py", "tests/test_encode.py"]).is_empty());
    let (own_left, unmarked) = placeholder_left(&read(&proj.root().join("src/encode.py")), "T02");
    assert!(own_left.is_empty() && unmarked.is_empty());

    // O09 failure: cumulative scaffold scope incorrectly labels T01 files as T02 violations.
    let cumulative = proj.diff_names(&scaffold_commit, &fix_commit);
    assert_eq!(
        cumulative,
        [
            "src/encode.py",
            "src/export.py",
            "tests/test_encode.py",
            "tests/test_export.py"
        ]
    );
    assert!(!diff3.iter().any(|f| f == "src/export.py"));

    let r3 = verify_report(
        "Accepted, all tasks complete",
        "T02",
        1,
        &t02_base,
        "- Approval version: matches\n- Changed scope: Both files, including repairs, are allowlisted\n- Placeholders: No placeholders remain for this task",
    );
    let s6 = submit_outputs(
        &env,
        &wid,
        &v3,
        &[("report", &r3)],
        "Accepted, all tasks complete",
    );
    // After all tasks, overall review checks the original baseline (G5).
    assert!(next_begin_nodes(&s6).contains(&"review".to_string()));
}

// Task: C002-T12
#[test]
fn out_of_whitelist_file_in_second_task_is_flagged() {
    let env = Env::new();
    let proj = Proj::init(&env, "g1n");
    let baseline0 = proj.head();
    let wid = start_spec_dev(&env, &proj);
    plan_and_approve(&env, &wid, &proj, &baseline0, "Conditions: none.");
    let sc = scaffold_two_files(&env, &wid, &proj);

    // Complete T01 through the same accepted path.
    let im1 = env.follow_begin(&sc, "implement");
    let (t01_base, t01_commit) = commit_export_task(&proj, "sim");
    let change1 = change_done(
        "T01",
        &t01_base,
        &t01_commit,
        0,
        &["src/export.py", "tests/test_export.py"],
        "",
    );
    let s1 = submit_outputs(&env, &wid, &im1, &[("change", &change1)], "Complete T01");
    let v1 = env.follow_begin(&s1, "verify");
    assert_t01_clean(&proj, &t01_base, &t01_commit, &v1);
    let r1 = verify_report(
        "Accepted, next task T02",
        "T01",
        0,
        &t01_base,
        "- All checks match",
    );
    let s2 = submit_outputs(
        &env,
        &wid,
        &v1,
        &[("report", &r1)],
        "Accepted, next task T02",
    );

    // Change one condition: T02 also edits T01 export.py outside its allowlist.
    let im2 = env.follow_begin(&s2, "implement");
    let t02_base = proj.head();
    std::fs::write(
        proj.root().join("src/encode.py"),
        "def encode(rows):\n    return \"|\".join(rows)\n",
    )
    .unwrap();
    std::fs::write(
        proj.root().join("tests/test_encode.py"),
        unskip(TEST_ENCODE_PY_SKIPPED),
    )
    .unwrap();
    std::fs::write(
        proj.root().join("src/export.py"),
        "def export(rows):\n    return \";\".join(rows)\n",
    )
    .unwrap();
    proj.gate();
    let t02_commit = proj.commit("feat(encode): Encoding\n\nTask: T02\nAgent: sim");
    let change2 = change_done(
        "T02",
        &t02_base,
        &t02_commit,
        0,
        &["src/encode.py", "tests/test_encode.py"],
        "",
    );
    submit_outputs(&env, &wid, &im2, &[("change", &change2)], "Complete T02");

    // The verifier checks baseline..commit and catches the extra file.
    let v2 = env.begin(&wid, "verify");
    let diff = proj.diff_names(&t02_base, &t02_commit);
    assert_eq!(
        diff,
        ["src/encode.py", "src/export.py", "tests/test_encode.py"]
    );
    assert_eq!(
        out_of_scope(&diff, &["src/encode.py", "tests/test_encode.py"]),
        ["src/export.py"]
    );
    let r2 = verify_report(
        "Rejected",
        "T02",
        0,
        &t02_base,
        "- Out-of-scope file changed: src/export.py",
    );
    let s3 = submit_outputs(&env, &wid, &v2, &[("report", &r2)], "Rejected");
    // The report locates the repair without clarification.
    let report_text = read(&output_file(&v2, "report"));
    assert_eq!(report_text.lines().next().unwrap(), "Rejected");
    assert!(report_text.contains("Out-of-scope file changed: src/export.py"));
    assert_eq!(field(&report_text, "Baseline"), t02_base);
    // Rejection routes to repair.
    assert!(next_begin_nodes(&s3).contains(&"fix".to_string()));
}

// Task: C002-T12
#[test]
fn shared_file_keeps_future_task_placeholders() {
    let env = Env::new();
    let proj = Proj::init(&env, "g2");
    let baseline0 = proj.head();
    let wid = start_spec_dev(&env, &proj);
    plan_and_approve(&env, &wid, &proj, &baseline0, "Conditions: none.");

    // Two tasks own separate placeholders in a shared source, with independently owned tests.
    let sc = do_scaffold(
        &env,
        &wid,
        &proj,
        &[
            (
                "src/feature.py",
                "def step_one():\n    raise NotImplementedError  # T01\n\n\ndef step_two():\n    raise NotImplementedError  # T02\n",
            ),
            (
                "tests/test_step_one.py",
                "import pytest\n\nfrom src.feature import step_one\n\n\n@pytest.mark.skip(reason=\"T01\")\ndef test_step_one():\n    assert step_one() == 1\n",
            ),
            (
                "tests/test_step_two.py",
                "import pytest\n\nfrom src.feature import step_two\n\n\n@pytest.mark.skip(reason=\"T02\")\ndef test_step_two():\n    assert step_two() == 2\n",
            ),
        ],
    );

    // T01 fills only its own placeholder and preserves T02.
    let im = env.follow_begin(&sc, "implement");
    // Also verify brief binding: implement receives the approval-record path.
    let decision = input_path(&im, "decision");
    assert!(
        decision
            .to_str()
            .unwrap()
            .ends_with("/attempts/plan-review/occurrence-001/attempt-000/outputs/decision.md")
    );
    assert!(brief_text(&im).contains(&format!("| decision | {} | ", decision.display())));
    let base = proj.head();
    std::fs::write(
        proj.root().join("src/feature.py"),
        "def step_one():\n    return 1\n\n\ndef step_two():\n    raise NotImplementedError  # T02\n",
    )
    .unwrap();
    std::fs::write(
        proj.root().join("tests/test_step_one.py"),
        unskip(
            "import pytest\n\nfrom src.feature import step_one\n\n\n@pytest.mark.skip(reason=\"T01\")\ndef test_step_one():\n    assert step_one() == 1\n",
        ),
    )
    .unwrap();
    proj.gate();
    let commit = proj.commit("feat(feature): Step one\n\nTask: T01\nAgent: sim");
    let change = change_done(
        "T01",
        &base,
        &commit,
        0,
        &["src/feature.py", "tests/test_step_one.py"],
        "",
    );
    let s = submit_outputs(&env, &wid, &im, &[("change", &change)], "Complete T01");

    // Verify this task only; retained future-task placeholders are valid.
    let v = env.follow_begin(&s, "verify");
    let change_in = input_path(&v, "change");
    assert!(
        change_in
            .to_str()
            .unwrap()
            .ends_with("/attempts/implement/occurrence-001/attempt-000/outputs/change.md")
    );
    assert!(brief_text(&v).contains(&format!("| change | {} | ", change_in.display())));
    let content = read(&proj.root().join("src/feature.py"));
    let (own_left, unmarked) = placeholder_left(&content, "T01");
    assert!(
        own_left.is_empty(),
        "This task retains placeholders: {own_left:?}"
    );
    assert!(unmarked.is_empty());
    assert!(
        content.contains("raise NotImplementedError  # T02"),
        "Future-task placeholders must remain: {content}"
    );
    let diff = proj.diff_names(&base, &commit);
    assert_eq!(diff, ["src/feature.py", "tests/test_step_one.py"]);
    let r = verify_report(
        "Accepted, next task T02",
        "T01",
        0,
        &base,
        "- Changed scope: Entirely allowlisted\n- Placeholders: This task is complete; retain T02 placeholders under the scaffold rules",
    );
    submit_outputs(&env, &wid, &v, &[("report", &r)], "Accepted, next task T02");
}

fn feature_task_fixture(env: &Env, name: &str, source: &str) -> (Proj, String, Value) {
    let proj = Proj::init(env, name);
    let baseline = proj.head();
    let work = start_spec_dev(env, &proj);
    plan_and_approve(env, &work, &proj, &baseline, "Conditions: none.");
    let scaffold = do_scaffold(env, &work, &proj, &[("src/feature.py", source)]);
    (proj, work, scaffold)
}

fn submit_feature_task(
    env: &Env,
    work: &str,
    proj: &Proj,
    implement: &Value,
    source: &str,
) -> (String, Value) {
    let base = proj.head();
    std::fs::write(proj.root().join("src/feature.py"), source).unwrap();
    proj.gate();
    let commit = proj.commit("feat(feature): Step two\n\nTask: T01\nAgent: sim");
    let change = change_done("T01", &base, &commit, 0, &["src/feature.py"], "");
    let submitted = submit_outputs(env, work, implement, &[("change", &change)], "Complete T01");
    (base, env.follow_begin(&submitted, "verify"))
}

// Task: C002-T12
#[test]
fn own_task_placeholder_left_behind_is_flagged() {
    let env = Env::new();
    let (proj, wid, sc) = feature_task_fixture(
        &env,
        "g2n",
        "def step_one():\n    raise NotImplementedError  # T01\n\n\ndef step_two():\n    raise NotImplementedError  # T02\n",
    );

    // Change one condition: fill T02 while leaving the current-task placeholder.
    let im = env.follow_begin(&sc, "implement");
    let decision = input_path(&im, "decision");
    assert!(
        decision
            .to_str()
            .unwrap()
            .ends_with("/attempts/plan-review/occurrence-001/attempt-000/outputs/decision.md")
    );
    let (base, v) = submit_feature_task(
        &env,
        &wid,
        &proj,
        &im,
        "def step_one():\n    raise NotImplementedError  # T01\n\n\ndef step_two():\n    return 2\n",
    );
    let (own_left, unmarked) = placeholder_left(&read(&proj.root().join("src/feature.py")), "T01");
    assert_eq!(own_left, ["    raise NotImplementedError  # T01"]);
    assert!(unmarked.is_empty());
    let r = verify_report(
        "Rejected",
        "T01",
        0,
        &base,
        "- Unfilled placeholder: src/feature.py  step_one still belongs to T01",
    );
    submit_outputs(&env, &wid, &v, &[("report", &r)], "Rejected");
    assert_eq!(
        read(&output_file(&v, "report")).lines().next().unwrap(),
        "Rejected"
    );
}

// Task: C002-T12
#[test]
fn unmarked_placeholder_is_flagged() {
    let env = Env::new();
    let (proj, wid, sc) = feature_task_fixture(
        &env,
        "g2u",
        "def step_one():\n    raise NotImplementedError\n\n\ndef step_two():\n    raise NotImplementedError  # T02\n",
    );

    // The only changed condition is a missing current-task Task tag.
    let im = env.follow_begin(&sc, "implement");
    let (base, v) = submit_feature_task(
        &env,
        &wid,
        &proj,
        &im,
        "def step_one():\n    raise NotImplementedError\n\n\ndef step_two():\n    return 2\n",
    );
    let (own_left, unmarked) = placeholder_left(&read(&proj.root().join("src/feature.py")), "T01");
    // An untagged placeholder still rejects even when the owned list is empty.
    assert!(own_left.is_empty());
    assert_eq!(unmarked, ["    raise NotImplementedError"]);
    let r = verify_report(
        "Rejected",
        "T01",
        0,
        &base,
        "- Placeholder without a task ID: src/feature.py step_one remains a placeholder",
    );
    submit_outputs(&env, &wid, &v, &[("report", &r)], "Rejected");
    let report_text = read(&output_file(&v, "report"));
    assert_eq!(report_text.lines().next().unwrap(), "Rejected");
    assert!(report_text.contains("Placeholder without a task ID"));
}

// Task: C002-T12
#[test]
fn conditional_approval_binds_decision_into_scaffold_implement_verify() {
    let env = Env::new();
    let proj = Proj::init(&env, "g3");
    let baseline0 = proj.head();
    let wid = start_spec_dev(&env, &proj);
    let conditions = "Conditions: export filename is export.csv.";
    plan_and_approve(&env, &wid, &proj, &baseline0, conditions);

    // Scaffold/implement/verify bind the same approval; check each bound file digest.
    let mut last = env.begin(&wid, "scaffold");
    let mut t01_base = String::new();
    for (idx, node) in ["scaffold", "implement", "verify"].iter().enumerate() {
        if idx > 0 {
            last = env.follow_begin(&last, node);
        }
        assert_eq!(last["data"]["node"].as_str().unwrap(), *node);
        let decision = input_path(&last, "decision");
        assert!(
            decision
                .to_str()
                .unwrap()
                .ends_with("/attempts/plan-review/occurrence-001/attempt-000/outputs/decision.md"),
            "{node}  bound decision: {decision:?}"
        );
        assert!(
            input_path(&last, "plan")
                .to_str()
                .unwrap()
                .ends_with("/attempts/plan/occurrence-001/attempt-000/outputs/plan.md"),
            "{node}  bound plan is not the approved version"
        );
        assert!(
            input_path(&last, "spec")
                .to_str()
                .unwrap()
                .ends_with("/attempts/spec/occurrence-001/attempt-000/outputs/spec.md"),
            "{node}  bound specification is not the approved version"
        );
        let brief = brief_text(&last);
        assert!(
            brief.contains(&format!("| decision | {} | ", decision.display())),
            "{node}"
        );
        // Approval matches; verification treats every recorded condition as acceptance.
        let decision_text = read(&decision);
        assert_eq!(
            check_approval(
                &decision_text,
                &input_path(&last, "spec"),
                &input_path(&last, "plan")
            ),
            Ok(())
        );
        assert!(decision_text.contains(conditions));
        // Execute and submit so the following node can run.
        match *node {
            "scaffold" => {
                std::fs::write(proj.root().join("src/export.py"), EXPORT_PY_STUB).unwrap();
                std::fs::write(
                    proj.root().join("tests/test_export.py"),
                    TEST_EXPORT_PY_SKIPPED,
                )
                .unwrap();
                std::fs::write(proj.root().join("src/encode.py"), ENCODE_PY_STUB).unwrap();
                std::fs::write(
                    proj.root().join("tests/test_encode.py"),
                    TEST_ENCODE_PY_SKIPPED,
                )
                .unwrap();
                proj.gate();
                let head = proj.commit("feat: scaffold\n\nTask: scaffold\nAgent: sim");
                let md = format!(
                    "Complete\nCommit: {head}\nFocused test command: pytest tests/ -k <Task>\n"
                );
                last = submit_outputs(&env, &wid, &last, &[("scaffold", &md)], "Scaffold ready");
            }
            "implement" => {
                t01_base = proj.head();
                std::fs::write(proj.root().join("src/export.py"), EXPORT_PY_DONE).unwrap();
                std::fs::write(
                    proj.root().join("tests/test_export.py"),
                    unskip(TEST_EXPORT_PY_SKIPPED),
                )
                .unwrap();
                proj.gate();
                let commit =
                    proj.commit("feat(export): Export entry point\n\nTask: T01\nAgent: sim");
                let change = change_done(
                    "T01",
                    &t01_base,
                    &commit,
                    0,
                    &["src/export.py", "tests/test_export.py"],
                    "",
                );
                last = submit_outputs(&env, &wid, &last, &[("change", &change)], "Complete T01");
            }
            _ => {
                let report_path = output_file(&last, "report");
                let report = verify_report(
                    "Accepted, next task T02",
                    "T01",
                    0,
                    &t01_base,
                    &format!(
                        "- Approval version: matches\n- Conditions as acceptance criteria: {conditions} holds"
                    ),
                );
                last = submit_outputs(
                    &env,
                    &wid,
                    &last,
                    &[("report", &report)],
                    "Accepted, next task T02",
                );
                // The verification report actually includes the condition.
                assert!(read(&report_path).contains(conditions));
            }
        }
    }
    // All three nodes checked approval versions; verification routes to the next task.
    assert!(next_begin_nodes(&last).contains(&"implement".to_string()));
}

// Task: C002-T12
#[test]
fn stale_approval_hash_makes_worker_stop() {
    let env = Env::new();
    let proj = Proj::init(&env, "g3n");
    let baseline0 = proj.head();
    let wid = start_spec_dev(&env, &proj);
    let s0 = env.begin(&wid, "spec");
    submit_outputs(
        &env,
        &wid,
        &s0,
        &[("spec", SPEC_MD)],
        "Specification complete",
    );

    // Plan#1 review records its digest and requests revision.
    let b1 = env.begin(&wid, "plan");
    let plan1 = plan_md(&baseline0, "- 2026-09-28 Initial version\n");
    submit_outputs(
        &env,
        &wid,
        &b1,
        &[
            ("plan", &plan1),
            (
                "tasks",
                &tasks_md(&[
                    (
                        "T01 Export entry point",
                        "src/export.py、tests/test_export.py",
                        "test_export",
                    ),
                    (
                        "T02 Encoding",
                        "src/encode.py、tests/test_encode.py",
                        "test_encode",
                    ),
                ]),
            ),
        ],
        "Plan complete",
    );
    let pr1 = env.begin(&wid, "plan-review");
    let plan1_sha = sha256_file(&input_path(&pr1, "plan"));
    let spec_sha = sha256_file(&input_path(&pr1, "spec"));
    let d1 = format!(
        "Revise plan\nApproved specification: {spec_sha}\nApproved plan: {plan1_sha}\n\nThe task decomposition is wrong.\n"
    );
    let s1 = submit_outputs(&env, &wid, &pr1, &[("decision", &d1)], "Revise plan");

    // Plan#2 preserves its baseline, but human approval incorrectly retains plan#1 digests.
    let b2 = env.follow_begin(&s1, "plan");
    assert_eq!(b2["data"]["attempt"], "plan#2.0");
    let plan2 = plan_md(
        &baseline0,
        "- 2026-09-28 Initial version\n- 2026-09-28 Rearrange the approach\n",
    );
    let s2 = submit_outputs(
        &env,
        &wid,
        &b2,
        &[
            ("plan", &plan2),
            (
                "tasks",
                &tasks_md(&[(
                    "T01 Encoding",
                    "src/encode.py、tests/test_encode.py",
                    "test_encode",
                )]),
            ),
        ],
        "Plan rearranged",
    );
    let pr2 = env.follow_begin(&s2, "plan-review");
    assert_eq!(pr2["data"]["attempt"], "plan-review#2.0");
    assert!(
        input_path(&pr2, "plan")
            .to_str()
            .unwrap()
            .ends_with("/attempts/plan/occurrence-002/attempt-000/outputs/plan.md")
    );
    let d2 = format!(
        "Accepted\nApproved specification: {spec_sha}\nApproved plan: {plan1_sha}\n\nConditions: none.\n"
    );
    let s3 = submit_outputs(&env, &wid, &pr2, &[("decision", &d2)], "Approve");

    // Scaffold stops: bound plan#2 differs from approved plan#1.
    let sb = env.follow_begin(&s3, "scaffold");
    assert_eq!(sb["data"]["attempt"], "scaffold#1.0");
    let decision_text = read(&input_path(&sb, "decision"));
    assert_eq!(field(&decision_text, "Approved plan"), plan1_sha);
    let err = check_approval(
        &decision_text,
        &input_path(&sb, "spec"),
        &input_path(&sb, "plan"),
    )
    .unwrap_err();
    assert_eq!(err.1, sha256_file(&input_path(&sb, "plan")));
    assert_ne!(
        err.1, plan1_sha,
        "Different digests must invalidate the old approval"
    );
    // Correct only Approved plan; the sole difference is its hash.
    let corrected = approval_md(&spec_sha, &err.1, "Conditions: none.");
    assert_eq!(
        check_approval(
            &corrected,
            &input_path(&sb, "spec"),
            &input_path(&sb, "plan")
        ),
        Ok(())
    );

    // The blocked path makes no project commit and does not start an unapproved plan.
    let head_before = proj.head();
    let stuck = format!(
        "Blocked\nNotes: Approved specification {}, Approved plan {},  bound-plan digest {}; The old approval is invalid.\n",
        spec_sha, plan1_sha, err.1
    );
    let s4 = submit_outputs(&env, &wid, &sb, &[("scaffold", &stuck)], "Blocked");
    assert_eq!(
        proj.head(),
        head_before,
        "Blocked scaffolding must not commit"
    );
    assert_eq!(head_before, baseline0);
    assert!(next_begin_nodes(&s4).contains(&"escalate".to_string()));
}

// Task: C002-T12
#[test]
fn verify_escalation_continue_binds_opinion_on_return() {
    let env = Env::new();
    let proj = Proj::init(&env, "g4a");
    let baseline0 = proj.head();
    let wid = start_spec_dev(&env, &proj);
    plan_and_approve(&env, &wid, &proj, &baseline0, "Conditions: none.");
    let sc = scaffold_two_files(&env, &wid, &proj);

    let im1 = env.follow_begin(&sc, "implement");
    let (t01_base, t01_commit) = commit_export_task(&proj, "sim");
    let change1 = change_done(
        "T01",
        &t01_base,
        &t01_commit,
        0,
        &["src/export.py", "tests/test_export.py"],
        "",
    );
    let s1 = submit_outputs(&env, &wid, &im1, &[("change", &change1)], "Complete T01");

    // Verification needs authorization, so report Rejected, needs human rather than repair.
    let v1 = env.follow_begin(&s1, "verify");
    assert_t01_clean(&proj, &t01_base, &t01_commit, &v1);
    let r1 = verify_report(
        "Rejected, needs human",
        "T01",
        0,
        &t01_base,
        "- Verification must run `./gate.sh --with-network`, authorization required",
    );
    let s2 = submit_outputs(&env, &wid, &v1, &[("report", &r1)], "Rejected, needs human");
    assert!(next_begin_nodes(&s2).contains(&"escalate".to_string()));

    // Human Continue grants authority; comments remain in the decision file.
    let es = env.follow_begin(&s2, "escalate");
    let opinion = "Allow running ./gate.sh --with-network, Only this command is authorized.";
    let d = escalation_md("Continue", opinion);
    let s3 = submit_outputs(&env, &wid, &es, &[("decision", &d)], "Continue");
    let verify_op = s3["next"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["op"] == "attempt begin" && n["args"]["node"] == "verify")
        .expect("escalate Continue must offer verify in next");
    assert_eq!(verify_op["edge"], "back");

    // Return to verify along the escalate back edge, binding human comments.
    let v2 = env.follow_begin(&s3, "verify");
    assert_eq!(v2["data"]["attempt"], "verify#2.0");
    let brief = brief_text(&v2);
    assert!(brief.contains("From: escalate#1 (back edge)"));
    let escalation = input_path(&v2, "escalation");
    assert!(
        escalation
            .to_str()
            .unwrap()
            .ends_with("/attempts/escalate/occurrence-001/attempt-000/outputs/decision.md")
    );
    assert!(brief.contains(&format!("| escalation | {} | ", escalation.display())));
    // Approval still binds and verifies; comments stay in the bound file, outside the brief.
    assert!(
        input_path(&v2, "decision")
            .to_str()
            .unwrap()
            .ends_with("/attempts/plan-review/occurrence-001/attempt-000/outputs/decision.md")
    );
    assert!(!brief.contains(opinion));
    assert_eq!(read(&escalation), d);

    let r2 = verify_report(
        "Accepted, next task T02",
        "T01",
        0,
        &t01_base,
        "- The human-authorized network gate was rerun and passed",
    );
    submit_outputs(
        &env,
        &wid,
        &v2,
        &[("report", &r2)],
        "Accepted, next task T02",
    );
}

// Task: C002-T12
#[test]
fn scaffold_escalation_continue_binds_opinion_on_return() {
    let env = Env::new();
    let proj = Proj::init(&env, "g4b");
    let baseline0 = proj.head();
    let wid = start_spec_dev(&env, &proj);
    plan_and_approve(&env, &wid, &proj, &baseline0, "Conditions: none.");

    // A plan/interface conflict blocks scaffolding.
    let sb1 = env.begin(&wid, "scaffold");
    assert!(brief_text(&sb1).contains("From: plan-review#1 (main edge)"));
    let stuck = "Blocked\nNotes: The plan conflicts with src/legacy.py  existing export interface; human ruling is required.\n";
    let s1 = submit_outputs(&env, &wid, &sb1, &[("scaffold", stuck)], "Blocked");
    assert_eq!(
        proj.head(),
        baseline0,
        "Blocked scaffolding must not commit"
    );
    let escalate_op = s1["next"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["op"] == "attempt begin" && n["args"]["node"] == "escalate")
        .expect("scaffold Blocked scaffolding must offer escalate in next");
    assert_eq!(escalate_op["edge"], "branch");

    // The human edits the interface and chooses Continue.
    let es = env.follow_begin(&s1, "escalate");
    let opinion =
        "I manually updated src/legacy.py  export interface; scaffold against the new signature.";
    let d = escalation_md("Continue", opinion);
    let s2 = submit_outputs(&env, &wid, &es, &[("decision", &d)], "Continue");
    let scaffold_op = s2["next"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["op"] == "attempt begin" && n["args"]["node"] == "scaffold")
        .expect("escalate Continue must offer scaffold in next");
    assert_eq!(scaffold_op["edge"], "back");

    // Return to scaffold along the escalate back edge and follow bound human comments.
    let sb2 = env.follow_begin(&s2, "scaffold");
    assert_eq!(sb2["data"]["attempt"], "scaffold#2.0");
    let brief = brief_text(&sb2);
    assert!(brief.contains("From: escalate#1 (back edge)"));
    let escalation = input_path(&sb2, "escalation");
    assert!(
        escalation
            .to_str()
            .unwrap()
            .ends_with("/attempts/escalate/occurrence-001/attempt-000/outputs/decision.md")
    );
    assert!(brief.contains(&format!("| escalation | {} | ", escalation.display())));
    assert!(!brief.contains(opinion));
    assert_eq!(read(&escalation), d);

    // Complete and submit the scaffold under the human ruling.
    std::fs::write(proj.root().join("src/export.py"), EXPORT_PY_STUB).unwrap();
    std::fs::write(
        proj.root().join("tests/test_export.py"),
        TEST_EXPORT_PY_SKIPPED,
    )
    .unwrap();
    proj.gate();
    let head = proj.commit("feat: scaffold\n\nTask: scaffold\nAgent: sim");
    let md = format!("Complete\nCommit: {head}\nFocused test command: pytest tests/ -k <Task>\n");
    submit_outputs(&env, &wid, &sb2, &[("scaffold", &md)], "Scaffold ready");
}

// Task: C002-T12
#[test]
fn replan_after_first_task_keeps_original_baseline_for_final_review() {
    let env = Env::new();
    let proj = Proj::init(&env, "g5");
    let baseline0 = proj.head();
    let wid = start_spec_dev(&env, &proj);
    plan_and_approve(&env, &wid, &proj, &baseline0, "Conditions: none.");
    let sc = scaffold_two_files(&env, &wid, &proj);

    // Task 1 completes.
    let im1 = env.follow_begin(&sc, "implement");
    let (t01_base, t01_commit) = commit_export_task(&proj, "sim");
    let change1 = change_done(
        "T01",
        &t01_base,
        &t01_commit,
        0,
        &["src/export.py", "tests/test_export.py"],
        "",
    );
    let s1 = submit_outputs(&env, &wid, &im1, &[("change", &change1)], "Complete T01");
    let v1 = env.follow_begin(&s1, "verify");
    assert_t01_clean(&proj, &t01_base, &t01_commit, &v1);
    let r1 = verify_report(
        "Accepted, next task T02",
        "T01",
        0,
        &t01_base,
        "- All checks match",
    );
    let s2 = submit_outputs(
        &env,
        &wid,
        &v1,
        &[("report", &r1)],
        "Accepted, next task T02",
    );

    // Task 2 blocks and the human revises the plan.
    let im2 = env.follow_begin(&s2, "implement");
    let s3 = submit_outputs(
        &env,
        &wid,
        &im2,
        &[(
            "change",
            &change_stuck(
                "T02",
                "Task decomposition is wrong; encoding needs a configuration item first.",
            ),
        )],
        "Blocked T02",
    );
    let es = env.follow_begin(&s3, "escalate");
    let s4 = submit_outputs(
        &env,
        &wid,
        &es,
        &[(
            "decision",
            &escalation_md(
                "Revise plan",
                "Task decomposition is wrong; rearrange the task list.",
            ),
        )],
        "Revise plan",
    );

    // Plan#2 copies the original baseline without resetting it.
    let b2 = env.follow_begin(&s4, "plan");
    assert_eq!(b2["data"]["attempt"], "plan#2.0");
    let replan_head = proj.head();
    assert_eq!(replan_head, t01_commit);
    let plan2 = plan_md(
        &baseline0,
        "- 2026-09-28 Initial version\n- 2026-09-28 Rearranged tasks; encoding remains T02\n",
    );
    let s5 = submit_outputs(
        &env,
        &wid,
        &b2,
        &[
            ("plan", &plan2),
            (
                "tasks",
                &tasks_md(&[(
                    "T02 Encoding",
                    "src/encode.py、tests/test_encode.py",
                    "test_encode",
                )]),
            ),
        ],
        "Plan rearranged",
    );
    let pr2 = env.follow_begin(&s5, "plan-review");
    let d2 = approval_md(
        &sha256_file(&input_path(&pr2, "spec")),
        &sha256_file(&input_path(&pr2, "plan")),
        "Conditions: none.",
    );
    let s6 = submit_outputs(&env, &wid, &pr2, &[("decision", &d2)], "Approve");

    // The replanned scaffold/implementation continues at T02.
    let sb2 = env.follow_begin(&s6, "scaffold");
    let s7 = submit_outputs(
        &env,
        &wid,
        &sb2,
        &[(
            "scaffold",
            "Complete\nCommit: Same signature; task IDs follow the new list\n",
        )],
        "Scaffold ready",
    );
    let im3 = env.follow_begin(&s7, "implement");
    assert_eq!(im3["data"]["attempt"], "implement#3.0");
    let t02_base = proj.head();
    std::fs::write(
        proj.root().join("src/encode.py"),
        "def encode(rows):\n    return \"|\".join(rows)\n",
    )
    .unwrap();
    std::fs::write(
        proj.root().join("tests/test_encode.py"),
        unskip(TEST_ENCODE_PY_SKIPPED),
    )
    .unwrap();
    proj.gate();
    let t02_commit = proj.commit("feat(encode): Encoding\n\nTask: T02\nAgent: sim");
    let change3 = change_done(
        "T02",
        &t02_base,
        &t02_commit,
        0,
        &["src/encode.py", "tests/test_encode.py"],
        "",
    );
    let s8 = submit_outputs(&env, &wid, &im3, &[("change", &change3)], "Complete T02");
    let v2 = env.follow_begin(&s8, "verify");
    let r2 = verify_report(
        "Accepted, all tasks complete",
        "T02",
        0,
        &t02_base,
        "- All checks match",
    );
    let s9 = submit_outputs(
        &env,
        &wid,
        &v2,
        &[("report", &r2)],
        "Accepted, all tasks complete",
    );

    // Overall review binds plan#2 but retains Work-start HEAD as baseline.
    let rv = env.follow_begin(&s9, "review");
    let plan_bound = read(&input_path(&rv, "plan"));
    assert!(
        input_path(&rv, "plan")
            .to_str()
            .unwrap()
            .ends_with("/attempts/plan/occurrence-002/attempt-000/outputs/plan.md")
    );
    assert_eq!(plan_baseline(&plan_bound), baseline0);
    assert_eq!(
        baseline0,
        proj.git(&["rev-list", "--max-parents=0", "HEAD"])
    );

    // Accepted case: overall original-baseline scope still includes pre-replan Task 1.
    let whole = proj.diff_names(&baseline0, &proj.head());
    assert_eq!(
        whole,
        [
            "src/encode.py",
            "src/export.py",
            "tests/test_encode.py",
            "tests/test_export.py"
        ]
    );
    assert!(whole.iter().any(|f| f == "src/export.py"));

    // Change one condition: resetting to plan#2 HEAD would omit Task 1 from the scope.
    let reset = proj.diff_names(&replan_head, &proj.head());
    assert_eq!(reset, ["src/encode.py", "tests/test_encode.py"]);
    assert!(!reset.iter().any(|f| f == "src/export.py"));
}
