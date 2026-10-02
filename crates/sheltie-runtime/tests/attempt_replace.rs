#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::collections::BTreeMap;
use std::sync::{Arc, Barrier};

use common::*;
use rusqlite::Connection;
use serde_json::{Value, json};
use sheltie_core::ErrorCode;
use sheltie_core::ids::{AttemptId, NodeId, WorkId};
use sheltie_core::work::Reply;
use sheltie_runtime::request::InputValue;
use sheltie_runtime::{Home, Response, StartArgs, WorkService};

fn attempt(name: &str) -> AttemptId {
    AttemptId::parse(name).unwrap()
}
fn node(name: &str) -> NodeId {
    NodeId::new(name).unwrap()
}

fn fixture(retries: u32, gate: bool) -> (OwnedTempDir, Home, WorkService, WorkId, Response) {
    let (directory, home) = temp_home();
    let source = directory.path().join("replacement-source");
    std::fs::create_dir_all(source.join("flows")).unwrap();
    std::fs::create_dir(source.join("resources")).unwrap();
    std::fs::write(source.join("resources/reference.txt"), b"frozen resource\n").unwrap();
    std::fs::write(source.join("workbook.toml"), "schema='workbook/v1'\nid='replacement'\nversion='1.0.0'\nname='Replacement'\nflows=['flows/default.toml']\n[[requires]]\nkind='skill'\nname='declared-tool'\nversion='1'\n").unwrap();
    std::fs::write(source.join("flows/default.toml"), format!(r#"schema='flow/v1'
id='default'
entry='implement'
[[nodes]]
id='implement'
title='Implement'
executor='agent'
instruction={{text='Keep the original frozen inputs and create report.md.'}}
inputs=[{{name='topic',from='start.topic'}},{{name='reference',from='resource.resources/reference.txt'}},{{name='stats',from='engine.stats'}},{{name='optional',from='finish.summary',required=false}}]
outputs=[{{name='report',path='report.md'}}]
requires=['skill:declared-tool']
gate={gate}
max_visits=2
max_retries={retries}
[[nodes]]
id='finish'
title='Finish'
executor='agent'
instruction={{text='Finish.'}}
outputs=[{{name='summary',path='summary.md'}}]
[[nodes]]
id='done'
title='Done'
executor='agent'
instruction={{text='Done.'}}
[[edges]]
from='implement'
to='finish'
kind='main'
[[edges]]
from='finish'
to='implement'
kind='back'
[[edges]]
from='finish'
to='done'
kind='main'
"#)).unwrap();
    repo(&home).add(&abs(&source), None).unwrap();
    let service = common::service(&home);
    let work = work_id_of(
        &service
            .start(
                StartArgs {
                    workbook_id: "replacement".into(),
                    version: None,
                    flow: "default".into(),
                    name: None,
                    inputs: inputs_lit(&[("topic", "frozen topic")]),
                },
                None,
            )
            .unwrap(),
    );
    let begun = service
        .begin(&work, &node("implement"), Some("original-begin".into()))
        .unwrap();
    (directory, home, service, work, begun)
}

fn state(connection: &Connection, work: &WorkId) -> Value {
    let raw: String = connection
        .query_row(
            "SELECT state_json FROM works WHERE work_id=?1",
            [work.as_str()],
            |row| row.get(0),
        )
        .unwrap();
    serde_json::from_str(&raw).unwrap()
}

fn set_state(connection: &Connection, work: &WorkId, state: &Value) {
    connection
        .execute(
            "UPDATE works SET state_json=?1 WHERE work_id=?2",
            rusqlite::params![state.to_string(), work.as_str()],
        )
        .unwrap();
}

fn effects(connection: &Connection, request: &str) -> Value {
    let raw: String = connection
        .query_row(
            "SELECT effects_json FROM requests WHERE request_id=?1",
            [request],
            |row| row.get(0),
        )
        .unwrap();
    serde_json::from_str(&raw).unwrap()
}

fn new_outputs(response: &Response) -> &BTreeMap<String, sheltie_core::path::AbsPath> {
    match &response.reply {
        Reply::AttemptReplaced { outputs, .. } => outputs,
        other => panic!("Expected replacement: {other:?}"),
    }
}

// Task: C005-T01
#[test]
fn replacement_records_one_atomic_pair_and_inherits_nonstats_refs_with_new_exact_stats() {
    let (_directory, home, service, work, begun) = fixture(1, false);
    write_output(&output_dir_of(&begun), "report.md", "old draft");
    let connection = Connection::open(home.store_path().as_str()).unwrap();
    let original = state(&connection, &work);
    let response = service
        .replace(
            &work,
            &attempt("implement#1.0"),
            &lit("take over"),
            Some("replace-once".into()),
        )
        .unwrap();
    assert_eq!(response.revision, 3);
    assert_eq!(response.data["replaced_attempt"], "implement#1.0");
    assert_eq!(response.data["attempt"], "implement#1.1");
    assert_eq!(response.data["number"], 1);
    assert!(response.data.get("retry").is_none());
    assert_eq!(response.data["requires"][0]["name"], "declared-tool");
    let current = state(&connection, &work);
    assert_eq!(current["attempts"].as_array().unwrap().len(), 2);
    assert_eq!(current["attempts"][0]["status"], "superseded");
    assert_eq!(current["attempts"][0]["replacement_reason"], "take over");
    assert_eq!(current["attempts"][0]["outputs"], json!({}));
    assert_eq!(current["attempts"][1]["status"], "running");
    assert_eq!(current["attempts"][1]["replacement_reason"], Value::Null);
    for input in ["topic", "reference", "optional"] {
        assert_eq!(
            current["attempts"][1]["inputs"][input],
            original["attempts"][0]["inputs"][input]
        );
    }
    assert_eq!(current["attempts"][1]["inputs"]["optional"], Value::Null);
    assert_ne!(
        current["attempts"][1]["inputs"]["stats"]["path"],
        original["attempts"][0]["inputs"]["stats"]["path"]
    );
    let stats: Value = serde_json::from_slice(
        &std::fs::read(
            current["attempts"][1]["inputs"]["stats"]["path"]
                .as_str()
                .unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(stats["nodes"][0]["attempts"], 2);
    assert_eq!(stats["nodes"][0]["failed"], 0);
    assert_eq!(stats["nodes"][0]["superseded"], 1);
    let registered = effects(&connection, "replace-once");
    for effect in registered
        .as_array()
        .unwrap()
        .iter()
        .filter(|effect| effect["kind"] == "write_file")
    {
        let bytes = std::fs::read(
            home.rel(effect["path"].as_str().unwrap())
                .unwrap()
                .as_path(),
        )
        .unwrap();
        assert_eq!(bytes, effect["content"].as_str().unwrap().as_bytes());
    }
    assert_eq!(
        std::fs::read(output_dir_of(&begun).join_segment("report.md").as_path()).unwrap(),
        b"old draft"
    );
    assert!(!new_outputs(&response)["report"].as_path().exists());
    assert!(service.status_read(&work).is_ok());
}

// Task: C005-T01
#[test]
fn failed_history_uses_its_original_prefix_after_replacement_and_later_exhaustion() {
    let (_directory, home, service, work, _) = fixture(1, false);
    service
        .replace(
            &work,
            &attempt("implement#1.0"),
            &lit("new executor"),
            Some("replace-history".into()),
        )
        .unwrap();
    let first = service
        .fail(
            &work,
            &attempt("implement#1.1"),
            &lit("first actual failure"),
            Some("first-failure".into()),
        )
        .unwrap();
    assert_eq!(first.data["work_status"], json!({"kind":"active"}));
    let next = service.begin(&work, &node("implement"), None).unwrap();
    assert_eq!(next.data["number"], 2);
    let second = service
        .fail(
            &work,
            &attempt("implement#1.2"),
            &lit("second actual failure"),
            None,
        )
        .unwrap();
    assert_eq!(
        second.data["work_status"],
        json!({"kind":"blocked","reason":"retries_exhausted"})
    );
    let replay = service
        .fail(
            &work,
            &attempt("implement#1.1"),
            &lit("first actual failure"),
            Some("first-failure".into()),
        )
        .unwrap();
    assert!(replay.replayed);
    assert_eq!(replay.data, first.data);
    assert_eq!(replay.revision, first.revision);
    assert!(service.status_read(&work).is_ok());
    let connection = Connection::open(home.store_path().as_str()).unwrap();
    let current = state(&connection, &work);
    assert_eq!(current["attempts"][0]["status"], "superseded");
    assert_eq!(current["attempts"][1]["id"]["number"], 1);
    assert_eq!(current["attempts"][2]["id"]["number"], 2);
}

// Task: C005-T01
#[test]
fn late_old_writes_and_second_replacement_are_rejected_without_new_records() {
    let (_directory, home, service, work, _) = fixture(0, true);
    let response = service
        .replace(&work, &attempt("implement#1.0"), &lit("new executor"), None)
        .unwrap();
    let connection = Connection::open(home.store_path().as_str()).unwrap();
    for (kind, error) in [
        (
            "submit",
            service
                .submit(
                    &work,
                    &attempt("implement#1.0"),
                    &lit("late"),
                    Some("late-submit".into()),
                )
                .unwrap_err(),
        ),
        (
            "fail",
            service
                .fail(
                    &work,
                    &attempt("implement#1.0"),
                    &lit("late"),
                    Some("late-fail".into()),
                )
                .unwrap_err(),
        ),
        (
            "quota",
            service
                .replace(
                    &work,
                    &attempt("implement#1.1"),
                    &lit("twice"),
                    Some("second-replace".into()),
                )
                .unwrap_err(),
        ),
    ] {
        assert_eq!(
            error.code(),
            if kind == "quota" {
                ErrorCode::ReplacementsExhausted
            } else {
                ErrorCode::AttemptNotRunning
            }
        );
    }
    assert_eq!(
        state(&connection, &work)["attempts"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(connection.query_row("SELECT COUNT(*) FROM requests WHERE request_id IN ('late-submit','late-fail','second-replace')",[],|row|row.get::<_,i64>(0)).unwrap(),0);
    std::fs::write(
        new_outputs(&response)["report"].as_path(),
        b"accepted report",
    )
    .unwrap();
    let submitted = service
        .submit(&work, &attempt("implement#1.1"), &lit("completed"), None)
        .unwrap();
    assert_eq!(
        submitted.data["work_status"],
        json!({"kind":"blocked","reason":"gate"})
    );
    service.approve(&work, &node("implement"), None).unwrap();
    assert!(
        service
            .status_read(&work)
            .unwrap()
            .1
            .card
            .next
            .iter()
            .any(|next| next["op"] == "attempt begin")
    );
}

// Task: C005-T01
#[test]
fn replacement_replay_after_new_attempt_ends_preserves_reason_source_identity_and_bytes() {
    let (directory, home, service, work, _) = fixture(1, false);
    let reason_file = directory.path().join("reason.txt");
    std::fs::write(&reason_file, b"canonical reason").unwrap();
    let reason = InputValue::AtFile {
        path: reason_file.to_str().unwrap().into(),
    };
    let replaced = service
        .replace(
            &work,
            &attempt("implement#1.0"),
            &reason,
            Some("replace-file".into()),
        )
        .unwrap();
    let connection = Connection::open(home.store_path().as_str()).unwrap();
    let registered = effects(&connection, "replace-file");
    let files = registered
        .as_array()
        .unwrap()
        .iter()
        .filter(|effect| effect["kind"] == "write_file")
        .map(|effect| {
            (
                home.rel(effect["path"].as_str().unwrap()).unwrap(),
                effect["content"].as_str().unwrap().as_bytes().to_vec(),
            )
        })
        .collect::<Vec<_>>();
    service
        .fail(
            &work,
            &attempt("implement#1.1"),
            &lit("execution failed"),
            None,
        )
        .unwrap();
    std::fs::remove_file(&reason_file).unwrap();
    for (path, _) in &files {
        std::fs::remove_file(path.as_path()).unwrap();
    }
    let before = store_rows(&connection);
    let replay = service
        .replace(
            &work,
            &attempt("implement#1.0"),
            &reason,
            Some("replace-file".into()),
        )
        .unwrap();
    assert!(replay.replayed);
    assert_eq!(replay.data, replaced.data);
    assert_eq!(replay.revision, replaced.revision);
    for (path, bytes) in files {
        assert_eq!(std::fs::read(path.as_path()).unwrap(), bytes);
    }
    assert_eq!(store_rows(&connection), before);
    assert_eq!(
        service
            .replace(
                &work,
                &attempt("implement#1.0"),
                &lit("different source"),
                Some("replace-file".into())
            )
            .unwrap_err()
            .code(),
        ErrorCode::RequestConflict
    );
    let begun = service
        .begin(&work, &node("implement"), Some("ordinary-begin".into()))
        .unwrap();
    service
        .fail(&work, &attempt("implement#1.2"), &lit("last failure"), None)
        .unwrap();
    assert_eq!(
        service
            .begin(&work, &node("implement"), Some("ordinary-begin".into()))
            .unwrap()
            .data,
        begun.data
    );
}

// Task: C005-T01
#[test]
fn qualification_precedes_reason_file_reads_and_modified_input_never_commits() {
    let (directory, home, service, work, _) = fixture(1, false);
    let missing = InputValue::AtFile {
        path: directory
            .path()
            .join("missing-reason")
            .to_str()
            .unwrap()
            .into(),
    };
    assert_eq!(
        service
            .replace(
                &work,
                &attempt("implement#1.9"),
                &missing,
                Some("missing-attempt".into())
            )
            .unwrap_err()
            .code(),
        ErrorCode::NotFound
    );
    let connection = Connection::open(home.store_path().as_str()).unwrap();
    let input = home
        .work_dir(&work)
        .join_segment("start-inputs")
        .join_segment("topic");
    std::fs::write(input.as_path(), b"changed").unwrap();
    let before = store_rows(&connection);
    assert_eq!(
        service
            .replace(
                &work,
                &attempt("implement#1.0"),
                &lit("take over"),
                Some("changed-input".into())
            )
            .unwrap_err()
            .code(),
        ErrorCode::ArtifactModified
    );
    assert_eq!(store_rows(&connection), before);
    std::fs::write(input.as_path(), b"frozen topic").unwrap();
    service
        .replace(&work, &attempt("implement#1.0"), &lit("take over"), None)
        .unwrap();
    assert_eq!(
        service
            .replace(
                &work,
                &attempt("implement#1.1"),
                &missing,
                Some("quota-before-read".into())
            )
            .unwrap_err()
            .code(),
        ErrorCode::ReplacementsExhausted
    );
    assert_eq!(
        service
            .replace(
                &work,
                &attempt("implement#1.0"),
                &missing,
                Some("old-before-read".into())
            )
            .unwrap_err()
            .code(),
        ErrorCode::AttemptNotRunning
    );
}

// Task: C005-T01
#[test]
fn replacement_and_submit_or_two_replacements_have_exactly_one_committed_winner() {
    for submit in [true, false] {
        let (_directory, home, service, work, begun) = fixture(1, false);
        write_output(&output_dir_of(&begun), "report.md", "completed original");
        let barrier = Arc::new(Barrier::new(3));
        let first_service = service.clone();
        let first_work = work.clone();
        let first_barrier = barrier.clone();
        let first = std::thread::spawn(move || {
            first_barrier.wait();
            first_service.replace(
                &first_work,
                &attempt("implement#1.0"),
                &lit("first replacement"),
                Some("race-replace".into()),
            )
        });
        let second_service = service.clone();
        let second_work = work.clone();
        let second_barrier = barrier.clone();
        let second = std::thread::spawn(move || {
            second_barrier.wait();
            if submit {
                second_service.submit(
                    &second_work,
                    &attempt("implement#1.0"),
                    &lit("completed"),
                    Some("race-other".into()),
                )
            } else {
                second_service.replace(
                    &second_work,
                    &attempt("implement#1.0"),
                    &lit("second replacement"),
                    Some("race-other".into()),
                )
            }
        });
        barrier.wait();
        let first = first.join().unwrap();
        let second = second.join().unwrap();
        assert_ne!(first.is_ok(), second.is_ok());
        let failure = first
            .as_ref()
            .err()
            .or_else(|| second.as_ref().err())
            .unwrap();
        assert_eq!(failure.code(), ErrorCode::AttemptNotRunning);
        let connection = Connection::open(home.store_path().as_str()).unwrap();
        assert_eq!(
            connection
                .query_row(
                    "SELECT revision FROM works WHERE work_id=?1",
                    [work.as_str()],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            3
        );
        assert_eq!(connection.query_row("SELECT COUNT(*) FROM requests WHERE request_id IN ('race-replace','race-other')",[],|row|row.get::<_,i64>(0)).unwrap(),1);
        let current = state(&connection, &work);
        if first.is_ok() || !submit {
            assert_eq!(current["attempts"].as_array().unwrap().len(), 2);
            assert_eq!(current["attempts"][0]["status"], "superseded");
            assert_eq!(current["attempts"][1]["status"], "running");
        } else {
            assert_eq!(current["attempts"].as_array().unwrap().len(), 1);
            assert_eq!(current["attempts"][0]["status"], "succeeded");
        }
        assert!(service.status_read(&work).is_ok());
    }
}

// Task: C005-T01
#[test]
fn schema_three_and_missing_nullable_reason_are_rejected_without_rewriting_records() {
    let (_directory, home, service, work, _) = fixture(1, false);
    let connection = Connection::open(home.store_path().as_str()).unwrap();
    let original = state(&connection, &work);
    assert_eq!(original["attempts"][0]["replacement_reason"], Value::Null);
    let mut missing = original.clone();
    missing["attempts"][0]
        .as_object_mut()
        .unwrap()
        .remove("replacement_reason");
    set_state(&connection, &work, &missing);
    let before = store_rows(&connection);
    assert_eq!(
        service.status_read(&work).unwrap_err().code(),
        ErrorCode::StoreCorrupt
    );
    assert_eq!(store_rows(&connection), before);
    set_state(&connection, &work, &original);
    connection.pragma_update(None, "user_version", 3).unwrap();
    drop(connection);
    let database = std::fs::read(home.store_path().as_path()).unwrap();
    assert_eq!(
        service.status_read(&work).unwrap_err().code(),
        ErrorCode::StoreSchemaMismatch
    );
    assert_eq!(
        std::fs::read(home.store_path().as_path()).unwrap(),
        database
    );
}

// Task: C005-T01
#[test]
fn replacement_history_rejects_forged_reason_input_binding_and_snapshot_identity() {
    for field in [
        "reason",
        "observed_input",
        "snapshot_number",
        "old_identity",
    ] {
        let (_directory, home, service, work, _) = fixture(1, false);
        service
            .replace(
                &work,
                &attempt("implement#1.0"),
                &lit("canonical"),
                Some("strict-replace".into()),
            )
            .unwrap();
        let connection = Connection::open(home.store_path().as_str()).unwrap();
        let (table, column) = if field == "reason" || field == "observed_input" {
            ("audit", "command_json")
        } else {
            ("requests", "reply_json")
        };
        let raw: String = connection
            .query_row(
                &format!("SELECT {column} FROM {table} WHERE request_id='strict-replace'"),
                [],
                |row| row.get(0),
            )
            .unwrap();
        let mut payload: Value = serde_json::from_str(&raw).unwrap();
        match field {
            "reason" => payload["reason"] = json!("forged"),
            "observed_input" => payload["observed_inputs"]["topic"]["bytes"] = json!(1),
            "snapshot_number" => payload["data"]["number"] = json!(2),
            "old_identity" => payload["reply"]["replaced_attempt"]["number"] = json!(1),
            _ => unreachable!(),
        }
        connection
            .execute(
                &format!("UPDATE {table} SET {column}=?1 WHERE request_id='strict-replace'"),
                [payload.to_string()],
            )
            .unwrap();
        let before = store_rows(&connection);
        assert_eq!(
            service.status_read(&work).unwrap_err().code(),
            ErrorCode::StoreCorrupt,
            "{field}"
        );
        assert_effect_pending_without_original(
            service
                .replace(
                    &work,
                    &attempt("implement#1.0"),
                    &lit("canonical"),
                    Some("strict-replace".into()),
                )
                .unwrap_err(),
            true,
            "strict-replace",
            None,
        );
        assert_eq!(store_rows(&connection), before);
    }
}

// Task: C005-T01
#[test]
fn replacement_complete_payloads_are_decoded_before_frozen_workbook_io() {
    for field in ["command", "reply", "data", "effects"] {
        let (_directory, home, service, work, _) = fixture(1, false);
        service
            .replace(
                &work,
                &attempt("implement#1.0"),
                &lit("canonical"),
                Some("strict-replace".into()),
            )
            .unwrap();
        let connection = Connection::open(home.store_path().as_str()).unwrap();
        let (table, column) = if field == "command" {
            ("audit", "command_json")
        } else if field == "effects" {
            ("requests", "effects_json")
        } else {
            ("requests", "reply_json")
        };
        let raw: String = connection
            .query_row(
                &format!("SELECT {column} FROM {table} WHERE request_id='strict-replace'"),
                [],
                |row| row.get(0),
            )
            .unwrap();
        let mut payload: Value = serde_json::from_str(&raw).unwrap();
        let target = match field {
            "command" => &mut payload,
            "reply" => &mut payload["reply"],
            "data" => &mut payload["data"],
            "effects" => &mut payload[0],
            _ => unreachable!(),
        };
        target["unexpected_contract_field"] = json!(true);
        connection
            .execute(
                &format!("UPDATE {table} SET {column}=?1 WHERE request_id='strict-replace'"),
                [payload.to_string()],
            )
            .unwrap();
        let frozen = home.work_dir(&work).join_segment("workbook");
        std::fs::rename(
            frozen.as_path(),
            home.work_dir(&work)
                .join_segment("retained-workbook")
                .as_path(),
        )
        .unwrap();
        let before = store_rows(&connection);
        let error = service.status_read(&work).unwrap_err();
        assert_eq!(error.code(), ErrorCode::StoreCorrupt);
        assert!(
            error.to_string().contains("unexpected_contract_field"),
            "{field}: {error}"
        );
        assert_eq!(store_rows(&connection), before);
    }
}

// Task: C005-T01
#[test]
fn replacement_with_no_inputs_rejects_changed_entry_source_before_another_write() {
    let (directory, home) = temp_home();
    let source = directory.path().join("empty-input-source");
    std::fs::create_dir_all(source.join("flows")).unwrap();
    std::fs::write(source.join("workbook.toml"),"schema='workbook/v1'\nid='empty-input'\nversion='1.0.0'\nname='Empty input'\nflows=['flows/default.toml']\n").unwrap();
    std::fs::write(source.join("flows/default.toml"),"schema='flow/v1'\nid='default'\nentry='first'\n[[nodes]]\nid='first'\ntitle='First'\nexecutor='agent'\ninstruction={text='First.'}\n[[nodes]]\nid='working'\ntitle='Working'\nexecutor='agent'\ninstruction={text='Working.'}\n[[nodes]]\nid='done'\ntitle='Done'\nexecutor='agent'\ninstruction={text='Done.'}\n[[edges]]\nfrom='first'\nto='working'\nkind='main'\n[[edges]]\nfrom='working'\nto='done'\nkind='main'\n").unwrap();
    repo(&home).add(&abs(&source), None).unwrap();
    let service = common::service(&home);
    let work = work_id_of(
        &service
            .start(
                StartArgs {
                    workbook_id: "empty-input".into(),
                    version: None,
                    flow: "default".into(),
                    name: None,
                    inputs: inputs_lit(&[]),
                },
                None,
            )
            .unwrap(),
    );
    service.begin(&work, &node("first"), None).unwrap();
    service
        .submit(&work, &attempt("first#1.0"), &lit("done"), None)
        .unwrap();
    service.begin(&work, &node("working"), None).unwrap();
    service
        .replace(&work, &attempt("working#1.0"), &lit("take over"), None)
        .unwrap();
    let connection = Connection::open(home.store_path().as_str()).unwrap();
    let mut current = state(&connection, &work);
    assert_eq!(current["attempts"][1]["inputs"], json!({}));
    assert_eq!(
        current["attempts"][2]["entered_from"],
        current["attempts"][1]["entered_from"]
    );
    assert!(!current["attempts"][2]["entered_from"].is_null());
    current["attempts"][2]["entered_from"] = Value::Null;
    set_state(&connection, &work, &current);
    let before = store_rows(&connection);
    assert_eq!(
        service
            .fail(
                &work,
                &attempt("working#1.1"),
                &lit("failed"),
                Some("after-origin-drift".into())
            )
            .unwrap_err()
            .code(),
        ErrorCode::StoreCorrupt
    );
    assert_eq!(store_rows(&connection), before);
}

// Task: C005-T01
#[cfg(feature = "failpoint")]
#[test]
fn replacement_transaction_crash_boundaries_preserve_exact_committed_history() {
    if let Ok(home) = std::env::var("C005_REPLACEMENT_PROBE_HOME") {
        let home = Home::resolve(Some(&home)).unwrap();
        let work = WorkId::parse(&std::env::var("C005_REPLACEMENT_PROBE_WORK").unwrap()).unwrap();
        common::service(&home)
            .replace(
                &work,
                &attempt("implement#1.0"),
                &lit("crash reason"),
                Some("crash-replace".into()),
            )
            .unwrap();
        panic!("指定故障点没有退出进程");
    }
    for point in ["before_commit", "after_commit_before_effects"] {
        let (_directory, home, service, work, _) = fixture(1, false);
        let connection = Connection::open(home.store_path().as_str()).unwrap();
        let before = store_rows(&connection);
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "replacement_transaction_crash_boundaries_preserve_exact_committed_history",
                "--nocapture",
            ])
            .env("C005_REPLACEMENT_PROBE_HOME", home.root().as_str())
            .env("C005_REPLACEMENT_PROBE_WORK", work.as_str())
            .env("SHELTIE_FAILPOINT", point)
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(70),
            "{point}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        if point == "before_commit" {
            assert_eq!(store_rows(&connection), before);
            assert_eq!(
                state(&connection, &work)["attempts"]
                    .as_array()
                    .unwrap()
                    .len(),
                1
            );
        } else {
            assert_eq!(
                state(&connection, &work)["attempts"][0]["status"],
                "superseded"
            );
            let registered = effects(&connection, "crash-replace");
            let expected = registered
                .as_array()
                .unwrap()
                .iter()
                .filter(|effect| effect["kind"] == "write_file")
                .map(|effect| {
                    (
                        home.rel(effect["path"].as_str().unwrap()).unwrap(),
                        effect["content"].as_str().unwrap().as_bytes().to_vec(),
                    )
                })
                .collect::<Vec<_>>();
            assert_eq!(expected.len(), 2);
            for (path, _) in &expected {
                assert!(!path.as_path().exists());
            }
            let replay = service
                .replace(
                    &work,
                    &attempt("implement#1.0"),
                    &lit("crash reason"),
                    Some("crash-replace".into()),
                )
                .unwrap();
            assert!(replay.replayed);
            assert_eq!(replay.data["attempt"], "implement#1.1");
            for (path, bytes) in expected {
                assert_eq!(std::fs::read(path.as_path()).unwrap(), bytes);
            }
        }
    }
}

// Task: C005-T01
#[cfg(feature = "failpoint")]
#[test]
fn replacement_rejects_a_path_swap_after_opening_the_original_input() {
    let (_directory, home, service, work, _) = fixture(1, false);
    let input = home
        .work_dir(&work)
        .join_segment("start-inputs")
        .join_segment("topic");
    let rendezvous = tempfile::tempdir().unwrap();
    sheltie_runtime::failpoint::arm_rendezvous(
        "replace_after_input_open",
        input.as_str(),
        rendezvous.path(),
    )
    .unwrap();
    let writer_service = service.clone();
    let writer_work = work.clone();
    let worker = std::thread::spawn(move || {
        writer_service.replace(
            &writer_work,
            &attempt("implement#1.0"),
            &lit("take over"),
            Some("path-swap".into()),
        )
    });
    let mut worker = RendezvousWorker::single(worker, rendezvous.path());
    worker.wait("替换输入没有停在受限打开后的同步点");
    let retained = input.as_path().with_file_name("retained-topic");
    std::fs::rename(input.as_path(), &retained).unwrap();
    std::fs::write(input.as_path(), b"frozen topic").unwrap();
    let connection = Connection::open(home.store_path().as_str()).unwrap();
    let before = store_rows(&connection);
    let error = worker.finish().unwrap().unwrap_err();
    assert_eq!(error.code(), ErrorCode::InvalidRequest);
    assert_eq!(store_rows(&connection), before);
    assert_eq!(std::fs::read(&retained).unwrap(), b"frozen topic");
    assert_eq!(std::fs::read(input.as_path()).unwrap(), b"frozen topic");
}
