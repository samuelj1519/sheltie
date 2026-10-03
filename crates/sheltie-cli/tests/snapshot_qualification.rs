//! 合法快照形状不能替代对已提交业务事实的绑定。
#![allow(clippy::unwrap_used, clippy::expect_used)]
mod common;
use common::store::store_rows;
use common::*;
use rusqlite::Connection;
use serde_json::{Value, json};

fn business_files(
    env: &Env,
) -> std::collections::BTreeMap<std::path::PathBuf, (u32, Option<Vec<u8>>)> {
    use std::os::unix::fs::PermissionsExt;
    fn walk(
        root: &std::path::Path,
        path: &std::path::Path,
        out: &mut std::collections::BTreeMap<std::path::PathBuf, (u32, Option<Vec<u8>>)>,
    ) {
        let metadata = std::fs::symlink_metadata(path).unwrap();
        let bytes = if metadata.is_file() {
            Some(std::fs::read(path).unwrap())
        } else {
            assert!(metadata.is_dir());
            None
        };
        out.insert(
            path.strip_prefix(root).unwrap().to_owned(),
            (metadata.permissions().mode(), bytes),
        );
        if metadata.is_dir() {
            for entry in std::fs::read_dir(path).unwrap() {
                walk(root, &entry.unwrap().path(), out);
            }
        }
    }
    let mut out = std::collections::BTreeMap::new();
    for name in ["works", "workbooks", "pending"] {
        let path = env.dir.path().join(name);
        if path.exists() {
            walk(env.dir.path(), &path, &mut out);
        }
    }
    out
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
            error.to_string().contains("data与Reply"),
            "{field}: {error}"
        );
        assert!(error.get("original").is_none());
        assert_eq!(store_rows(&env), before);
        assert_eq!(business_files(&env), files_before);
        assert!(!frozen.exists());
        assert!(retained.is_dir());
    }
}
