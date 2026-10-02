#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::store::store_rows;
use common::*;
use serde_json::{Value, json};
use std::path::Path;

fn fixture(gate: bool, selected: bool) -> (Env, tempfile::TempDir, String) {
    let source = tempfile::tempdir().unwrap();
    std::fs::create_dir(source.path().join("flows")).unwrap();
    std::fs::write(
        source.path().join("workbook.toml"),
        "schema = \"workbook/v1\"\nid = \"result-fixture\"\nversion = \"1.0.0\"\nname = \"结果夹具\"\nflows = [\"flows/default.toml\"]\n",
    )
    .unwrap();
    std::fs::write(
        source.path().join("flows/default.toml"),
        format!(
            r#"schema = "flow/v1"
id = "default"
entry = "finish"
[[nodes]]
id = "finish"
title = "完成"
executor = "agent"
instruction = {{ text = "写明实际结果" }}
gate = {gate}
inputs = [{{ name = "original", from = "start.topic", result = {selected} }}]
outputs = [{{ name = "report", path = "report.md", result = {selected} }}]
"#
        ),
    )
    .unwrap();
    let env = Env::new();
    env.ok(&["workbook", "add", source.path().to_str().unwrap()]);
    let work = env.start("result-fixture", &[("topic", "original input")]);
    (env, source, work)
}

fn submit(env: &Env, work: &str) -> Value {
    let begin = env.begin(work, "finish");
    std::fs::write(
        begin["data"]["outputs"]["report"].as_str().unwrap(),
        b"result report\n",
    )
    .unwrap();
    env.ok(&[
        "attempt",
        "submit",
        work,
        "--attempt",
        "finish#1.0",
        "--summary",
        "报告已完成，内容由使用者核对",
    ])
}

// Task: C004-T02
#[test]
#[ignore = "C004-T02"]
fn result_lists_explicit_terminal_bindings_and_sealed_outputs_without_writes() {
    let (env, _source, work) = fixture(false, true);
    submit(&env, &work);
    let before = store_rows(&env);
    let card_before = std::fs::read(env.work_dir(&work).join("status-card.md")).unwrap();
    let result = env.ok(&["work", "result", &work]);
    let data = &result["data"];
    assert_eq!(data["format"], "work-result/v1");
    assert_eq!(data["work_id"], work);
    assert_eq!(data["revision"], 3);
    assert_eq!(data["workbook"]["id"], "result-fixture");
    assert_eq!(data["flow"], "default");
    assert_eq!(data["status"], json!({"kind": "succeeded"}));
    assert_eq!(data["effects_pending"], false);
    assert_eq!(data["final"], true);
    assert_eq!(result["next"], json!([]));
    let artifacts = data["artifacts"].as_array().unwrap();
    assert_eq!(artifacts.len(), 2);
    assert_eq!(artifacts[0]["key"], "original");
    assert_eq!(
        artifacts[0]["source"],
        json!({"attempt": "finish#1.0", "kind": "input", "name": "original"})
    );
    assert_eq!(
        artifacts[0]["path"],
        env.work_dir(&work)
            .join("start-inputs/topic")
            .to_str()
            .unwrap()
    );
    assert_eq!(artifacts[0]["bytes"], 14);
    assert_eq!(artifacts[1]["key"], "report");
    assert_eq!(
        artifacts[1]["source"],
        json!({"attempt": "finish#1.0", "kind": "output", "name": "report"})
    );
    assert_eq!(artifacts[1]["bytes"], 14);
    assert_eq!(
        artifacts[1]["sha256"],
        "23d5ac43e5b70dce4db5a748c698a9053dc4bfd6df16643e2f430383b8549c60"
    );
    let text = env.cmd_text(&["work", "result", &work]).output().unwrap();
    assert!(text.status.success());
    let text = String::from_utf8(text.stdout).unwrap();
    for item in artifacts {
        for field in ["key", "path", "sha256"] {
            assert!(
                text.contains(item[field].as_str().unwrap()),
                "{field} 未显示"
            );
        }
    }
    assert_eq!(store_rows(&env), before);
    assert_eq!(
        std::fs::read(env.work_dir(&work).join("status-card.md")).unwrap(),
        card_before
    );
}

