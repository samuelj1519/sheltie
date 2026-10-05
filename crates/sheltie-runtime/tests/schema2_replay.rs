//! C002-T07: schema 2, request intents, snapshot replay, and effect recovery rejection cases (O02/O04/O05/O08/N03).
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;

use common::*;
use sheltie_core::error::ErrorCode;
use sheltie_core::ids::{AttemptId, NodeId};
use sheltie_core::path::AbsPath;
use sheltie_runtime::{Error, Home, StartArgs, WorkService, WorkbookRepo};

/// Every real write entry rejects schema 1 main/WAL before creating .lock or changing persisted bytes (D-033).
// Task: C002-T24
#[test]
fn schema1_store_rejected_without_touching_file() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("store.db");
    // Hand-build a schema 1 database with v0.1.0 tables.
    let conn = rusqlite::Connection::open(&db).unwrap();
    conn.execute_batch(
        "CREATE TABLE workbooks (id TEXT NOT NULL, version TEXT NOT NULL, digest TEXT NOT NULL, dir TEXT NOT NULL, added_at TEXT NOT NULL, PRIMARY KEY (id, version));
         CREATE TABLE works (work_id TEXT PRIMARY KEY, revision INTEGER NOT NULL, status TEXT NOT NULL, state_json TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL);
         CREATE TABLE work_sequence (day TEXT PRIMARY KEY, last INTEGER NOT NULL);
         CREATE TABLE requests (request_id TEXT PRIMARY KEY, work_id TEXT, payload_hash TEXT NOT NULL, reply_json TEXT NOT NULL, at TEXT NOT NULL);
         CREATE TABLE audit (seq INTEGER PRIMARY KEY AUTOINCREMENT, work_id TEXT NOT NULL, revision INTEGER NOT NULL, request_id TEXT NOT NULL, principal TEXT NOT NULL, command_json TEXT NOT NULL, at TEXT NOT NULL);
         PRAGMA user_version = 1;",
    )
    .unwrap();
    drop(conn);
    let writer = rusqlite::Connection::open(&db).unwrap();
    assert!(
        writer
            .set_db_config(
                rusqlite::config::DbConfig::SQLITE_DBCONFIG_NO_CKPT_ON_CLOSE,
                true,
            )
            .unwrap()
    );
    writer
        .execute_batch(
            "PRAGMA journal_mode = WAL;
             CREATE TABLE user_records(value TEXT);
             INSERT INTO user_records VALUES ('keep me');
             PRAGMA user_version = 1;",
        )
        .unwrap();
    drop(writer);
    let before = std::fs::read(&db).unwrap();
    let wal = dir.path().join("store.db-wal");
    let wal_before = std::fs::read(&wal).unwrap();

    let home = Home::resolve(Some(
        (AbsPath::new(dir.path().to_str().unwrap()).unwrap()).as_str(),
    ))
    .unwrap();
    assert!(matches!(
        WorkService::new(home.clone()).list(),
        Err(Error::StoreSchemaMismatch { .. })
    ));
    assert!(matches!(
        WorkbookRepo::new(home.clone()).add(&abs(&example_dir("two-step")), None),
        Err(Error::StoreSchemaMismatch { .. })
    ));
    assert!(matches!(
        sheltie_runtime::selfmgmt::install(&home),
        Err(Error::StoreSchemaMismatch { .. })
    ));
    assert!(
        !home.lock_path().as_path().exists(),
        "Old-schema rejection must not create .lock"
    );
    assert_eq!(
        std::fs::read(&db).unwrap(),
        before,
        "Rejection must not rewrite main"
    );
    assert_eq!(
        std::fs::read(&wal).unwrap(),
        wal_before,
        "Rejection must not rewrite WAL"
    );
}

/// Cross-Work request-id: cancel A then cancel B with the same ID yields REQUEST_CONFLICT, leaving B unchanged (O02).
// Task: C002-T07
#[test]
fn cross_work_request_id_is_request_conflict_and_target_untouched() {
    let (_d, _home, svc) = home_with_example("two-step");
    let a = work_id_of(&svc.start(two_step_args(&[("topic", "t")]), None).unwrap());
    let args2 = StartArgs {
        name: Some("second".into()),
        ..two_step_args(&[("topic", "t")])
    };
    let b = work_id_of(&svc.start(args2, None).unwrap());

    svc.cancel(&a, Some("shared-r".into())).unwrap();
    let err = svc.cancel(&b, Some("shared-r".into())).unwrap_err();
    assert_eq!(err.code(), ErrorCode::RequestConflict, "{err:?}");
    // B remains active; same-ID different-target intent creates no effects.
    let (_, card) = svc.status(&b).unwrap();
    assert_eq!(card.status, sheltie_core::work::WorkStatus::Active);
}

