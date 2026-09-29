//! T19：`attempt` 与 `gate` 组；两步样例从 CLI 走完。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::path::Path;

use common::*;

// Task: T19
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
    // 每一步只从上一步响应的 next 里取命令。
    let b1 = env.follow_begin(&started, "outline");
    let s1 = env.submit_all(&wid, &b1, "提纲好了");
    let b2 = env.follow_begin(&s1, "summary");
    let s2 = env.submit_all(&wid, &b2, "摘要好了");
    assert_eq!(s2["data"]["work_status"]["kind"], "succeeded");
    assert!(s2["next"].as_array().unwrap().is_empty());
}

// Task: T19
#[test]
fn attempt_begin_returns_brief_path_that_exists() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "x")]);
    let b = env.begin(&wid, "outline");
    let brief = Path::new(b["data"]["brief_path"].as_str().unwrap());
    assert!(brief.exists());
    let text = std::fs::read_to_string(brief).unwrap();
    assert!(text.starts_with("# 任务书：列提纲"));
    assert_eq!(b["data"]["attempt"], "outline#1.0");
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
fn attempt_submit_summary_from_at_file() {
    let env = Env::new();
    env.add_example("two-step");
    let wid = env.start("two-step", &[("topic", "x")]);
    let b = env.begin(&wid, "outline");
    let out_dir = Path::new(b["data"]["output_dir"].as_str().unwrap());
    std::fs::write(out_dir.join("outline.md"), "提纲").unwrap();
    let f = env.dir.path().join("summary.txt");
    std::fs::write(&f, "来自文件的摘要").unwrap();
    env.ok(&[
        "attempt",
        "submit",
        &wid,
        "--attempt",
        "outline#1.0",
        "--summary",
        &format!("@{}", f.display()),
    ]);
    let st = env.status(&wid);
    assert_eq!(st["data"]["last_attempt"]["summary"], "来自文件的摘要");
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
        "超时",
    ]);
    assert_eq!(next_begin_nodes(&f), vec!["outline"]);
    let b = env.begin(&wid, "outline");
    assert_eq!(b["data"]["attempt"], "outline#1.1");
}
