#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::store::store_rows;
use common::*;
use serde_json::{Value, json};
use std::path::Path;

fn fixture(retries: u32, gate: bool) -> (Env, tempfile::TempDir, String) {
    let source = tempfile::tempdir().unwrap();
    std::fs::create_dir(source.path().join("flows")).unwrap();
    std::fs::write(source.path().join("workbook.toml"), "schema = 'workbook/v1'\nid = 'replace-fixture'\nversion = '1.0.0'\nname = 'Replacement fixture'\nflows = ['flows/default.toml']\n").unwrap();
    std::fs::write(source.path().join("flows/default.toml"), format!(
        "schema = 'flow/v1'\nid = 'default'\nentry = 'execute'\n[[nodes]]\nid = 'execute'\ntitle = 'Execute'\nexecutor = 'agent'\ninstruction = {{ text = 'Complete the report from frozen inputs' }}\ninputs = [{{ name = 'task', from = 'start.task' }}, {{ name = 'facts', from = 'engine.stats' }}]\noutputs = [{{ name = 'report', path = 'report.md', result = true }}]\nmax_retries = {retries}\ngate = {gate}\n"
    )).unwrap();
    let env = Env::new();
    env.ok(&["workbook", "add", source.path().to_str().unwrap()]);
    let work = env.start("replace-fixture", &[("task", "original frozen goal")]);
    (env, source, work)
}

fn replace(env: &Env, work: &str, old: &str, reason: &str, request: &str) -> Value {
    env.ok(&[
        "--request-id",
        request,
        "attempt",
        "replace",
        work,
        "--attempt",
        old,
        "--reason",
        reason,
    ])
}

fn state(env: &Env, work: &str) -> Value {
    let connection = rusqlite::Connection::open(env.dir.path().join("store.db")).unwrap();
    let raw: String = connection
        .query_row(
            "SELECT state_json FROM works WHERE work_id=?1",
            [work],
            |row| row.get(0),
        )
        .unwrap();
    serde_json::from_str(&raw).unwrap()
}