// Task: C002-T22
#[test]
fn completed_submit_replay_does_not_seal_or_rewrite_the_output_again() {
    let (_d, _home, svc) = home_with_example("two-step");
    let wid = work_id_of(&svc.start(two_step_args(&[("topic", "t")]), None).unwrap());
    let begun = svc
        .begin(&wid, &NodeId::new("outline").unwrap(), None)
        .unwrap();
    let output_dir = output_dir_of(&begun);
    write_output(&output_dir, "outline.md", "committed output");
    let request_id = Some("t22-completed-submit".to_string());
    let first = svc
        .submit(
            &wid,
            &AttemptId::parse("outline#1.0").unwrap(),
            &lit("done"),
            request_id.clone(),
        )
        .unwrap();
    let output = Path::new(output_dir.as_str()).join("outline.md");
    let mut permissions = std::fs::metadata(&output).unwrap().permissions();
    permissions.set_mode(0o644);
    std::fs::set_permissions(&output, permissions).unwrap();
    std::fs::write(&output, "later bytes").unwrap();
    let mode = std::fs::metadata(&output).unwrap().permissions().mode();

    let replay = svc
        .submit(
            &wid,
            &AttemptId::parse("outline#1.0").unwrap(),
            &lit("done"),
            request_id,
        )
        .unwrap();
    assert!(replay.replayed);
    assert_eq!(replay.revision, first.revision);
    assert_eq!(replay.reply, first.reply);
    assert_eq!(replay.data, first.data);
    assert_eq!(std::fs::read(&output).unwrap(), b"later bytes");
    assert_eq!(
        std::fs::metadata(output).unwrap().permissions().mode(),
        mode
    );
}

/// Replay start after Workbook deletion without rereading the repository, returning the original snapshot (O04).
// Task: C002-T07
#[test]
fn start_replay_after_workbook_removed_returns_original_snapshot() {
    let (_d, home, svc) = home_with_example("two-step");
    let args = two_step_args(&[("topic", "t")]);
    let wid = work_id_of(&svc.start(args.clone(), Some("r-start".into())).unwrap());
    svc.cancel(&wid, None).unwrap();
    WorkbookRepo::new(home.clone())
        .remove("two-step", "1.0.1", None)
        .unwrap();

    let again = svc.start(args, Some("r-start".into())).unwrap();
    assert!(again.replayed);
    assert_eq!(
        match &again.reply {
            sheltie_core::work::Reply::Started { work_id, .. } => work_id.clone(),
            other => panic!("{other:?}"),
        },
        wid
    );
}

/// Replay old submit after cancellation, returning the original snapshot/historical next without cancelled state (O04).
// Task: C002-T07
#[test]
fn old_submit_replay_after_cancel_returns_original_reply() {
    let (_d, _home, svc) = home_with_example("two-step");
    let wid = work_id_of(&svc.start(two_step_args(&[("topic", "t")]), None).unwrap());
    let begun = svc
        .begin(&wid, &NodeId::new("outline").unwrap(), None)
        .unwrap();
    let output_dir = match &begun.reply {
        sheltie_core::work::Reply::AttemptBegun { output_dir, .. } => output_dir.clone(),
        other => panic!("{other:?}"),
    };
    write_output(&output_dir, "outline.md", "Content");
    let first = svc
        .submit(
            &wid,
            &AttemptId::parse("outline#1.0").unwrap(),
            &lit("Completed"),
            Some("r-old".into()),
        )
        .unwrap();
    svc.cancel(&wid, None).unwrap();

    let again = svc
        .submit(
            &wid,
            &AttemptId::parse("outline#1.0").unwrap(),
            &lit("Completed"),
            Some("r-old".into()),
        )
        .unwrap();
    assert!(again.replayed);
    // Historical next retains commit-time begin(summary), without mixing in cancelled state.
    assert_eq!(again.next, first.next);
    assert!(!again.next.is_empty(), "Preserve historical next");
    // Current status queries remain authoritative for current state.
    let (_, card) = svc.status(&wid).unwrap();
    assert_eq!(card.status, sheltie_core::work::WorkStatus::Cancelled);
}

