//! C002-T02：`work start` 的无副作用预检（GF-30）。
//! 确定性拒绝发生在当日序号分配与任何目录物化之前；拒绝后补齐条件即成功，不烧号。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::*;
use sheltie_core::error::ErrorCode;
use sheltie_runtime::{Error, Home, StartArgs};

/// 独立 oracle：绕过 runtime 直接查 SQLite，读 `works`、`work_sequence`、`requests` 行数。
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

fn start_args(inputs: &[(&str, &str)]) -> StartArgs {
    StartArgs {
        workbook_id: "two-step".into(),
        version: None,
        flow: "default".into(),
        name: None,
        inputs: inputs
            .iter()
            .map(|(k, v)| {
                (
                    k.to_string(),
                    sheltie_runtime::request::InputValue::Literal {
                        text: v.to_string(),
                    },
                )
            })
            .collect(),
    }
}

/// 断言一次失败之后：三张表与调用前相同、`works/` 下没有目录、报错码正确。
fn assert_unchanged(home: &Home, before: (i64, i64, i64), err: &Error, code: ErrorCode) {
    assert_eq!(err.code(), code, "{err:?}");
    assert_eq!(
        db_counts(home),
        before,
        "失败后 works/sequence/requests 不得有变化"
    );
    let works = home.works_dir();
    let entries = std::fs::read_dir(works.as_path())
        .map(|it| it.count())
        .unwrap_or(0);
    assert_eq!(entries, 0, "失败的 start 不得物化任何 Work 目录");
}

// Task: C002-T02
#[test]
fn missing_start_input_rejected_before_seq_and_materialization() {
    let (_d, home, svc) = home_with_example("two-step");
    let before = db_counts(&home);
    let err = svc.start(start_args(&[]), None).unwrap_err();
    assert_unchanged(&home, before, &err, ErrorCode::InputMissing);

    // 拒绝后只补缺条件、用同一个 request-id 即成功；序号未被烧掉。
    let resp = svc
        .start(
            start_args(&[("topic", "t")]),
            Some("11111111-1111-1111-1111-111111111111".into()),
        )
        .unwrap();
    assert!(
        matches!(&resp.reply, sheltie_core::work::Reply::Started { work_id, .. }
        if work_id.as_str().contains("-001-"))
    );
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
    assert!(
        matches!(&resp.reply, sheltie_core::work::Reply::Started { work_id, .. }
        if work_id.as_str().contains("-001-"))
    );
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
    assert!(
        matches!(&resp.reply, sheltie_core::work::Reply::Started { work_id, .. }
        if work_id.as_str().contains("-001-"))
    );
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
