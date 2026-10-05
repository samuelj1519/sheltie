#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::*;
use rusqlite::Connection;
use serde_json::{Value, json};
use sheltie_core::ErrorCode;
use sheltie_core::ids::{AttemptId, NodeId};
use sheltie_runtime::{Home, StartArgs, WorkService};

fn begin_outline(service: &WorkService, work: &sheltie_core::ids::WorkId) {
    service
        .begin(
            work,
            &NodeId::new("outline").unwrap(),
            Some("read-begin".into()),
        )
        .unwrap();
}

// Task: C004-T01
#[test]
fn status_read_combines_revision_and_current_frozen_pointers_without_writing() {
    let (_dir, home, service) = home_with_example("two-step");
    let started = start_two_step(&service);
    let work = work_id_of(&started);
    let (_, initial) = service.status_read(&work).unwrap();
    assert_eq!(initial.revision, 1);
    assert!(initial.card.resume.is_none());
    assert!(!initial.effects_pending && !initial.pending_publish);
    begin_outline(&service, &work);
    let connection = Connection::open(home.store_path().as_str()).unwrap();
    let before = store_rows(&connection);
    let card_path = home.work_dir(&work).join_segment("status-card.md");
    let card_bytes = std::fs::read(card_path.as_path()).unwrap();
    let (text, view) = service.status_read(&work).unwrap();
    assert_eq!(view.revision, 2);
    let resume = view.card.resume.as_ref().unwrap();
    assert_eq!(resume.attempt, "outline#1.0");
    assert!(resume.brief_path.as_str().ends_with("attempt-000/brief.md"));
    assert!(
        resume.draft_outputs["outline"]
            .as_str()
            .ends_with("outputs/outline.md")
    );
    assert_eq!(
        resume.inputs["topic"].as_ref().unwrap().bytes,
        "给新人介绍 Sheltie".len() as u64
    );
    assert!(text.contains("revision: 2") && text.contains("effects_pending: false"));
    assert_eq!(store_rows(&connection), before);
    assert_eq!(std::fs::read(card_path.as_path()).unwrap(), card_bytes);
}

// Task: C004-T01
#[test]
fn status_read_rejects_incomplete_or_conflicting_request_audit_closure() {
    for change in [
        "missing_request",
        "missing_audit",
        "request_work",
        "audit_work",
        "both_work",
        "duplicate_audit",
        "published",
        "completed_effects",
        "snapshot_revision",
        "audit_time",
        "execution_time",
        "principal",
    ] {
        let (_dir, home, service) = home_with_example("two-step");
        let work = work_id_of(&start_two_step(&service));
        begin_outline(&service, &work);
        let connection = Connection::open(home.store_path().as_str()).unwrap();
        match change {
            "missing_request" => connection.execute("DELETE FROM requests WHERE request_id='read-begin'", []).unwrap(),
            "missing_audit" => connection.execute("DELETE FROM audit WHERE request_id='read-begin'", []).unwrap(),
            "request_work" => connection.execute("UPDATE requests SET work_id='other' WHERE request_id='read-begin'", []).unwrap(),
            "audit_work" => connection.execute("UPDATE audit SET work_id='other' WHERE request_id='read-begin'", []).unwrap(),
            "both_work" => {
                connection.execute("UPDATE requests SET work_id='other' WHERE request_id='read-begin'", []).unwrap();
                connection.execute("UPDATE audit SET work_id='other' WHERE request_id='read-begin'", []).unwrap()
            },
            "duplicate_audit" => connection.execute("INSERT INTO audit(work_id,revision,request_id,principal,command_json,at) SELECT work_id,revision,request_id,principal,command_json,at FROM audit WHERE request_id='read-begin'", []).unwrap(),
            "published" => connection.execute("UPDATE requests SET published=2 WHERE request_id='read-begin'", []).unwrap(),
            "completed_effects" => connection.execute("UPDATE requests SET effects_json='[]' WHERE request_id='read-begin'", []).unwrap(),
            "snapshot_revision" => {
                let raw: String = connection.query_row("SELECT reply_json FROM requests WHERE request_id='read-begin'", [], |row| row.get(0)).unwrap();
                let mut snapshot: Value = serde_json::from_str(&raw).unwrap();
                snapshot["revision"] = json!(1);
                connection.execute("UPDATE requests SET reply_json=?1 WHERE request_id='read-begin'", [snapshot.to_string()]).unwrap()
            },
            "audit_time" => connection.execute("UPDATE audit SET at='1999-01-01T00:00:00Z' WHERE request_id='read-begin'", []).unwrap(),
            "execution_time" => {
                connection.execute("UPDATE requests SET at='1999-01-01T00:00:00Z' WHERE request_id='read-begin'", []).unwrap();
                connection.execute("UPDATE audit SET at='1999-01-01T00:00:00Z' WHERE request_id='read-begin'", []).unwrap()
            },
            "principal" => connection.execute("UPDATE audit SET principal='' WHERE request_id='read-begin'", []).unwrap(),
            _ => unreachable!(),
        };
        let before = store_rows(&connection);
        let error = service.status_read(&work).unwrap_err();
        assert_eq!(error.code(), ErrorCode::StoreCorrupt, "{change}: {error}");
        assert_eq!(store_rows(&connection), before, "{change}");
    }
}