/// Status card is a current projection; old-request replay must not restore old card versions (§6).
// Task: C002-T07
#[test]
fn replay_does_not_rewrite_status_card_to_old_revision() {
    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&svc.start(two_step_args(&[("topic", "t")]), None).unwrap());
    let card_path = Path::new(home.work_dir(&wid).as_str()).join("status-card.md");

    let begun = svc
        .begin(&wid, &NodeId::new("outline").unwrap(), Some("r-b".into()))
        .unwrap();
    let at_begin = std::fs::read_to_string(&card_path).unwrap();

    let output_dir = match &begun.reply {
        sheltie_core::work::Reply::AttemptBegun { output_dir, .. } => output_dir.clone(),
        other => panic!("{other:?}"),
    };
    write_output(&output_dir, "outline.md", "Content");
    svc.submit(
        &wid,
        &AttemptId::parse("outline#1.0").unwrap(),
        &lit("Completed"),
        None,
    )
    .unwrap();
    let at_submit = std::fs::read_to_string(&card_path).unwrap();
    assert_ne!(at_begin, at_submit, "Submission updated the card");

    // Old begin replay must not regress the card to its begin-time version.
    svc.begin(&wid, &NodeId::new("outline").unwrap(), Some("r-b".into()))
        .unwrap();
    assert_eq!(
        std::fs::read_to_string(&card_path).unwrap(),
        at_submit,
        "Old-request replay must not rewrite old status cards"
    );
}

/// Historical-file digest mismatch yields STORE_CORRUPT on replay, without hiding changes (§3.2 write_file).
// Task: C002-T07
#[test]
fn replay_with_modified_brief_reports_integrity_error() {
    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&svc.start(two_step_args(&[("topic", "t")]), None).unwrap());
    let begun = svc
        .begin(&wid, &NodeId::new("outline").unwrap(), Some("r-m".into()))
        .unwrap();
    let brief = match &begun.reply {
        sheltie_core::work::Reply::AttemptBegun { brief_path, .. } => {
            std::path::PathBuf::from(brief_path.as_str())
        }
        other => panic!("{other:?}"),
    };
    let _ = home;
    let mut m = std::fs::metadata(&brief).unwrap().permissions();
    m.set_mode(0o644);
    std::fs::set_permissions(&brief, m).unwrap();
    std::fs::write(&brief, "Modified brief").unwrap();

    let err = svc
        .begin(&wid, &NodeId::new("outline").unwrap(), Some("r-m".into()))
        .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("differs") || msg.contains("digest"), "{msg}");
}

/// Workbook add/remove replay returns original snapshots for the same ID without reinstalling changed sources (O08/§2.1).
// Task: C002-T07
#[test]
fn workbook_add_replay_ignores_source_changes() {
    let dir = tempfile::tempdir().unwrap();
    let src = copy_example("two-step", dir.path());
    let (_d, home) = temp_home();
    let repo = WorkbookRepo::new(home.clone());

    let first = repo.add(&abs(&src), Some("r-add".into())).unwrap();
    assert!(!first.replayed);
    let digest1 = first.data["digest"].as_str().unwrap().to_string();

    // Source content changed; replay must not reread it.
    std::fs::write(src.join("workbook.toml"), "Revised").unwrap();
    let again = repo.add(&abs(&src), Some("r-add".into())).unwrap();
    assert!(again.replayed);
    assert_eq!(again.data["digest"].as_str().unwrap(), digest1);

    // New request-id means new install intent, using new-copy bytes; reject here
    // because the manifest is invalid, without silently reusing old content.
    let err = repo.add(&abs(&src), None).unwrap_err();
    assert_eq!(err.code(), ErrorCode::WorkbookInvalid, "{err:?}");
}

