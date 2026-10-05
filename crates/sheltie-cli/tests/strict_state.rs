//! Reject nested unknown fields in complete persisted payloads before business I/O.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::collections::BTreeMap;
use std::path::PathBuf;

use common::store::{StoreRows, store_rows};
use common::*;
use rusqlite::Connection;
use serde_json::{Value, json};

#[derive(Debug, PartialEq)]
struct Persisted {
    rows: StoreRows,
    files: BTreeMap<PathBuf, Option<Vec<u8>>>,
}

fn connection(env: &Env) -> Connection {
    Connection::open(env.dir.path().join("store.db")).unwrap()
}

fn persisted(env: &Env) -> Persisted {
    let rows = store_rows(env);
    let files = tree_objects(env.dir.path(), &["works", "workbooks", "pending"])
        .into_iter()
        .map(|(path, object)| (path, object.bytes))
        .collect();
    Persisted { rows, files }
}

fn read_json(connection: &Connection, query: &str, id: &str) -> Value {
    let text: String = connection.query_row(query, [id], |row| row.get(0)).unwrap();
    serde_json::from_str(&text).unwrap()
}

fn assert_corrupt_state_is_unchanged(env: &Env, work: &str) {
    let before = persisted(env);
    for args in [
        vec!["work", "status", work],
        vec!["work", "stats", work],
        vec!["work", "list"],
        vec!["--request-id", "after-corruption", "work", "cancel", work],
    ] {
        let (error, exit) = env.fail(&args);
        assert_eq!(exit, 1, "{args:?}: {error}");
        assert_eq!(error["error"]["code"], "STORE_CORRUPT", "{error}");
        assert!(error.get("original").is_none(), "{error}");
        assert_eq!(persisted(env), before, "{args:?}");
    }
}

fn assert_corrupt_replay_is_unchanged(env: &Env, args: &[&str], request_id: &str) {
    let before = persisted(env);
    let (error, exit) = env.fail(args);
    assert_eq!(exit, 1);
    assert_eq!(error["error"]["code"], "EFFECT_PENDING", "{error}");
    assert_eq!(error["error"]["detail"]["cause"], "STORE_CORRUPT");
    assert_eq!(error["committed"], true);
    assert_eq!(error["request_id"], request_id);
    assert!(error.get("original").is_none(), "{error}");
    assert!(
        error["error"]["detail"].get("pending_original").is_none(),
        "{error}"
    );
    assert_eq!(persisted(env), before);
}

// Task: C002-T39
#[test]
fn unknown_work_state_and_nested_attempt_fields_are_rejected_without_rewriting_data() {
    for pointer in ["", "/attempts/0/id"] {
        let env = Env::new();
        env.add_example("two-step");
        let work = env.start("two-step", &[("topic", "strict state")]);
        env.begin(&work, "outline");
        let before_control = persisted(&env);
        assert_eq!(
            env.status(&work)["data"]["status"],
            json!({"kind": "active"})
        );
        assert_eq!(persisted(&env), before_control);
        let connection = connection(&env);
        let mut state = read_json(
            &connection,
            "SELECT state_json FROM works WHERE work_id=?1",
            &work,
        );
        state.pointer_mut(pointer).unwrap()["unexpected"] = json!(true);
        connection
            .execute(
                "UPDATE works SET state_json=?1 WHERE work_id=?2",
                rusqlite::params![state.to_string(), &work],
            )
            .unwrap();
        assert_corrupt_state_is_unchanged(&env, &work);
    }
}

// Task: C002-T39
#[test]
fn every_work_status_shape_rejects_unknown_fields_without_rewriting_data() {
    for kind in ["active", "blocked", "succeeded", "cancelled"] {
        let env = Env::new();
        let workbook = if kind == "blocked" {
            "gated-release"
        } else {
            "two-step"
        };
        env.add_example(workbook);
        let input = if kind == "blocked" {
            "version"
        } else {
            "topic"
        };
        let work = env.start(workbook, &[(input, "strict status")]);
        let node = if kind == "blocked" {
            "notes"
        } else {
            "outline"
        };
        let begun = env.begin(&work, node);
        match kind {
            "blocked" => {
                env.submit_all(&work, &begun, "gate needed");
            }
            "succeeded" => {
                env.submit_all(&work, &begun, "outline ready");
                let begun = env.begin(&work, "summary");
                env.submit_all(&work, &begun, "summary ready");
            }
            "cancelled" => {
                env.ok(&["work", "cancel", &work]);
            }
            "active" => {}
            _ => unreachable!(),
        }
        let before_control = persisted(&env);
        assert_eq!(env.status(&work)["data"]["status"]["kind"], kind);
        assert_eq!(persisted(&env), before_control);
        let connection = connection(&env);
        let mut state = read_json(
            &connection,
            "SELECT state_json FROM works WHERE work_id=?1",
            &work,
        );
        if kind == "blocked" {
            assert_eq!(
                state["status"],
                json!({"kind": "blocked", "reason": "gate"})
            );
        }
        state["status"]["unexpected"] = json!(true);
        connection
            .execute(
                "UPDATE works SET state_json=?1 WHERE work_id=?2",
                rusqlite::params![state.to_string(), &work],
            )
            .unwrap();
        assert_corrupt_state_is_unchanged(&env, &work);
    }
}

