//! C002-T02: side-effect-free work start preflight (GF-30).
//! Reject deterministically before sequence allocation/materialization; fixing only the missing condition succeeds without consuming sequence.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::two_step_args as start_args;
use common::*;
use sheltie_core::error::ErrorCode;
use sheltie_runtime::{Error, Home};

/// Independent oracle: query SQLite directly for works/work_sequence/requests row counts.
fn db_counts(home: &Home) -> (i64, i64, i64) {
    let conn = rusqlite::Connection::open_with_flags(
        home.store_path().as_str(),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    let count = |table: &str| -> i64 {
        conn.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
            .unwrap()
    };
    (count("works"), count("work_sequence"), count("requests"))
}

/// After failure, assert unchanged tables, no works/ directories, and the correct error code.
fn assert_unchanged(home: &Home, before: (i64, i64, i64), err: &Error, code: ErrorCode) {
    assert_eq!(err.code(), code, "{err:?}");
    assert_eq!(
        db_counts(home),
        before,
        "Failure must not change works/sequence/requests"
    );
    let works = home.works_dir();
    let entries = std::fs::read_dir(works.as_path())
        .map(|it| it.count())
        .unwrap_or(0);
    assert_eq!(
        entries, 0,
        "Failed start must not materialize Work directories"
    );
}

fn assert_first_sequence(response: &sheltie_runtime::Response) {
    assert!(
        matches!(&response.reply, sheltie_core::work::Reply::Started { work_id, .. }
        if work_id.as_str().contains("-001-"))
    );
}

// Task: C002-T02
#[test]
fn missing_start_input_rejected_before_seq_and_materialization() {
    let (_d, home, svc) = home_with_example("two-step");
    let before = db_counts(&home);
    let err = svc.start(start_args(&[]), None).unwrap_err();
    assert_unchanged(&home, before, &err, ErrorCode::InputMissing);

    // Fix only the missing condition and reuse request-id; succeed without consuming sequence.
    let resp = svc
        .start(
            start_args(&[("topic", "t")]),
            Some("11111111-1111-1111-1111-111111111111".into()),
        )
        .unwrap();
    assert_first_sequence(&resp);
}

// Task: C002-T02
#[test]
fn extra_start_input_rejected_before_seq() {
    let (_d, home, svc) = home_with_example("two-step");
    let before = db_counts(&home);
    let err = svc
        .start(start_args(&[("topic", "t"), ("bonus", "x")]), None)
        .unwrap_err();
    assert_unchanged(&home, before, &err, ErrorCode::InputMissing);

    let resp = svc.start(start_args(&[("topic", "t")]), None).unwrap();
    assert_first_sequence(&resp);
}

// Task: C002-T02
#[test]
fn invalid_name_rejected_before_seq() {
    let (_d, home, svc) = home_with_example("two-step");
    let before = db_counts(&home);
    let mut args = start_args(&[("topic", "t")]);
    args.name = Some("a/b".into());
    let err = svc.start(args, None).unwrap_err();
    assert_unchanged(&home, before, &err, ErrorCode::InvalidRequest);

    let mut args = start_args(&[("topic", "t")]);
    args.name = Some("合法 名字".into());
    let resp = svc.start(args, None).unwrap();
    assert!(
        matches!(&resp.reply, sheltie_core::work::Reply::Started { work_id, .. }
        if work_id.as_str().ends_with("-001-合法-名字"))
    );
}

// Task: C002-T02
#[test]
fn missing_flow_rejected_before_seq() {
    let (_d, home, svc) = home_with_example("two-step");
    let before = db_counts(&home);
    let mut args = start_args(&[("topic", "t")]);
    args.flow = "ghost".into();
    let err = svc.start(args, None).unwrap_err();
    assert_unchanged(&home, before, &err, ErrorCode::NotFound);

    let resp = svc.start(start_args(&[("topic", "t")]), None).unwrap();
    assert_first_sequence(&resp);
}

// Task: C002-T02
#[test]
fn missing_workbook_rejected_without_side_effects() {
    let (_d, home, svc) = home_with_example("two-step");
    let before = db_counts(&home);
    let mut args = start_args(&[("topic", "t")]);
    args.workbook_id = "ghost".into();
    let err = svc.start(args, None).unwrap_err();
    assert_unchanged(&home, before, &err, ErrorCode::NotFound);
}