// Task: C004-T02
#[test]
#[ignore = "C004-T02"]
fn result_is_empty_until_the_terminal_gate_is_approved() {
    let (env, _source, work) = fixture(true, true);
    submit(&env, &work);
    let blocked = env.ok(&["work", "result", &work]);
    assert_eq!(
        blocked["data"]["status"],
        json!({"kind": "blocked", "reason": "gate"})
    );
    assert_eq!(blocked["data"]["final"], false);
    assert_eq!(blocked["data"]["artifacts"], json!([]));
    env.ok(&["gate", "approve", &work, "--node", "finish"]);
    let approved = env.ok(&["work", "result", &work]);
    assert_eq!(approved["data"]["revision"], 4);
    assert_eq!(approved["data"]["final"], true);
    assert_eq!(approved["data"]["artifacts"].as_array().unwrap().len(), 2);
}

// Task: C004-T02
#[test]
#[ignore = "C004-T02"]
fn result_distinguishes_no_selection_from_cancelled_and_active_work() {
    let (env, _source, work) = fixture(false, false);
    let active = env.ok(&["work", "result", &work]);
    assert_eq!(active["data"]["final"], false);
    assert_eq!(active["data"]["artifacts"], json!([]));
    submit(&env, &work);
    let empty = env.ok(&["work", "result", &work]);
    assert_eq!(empty["data"]["final"], true);
    assert_eq!(empty["data"]["artifacts"], json!([]));
    let text = env.cmd_text(&["work", "result", &work]).output().unwrap();
    assert!(
        String::from_utf8(text.stdout)
            .unwrap()
            .contains("未声明最终成果")
    );
    let (cancelled_env, _source, cancelled_work) = fixture(false, true);
    cancelled_env.begin(&cancelled_work, "finish");
    cancelled_env.ok(&["work", "cancel", &cancelled_work]);
    let cancelled = cancelled_env.ok(&["work", "result", &cancelled_work]);
    assert_eq!(cancelled["data"]["status"], json!({"kind": "cancelled"}));
    assert_eq!(cancelled["data"]["final"], false);
    assert_eq!(cancelled["data"]["artifacts"], json!([]));
}

// Task: C004-T02
#[test]
#[ignore = "C004-T02"]
fn status_provides_the_current_brief_frozen_inputs_and_running_drafts() {
    let (env, _source, work) = fixture(false, true);
    let initial = env.status(&work);
    assert!(initial["data"]["resume"].is_null());
    let begun = env.begin(&work, "finish");
    let before = store_rows(&env);
    let status = env.status(&work);
    assert_eq!(status["data"]["revision"], 2);
    assert_eq!(status["data"]["effects_pending"], false);
    assert_eq!(status["data"]["pending_publish"], false);
    let resume = &status["data"]["resume"];
    assert_eq!(resume["attempt"], "finish#1.0");
    assert_eq!(resume["brief_path"], begun["data"]["brief_path"]);
    assert_eq!(
        resume["inputs"]["original"]["path"],
        begun["data"]["inputs"]["original"]
    );
    assert_eq!(resume["inputs"]["original"]["bytes"], 14);
    assert_eq!(resume["draft_outputs"], begun["data"]["outputs"]);
    assert!(!Path::new(resume["draft_outputs"]["report"].as_str().unwrap()).exists());
    let text = env.cmd_text(&["work", "status", &work]).output().unwrap();
    let text = String::from_utf8(text.stdout).unwrap();
    for path in [
        resume["brief_path"].as_str().unwrap(),
        resume["inputs"]["original"]["path"].as_str().unwrap(),
        resume["draft_outputs"]["report"].as_str().unwrap(),
    ] {
        assert!(text.contains(path));
    }
    assert_eq!(store_rows(&env), before);
    std::fs::write(
        begun["data"]["outputs"]["report"].as_str().unwrap(),
        b"done",
    )
    .unwrap();
    env.ok(&[
        "attempt",
        "submit",
        &work,
        "--attempt",
        "finish#1.0",
        "--summary",
        "done",
    ]);
    let ended = env.status(&work);
    assert_eq!(ended["data"]["resume"]["attempt"], "finish#1.0");
    assert_eq!(ended["data"]["resume"]["draft_outputs"], json!({}));
}

