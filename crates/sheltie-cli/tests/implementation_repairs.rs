//! C002-T31：实现审查发现的真实初始化与门槛事实回归。
#![cfg(feature = "failpoint")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::*;
use rusqlite::Connection;
use serde_json::Value;
use std::os::unix::process::ExitStatusExt;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

// Task: C002-T31
#[test]
fn killed_first_store_initializer_allows_the_same_add_request_to_retry() {
    let env = Env::new();
    let sync = tempfile::tempdir().unwrap();
    let home = env.dir.path().canonicalize().unwrap();
    let source = example_dir("two-step");
    let args = [
        "--request-id",
        "init-kill",
        "workbook",
        "add",
        source.to_str().unwrap(),
    ];
    let mut child = Command::new(env!("CARGO_BIN_EXE_sheltie"))
        .args(["--home", &env.home(), "--json"])
        .args(args)
        .env(
            "SHELTIE_TEST_RENDEZVOUS_NAME",
            "write_session_after_store_create",
        )
        .env("SHELTIE_TEST_RENDEZVOUS_ID", &home)
        .env("SHELTIE_TEST_RENDEZVOUS_DIR", sync.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !sync.path().join("reached").exists() {
        if child.try_wait().unwrap().is_some() || Instant::now() >= deadline {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            panic!("初始化未到指定窗口：{output:?}");
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    child.kill().unwrap();
    assert_eq!(child.wait_with_output().unwrap().status.signal(), Some(9));
    let reply = env.ok(&args);
    assert_eq!(reply["request_id"], "init-kill");
    assert_eq!(reply["data"]["replayed"], false);
    let connection = Connection::open(home.join("store.db")).unwrap();
    assert_eq!(
        connection
            .pragma_query_value(None, "user_version", |row| row.get::<_, i64>(0))
            .unwrap(),
        2
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT COUNT(*) FROM requests WHERE request_id='init-kill'",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        1
    );
    assert_eq!(env.ok(&args)["data"]["replayed"], true);
}

// Task: C002-T31
#[test]
fn missing_current_or_historical_gate_approval_is_rejected_without_a_new_request() {
    for already_left in [false, true] {
        let env = Env::new();
        env.add_example("gated-release");
        let work = env.start("gated-release", &[("version", "test")]);
        let begun = env.begin(&work, "notes");
        env.submit_all(&work, &begun, "notes ready");
        env.ok(&["gate", "approve", &work, "--node", "notes"]);
        if already_left {
            env.begin(&work, "archive");
        }
        let connection = Connection::open(env.dir.path().join("store.db")).unwrap();
        let before: String = connection
            .query_row(
                "SELECT state_json FROM works WHERE work_id=?1",
                [&work],
                |row| row.get(0),
            )
            .unwrap();
        let mut state: Value = serde_json::from_str(&before).unwrap();
        assert_eq!(state["approvals"].as_array().unwrap().len(), 1);
        state["approvals"] = serde_json::json!([]);
        let corrupted = serde_json::to_string(&state).unwrap();
        connection
            .execute(
                "UPDATE works SET state_json=?1 WHERE work_id=?2",
                [&corrupted, &work],
            )
            .unwrap();
        for command in [
            vec!["work", "status", &work],
            vec!["work", "stats", &work],
            vec!["work", "list"],
        ] {
            let (reply, code) = env.fail(&command);
            assert_eq!(code, 1);
            assert_eq!(reply["error"]["code"], "STORE_CORRUPT");
        }
        let (reply, code) = env.fail(&[
            "--request-id",
            "after-corruption",
            "attempt",
            "begin",
            &work,
            "--node",
            "archive",
        ]);
        assert_eq!(code, 1);
        assert_eq!(reply["error"]["code"], "STORE_CORRUPT");
        assert_eq!(
            connection
                .query_row(
                    "SELECT state_json FROM works WHERE work_id=?1",
                    [&work],
                    |row| row.get::<_, String>(0)
                )
                .unwrap(),
            corrupted
        );
        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM requests WHERE request_id='after-corruption'",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            0
        );
    }
}

// Task: C002-T31
#[test]
fn approval_of_another_gate_at_the_same_occurrence_does_not_allow_progress() {
    let env = Env::new();
    let source = tempfile::tempdir().unwrap();
    std::fs::create_dir(source.path().join("flows")).unwrap();
    std::fs::write(
        source.path().join("workbook.toml"),
        r#"schema = "workbook/v1"
id = "two-gates"
version = "1.0.0"
name = "Two gates"
description = "Independent occurrence approval"
flows = ["flows/default.toml"]
"#,
    )
    .unwrap();
    std::fs::write(
        source.path().join("flows/default.toml"),
        r#"schema = "flow/v1"
id = "default"
entry = "first"
[[nodes]]
id = "first"
title = "First"
executor = "agent"
instruction = { text = "first" }
gate = true
[[nodes]]
id = "second"
title = "Second"
executor = "agent"
instruction = { text = "second" }
gate = true
[[nodes]]
id = "done"
title = "Done"
executor = "agent"
instruction = { text = "done" }
[[edges]]
from = "first"
to = "second"
kind = "main"
[[edges]]
from = "second"
to = "done"
kind = "main"
"#,
    )
    .unwrap();
    env.ok(&["workbook", "add", source.path().to_str().unwrap()]);
    let work = env.start("two-gates", &[]);
    for node in ["first", "second"] {
        let begun = env.begin(&work, node);
        env.submit_all(&work, &begun, "ready");
        env.ok(&["gate", "approve", &work, "--node", node]);
    }
    env.status(&work);
    let connection = Connection::open(env.dir.path().join("store.db")).unwrap();
    let original: String = connection
        .query_row(
            "SELECT state_json FROM works WHERE work_id=?1",
            [&work],
            |row| row.get(0),
        )
        .unwrap();
    let mut state: Value = serde_json::from_str(&original).unwrap();
    let approvals = state["approvals"].as_array_mut().unwrap();
    assert_eq!(approvals.len(), 2);
    assert_eq!(approvals[0]["node"], "first");
    assert_eq!(approvals[0]["occurrence"], 1);
    assert_eq!(approvals.pop().unwrap()["node"], "second");
    let corrupted = serde_json::to_string(&state).unwrap();
    connection
        .execute(
            "UPDATE works SET state_json=?1 WHERE work_id=?2",
            [&corrupted, &work],
        )
        .unwrap();
    let (reply, _) = env.fail(&["work", "status", &work]);
    assert_eq!(reply["error"]["code"], "STORE_CORRUPT");
    let (reply, _) = env.fail(&[
        "--request-id",
        "cross-gate-approval",
        "attempt",
        "begin",
        &work,
        "--node",
        "done",
    ]);
    assert_eq!(reply["error"]["code"], "STORE_CORRUPT");
    assert_eq!(
        connection
            .query_row(
                "SELECT state_json FROM works WHERE work_id=?1",
                [&work],
                |row| row.get::<_, String>(0)
            )
            .unwrap(),
        corrupted
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT COUNT(*) FROM requests WHERE request_id='cross-gate-approval'",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        0
    );
}