/// remove replay returns the original snapshot without repeated deletion; re-add the same version in a new lifecycle.
// Task: C002-T07
#[test]
fn workbook_remove_replay_and_readd() {
    let (_d, home) = temp_home();
    let repo = WorkbookRepo::new(home.clone());
    repo.add(&abs(&example_dir("two-step")), Some("r-a".into()))
        .unwrap();

    let first = repo
        .remove("two-step", "1.0.1", Some("r-r".into()))
        .unwrap();
    assert!(!first.replayed);
    let again = repo
        .remove("two-step", "1.0.1", Some("r-r".into()))
        .unwrap();
    assert!(again.replayed, "Replay must not repeat deletion");
    assert!(!home.workbook_dir("two-step", "1.0.1").as_path().exists());

    // New lifecycle: same-version add succeeds again.
    let re = repo
        .add(&abs(&example_dir("two-step")), Some("r-a2".into()))
        .unwrap();
    assert!(!re.replayed);
    assert!(home.workbook_dir("two-step", "1.0.1").as_path().exists());
}

/// Concurrent writers serialize through the root lock; both succeed, revisions increase, no cross-corruption (§2.2).
// Task: C002-T07
#[test]
fn two_writers_serialize_under_home_lock() {
    let (_d, _home, svc) = home_with_example("two-step");
    let wid = work_id_of(&svc.start(two_step_args(&[("topic", "t")]), None).unwrap());
    let begun = svc
        .begin(&wid, &NodeId::new("outline").unwrap(), None)
        .unwrap();
    let output_dir = match &begun.reply {
        sheltie_core::work::Reply::AttemptBegun { output_dir, .. } => output_dir.clone(),
        other => panic!("{other:?}"),
    };
    write_output(&output_dir, "outline.md", "Content");

    // Concurrent cancel/submit: cancel terminates Work; submit either succeeds first or follows
    //with WorkTerminal (not RequestConflict). Neither outcome may leave partial state.
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(3));
    let a = {
        let svc = svc.clone();
        let wid = wid.clone();
        let barrier = barrier.clone();
        std::thread::spawn(move || {
            barrier.wait();
            svc.cancel(&wid, None)
        })
    };
    let b = {
        let svc = svc.clone();
        let wid = wid.clone();
        let barrier = barrier.clone();
        std::thread::spawn(move || {
            barrier.wait();
            svc.submit(
                &wid,
                &AttemptId::parse("outline#1.0").unwrap(),
                &lit("Concurrent"),
                None,
            )
        })
    };
    barrier.wait();
    a.join().unwrap().unwrap();
    let submitted = b.join().unwrap();
    assert!(
        submitted.is_ok() || submitted.unwrap_err().code() == sheltie_core::ErrorCode::WorkTerminal
    );

    // Consistent terminal state: cancellation takes effect and status card matches storage.
    let (_, card) = svc.status(&wid).unwrap();
    assert_eq!(card.status, sheltie_core::work::WorkStatus::Cancelled);
}

/// begin crash window after COMMIT/before effects; the next write recovers before its own command (§3.1).
// Task: C002-T40
#[test]
fn begin_effects_recovered_by_next_write() {
    let (_d, home, svc) = home_with_example("two-step");
    let wid = work_id_of(&svc.start(two_step_args(&[("topic", "t")]), None).unwrap());
    let begun = svc
        .begin(&wid, &NodeId::new("outline").unwrap(), None)
        .unwrap();
    let brief = match &begun.reply {
        sheltie_core::work::Reply::AttemptBegun { brief_path, .. } => {
            std::path::PathBuf::from(brief_path.as_str())
        }
        other => panic!("{other:?}"),
    };
    // Simulate termination after COMMIT/before publication: committed request, absent effect files.
    let original_brief = std::fs::read(&brief).unwrap();
    std::fs::remove_file(&brief).unwrap();
    let conn = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    conn.execute("UPDATE requests SET published = 0", [])
        .unwrap();
    drop(conn);

    // The next write restores the brief before its own operation.
    svc.fail(
        &wid,
        &AttemptId::parse("outline#1.0").unwrap(),
        &lit("Stop working"),
        None,
    )
    .unwrap();
    assert_eq!(
        std::fs::read(&brief).unwrap(),
        original_brief,
        "Recovery restores the brief's original commit-time bytes"
    );
    let connection = rusqlite::Connection::open(home.store_path().as_str()).unwrap();
    let pending: i64 = connection
        .query_row(
            "SELECT count(*) FROM requests WHERE published = 0",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(pending, 0);
}
