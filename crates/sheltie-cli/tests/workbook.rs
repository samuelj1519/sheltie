//! T17：`workbook` 组。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::*;
use predicates::prelude::*;

fn make_tree_writable(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    let metadata = std::fs::symlink_metadata(path).unwrap();
    let mode = if metadata.is_dir() { 0o755 } else { 0o644 };
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
    if metadata.is_dir() {
        for entry in std::fs::read_dir(path).unwrap() {
            make_tree_writable(&entry.unwrap().path());
        }
    }
}

// Task: T17
#[test]
fn workbook_add_prints_id_version_digest() {
    let env = Env::new();
    env.cmd_text(&["workbook", "add", example_dir("two-step").to_str().unwrap()])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("two-step")
                .and(predicate::str::contains("1.0.0"))
                .and(predicate::str::is_match("[0-9a-f]{64}").unwrap()),
        );
}

// Task: T17
#[test]
fn workbook_add_json_has_ok_true_and_data() {
    let env = Env::new();
    let v = env.add_example("two-step");
    assert_eq!(v["data"]["id"], "two-step");
    assert_eq!(v["data"]["flows"], serde_json::json!(["default"]));
    assert!(v["next"].as_array().unwrap().is_empty());
}

// Task: T17
#[test]
fn workbook_add_invalid_dir_exits_1_with_workbook_invalid() {
    let env = Env::new();
    let bad = env.dir.path().join("bad");
    std::fs::create_dir_all(&bad).unwrap();
    std::fs::write(bad.join("workbook.toml"), "schema = \"workbook/v9\"\n").unwrap();
    let (v, code) = env.fail(&["workbook", "add", bad.to_str().unwrap()]);
    assert_eq!(code, 1);
    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["code"], "WORKBOOK_INVALID");
}