// Task: C004-T02
#[test]
#[ignore = "C004-T02"]
fn result_rejects_request_id_before_opening_a_store() {
    let env = Env::new();
    let (error, code) = env.fail(&[
        "--request-id",
        "01930000-0000-7000-8000-000000000001",
        "work",
        "result",
        "missing",
    ]);
    assert_eq!(code, 2);
    assert_eq!(error["error"]["code"], "INVALID_REQUEST");
    assert!(!env.dir.path().join("store.db").exists());
    assert!(!env.dir.path().join(".lock").exists());
}

// Task: C004-T02
#[test]
#[ignore = "C004-T02"]
fn result_returns_frozen_references_without_claiming_source_bytes_were_rechecked() {
    let (env, _source, work) = fixture(false, true);
    submit(&env, &work);
    let original = env.ok(&["work", "result", &work]);
    let path = Path::new(original["data"]["artifacts"][1]["path"].as_str().unwrap());
    make_writable(path);
    std::fs::write(path, b"edited source").unwrap();
    let again = env.ok(&["work", "result", &work]);
    assert_eq!(again, original);
    assert_eq!(std::fs::read(path).unwrap(), b"edited source");
}

// Task: C004-T02
#[test]
#[ignore = "C004-T02"]
fn result_rejects_a_published_request_with_invalid_effects_without_repairing_it() {
    let (env, _source, work) = fixture(false, true);
    submit(&env, &work);
    let connection = rusqlite::Connection::open(env.dir.path().join("store.db")).unwrap();
    connection.execute("UPDATE requests SET effects_json='[{\"kind\":\"unknown\"}]' WHERE work_id=?1 AND published=1", [&work]).unwrap();
    let before = store_rows(&env);
    for operation in ["status", "result"] {
        let (error, code) = env.fail(&["work", operation, &work]);
        assert_eq!(code, 1);
        assert_eq!(error["error"]["code"], "STORE_CORRUPT");
    }
    assert_eq!(store_rows(&env), before);
}

// Task: C004-T02
#[cfg(feature = "failpoint")]
#[test]
#[ignore = "C004-T02"]
fn result_hides_committed_outputs_until_their_file_effects_finish() {
    let (env, _source, work) = fixture(false, true);
    let begin = env.begin(&work, "finish");
    std::fs::write(
        begin["data"]["outputs"]["report"].as_str().unwrap(),
        b"result report\n",
    )
    .unwrap();
    let args = [
        "--request-id",
        "01930000-0000-7000-8000-000000000002",
        "attempt",
        "submit",
        &work,
        "--attempt",
        "finish#1.0",
        "--summary",
        "completed",
    ];
    let stopped = env
        .cmd(&args)
        .env("SHELTIE_FAILPOINT", "after_commit_before_effects")
        .output()
        .unwrap();
    assert!(!stopped.status.success());
    let before = store_rows(&env);
    let result = env.ok(&["work", "result", &work]);
    assert_eq!(result["data"]["status"], json!({"kind": "succeeded"}));
    assert_eq!(result["data"]["effects_pending"], true);
    assert_eq!(result["data"]["final"], false);
    assert_eq!(result["data"]["artifacts"], json!([]));
    assert_eq!(store_rows(&env), before);
    env.ok(&args);
    assert_eq!(env.ok(&["work", "result", &work])["data"]["final"], true);
}
