//! T19: attempt/gate commands, completing the two-step example through CLI.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::path::Path;

use common::*;

// Task: C009-T02
#[test]
fn two_step_via_cli_reaches_succeeded() {
    let env = Env::new();
    env.add_example("two-step");
    let started = env.ok(&[
        "work",
        "start",
        "--workbook",
        "two-step",
        "--flow",
        "default",
        "--input",
        "topic=x",
    ]);
    let wid = started["data"]["work_id"].as_str().unwrap().to_string();
    // Each step uses only the previous response's next actions.
    let b1 = env.follow_begin(&started, "outline");
    let brief = Path::new(b1["data"]["brief_path"].as_str().unwrap());
    assert!(brief.exists());
    assert!(
        std::fs::read_to_string(brief)
            .unwrap()
            .starts_with("# Brief: Outline")
    );
    assert_eq!(b1["data"]["attempt"], "outline#1.0");
    let s1 = env.submit_all(&wid, &b1, "Outline ready");
    let b2 = env.follow_begin(&s1, "summary");
    let s2 = env.submit_all(&wid, &b2, "Summary ready");
    assert_eq!(s2["data"]["work_status"]["kind"], "succeeded");
    assert!(s2["next"].as_array().unwrap().is_empty());
}

// Task: C002-T25
#[test]
fn effect_pending_cli_json_contains_full_original_work_success_envelope() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "x")]);
    let card = env
        .dir
        .path()
        .join("works")
        .join(&wid)
        .join("status-card.md");
    std::fs::remove_file(&card).unwrap();
    std::fs::create_dir(&card).unwrap();

    let out = env
        .cmd(&[
            "--request-id",
            "t25-cli-card-failure",
            "attempt",
            "begin",
            &wid,
            "--node",
            "outline",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let response: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(response["ok"], false);
    assert_eq!(response["error"]["code"], "EFFECT_PENDING");
    assert_eq!(response["committed"], true);
    assert_eq!(response["request_id"], "t25-cli-card-failure");
    assert_eq!(response["revision"], 2);
    let original = &response["original"];
    assert_eq!(original["ok"], true);
    assert_eq!(original["request_id"], "t25-cli-card-failure");
    assert_eq!(original["revision"], 2);
    assert_eq!(original["data"]["replayed"], false);
    assert_eq!(original["next"][0]["op"], "attempt submit");
    assert_eq!(original["next"][0]["args"]["work"], wid);
    assert_eq!(original["next"][0]["args"]["attempt"], "outline#1.0");
    assert_eq!(response["error"]["detail"]["cause"], "IO");
}

// Task: T19
#[test]
fn attempt_fail_then_begin_retries_same_occurrence() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "x")]);
    env.begin(&wid, "outline");
    let f = env.ok(&[
        "attempt",
        "fail",
        &wid,
        "--attempt",
        "outline#1.0",
        "--reason",
        "Timed out",
    ]);
    assert_eq!(next_begin_nodes(&f), vec!["outline"]);
    let b = env.begin(&wid, "outline");
    assert_eq!(b["data"]["attempt"], "outline#1.1");
}