// Task: T17
#[test]
fn workbook_list_after_add_shows_one_row_marked_latest() {
    let env = Env::new();
    env.add_example("two-step");
    let v = env.ok(&["workbook", "list"]);
    let rows = v["data"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["latest"], true);
    env.cmd_text(&["workbook", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("two-step"));
}

// Task: T17
#[test]
fn workbook_show_lists_nodes_edges_and_requires() {
    let env = Env::new();
    env.add_example("article-review");
    let v = env.ok(&["workbook", "show", "article-review"]);
    assert_eq!(v["data"]["flows"][0]["nodes"].as_array().unwrap().len(), 3);
    assert_eq!(v["data"]["flows"][0]["edges"].as_array().unwrap().len(), 3);
    assert!(v["data"]["requires"].as_array().unwrap().is_empty());
}

// Task: T17
#[test]
fn workbook_remove_without_version_exits_2() {
    let env = Env::new();
    env.add_example("two-step");
    let (_v, code) = env.fail(&["workbook", "remove", "two-step"]);
    assert_eq!(code, 2);
}

// Task: T17
#[test]
fn workbook_remove_then_list_is_empty() {
    let env = Env::new();
    env.add_example("two-step");
    env.ok(&["workbook", "remove", "two-step@1.0.0"]);
    assert!(
        env.ok(&["workbook", "list"])["data"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

// Task: T17
#[test]
fn workbook_verify_exits_1_after_tamper() {
    let env = Env::new();
    env.add_example("two-step");
    env.ok(&["workbook", "verify"]);
    let f = env
        .workbook_dir("two-step", "1.0.0")
        .join("instructions/outline.md");
    make_writable(&f);
    std::fs::write(&f, "改了").unwrap();
    let (v, code) = env.fail(&["workbook", "verify"]);
    assert_eq!(code, 1);
    assert_eq!(v["error"]["code"], "WORKBOOK_TAMPERED");
}

// Task: C002-T02
#[test]
fn show_lists_ordered_start_inputs_in_json_and_text() {
    let env = Env::new();
    env.add_example("article-review");
    // article-review 只有 draft 声明 start.topic；协调者不用失败 start 探测。
    let v = env.ok(&["workbook", "show", "article-review"]);
    assert_eq!(
        v["data"]["flows"][0]["start_inputs"],
        serde_json::json!(["topic"])
    );
    let out = env
        .cmd_text(&["workbook", "show", "article-review"])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("起始输入: topic"), "{text}");

    // 没有起始输入的 Flow 写「无」。用临时目录造一个无键 Workbook。
    let src = env.dir.path().join("nokeys");
    std::fs::create_dir_all(src.join("flows")).unwrap();
    std::fs::write(
        src.join("workbook.toml"),
        "schema = \"workbook/v1\"\nid = \"nokeys\"\nversion = \"1.0.0\"\nname = \"无键\"\nflows = [\"flows/default.toml\"]\n",
    )
    .unwrap();
    std::fs::write(
        src.join("flows/default.toml"),
        "schema = \"flow/v1\"\nid = \"default\"\nentry = \"only\"\n\n[[nodes]]\nid = \"only\"\ntitle = \"唯一\"\nexecutor = \"agent\"\ninstruction = { text = \"做。\" }\n",
    )
    .unwrap();
    env.ok(&["workbook", "add", src.to_str().unwrap()]);
    let v = env.ok(&["workbook", "show", "nokeys"]);
    assert_eq!(v["data"]["flows"][0]["start_inputs"], serde_json::json!([]));
}

// Task: C002-T28
#[test]
fn cleanup_warning_goes_to_stderr_without_changing_success_json_or_replaying_business_effects() {
    let env = Env::new();
    let mut command = env.cmd(&[
        "workbook",
        "add",
        example_dir("two-step").to_str().unwrap(),
        "--request-id",
        "t28-cli-cleanup-warning",
    ]);
    command.env("SHELTIE_FAILPOINT", "pending_cleanup_before_remove");
    let output = command.output().unwrap();
    assert!(output.status.success());
    let response: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["ok"], true);
    assert_eq!(response["data"]["id"], "two-step");
    let warning = String::from_utf8_lossy(&output.stderr);
    assert!(warning.contains("warning: pending维护"));
    assert!(
        warning.contains("request_id=t28-cli-cleanup-warning"),
        "{warning}"
    );
    assert!(warning.contains("object=pending/"), "{warning}");
    assert!(warning.contains("reason=清理失败"), "{warning}");

    let replay = env.ok(&[
        "workbook",
        "add",
        example_dir("two-step").to_str().unwrap(),
        "--request-id",
        "t28-cli-cleanup-warning",
    ]);
    assert_eq!(replay["ok"], true);
    assert_eq!(replay["data"]["replayed"], true);
    let connection = rusqlite::Connection::open(env.dir.path().join("store.db")).unwrap();
    let request_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM requests WHERE request_id = 't28-cli-cleanup-warning'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(request_count, 1);
    assert!(env.workbook_dir("two-step", "1.0.0").is_dir());
}

// Task: C002-T28
#[test]
fn workbook_views_read_a_committed_pending_add_without_recovery() {
    let env = Env::new();
    let added = env.ok(&[
        "workbook",
        "add",
        example_dir("two-step").to_str().unwrap(),
        "--request-id",
        "t28-cli-pending-workbook",
    ]);
    assert_eq!(added["ok"], true);
    let connection = rusqlite::Connection::open(env.dir.path().join("store.db")).unwrap();
    let effects: String = connection
        .query_row(
            "SELECT effects_json FROM requests WHERE request_id = 't28-cli-pending-workbook'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let effects: serde_json::Value = serde_json::from_str(&effects).unwrap();
    let pending = effects[0]["pending"].as_str().unwrap();
    let internal_id = pending.split('/').nth(1).unwrap();
    let payload = env.dir.path().join(pending);
    std::fs::create_dir_all(payload.parent().unwrap()).unwrap();
    std::fs::write(
        env.dir.path().join(format!("pending/{internal_id}.owner")),
        format!(
            "{}\n",
            serde_json::to_string(&serde_json::json!({
                "format": "pending/v1",
                "internal_id": internal_id,
                "request_id": "t28-cli-pending-workbook",
                "op": "add_workbook",
            }))
            .unwrap()
        ),
    )
    .unwrap();
    connection
        .execute(
            "UPDATE requests SET published = 0 WHERE request_id = 't28-cli-pending-workbook'",
            [],
        )
        .unwrap();
    drop(connection);
    let workbook_parent = env
        .workbook_dir("two-step", "1.0.0")
        .parent()
        .unwrap()
        .to_path_buf();
    make_tree_writable(&workbook_parent);
    std::fs::rename(env.workbook_dir("two-step", "1.0.0"), &payload).unwrap();
    let lock_path = env.dir.path().join(".lock");
    let store_before = std::fs::read(env.dir.path().join("store.db")).unwrap();
    std::fs::remove_file(&lock_path).unwrap();

    let listed = env.ok(&["workbook", "list"]);
    assert_eq!(listed["data"][0]["pending_publish"], true);
    let shown = env.ok(&["workbook", "show", "two-step"]);
    assert_eq!(shown["data"]["pending_publish"], true);
    let verified = env.ok(&["workbook", "verify", "two-step@1.0.0"]);
    assert_eq!(verified["data"]["results"][0]["status"], "ok");
    assert_eq!(verified["data"]["results"][0]["pending_publish"], true);
    assert_eq!(
        std::fs::read(env.dir.path().join("store.db")).unwrap(),
        store_before
    );
    assert!(!lock_path.exists(), "只读命令不能创建HomeLock");
    assert!(!env.workbook_dir("two-step", "1.0.0").exists());
    assert!(payload.is_dir());
}

// Task: C002-T28
#[test]
fn successful_new_add_keeps_json_success_when_old_completed_effects_break_maintenance() {
    let env = Env::new();
    env.add_example("two-step");
    let connection = rusqlite::Connection::open(env.dir.path().join("store.db")).unwrap();
    let old_request: String = connection
        .query_row("SELECT request_id FROM requests", [], |row| row.get(0))
        .unwrap();
    connection
        .execute(
            "UPDATE requests SET effects_json = 'broken-json' WHERE request_id = ?1",
            [&old_request],
        )
        .unwrap();
    drop(connection);
    let output = env
        .cmd(&[
            "workbook",
            "add",
            example_dir("gated-release").to_str().unwrap(),
            "--request-id",
            "t28-new-business-success",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let response: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["ok"], true);
    assert_eq!(response["request_id"], "t28-new-business-success");
    assert_eq!(response["data"]["id"], "gated-release");
    let warning = String::from_utf8_lossy(&output.stderr);
    assert!(
        warning.contains(&format!("request_id={old_request}")),
        "{warning}"
    );
    assert!(
        warning.contains(&format!("object=requests/{old_request}.effects_json")),
        "{warning}"
    );
    assert!(warning.contains("reason="), "{warning}");
    assert!(env.workbook_dir("gated-release", "1.0.0").is_dir());
}