// Task: C005-T02
#[test]
fn replacement_is_atomic_and_business_failures_use_history_not_attempt_number() {
    let (env, _source, work) = fixture(1, false);
    let original = env.begin(&work, "execute");
    let old_output = Path::new(original["data"]["outputs"]["report"].as_str().unwrap());
    std::fs::write(old_output, b"old unsealed draft").unwrap();
    let replaced = replace(
        &env,
        &work,
        "execute#1.0",
        "Revoke submission qualification",
        "replace-once",
    );
    assert_eq!(replaced["revision"], 3);
    assert_eq!(replaced["data"]["replaced_attempt"], "execute#1.0");
    assert_eq!(replaced["data"]["attempt"], "execute#1.1");
    assert_eq!(replaced["data"]["number"], 1);
    assert!(replaced["data"].get("retry").is_none());
    assert_eq!(
        replaced["data"]["inputs"]["task"],
        original["data"]["inputs"]["task"]
    );
    assert_ne!(
        replaced["data"]["inputs"]["facts"],
        original["data"]["inputs"]["facts"]
    );
    assert_eq!(std::fs::read(old_output).unwrap(), b"old unsealed draft");
    assert!(!Path::new(replaced["data"]["outputs"]["report"].as_str().unwrap()).exists());
    let persisted = state(&env, &work);
    assert_eq!(persisted["attempts"].as_array().unwrap().len(), 2);
    assert_eq!(persisted["attempts"][0]["status"], "superseded");
    assert_eq!(
        persisted["attempts"][0]["replacement_reason"],
        "Revoke submission qualification"
    );
    assert!(persisted["attempts"][0]["ended_at"].is_string());
    assert_eq!(persisted["attempts"][0]["outputs"], json!({}));
    assert_eq!(persisted["attempts"][1]["status"], "running");
    assert_eq!(persisted["attempts"][1]["replacement_reason"], Value::Null);
    let statistics: Value = serde_json::from_slice(
        &std::fs::read(replaced["data"]["inputs"]["facts"].as_str().unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(statistics["nodes"][0]["attempts"], 2);
    assert_eq!(statistics["nodes"][0]["superseded"], 1);
    assert_eq!(statistics["nodes"][0]["failed"], 0);
    for operation in ["submit", "fail"] {
        let argument = if operation == "submit" {
            "--summary"
        } else {
            "--reason"
        };
        let before = store_rows(&env);
        let (error, code) = env.fail(&[
            "attempt",
            operation,
            &work,
            "--attempt",
            "execute#1.0",
            argument,
            "late",
        ]);
        assert_eq!(code, 1);
        assert_eq!(error["error"]["code"], "ATTEMPT_NOT_RUNNING");
        assert_eq!(store_rows(&env), before);
    }
    let before = store_rows(&env);
    let (exhausted, _) = env.fail(&[
        "attempt",
        "replace",
        &work,
        "--attempt",
        "execute#1.1",
        "--reason",
        "second",
    ]);
    assert_eq!(exhausted["error"]["code"], "REPLACEMENTS_EXHAUSTED");
    assert_eq!(store_rows(&env), before);
    let failed = env.ok(&[
        "--request-id",
        "first-real-failure",
        "attempt",
        "fail",
        &work,
        "--attempt",
        "execute#1.1",
        "--reason",
        "actual execution failed",
    ]);
    assert_eq!(failed["data"]["work_status"], json!({"kind": "active"}));
    let retry = env.begin(&work, "execute");
    assert_eq!(retry["data"]["attempt"], "execute#1.2");
    assert_eq!(retry["data"]["number"], 2);
    let blocked = env.ok(&[
        "attempt",
        "fail",
        &work,
        "--attempt",
        "execute#1.2",
        "--reason",
        "second real failure",
    ]);
    assert_eq!(
        blocked["data"]["work_status"],
        json!({"kind": "blocked", "reason": "retries_exhausted"})
    );
    let replay = env.ok(&[
        "--request-id",
        "first-real-failure",
        "attempt",
        "fail",
        &work,
        "--attempt",
        "execute#1.1",
        "--reason",
        "actual execution failed",
    ]);
    assert_eq!(replay["data"]["work_status"], json!({"kind": "active"}));
    assert_eq!(replay["data"]["replayed"], true);
    assert_eq!(replay["next"], failed["next"]);
    let stats = env.ok(&["work", "stats", &work]);
    assert_eq!(stats["data"]["nodes"][0]["attempts"], 3);
    assert_eq!(stats["data"]["nodes"][0]["failed"], 2);
    assert_eq!(stats["data"]["nodes"][0]["superseded"], 1);
}

// Task: C005-T02
#[test]
fn replacement_does_not_consume_zero_business_retries_or_approve_a_gate() {
    let (env, _source, work) = fixture(0, true);
    env.begin(&work, "execute");
    let replaced = replace(
        &env,
        &work,
        "execute#1.0",
        "administrative change",
        "replace-zero",
    );
    let submitted = env.submit_all(&work, &replaced, "Execution complete; awaiting gate");
    assert_eq!(
        submitted["data"]["work_status"],
        json!({"kind": "blocked", "reason": "gate"})
    );
    assert_eq!(env.ok(&["work", "result", &work])["data"]["final"], false);
    env.ok(&["gate", "approve", &work, "--node", "execute"]);
    assert_eq!(env.ok(&["work", "result", &work])["data"]["final"], true);
    let (error, _) = env.fail(&[
        "attempt",
        "submit",
        &work,
        "--attempt",
        "execute#1.0",
        "--summary",
        "late",
    ]);
    assert_eq!(error["error"]["code"], "WORK_TERMINAL");
    let (failed_env, _source, failed_work) = fixture(0, false);
    failed_env.begin(&failed_work, "execute");
    replace(
        &failed_env,
        &failed_work,
        "execute#1.0",
        "administrative",
        "zero-failure-replace",
    );
    let failed = failed_env.ok(&[
        "attempt",
        "fail",
        &failed_work,
        "--attempt",
        "execute#1.1",
        "--reason",
        "first real failure",
    ]);
    assert_eq!(
        failed["data"]["work_status"],
        json!({"kind": "blocked", "reason": "retries_exhausted"})
    );
}

// Task: C005-T02
#[test]
fn replacement_replays_original_file_reason_and_rejects_conflicting_intent() {
    let (env, source, work) = fixture(1, false);
    env.begin(&work, "execute");
    let reason_file = source.path().join("reason.txt");
    std::fs::write(&reason_file, "saved reason").unwrap();
    let argument = format!("@{}", reason_file.display());
    let original = replace(&env, &work, "execute#1.0", &argument, "file-replace");
    std::fs::remove_file(&reason_file).unwrap();
    let before = store_rows(&env);
    let replay = replace(&env, &work, "execute#1.0", &argument, "file-replace");
    let mut expected = original;
    expected["data"]["replayed"] = json!(true);
    assert_eq!(replay, expected);
    assert_eq!(store_rows(&env), before);
    let (conflict, _) = env.fail(&[
        "--request-id",
        "file-replace",
        "attempt",
        "replace",
        &work,
        "--attempt",
        "execute#1.0",
        "--reason",
        "different",
    ]);
    assert_eq!(conflict["error"]["code"], "REQUEST_CONFLICT");
    assert_eq!(store_rows(&env), before);
}

// Task: C005-T02
#[test]
fn replacement_reason_has_exact_limit_and_missing_identity_is_not_found() {
    for length in [4096, 4097] {
        let (env, _source, work) = fixture(1, false);
        env.begin(&work, "execute");
        let reason = "x".repeat(length);
        let before = store_rows(&env);
        if length == 4096 {
            let accepted = replace(&env, &work, "execute#1.0", &reason, "reason-boundary");
            assert_eq!(accepted["data"]["attempt"], "execute#1.1");
        } else {
            let (error, _) = env.fail(&[
                "attempt",
                "replace",
                &work,
                "--attempt",
                "execute#1.0",
                "--reason",
                &reason,
            ]);
            assert_eq!(error["error"]["code"], "SUMMARY_TOO_LONG");
            assert_eq!(store_rows(&env), before);
        }
    }
    let (env, _source, work) = fixture(1, false);
    env.begin(&work, "execute");
    let before = store_rows(&env);
    let (error, _) = env.fail(&[
        "attempt",
        "replace",
        &work,
        "--attempt",
        "execute#1.9",
        "--reason",
        "missing",
    ]);
    assert_eq!(error["error"]["code"], "NOT_FOUND");
    assert_eq!(store_rows(&env), before);
}

// Task: C005-T02
#[test]
fn replacement_refuses_modified_frozen_input_without_revoking_the_running_attempt() {
    let (env, _source, work) = fixture(1, false);
    let begun = env.begin(&work, "execute");
    let input = Path::new(begun["data"]["inputs"]["task"].as_str().unwrap());
    make_writable(input);
    std::fs::write(input, b"modified frozen goal").unwrap();
    let before = store_rows(&env);
    let (error, _) = env.fail(&[
        "--request-id",
        "modified-input-replace",
        "attempt",
        "replace",
        &work,
        "--attempt",
        "execute#1.0",
        "--reason",
        "administrative",
    ]);
    assert_eq!(error["error"]["code"], "ARTIFACT_MODIFIED");
    assert_eq!(store_rows(&env), before);
    assert_eq!(state(&env, &work)["attempts"][0]["status"], "running");
    std::fs::write(input, b"original frozen goal").unwrap();
    assert_eq!(
        replace(
            &env,
            &work,
            "execute#1.0",
            "administrative",
            "modified-input-replace"
        )["data"]["attempt"],
        "execute#1.1"
    );
}

// Task: C005-T02
#[cfg(feature = "failpoint")]
#[test]
fn replacement_crash_windows_preserve_atomic_state_and_exact_brief_and_stats() {
    for point in ["before_commit", "after_commit_before_effects"] {
        let (env, _source, work) = fixture(1, false);
        env.begin(&work, "execute");
        let args = [
            "--request-id",
            "crash-replace",
            "attempt",
            "replace",
            &work,
            "--attempt",
            "execute#1.0",
            "--reason",
            "window",
        ];
        let before = store_rows(&env);
        let stopped = env
            .cmd(&args)
            .env("SHELTIE_FAILPOINT", point)
            .output()
            .unwrap();
        assert!(!stopped.status.success());
        if point == "before_commit" {
            assert_eq!(store_rows(&env), before);
        } else {
            let value = state(&env, &work);
            assert_eq!(value["attempts"][0]["status"], "superseded");
            assert_eq!(value["attempts"][1]["status"], "running");
            assert_eq!(env.status(&work)["data"]["effects_pending"], true);
        }
        let recovered = env.ok(&args);
        let brief = Path::new(recovered["data"]["brief_path"].as_str().unwrap());
        let stats = Path::new(recovered["data"]["inputs"]["facts"].as_str().unwrap());
        let brief_bytes = std::fs::read(brief).unwrap();
        let stats_bytes = std::fs::read(stats).unwrap();
        let connection = rusqlite::Connection::open(env.dir.path().join("store.db")).unwrap();
        let registered: String = connection
            .query_row(
                "SELECT effects_json FROM requests WHERE request_id='crash-replace'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let effects: Value = serde_json::from_str(&registered).unwrap();
        for (path, bytes) in [(brief, &brief_bytes), (stats, &stats_bytes)] {
            let relative = path
                .strip_prefix(env.dir.path().canonicalize().unwrap())
                .unwrap()
                .to_str()
                .unwrap();
            let entry = effects
                .as_array()
                .unwrap()
                .iter()
                .find(|entry| entry["kind"] == "write_file" && entry["path"] == relative)
                .unwrap();
            assert_eq!(entry["content"].as_str().unwrap().as_bytes(), bytes);
        }
        std::fs::remove_file(brief).unwrap();
        std::fs::remove_file(stats).unwrap();
        let replay = env.ok(&args);
        assert_eq!(replay["data"]["replayed"], true);
        assert_eq!(std::fs::read(brief).unwrap(), brief_bytes);
        assert_eq!(std::fs::read(stats).unwrap(), stats_bytes);
        assert_eq!(state(&env, &work)["attempts"].as_array().unwrap().len(), 2);
    }
}