// Task: C004-T01
#[test]
fn status_read_reports_this_work_pending_effects_and_ignores_unrelated_corruption() {
    let (_dir, home, service) = home_with_example("two-step");
    let work = work_id_of(&start_two_step(&service));
    begin_outline(&service, &work);
    let other = work_id_of(&start_two_step(&service));
    let unrelated = service
        .begin(
            &other,
            &NodeId::new("outline").unwrap(),
            Some("other-begin".into()),
        )
        .unwrap();
    let connection = Connection::open(home.store_path().as_str()).unwrap();
    connection
        .execute(
            "UPDATE requests SET effects_json='[]',published=0 WHERE request_id=?1",
            [&unrelated.request_id],
        )
        .unwrap();
    let before = store_rows(&connection);
    assert!(!service.status_read(&work).unwrap().1.effects_pending);
    connection
        .execute(
            "UPDATE requests SET published=0 WHERE request_id='read-begin'",
            [],
        )
        .unwrap();
    let pending_before = store_rows(&connection);
    let (text, view) = service.status_read(&work).unwrap();
    assert_eq!(view.revision, 2);
    assert!(view.effects_pending);
    assert!(!view.pending_publish);
    assert!(text.contains("File effects are pending"));
    assert_eq!(store_rows(&connection), pending_before);
    assert_ne!(before, pending_before);
}

// Task: C004-T01
#[cfg(feature = "failpoint")]
#[test]
fn status_read_uses_one_snapshot_when_a_new_attempt_commits_between_queries() {
    let (_dir, _home, service) = home_with_example("two-step");
    let work = work_id_of(&start_two_step(&service));
    let rendezvous = tempfile::tempdir().unwrap();
    sheltie_runtime::failpoint::arm_rendezvous(
        "read_bundle_after_work",
        work.as_str(),
        rendezvous.path(),
    )
    .unwrap();
    let reader_service = service.clone();
    let reader_work = work.clone();
    let reader = std::thread::spawn(move || reader_service.status_read(&reader_work));
    let mut reader = RendezvousWorker::single(reader, rendezvous.path());
    reader.wait("Read transaction did not capture Work state");
    begin_outline(&service, &work);
    let (_, view) = reader.finish().unwrap().unwrap();
    assert_eq!(view.revision, 1);
    assert!(view.card.resume.is_none());
    assert!(!view.effects_pending);
    assert_eq!(view.card.next[0]["op"], "attempt begin");
    let current = service.status_read(&work).unwrap().1;
    assert_eq!(current.revision, 2);
    assert_eq!(current.card.resume.unwrap().attempt, "outline#1.0");
}

// Task: C004-T01
#[test]
fn schema_two_is_rejected_without_migrating_or_rewriting_original_bytes() {
    let (_dir, home, service) = home_with_example("two-step");
    let work = work_id_of(&start_two_step(&service));
    let connection = Connection::open(home.store_path().as_str()).unwrap();
    connection.pragma_update(None, "user_version", 2).unwrap();
    drop(connection);
    let before = std::fs::read(home.store_path().as_path()).unwrap();
    let error = service.status_read(&work).unwrap_err();
    assert_eq!(error.code(), ErrorCode::StoreSchemaMismatch);
    assert_eq!(std::fs::read(home.store_path().as_path()).unwrap(), before);
}