// Task: C002-T39
#[test]
fn historical_reply_and_next_attempt_fields_reject_unknown_data_without_successful_original() {
    for pointer in ["/reply/attempt", "/next/0/attempt", "/next/1/attempt"] {
        let env = Env::new();
        env.add_example("two-step");
        let work = env.start("two-step", &[("topic", "strict snapshot")]);
        let request_id = "strict-begin";
        let args = [
            "--request-id",
            request_id,
            "attempt",
            "begin",
            &work,
            "--node",
            "outline",
        ];
        let original = env.ok(&args);
        env.ok(&["work", "cancel", &work]);
        let before_control = persisted(&env);
        let mut expected = original;
        expected["data"]["replayed"] = json!(true);
        assert_eq!(env.ok(&args), expected);
        assert_eq!(persisted(&env), before_control);
        let connection = connection(&env);
        let mut snapshot = read_json(
            &connection,
            "SELECT reply_json FROM requests WHERE request_id=?1",
            request_id,
        );
        assert_eq!(snapshot["next"][0]["op"], "attempt submit");
        assert_eq!(snapshot["next"][1]["op"], "attempt fail");
        snapshot.pointer_mut(pointer).unwrap()["unexpected"] = json!(true);
        connection
            .execute(
                "UPDATE requests SET reply_json=?1 WHERE request_id=?2",
                rusqlite::params![snapshot.to_string(), request_id],
            )
            .unwrap();
        assert_corrupt_replay_is_unchanged(&env, &args, request_id);
    }
}

// Task: C002-T39
#[test]
fn audit_attempt_fields_reject_unknown_data_without_successful_original() {
    for operation in ["submit", "fail"] {
        let env = Env::new();
        env.add_example("two-step");
        let work = env.start("two-step", &[("topic", "strict audit")]);
        let begun = env.begin(&work, "outline");
        for path in begun["data"]["outputs"].as_object().unwrap().values() {
            std::fs::write(path.as_str().unwrap(), b"recorded output\n").unwrap();
        }
        let request_id = "strict-attempt";
        let parameter = if operation == "submit" {
            "--summary"
        } else {
            "--reason"
        };
        let args = [
            "--request-id",
            request_id,
            "attempt",
            operation,
            &work,
            "--attempt",
            "outline#1.0",
            parameter,
            "recorded result",
        ];
        let original = env.ok(&args);
        env.ok(&["work", "cancel", &work]);
        let before_control = persisted(&env);
        let mut expected = original;
        expected["data"]["replayed"] = json!(true);
        assert_eq!(env.ok(&args), expected);
        assert_eq!(persisted(&env), before_control);
        let connection = connection(&env);
        let mut command = read_json(
            &connection,
            "SELECT command_json FROM audit WHERE request_id=?1",
            request_id,
        );
        assert_eq!(command["command"], format!("{operation}_attempt"));
        command["attempt"]["unexpected"] = json!(true);
        connection
            .execute(
                "UPDATE audit SET command_json=?1 WHERE request_id=?2",
                rusqlite::params![command.to_string(), request_id],
            )
            .unwrap();
        assert_corrupt_replay_is_unchanged(&env, &args, request_id);
    }
}

// Task: C002-T46
#[test]
fn replay_decodes_command_and_data_before_reading_the_frozen_workbook() {
    for (field, blocks_new_request) in [
        ("command", false),
        ("data", false),
        ("command", true),
        ("data", true),
    ] {
        let env = Env::new();
        env.add_example("two-step");
        let work = env.start("two-step", &[("topic", "decode before frozen read")]);
        let request_id = "decode-order-begin";
        let args = [
            "--request-id",
            request_id,
            "attempt",
            "begin",
            &work,
            "--node",
            "outline",
        ];
        let mut expected = env.ok(&args);
        expected["data"]["replayed"] = json!(true);
        let legal_before = persisted(&env);
        assert_eq!(env.ok(&args), expected);
        assert_eq!(persisted(&env), legal_before);
        let connection = connection(&env);
        let (table, column) = if field == "command" {
            ("audit", "command_json")
        } else {
            ("requests", "reply_json")
        };
        let mut payload = read_json(
            &connection,
            &format!("SELECT {column} FROM {table} WHERE request_id=?1"),
            request_id,
        );
        let target = if field == "command" {
            &mut payload
        } else {
            &mut payload["data"]
        };
        target["unexpected_contract_field"] = json!(true);
        connection
            .execute(
                &format!("UPDATE {table} SET {column}=?1 WHERE request_id=?2"),
                rusqlite::params![payload.to_string(), request_id],
            )
            .unwrap();
        if blocks_new_request {
            connection
                .execute(
                    "UPDATE requests SET published=0 WHERE request_id=?1",
                    [request_id],
                )
                .unwrap();
        }
        let frozen = env.dir.path().join("works").join(&work).join("workbook");
        let retained = frozen.with_file_name("retained-workbook");
        std::fs::rename(&frozen, &retained).unwrap();
        let before = persisted(&env);
        let new_args = [
            "--request-id",
            "after-invalid-history",
            "work",
            "cancel",
            &work,
        ];
        let (error, exit) = env.fail(if blocks_new_request { &new_args } else { &args });
        assert_eq!(exit, 1);
        assert_eq!(error["error"]["code"], "EFFECT_PENDING", "{field}: {error}");
        assert_eq!(
            error["error"]["detail"]["cause"], "STORE_CORRUPT",
            "{field}: {error}"
        );
        assert!(
            error.to_string().contains("unexpected_contract_field"),
            "{field}: {error}"
        );
        assert_eq!(error["committed"], !blocks_new_request);
        if blocks_new_request {
            assert_eq!(error["request_id"], "after-invalid-history");
            assert_eq!(error["error"]["detail"]["pending_request_id"], request_id);
            assert!(
                error["error"]["detail"].get("pending_original").is_none(),
                "{error}"
            );
        } else {
            assert_eq!(error["request_id"], request_id);
        }
        assert!(error.get("original").is_none(), "{error}");
        assert!(error.get("revision").is_none(), "{error}");
        assert_eq!(persisted(&env), before, "{field}");
        assert!(!frozen.exists());
        assert!(retained.is_dir());
    }
}
