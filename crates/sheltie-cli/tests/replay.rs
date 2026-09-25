//! T23：请求重放（cli 层）。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::*;

// Task: T23
#[test]
#[ignore = "T23"]
fn same_request_id_same_payload_returns_replayed_true() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "x")]);
    let a = env.ok(&[
        "--request-id",
        "r-1",
        "attempt",
        "begin",
        &wid,
        "--node",
        "outline",
    ]);
    let b = env.ok(&[
        "--request-id",
        "r-1",
        "attempt",
        "begin",
        &wid,
        "--node",
        "outline",
    ]);
    assert_eq!(a["data"]["attempt"], b["data"]["attempt"]);
    assert_eq!(b["data"]["replayed"], true);
    assert_eq!(a["revision"], b["revision"]);
}

// Task: T23
#[test]
#[ignore = "T23"]
fn same_request_id_different_payload_is_request_conflict() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "x")]);
    env.ok(&[
        "--request-id",
        "r-1",
        "attempt",
        "begin",
        &wid,
        "--node",
        "outline",
    ]);
    let (e, _) = env.fail(&[
        "--request-id",
        "r-1",
        "attempt",
        "fail",
        &wid,
        "--attempt",
        "outline#1.0",
        "--reason",
        "x",
    ]);
    assert_eq!(e["error"]["code"], "REQUEST_CONFLICT");
}