fn result_fixture(
    gate: bool,
    selected: bool,
) -> (OwnedTempDir, Home, WorkService, sheltie_core::ids::WorkId) {
    let (directory, home) = temp_home();
    let source = directory.path().join("source");
    std::fs::create_dir_all(source.join("flows")).unwrap();
    std::fs::write(source.join("workbook.toml"), "schema='workbook/v1'\nid='results'\nversion='1.0.0'\nname='Results'\nflows=['flows/default.toml']\n").unwrap();
    std::fs::write(source.join("flows/default.toml"), format!("schema='flow/v1'\nid='default'\nentry='deliver'\n[[nodes]]\nid='deliver'\ntitle='Deliver'\nexecutor='agent'\ninstruction={{text='Write final bytes.'}}\ngate={gate}\ninputs=[{{name='origin',from='start.input',result={selected}}}]\noutputs=[{{name='archive',path='final.bin',result={selected}}}]\n")).unwrap();
    repo(&home).add(&abs(&source), None).unwrap();
    let service = common::service(&home);
    let started = service
        .start(
            StartArgs {
                workbook_id: "results".into(),
                version: None,
                flow: "default".into(),
                name: None,
                inputs: inputs_lit(&[("input", "frozen original")]),
            },
            None,
        )
        .unwrap();
    let work = work_id_of(&started);
    (directory, home, service, work)
}

fn resource_result_fixture() -> (OwnedTempDir, Home, WorkService, sheltie_core::ids::WorkId) {
    let (directory, home) = temp_home();
    let source = directory.path().join("resource-source");
    std::fs::create_dir_all(source.join("flows")).unwrap();
    std::fs::create_dir_all(source.join("resources")).unwrap();
    std::fs::write(source.join("resources/reference.bin"), b"resource bytes\n").unwrap();
    std::fs::write(source.join("workbook.toml"), "schema='workbook/v1'\nid='resource-result'\nversion='1.0.0'\nname='Resource result'\nflows=['flows/default.toml']\n").unwrap();
    std::fs::write(source.join("flows/default.toml"), "schema='flow/v1'\nid='default'\nentry='deliver'\n[[nodes]]\nid='deliver'\ntitle='Deliver'\nexecutor='agent'\ninstruction={text='Use the frozen resource.'}\ninputs=[{name='reference',from='resource.resources/reference.bin',result=true}]\n").unwrap();
    repo(&home).add(&abs(&source), None).unwrap();
    let service = common::service(&home);
    let work = work_id_of(
        &service
            .start(
                StartArgs {
                    workbook_id: "resource-result".into(),
                    version: None,
                    flow: "default".into(),
                    name: None,
                    inputs: inputs_lit(&[]),
                },
                Some("resource-start".into()),
            )
            .unwrap(),
    );
    service
        .begin(&work, &NodeId::new("deliver").unwrap(), None)
        .unwrap();
    service
        .submit(
            &work,
            &AttemptId::parse("deliver#1.0").unwrap(),
            &lit("resource bound"),
            None,
        )
        .unwrap();
    (directory, home, service, work)
}

