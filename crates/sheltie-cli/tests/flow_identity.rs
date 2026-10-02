//! Workbook 内跨文件的 Flow 身份校验。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::*;
use rusqlite::Connection;
use serde_json::json;
use std::path::Path;

fn write_flow(path: &Path, id: &str) {
    std::fs::write(
        path,
        format!(
            "schema = \"flow/v1\"\nid = \"{id}\"\nentry = \"only\"\n\n[[nodes]]\nid = \"only\"\ntitle = \"唯一\"\nexecutor = \"agent\"\ninstruction = {{ text = \"做。\" }}\n"
        ),
    )
    .unwrap();
}

fn write_workbook(root: &Path, second_id: &str) {
    std::fs::create_dir_all(root.join("flows")).unwrap();
    std::fs::write(
        root.join("workbook.toml"),
        "schema = \"workbook/v1\"\nid = \"two-flows\"\nversion = \"1.0.0\"\nname = \"两张图\"\nflows = [\"flows/first.toml\", \"flows/second.toml\"]\n",
    )
    .unwrap();
    write_flow(&root.join("flows/first.toml"), "default");
    write_flow(&root.join("flows/second.toml"), second_id);
}

// Task: C002-T39
#[test]
fn different_flow_ids_can_be_added_shown_and_started() {
    let env = Env::new();
    let source = tempfile::tempdir().unwrap();
    write_workbook(source.path(), "secondary");

    let added = env.ok(&["workbook", "add", source.path().to_str().unwrap()]);
    assert_eq!(added["data"]["flows"], json!(["default", "secondary"]));
    let shown = env.ok(&["workbook", "show", "two-flows@1.0.0"]);
    assert_eq!(shown["data"]["flows"][0]["id"], "default");
    assert_eq!(shown["data"]["flows"][1]["id"], "secondary");
    for flow in ["default", "secondary"] {
        let started = env.ok(&[
            "work",
            "start",
            "--workbook",
            "two-flows@1.0.0",
            "--flow",
            flow,
        ]);
        let work_id = started["data"]["work_id"].as_str().unwrap();
        let status = env.status(work_id);
        assert_eq!(status["data"]["flow"], flow);
        assert_eq!(next_begin_nodes(&status), ["only"]);
    }
}

// Task: C002-T39
#[test]
fn duplicate_flow_id_is_rejected_before_commit_and_same_request_can_retry() {
    let env = Env::new();
    let source = tempfile::tempdir().unwrap();
    write_workbook(source.path(), "default");
    let args = [
        "--request-id",
        "duplicate-flow-id",
        "workbook",
        "add",
        source.path().to_str().unwrap(),
    ];

    let (reply, code) = env.fail(&args);
    assert_eq!(code, 1);
    assert_eq!(reply["error"]["code"], "FLOW_INVALID");
    assert_eq!(reply["error"]["detail"]["rule"], "1");
    assert_eq!(reply["error"]["detail"]["path"], "flows/second.toml.id");
    assert!(
        reply["error"]["detail"]["reason"]
            .as_str()
            .unwrap()
            .contains("default")
    );
    assert!(reply.get("committed").is_none());
    let connection = Connection::open(env.dir.path().join("store.db")).unwrap();
    for table in ["workbooks", "works", "requests", "audit", "work_sequence"] {
        let count = connection
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get::<_, i64>(0)
            })
            .unwrap();
        assert_eq!(count, 0, "拒绝后不得新增 {table} 行");
    }
    assert!(!env.workbook_dir("two-flows", "1.0.0").exists());

    let independent = env.add_example("two-step");
    assert_eq!(independent["data"]["id"], "two-step");
    write_flow(&source.path().join("flows/second.toml"), "secondary");
    let retried = env.ok(&args);
    assert_eq!(retried["request_id"], "duplicate-flow-id");
    assert_eq!(retried["data"]["replayed"], false);
    assert_eq!(retried["data"]["flows"], json!(["default", "secondary"]));
    assert!(env.workbook_dir("two-flows", "1.0.0").exists());
    assert_eq!(env.ok(&args)["data"]["replayed"], true);
    for table in ["workbooks", "requests", "audit"] {
        assert_eq!(
            connection
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                    row.get::<_, i64>(0)
                })
                .unwrap(),
            2,
            "{table} 应只记录两个成功请求"
        );
    }
}

