//! Valid snapshot shape cannot replace binding to committed business facts.
#![allow(clippy::unwrap_used, clippy::expect_used)]
mod common;
use common::store::store_rows;
use common::*;
use rusqlite::Connection;
use serde_json::{Value, json};

fn business_files(
    env: &Env,
) -> std::collections::BTreeMap<std::path::PathBuf, (u32, Option<Vec<u8>>)> {
    tree_objects(env.dir.path(), &["works", "workbooks", "pending"])
        .into_iter()
        .map(|(path, object)| (path, (object.mode, object.bytes)))
        .collect()
}

// Task: C002-T48
#[test]
fn replay_rejects_coherent_but_forged_start_submit_and_fail_snapshots() {
    for field in [
        "start_requires",
        "submit_attempt",
        "submit_output",
        "fail_attempt",
    ] {
        let env = Env::new();
        let source = tempfile::tempdir().unwrap();
        copy_dir(&example_dir("two-step"), source.path());
        let flow_path = source.path().join("flows/default.toml");
        let flow = std::fs::read_to_string(&flow_path)
            .unwrap()
            .replace("id = \"outline\"\n", "id = \"outline\"\nmax_retries = 3\n");
        std::fs::write(&flow_path, format!("{flow}\n[[nodes]]\nid='finish'\ntitle='Finish'\nexecutor='agent'\ninstruction={{text='Finish.'}}\n[[edges]]\nfrom='summary'\nto='finish'\nkind='main'\n")).unwrap();
        env.ok(&["workbook", "add", source.path().to_str().unwrap()]);
        let start_args = [
            "--request-id",
            "qualified-start",
            "work",
            "start",
            "--workbook",
            "two-step",
            "--flow",
            "default",
            "--input",
            "topic=qualification",
        ];
        let started = env.ok(&start_args);
        let work = started["data"]["work_id"].as_str().unwrap().to_owned();
        let mut replay = start_args
            .iter()
            .copied()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let mut rid = "qualified-start";
        if field != "start_requires" {
            let begun = env.begin(&work, "outline");
            let op = if field == "fail_attempt" {
                "fail"
            } else {
                "submit"
            };
            let flag = if op == "fail" {
                "--reason"
            } else {
                "--summary"
            };
            if op == "submit" {
                for path in begun["data"]["outputs"].as_object().unwrap().values() {
                    std::fs::write(path.as_str().unwrap(), b"actual outline\n").unwrap();
                }
            }
            rid = "qualified-operation";
            replay = [
                "--request-id",
                rid,
                "attempt",
                op,
                &work,
                "--attempt",
                "outline#1.0",
                flag,
                "canonical",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect();
            env.ok(&replay.iter().map(String::as_str).collect::<Vec<_>>());
            if field == "fail_attempt" {
                env.begin(&work, "outline");
                env.ok(&[
                    "attempt",
                    "fail",
                    &work,
                    "--attempt",
                    "outline#1.1",
                    "--reason",
                    "second real failure",
                ]);
            } else if field == "submit_attempt" {
                let other = env.begin(&work, "summary");
                env.submit_all(&work, &other, "second real success");
            }
        }
        let args = replay.iter().map(String::as_str).collect::<Vec<_>>();
        assert_eq!(env.ok(&args)["data"]["replayed"], true);
        let connection = Connection::open(env.dir.path().join("store.db")).unwrap();
        let raw: String = connection
            .query_row(
                "SELECT reply_json FROM requests WHERE request_id=?1",
                [rid],
                |row| row.get(0),
            )
            .unwrap();
        let mut snapshot: Value = serde_json::from_str(&raw).unwrap();
        match field {
            "start_requires" => {
                let requires = json!([{"kind":"skill", "name":"undeclared"}]);
                snapshot["reply"]["requires"] = requires.clone();
                snapshot["data"]["requires"] = requires;
            }
            "submit_attempt" => {
                snapshot["reply"]["attempt"]["node"] = json!("summary");
                snapshot["data"]["attempt"] = json!("summary#1.0");
            }
            "fail_attempt" => {
                snapshot["reply"]["attempt"]["number"] = json!(1);
                snapshot["data"]["attempt"] = json!("outline#1.1");
            }
            "submit_output" => {
                for side in ["reply", "data"] {
                    for output in snapshot[side]["outputs"]
                        .as_object_mut()
                        .unwrap()
                        .values_mut()
                    {
                        output["sha256"] = json!("a".repeat(64));
                    }
                }
            }
            _ => unreachable!(),
        }
        connection
            .execute(
                "UPDATE requests SET reply_json=?1 WHERE request_id=?2",
                rusqlite::params![snapshot.to_string(), rid],
            )
            .unwrap();
        let before = store_rows(&env);
        let files_before = business_files(&env);
        let (error, exit) = env.fail(&args);
        assert_eq!(exit, 1);
        assert_eq!(error["error"]["code"], "EFFECT_PENDING", "{field}: {error}");
        assert_eq!(
            error["error"]["detail"]["cause"], "STORE_CORRUPT",
            "{field}: {error}"
        );
        assert_eq!(error["committed"], true);
        assert_eq!(error["request_id"], rid);
        assert!(error.get("original").is_none(), "{field}: {error}");
        assert!(error.get("revision").is_none(), "{field}: {error}");
        assert_eq!(store_rows(&env), before);
        assert_eq!(business_files(&env), files_before);
    }
}

// Task: C002-T48
#[test]
fn start_snapshot_identity_mismatch_is_rejected_before_frozen_workbook_read() {
    for field in ["work_id", "work_dir"] {
        let env = Env::new();
        env.add_example("two-step");
        let args = [
            "--request-id",
            "start-identity",
            "work",
            "start",
            "--workbook",
            "two-step",
            "--flow",
            "default",
            "--input",
            "topic=identity",
        ];
        let started = env.ok(&args);
        let work = started["data"]["work_id"].as_str().unwrap();
        assert_eq!(env.ok(&args)["data"]["replayed"], true);
        let connection = Connection::open(env.dir.path().join("store.db")).unwrap();
        let raw: String = connection
            .query_row(
                "SELECT reply_json FROM requests WHERE request_id='start-identity'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let mut snapshot: Value = serde_json::from_str(&raw).unwrap();
        snapshot["data"][field] = match field {
            "work_id" => json!(format!("{work}-other")),
            "work_dir" => json!(env.home()),
            _ => unreachable!(),
        };
        connection
            .execute(
                "UPDATE requests SET reply_json=?1 WHERE request_id='start-identity'",
                [snapshot.to_string()],
            )
            .unwrap();
        let frozen = env.work_dir(work).join("workbook");
        let retained = frozen.with_file_name("retained-workbook");
        std::fs::rename(&frozen, &retained).unwrap();
        let before = store_rows(&env);
        let files_before = business_files(&env);
        let (error, exit) = env.fail(&args);
        assert_eq!(exit, 1);
        assert_eq!(error["error"]["code"], "EFFECT_PENDING");
        assert_eq!(error["error"]["detail"]["cause"], "STORE_CORRUPT");
        assert!(
            error.to_string().contains("data differs from Reply"),
            "{field}: {error}"
        );
        assert!(error.get("original").is_none());
        assert_eq!(store_rows(&env), before);
        assert_eq!(business_files(&env), files_before);
        assert!(!frozen.exists());
        assert!(retained.is_dir());
    }
}

// Task: C002-T49
#[test]
fn completed_workbook_add_replay_rejects_invalid_flow_requirement_and_identity_shapes() {
    for field in [
        "empty_flows",
        "duplicate_flows",
        "invalid_flow",
        "empty_kind",
        "empty_name",
        "invalid_id",
        "invalid_version",
    ] {
        let env = Env::new();
        let source = example_dir("two-step");
        let args = [
            "--request-id",
            "workbook-shape",
            "workbook",
            "add",
            source.to_str().unwrap(),
        ];
        env.ok(&args);
        assert_eq!(env.ok(&args)["data"]["replayed"], true);
        let connection = Connection::open(env.dir.path().join("store.db")).unwrap();
        let (raw, effects): (String, String) = connection
            .query_row(
                "SELECT reply_json,effects_json FROM requests WHERE request_id='workbook-shape'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        let mut snapshot: Value = serde_json::from_str(&raw).unwrap();
        let mut effects: Value = serde_json::from_str(&effects).unwrap();
        match field {
            "empty_flows" => snapshot["data"]["flows"] = json!([]),
            "duplicate_flows" => snapshot["data"]["flows"] = json!(["default", "default"]),
            "invalid_flow" => snapshot["data"]["flows"] = json!(["INVALID"]),
            "empty_kind" => snapshot["data"]["requires"] = json!([":named"]),
            "empty_name" => snapshot["data"]["requires"] = json!(["skill:"]),
            "invalid_id" | "invalid_version" => {
                let (id, version) = if field == "invalid_id" {
                    ("INVALID", "1.0.0")
                } else {
                    ("two-step", ".")
                };
                snapshot["data"]["id"] = json!(id);
                snapshot["data"]["version"] = json!(version);
                effects[0]["final"] = json!(format!("workbooks/{id}/{version}"));
                effects[0]["owner"] = json!(format!("workbook:{id}@{version}"));
            }
            _ => unreachable!(),
        }
        connection
            .execute(
                "UPDATE requests SET reply_json=?1,effects_json=?2 WHERE request_id='workbook-shape'",
                rusqlite::params![snapshot.to_string(), effects.to_string()],
            )
            .unwrap();
        let before = store_rows(&env);
        let files = business_files(&env);
        let (error, exit) = env.fail(&args);
        assert_eq!(exit, 1);
        assert_eq!(error["error"]["code"], "EFFECT_PENDING", "{field}: {error}");
        assert_eq!(
            error["error"]["detail"]["cause"], "STORE_CORRUPT",
            "{field}: {error}"
        );
        assert!(error.get("original").is_none(), "{field}: {error}");
        assert_eq!(error["committed"], true);
        assert_eq!(error["request_id"], "workbook-shape");
        assert_eq!(store_rows(&env), before);
        assert_eq!(business_files(&env), files);
    }
}

// Task: C002-T49
#[test]
fn current_workbook_lifecycle_cannot_borrow_an_older_identical_publisher() {
    use std::os::unix::fs::MetadataExt;
    for field in [
        "snapshot_id",
        "snapshot_version",
        "audit_target",
        "non_workbook_audit",
        "request_work",
        "row_time",
        "row_digest",
        "row_dir",
        "effect_digest",
        "effect_final",
        "effect_owner",
    ] {
        let env = Env::new();
        let source = example_dir("two-step");
        let old_args = [
            "--request-id",
            "life-old",
            "workbook",
            "add",
            source.to_str().unwrap(),
        ];
        let old = env.ok(&old_args);
        let remove = [
            "--request-id",
            "life-remove",
            "workbook",
            "remove",
            "two-step@1.0.1",
        ];
        env.ok(&remove);
        let new_args = [
            "--request-id",
            "life-new",
            "workbook",
            "add",
            source.to_str().unwrap(),
        ];
        let new = env.ok(&new_args);
        assert_eq!(old["data"]["digest"], new["data"]["digest"]);
        let connection = Connection::open(env.dir.path().join("store.db")).unwrap();
        let at = "2026-10-03T00:00:00Z";
        connection
            .execute("UPDATE requests SET at=?1", [at])
            .unwrap();
        connection.execute("UPDATE audit SET at=?1", [at]).unwrap();
        connection
            .execute("UPDATE workbooks SET added_at=?1", [at])
            .unwrap();
        let installed = env.workbook_dir("two-step", "1.0.1");
        let inode = std::fs::metadata(&installed).unwrap().ino();
        let healthy_files = business_files(&env);
        env.ok(&["workbook", "show", "two-step@1.0.1"]);
        env.ok(&old_args);
        env.ok(&remove);
        assert_eq!(business_files(&env), healthy_files);
        assert_eq!(std::fs::metadata(&installed).unwrap().ino(), inode);
        match field {
            "snapshot_id" | "snapshot_version" => {
                let raw: String = connection
                    .query_row(
                        "SELECT reply_json FROM requests WHERE request_id='life-new'",
                        [],
                        |row| row.get(0),
                    )
                    .unwrap();
                let mut snapshot: Value = serde_json::from_str(&raw).unwrap();
                if field == "snapshot_id" {
                    snapshot["data"]["id"] = json!("other");
                } else {
                    snapshot["data"]["version"] = json!("2.0.0");
                }
                connection
                    .execute(
                        "UPDATE requests SET reply_json=?1 WHERE request_id='life-new'",
                        [snapshot.to_string()],
                    )
                    .unwrap();
            }
            "audit_target" => {
                connection
                    .execute(
                        "UPDATE audit SET command_json=?1 WHERE request_id='life-new'",
                        [json!({"intent":"remove_workbook", "target":"other@1.0.0"}).to_string()],
                    )
                    .unwrap();
            }
            "non_workbook_audit" => {
                env.start("two-step", &[("topic", "real work audit")]);
                let command: String = connection
                    .query_row(
                        "SELECT command_json FROM audit WHERE work_id <> '' ORDER BY seq LIMIT 1",
                        [],
                        |row| row.get(0),
                    )
                    .unwrap();
                connection
                    .execute(
                        "UPDATE audit SET command_json=?1 WHERE request_id='life-new'",
                        [command],
                    )
                    .unwrap();
            }
            "request_work" => {
                connection
                    .execute(
                        "UPDATE requests SET work_id='2026-10-03-999-other' WHERE request_id='life-new'",
                        [],
                    )
                    .unwrap();
            }
            "row_time" => {
                connection
                    .execute("UPDATE workbooks SET added_at='2026-10-03T00:00:01Z'", [])
                    .unwrap();
            }
            "row_digest" => {
                connection
                    .execute("UPDATE workbooks SET digest=?1", ["a".repeat(64)])
                    .unwrap();
            }
            "row_dir" => {
                connection
                    .execute("UPDATE workbooks SET dir='workbooks/two-step/other'", [])
                    .unwrap();
            }
            "effect_digest" | "effect_final" | "effect_owner" => {
                let raw: String = connection
                    .query_row(
                        "SELECT effects_json FROM requests WHERE request_id='life-new'",
                        [],
                        |row| row.get(0),
                    )
                    .unwrap();
                let mut effects: Value = serde_json::from_str(&raw).unwrap();
                match field {
                    "effect_digest" => effects[0]["digest"] = json!("a".repeat(64)),
                    "effect_final" => effects[0]["final"] = json!("workbooks/other/1.0.0"),
                    "effect_owner" => effects[0]["owner"] = json!("workbook:other@1.0.0"),
                    _ => unreachable!(),
                }
                connection
                    .execute(
                        "UPDATE requests SET effects_json=?1 WHERE request_id='life-new'",
                        [effects.to_string()],
                    )
                    .unwrap();
            }
            _ => unreachable!(),
        }
        let before = store_rows(&env);
        let files = business_files(&env);
        let (error, exit) = env.fail(&["workbook", "show", "two-step@1.0.1"]);
        assert_eq!(exit, 1);
        assert_eq!(error["error"]["code"], "STORE_CORRUPT", "{field}: {error}");
        if field == "snapshot_id" || field == "snapshot_version" {
            assert!(
                error["error"]["message"].as_str().unwrap().contains(
                    "Workbook snapshot business identity differs from registered effects"
                ),
                "{field}: {error}"
            );
        }

        if field == "audit_target" {
            assert!(
                error["error"]["message"]
                    .as_str()
                    .unwrap()
                    .contains("Workbook remove request life-new intent_hash differs from audit"),
                "{error}"
            );
        } else if field == "non_workbook_audit" {
            assert!(
                error["error"]["message"]
                    .as_str()
                    .unwrap()
                    .contains("Cannot decode Workbook audit life-new command"),
                "{error}"
            );
        }
        assert_eq!(store_rows(&env), before);
        assert_eq!(business_files(&env), files);
        assert_eq!(std::fs::metadata(&installed).unwrap().ino(), inode);
    }
}

fn wait_for_a_later_utc_second() {
    let second = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    while std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        <= second
    {
        assert!(
            std::time::Instant::now() < deadline,
            "UTC clock did not reach next second"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

fn completed_approval_fixture() -> (Env, String, Vec<String>, Value) {
    let env = Env::new();
    env.add_example("gated-release");
    let work = env.start("gated-release", &[("version", "record-consistency")]);
    let begun = env.begin(&work, "notes");
    env.submit_all(&work, &begun, "actual notes for record consistency");
    let args = [
        "--request-id",
        "recorded-approval",
        "gate",
        "approve",
        &work,
        "--node",
        "notes",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let original = env.ok(&args.iter().map(String::as_str).collect::<Vec<_>>());
    wait_for_a_later_utc_second();
    let archive = env.begin(&work, "archive");
    env.submit_all(&work, &archive, "later legitimate completion");
    assert_eq!(
        env.ok(&["work", "status", &work])["data"]["status"]["kind"],
        "succeeded"
    );
    (env, work, args, original)
}

fn audit_fixture_objects(env: &Env) -> std::collections::BTreeMap<std::path::PathBuf, TreeObject> {
    tree_objects(env.dir.path(), &["works", "workbooks", "pending"])
}

// Task: C002-T58
#[test]
fn historical_replays_keep_recorded_responses_after_later_attempt_and_work_changes() {
    let (env, _work, args, mut expected) = completed_approval_fixture();
    assert_eq!(expected["data"]["work_status"]["kind"], "active");
    expected["data"]["replayed"] = json!(true);
    let before = store_rows(&env);
    let objects = audit_fixture_objects(&env);
    assert_eq!(
        env.ok(&args.iter().map(String::as_str).collect::<Vec<_>>()),
        expected
    );
    assert_eq!(store_rows(&env), before);
    assert_eq!(audit_fixture_objects(&env), objects);

    let env = Env::new();
    env.add_example("two-step");
    let mut recorded = Vec::<(Vec<String>, Value)>::new();
    {
        let mut execute = |args: Vec<String>| {
            let reply = env.ok(&args.iter().map(String::as_str).collect::<Vec<_>>());
            recorded.push((args, reply.clone()));
            reply
        };
        let strings = |args: &[&str]| args.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        let start = execute(strings(&[
            "--request-id",
            "history-start",
            "work",
            "start",
            "--workbook",
            "two-step",
            "--flow",
            "default",
            "--input",
            "topic=historical facts",
        ]));
        let work = start["data"]["work_id"].as_str().unwrap();
        let first = execute(strings(&[
            "--request-id",
            "history-begin",
            "attempt",
            "begin",
            work,
            "--node",
            "outline",
        ]));
        let replaced = execute(strings(&[
            "--request-id",
            "history-replace",
            "attempt",
            "replace",
            work,
            "--attempt",
            first["data"]["attempt"].as_str().unwrap(),
            "--reason",
            "new real attempt",
        ]));
        execute(strings(&[
            "--request-id",
            "history-fail",
            "attempt",
            "fail",
            work,
            "--attempt",
            replaced["data"]["attempt"].as_str().unwrap(),
            "--reason",
            "actual failed attempt",
        ]));
        wait_for_a_later_utc_second();
        let retry = env.begin(work, "outline");
        for output in retry["data"]["outputs"].as_object().unwrap().values() {
            std::fs::write(output.as_str().unwrap(), b"actual completed outline\n").unwrap();
        }
        execute(strings(&[
            "--request-id",
            "history-submit",
            "attempt",
            "submit",
            work,
            "--attempt",
            retry["data"]["attempt"].as_str().unwrap(),
            "--summary",
            "real later success",
        ]));
        wait_for_a_later_utc_second();
        env.begin(work, "summary");
        execute(strings(&[
            "--request-id",
            "history-cancel",
            "work",
            "cancel",
            work,
        ]));
    }
    let before = store_rows(&env);
    let objects = audit_fixture_objects(&env);
    for (args, mut expected) in recorded {
        expected["data"]["replayed"] = json!(true);
        assert_eq!(
            env.ok(&args.iter().map(String::as_str).collect::<Vec<_>>()),
            expected
        );
    }
    assert_eq!(store_rows(&env), before);
    assert_eq!(audit_fixture_objects(&env), objects);
}

// Task: C002-T58
#[test]
fn historical_gate_replay_refuses_an_inconsistent_audit_subject_before_reading_frozen_files() {
    for field in ["other-principal", "empty-principal", "approval-time"] {
        for missing_frozen in [false, true] {
            let (env, work, args, original) = completed_approval_fixture();
            let before = store_rows(&env);
            let connection = Connection::open(env.dir.path().join("store.db")).unwrap();
            let principal = if field == "empty-principal" {
                ""
            } else {
                "record-check-other"
            };
            if field == "approval-time" {
                assert_eq!(
                    connection
                        .execute(
                            "UPDATE audit SET at='2000-01-01T00:00:00Z' WHERE request_id='recorded-approval'",
                            [],
                        )
                        .unwrap(),
                    1
                );
                assert_eq!(
                    connection
                        .execute(
                            "UPDATE requests SET at='2000-01-01T00:00:00Z' WHERE request_id='recorded-approval'",
                            [],
                        )
                        .unwrap(),
                    1
                );
            } else {
                assert_ne!(original["data"]["by"], principal);
                assert_eq!(
                    connection
                        .execute(
                            "UPDATE audit SET principal=?1 WHERE request_id='recorded-approval'",
                            [principal]
                        )
                        .unwrap(),
                    1
                );
            }
            let changed = store_rows(&env);
            let mut expected = before;
            let row = expected
                .get_mut("audit")
                .unwrap()
                .iter_mut()
                .find(|row| row[3] == rusqlite::types::Value::Text("recorded-approval".into()))
                .unwrap();
            if field == "approval-time" {
                row[6] = rusqlite::types::Value::Text("2000-01-01T00:00:00Z".into());
                let request = expected
                    .get_mut("requests")
                    .unwrap()
                    .iter_mut()
                    .find(|row| row[0] == rusqlite::types::Value::Text("recorded-approval".into()))
                    .unwrap();
                request[6] = rusqlite::types::Value::Text("2000-01-01T00:00:00Z".into());
            } else {
                row[4] = rusqlite::types::Value::Text(principal.into());
            }
            assert_eq!(changed, expected, "only the selected record columns change");
            if missing_frozen {
                use std::os::unix::fs::PermissionsExt;
                let parent = env.dir.path().join("works").join(&work);
                let permissions = std::fs::metadata(&parent).unwrap().permissions();
                std::fs::set_permissions(
                    &parent,
                    std::fs::Permissions::from_mode(permissions.mode() | 0o700),
                )
                .unwrap();
                let moved =
                    std::fs::rename(parent.join("workbook"), parent.join("retained-workbook"));
                std::fs::set_permissions(&parent, permissions).unwrap();
                moved.unwrap();
            }
            let objects = audit_fixture_objects(&env);
            let (error, exit) = env.fail(&args.iter().map(String::as_str).collect::<Vec<_>>());
            assert_eq!(exit, 1);
            assert_eq!(error["error"]["code"], "EFFECT_PENDING");
            assert_eq!(error["error"]["detail"]["cause"], "STORE_CORRUPT");
            assert_eq!(error["committed"], true);
            assert_eq!(error["request_id"], "recorded-approval");
            assert!(error.get("original").is_none(), "{error}");
            assert!(error.get("revision").is_none(), "{error}");
            assert!(
                error.to_string().contains("audit"),
                "pure record eligibility must precede frozen IO: {error}"
            );
            assert_eq!(store_rows(&env), changed);
            assert_eq!(audit_fixture_objects(&env), objects);
            let (status, exit) = env.fail(&["work", "status", &work]);
            assert_eq!(exit, 1);
            assert_eq!(status["error"]["code"], "STORE_CORRUPT");
            assert_eq!(store_rows(&env), changed);
            assert_eq!(audit_fixture_objects(&env), objects);
        }
    }
}

// Task: C002-T58
#[test]
fn historical_work_replay_refuses_empty_audit_subjects_and_self_consistent_wrong_event_times() {
    for field in ["start-principal", "start-time", "cancel-time"] {
        let env = Env::new();
        env.add_example("gated-release");
        let start_args = [
            "--request-id",
            "record-start",
            "work",
            "start",
            "--workbook",
            "gated-release",
            "--flow",
            "default",
            "--input",
            "version=event-time",
        ];
        let start = env.ok(&start_args);
        let work = start["data"]["work_id"].as_str().unwrap();
        let cancel_args = ["--request-id", "record-cancel", "work", "cancel", work];
        env.ok(&cancel_args);
        let (rid, args) = if field == "cancel-time" {
            ("record-cancel", cancel_args.as_slice())
        } else {
            ("record-start", start_args.as_slice())
        };
        assert_eq!(env.ok(args)["data"]["replayed"], true);
        let connection = Connection::open(env.dir.path().join("store.db")).unwrap();
        if field == "start-principal" {
            assert_eq!(
                connection
                    .execute("UPDATE audit SET principal='' WHERE request_id=?1", [rid])
                    .unwrap(),
                1
            );
        } else {
            let unrelated_time = "2000-01-01T00:00:00Z";
            assert_eq!(
                connection
                    .execute(
                        "UPDATE audit SET at=?1 WHERE request_id=?2",
                        rusqlite::params![unrelated_time, rid]
                    )
                    .unwrap(),
                1
            );
            assert_eq!(
                connection
                    .execute(
                        "UPDATE requests SET at=?1 WHERE request_id=?2",
                        rusqlite::params![unrelated_time, rid]
                    )
                    .unwrap(),
                1
            );
        }
        let before = store_rows(&env);
        let objects = audit_fixture_objects(&env);
        let (error, exit) = env.fail(args);
        assert_eq!(exit, 1);
        assert_eq!(error["error"]["code"], "EFFECT_PENDING");
        assert_eq!(error["error"]["detail"]["cause"], "STORE_CORRUPT");
        assert_eq!(error["committed"], true);
        assert_eq!(error["request_id"], rid);
        assert!(error.get("original").is_none(), "{field}: {error}");
        assert!(error.get("revision").is_none(), "{field}: {error}");
        assert_eq!(store_rows(&env), before);
        assert_eq!(audit_fixture_objects(&env), objects);
    }
}