// Task: C004-T01
#[test]
fn result_rejects_resource_reference_digest_or_size_drift_without_changing_originals() {
    let (directory, home, service, work) = resource_result_fixture();
    let source = directory
        .path()
        .join("resource-source/resources/reference.bin");
    let frozen = home
        .work_dir(&work)
        .join_segment("workbook")
        .join_segment("resources")
        .join_segment("reference.bin");
    let source_before = snapshot(&source);
    let frozen_before = snapshot(frozen.as_path().as_std_path());
    let original = service.result(&work).unwrap().0;
    assert!(original.r#final);
    assert_eq!(original.artifacts.len(), 1);
    assert_eq!(original.artifacts[0].bytes, 15);
    assert_eq!(original.artifacts[0].path, frozen);
    assert_eq!(std::fs::read(&source).unwrap(), b"resource bytes\n");
    let connection = Connection::open(home.store_path().as_str()).unwrap();
    let raw: String = connection
        .query_row(
            "SELECT state_json FROM works WHERE work_id=?1",
            [work.as_str()],
            |row| row.get(0),
        )
        .unwrap();
    let original_state: Value = serde_json::from_str(&raw).unwrap();
    for field in ["sha256", "bytes"] {
        let mut state = original_state.clone();
        let reference = &mut state["attempts"][0]["inputs"]["reference"];
        reference[field] = match field {
            "sha256" => json!("0".repeat(64)),
            "bytes" => json!(16),
            _ => unreachable!(),
        };
        connection
            .execute(
                "UPDATE works SET state_json=?1 WHERE work_id=?2",
                rusqlite::params![state.to_string(), work.as_str()],
            )
            .unwrap();
        let before = store_rows(&connection);
        for error in [
            service.result(&work).unwrap_err(),
            service.status_read(&work).unwrap_err(),
        ] {
            assert_eq!(error.code(), ErrorCode::StoreCorrupt, "{field}: {error}");
            assert!(error.to_string().contains("reference"), "{field}: {error}");
        }
        assert_eq!(store_rows(&connection), before);
        assert_eq!(snapshot(&source), source_before);
        assert_eq!(snapshot(frozen.as_path().as_std_path()), frozen_before);
    }
}

// Task: C004-T01
#[test]
fn pending_resource_loading_compares_logical_path_and_observed_bytes_separately() {
    let (_directory, home, service, work) = resource_result_fixture();
    let connection = Connection::open(home.store_path().as_str()).unwrap();
    let raw: String = connection
        .query_row(
            "SELECT effects_json FROM requests WHERE request_id='resource-start'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let effects: Value = serde_json::from_str(&raw).unwrap();
    let payload = home.rel(effects[0]["pending"].as_str().unwrap()).unwrap();
    std::fs::rename(home.work_dir(&work).as_path(), payload.as_path()).unwrap();
    connection
        .execute(
            "UPDATE requests SET published=0 WHERE request_id='resource-start'",
            [],
        )
        .unwrap();
    let before = store_rows(&connection);
    let (_, status) = service.status_read(&work).unwrap();
    assert!(status.effects_pending && status.pending_publish);
    let reference = status.card.resume.unwrap().inputs["reference"]
        .clone()
        .unwrap();
    assert!(
        reference
            .path
            .as_str()
            .contains(&format!("works/{work}/workbook/resources/reference.bin"))
    );
    assert_eq!(reference.bytes, 15);
    assert_eq!(
        std::fs::read(
            payload
                .join_segment("workbook")
                .join_segment("resources")
                .join_segment("reference.bin")
                .as_path()
        )
        .unwrap(),
        b"resource bytes\n"
    );
    assert_eq!(store_rows(&connection), before);
}

// Task: C004-T01
#[test]
fn all_read_payloads_reject_unknown_fields_before_frozen_workbook_io() {
    for field in ["command", "reply", "data", "effects"] {
        let (_directory, home, service) = home_with_example("two-step");
        let work = work_id_of(&start_two_step(&service));
        begin_outline(&service, &work);
        let connection = Connection::open(home.store_path().as_str()).unwrap();
        let (table, column) = match field {
            "command" => ("audit", "command_json"),
            "reply" | "data" => ("requests", "reply_json"),
            "effects" => ("requests", "effects_json"),
            _ => unreachable!(),
        };
        let raw: String = connection
            .query_row(
                &format!("SELECT {column} FROM {table} WHERE request_id='read-begin'"),
                [],
                |row| row.get(0),
            )
            .unwrap();
        let mut payload: Value = serde_json::from_str(&raw).unwrap();
        match field {
            "command" => payload["unexpected_contract_field"] = json!(true),
            "reply" => payload["reply"]["unexpected_contract_field"] = json!(true),
            "data" => payload["data"]["unexpected_contract_field"] = json!(true),
            "effects" => payload[0]["unexpected_contract_field"] = json!(true),
            _ => unreachable!(),
        }
        connection
            .execute(
                &format!("UPDATE {table} SET {column}=?1 WHERE request_id='read-begin'"),
                [payload.to_string()],
            )
            .unwrap();
        let frozen = home.work_dir(&work).join_segment("workbook");
        let retained = home.work_dir(&work).join_segment("retained-workbook");
        retain_frozen_directory(
            frozen.as_path().as_std_path(),
            retained.as_path().as_std_path(),
        );
        let before = store_rows(&connection);
        for error in [
            service.status_read(&work).unwrap_err(),
            service.result(&work).unwrap_err(),
        ] {
            assert_eq!(error.code(), ErrorCode::StoreCorrupt, "{field}: {error}");
            assert!(
                error.to_string().contains("unexpected_contract_field"),
                "Raw payload errors must precede frozen I/O: {field}: {error}"
            );
        }
        assert_eq!(store_rows(&connection), before);
        assert!(!frozen.as_path().exists());
        assert!(retained.as_path().join("workbook.toml").exists());
    }
}

fn submit_result(service: &WorkService, work: &sheltie_core::ids::WorkId) -> String {
    let begun = service
        .begin(work, &NodeId::new("deliver").unwrap(), None)
        .unwrap();
    write_output(&output_dir_of(&begun), "final.bin", "final bytes\n");
    service
        .submit(
            work,
            &AttemptId::parse("deliver#1.0").unwrap(),
            &lit("execution complete"),
            Some("result-submit".into()),
        )
        .unwrap()
        .request_id
}

// Task: C004-T02
#[test]
fn result_selects_terminal_bound_input_and_sealed_output_in_key_order() {
    let (_dir, home, service, work) = result_fixture(false, true);
    assert!(!service.result(&work).unwrap().0.r#final);
    submit_result(&service, &work);
    let connection = Connection::open(home.store_path().as_str()).unwrap();
    let before = store_rows(&connection);
    let (view, next) = service.result(&work).unwrap();
    assert!(view.r#final && !view.effects_pending);
    assert_eq!(view.revision, 3);
    assert_eq!(
        view.artifacts
            .iter()
            .map(|item| item.key.as_str())
            .collect::<Vec<_>>(),
        ["archive", "origin"]
    );
    assert_eq!(view.artifacts[0].bytes, 12);
    assert_eq!(view.artifacts[1].bytes, 15);
    assert_eq!(
        view.artifacts[0].sha256.as_str(),
        "e8ef8eb5c16fac6019371fc3bf24f1f4d94d7488dad6dd2633067cb7233b3a0c"
    );
    assert_eq!(
        view.artifacts[1].sha256.as_str(),
        "a6e3f8ce6c03c8c3718f4214b98bdb8826428b3b29df8743f3e943b2a680fdba"
    );
    assert_eq!(view.artifacts[0].source.attempt, "deliver#1.0");
    assert_eq!(view.artifacts[1].source.attempt, "deliver#1.0");
    assert_eq!(
        serde_json::to_value(&view).unwrap()["artifacts"][1]["source"]["kind"],
        "input"
    );
    assert!(
        view.artifacts[1]
            .path
            .as_str()
            .ends_with("start-inputs/input")
    );
    assert!(next.is_empty());
    assert_eq!(store_rows(&connection), before);
}

// Task: C004-T02
#[test]
fn result_hides_artifacts_until_gate_and_file_effects_are_complete() {
    let (_dir, home, service, work) = result_fixture(true, true);
    let submitted = submit_result(&service, &work);
    let (blocked, _) = service.result(&work).unwrap();
    assert!(!blocked.r#final && blocked.artifacts.is_empty());
    service
        .approve(&work, &NodeId::new("deliver").unwrap(), None)
        .unwrap();
    assert!(service.result(&work).unwrap().0.r#final);
    let connection = Connection::open(home.store_path().as_str()).unwrap();
    connection
        .execute(
            "UPDATE requests SET published=0 WHERE request_id=?1",
            [submitted],
        )
        .unwrap();
    let before = store_rows(&connection);
    let (pending, _) = service.result(&work).unwrap();
    assert!(pending.effects_pending && !pending.r#final && pending.artifacts.is_empty());
    assert_eq!(store_rows(&connection), before);
}

// Task: C004-T02
#[test]
fn result_distinguishes_cancelled_work_and_success_without_explicit_selection() {
    let (_dir, _home, service, work) = result_fixture(false, true);
    service.cancel(&work, None).unwrap();
    let cancelled = service.result(&work).unwrap().0;
    assert!(!cancelled.r#final && cancelled.artifacts.is_empty());
    let (_dir, _home, service, work) = result_fixture(false, false);
    submit_result(&service, &work);
    let empty = service.result(&work).unwrap().0;
    assert!(empty.r#final && empty.artifacts.is_empty());
}