// Task: C002-T39
#[test]
fn load_dir_rejects_duplicate_flow_id_and_accepts_distinct_ids() {
    let env = Env::new();
    let source = env.dir.path().join("private-copy");
    write_workbook(&source, "default");
    let home = sheltie_runtime::home::Home::resolve(Some(&env.home())).unwrap();
    let repo = sheltie_runtime::WorkbookRepo::new(home);
    let canonical_source = source.canonicalize().unwrap();
    let directory = sheltie_core::path::AbsPath::new(canonical_source.to_str().unwrap()).unwrap();
    let error = repo.load_dir(&directory).unwrap_err();
    assert!(matches!(
        error,
        sheltie_runtime::Error::Core(sheltie_core::Error::FlowInvalid { rule: "1", path, .. })
            if path == "flows/second.toml.id"
    ));
    write_flow(&source.join("flows/second.toml"), "secondary");
    let loaded = repo.load_dir(&directory).unwrap();
    assert_eq!(loaded.flows.len(), 2);
    assert!(loaded.flow("default").is_some());
    assert!(loaded.flow("secondary").is_some());
    assert!(!env.dir.path().join("store.db").exists());
}

// Task: C002-T39
#[test]
fn duplicate_flow_ids_in_a_committed_snapshot_stay_rejected_and_unmodified() {
    let env = Env::new();
    let source = tempfile::tempdir().unwrap();
    write_workbook(source.path(), "secondary");
    let args = [
        "--request-id",
        "historical-duplicate-flow",
        "workbook",
        "add",
        source.path().to_str().unwrap(),
    ];
    env.ok(&args);
    let connection = Connection::open(env.dir.path().join("store.db")).unwrap();
    let (reply_json, effects_json): (String, String) = connection
        .query_row(
            "SELECT reply_json, effects_json FROM requests WHERE request_id = ?1",
            ["historical-duplicate-flow"],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    let mut snapshot: serde_json::Value = serde_json::from_str(&reply_json).unwrap();
    snapshot["data"]["flows"] = json!(["default", "default"]);
    let corrupted = serde_json::to_string(&snapshot).unwrap();
    connection
        .execute(
            "UPDATE requests SET reply_json = ?1, published = 0 WHERE request_id = ?2",
            [&corrupted, "historical-duplicate-flow"],
        )
        .unwrap();

    let (replayed, code) = env.fail(&args);
    assert_eq!(code, 1);
    assert_eq!(replayed["error"]["code"], "EFFECT_PENDING");
    assert_eq!(replayed["error"]["detail"]["cause"], "STORE_CORRUPT");
    assert_eq!(replayed["committed"], true);
    let (blocked, code) = env.fail(&[
        "--request-id",
        "after-historical-duplicate",
        "workbook",
        "add",
        example_dir("two-step").to_str().unwrap(),
    ]);
    assert_eq!(code, 1);
    assert_eq!(blocked["error"]["code"], "EFFECT_PENDING");
    assert_eq!(blocked["committed"], false);
    assert_eq!(
        blocked["error"]["detail"]["pending_request_id"],
        "historical-duplicate-flow"
    );
    let after: (String, String, i64) = connection
        .query_row(
            "SELECT reply_json, effects_json, published FROM requests WHERE request_id = ?1",
            ["historical-duplicate-flow"],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!(after, (corrupted, effects_json, 0));
    for table in ["workbooks", "requests", "audit"] {
        assert_eq!(
            connection
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                    row.get::<_, i64>(0)
                })
                .unwrap(),
            1,
            "{table} 不得迁移、删除历史记录或登记被阻断请求"
        );
    }
    assert!(env.workbook_dir("two-flows", "1.0.0").exists());
    assert!(!env.workbook_dir("two-step", "1.0.0").exists());
}
